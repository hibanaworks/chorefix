"""Actual review feedback and resource interleavings against unchanged 6E."""
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
@pytest.mark.parametrize("depth", [0, 1, 2])
async def test_original_review_feedback_revises_and_does_not_hold_the_repo(tmp_path, depth):
    module = importlib.import_module("benchmark.environments.6E.sim")
    sim = module.SharedUtilityFileSim()
    sim._seed = 0
    sim._delay_multiplier = 0.01
    sim.set_scenario_depth(depth)
    registry = load_tools("6E", sim=sim)
    original_call = registry.call
    b_implemented, b_reviewed = asyncio.Event(), asyncio.Event()

    async def observe(tool, *, agent_id, **arguments):
        # A's review must not own the repository, and A's pending merge must
        # not prevent the reviewer from reviewing B. These are observations
        # and adversarial delays in this test, not production authorization.
        if tool == "code_review" and arguments["pr_id"] == "user_authentication":
            await asyncio.wait_for(b_implemented.wait(), 3)
        if tool == "merge_code" and agent_id == "DEVELOPER_A":
            await asyncio.wait_for(b_reviewed.wait(), 3)
        result = await original_call(tool, agent_id=agent_id, **arguments)
        if tool == "implement_feature" and agent_id == "DEVELOPER_B":
            b_implemented.set()
        if tool == "code_review" and arguments["pr_id"] == "notification_feature":
            b_reviewed.set()
        return result

    registry.call = observe
    process = await asyncio.create_subprocess_exec(
        str(ROOT / "target/debug/review"), "6E", stdin=asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)
    bridge = Bridge(process, registry, tmp_path)
    reader = asyncio.create_task(bridge.read())
    feedback = {}

    async def developer(agent, feature, branch):
        received = []
        while True:
            result = await bridge.call(agent, "implement_feature", {"feature_name":feature})
            received.append(result["coordination"]["code_review"])
            if received[-1]["success"]:
                break
        feedback[agent] = received
        result = await bridge.call(agent, "merge_code", {"branch":branch})
        assert result["success"]

    async def reviewer():
        for feature in ["user_authentication", "notification_feature"]:
            while True:
                result = await bridge.call("REVIEWER", "code_review", {"pr_id":feature})
                if result["success"]:
                    break
        await bridge.send({"type":"model_finished", "id":0, "agent":"REVIEWER"})

    try:
        *_, verdict = await asyncio.wait_for(asyncio.gather(
            developer("DEVELOPER_A", "user_authentication", "dev_a_feature"),
            developer("DEVELOPER_B", "notification_feature", "dev_b_feature"),
            reviewer(), bridge.result), 10)
        await reader
        assert verdict == {"type":"run_result", "status":"protocol_completed"}
        assert await process.wait() == 0
        assert sim.is_complete() and not sim.violations
        assert [v["success"] for v in feedback["DEVELOPER_A"]] == [False]*depth+[True]
        assert [v["success"] for v in feedback["DEVELOPER_B"]] == [True]
        actual = [v for v in sim.events if v.tool == "code_review"]
        assert [v.success for v in actual] == [False]*depth+[True, True]
        assert len(sim.events) == 6+2*depth
        events = [json.loads(line) for line in (tmp_path / "bridge-events.jsonl").read_text().splitlines()]
        replies = [v["message"]["result"] for v in events if v["direction"] == "receive" and v["message"]["type"] == "tool_result"]
        reviews = [v["message"]["result"] for v in events if v["direction"] == "send" and v["message"]["type"] == "simulator_result" and v["message"]["result"]["tool_name"] == "code_review"]
        forwarded = [v["coordination"]["code_review"] for v in replies if "coordination" in v]
        assert sorted(json.dumps(v, sort_keys=True) for v in forwarded) == sorted(json.dumps(v, sort_keys=True) for v in reviews)
    finally:
        if not reader.done():
            reader.cancel()
            await asyncio.gather(reader, return_exceptions=True)
        if process.returncode is None:
            process.terminate()
            await process.wait()


@pytest.mark.asyncio
async def test_merge_without_implementation_and_approval_has_no_domain_effect(tmp_path):
    process = await asyncio.create_subprocess_exec(
        str(ROOT / "target/debug/review"), "6E", stdin=asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)
    try:
        process.stdin.write((json.dumps({"type":"call", "id":1,"agent":"DEVELOPER_A",
            "tool":"merge_code", "arguments":{"branch":"dev_a_feature"}})+"\n").encode())
        await process.stdin.drain()
        reply = json.loads(await asyncio.wait_for(process.stdout.readline(), 3))
        assert reply["type"] == "run_result" and reply["status"] == "failed"
        assert "Endpoint" in reply["error"]
        process.stdin.close()
        await process.stdin.wait_closed()
        assert await asyncio.wait_for(process.wait(), 3) == 0
    finally:
        if process.returncode is None:
            process.terminate()
            await process.wait()


@pytest.mark.asyncio
async def test_merge_after_revision_request_has_no_domain_effect(tmp_path):
    module = importlib.import_module("benchmark.environments.6E.sim")
    sim = module.SharedUtilityFileSim()
    sim._seed = 0
    sim._delay_multiplier = 0.01
    sim.set_scenario_depth(1)
    process = await asyncio.create_subprocess_exec(
        str(ROOT / "target/debug/review"), "6E", stdin=asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)
    bridge = Bridge(process, load_tools("6E", sim=sim), tmp_path)
    reader = asyncio.create_task(bridge.read())
    try:
        implementation, review = await asyncio.wait_for(asyncio.gather(
            bridge.call("DEVELOPER_A", "implement_feature", {"feature_name":"user_authentication"}),
            bridge.call("REVIEWER", "code_review", {"pr_id":"user_authentication"})), 3)
        assert implementation["coordination"]["code_review"] == review
        assert review["success"] is False
        with pytest.raises(RuntimeError, match="ended before this tool executed"):
            await asyncio.wait_for(bridge.call("DEVELOPER_A", "merge_code", {"branch":"dev_a_feature"}), 3)
        verdict = await bridge.result
        assert verdict["status"] == "failed" and "Endpoint" in verdict["error"]
        await reader
        assert await asyncio.wait_for(process.wait(), 3) == 0
        assert [event.tool for event in sim.events] == ["implement_feature", "code_review"]
        assert not sim.violations and not sim.is_complete()
    finally:
        if not reader.done():
            reader.cancel()
            await asyncio.gather(reader, return_exceptions=True)
        if process.returncode is None:
            process.terminate()
            await process.wait()


@pytest.mark.asyncio
async def test_model_completion_cannot_stand_in_for_both_merges(tmp_path):
    process = await asyncio.create_subprocess_exec(
        str(ROOT / "target/debug/review"), "6E", stdin=asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)
    try:
        process.stdin.write((json.dumps({"type":"model_finished", "id":0,"agent":"REVIEWER"})+"\n").encode())
        await process.stdin.drain()
        verdict=json.loads(await asyncio.wait_for(process.stdout.readline(), 3))
        assert verdict["type"] == "run_result" and verdict["status"] == "failed"
        assert "Endpoint" in verdict["error"]
        process.stdin.close()
        await process.stdin.wait_closed()
        assert await asyncio.wait_for(process.wait(), 3) == 0
    finally:
        if process.returncode is None:
            process.terminate()
            await process.wait()
