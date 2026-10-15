"""Read-only source and model evidence checks shared by task-specific audits."""
import hashlib
import json


def audit_sources(directory, result):
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


def audit_participants(directory, result, entries, expected_agents):
    participants = result["participants"]
    assert {value["participant"] for value in participants} == set(expected_agents)
    assert len(participants) == len(expected_agents)
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
        events = [json.loads(line) for line in raw.read_text().splitlines()]
        thread_requests = [entry["message"] for entry in events if entry["direction"] == "send" and entry["message"].get("method") == "thread/start"]
        thread_request, = thread_requests
        prompt = (directory / f"{agent}-prompt.txt").read_text()
        assert thread_request["params"]["baseInstructions"].startswith(prompt + "\n")
        assert [tool["name"] for tool in thread_request["params"]["dynamicTools"]] == ["shell"]
        thread_response, = [entry["message"]["result"] for entry in events if entry["direction"] == "receive" and entry["message"].get("id") == thread_request["id"] and "result" in entry["message"] and "method" not in entry["message"]]
        assert thread_response["model"] == "gpt-6-luna" and thread_response["reasoningEffort"] == "xhigh"
        turn_request, = [entry["message"] for entry in events if entry["direction"] == "send" and entry["message"].get("method") == "turn/start"]
        assert turn_request["params"]["model"] == "gpt-6-luna" and turn_request["params"]["effort"] == "xhigh"
        turn, = [entry["message"]["params"]["turn"] for entry in events if entry["direction"] == "receive" and entry["message"].get("method") == "turn/completed"]
        assert turn["status"] == participant["status"] and turn.get("error") is None
        for entry in events:
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
        assert len({identifier for identifier, _ in commands}) == len(commands)
        assert participant["usage"] == usage and usage["totalTokens"] > 0
        assert usage["totalTokens"] == usage["inputTokens"] + usage["outputTokens"]
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
    return {"participants": len(participants), "shell_calls": calls, "total_tokens": tokens}
