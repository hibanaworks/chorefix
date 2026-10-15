"""Actual process, original task data, and unchanged external scoring."""
import asyncio
import importlib
import json
import sys
from pathlib import Path

import pytest
from benchmark.tools import load_tools

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from run_preflight import Bridge


@pytest.mark.asyncio
@pytest.mark.parametrize("task", ["2E", "2M", "2H"])
async def test_original_writing_work_finishes_without_shared_section_conflicts(task, tmp_path):
    module = importlib.import_module(f"benchmark.environments.{task}.sim")
    sim = module.ResearchPaperSim()
    sim._seed = 0
    sim._delay_multiplier = 0.01
    sim.set_scenario_depth(2)
    registry = load_tools(task, sim=sim)
    metadata = json.loads((ROOT / "vendor/tracefix/benchmark/descriptions" / task / "metadata.json").read_text())
    process = await asyncio.create_subprocess_exec(str(ROOT / "target/debug/writing"), task,
        stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)
    bridge = Bridge(process, registry, tmp_path)
    reader = asyncio.create_task(bridge.read())
    async def participant(agent):
        topic = next(step.topic for step in module._READ_STEPS if step.agent_id == agent)
        section = next(step.section for step in module._DRAFT_STEPS if step.agent_id == agent)
        calls = [("read_literature", {"topic": topic}), ("draft_content", {"section_name": section}),
                 ("write_to_document", {"sections": ", ".join(metadata["agent_resources"][agent])}),
                 ("review_own_work", {"section_name": section})]
        for tool, arguments in calls:
            result = await bridge.call(agent, tool, arguments)
            assert result["success"] is True
    try:
        results = await asyncio.wait_for(asyncio.gather(*(participant(agent) for agent in metadata["agents"]), bridge.result), 5)
        assert results[-1] == {"type": "run_result", "status": "protocol_completed"}
        await reader
        assert await process.wait() == 0
        assert sim.is_complete()
        assert not sim.violations
        assert len(sim.events) == 4 * len(metadata["agents"])
    finally:
        if not reader.done():
            reader.cancel()
            await asyncio.gather(reader, return_exceptions=True)
        if process.returncode is None:
            process.terminate()
            await process.wait()
