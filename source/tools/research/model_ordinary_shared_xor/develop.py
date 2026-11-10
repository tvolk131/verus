#!/usr/bin/env python3
"""Freeze every private trial before normal compilation; retain failures."""
import subprocess, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
n = 1
while Path(f'/tmp/model-ordinary-shared-xor-trial-{n:03}').exists() or Path(f'/tmp/model-ordinary-shared-xor-trial-{n:03}-freeze.json').exists():
    n += 1
base = [sys.executable, '-B', str(HERE/'check.py'), '--freeze', f'/tmp/model-ordinary-shared-xor-trial-{n:03}-freeze.json']
subprocess.run(base + ['--create-freeze'], check=True)
raise SystemExit(subprocess.run(base + ['--output', f'/tmp/model-ordinary-shared-xor-trial-{n:03}']).returncode)
