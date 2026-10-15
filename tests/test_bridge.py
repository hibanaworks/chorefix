"""Check the actual process boundary before connecting an external simulator."""

import json
import select
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read_reply(process):
    assert select.select([process.stdout], [], [], 5)[0], "runtime did not respond"
    return json.loads(process.stdout.readline())


def test_early_eat_never_reaches_simulator():
    process = subprocess.Popen(
        [str(ROOT / "target/debug/philosophers"), "9E"],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        text=True,
    )
    try:
        process.stdin.write(json.dumps({"type": "call", "id": 1,
                                       "agent": "PHILOSOPHER_A", "tool": "eat",
                                       "arguments": {"philosopher": "PHILOSOPHER_A"}})+"\n")
        process.stdin.flush()
        reply = read_reply(process)
        # No 'execute' RPC was published. This is the endpoint's rejection,
        # not a simulator result or a separate expected-step check in Python.
        assert reply["type"] == "run_result"
        assert reply["status"] == "failed"
        assert "Endpoint" in reply["error"]
        assert "LabelMismatch" in reply["error"]
    finally:
        process.stdin.close()
        if process.poll() is None:
            process.terminate()
        process.wait(timeout=5)


def test_duplicate_think_cannot_execute_a_second_domain_operation():
    process = subprocess.Popen([str(ROOT / "target/debug/philosophers"), "9E"],
                               stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
    def send(value):
        process.stdin.write(json.dumps(value)+"\n")
        process.stdin.flush()
    try:
        call = {"type":"call", "id":1, "agent":"PHILOSOPHER_A", "tool":"think",
                "arguments":{"philosopher":"PHILOSOPHER_A"}}
        send(call)
        assert read_reply(process)["type"] == "execute"
        send({"type":"simulator_result", "id":1, "result":{"success":True}})
        assert read_reply(process)["type"] == "tool_result"
        send({**call, "id":2})
        reply = read_reply(process)
        assert reply["type"] == "run_result"
        assert reply["status"] == "failed"
        assert "Endpoint" in reply["error"]
        assert "LabelMismatch" in reply["error"]
    finally:
        process.stdin.close()
        if process.poll() is None:
            process.terminate()
        process.wait(timeout=5)


def test_closed_participant_input_wakes_waiter_and_fails_run():
    process = subprocess.Popen([str(ROOT / "target/debug/philosophers"), "9E"],
                               stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
    try:
        process.stdin.write(json.dumps({"type":"close_role", "id":0, "agent":"PHILOSOPHER_A"})+"\n")
        process.stdin.flush()
        reply = read_reply(process)
        assert reply["type"] == "run_result"
        assert reply["status"] == "failed"
        assert "closed" in reply["error"]
    finally:
        process.stdin.close()
        if process.poll() is None:
            process.terminate()
        process.wait(timeout=5)
