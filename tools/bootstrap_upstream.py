#!/usr/bin/env python3
"""Fetch both pinned MIT-licensed TraceFix releases and verify their source bytes."""

import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

from runtime_inventory import ROOT, SOURCES

def main():
    missing = [(name, revision) for name, (revision, _) in SOURCES.items()
               if not (ROOT / "vendor" / name).exists()]
    if missing:
        with tempfile.TemporaryDirectory(prefix="hibana-tracefix-source-") as temporary:
            clone = Path(temporary) / "repo"
            subprocess.run(["git", "clone", "--no-checkout", "https://github.com/Sensing-And-Reasoning/TraceFix.git", str(clone)], check=True)
            for name, revision in missing:
                archive = Path(temporary) / f"{name}.tar"
                subprocess.run(["git", "-C", str(clone), "archive", revision,
                                "--output", str(archive)], check=True)
                destination = ROOT / "vendor" / name
                destination.mkdir(parents=True)
                with tarfile.open(archive) as source:
                    source.extractall(destination, filter="data")
    for script in ["inventory.py", "runtime_inventory.py"]:
        subprocess.run([sys.executable, str(ROOT / "tools" / script), "--check"], check=True)


if __name__ == "__main__":
    main()
