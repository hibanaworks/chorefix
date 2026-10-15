"""An unsupported request cannot be silently removed from experiment evidence."""
import pytest
from tools.audit_preflight import admitted_effects


def test_unknown_request_requires_its_actual_rejection_response():
    request = {"tool":"send_notification", "arguments":{"to":"TESTER"}}
    events = [{"direction":"receive", "message":{"type":"effect", "request":request}},
              {"direction":"send", "message":{"status":"ok"}}]
    with pytest.raises(AssertionError):
        admitted_effects(events, {}, "DEVELOPER", [])
    events[1]["message"] = {"status":"invalid_tool_arguments", "error":"'send_notification'"}
    rejected = []
    assert admitted_effects(events, {}, "DEVELOPER", rejected) == []
    assert rejected == [{"agent":"DEVELOPER", "request":request, "boundary":"tool_schema", "response":events[1]["message"]}]
