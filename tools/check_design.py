#!/usr/bin/env python3
"""Check saved design artifacts without trusting the designer's verdict."""

import argparse
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def configuration_errors(workspace: Path) -> list[str]:
    config = workspace / "Protocol.cfg"
    if not config.is_file():
        return ["Protocol.cfg is missing"]
    text = re.sub(r"\\\*[^\n]*", "", config.read_text())
    errors = []
    if re.search(r"\bCHECK_DEADLOCK\s+FALSE\b", text, re.IGNORECASE):
        errors.append("TLC deadlock checking was disabled")
    if re.search(r"(?m)^\s*(?:ACTION_)?CONSTRAINT\b", text):
        errors.append("TLC state/action constraints require independent review")
    return errors


def check(workspace: Path) -> dict:
    task = workspace.name
    inputs = ROOT / "vendor/tracefix/benchmark"
    metadata = json.loads((inputs / "descriptions" / task / "metadata.json").read_text())
    checklist = json.loads((inputs / "environments" / task / "checklist.json").read_text())
    errors = configuration_errors(workspace)
    required = ["ir.json", "Protocol.tla", "Protocol.cfg", "states.json",
                "Protocol_translated.tla", "tlc_output.log", "design-result.json",
                "notes/fidelity.json"]
    errors += [f"{name} is missing" for name in required if not (workspace / name).is_file()]
    if (workspace / "ir.json").is_file():
        ir = json.loads((workspace / "ir.json").read_text())
        for field in ["agents", "resources"]:
            actual = [entry["id"] for entry in ir[field]]
            if len(actual) != len(set(actual)) or set(actual) != set(metadata[field]):
                errors.append(f"canonical {field} changed")
    if (workspace / "states.json").is_file():
        states = json.loads((workspace / "states.json").read_text())
        if set(states.get("initial_states", {})) != set(metadata["agents"]):
            errors.append("states.json does not include every original participant")
    if (workspace / "design-result.json").is_file():
        result = json.loads((workspace / "design-result.json").read_text())
        if not result["verification"]["final_passed"]:
            errors.append("saved TLC verification failed")
        if result["model_outcome"]["status"] != "completed":
            errors.append("designer turn did not complete")
    if (workspace / "notes/fidelity.json").is_file():
        entries = json.loads((workspace / "notes/fidelity.json").read_text())
        ids = [entry["id"] for entry in entries]
        if len(ids) != len(set(ids)) or set(ids) != {entry["id"] for entry in checklist}:
            errors.append("fidelity evidence does not cover the complete checklist")
        for entry in entries:
            if entry["status"] not in {"covered", "inconsistent", "missing"}:
                errors.append(f"invalid fidelity status for {entry['id']}")
            if entry["status"] == "missing":
                errors.append(f"missing requirement: {entry['id']}")
            if entry["status"] == "covered" and not entry.get("evidence"):
                errors.append(f"no evidence for {entry['id']}")
            for evidence in entry.get("evidence", []):
                path = (workspace / evidence["path"]).resolve()
                if not path.is_relative_to(workspace.resolve()):
                    errors.append(f"evidence leaves workspace: {entry['id']}")
                elif not path.is_file() or not evidence["needle"] or evidence["needle"] not in path.read_text():
                    errors.append(f"evidence anchor missing: {entry['id']}")
    return {
        "task": task,
        "artifact_checks_passed": not errors,
        "fidelity_review": "required",
        "accepted_for_benchmark": False,
        "errors": errors,
        "sha256": {name: hashlib.sha256((workspace / name).read_bytes()).hexdigest()
                   for name in required if (workspace / name).is_file()},
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("workspace", type=Path)
    args = parser.parse_args()
    result = check(args.workspace)
    print(json.dumps(result, indent=2))
    raise SystemExit(0 if result["artifact_checks_passed"] else 1)


if __name__ == "__main__":
    main()
