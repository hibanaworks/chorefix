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
    sim = module.ResearchPaperSim()
    sim._seed = 0
    sim._delay_multiplier = 0.01
    sim.set_scenario_depth(2)
    terminal_review_failure = sys.argv[2:] == ["terminal-review-failure"]
    if terminal_review_failure:
        sim._failure_specs.append(FailureSpec(tool_name="review_own_work", call_index=0))
    registry = load_tools(task, sim=sim)
    metadata = json.loads((Path("vendor/tracefix/benchmark/descriptions") / task / "metadata.json").read_text())
    fixtures = []
    for agent in metadata["agents"]:
        topic = next(step.topic for step in module._READ_STEPS if step.agent_id == agent)
        section = next(step.section for step in module._DRAFT_STEPS if step.agent_id == agent)
        fixtures.append({"agent": agent, "sections": metadata["agent_resources"][agent], "calls": [
            {"tool": "read_literature", "arguments": {"topic": topic}},
            {"tool": "draft_content", "arguments": {"section_name": section}},
            {"tool": "write_to_document", "arguments": {"sections": ", ".join(metadata["agent_resources"][agent])}},
            {"tool": "review_own_work", "arguments": {"section_name": section}},
        ]})
    first_writers = {"RESEARCHER_A"}
    if task in ("2M", "2H"):
        first_writers.add("RESEARCHER_C")
    if task == "2H":
        first_writers.add("RESEARCHER_E")
    emit({"type": "fixtures", "task": task, "fixtures": fixtures,
          "test_delays": {"first_write_barrier": sorted(first_writers), "a_review_waits_for_b_write": True},
          "delay_multiplier": 0.01, "depth": 2, "seed": 0,
          "terminal_review_failure": terminal_review_failure})
    arrived = set()
    barrier, b_written = asyncio.Event(), asyncio.Event()

    async def execute(call):
        start = time.monotonic_ns()
        if call["tool"] == "write_to_document" and call["agent"] in first_writers:
            arrived.add(call["agent"])
            if arrived == first_writers:
                barrier.set()
            await barrier.wait()
        if call["tool"] == "review_own_work" and call["agent"] == "RESEARCHER_A":
            await b_written.wait()
        result = await registry.call(call["tool"], agent_id=call["agent"], **call["arguments"])
        if call["tool"] == "write_to_document" and call["agent"] == "RESEARCHER_B":
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
