#!/usr/bin/env python3
"""Retain exact inspected primary bytes. Never a semantic/native proof check."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard, support_artifacts
from infer_experiment import replay_receipt, require

COMMIT='58cf8f9c35b355199727627d07ae34a4e2846d67'
FILES={
    'machine.rs', 'eval.rs', 'lib.rs', 'borrow_tracker/mod.rs',
    'borrow_tracker/stacked_borrows/mod.rs', 'borrow_tracker/stacked_borrows/stack.rs',
    'borrow_tracker/stacked_borrows/item.rs', 'borrow_tracker/tree_borrows/mod.rs',
    'borrow_tracker/tree_borrows/perms.rs', 'borrow_tracker/tree_borrows/tree.rs',
}


def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True)
    a=p.parse_args();a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
    receipt=dict(schema=1,classification='source_identity_and_inspection_only',
        source_map_matches_pins=False,semantic_equivalence_proved=False,
        model_kernel_checked=False,native_correspondences=0,publication=False,goal_complete=False)
    with replay_receipt(a.output,receipt):
        pin_path=HERE/'pins.json';pin_bytes=pin_path.read_bytes();pins=json.loads(pin_bytes)
        freeze_path=HERE/'source-freeze.json';freeze_bytes=freeze_path.read_bytes();freeze=json.loads(freeze_bytes)
        require(pins['compiler_commit']==COMMIT, 'wrong compiler identity')
        prefix='src/tools/miri/src/'
        require(set(pins['sources'])=={prefix+n for n in FILES}, 'wrong inspected-file inventory')
        files=[Path(x['path']) for x in pins['sources'].values()]
        frozen=[HERE/'check_sources.py',pin_path,HERE/'REPORT.md',*support_artifacts()]
        guard=ArtifactGuard([*files,*frozen,freeze_path])
        def retained(path): return guard.initial[str(path.resolve())]['sha256']
        require(retained(pin_path)==hashlib.sha256(pin_bytes).hexdigest() and
            retained(freeze_path)==hashlib.sha256(freeze_bytes).hexdigest(), 'metadata changed during load')
        require(freeze['before_first_trial'] is True and
            set(freeze['files'])=={str(x.relative_to(ROOT)) for x in frozen}, 'incomplete source pretrial freeze')
        for name,h in freeze['files'].items():require(retained(ROOT/name)==h,'frozen inspector changed '+name)
        for name,item in pins['sources'].items():
            require(item['url']=='https://raw.githubusercontent.com/rust-lang/rust/'+COMMIT+'/'+name,
                    'primary URL identity changed')
            path=Path(item['path'])
            require(retained(path)==item['sha256'] and path.stat().st_size==item['bytes'],
                    'inspected source differs '+name)
        guard.check()
        receipt.update(source_map_matches_pins=True,inspected_files=len(files),inputs=guard.initial,
            compiler_commit=COMMIT,all_final_guards_passed=True)
        print(json.dumps(dict(inspected_files=len(files),native_correspondences=0,publication=False)))


if __name__=='__main__':main()
