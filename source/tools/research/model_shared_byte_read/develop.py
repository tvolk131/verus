#!/usr/bin/env python3
"""Retain a fresh frozen trial under /tmp; never overwrite proof objects."""
import subprocess
import sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
n = 1
while Path(f'/tmp/shared-byte-read-trial-{n:03}').exists() or Path(f'/tmp/shared-byte-read-trial-{n:03}-freeze.json').exists():
    n += 1
freeze = f'/tmp/shared-byte-read-trial-{n:03}-freeze.json'
base = [sys.executable, '-B', str(HERE/'check.py'), '--freeze', freeze]
subprocess.run(base + ['--create-freeze'], check=True)
raise SystemExit(subprocess.run(base + ['--output', f'/tmp/shared-byte-read-trial-{n:03}']).returncode)
