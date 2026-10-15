#!/usr/bin/env python3
"""Unscored 16E Luna run through wash, projected service requests, and ChoreoFS."""

import argparse
import asyncio
import dataclasses
import hashlib
import importlib
import json
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from benchmark.tools import load_tools
from src.imp.direct_wasi import DirectWasi
from src.imp.luna import run_participant
from inventory import inventory


async def run(run_id, scenario):
    config = json.loads((ROOT / "experiment.json").read_text())
    inputs = ROOT / "vendor/tracefix/benchmark/descriptions/16E"
    metadata = json.loads((inputs / "metadata.json").read_text())
    module = importlib.import_module("benchmark.environments.16E.sim")
    upstream = json.loads((ROOT / "benchmark/upstream-manifest.json").read_text())
    if inventory(ROOT / "vendor/tracefix") != upstream:
        raise RuntimeError("upstream inputs differ from the pinned source manifest")
    sim = module.SimpleCICDSim()
    sim._seed = config["python_hash_seed"]
    sim._delay_multiplier = config["domain_tool_delay_multiplier"]
    sim.set_scenario_depth(scenario)
    registry = load_tools("16E", sim=sim)
    output = ROOT / "output/direct-wasi-preflight" / run_id / "16E"
    output.mkdir(parents=True, exist_ok=False)

    # Freeze source and inputs before starting any participant. These are
    # evidence snapshots, never another owner of protocol progress.
    source_files = [path for base in ("src", "vendor/hibana-wash/src", "vendor/tracefix/benchmark")
                    for path in (ROOT / base).rglob("*")
                    if path.is_file() and "__pycache__" not in path.parts]
    source_files += [ROOT / name for name in (
        "Cargo.toml", "Cargo.lock", "experiment.json", "requirements.lock", "AGENTS.md", "plan.md",
        "tools/run_direct_wasi_preflight.py", "tools/audit_direct_wasi.py",
        "tools/build_shell.sh", "vendor/hibana-wash/Cargo.toml",
        "vendor/hibana-wash/Cargo.lock", "vendor/hibana-wash/scripts/build.sh",
        "benchmark/upstream-manifest.json", "tools/inventory.py",
    )]
    manifest = {}
    for path in sorted(source_files):
        relative = path.relative_to(ROOT)
        content = path.read_bytes()
        manifest[str(relative)] = hashlib.sha256(content).hexdigest()
        saved = output / "source" / relative
        saved.parent.mkdir(parents=True, exist_ok=True)
        saved.write_bytes(content)
    (output / "source-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    versions = {name: subprocess.check_output(
        ["git", "-C", str(ROOT / directory), "rev-parse", "HEAD"], text=True,
    ).strip() for name, directory in (
        ("consumer", "."), ("wash", "vendor/hibana-wash"),
    )}
    # TraceFix is an extracted archive, not a nested Git checkout. A git query
    # there would walk up to the consumer repository and return its revision.
    versions["tracefix"] = upstream["upstream_revision"]
    frozen = {}
    for name, source in (
        ("runtime", ROOT / "target/debug/cicd-wasi"),
        ("wash.wasm", ROOT / "vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm"),
    ):
        content = source.read_bytes()
        (output / name).write_bytes(content)
        frozen[name] = hashlib.sha256(content).hexdigest()
    (output / "runtime").chmod(0o700)

    runtime = None
    participants = []
    protocol = None
    error = None
    started = time.monotonic()

    async def participant(agent):
        schemas = [entry["function"] for entry in registry.openai_schemas(agent)]
        # Input values come from the original task's static steps, not its
        # mutable resource or completion state.
        task_data = {
            "participant": agent,
            "resources": metadata["agent_resources"][agent],
            "build_steps": [dataclasses.asdict(step) for step in module._BUILD_STEPS],
            "test_steps": [dataclasses.asdict(step) for step in module._TEST_STEPS],
            "deploy_steps": [dataclasses.asdict(step) for step in module._DEPLOY_STEPS],
        }
        prompt = f"""You are {agent}. Carry out your work in the original task below.
{(inputs / 'description.md').read_text()}

Original task input data: {json.dumps(task_data)}
Your domain operation schemas: {json.dumps(schemas)}

You have one tool, shell. It runs your own scripts in a Rust std WASI shell.
All file I/O uses projected Hibana Endpoints and ChoreoFS. There is no host
filesystem, native process execution, or second guest filesystem.
Use help to see the shell syntax and commands. You can use quotes, variables,
conditions, loops, pipelines, redirection, calculations, and saved scripts.
Private files under /work have names you choose. They do not grant permission.
There are eight private file slots of 8192 bytes each. A script can contain up
to 4096 UTF-8 bytes. Each guest starts at 64 KiB and can request memory growth
through its projected protocol, up to 1 MiB of supplied backing.

Each domain operation above has a writable object /service/NAME, where NAME
is its schema name. Write its JSON argument object and close it to request the
operation. Read /service/result for the actual response. Shell redirection
closes its output after writing. These objects are not native executables.
Resource loans, notifications and dependencies are carried by the projected
protocol. An operation can wait for a dependency without blocking other guests.
Choose your own scripts. No work command sequence is supplied.

Ordinary shell errors and invalid argument data can be corrected. Use actual
responses to reconsider your work. Do not add retries to a terminal domain
failure. When your assigned work is finished, close your guest with exit 0
before your final response. An exit_status reply means your guest has closed.
Do not call shell after that reply. Leaving before doing the work fails the run.
"""
        (output / f"{agent}-prompt.txt").write_text(prompt)

        async def dispatch(name, arguments):
            if name != "shell":
                raise RuntimeError("only the guest shell is exposed")
            if len(arguments["command"].encode()) > 4096:
                return {"status": "script_too_long", "limit_utf8_bytes": 4096}
            return await runtime.command(agent, arguments["command"])

        return await run_participant(
            participant=agent, prompt=prompt,
            tools=[{"type": "function", "name": "shell",
                    "description": "Run your script in the WASI wash guest over ChoreoFS.",
                    "inputSchema": {"type": "object", "properties": {
                        "command": {"type": "string"}}, "required": ["command"],
                        "additionalProperties": False}}],
            dispatch=dispatch, output=output / agent,
            timeout=config["run_timeout_seconds"],
            tool_limit=config["participant_tool_call_limit"],
        )

    try:
        async with asyncio.timeout(config["run_timeout_seconds"]):
            runtime = await DirectWasi.start(
                output / "runtime", output / "wash.wasm", registry,
                metadata["agents"], output / "io",
            )
            participants = [asyncio.create_task(participant(agent)) for agent in metadata["agents"]]
            _, protocol = await asyncio.gather(
                asyncio.gather(*participants), asyncio.shield(runtime.result),
            )
    except Exception as failure:
        error = str(failure) or type(failure).__name__
    finally:
        for future in participants:
            if not future.done():
                future.cancel()
        await asyncio.gather(*participants, return_exceptions=True)
        if runtime is not None:
            if runtime.result.done() and not runtime.result.cancelled() and runtime.result.exception() is None:
                protocol = runtime.result.result()
            await runtime.stop()
    outcomes = [future.result() for future in participants
                if not future.cancelled() and future.exception() is None]
    result = {
        "kind": "direct_wasi_integration_preflight", "scored_benchmark": False,
        "task": "16E", "scenario_depth": scenario,
        "scenario_note": "Original 16E disables injected domain failures at every depth.",
        "protocol": protocol, "error": error,
        "simulator_complete": sim.is_complete(), "simulator_progress": sim.progress,
        "simulator_violations": [dataclasses.asdict(value) for value in sim.violations],
        "simulator_events": [dataclasses.asdict(value) for value in sim.events],
        "participants": outcomes, "elapsed_seconds": time.monotonic() - started,
        "runtime": "tokio_current_thread", "versions": versions,
        "source_manifest": manifest, "binary_sha256": frozen,
        "io_events_sha256": (hashlib.sha256((output / "io/events.jsonl").read_bytes()).hexdigest()
                              if (output / "io/events.jsonl").exists() else None),
        "runtime_exit_code": runtime.process.returncode if runtime is not None else None,
    }
    (output / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({key: result[key] for key in (
        "task", "protocol", "error", "simulator_complete", "scored_benchmark",
    )}))


async def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--scenario", type=int, choices=[0, 1, 2], default=0)
    args = parser.parse_args()
    await run(args.run_id, args.scenario)


if __name__ == "__main__":
    asyncio.run(main())
