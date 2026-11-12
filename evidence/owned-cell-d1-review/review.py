#!/usr/bin/env python3
"""Private D1 replay using the existing authenticated execution helpers."""
import json, shutil, sys
sys.dont_write_bytecode=True
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
IMPL=ROOT/'source/tools/research/redb_crash_protocol/native_state_link/implementation'
sys.path.insert(0,str(IMPL))
import check
from infer_eval_support import digest
OUT=Path('/tmp/owned-cell-d1-review-973ca52d')
SNAP=Path('/tmp/owned-cell-D1-2e7b8b319-parent-snapshot')
a=sys.argv[1]
assert digest(SNAP/'receipt.json')=='c15b5585f14cb9c5441b25b3667d3aa31ac34fed3ad37830a3edfdaacfcfe0ed'
identity=json.loads((SNAP/'receipt.json').read_text())['tools_sha256']
assert all(digest(SNAP/n)==h for n,h in identity.items())
inputs=[Path(__file__),*sorted((ROOT/'source/vir/src').rglob('*.rs')),
        ROOT/'source/rust_verify_test/tests/local_checked_owned_cell_admission.rs',
        ROOT/'source/tools/infer_experiment.py',ROOT/'source/tools/infer_eval_support.py']
cargo=check.TC/'bin/cargo'
if a=='build':
    for n in [*identity,'verus-root']:
        dst=OUT/'bundle'/n
        if dst.exists(): dst.unlink()
        shutil.copy2(SNAP/n,dst)
    argv=[cargo,'build','--offline','--locked','-p','rust_verify']
elif a=='native':
    argv=[cargo,'test','--offline','--locked','-p','rust_verify','--lib',
          'checked_owned_cell::tests::loaded_native_methods_and_complete_mir_corruptions',
          '--','--nocapture','--test-threads=1']
elif a=='rev':
    argv=[cargo,'test','--offline','--locked','-p','vir','internal_representation','--','--test-threads=1']
elif a=='vir':
    argv=[cargo,'test','--offline','--locked','-p','vir','--lib','--','--test-threads=1']
elif a=='combined':
    profile=OUT/'build/debug'; profile.mkdir(parents=True,exist_ok=True)
    for n in [*identity,'verus-root']:
        dst=profile/n
        if dst.exists(): dst.unlink()
        shutil.copy2(OUT/'bundle'/n,dst)
        assert digest(dst)==digest(OUT/'bundle'/n)
    argv=[cargo,'test','--offline','--locked','-p','rust_verify_test',
          '--test','local_checked_owned_cell','--test','local_checked_owned_cell_admission',
          '--','--test-threads=1']
else: raise ValueError(a)
code=check.run(OUT,'review-'+a+('-v2' if a=='build' else ''),argv,extra_inputs=inputs)
if a=='build' and code==0:
    (OUT/'bundle/rust_verify').unlink()
    shutil.copy2(OUT/'target/debug/rust_verify',OUT/'bundle/rust_verify')
    (OUT/'combined-identity.json').write_text(json.dumps({n:digest(OUT/'bundle'/n) for n in identity},indent=2)+'\n')
assert all(digest(SNAP/n)==h for n,h in identity.items())
sys.exit(code)
