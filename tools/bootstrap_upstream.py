#!/usr/bin/env python3
"""Fetch the pinned MIT-licensed TraceFix source without running its scripts."""

import subprocess
import tarfile
import tempfile
from pathlib import Path

from inventory import ROOT, UPSTREAM_REVISION

destination = ROOT / "vendor" / "tracefix"
if destination.exists():
    subprocess.run(["python3", str(ROOT / "tools" / "inventory.py"), "--check"], check=True)
else:
    with tempfile.TemporaryDirectory(prefix="hibana-tracefix-source-") as temporary:
        clone = Path(temporary) / "repo"
        subprocess.run(["git", "clone", "--no-checkout", "https://github.com/Sensing-And-Reasoning/TraceFix.git", str(clone)], check=True)
        archive = Path(temporary) / "source.tar"
        subprocess.run(["git", "-C", str(clone), "archive", UPSTREAM_REVISION, "--output", str(archive)], check=True)
        destination.mkdir(parents=True)
        with tarfile.open(archive) as source:
            source.extractall(destination, filter="data")
    subprocess.run(["python3", str(ROOT / "tools" / "inventory.py"), "--check"], check=True)
