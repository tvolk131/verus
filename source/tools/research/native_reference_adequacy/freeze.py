#!/usr/bin/env python3
"""Freeze candidates/tools before any trial, refusing overwrite."""
import argparse, json, sys
from pathlib import Path
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import digest
p = argparse.ArgumentParser()
p.add_argument('--output', type=Path, required=True)
p.add_argument('files', nargs='+', type=Path)
a = p.parse_args()
r = {'schema': 1, 'before_first_trial': True, 'files': {
    str(q.resolve().relative_to(ROOT)): digest(q) for q in a.files}}
with a.output.open('x') as f:
    json.dump(r, f, indent=2); f.write('\n')
