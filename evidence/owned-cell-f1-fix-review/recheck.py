#!/usr/bin/env python3
"""Bounded frozen review; existing execution/guard helpers, private raw outputs."""
import sys
sys.dont_write_bytecode = True
import json, os, shutil, subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
WORK=Path('/tmp/owned-cell-f1-fix-review-ad51b8c2')
SNAP=Path('/tmp/owned-cell-F1-skip-fix-9cc0f65c-parent-snapshot')
sys.path.insert(0,str(ROOT/'source/tools'))
sys.path.insert(0,str(ROOT/'source/tools/research/redb_crash_protocol/native_state_link/implementation'))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require
import check

def authenticate():
    require(not WORK.exists(),'private output already exists')
    require(digest(SNAP/'receipt.json')=='9b414f0493385c649d856fd181625ec74778173e10623ee6ba3c3fda959b945f','receipt')
    r=json.loads((SNAP/'receipt.json').read_text())
    require(r['source_commit']=='9cc0f65c5e8ebb75f6858475277991e3bfd940d7','source identity')
    for n,h in r['tools_sha256'].items():require(digest(SNAP/n)==h,'snapshot '+n)
    frozen=json.loads((ROOT/'source/tools/research/redb_crash_protocol/native_state_link/implementation/F1_FIX_RESULTS.json').read_text())
    require(r['tools_sha256']==frozen['tools'],'complete tool map')
    for p,h in frozen['sources'].items():require(digest(ROOT/p)==h,'corrected source '+p)
    old=json.loads((ROOT/'evidence/owned-cell-f1-review/AUTHENTICATION.json').read_text())
    require(all(old['tools'][n]==h for n,h in r['tools_sha256'].items() if n!='rust_verify'),'unchanged seven')
    WORK.mkdir(); b=WORK/'bundle';b.mkdir()
    for n,h in r['tools_sha256'].items():
        shutil.copy2(SNAP/n,b/n);require(digest(b/n)==h,'private '+n)
    (b/'verus-root').touch()
    ids={s:subprocess.check_output(['git','rev-parse',s],cwd=ROOT,text=True).strip() for s in ['HEAD','9cc0f65c','6f0b3b4c']}
    a=dict(identities=ids,snapshot=str(SNAP),receipt_sha256=digest(SNAP/'receipt.json'),tools=r['tools_sha256'],sources=frozen['sources'],private=str(WORK),authenticated_before_copy=True,unchanged_seven=True)
    (HERE/'AUTHENTICATION.json').write_text(json.dumps(a,indent=2)+'\n')
    print('authenticated eight tools and corrected sources')

if __name__=='__main__':
    if sys.argv[1]=='authenticate':authenticate()
