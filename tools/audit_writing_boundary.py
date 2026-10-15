#!/usr/bin/env python3
"""Replay deterministic native writing traces. This does not qualify WASI I/O."""
import argparse
import asyncio
import gzip
import hashlib
import importlib
import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from benchmark.tools import load_tools


async def audit(directory):
    case = json.loads((directory / "case.json").read_text())
    task = case["task"]
    assert task in {"2E", "2M", "2H"}
    assert case["kind"] == "deterministic_native_boundary"
    assert case["model_calls"] == 0 and case["scored_benchmark"] is False
    log = gzip.decompress((directory / "bridge-events.jsonl.gz").read_bytes())
    assert hashlib.sha256(log).hexdigest() == case["events_sha256"]
    entries = [json.loads(line) for line in log.splitlines()]
    metadata = json.loads((ROOT / f"vendor/tracefix/benchmark/descriptions/{task}/metadata.json").read_text())
    module = importlib.import_module(f"benchmark.environments.{task}.sim")
    manifest = json.loads((ROOT / "benchmark/upstream-manifest.json").read_text())
    for name, digest in manifest["simulator_files"].items():
        assert hashlib.sha256((ROOT / "vendor/tracefix" / name).read_bytes()).hexdigest() == digest
    assert hashlib.sha256(Path(module.__file__).read_bytes()).hexdigest() == case["simulator_source_sha256"]
    sim = module.ResearchPaperSim()
    sim._seed = 0
    sim._delay_multiplier = 0.01
    sim.set_scenario_depth(2)
    registry = load_tools(task, sim=sim)
    requests, executions, replies, delivered = {}, {}, {}, {}
    active, starts, verdicts = {}, [], []
    maximum = 0
    for entry in entries:
        direction, message = entry["direction"], entry["message"]
        if direction == "send" and message["type"] == "call":
            assert message["id"] not in requests
            requests[message["id"]] = message
        elif direction == "receive" and message["type"] == "execute":
            identifier = message["id"]
            assert identifier not in executions
            assert message == {**requests[identifier], "type":"execute"}
            executions[identifier] = message
        elif direction == "domain_start":
            identifier = message["id"]
            assert identifier not in active and identifier not in replies
            assert message == executions[identifier]
            resources = set(sim.resource_requirements(message["tool"], agent_id=message["agent"], **message["arguments"]))
            assert not any(resources & held for held in active.values())
            active[identifier] = resources
            maximum = max(maximum, sum(bool(held) for held in active.values()))
            starts.append(message)
        elif direction == "domain_end":
            assert message == executions[message["id"]]
            del active[message["id"]]
        elif direction == "send" and message["type"] == "simulator_result":
            identifier = message["id"]
            assert identifier not in active and identifier not in replies
            replies[identifier] = message["result"]
        elif direction == "receive" and message["type"] == "tool_result":
            identifier = message["id"]
            assert identifier not in delivered
            assert message["result"] == replies[identifier]
            delivered[identifier] = message["result"]
        elif direction == "receive" and message["type"] == "run_result":
            verdicts.append(message)
    assert not active and set(requests) == set(executions) == set(replies) == set(delivered)
    assert {call["id"] for call in starts} == set(executions)
    assert verdicts == [{"type":"run_result", "status":"protocol_completed"}]
    assert len(starts) == 4 * len(metadata["agents"])
    for agent in metadata["agents"]:
        calls = [call for call in starts if call["agent"] == agent]
        assert [call["tool"] for call in calls] == ["read_literature", "draft_content", "write_to_document", "review_own_work"]
    for call in starts:
        replayed = (await registry.call(call["tool"], agent_id=call["agent"], **call["arguments"])).to_dict()
        recorded = replies[call["id"]]
        lo, hi = sim.tool_delay(call["tool"], **call["arguments"])
        assert math.isfinite(recorded["delay_seconds"])
        assert lo * 0.01 - 0.0005 <= recorded["delay_seconds"] <= hi * 0.01 + 0.0005
        assert {k:v for k,v in replayed.items() if k != "delay_seconds"} == {k:v for k,v in recorded.items() if k != "delay_seconds"}
    assert sim.is_complete() and not sim.violations
    return {**case, "audit_passed":True, "domain_calls":len(starts),
            "maximum_overlapping_resource_users":maximum,
            "resource_intervals":"no overlap on a shared section",
            "original_simulator_replay":"passed", "wasi_boundary_qualified":False}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    print(json.dumps(asyncio.run(audit(parser.parse_args().directory)), indent=2))
