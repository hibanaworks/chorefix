#!/usr/bin/env python3
"""Record and verify the complete, unchanged upstream benchmark inputs."""

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
UPSTREAM_REVISION = "36b4bd108fcd74f7efb17613aebbe3fdf55499a9"


def inventory(source: Path) -> dict:
    benchmark = source / "benchmark"
    expected = {f"{scenario}{level}" for scenario in range(1, 17) for level in "EMH"}
    actual = {p.name for p in (benchmark / "descriptions").iterdir() if p.is_dir()}
    if actual != expected:
        raise ValueError(f"task set changed: missing={expected-actual}, extra={actual-expected}")
    tasks = []
    for scenario in range(1, 17):
        for level in "EMH":
            task_id = f"{scenario}{level}"
            description = benchmark / "descriptions" / task_id
            environment = benchmark / "environments" / task_id
            metadata = json.loads((description / "metadata.json").read_text())
            tools = json.loads((description / "tools.json").read_text())
            checklist = json.loads((environment / "checklist.json").read_text())
            paths = sorted(
                p for base in (description, environment) for p in base.rglob("*")
                if p.is_file() and "__pycache__" not in p.parts
            )
            tasks.append({
                "id": task_id,
                "agents": metadata["agents"],
                "resources": metadata["resources"],
                "tools": [entry["function"]["name"] for entry in tools],
                "checklist_items": len(checklist),
                "files": {
                    str(p.relative_to(source)): hashlib.sha256(p.read_bytes()).hexdigest()
                    for p in paths
                },
            })
    simulator_files = {
        str(p.relative_to(source)): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in sorted((benchmark / "tools").rglob("*.py"))
    }
    return {"upstream_revision": UPSTREAM_REVISION, "tasks": tasks, "simulator_files": simulator_files}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=ROOT / "vendor" / "tracefix")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    result = inventory(args.source)
    manifest = ROOT / "benchmark" / "upstream-manifest.json"
    if args.check:
        if result != json.loads(manifest.read_text()):
            raise SystemExit("upstream benchmark bytes differ from the pinned manifest")
    else:
        manifest.parent.mkdir(parents=True, exist_ok=True)
        manifest.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({
        "tasks": len(result["tasks"]),
        "scenarios": 16,
        "roles": [min(len(t["agents"]) for t in result["tasks"]), max(len(t["agents"]) for t in result["tasks"])],
        "checklist_items": sum(t["checklist_items"] for t in result["tasks"]),
        "unchanged": args.check,
    }))


if __name__ == "__main__":
    main()
