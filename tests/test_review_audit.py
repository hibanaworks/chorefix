"""Do not accept substituted feedback or feedback before the actual review."""
import copy
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
from audit_preflight import check_review_feedback


def transcript():
    implementation = {"id":1,"agent":"DEVELOPER_A","tool":"implement_feature", "arguments":{"feature_name":"user_authentication"}}
    review = {"id":2,"agent":"REVIEWER","tool":"code_review", "arguments":{"pr_id":"user_authentication"}}
    original = {"success":False,"message":"changes requested"}
    entries = [
        {"direction":"send","message":{"type":"simulator_result","id":1,"result":{"success":True}}},
        {"direction":"send","message":{"type":"simulator_result","id":2,"result":original}},
        {"direction":"receive","message":{"type":"tool_result","id":2,"result":original}},
        {"direction":"receive","message":{"type":"tool_result","id":1,"result":{"success":True,"coordination":{"code_review":copy.deepcopy(original)}}}},
        {"direction":"send","message":{"type":"model_finished","id":0,"agent":"REVIEWER"}},
    ]
    return entries, [implementation, review]


def test_audit_rejects_feedback_that_changes_the_original_review():
    entries, executed=transcript()
    entries[3]["message"]["result"]["coordination"]["code_review"]["success"]=True
    with pytest.raises(AssertionError):
        check_review_feedback(entries, executed)


def test_audit_rejects_feedback_reported_before_the_review_result_exists():
    entries, executed=transcript()
    entries[1], entries[3]=entries[3], entries[1]
    with pytest.raises(AssertionError):
        check_review_feedback(entries, executed)
