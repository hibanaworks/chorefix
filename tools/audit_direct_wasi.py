#!/usr/bin/env python3
"""Check persisted direct-WASI preflight evidence, without running a model."""

import argparse
import hashlib
import json
from pathlib import Path


def audit(directory):
    result = json.loads((directory / "result.json").read_text())
    assert result["kind"] == "direct_wasi_integration_preflight"
    assert result["task"] == "16E" and result["scored_benchmark"] is False
    assert result["error"] is None
    assert result["protocol"] == {"type": "run_result", "status": "complete"}
    assert result["simulator_complete"] is True and not result["simulator_violations"]
    assert result["runtime"] == "tokio_current_thread" and result["runtime_exit_code"] == 0
    for name, digest in result["binary_sha256"].items():
        assert hashlib.sha256((directory / name).read_bytes()).hexdigest() == digest
    manifest = json.loads((directory / "source-manifest.json").read_text())
    assert manifest == result["source_manifest"]
    for relative, digest in manifest.items():
        assert hashlib.sha256((directory / "source" / relative).read_bytes()).hexdigest() == digest
    upstream = json.loads((directory / "source/benchmark/upstream-manifest.json").read_text())
    assert result["versions"]["tracefix"] == upstream["upstream_revision"]
    for task in upstream["tasks"]:
        for relative, digest in task["files"].items():
            assert manifest[f"vendor/tracefix/{relative}"] == digest
    for relative, digest in upstream["simulator_files"].items():
        assert manifest[f"vendor/tracefix/{relative}"] == digest
    log = directory / "io/events.jsonl"
    assert hashlib.sha256(log.read_bytes()).hexdigest() == result["io_events_sha256"]
    entries = [json.loads(line) for line in log.read_text().splitlines()]
    executed = [(index, entry["message"]) for index, entry in enumerate(entries)
                if entry["direction"] == "receive" and entry["message"]["type"] == "execute"]
    assert [call["tool"] for _, call in executed] == [
        "compile_and_test", "run_integration_tests", "deploy_application",
    ]
    assert [call["agent"] for _, call in executed] == ["DEVELOPER", "TESTER", "DEPLOYER"]
    assert len({call["id"] for _, call in executed}) == 3
    for number, (index, call) in enumerate(executed):
        start, = [entry["message"] for entry in entries if entry["direction"] == "domain_start"
                  and entry["message"]["id"] == call["id"]]
        end, = [(position, entry["message"]) for position, entry in enumerate(entries)
                if entry["direction"] == "domain_end" and entry["message"]["id"] == call["id"]]
        response, = [entry["message"] for entry in entries if entry["direction"] == "send"
                     and entry["message"]["type"] == "simulator_result"
                     and entry["message"]["id"] == call["id"]]
        assert start == call and end[0] > index
        assert end[1] == {**call, "result": response["result"]}
        assert response["result"]["success"] is True
        if number < 2:
            assert end[0] < executed[number + 1][0]
        event = result["simulator_events"][number]
        assert (event["agent"], event["tool"], event["args"], event["success"]) == (
            call["agent"], call["tool"], call["arguments"], True,
        )
    assert len(result["simulator_events"]) == 3
    assert len([entry for entry in entries if entry["direction"] == "domain_start"]) == 3
    assert len([entry for entry in entries if entry["direction"] == "domain_end"]) == 3
    participants = result["participants"]
    assert {value["participant"] for value in participants} == {"DEVELOPER", "TESTER", "DEPLOYER"}
    assert len(participants) == 3
    tokens = 0
    calls = 0
    for participant in participants:
        agent = participant["participant"]
        assert participant["status"] == "completed"
        assert participant["requested_model"] == participant["returned_model"] == "gpt-6-luna"
        assert participant["requested_effort"] == participant["returned_effort"] == "xhigh"
        raw = directory / agent / "events.jsonl"
        assert hashlib.sha256(raw.read_bytes()).hexdigest() == participant["raw_events_sha256"]
        commands = []
        tool_replies = {}
        usage = None
        for line in raw.read_text().splitlines():
            entry = json.loads(line)
            message = entry["message"]
            if message.get("method") == "thread/start":
                params = message["params"]
                assert params["model"] == "gpt-6-luna"
                assert params["allowProviderModelFallback"] is False
                assert params["config"]["model_reasoning_effort"] == "xhigh"
                assert params["config"]["features.shell_tool"] is False
                assert params["config"]["features.multi_agent"] is False
            if entry["direction"] == "receive" and message.get("id") == 2 and "result" in message and "method" not in message:
                assert message["result"]["model"] == "gpt-6-luna"
                assert message["result"]["reasoningEffort"] == "xhigh"
            if message.get("method") == "item/tool/call":
                assert message["params"]["tool"] == "shell"
                commands.append((message["id"], message["params"]["arguments"]["command"]))
            if entry["direction"] == "send" and "contentItems" in message.get("result", {}):
                tool_replies[message["id"]] = json.loads(message["result"]["contentItems"][0]["text"])
            if message.get("method") == "thread/tokenUsage/updated":
                usage = message["params"]["tokenUsage"]["total"]
            item = message.get("params", {}).get("item", {})
            assert item.get("type") in {None, "dynamicToolCall", "reasoning", "userMessage", "agentMessage"}
        assert participant["tool_calls"] == len(commands)
        assert participant["usage"] == usage and usage["totalTokens"] > 0
        sent = [entry["message"]["command"] for entry in entries
                if entry["direction"] == "send" and entry["message"]["type"] == "command"
                and entry["message"]["agent"] == agent]
        assert sent == [command for identifier, command in commands if len(command.encode()) <= 4096
                        and tool_replies[identifier].get("status") != "runtime_closed"]
        exits = [entry["message"] for entry in entries if entry["direction"] == "receive"
                 and entry["message"]["type"] == "exit" and entry["message"]["agent"] == agent]
        assert len(exits) == 1 and exits[0]["status"] == 0
        tokens += usage["totalTokens"]
        calls += len(commands)
    return {"kind": result["kind"], "task": "16E", "scored_benchmark": False,
            "audit_passed": True, "participants": 3, "domain_calls": 3,
            "shell_calls": calls, "total_tokens": tokens,
            "elapsed_seconds": result["elapsed_seconds"],
            "result_sha256": hashlib.sha256((directory / "result.json").read_bytes()).hexdigest(),
            "binary_sha256": result["binary_sha256"], "versions": result["versions"]}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    print(json.dumps(audit(parser.parse_args().directory), indent=2))
