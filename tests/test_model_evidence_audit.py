"""Reject model provenance gaps even when an artifact's hashes are recomputed."""
import copy
import hashlib
import json

import pytest
from tools.audit_model_evidence import audit_participants


def evidence(tmp_path, mutate):
    usage = {"inputTokens": 10, "outputTokens": 3, "totalTokens": 13}
    request = {"model":"gpt-6-luna", "allowProviderModelFallback":False,
               "baseInstructions":"original task\nexperiment tools only", "dynamicTools":[{"name":"shell"}],
               "config":{"model_reasoning_effort":"xhigh", "features.shell_tool":False, "features.multi_agent":False}}
    messages = [
        {"direction":"send", "message":{"id":2, "method":"thread/start", "params":request}},
        {"direction":"receive", "message":{"id":2, "result":{"model":"gpt-6-luna", "reasoningEffort":"xhigh"}}},
        {"direction":"send", "message":{"id":3, "method":"turn/start", "params":{"model":"gpt-6-luna", "effort":"xhigh"}}},
        {"direction":"receive", "message":{"id":5, "method":"item/tool/call", "params":{"tool":"shell", "arguments":{"command":"exit 0"}}}},
        {"direction":"send", "message":{"id":5, "result":{"contentItems":[{"text":json.dumps({"exit_status":0})}]}}},
        {"direction":"receive", "message":{"method":"thread/tokenUsage/updated", "params":{"tokenUsage":{"total":usage}}}},
        {"direction":"receive", "message":{"method":"turn/completed", "params":{"turn":{"status":"completed", "error":None}}}},
    ]
    mutate(messages)
    directory = tmp_path / "AGENT"
    directory.mkdir()
    raw = directory / "events.jsonl"
    raw.write_text("\n".join(json.dumps(message) for message in messages)+"\n")
    (tmp_path / "AGENT-prompt.txt").write_text("original task")
    participant = {"participant":"AGENT", "status":"completed", "requested_model":"gpt-6-luna", "returned_model":"gpt-6-luna", "requested_effort":"xhigh", "returned_effort":"xhigh", "usage":usage, "tool_calls":1, "raw_events_sha256":hashlib.sha256(raw.read_bytes()).hexdigest()}
    io = [{"direction":"send", "message":{"type":"command", "agent":"AGENT", "command":"exit 0"}},
          {"direction":"receive", "message":{"type":"exit", "agent":"AGENT", "status":0}}]
    return audit_participants(tmp_path, {"participants":[participant]}, io, ["AGENT"])


def test_model_evidence_matches_commands_and_inclusive_tokens(tmp_path):
    assert evidence(tmp_path, lambda _: None) == {"participants":1, "shell_calls":1, "total_tokens":13}


@pytest.mark.parametrize("mutation", [
    lambda messages: messages.pop(1),
    lambda messages: messages[1]["message"]["result"].update(model="gpt-6.1-sol"),
    lambda messages: messages[-1]["message"]["params"]["turn"].update(status="failed"),
    lambda messages: messages.insert(4, copy.deepcopy(messages[3])),
    lambda messages: messages[-2]["message"]["params"]["tokenUsage"]["total"].update(totalTokens=14),
    lambda messages: messages[0]["message"]["params"].update(baseInstructions="different task"),
])
def test_model_evidence_rejects_mutated_provenance(tmp_path, mutation):
    with pytest.raises((AssertionError, ValueError)):
        evidence(tmp_path, mutation)
