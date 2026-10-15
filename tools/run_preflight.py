#!/usr/bin/env python3
"""Unscored integration preflight with real Luna participants and the original simulator."""

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


class Bridge:
    """RPC correlation only. Rust endpoints authorize all domain execution."""

    def __init__(self, process, registry, output):
        self.process = process
        self.registry = registry
        self.sequence = 0
        self.pending = {}
        self.write_lock = asyncio.Lock()
        self.result = asyncio.get_running_loop().create_future()
        self.log = (output / "bridge-events.jsonl").open("w")
        self.started = time.monotonic()

    def record(self, direction, value):
        self.log.write(json.dumps({"elapsed_seconds": time.monotonic()-self.started,
                                   "direction": direction, "message": value}) + "\n")
        self.log.flush()

    async def send(self, value):
        async with self.write_lock:
            self.record("send", value)
            self.process.stdin.write((json.dumps(value) + "\n").encode())
            await self.process.stdin.drain()

    async def call(self, agent, tool, arguments):
        self.sequence += 1
        identifier = self.sequence
        future = asyncio.get_running_loop().create_future()
        self.pending[identifier] = future
        await self.send({"type": "call", "id": identifier, "agent": agent,
                         "tool": tool, "arguments": arguments})
        return await future

    async def execute(self, value):
        try:
            self.record("domain_start", value)
            result = await self.registry.call(value["tool"], agent_id=value["agent"], **value["arguments"])
            self.record("domain_end", value)
            await self.send({"type": "simulator_result", "id": value["id"], "result": result.to_dict()})
        except Exception as error:
            if not self.result.done():
                self.result.set_exception(error)
            raise

    async def read(self):
        executions = set()
        try:
            while raw := await self.process.stdout.readline():
                value = json.loads(raw)
                self.record("receive", value)
                if value["type"] == "execute":
                    task = asyncio.create_task(self.execute(value))
                    executions.add(task)
                elif value["type"] == "tool_result":
                    self.pending.pop(value["id"]).set_result(value["result"])
                elif value["type"] == "run_result":
                    self.result.set_result(value)
                    self.process.stdin.close()
                    await self.process.stdin.wait_closed()
                    break
                else:
                    raise RuntimeError("unknown Rust bridge message")
            if not self.result.done():
                raise RuntimeError("Rust bridge closed without a run verdict")
        except Exception as error:
            if not self.result.done():
                self.result.set_exception(error)
        finally:
            for future in self.pending.values():
                if not future.done():
                    future.set_exception(RuntimeError("protocol run ended before this tool executed"))
            for task in executions:
                if not task.done():
                    task.cancel()
            await asyncio.gather(*executions, return_exceptions=True)
            self.log.close()


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
    output = ROOT / "output" / "integration-preflight" / run_id / task
    output.mkdir(parents=True, exist_ok=False)
    source_files = [path for path in (ROOT / "src").rglob("*")
                    if path.is_file() and "__pycache__" not in path.parts]
    source_files += [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "experiment.json",
                     Path(__file__).resolve()]
    source_hashes = {}
    for path in sorted(source_files):
        relative = path.relative_to(ROOT)
        content = path.read_bytes()
        source_hashes[str(relative)] = hashlib.sha256(content).hexdigest()
        snapshot = output / "source" / relative
        snapshot.parent.mkdir(parents=True, exist_ok=True)
        snapshot.write_bytes(content)
    (output / "source-manifest.json").write_text(json.dumps(source_hashes, indent=2)+"\n")
    binary = ROOT / "target/debug" / {"1":"codebase", "2":"writing", "6":"review", "8":"api", "9":"philosophers", "16":"cicd"}[task[:-1]]
    binary_content = binary.read_bytes()
    runtime_binary_sha256 = hashlib.sha256(binary_content).hexdigest()
    runtime_snapshot = output / "runtime"
    runtime_snapshot.write_bytes(binary_content)
    runtime_snapshot.chmod(0o700)
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

        async def dispatch(name, arguments):
            jsonschema.validate(arguments, by_name[name]["parameters"])
            return await bridge.call(agent, name, arguments)

        if task[:-1] == "1":
            features = {step.agent_id:step.feature for step in module._DESIGN_STEPS}
            feature = features[agent]
            modules = ", ".join(metadata["agent_resources"][agent])
            conclusion = ("If tests fail, use the actual failure to reconsider your fix. Briefly state your revised approach, then submit new implement_code, commit_changes, and run_local_tests calls until the tests pass. Do not replay the previous calls blindly."
                          if task == "1H" else "The test result is terminal. Report any failed test and finish. Do not repeat work.")
            prompt = f"""You are {agent} in the full original TraceFix task {task}.
Call design_feature(feature_name="{feature}") once.
Then call implement_code(feature_name="{feature}"),
commit_changes(modules="{modules}"), and
run_local_tests(feature_name="{feature}") in that order.
{conclusion} Do not repeat design.
Hibana endpoints coordinate both module leases. Calls wait for those leases.
Do all real domain calls. Do not simulate a tool result in text. Finish only
when the workflow described above is complete. Use only the supplied tools.
"""
        elif task[:-1] == "2":
            topic = {step.agent_id:step.topic for step in module._READ_STEPS}[agent]
            section = {step.agent_id:step.section for step in module._DRAFT_STEPS}[agent]
            sections = ", ".join(metadata["agent_resources"][agent])
            prompt = f"""You are {agent} in the original TraceFix task {task}.
Call read_literature(topic="{topic}"), draft_content(section_name="{section}"),
write_to_document(sections="{sections}"), review_own_work(section_name="{section}")
in that order. Review is terminal even if it finds issues. Do not repeat work.
Hibana endpoints coordinate both section loans. Perform actual calls with the
supplied tools. Finish after the review response.
"""
        elif task == "6E":
            prompt = f"""You are {agent} in the original TraceFix task 6E.
DEVELOPER_A implements user_authentication and merges dev_a_feature.
DEVELOPER_B implements notification_feature and merges dev_b_feature.
An implement_feature reply includes the actual reviewer feedback in
coordination.code_review after the repository has been returned. Reconsider
and implement again when it requests changes. Merge only after approval.
REVIEWER calls code_review with pr_id user_authentication or
notification_feature. Review both features until each is approved. A review
waits for an actual submission. Do not issue more reviews after approval.
Hibana carries the ready and feedback messages and repository loans.
Use actual tools, perform only your role's work, and finish when it is done.
"""
        elif task == "8E":
            prompt = f"""You are {agent} in the original TraceFix task 8E.
DB_ADMIN calls migrate_schema(migration_name="add_new_tables").
BACKEND_DEV calls implement_backend(component="new_api_endpoints").
TESTER calls run_integration_tests(suite="backend_schema_integration").
Perform only your role's domain call. Hibana coordinates prerequisites,
resource loans, and test-result notifications. The test result is terminal,
including failure. Report the actual result and finish. Do not retry.
"""
        elif task == "16E":
            prompt = f"""You are {agent} in the original TraceFix task 16E.
DEVELOPER calls compile_and_test(component="application").
TESTER calls run_integration_tests(suite="integration").
DEPLOYER calls deploy_application(target="production").
Perform only your role's domain call. Hibana coordinates notifications and
infrastructure loans. A call may wait for a prerequisite. Use the actual tool.
"""
        else:
            prompt = f"""You are {agent} in the original TraceFix task {task}.
    Call think(philosopher=\"{agent}\"), then eat(philosopher=\"{agent}\").
    Perform both real tool calls exactly once. Resource grants and returns are
    carried out by the Hibana endpoint functions. Each call waits for the resources
    it needs. Do not skip work or simulate a tool result in text. Finish after both
    tool responses succeed. Only the supplied domain tools are available.
    """
        result = await run_participant(
            participant=agent, prompt=prompt, tools=[tool_spec(schema) for schema in schemas],
            dispatch=dispatch, output=output / agent,
            timeout=config["run_timeout_seconds"], tool_limit=config["participant_tool_call_limit"],
        )
        if not bridge.result.done():
            event = "model_finished" if result["status"] == "completed" else "close_role"
            await bridge.send({"type":event, "id":0, "agent":agent})
        return result

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
        "task": task, "scenario_depth":scenario, "kind": "integration_preflight", "scored_benchmark": False,
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
        "coordination_mode": "endpoint_functions_with_domain_only_model_tools",
    }
    (output / "result.json").write_text(json.dumps(result, indent=2)+"\n")
    print(json.dumps({"task": task, "protocol": protocol, "error": error, "simulator_complete": sim.is_complete(),
                      "violations": len(sim.violations), "scored_benchmark": False}))


async def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tasks", nargs="+", choices=["1E", "1M", "1H", "2E", "2M", "2H", "6E", "8E", "9E", "9M", "9H", "16E"], required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--scenario", type=int, choices=[0,1,2], default=0)
    args = parser.parse_args()
    for task in args.tasks:
        await run(task, args.run_id, args.scenario)


if __name__ == "__main__":
    asyncio.run(main())
