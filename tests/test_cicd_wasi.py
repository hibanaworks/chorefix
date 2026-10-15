"""Real guest imports, projected ChoreoFS services, unchanged 16E simulator.

These deterministic hibana-wash scripts test the boundary, not model success.
"""

import asyncio
import copy
import importlib
import json
from pathlib import Path

import pytest
from benchmark.tools import load_tools
from src.imp.direct_wasi import DirectWasi
from tools.audit_direct_wasi import audit_protocol_path

ROOT = Path(__file__).resolve().parents[1]
AGENTS = ["DEVELOPER", "TESTER", "DEPLOYER"]


@pytest.mark.asyncio
@pytest.mark.parametrize("scenario", [0, 1, 2])
async def test_real_guest_recovers_then_runs_the_original_pipeline_once(tmp_path, scenario):
    module = importlib.import_module("benchmark.environments.16E.sim")
    sim = module.SimpleCICDSim()
    sim._seed = 0
    sim._delay_multiplier = 0.01
    sim.set_scenario_depth(scenario)
    registry = load_tools("16E", sim=sim)
    runtime = await asyncio.wait_for(DirectWasi.start(
        ROOT / "target/debug/cicd-wasi",
        ROOT / "vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm",
        registry, AGENTS, tmp_path / "runtime",
    ), 10)
    try:
        # Private files remain editable before any task action, but confer no
        # permission. Invalid object names and data must not reach the simulator.
        await runtime.command("DEVELOPER", 'echo private notes > /work/2')
        note = await runtime.command("DEVELOPER", 'cat /work/2')
        assert "private notes" in note["stdout"]
        unknown = await runtime.command("DEVELOPER", 'not_a_command')
        assert "command not found" in unknown["stdout"]
        missing = await runtime.command("DEVELOPER", 'cat /missing')
        assert "No such file" in missing["stdout"]
        denied = await runtime.command(
            "DEVELOPER", 'echo {} > /service/deploy_application',
        )
        assert "wash:" in denied["stdout"]
        await runtime.command(
            "DEVELOPER", "echo '{\"wrong\":\"application\"}' > /service/compile_and_test",
        )
        rejected = await runtime.command("DEVELOPER", 'cat /service/result')
        assert "invalid_arguments" in rejected["stdout"]
        assert not sim.events
        # Start dependants first. Their actual open imports must wait; private
        # work and shell parsing alone cannot authorize the shared effect.
        test = asyncio.create_task(runtime.command(
            "TESTER", "echo '{\"suite\":\"integration\"}' > /service/run_integration_tests",
        ))
        deploy = asyncio.create_task(runtime.command(
            "DEPLOYER", "echo '{\"target\":\"production\"}' > /service/deploy_application",
        ))
        await asyncio.sleep(0.05)
        assert not test.done() and not deploy.done()
        assert not sim.events
        await asyncio.wait_for(runtime.command(
            "DEVELOPER", "echo '{\"component\":\"application\"}' > /service/compile_and_test",
        ), 5)
        await asyncio.wait_for(asyncio.gather(test, deploy), 5)
        for agent in AGENTS:
            reply = await runtime.command(agent, 'cat /service/result')
            assert '"success":true' in reply["stdout"]
        duplicate = await runtime.command(
            "DEVELOPER", "echo '{\"component\":\"application\"}' > /service/compile_and_test",
        )
        assert "os error 8" in duplicate["stdout"]
        assert [event.tool for event in sim.events] == [
            "compile_and_test", "run_integration_tests", "deploy_application",
        ]
        assert sim.is_complete()
        assert not sim.violations
        exits = await asyncio.wait_for(asyncio.gather(*(
            runtime.command(agent, "exit 0") for agent in AGENTS
        )), 5)
        assert exits == [{"exit_status": 0, "stdout": ""}] * 3
        assert await asyncio.wait_for(runtime.result, 5) == {
            "type": "run_result", "status": "complete",
        }
        await runtime.reader
        assert await asyncio.wait_for(runtime.process.wait(), 5) == 0
        events = [json.loads(line) for line in
                  (tmp_path / "runtime/events.jsonl").read_text().splitlines()]
        assert len([event for event in events if event["direction"] == "domain_start"]) == 3
        trace = audit_protocol_path(events)
        assert [(path["wasi_role"], path["owner_role"]) for path in trace["paths"]] == [(0, 3), (1, 4), (2, 5)]
        # A simulator success without its real projected request is insufficient.
        close = trace["paths"][0]["close_frame"]
        missing = [event for event in events if not (
            event["direction"] == "receive" and event["message"]["type"] == "protocol_frame"
            and event["message"]["frame_id"] == close
        )]
        with pytest.raises(AssertionError):
            audit_protocol_path(missing)
        altered = copy.deepcopy(events)
        for event in altered:
            if event["direction"] == "receive" and event["message"]["type"] == "protocol_frame" and event["message"]["frame_id"] == close:
                event["message"]["source"] = 8
        with pytest.raises(AssertionError):
            audit_protocol_path(altered)
    finally:
        await runtime.stop()


@pytest.mark.asyncio
async def test_guest_exit_before_work_is_a_failed_run_with_no_domain_effect(tmp_path):
    module = importlib.import_module("benchmark.environments.16E.sim")
    sim = module.SimpleCICDSim()
    runtime = await asyncio.wait_for(DirectWasi.start(
        ROOT / "target/debug/cicd-wasi",
        ROOT / "vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm",
        load_tools("16E", sim=sim), AGENTS, tmp_path / "runtime",
    ), 10)
    try:
        # Exit has no projected service step before the assigned work. The
        # endpoint rejects it before any effect; no channel-closure workaround.
        with pytest.raises(RuntimeError, match="LabelMismatch"):
            await runtime.command("DEVELOPER", "exit")
        verdict = await asyncio.wait_for(runtime.result, 5)
        assert verdict["status"] == "failed"
        assert 'operation: "send"' in verdict["error"]
        assert "actual: 241" in verdict["error"]
        assert not sim.events
        assert not sim.is_complete()
        await runtime.reader
        assert await asyncio.wait_for(runtime.process.wait(), 5) == 1
    finally:
        await runtime.stop()
