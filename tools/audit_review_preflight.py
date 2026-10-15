#!/usr/bin/env python3
"""Audit saved 6E Luna/WASI evidence and replay the unchanged original simulator."""
import argparse
import asyncio
import dataclasses
import hashlib
import importlib
import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from benchmark.tools import load_tools
from tools.audit_model_evidence import audit_participants, audit_sources


async def audit(directory):
    result = json.loads((directory / "result.json").read_text())
    assert result["kind"] == "direct_wasi_integration_preflight"
    assert result["task"] == "6E" and result["scored_benchmark"] is False
    assert result["error"] is None
    assert result["protocol"] == {"type": "run_result", "status": "complete"}
    assert result["runtime"] == "tokio_current_thread" and result["runtime_exit_code"] == 0
    assert result["simulator_complete"] is True and not result["simulator_violations"]
    audit_sources(directory, result)
    metadata = json.loads((directory / "source/vendor/tracefix/benchmark/descriptions/6E/metadata.json").read_text())
    config = json.loads((directory / "source/experiment.json").read_text())
    manifest = result["source_manifest"]
    upstream = json.loads((directory / "source/benchmark/upstream-manifest.json").read_text())
    # The replay below imports the original local library. Its entire saved
    # simulator inventory must still match before it can supply evidence.
    for name, digest in upstream["simulator_files"].items():
        assert hashlib.sha256((ROOT / "vendor/tracefix" / name).read_bytes()).hexdigest() == digest
    log = directory / "io/events.jsonl"
    assert hashlib.sha256(log.read_bytes()).hexdigest() == result["io_events_sha256"]
    entries = [json.loads(line) for line in log.read_text().splitlines()]
    model = audit_participants(directory, result, entries, metadata["agents"])
    module = importlib.import_module("benchmark.environments.6E.sim")
    assert hashlib.sha256(Path(module.__file__).read_bytes()).hexdigest() == manifest["vendor/tracefix/benchmark/environments/6E/sim.py"]
    sim = module.SharedUtilityFileSim()
    sim._seed = config["python_hash_seed"]
    sim._delay_multiplier = config["domain_tool_delay_multiplier"]
    sim.set_scenario_depth(result["scenario_depth"])
    registry = load_tools("6E", sim=sim)
    description = (directory / "source/vendor/tracefix/benchmark/descriptions/6E/description.md").read_text()
    for agent in metadata["agents"]:
        prompt = (directory / f"{agent}-prompt.txt").read_text()
        assert description in prompt and f"You are {agent}." in prompt
        assert json.dumps([entry["function"] for entry in registry.openai_schemas(agent)]) in prompt
    executed, pending, responses, ends, frames = {}, {}, {}, {}, {}
    starts, observations, verdicts = [], [], []
    for index, entry in enumerate(entries):
        message = entry["message"]
        if entry["direction"] == "receive" and message["type"] == "protocol_frame":
            assert message["session"] == 6
            identifier = message["frame_id"]
            identity = {key: value for key, value in message.items() if key != "action"}
            if message["action"] == "send":
                assert identifier not in frames
                frames[identifier] = (identity, "queued")
            else:
                original, location = frames[identifier]
                assert original == identity
                if message["action"] == "receive":
                    assert location == "queued"
                    frames[identifier] = (identity, "received")
                else:
                    assert message["action"] == "requeue" and location == "received"
                    frames[identifier] = (identity, "queued")
            observations.append((index, message))
        elif entry["direction"] == "receive" and message["type"] == "execute":
            assert message["id"] not in executed
            executed[message["id"]] = message
        elif entry["direction"] == "domain_start":
            identifier = message["id"]
            assert executed[identifier] == message and identifier not in pending
            assert identifier not in responses
            resources = set(sim.resource_requirements(message["tool"], agent_id=message["agent"], **message["arguments"]))
            assert not any(resources & held for _, held in pending.values())
            pending[identifier] = (message, resources)
            starts.append((index, message))
        elif entry["direction"] == "domain_end":
            request, _ = pending.pop(message["id"])
            assert message == {**request, "result": message["result"]}
            responses[message["id"]] = message["result"]
            ends[message["id"]] = index
        elif entry["direction"] == "send" and message["type"] == "simulator_result":
            assert responses[message["id"]] == message["result"]
        elif entry["direction"] == "receive" and message["type"] == "run_result":
            verdicts.append(message)
    assert not pending and set(executed) == set(responses)
    assert frames and all(location == "received" for _, location in frames.values())
    assert verdicts == [{"type": "run_result", "status": "complete"}]
    assert len(starts) == 6 + 2 * result["scenario_depth"]
    used_closes = set()
    paths = []
    for start, request in starts:
        original = responses[request["id"]]
        replayed = (await registry.call(request["tool"], agent_id=request["agent"], **request["arguments"])).to_dict()
        lo, hi = sim.tool_delay(request["tool"], **request["arguments"])
        delay = original["delay_seconds"]
        assert isinstance(delay, (int, float)) and math.isfinite(delay)
        assert lo * sim._delay_multiplier - 0.0005 <= delay <= hi * sim._delay_multiplier + 0.0005
        assert {key: value for key, value in replayed.items() if key != "delay_seconds"} == {key: value for key, value in original.items() if key != "delay_seconds"}
        if request["agent"] == "REVIEWER":
            target = request["arguments"]["pr_id"]
            owner, fd = (5, 4) if target == "user_authentication" else (6, 16)
            caller = 2
        else:
            owner, fd = 7, 4
            caller = 3 if request["agent"] == "DEVELOPER_A" else 4
        close_index, close = [(i, frame) for i, frame in observations if i < start
                              and frame["action"] == "receive"
                              and (frame["source"], frame["target"], frame["payload"]) == (caller, owner, [fd])][-1]
        assert close["frame_id"] not in used_closes
        used_closes.add(close["frame_id"])
        if owner == 7:
            vm = caller - 3
            vm_close_index, vm_close = [(i, frame) for i, frame in observations if i < close_index
                                       and frame["action"] == "receive"
                                       and (frame["source"], frame["target"], frame["payload"]) == (vm, caller, [fd])][-1]
            assert vm_close_index < close_index < start
        paths.append({"rpc":request["id"], "caller":caller, "owner":owner, "close_frame":close["frame_id"]})
    assert sim.is_complete() and not sim.violations
    assert all(sim.progress["merges"].values()) and sim.progress == result["simulator_progress"]
    # Domain timestamps and sampled delays are observations, not protocol state.
    replay_events = [dataclasses.asdict(event) for event in sim.events]
    for recorded, replayed in zip(result["simulator_events"], replay_events, strict=True):
        assert {key: value for key, value in recorded.items() if key != "timestamp"} == {key: value for key, value in replayed.items() if key != "timestamp"}
    reviews = [request for _, request in starts if request["tool"] == "code_review"]
    assert [responses[request["id"]]["success"] for request in reviews] == [False] * result["scenario_depth"] + [True, True]
    for agent, feature, branch, local, vm in [("DEVELOPER_A", "user_authentication", "dev_a_feature", 3, 0), ("DEVELOPER_B", "notification_feature", "dev_b_feature", 4, 1)]:
        implementations = [(i, call) for i, call in starts if call["agent"] == agent and call["tool"] == "implement_feature"]
        inspected = [(i, call) for i, call in starts if call["tool"] == "code_review" and call["arguments"]["pr_id"] == feature]
        merge, = [(i, call) for i, call in starts if call["agent"] == agent and call["tool"] == "merge_code"]
        assert merge[1]["arguments"] == {"branch":branch}
        assert len(implementations) == len(inspected)
        feedback = []
        for i, frame in observations:
            if frame["action"] == "send" and (frame["source"], frame["target"]) == (local, vm) and bytes(frame["payload"]).startswith(b'{'):
                value = json.loads(bytes(frame["payload"]))
                if "coordination" in value:
                    feedback.append((i, value))
        assert len(feedback) == len(inspected)
        for visit, ((_, implementation), (review_start, review), (feedback_index, value)) in enumerate(zip(implementations, inspected, feedback, strict=True)):
            assert implementation["arguments"] == {"feature_name":feature}
            assert ends[implementation["id"]] < review_start < ends[review["id"]] < feedback_index
            assert value["coordination"] == {"code_review":responses[review["id"]]}
            assert {key: item for key, item in value.items() if key != "coordination"} == responses[implementation["id"]]
            assert responses[review["id"]]["success"] is (visit == len(inspected)-1)
            if visit:
                assert ends[inspected[visit-1][1]["id"]] < implementations[visit][0]
        assert ends[inspected[-1][1]["id"]] < merge[0]
    return {"audit_passed":True, "task":"6E", "kind":result["kind"], "scored_benchmark":False,
            "scenario_depth":result["scenario_depth"], **model,
            "domain_calls":len(starts), "protocol_frames":len(frames),
            "protocol_paths":paths, "original_simulator_replay":"passed",
            "elapsed_seconds":result["elapsed_seconds"], "versions":result["versions"],
            "binary_sha256":result["binary_sha256"],
            "result_sha256":hashlib.sha256((directory / "result.json").read_bytes()).hexdigest()}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    print(json.dumps(asyncio.run(audit(parser.parse_args().directory)), indent=2))
