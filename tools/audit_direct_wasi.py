#!/usr/bin/env python3
"""Check persisted direct-WASI preflight evidence, without running a model."""

import argparse
import hashlib
import json
from pathlib import Path


def audit_protocol_path(entries):
    """Inspect carrier observations only. This never drives a live Endpoint."""
    frames = {}
    observations = []
    for index, entry in enumerate(entries):
        message = entry["message"]
        if entry["direction"] != "receive" or message["type"] != "protocol_frame":
            continue
        observations.append((index, message))
        identifier = message["frame_id"]
        content = {key: message[key] for key in (
            "session", "lane", "source", "target", "frame_label", "payload",
        )}
        assert content["session"] == 16
        action = message["action"]
        if action == "send":
            assert identifier == len(frames)
            frames[identifier] = (content, "queued")
        else:
            original, location = frames[identifier]
            assert original == content
            if action == "receive":
                assert location == "queued"
                frames[identifier] = (original, "received")
            else:
                assert action == "requeue" and location == "received"
                frames[identifier] = (original, "queued")
    assert frames and all(location == "received" for _, location in frames.values())
    calls = [(i, entry["message"]) for i, entry in enumerate(entries)
             if entry["direction"] == "receive" and entry["message"]["type"] == "execute"]
    assert [call["agent"] for _, call in calls] == ["DEVELOPER", "TESTER", "DEPLOYER"]
    paths = []
    for vm, (index, call) in enumerate(calls):
        owner = vm + 3
        # FdCloseReq encodes its one fd byte. Other requests have larger payloads.
        close_index, close = [(i, frame) for i, frame in observations if i < index
                              and frame["action"] == "receive"
                              and (frame["source"], frame["target"]) == (vm, owner)][-1]
        assert close["payload"] == [4]
        grant_index, _ = [(i, frame) for i, frame in observations if i < close_index
                          and frame["action"] == "receive"
                          and (frame["source"], frame["target"]) == (6 if vm < 2 else 7, owner)][-1]
        assert grant_index < close_index < index
        (end_index, end), = [(i, entry["message"]) for i, entry in enumerate(entries)
                            if entry["direction"] == "domain_end"
                            and entry["message"]["id"] == call["id"]]
        assert end_index > index and end["result"]["success"] is True
        reply_index, reply = [(i, frame) for i, frame in observations if i > end_index
                              and frame["action"] == "send"
                              and (frame["source"], frame["target"]) == (owner, vm)
                              and bytes(frame["payload"]).startswith(b'{')][0]
        assert json.loads(bytes(reply["payload"])) == end["result"]
        received_index, = [i for i, frame in observations if i > reply_index
                           and frame["frame_id"] == reply["frame_id"]
                           and frame["action"] == "receive"]
        assert received_index > reply_index
        paths.append({"agent":call["agent"], "wasi_role":vm, "owner_role":owner,
                      "close_frame":close["frame_id"], "reply_frame":reply["frame_id"]})
    notifications = []
    for source, target in ((3, 4), (4, 5), (5, 3), (5, 4)):
        (sent_index, sent), = [(i, frame) for i, frame in observations
                              if frame["action"] == "send"
                              and (frame["source"], frame["target"]) == (source, target)]
        received_index, = [i for i, frame in observations
                           if frame["action"] == "receive" and frame["frame_id"] == sent["frame_id"]]
        assert sent["payload"] == [] and sent_index < received_index
        producing_call = calls[source - 3][1]
        producing_end, = [i for i, entry in enumerate(entries)
                          if entry["direction"] == "domain_end"
                          and entry["message"]["id"] == producing_call["id"]]
        assert producing_end < sent_index
        if source < 5:
            assert received_index < calls[target - 3][0]
        notifications.append({"source":source, "target":target, "frame":sent["frame_id"]})
    return {"frames":len(frames), "observations":len(observations),
            "paths":paths, "notifications":notifications}


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
    protocol_path = audit_protocol_path(entries)
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
            "protocol_path":protocol_path,
            "shell_calls": calls, "total_tokens": tokens,
            "elapsed_seconds": result["elapsed_seconds"],
            "result_sha256": hashlib.sha256((directory / "result.json").read_bytes()).hexdigest(),
            "binary_sha256": result["binary_sha256"], "versions": result["versions"]}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    print(json.dumps(audit(parser.parse_args().directory), indent=2))
