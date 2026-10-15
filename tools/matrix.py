#!/usr/bin/env python3
"""Build the fixed full experiment matrix without issuing model calls."""

import collections
import itertools
import json
from pathlib import Path

from inventory import ROOT


def matrix(config: dict) -> list[dict]:
    if config["model"] != "gpt-6-luna" or config["reasoning_effort"] != "xhigh":
        raise ValueError("participant model or reasoning effort changed")
    conditions = {"topology_monitored", "mediator_enforced", "prompt_only", "chat_only", "hibana"}
    if len(config["conditions"]) != 5 or set(config["conditions"]) != conditions:
        raise ValueError("the experiment requires the four original paper conditions and Hibana")
    if config["scenario_depths"] != [0, 1, 2] or config["repetitions"] != 3:
        raise ValueError("fault conditions or repetitions changed")
    tasks = [f"{number}{level}" for number in range(1, 17) for level in "EMH"]
    runs = []
    for task, depth, repetition, condition in itertools.product(tasks, config["scenario_depths"], range(3), config["conditions"]):
        runs.append({
            "run_id": f"{task}-S{depth}-R{repetition+1}-{condition}",
            "task": task,
            "scenario_depth": depth,
            "repetition": repetition+1,
            "seed": repetition,
            "condition": condition,
        })
    if len(runs) != config["expected_runs"] or len(runs) != 2160:
        raise ValueError("matrix size does not match the selected experiment")
    if len({r["run_id"] for r in runs}) != len(runs):
        raise ValueError("duplicate run identity")
    return runs


def main():
    config = json.loads((ROOT / "experiment.json").read_text())
    runs = matrix(config)
    output = ROOT / "benchmark" / "matrix.jsonl"
    output.write_text("".join(json.dumps(r, sort_keys=True) + "\n" for r in runs))
    if config["supplementary_conditions"] != ["latest_tracefix_monitored"]:
        raise ValueError("the supplemental condition must use the unchanged latest monitoring runtime")
    latest = []
    for original in runs:
        if original["condition"] != "hibana":
            continue
        run = dict(original)
        run["condition"] = "latest_tracefix_monitored"
        run["run_id"] = original["run_id"].removesuffix("hibana") + run["condition"]
        run["upstream_revision"] = config["latest_baseline_revision"]
        latest.append(run)
    if len(latest) != config["expected_supplementary_runs"] or len(latest) != 432:
        raise ValueError("supplemental matrix size changed")
    (ROOT / "benchmark/latest-matrix.jsonl").write_text(
        "".join(json.dumps(r, sort_keys=True) + "\n" for r in latest))
    print(json.dumps({"runs": len(runs), "per_condition": dict(collections.Counter(r["condition"] for r in runs)),
                      "supplementary_runs":len(latest), "total_planned_runs":len(runs)+len(latest)}))


if __name__ == "__main__":
    main()
