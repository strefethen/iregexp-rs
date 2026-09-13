#!/usr/bin/env python3
"""Verify imported inputs without rewriting fixtures or derived output."""

import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent.parent
entries = json.loads((root / "PROVENANCE.json").read_text())["files"]
for entry in entries:
    actual = hashlib.sha256((root / entry["path"]).read_bytes()).hexdigest()
    if actual != entry["sha256"]:
        raise SystemExit(f"checksum mismatch: {entry['path']}")
print(f"PASS: {len(entries)} imported files match PROVENANCE.json")
