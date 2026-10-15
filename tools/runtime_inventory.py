#!/usr/bin/env python3
"""Pin the exact compiler and baseline runtime source bytes."""

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCES = {
    "tracefix": ("36b4bd108fcd74f7efb17613aebbe3fdf55499a9", ["tracefix/runtime", "tracefix/pipeline"]),
    "tracefix-paper": ("87b01a2c0ae53c5045c9ce3d43cd0235b1831275",
                       ["runtime_A", "runtime_B", "runtime_base_1", "runtime_base_2"]),
}


def inventory():
    result = {}
    for name, (revision, directories) in SOURCES.items():
        source = ROOT / "vendor" / name
        paths = []
        for directory in directories:
            base = source / directory
            if not base.is_dir():
                raise ValueError(f"missing pinned source: {base}")
            paths += [path for path in base.rglob("*.py") if "__pycache__" not in path.parts]
        paths += [source / "LICENSE"]
        result[name] = {
            "revision": revision,
            "files": {str(path.relative_to(source)): hashlib.sha256(path.read_bytes()).hexdigest()
                      for path in sorted(paths)},
        }
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    result = inventory()
    path = ROOT / "benchmark/runtime-manifest.json"
    if args.check:
        if result != json.loads(path.read_text()):
            raise SystemExit("baseline runtime/compiler bytes differ from the pinned manifest")
    else:
        path.write_text(json.dumps(result, indent=2)+"\n")
    print(json.dumps({name: len(value["files"]) for name, value in result.items()}))


if __name__ == "__main__":
    main()
