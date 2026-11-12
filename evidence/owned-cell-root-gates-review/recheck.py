#!/usr/bin/env python3
"""Scoped parent-artifact replay, no builds or harness/tool transformations."""
import datetime, json, os, re, shutil, subprocess, sys
from pathlib import Path
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, support_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require
PIN = '9761b5f55405fdcb2901f6599b52973cb1e31c43'
PARENT = Path('/tmp/owned-cell-root-gate-fixes-d18c0f62-parent-snapshot')
RECEIPT = 'fb56f75c515ab64a6c87b8ac465a1bb282b819f0d5aa10a9069eb102cf8c8f81'
OUT = Path('/tmp/owned-cell-root-gates-review-9761b5f5')
HERE = Path(__file__).resolve().parent

def authenticate():
    require(subprocess.check_output(['git','rev-parse','HEAD'], cwd=ROOT, text=True).strip() == PIN, 'prepared root')
    require(digest(PARENT/'receipt.json') == RECEIPT, 'receipt pin')
    manifest = json.loads((PARENT/'receipt.json').read_text())
    paths = {n:h for n,h in manifest['tools_sha256'].items()}
    paths.update({'checks/'+n:v['sha256'] for n,v in manifest['checks'].items()})
    require(len(manifest['tools_sha256']) == 8 and len(manifest['checks']) == 4, 'exact tool inventory')
    require((PARENT/'verus-root').read_bytes() == b'', 'root marker')
    for n,h in paths.items():
        require(digest(PARENT/n) == h, 'parent authentication '+n)
    return manifest, paths

def env():
    e = dict(os.environ)
    tc = Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
    e.update(RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN='1.99.1', VERUS_IN_VARGO='1',
             VERUS_ROOT=str(OUT/'snapshot'), VERUS_Z3_PATH=str(OUT/'snapshot/z3'),
             RUST_MIN_STACK='20971520', LD_LIBRARY_PATH=f'{tc}/lib:{tc}/lib/rustlib/x86_64-unknown-linux-gnu/lib',
             PATH=f'{tc}/bin:/home/dev/.cargo/bin:/usr/local/bin:/usr/bin:/bin', TMPDIR=str(OUT/'tmp'),
             VERUS_INFER_CACHE=str(OUT/'cache'), CARGO_TARGET_DIR=str(OUT/'unused-target'),
             PYTHONDONTWRITEBYTECODE='1')
    return e

def prepare():
    manifest, paths = authenticate()  # Must precede private copying.
    OUT.mkdir(exist_ok=False)
    for d in ['snapshot/checks','tmp','cache','logs','cases']:
        (OUT/d).mkdir(parents=True, exist_ok=True)
    for n in [*paths, 'verus-root', 'receipt.json']:
        shutil.copy2(PARENT/n, OUT/'snapshot'/n)
        require(digest(PARENT/n) == digest(OUT/'snapshot'/n), 'private copy '+n)
        (OUT/'snapshot'/n).chmod(0o555 if os.access(PARENT/n, os.X_OK) else 0o444)
    relatives = [
        'source/tools/research/redb_crash_protocol/GOAL.md',
        'source/tools/research/redb_crash_protocol/CURRENT.md',
        'source/tools/research/redb_crash_protocol/WORKER_BRIEFS.md',
        'source/tools/research/redb_crash_protocol/native_state_link/ACCEPTANCE_REFINEMENTS.md',
    ]
    relatives += [str(p.relative_to(ROOT)) for d in ['source/vir/src','source/rust_verify/src','source/rust_verify_test/tests']
                  for p in sorted((ROOT/d).rglob('*.rs')) if '/common/' in str(p) or d != 'source/rust_verify_test/tests' or p.name in ['local_checked_owned_cell.rs','local_checked_owned_cell_admission.rs']]
    relatives += [str(p.relative_to(ROOT)) for p in support_artifacts()]
    relatives += ['source/tools/research/redb_crash_protocol/native_state_link/implementation/'+n for n in ['ROOT_GATE_FIXES.md','ROOT_GATE_FIXES_REPORT.md','ROOT_GATE_FIXES_RESULTS.json','root_gate_fixes.py']]
    hashes = {}
    import hashlib
    for n in relatives:
        h = digest(ROOT/n)
        blob = subprocess.check_output(['git','show',PIN+':'+n], cwd=ROOT)
        require(hashlib.sha256(blob).hexdigest() == h, 'frozen source '+n)
        hashes[n] = h
    binding = dict(prepared_root=PIN, source_build=manifest['source_commit'], parent_receipt_sha256=RECEIPT,
                   parent_snapshot=str(PARENT), private_output=str(OUT), authenticated_files=paths,
                   source_sha256=hashes, started_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                   replay_not_independent_rebuild=True, cache_copies=False, source_transformations=0,
                   tool_transformations=0, vstd_inherited='2257/0; not reverified', exact_environment=env())
    (HERE/'BINDING.json').write_text(json.dumps(binding, indent=2)+'\n')
    (OUT/'binding.json').write_text(json.dumps(binding, indent=2)+'\n')
    print(json.dumps(dict(authenticated_tools=8, authenticated_original_tests=4, sources=len(hashes), private_output=str(OUT))))

if __name__ == '__main__':
    if sys.argv[1:] == ['prepare']: prepare()
