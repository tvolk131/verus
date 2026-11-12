#!/usr/bin/env python3
"""F1 focused commands; uses the existing private build and execution helpers."""
import argparse, json, shutil, sys
from pathlib import Path
import check
from infer_eval_support import ArtifactGuard, digest
HERE=Path(__file__).resolve().parent
SNAPSHOT=Path('/tmp/owned-cell-preflight-6e5fb54d-parent-snapshot')

def main():
    p=argparse.ArgumentParser();p.add_argument('action',choices=['tests','native']);p.add_argument('--output',type=Path,required=True);p.add_argument('--label',required=True);a=p.parse_args()
    if not a.label.startswith('F1-'):p.error('new F1- labels required; old receipts must never be replaced')
    out=a.output.resolve();receipt=json.loads((SNAPSHOT/'receipt.json').read_text())
    old=receipt['identity']['tools_sha256'];guard=ArtifactGuard([SNAPSHOT/'receipt.json',*[SNAPSHOT/n for n in old]])
    for n,h in old.items():check.require(digest(SNAPSHOT/n)==h,'immutable old tool '+n)
    # Types/library/macro sources are unchanged; reuse their fully verified
    # artifacts, never rebuild or replace the immutable historical snapshot.
    for n,h in old.items():
        if n!='rust_verify':check.require(digest(out/'bundle'/n)==h,'unchanged library/tool '+n)
    tools=ArtifactGuard([out/'bundle'/n for n in old]);profile=out/'build/debug'
    for n in [*old,'verus-root']:
        src=out/'bundle'/n;dst=profile/n
        if dst.exists() and digest(dst)==digest(src):continue
        if dst.exists():dst.unlink()
        shutil.copy2(src,dst)
        check.require(digest(dst)==digest(src),'private profile '+n)
    extra=[HERE/'f1.py',HERE/'F1_INTERFACE.md',check.ROOT/'source/rust_verify_test/tests/local_checked_owned_cell_admission.rs']
    if a.action=='tests':
        argv=[check.TC/'bin/cargo','test','--offline','--locked','-p','rust_verify_test','--test','local_checked_owned_cell','--test','local_checked_owned_cell_admission','--','--test-threads=1']
    else:
        argv=[check.TC/'bin/cargo','test','--offline','--locked','-p','rust_verify','--lib','checked_owned_cell::tests::loaded_native_methods_and_complete_mir_corruptions','--','--nocapture','--test-threads=1']
    code=check.run(out,a.label,argv,extra_inputs=extra);guard.check();tools.check();sys.exit(code)
if __name__=='__main__':main()
