"""8E preserves both successful and terminal failed original test outcomes."""
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
@pytest.mark.parametrize("depth", [0, 1, 2])
async def test_original_api_result_is_terminal_without_remigration(tmp_path, depth):
    module = importlib.import_module("benchmark.environments.8E.sim")
    sim = module.BackendDatabaseSyncSim()
    sim._seed = 0
    sim._delay_multiplier = 0.01
    sim.set_scenario_depth(depth)
    registry = load_tools("8E", sim=sim)
    process = await asyncio.create_subprocess_exec(
        str(ROOT / "target/debug/api"), "8E",
        stdin=asyncio.subprocess.PIPE, stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE)
    bridge = Bridge(process, registry, tmp_path)
    reader = asyncio.create_task(bridge.read())
    try:
        results = await asyncio.wait_for(asyncio.gather(
            bridge.call("TESTER", "run_integration_tests", {"suite":"backend_schema_integration"}),
            bridge.call("BACKEND_DEV", "implement_backend", {"component":"new_api_endpoints"}),
            bridge.call("DB_ADMIN", "migrate_schema", {"migration_name":"add_new_tables"}),
            bridge.result), 5)
        assert results[-1] == {"type":"run_result", "status":"protocol_completed"}
        assert results[0]["success"] is (depth == 0)
        assert results[1]["success"] and results[2]["success"]
        await reader
        assert await process.wait() == 0
        assert sim.is_complete() is (depth == 0)
        assert not sim.violations
        assert [event.tool for event in sim.events] == [
            "migrate_schema", "implement_backend", "run_integration_tests"]
    finally:
        if not reader.done():
            reader.cancel()
            await asyncio.gather(reader, return_exceptions=True)
        if process.returncode is None:
            process.terminate()
            await process.wait()
