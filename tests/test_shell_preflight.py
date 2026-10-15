"""Retain the primary task rejection when a real guest call is interrupted."""
import importlib
import asyncio
import json
from pathlib import Path
import shutil
import sys
import uuid

import pytest
from benchmark.tools import load_tools

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))


@pytest.mark.asyncio
async def test_task_rejection_is_not_replaced_by_a_shell_exit_error(monkeypatch):
    module = importlib.import_module("run_shell_preflight")
    run_id = "pytest-task-rejection-" + uuid.uuid4().hex
    directory = ROOT / "output/shell-preflight" / run_id

    async def participant(*, participant, dispatch, **unused):
        # Model access is replaced only in this test. Commands still use the
        # actual guest, ChoreoFS, Rust endpoints, RPC bridge and simulator.
        call = ({"tool":"code_review", "arguments":{"pr_id":"dev_a_feature"}}
                if participant == "REVIEWER" else
                {"tool":"implement_feature", "arguments":{"feature_name":{
                    "DEVELOPER_A":"user_authentication", "DEVELOPER_B":"notification_feature"}[participant]}})
        try:
            await dispatch("shell", {"command":"call " + json.dumps(call)})
        except RuntimeError as error:
            return {"participant":participant, "status":"harness_error", "error":str(error)}
        raise AssertionError("invalid target should stop the run before a reply")

    monkeypatch.setattr(module, "run_participant", participant)
    try:
        await module.run("6E", run_id, 1)
        result = json.loads((directory / "6E/result.json").read_text())
        assert result["error"] is None
        assert result["protocol"]["status"] == "failed"
        assert "unknown review tool or original feature target" in result["protocol"]["error"]
        assert len(result["participants"]) == 3
        assert all(value["status"] == "harness_error" for value in result["participants"])
        for agent in ["DEVELOPER_A", "DEVELOPER_B", "REVIEWER"]:
            events = [json.loads(line) for line in (directory / "6E" / (agent+"-shell") / "events.jsonl").read_text().splitlines()]
            assert not any(value["direction"] == "send" and value["message"] == {"command":"exit"} for value in events)
        assert not result["simulator_complete"]
        assert not any(event["tool"] == "code_review" for event in result["simulator_events"])
    finally:
        if directory.exists():
            shutil.rmtree(directory)


@pytest.mark.asyncio
async def test_call_after_closed_rpc_keeps_the_original_task_rejection(tmp_path):
    module = importlib.import_module("run_preflight")
    simulator_module = importlib.import_module("benchmark.environments.6E.sim")
    sim = simulator_module.SharedUtilityFileSim()
    process = await asyncio.create_subprocess_exec(
        str(ROOT / "target/debug/review"), "6E", stdin=asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE,
    )
    bridge = module.Bridge(process, load_tools("6E", sim=sim), tmp_path)
    reader = asyncio.create_task(bridge.read())
    try:
        with pytest.raises(RuntimeError):
            await asyncio.wait_for(bridge.call(
                "REVIEWER", "code_review", {"pr_id": "dev_a_feature"},
            ), 5)
        verdict = await asyncio.wait_for(bridge.result, 5)
        await reader
        assert "unknown review tool or original feature target" in verdict["error"]
        assert bridge.log.closed
        # Exercise the exact ordering seen on CI: another participant submits
        # after the verdict, stream and logger have already become terminal.
        with pytest.raises(RuntimeError, match="unknown review tool or original feature target"):
            await bridge.call(
                "DEVELOPER_A", "implement_feature", {"feature_name": "user_authentication"},
            )
        assert not bridge.pending
        assert not sim.events
        assert await asyncio.wait_for(process.wait(), 5) == 0
    finally:
        if not reader.done():
            reader.cancel()
            await asyncio.gather(reader, return_exceptions=True)
        if process.returncode is None:
            process.terminate()
            await process.wait()
