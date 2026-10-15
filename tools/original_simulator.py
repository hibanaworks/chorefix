"""Only an RPC adapter to the unchanged Python simulator, not an admission layer.

The Rust boundary runner owns stimuli and audits. These delays deliberately
challenge parallelism and release-before-review; no simulator result is changed.
"""
import asyncio
from dataclasses import asdict
import importlib
import json
from pathlib import Path
import sys
import time

from benchmark.tools import load_tools
from benchmark.tools.sim_base import FailureSpec


def emit(value):
    print(json.dumps(value), flush=True)


async def main():
    task = sys.argv[1]
    module = importlib.import_module(f"benchmark.environments.{task}.sim")
    simulator_class = {"1E": "SharedCodebaseSim", "1M": "SharedCodebaseSim", "8E": "BackendDatabaseSyncSim", "9E": "ThreePhilosophersSim", "9M": "FivePhilosophersSim", "9H": "SevenPhilosophersSim"}.get(task, "ResearchPaperSim")
    sim = getattr(module, simulator_class)()
    model_run = len(sys.argv) == 4 and sys.argv[2] == "model"
    config = json.loads(Path("experiment.json").read_text())
    depth = int(sys.argv[3]) if len(sys.argv) == 4 and sys.argv[2] in ("model", "boundary") else 2
    if depth not in (0, 1, 2):
        raise ValueError("invalid scenario depth")
    sim._seed = config["python_hash_seed"]
    sim._delay_multiplier = config["domain_tool_delay_multiplier"] if model_run else 0.01
    sim.set_scenario_depth(depth)
    terminal_review_failure = sys.argv[2:] == ["terminal-review-failure"]
    if terminal_review_failure:
        sim._failure_specs.append(FailureSpec(tool_name="review_own_work", call_index=0))
    terminal_test_failure = sys.argv[2:] == ["terminal-test-failure"]
    if terminal_test_failure:
        sim._failure_specs.append(FailureSpec(tool_name="run_local_tests", call_index=0))
    registry = load_tools(task, sim=sim)
    metadata = json.loads((Path("vendor/tracefix/benchmark/descriptions") / task / "metadata.json").read_text())
    fixtures = []
    for agent in metadata["agents"]:
        if task == "8E":
            if agent == "DB_ADMIN":
                tool, key = "migrate_schema", "migration_name"
                value = next(step.migration_name for step in module._MIGRATE_SCHEMA_STEPS if step.agent_id == agent)
            elif agent == "BACKEND_DEV":
                tool, key = "implement_backend", "component"
                value = next(step.component for step in module._IMPLEMENT_BACKEND_STEPS if step.agent_id == agent)
            else:
                tool, key = "run_integration_tests", "suite"
                value = next(step.suite for step in module._INTEGRATION_TEST_STEPS if step.agent_id == agent)
            fixtures.append({"agent": agent, "resources": metadata["tool_resource_map"][tool],
                             "calls": [{"tool": tool, "arguments": {key: value}}]})
            continue
        if task.startswith("9"):
            fixtures.append({"agent": agent, "forks": module._FORKS[agent], "calls": [
                                 {"tool": tool, "arguments": {"philosopher": agent}} for tool in ("think", "eat")]})
            continue
        if task.startswith("1"):
            feature = next(step.feature for step in module._DESIGN_STEPS if step.agent_id == agent)
            fixtures.append({"agent": agent, "modules": metadata["agent_resources"][agent], "calls": [
                {"tool": "design_feature", "arguments": {"feature_name": feature}},
                {"tool": "implement_code", "arguments": {"feature_name": feature}},
                {"tool": "commit_changes", "arguments": {"modules": ", ".join(metadata["agent_resources"][agent])}},
                {"tool": "run_local_tests", "arguments": {"feature_name": feature}},
            ]})
            continue
        topic = next(step.topic for step in module._READ_STEPS if step.agent_id == agent)
        section = next(step.section for step in module._DRAFT_STEPS if step.agent_id == agent)
        fixtures.append({"agent": agent, "sections": metadata["agent_resources"][agent], "calls": [
            {"tool": "read_literature", "arguments": {"topic": topic}},
            {"tool": "draft_content", "arguments": {"section_name": section}},
            {"tool": "write_to_document", "arguments": {"sections": ", ".join(metadata["agent_resources"][agent])}},
            {"tool": "review_own_work", "arguments": {"section_name": section}},
        ]})
    actor_prefix = "PHILOSOPHER" if task.startswith("9") else "DEVELOPER" if task.startswith("1") else "RESEARCHER"
    constrained_tool = "eat" if task.startswith("9") else "commit_changes" if task.startswith("1") else "write_to_document"
    first_writers = {f"{actor_prefix}_A"}
    if task.endswith(("M", "H")):
        first_writers.add(f"{actor_prefix}_C")
    if task.endswith("H"):
        first_writers.add(f"{actor_prefix}_E")
    emit({"type": "fixtures", "task": task, "fixtures": fixtures, "model_run": model_run,
          "test_delays": {} if model_run else {"outcome_open_before_test_reply": True} if task == "8E" else {"first_effect_barrier": sorted(first_writers), "constrained_tool": constrained_tool, "a_review_waits_for_b_write": task.startswith("2"), "a_test_waits_for_b_commit": task.startswith("1")},
          "delay_multiplier": sim._delay_multiplier, "depth": depth, "seed": sim._seed,
          "terminal_review_failure": terminal_review_failure, "terminal_test_failure": terminal_test_failure})
    arrived = set()
    barrier, b_written = asyncio.Event(), asyncio.Event()

    async def execute(call):
        start = time.monotonic_ns()
        if not model_run and call["tool"] == constrained_tool and call["agent"] in first_writers:
            arrived.add(call["agent"])
            if arrived == first_writers:
                barrier.set()
            await barrier.wait()
        if not model_run and call["tool"] == "review_own_work" and call["agent"] == "RESEARCHER_A":
            await b_written.wait()
        if not model_run and call["tool"] == "run_local_tests" and call["agent"] == "DEVELOPER_A":
            await b_written.wait()
        result = await registry.call(call["tool"], agent_id=call["agent"], **call["arguments"])
        if call["tool"] == "write_to_document" and call["agent"] == "RESEARCHER_B":
            b_written.set()
        if call["tool"] == "commit_changes" and call["agent"] == "DEVELOPER_B":
            b_written.set()
        emit({"type": "simulator_result", "id": call["id"], "result": result.to_dict(),
              "started_ns": start, "finished_ns": time.monotonic_ns()})

    pending = []
    while line := await asyncio.to_thread(sys.stdin.readline):
        message = json.loads(line)
        if message["type"] == "score":
            await asyncio.gather(*pending)
            emit({"type": "score", "complete": sim.is_complete(),
                  "violations": [asdict(value) for value in sim.violations],
                  "events": [asdict(value) for value in sim.events], "progress": sim.progress})
            return
        pending.append(asyncio.create_task(execute(message)))


asyncio.run(main())
