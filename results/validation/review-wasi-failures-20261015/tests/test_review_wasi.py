"""Real wash guests, one projected REPO owner, and unchanged TraceFix 6E."""
import asyncio
import importlib
import json
import os
from pathlib import Path

import pytest
from benchmark.tools import load_tools
from src.imp.direct_wasi import DirectWasi

ROOT = Path(__file__).resolve().parents[1]
AGENTS = ["DEVELOPER_A", "DEVELOPER_B", "REVIEWER"]


def response(reply):
    output = reply["stdout"]
    return json.JSONDecoder().raw_decode(output[output.index("{"):])[0]


@pytest.mark.asyncio
@pytest.mark.parametrize("depth", [0, 1, 2])
async def test_real_shell_revision_and_independent_review(tmp_path, depth):
    module = importlib.import_module("benchmark.environments.6E.sim")
    sim = module.SharedUtilityFileSim()
    sim._seed = 0
    sim._delay_multiplier = 0.01
    sim.set_scenario_depth(depth)
    registry = load_tools("6E", sim=sim)
    original_call = registry.call
    b_implemented, b_reviewed = asyncio.Event(), asyncio.Event()

    async def observe(tool, *, agent_id, **arguments):
        # Delays and observations belong to this test, not runtime permission.
        if tool == "code_review" and arguments["pr_id"] == "user_authentication":
            await asyncio.wait_for(b_implemented.wait(), 5)
        if tool == "merge_code" and agent_id == "DEVELOPER_A":
            await asyncio.wait_for(b_reviewed.wait(), 5)
        result = await original_call(tool, agent_id=agent_id, **arguments)
        if tool == "implement_feature" and agent_id == "DEVELOPER_B":
            b_implemented.set()
        if tool == "code_review" and arguments["pr_id"] == "notification_feature":
            b_reviewed.set()
        return result

    registry.call = observe
    runtime = await asyncio.wait_for(DirectWasi.start(
        Path(os.environ.get("CHOREOFIX_REVIEW_WASI_BINARY", ROOT / "target/debug/review-wasi")),
        ROOT / "vendor/hibana-wash/target/wasm32-wasip1/release/hibana-wash.wasm",
        registry, AGENTS, tmp_path / "runtime",
    ), 10)
    feedback = {}

    async def developer(agent, feature, branch):
        # Wrong-order requests and malformed data are ordinary shell errors.
        early = await runtime.command(agent, "echo '{}' > /service/merge_code")
        assert "wash:" in early["stdout"]
        await runtime.command(agent, "echo '{}' > /service/implement_feature")
        assert response(await runtime.command(agent, "cat /service/result"))["status"] == "invalid_arguments"
        received = []
        while True:
            result = response(await runtime.command(agent,
                f"printf '%s' '{{\"feature_name\":\"{feature}\"}}' > /service/implement_feature; cat /service/result"))
            assert result["success"] is True
            received.append(result["coordination"]["code_review"])
            if received[-1]["success"]:
                break
            denied = await runtime.command(agent, "echo '{}' > /service/merge_code")
            assert "wash:" in denied["stdout"]
            assert not any(event.tool == "merge_code" and event.agent == agent for event in sim.events)
        feedback[agent] = received
        result = response(await runtime.command(agent,
            f"echo '{{\"branch\":\"{branch}\"}}' > /service/merge_code; cat /service/result"))
        assert result["success"] is True
        await runtime.command(agent, "exit 0")

    async def reviewer():
        for feature in ("user_authentication", "notification_feature"):
            while True:
                result = response(await runtime.command("REVIEWER",
                    f"echo '{{\"pr_id\":\"{feature}\"}}' > /service/code_review/{feature}; cat /service/result"))
                if result["success"]:
                    break
        await runtime.command("REVIEWER", "exit 0")

    try:
        *_, verdict = await asyncio.wait_for(asyncio.gather(
            developer("DEVELOPER_A", "user_authentication", "dev_a_feature"),
            developer("DEVELOPER_B", "notification_feature", "dev_b_feature"),
            reviewer(), runtime.result,
        ), 20)
        assert verdict == {"type":"run_result", "status":"complete"}
        await runtime.reader
        assert await runtime.process.wait() == 0
        assert sim.is_complete() and not sim.violations
        assert [value["success"] for value in feedback["DEVELOPER_A"]] == [False] * depth + [True]
        assert [value["success"] for value in feedback["DEVELOPER_B"]] == [True]
        assert len(sim.events) == 6 + 2 * depth
        events = [json.loads(line) for line in (tmp_path / "runtime/events.jsonl").read_text().splitlines()]
        active = set()
        for event in events:
            if event["direction"] not in ("domain_start", "domain_end"):
                continue
            message = event["message"]
            if message["tool"] not in ("implement_feature", "merge_code"):
                continue
            if event["direction"] == "domain_start":
                assert not active
                active.add(message["id"])
            else:
                active.remove(message["id"])
        assert not active
    finally:
        await runtime.stop()
