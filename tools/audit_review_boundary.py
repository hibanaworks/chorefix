"""Post-run checks for deterministic 6E boundary traces, never task authority."""
import argparse
import asyncio
import hashlib
import importlib
import json
import math
from pathlib import Path

from benchmark.tools import load_tools


def require(condition, message):
    if not condition:
        raise ValueError(message)


async def audit_case(directory, binary):
    case = json.loads((directory / "case.json").read_text())
    require(case["task"] == "6E" and case["kind"] == "deterministic_boundary_test", "wrong evidence kind")
    require(case["model_calls"] == 0 and case["scored_benchmark"] is False, "boundary evidence cannot be scored")
    require(hashlib.sha256(binary.read_bytes()).hexdigest() == case["runtime_binary_sha256"], "binary changed")
    module = importlib.import_module("benchmark.environments.6E.sim")
    require(hashlib.sha256(Path(module.__file__).read_bytes()).hexdigest() == case["simulator_source_sha256"], "simulator changed")
    events = [json.loads(line) for line in (directory / "runtime/events.jsonl").read_text().splitlines()]
    pending, starts, results, frames, received, verdicts = {}, [], {}, {}, set(), []
    sim = module.SharedUtilityFileSim()
    sim._seed = 0
    sim._delay_multiplier = 0.01
    sim.set_scenario_depth(case["depth"])
    registry = load_tools("6E", sim=sim)
    for event in events:
        message = event["message"]
        if event["direction"] == "domain_start":
            identifier = message["id"]
            require(identifier not in pending and identifier not in results, "reused domain RPC")
            resources = set(sim.resource_requirements(message["tool"], agent_id=message["agent"], **message["arguments"]))
            require(not any(resources & held for _, held in pending.values()), "overlapping resource use")
            pending[identifier] = (message, resources)
            starts.append(message)
        elif event["direction"] == "domain_end":
            identifier = message["id"]
            require(identifier in pending, "domain result without request")
            request, _ = pending.pop(identifier)
            require(all(message[key] == request[key] for key in ("agent", "arguments", "tool")), "domain identity changed")
            require(message["result"]["tool_name"] == request["tool"], "wrong domain result")
            results[identifier] = message["result"]
        elif event["direction"] == "receive" and message["type"] == "protocol_frame":
            identifier = message["frame_id"]
            identity = {key: value for key, value in message.items() if key != "action"}
            if message["action"] == "send":
                require(identifier not in frames, "duplicate frame creation")
                frames[identifier] = identity
            else:
                require(identifier in frames and frames[identifier] == identity, "frame identity changed")
                if message["action"] == "receive":
                    received.add(identifier)
        elif event["direction"] == "receive" and message["type"] == "run_result":
            verdicts.append(message)
    require(not pending and set(results) == {message["id"] for message in starts}, "incomplete domain RPC")
    require(verdicts == [{"type": "run_result", "status": "complete"}], "runtime did not complete")
    require(received == set(frames), "undelivered accepted frame")
    require(len(starts) == 6 + 2 * case["depth"], "wrong number of domain operations")
    reviews = {"user_authentication": [], "notification_feature": []}
    for request in starts:
        recorded = results[request["id"]]
        replayed = (await registry.call(request["tool"], agent_id=request["agent"], **request["arguments"])).to_dict()
        # Randomly sampled delay is an observation, not a domain outcome.
        lo, hi = sim.tool_delay(request["tool"], **request["arguments"])
        delay = recorded["delay_seconds"]
        require(isinstance(delay, (int, float)) and math.isfinite(delay)
                and lo * 0.01 - 0.0005 <= delay <= hi * 0.01 + 0.0005,
                "delay outside the original simulator range")
        require({key: value for key, value in replayed.items() if key != "delay_seconds"}
                == {key: value for key, value in recorded.items() if key != "delay_seconds"},
                "original simulator replay differs")
        if request["tool"] == "code_review":
            reviews[request["arguments"]["pr_id"]].append(recorded["success"])
    require(reviews["user_authentication"] == [False] * case["depth"] + [True], "wrong revision history")
    require(reviews["notification_feature"] == [True], "independent feature changed")
    require(sim.is_complete() and not sim.violations and all(sim.progress["merges"].values()), "original task incomplete or violated")
    return {**case, "status": "passed", "domain_operations": len(starts), "accepted_frames": len(frames), "observations": len(events), "original_simulator_replay": "passed", "resource_intervals": "no overlap"}


async def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--binary", type=Path, required=True)
    args = parser.parse_args()
    cases = sorted(path for path in args.directory.iterdir() if path.is_dir() and not path.is_symlink() and (path / "case.json").is_file())
    require(len(cases) == 3, "expected all three boundary conditions")
    results = [await audit_case(path, args.binary) for path in cases]
    require(sorted(case["depth"] for case in results) == [0, 1, 2], "missing boundary condition")
    print(json.dumps({"status": "passed", "kind": "deterministic_boundary_tests", "scored_benchmark": False, "model_calls": 0, "cases": results}, indent=2))


if __name__ == "__main__":
    asyncio.run(main())
