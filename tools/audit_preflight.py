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
    assert result["kind"] in {"integration_preflight", "shell_integration_preflight"}
    shell = result["kind"] == "shell_integration_preflight"
    if shell:
        assert all(value["exit_code"] == 0 for value in result["wasi_processes"].values())
        for name, key in [("wasi-runtime", "wasi_runtime_sha256"), ("shell.wasm", "guest_sha256")]:
            assert hashlib.sha256((directory / name).read_bytes()).hexdigest() == result[key]
    assert result["error"] is None
    assert result["protocol"]["status"] == "protocol_completed"
    assert result["simulator_complete"] is True
    assert not result["simulator_violations"]
    assert result["runtime"] == "tokio_current_thread"
    assert result["runtime_exit_code"] == 0
    assert hashlib.sha256((directory / "runtime").read_bytes()).hexdigest() == result["runtime_binary_sha256"]
    saved_manifest = json.loads((directory / "source-manifest.json").read_text())
    assert saved_manifest == result["source_manifest"]
    for relative, digest in saved_manifest.items():
        assert hashlib.sha256((directory / "source" / relative).read_bytes()).hexdigest() == digest
    expected = json.loads((ROOT / "vendor/tracefix/benchmark/descriptions" / result["task"] / "metadata.json").read_text())["agents"]
    assert {agent["participant"] for agent in result["participants"]} == set(expected)
    total_tokens = 0
    codebase = result["task"].startswith("1")
    depth = result["scenario_depth"]
    retries = depth if result["task"] == "1H" else 0
    expected_calls = (["design_feature"] + ["implement_code", "commit_changes", "run_local_tests"]*(retries+1)
                      if codebase else ["think", "eat"])
    for agent in result["participants"]:
        assert agent["status"] == "completed"
        assert agent["returned_model"] == agent["requested_model"] == "gpt-6-luna"
        assert agent["returned_effort"] == agent["requested_effort"] == "xhigh"
        if not shell:
            assert agent["tool_calls"] == len(expected_calls)
        raw = directory / agent["participant"] / "events.jsonl"
        assert hashlib.sha256(raw.read_bytes()).hexdigest() == agent["raw_events_sha256"]
        calls = []
        commands = []
        usage = None
        for line in raw.read_text().splitlines():
            message = json.loads(line)["message"]
            if message.get("method") == "item/tool/call":
                calls.append(message["params"]["tool"])
                if shell:
                    commands.append(message["params"]["arguments"]["command"])
            if message.get("method") == "thread/tokenUsage/updated":
                usage = message["params"]["tokenUsage"]["total"]
            item = message.get("params", {}).get("item", {})
            assert item.get("type") in {None, "dynamicToolCall", "reasoning", "userMessage", "agentMessage"}
        assert usage == agent["usage"]
        if shell:
            assert calls == ["shell"] * agent["tool_calls"]
            shell_path = directory / (agent["participant"] + "-shell") / "events.jsonl"
            assert hashlib.sha256(shell_path.read_bytes()).hexdigest() == result["shell_events_sha256"][agent["participant"]]
            events = [json.loads(line) for line in shell_path.read_text().splitlines()]
            observed_commands = [entry["message"]["command"] for entry in events if entry["direction"] == "send" and "command" in entry["message"]]
            assert observed_commands == (commands if commands and commands[-1] == "exit" else commands + ["exit"])
            effects = [entry["message"]["request"] for entry in events if entry["direction"] == "receive" and entry["message"]["type"] == "effect"]
            assert [effect["tool"] for effect in effects] == expected_calls
            assert events[-1]["message"] == {"type":"exit", "status":0}
        else:
            assert calls == expected_calls
        total_tokens += agent["usage"]["totalTokens"]
    log = directory / "bridge-events.jsonl"
    assert hashlib.sha256(log.read_bytes()).hexdigest() == result["bridge_events_sha256"]
    entries = [json.loads(line) for line in log.read_text().splitlines()]
    executed = [entry["message"] for entry in entries if entry["direction"] == "receive" and entry["message"]["type"] == "execute"]
    assert len(executed) == len(expected_calls)*len(expected)
    assert len({call["id"] for call in executed}) == len(executed)
    for agent in expected:
        if shell:
            events = [json.loads(line) for line in (directory / (agent + "-shell") / "events.jsonl").read_text().splitlines()]
            effects = [entry["message"]["request"] for entry in events if entry["direction"] == "receive" and entry["message"]["type"] == "effect"]
            forwarded = [entry["message"] for entry in entries if entry["direction"] == "send" and entry["message"]["type"] == "call" and entry["message"]["agent"] == agent]
            assert effects == [{"tool":call["tool"], "arguments":call["arguments"]} for call in forwarded]
        assert [call["tool"] for call in executed if call["agent"] == agent] == expected_calls
        events = [event for event in result["simulator_events"] if event["agent"] == agent]
        assert [event["tool"] for event in events] == expected_calls
        if codebase:
            assert [event["success"] for event in events if event["tool"] == "run_local_tests"] == [False]*retries+[True]
    active = set()
    maximum = 0
    for entry in entries:
        if entry["direction"] not in {"domain_start", "domain_end"} or entry["message"]["tool"] != ("commit_changes" if codebase else "eat"):
            continue
        agent = entry["message"]["agent"]
        if entry["direction"] == "domain_start":
            assert agent not in active
            active.add(agent)
            maximum = max(maximum, len(active))
        else:
            active.remove(agent)
    assert not active
    return {"task": result["task"], "kind":result["kind"], "scored_benchmark":False,
            "audit_passed":True, "runtime":result["runtime"], "runtime_exit_code":result["runtime_exit_code"], "participants":len(expected), "domain_calls":len(executed),
            "scenario_depth":depth, "maximum_concurrent_resource_users":maximum, "total_tokens":total_tokens,
            "result_sha256":hashlib.sha256((directory / "result.json").read_bytes()).hexdigest(),
            "source_manifest": result["source_manifest"]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    print(json.dumps(audit(args.directory), indent=2))


if __name__ == "__main__":
    main()
