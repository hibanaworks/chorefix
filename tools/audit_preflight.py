#!/usr/bin/env python3
"""Read saved raw events and check the integration preflight's claimed outcomes."""

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def audit(directory):
    result = json.loads((directory / "result.json").read_text())
    assert result["scored_benchmark"] is False
    assert result["kind"] == "integration_preflight"
    assert result["error"] is None
    assert result["protocol"]["status"] == "protocol_completed"
    assert result["simulator_complete"] is True
    assert not result["simulator_violations"]
    expected = json.loads((ROOT / "vendor/tracefix/benchmark/descriptions" / result["task"] / "metadata.json").read_text())["agents"]
    assert {agent["participant"] for agent in result["participants"]} == set(expected)
    total_tokens = 0
    for agent in result["participants"]:
        assert agent["status"] == "completed"
        assert agent["returned_model"] == agent["requested_model"] == "gpt-6-luna"
        assert agent["returned_effort"] == agent["requested_effort"] == "xhigh"
        assert agent["tool_calls"] == 2
        raw = directory / agent["participant"] / "events.jsonl"
        assert hashlib.sha256(raw.read_bytes()).hexdigest() == agent["raw_events_sha256"]
        calls = []
        for line in raw.read_text().splitlines():
            message = json.loads(line)["message"]
            if message.get("method") == "item/tool/call":
                calls.append(message["params"]["tool"])
            item = message.get("params", {}).get("item", {})
            assert item.get("type") in {None, "dynamicToolCall", "reasoning", "userMessage", "agentMessage"}
        assert calls == ["think", "eat"]
        total_tokens += agent["usage"]["totalTokens"]
    log = directory / "bridge-events.jsonl"
    assert hashlib.sha256(log.read_bytes()).hexdigest() == result["bridge_events_sha256"]
    entries = [json.loads(line) for line in log.read_text().splitlines()]
    executed = [entry["message"] for entry in entries if entry["direction"] == "receive" and entry["message"]["type"] == "execute"]
    assert len(executed) == 2*len(expected)
    assert len({call["id"] for call in executed}) == len(executed)
    for agent in expected:
        assert [call["tool"] for call in executed if call["agent"] == agent] == ["think", "eat"]
        events = [event["tool"] for event in result["simulator_events"] if event["agent"] == agent]
        assert events == ["think", "eat"]
    active = set()
    maximum = 0
    for entry in entries:
        if entry["direction"] not in {"domain_start", "domain_end"} or entry["message"]["tool"] != "eat":
            continue
        agent = entry["message"]["agent"]
        if entry["direction"] == "domain_start":
            assert agent not in active
            active.add(agent)
            maximum = max(maximum, len(active))
        else:
            active.remove(agent)
    assert not active
    return {"task": result["task"], "kind":"integration_preflight", "scored_benchmark":False,
            "audit_passed":True, "participants":len(expected), "domain_calls":len(executed),
            "maximum_concurrent_eaters":maximum, "total_tokens":total_tokens,
            "result_sha256":hashlib.sha256((directory / "result.json").read_bytes()).hexdigest(),
            "source_manifest": result["source_manifest"]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    print(json.dumps(audit(args.directory), indent=2))


if __name__ == "__main__":
    main()
