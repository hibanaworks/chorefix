#!/usr/bin/env python3
"""Read saved raw events and check the integration preflight's claimed outcomes."""

import argparse
import hashlib
import json
import jsonschema
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def admitted_effects(events, schemas, agent, rejected):
    """Account for every request, including schema errors returned before RPC."""
    admitted = []
    for index, entry in enumerate(events):
        message = entry["message"]
        if entry["direction"] != "receive" or message.get("type") != "effect":
            continue
        request = message["request"]
        try:
            jsonschema.validate(request["arguments"], schemas[request["tool"]]["parameters"])
        except (KeyError, jsonschema.ValidationError) as failure:
            # Invalid requests must have the exact validation response and must
            # be absent from forwarded RPCs (checked separately below).
            reply = events[index+1]
            assert reply["direction"] == "send"
            assert reply["message"] == {"status":"invalid_tool_arguments", "error":str(failure)}
            rejected.append({"agent":agent, "request":request,
                             "boundary":"tool_schema", "response":reply["message"]})
        else:
            admitted.append(request)
    return admitted


def check_review_feedback(entries, executed):
    # Verify received feedback against the separate original simulator
    # responses, including values and its position before the model reply.
    simulator_replies = {entry["message"]["id"]:entry["message"]["result"] for entry in entries
        if entry["direction"] == "send" and entry["message"]["type"] == "simulator_result"}
    model_replies = {entry["message"]["id"]:entry["message"]["result"] for entry in entries
        if entry["direction"] == "receive" and entry["message"]["type"] == "tool_result"}
    assert model_replies.keys() == simulator_replies.keys() == {call["id"] for call in executed}
    for call in executed:
        reply = dict(model_replies[call["id"]])
        if call["tool"] == "implement_feature":
            feedback = reply.pop("coordination")["code_review"]
            reviews = [candidate for candidate in executed if candidate["tool"] == "code_review"
                and candidate["arguments"]["pr_id"] == call["arguments"]["feature_name"]]
            implementations = [candidate for candidate in executed if candidate["tool"] == "implement_feature"
                and candidate["agent"] == call["agent"]]
            review = reviews[implementations.index(call)]
            assert feedback == simulator_replies[review["id"]]
            assert executed.index(call) < executed.index(review)
            received_index = next(index for index, entry in enumerate(entries) if entry["direction"] == "receive"
                and entry["message"]["type"] == "tool_result" and entry["message"]["id"] == call["id"])
            review_index = next(index for index, entry in enumerate(entries) if entry["direction"] == "send"
                and entry["message"]["type"] == "simulator_result" and entry["message"]["id"] == review["id"])
            assert review_index < received_index
        assert reply == simulator_replies[call["id"]]
    assert any(entry["direction"] == "send" and entry["message"]["type"] == "model_finished"
               and entry["message"]["agent"] == "REVIEWER" for entry in entries)


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
    # 8E explicitly ends after a failed test. Protocol completion and the
    # simulator's successful domain outcome are separate facts.
    if result["task"] == "8E":
        assert result["simulator_complete"] is (result["scenario_depth"] == 0)
    else:
        assert result["simulator_complete"] is True
    assert not result["simulator_violations"]
    assert result["runtime"] == "tokio_current_thread"
    assert result["runtime_exit_code"] == 0
    assert hashlib.sha256((directory / "runtime").read_bytes()).hexdigest() == result["runtime_binary_sha256"]
    saved_manifest = json.loads((directory / "source-manifest.json").read_text())
    assert saved_manifest == result["source_manifest"]
    for relative, digest in saved_manifest.items():
        assert hashlib.sha256((directory / "source" / relative).read_bytes()).hexdigest() == digest
    inputs = ROOT / "vendor/tracefix/benchmark/descriptions" / result["task"]
    expected = json.loads((inputs / "metadata.json").read_text())["agents"]
    tools = [entry["function"] for entry in json.loads((inputs / "tools.json").read_text())]
    schemas_by_agent = {agent:{tool["name"]:tool for tool in tools if agent in tool["agent_ids"]} for agent in expected}
    rejected = []
    accepted_by_agent = {}
    assert {agent["participant"] for agent in result["participants"]} == set(expected)
    total_tokens = 0
    family = result["task"][:-1]
    codebase = family == "1"
    depth = result["scenario_depth"]
    retries = depth if result["task"] == "1H" else 0
    expected_calls = {
        "1": ["design_feature"] + ["implement_code", "commit_changes", "run_local_tests"]*(retries+1),
        "2": ["read_literature", "draft_content", "write_to_document", "review_own_work"],
        "6": ["implement_feature", "code_review", "merge_code"],
        "8": ["migrate_schema", "implement_backend", "run_integration_tests"],
        "9": ["think", "eat"],
        "16": ["compile_and_test", "run_integration_tests", "deploy_application"],
    }[family]
    expected_by_agent = {agent:expected_calls for agent in expected}
    if family == "16":
        expected_by_agent = {"DEVELOPER":["compile_and_test"], "TESTER":["run_integration_tests"], "DEPLOYER":["deploy_application"]}
    if family == "8":
        expected_by_agent = {"DB_ADMIN":["migrate_schema"], "BACKEND_DEV":["implement_backend"], "TESTER":["run_integration_tests"]}
    if family == "6":
        reviews = [event for event in result["simulator_events"] if event["tool"] == "code_review"]
        assert [event["success"] for event in reviews] == [False]*depth+[True, True]
        expected_by_agent = {"REVIEWER":["code_review"]*(depth+2)}
        for agent, feature in [("DEVELOPER_A", "user_authentication"), ("DEVELOPER_B", "notification_feature")]:
            feature_reviews = [event for event in reviews if event["args"]["target"] == feature]
            assert feature_reviews and feature_reviews[-1]["success"] is True
            assert all(event["success"] is False for event in feature_reviews[:-1])
            expected_by_agent[agent] = ["implement_feature"]*len(feature_reviews)+["merge_code"]
    shared_tools = {"1":{"commit_changes"}, "2":{"write_to_document"}, "6":{"implement_feature", "merge_code"}, "8":{"migrate_schema", "run_integration_tests"}, "9":{"eat"}, "16":set(expected_calls)}[family]
    for agent in result["participants"]:
        expected_calls = expected_by_agent[agent["participant"]]
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
            effects = admitted_effects(events, schemas_by_agent[agent["participant"]], agent["participant"], rejected)
            accepted_by_agent[agent["participant"]] = effects
            assert [effect["tool"] for effect in effects] == expected_calls
            assert events[-1]["message"] == {"type":"exit", "status":0}
        else:
            assert calls == expected_calls
        total_tokens += agent["usage"]["totalTokens"]
    log = directory / "bridge-events.jsonl"
    assert hashlib.sha256(log.read_bytes()).hexdigest() == result["bridge_events_sha256"]
    entries = [json.loads(line) for line in log.read_text().splitlines()]
    executed = [entry["message"] for entry in entries if entry["direction"] == "receive" and entry["message"]["type"] == "execute"]
    assert len(executed) == sum(len(calls) for calls in expected_by_agent.values())
    assert len({call["id"] for call in executed}) == len(executed)
    if family == "6":
        check_review_feedback(entries, executed)
        simulator_replies = {entry["message"]["id"]:entry["message"]["result"] for entry in entries
            if entry["direction"] == "send" and entry["message"]["type"] == "simulator_result"}
        starts = {entry["message"]["id"]:index for index, entry in enumerate(entries) if entry["direction"] == "domain_start"}
        ends = {entry["message"]["id"]:index for index, entry in enumerate(entries) if entry["direction"] == "domain_end"}
        for agent, feature in [("DEVELOPER_A", "user_authentication"), ("DEVELOPER_B", "notification_feature")]:
            implementations = [call for call in executed if call["agent"] == agent and call["tool"] == "implement_feature"]
            reviews = [call for call in executed if call["tool"] == "code_review" and call["arguments"]["pr_id"] == feature]
            merge = [call for call in executed if call["agent"] == agent and call["tool"] == "merge_code"]
            assert len(implementations) == len(reviews) and len(merge) == 1
            for index, (implementation, review) in enumerate(zip(implementations, reviews)):
                assert ends[implementation["id"]] < starts[review["id"]]
                if index:
                    assert simulator_replies[reviews[index-1]["id"]]["success"] is False
                    assert ends[reviews[index-1]["id"]] < starts[implementation["id"]]
            assert simulator_replies[reviews[-1]["id"]]["success"] is True
            assert ends[reviews[-1]["id"]] < starts[merge[0]["id"]]
    for agent in expected:
        expected_calls = expected_by_agent[agent]
        if shell:
            events = [json.loads(line) for line in (directory / (agent + "-shell") / "events.jsonl").read_text().splitlines()]
            effects = accepted_by_agent[agent]
            forwarded = [entry["message"] for entry in entries if entry["direction"] == "send" and entry["message"]["type"] == "call" and entry["message"]["agent"] == agent]
            assert effects == [{"tool":call["tool"], "arguments":call["arguments"]} for call in forwarded]
        assert [call["tool"] for call in executed if call["agent"] == agent] == expected_calls
        events = [event for event in result["simulator_events"] if event["agent"] == agent]
        assert [event["tool"] for event in events] == expected_calls
        if codebase:
            assert [event["success"] for event in events if event["tool"] == "run_local_tests"] == [False]*retries+[True]
        if family == "8" and agent == "TESTER":
            assert [event["success"] for event in events] == [depth == 0]
    active = set()
    maximum = 0
    for entry in entries:
        if entry["direction"] not in {"domain_start", "domain_end"} or entry["message"]["tool"] not in shared_tools:
            continue
        agent = entry["message"]["agent"]
        if entry["direction"] == "domain_start":
            assert agent not in active
            active.add(agent)
            maximum = max(maximum, len(active))
        else:
            active.remove(agent)
    assert not active
    if family == "6":
        assert maximum == 1
    return {"task": result["task"], "kind":result["kind"], "scored_benchmark":False,
            "domain_complete":result["simulator_complete"],
            "audit_passed":True, "runtime":result["runtime"], "runtime_exit_code":result["runtime_exit_code"], "participants":len(expected), "domain_calls":len(executed),
            "scenario_depth":depth, "maximum_concurrent_resource_users":maximum, "total_tokens":total_tokens,
            "rejected_request_count":len(rejected), "rejected_requests":rejected,
            "result_sha256":hashlib.sha256((directory / "result.json").read_bytes()).hexdigest(),
            "source_manifest": result["source_manifest"]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    print(json.dumps(audit(args.directory), indent=2))


if __name__ == "__main__":
    main()
