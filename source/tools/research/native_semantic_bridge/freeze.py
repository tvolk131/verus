#!/usr/bin/env python3
"""Freeze selected sources before trials; no acceptance by Python assertions."""
import argparse
import json
import sys
from pathlib import Path
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import digest
from infer_experiment import require
p=argparse.ArgumentParser()
p.add_argument('--candidate',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
require(not a.output.exists(),'freeze destination already exists')
candidate=a.candidate.resolve()
require(candidate.is_relative_to(HERE/'candidates'),'candidate outside owned directory')
paths=[HERE/'check.py',HERE/'source_map.py',HERE/'audit.py',HERE/'gate_controls.py',HERE/'guard_controls.py',Path(__file__),*candidate.glob('*.v'),
 HERE/'proof_checks.py', HERE/'source-bindings.json', HERE/'baseline-bindings.json',
 HERE.parent/'mutable_buffer_goal/foundation-pin.json',
 HERE.parent/'mutable_buffer_goal/foundation-baseline.json',
 HERE.parent/'mutable_buffer_goal/transfer-freeze.json',
 HERE.parent/'native_heap_lowering/final-freeze.json',
 HERE.parent/'native_heap_lowering/lower.py']
modules=['borrow_handoff','byte_array','raw_view','model_limits']
require({p.stem for p in candidate.glob('*.v')}==set(modules),'unexpected proof module inventory')
f=dict(schema=1,before_first_trial=True,publication=False,native_bridge_proved=False,
 candidate_directory=str(candidate.relative_to(ROOT)),modules=modules,
 files={str(p.relative_to(ROOT)):digest(p) for p in sorted(paths)})
a.output.write_text(json.dumps(f,indent=2)+'\n')
