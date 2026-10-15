"""The combined 16E tools are executed by their original simulator."""
import asyncio
import importlib
import sys
from pathlib import Path

import pytest
from benchmark.tools import load_tools

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from run_preflight import Bridge


@pytest.mark.asyncio
async def test_original_cicd_pipeline_finishes_and_releases_infrastructure(tmp_path):
    module = importlib.import_module("benchmark.environments.16E.sim")
    sim = module.SimpleCICDSim()
    sim._seed = 0
    sim._delay_multiplier = 0.01
    sim.set_scenario_depth(2)
    registry = load_tools("16E", sim=sim)
    process = await asyncio.create_subprocess_exec(str(ROOT / "target/debug/cicd"), "16E",
        stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)
    bridge = Bridge(process, registry, tmp_path)
    reader = asyncio.create_task(bridge.read())
    try:
        results = await asyncio.wait_for(asyncio.gather(
            bridge.call("DEPLOYER", "deploy_application", {"target":"production"}),
            bridge.call("TESTER", "run_integration_tests", {"suite":"integration"}),
            bridge.call("DEVELOPER", "compile_and_test", {"component":"application"}),
            bridge.result), 5)
        assert results[-1] == {"type":"run_result", "status":"protocol_completed"}
        assert all(result["success"] for result in results[:-1])
        await reader
        assert await process.wait() == 0
        assert sim.is_complete()
        assert not sim.violations
        assert [event.tool for event in sim.events] == ["compile_and_test", "run_integration_tests", "deploy_application"]
    finally:
        if not reader.done():
            reader.cancel()
            await asyncio.gather(reader, return_exceptions=True)
        if process.returncode is None:
            process.terminate()
            await process.wait()
