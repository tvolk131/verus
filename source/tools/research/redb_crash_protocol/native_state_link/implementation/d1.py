#!/usr/bin/env python3
"""D1 focused private commands; reuse existing execution/authentication helpers."""
import argparse, json, os, shutil, sys
from pathlib import Path
import check
from infer_eval_support import ArtifactGuard, digest
HERE = Path(__file__).resolve().parent
SNAPSHOTS = [Path('/tmp/owned-cell-preflight-6e5fb54d-parent-snapshot'),
             Path('/tmp/owned-cell-F1-2cb432c6-parent-snapshot')]

def inputs():
    return [HERE/'d1.py', HERE/'D1_INTERFACE.md', HERE/'D2_INTERFACE.md', HERE/'C1_INTERFACE.md',
            check.ROOT/'source/rust_verify_test/tests/local_checked_owned_cell_admission.rs',
            *[check.ROOT/('source/vir/src/'+n) for n in
              ['internal_repr.rs', 'context.rs', 'well_formed.rs', 'ast_to_sst_func.rs', 'ast_to_sst_crate.rs',
               'infer_checks/tests/internal_representation.rs']]]

def main():
    p = argparse.ArgumentParser()
    p.add_argument('action', choices=['build', 'native', 'rev', 'tests'])
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--label', required=True)
    a = p.parse_args()
    if not a.label.startswith(('D1-', 'D2-', 'C1-')): p.error('new D1-/D2-/C1- labels required')
    if a.label.startswith('C1-'):
        os.environ.update(CARGO_PROFILE_DEV_DEBUG='0', CARGO_PROFILE_TEST_DEBUG='0')
        if shutil.disk_usage('/tmp').free < 25*1024**3: p.error('less than25GiB free; checkpoint')
    if a.label.startswith(('D2-', 'C1-')):
        os.environ['CARGO_INCREMENTAL'] = '0'
        SNAPSHOTS.append(Path('/tmp/owned-cell-D1-2e7b8b319-parent-snapshot'))
        check.require(digest(SNAPSHOTS[-1]/'receipt.json') ==
            'c15b5585f14cb9c5441b25b3667d3aa31ac34fed3ad37830a3edfdaacfcfe0ed',
            'authenticated D1 receipt')
    out = a.output.resolve()
    paths = []
    for snapshot in SNAPSHOTS:
        receipt = json.loads((snapshot/'receipt.json').read_text())
        paths.append(snapshot/'receipt.json')
        for name, h in receipt.get('identity', receipt)['tools_sha256'].items():
            check.require(digest(snapshot/name) == h, 'immutable tool '+name)
            paths.append(snapshot/name)
    guard = ArtifactGuard(paths)
    old = (lambda r:r.get('identity',r)['tools_sha256'])(json.loads((SNAPSHOTS[-1]/'receipt.json').read_text()))
    check.require(digest(SNAPSHOTS[1]/'receipt.json') ==
        '51ae6b41380ac4bdd8dd362889a4818df7b352d86aa032d306055377a6b98bf1',
        'authenticated F1 receipt')
    for name, h in old.items():
        if name != 'rust_verify':
            check.require(digest(out/'bundle'/name) == h, 'unchanged tool '+name)
    runtime = ArtifactGuard([out/'bundle'/n for n in old if a.action != 'build' or n != 'rust_verify'])
    cargo = check.TC/'bin/cargo'
    if a.action == 'build':
        argv = [cargo, 'build', '--offline', '--locked', '-p', 'rust_verify']
    elif a.action == 'native':
        argv = [cargo, 'test', '--offline', '--locked', '-p', 'rust_verify', '--lib',
                'checked_owned_cell::tests::loaded_native_methods_and_complete_mir_corruptions',
                '--', '--nocapture', '--test-threads=1']
    elif a.action == 'rev':
        argv = [cargo, 'test', '--offline', '--locked', '-p', 'vir',
                'internal_representation', '--', '--test-threads=1']
    else:
        profile = out/'build/debug'
        for name in [*old, 'verus-root']:
            src, dst = out/'bundle'/name, profile/name
            if dst.exists() and digest(dst) == digest(src): continue
            if dst.exists(): dst.unlink()
            shutil.copy2(src, dst)
            check.require(digest(dst) == digest(src), 'private profile '+name)
        argv = [cargo, 'test', '--offline', '--locked', '-p', 'rust_verify_test',
                '--test', 'local_checked_owned_cell', '--test', 'local_checked_owned_cell_admission',
                '--', '--test-threads=1']
    code = check.run(out, a.label, argv, extra_inputs=inputs())
    if not code and a.action == 'build':
        shutil.copy2(out/'target/debug/rust_verify', out/'bundle/rust_verify')
    guard.check()
    runtime.check()
    sys.exit(code)
if __name__ == '__main__': main()
