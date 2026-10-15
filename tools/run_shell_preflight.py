#!/usr/bin/env python3
"""Unscored shell integration with a real WASI guest, ChoreoFS, Luna, and the original simulator."""

import argparse
import asyncio
import dataclasses
import hashlib
import importlib
import json
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from src.imp.luna import run_participant, tool_spec
from benchmark.tools import load_tools
import jsonschema


from run_preflight import Bridge
from src.imp.shell import Shell


async def run(task, run_id, scenario):
    config = json.loads((ROOT / "experiment.json").read_text())
    metadata_path = ROOT / "vendor/tracefix/benchmark/descriptions" / task / "metadata.json"
    metadata = json.loads(metadata_path.read_text())
    module = importlib.import_module(f"benchmark.environments.{task}.sim")
    constructors = [value for value in vars(module).values()
                    if isinstance(value, type) and value.__module__ == module.__name__
                    and value.__name__.endswith("Sim")]
    if len(constructors) != 1:
        raise RuntimeError("could not identify the original task simulator")
    sim = constructors[0]()
    sim._seed = 0
    sim._delay_multiplier = config["domain_tool_delay_multiplier"]
    sim.set_scenario_depth(scenario)
    registry = load_tools(task, sim=sim)
    output = ROOT / "output" / "shell-preflight" / run_id / task
    output.mkdir(parents=True, exist_ok=False)
    source_files = [path for path in (ROOT / "src").rglob("*")
                    if path.is_file() and "__pycache__" not in path.parts]
    source_files += [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "experiment.json",
                     Path(__file__).resolve(), ROOT / "tools/run_preflight.py", ROOT / "tools/build_shell.sh", ROOT / "requirements.lock"]
    source_files += [path for path in (ROOT / "guests/shell").rglob("*") if path.is_file() and "target" not in path.parts]
    source_hashes = {}
    for path in sorted(source_files):
        relative = path.relative_to(ROOT)
        content = path.read_bytes()
        source_hashes[str(relative)] = hashlib.sha256(content).hexdigest()
        snapshot = output / "source" / relative
        snapshot.parent.mkdir(parents=True, exist_ok=True)
        snapshot.write_bytes(content)
    (output / "source-manifest.json").write_text(json.dumps(source_hashes, indent=2)+"\n")
    binary = ROOT / "target/debug" / ("codebase" if task.startswith("1") else "philosophers")
    binary_content = binary.read_bytes()
    runtime_binary_sha256 = hashlib.sha256(binary_content).hexdigest()
    runtime_snapshot = output / "runtime"
    runtime_snapshot.write_bytes(binary_content)
    runtime_snapshot.chmod(0o700)
    wasi_snapshot = output / "wasi-runtime"
    guest_snapshot = output / "shell.wasm"
    wasi_snapshot.write_bytes((ROOT / "target/debug/wasi-shell").read_bytes())
    wasi_snapshot.chmod(0o700)
    guest_snapshot.write_bytes((ROOT / "guests/shell/target/wasm32-wasip1/release/tracefix-shell-guest.wasm").read_bytes())
    wasi_processes = {}
    process = await asyncio.create_subprocess_exec(
        str(runtime_snapshot), task,
        stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    bridge = Bridge(process, registry, output)
    reader = asyncio.create_task(bridge.read())
    started = time.monotonic()

    async def participant(agent):
        schemas = [entry["function"] for entry in registry.openai_schemas(agent)]
        by_name = {schema["name"]: schema for schema in schemas}

        shell = await Shell.start(wasi_snapshot, guest_snapshot, output / (agent + "-shell"))
        async def effect(request):
            try:
                name, arguments = request["tool"], request["arguments"]
                jsonschema.validate(arguments, by_name[name]["parameters"])
            except (KeyError, jsonschema.ValidationError) as failure:
                return {"status":"invalid_tool_arguments", "error":str(failure)}
            return await bridge.call(agent, name, arguments)

        async def dispatch(name, arguments):
            if name != "shell":
                raise RuntimeError("only the guest shell is exposed")
            return await shell.command(arguments["command"], effect)

        task_description = (metadata_path.parent / "description.md").read_text()
        inputs = {"participant":agent, "resources":metadata["agent_resources"][agent]}
        if task.startswith("1"):
            inputs["feature_name"] = {step.agent_id:step.feature for step in module._DESIGN_STEPS}[agent]
        prompt = f"""You are {agent}. Complete your work in the task below with actual tool effects.
{task_description}

Your assigned input data: {json.dumps(inputs)}
Available domain tool schemas: {json.dumps(schemas)}

You have one tool, shell. It sends text to a real bounded WASI shell. All file
operations use ChoreoFS, with no host filesystem or native shell access.
Commands: help, ls /work, cat PATH, echo TEXT > PATH, apply SOURCE TARGET,
script PATH, and call {{"tool":"NAME","arguments":{{...}}}}.
Work files /work/0 through /work/7 can hold notes or scripts. Echo decodes a
literal backslash-n into a newline. A script executes its lines as commands.
Choose your own commands and work files. No command recipe is supplied.
Domain effects still go through the original simulator. Hibana endpoints
coordinate resource loans and constrain task dependencies. A call may wait for
shared resources. Use actual results, handle task-required retries, and finish
only after your assigned work is complete. A private file cannot grant rights.
"""
        try:
            result = await run_participant(
                participant=agent, prompt=prompt, tools=[{
                    "type":"function", "name":"shell", "description":"Run commands in your ChoreoFS WASI guest shell.",
                    "inputSchema":{"type":"object", "properties":{"command":{"type":"string"}}, "required":["command"], "additionalProperties":False},
                }], dispatch=dispatch, output=output / agent,
                timeout=config["run_timeout_seconds"], tool_limit=config["participant_tool_call_limit"],
            )
            await shell.finish()
            return result
        finally:
            await shell.stop()
            wasi_processes[agent] = {"exit_code":shell.process.returncode}
            if not bridge.result.done():
                await bridge.send({"type":"close_role", "id":0, "agent":agent})

    participants = [asyncio.create_task(participant(agent)) for agent in metadata["agents"]]
    outcomes = None
    protocol = None
    error = None
    try:
        async with asyncio.timeout(config["run_timeout_seconds"]):
            outcomes, protocol = await asyncio.gather(asyncio.gather(*participants), bridge.result)
    except Exception as failure:
        error = str(failure) or type(failure).__name__
    finally:
        for task_future in participants:
            if not task_future.done():
                task_future.cancel()
        await asyncio.gather(*participants, return_exceptions=True)
        reader.cancel()
        await asyncio.gather(reader, return_exceptions=True)
        if not process.stdin.is_closing():
            process.stdin.close()
        try:
            await asyncio.wait_for(process.wait(), 5)
        except asyncio.TimeoutError:
            process.terminate()
            await process.wait()
    if outcomes is None:
        outcomes = [future.result() for future in participants
                    if future.done() and not future.cancelled() and future.exception() is None]
    result = {
        "task": task, "scenario_depth":scenario, "kind": "shell_integration_preflight", "scored_benchmark": False,
        "protocol": protocol, "simulator_complete": sim.is_complete(),
        "simulator_progress": sim.progress,
        "simulator_violations": [dataclasses.asdict(value) for value in sim.violations],
        "simulator_events": [dataclasses.asdict(value) for value in sim.events],
        "participants": outcomes, "elapsed_seconds": time.monotonic()-started,
        "error": error, "source_manifest": source_hashes,
        "bridge_events_sha256": hashlib.sha256((output / "bridge-events.jsonl").read_bytes()).hexdigest(),
        "runtime_binary_sha256": runtime_binary_sha256,
        "runtime_exit_code": process.returncode,
        "runtime": "tokio_current_thread",
        "coordination_mode": "wasi_choreofs_and_task_endpoints",
        "wasi_processes":wasi_processes,
        "wasi_runtime_sha256":hashlib.sha256(wasi_snapshot.read_bytes()).hexdigest(),
        "guest_sha256":hashlib.sha256(guest_snapshot.read_bytes()).hexdigest(),
        "shell_events_sha256":{agent:hashlib.sha256((output / (agent + "-shell") / "events.jsonl").read_bytes()).hexdigest() for agent in wasi_processes},
    }
    (output / "result.json").write_text(json.dumps(result, indent=2)+"\n")
    print(json.dumps({"task": task, "protocol": protocol, "error": error, "simulator_complete": sim.is_complete(),
                      "violations": len(sim.violations), "scored_benchmark": False}))


async def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tasks", nargs="+", choices=["1E", "1M", "1H", "9E", "9M", "9H"], required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--scenario", type=int, choices=[0,1,2], default=0)
    args = parser.parse_args()
    for task in args.tasks:
        await run(task, args.run_id, args.scenario)


if __name__ == "__main__":
    asyncio.run(main())
