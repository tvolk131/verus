#!/usr/bin/env python3
"""Scoped offline build/check controller; existing execution/authentication helpers."""
import argparse, json, os, shutil, sys, tempfile, tomllib
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[5]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt, require
TC = Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
BOOTSTRAP = Path('/tmp/certified-foreign-bool-46i7h2jy')

def setup(out):
    out.mkdir(exist_ok=False)
    for d in ['home', 'tmp', 'bundle', 'cargo-home/registry', 'cargo-home/git', 'cache']:
        (out/d).mkdir(parents=True, exist_ok=True)
    f = json.loads((HERE.parent/'BASELINE_FREEZE.json').read_text())
    src = Path(f['runtime']['bundle'])
    require(digest(src/'receipt.json') == f['runtime']['receipt_sha256'], 'baseline receipt')
    for name, h in f['runtime']['tool_hashes'].items():
        require(digest(src/name) == h, 'baseline tool '+name)
    # Only accepted solver is reused. Compiler/macros/vstd will be newly built.
    shutil.copy2(src/'z3', out/'bundle/z3')
    archives = json.loads((BOOTSTRAP/'cache-inputs.json').read_text())['archives']
    index = json.loads((BOOTSTRAP/'index-git-authentication.json').read_text())
    lock = tomllib.loads((ROOT/'source/Cargo.lock').read_text())
    checks = {p['name']+'-'+p['version']+'.crate': p['checksum'] for p in lock['package'] if 'checksum' in p}
    for rel, h in archives.items():
        p = Path(rel)
        require(not p.is_absolute() and '..' not in p.parts and rel.startswith('cargo-home/registry/cache/'), 'archive path')
        require(digest(BOOTSTRAP/p) == h, 'bootstrap archive')
        if p.name in checks: require(checks[p.name] == h, 'Cargo.lock checksum')
        q = out/p; q.parent.mkdir(parents=True,exist_ok=True); shutil.copy2(BOOTSTRAP/p,q)
    for rel, h in index.items():
        p = Path(rel)
        require(not p.is_absolute() and '..' not in p.parts and (rel.startswith('registry/') or rel.startswith('git/')), 'index path')
        require(digest(BOOTSTRAP/'cargo-home'/p) == h, 'bootstrap index/git')
        q = out/'cargo-home'/p; q.parent.mkdir(parents=True,exist_ok=True); shutil.copy2(BOOTSTRAP/'cargo-home'/p,q)
    (out/'cargo-home/git/db/getopts-a5b2265512d62342/refs').mkdir(exist_ok=True)
    (out/'bootstrap.json').write_text(json.dumps(dict(archives=archives,index=index,lock=digest(ROOT/'source/Cargo.lock')),indent=2)+'\n')

def env(out):
    e = dict(os.environ)
    e.update(PATH='/home/dev/.cargo/bin:/usr/bin:/bin', RUSTUP_TOOLCHAIN='1.99.1', RUSTUP_HOME='/home/dev/.rustup',
        RUSTC_BOOTSTRAP='1', LD_LIBRARY_PATH=str(TC/'lib'), CARGO_HOME=str(out/'cargo-home'),
        CARGO_TARGET_DIR=str(out/'target'), CARGO_BUILD_BUILD_DIR=str(out/'build'), CARGO_BUILD_JOBS='2',
        CARGO_NET_OFFLINE='true', XDG_CACHE_HOME=str(out/'cache'), TMPDIR=str(out/'tmp'),
        VERUS_ROOT=str(out/'bundle'), VERUS_Z3_PATH=str(out/'bundle/z3'), VERUS_INFER_CACHE=str(out/'infer-cache'),
        RUST_MIN_STACK='20971520', RUSTFLAGS='--cfg proc_macro_span --cfg verus_keep_ghost --cfg span_locations --cfg procmacro2_semver_exempt',
        VERUS_IN_VARGO='1', VERUS_TEST_LOG_DIR=str(out/'test-logs'))
    return e

def run(out,label,argv,extra_inputs=()):
    d=out/label;d.mkdir(exist_ok=False)
    paths=[*sorted((ROOT/'source/rust_verify/src').glob('*.rs')),HERE/'check.py',ROOT/'source/Cargo.lock',ROOT/'source/rust_verify_test/tests/local_checked_owned_cell.rs',*extra_inputs]
    guard=ArtifactGuard(paths)
    receipt=dict(commands=[],source={str(p.relative_to(ROOT)):digest(p) for p in paths})
    with replay_receipt(d,receipt):
        command=LoggedCommands(d,ROOT/'source',receipt['commands'],timeout=1800,guard=guard)
        environment=env(out);environment['VERUS_INFER_CACHE']=str(d/'infer-cache')
        code=command(label,list(map(str,argv)),environment);receipt['exit']=code
        guard.check();receipt['guard_passed']=True
        print(label,code,flush=True)
    return code

def main():
    p=argparse.ArgumentParser();p.add_argument('action',choices=['setup','build','vstd','tests']);p.add_argument('--output',type=Path,required=True);p.add_argument('--label')
    a=p.parse_args();out=a.output.resolve()
    if a.action=='setup':setup(out);print(out);return
    if a.action=='build':
        code=run(out,a.label or 'build',[TC/'bin/cargo','build','--offline','--locked','-j','2',*sum((['-p',p] for p in ['rust_verify','verus','verus_builtin','verus_builtin_macros','verus_state_machines_macros','vstd_build']),[])])
        if code:sys.exit(code)
        for name in ['rust_verify','verus','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so']:
            shutil.copy2(out/'target/debug'/name,out/'bundle'/name)
        (out/'bundle/verus-root').touch()
        shutil.copytree(ROOT/'source/vstd',out/'bundle/vstd',dirs_exist_ok=True)
    elif a.action=='vstd':
        sys.exit(run(out,a.label or 'vstd',[out/'target/debug/vstd_build',out/'bundle']))
    else:
        # The harness resolves tools beside the test executable in build/debug.
        profile=out/'build/debug'
        for name in ['rust_verify','verus','z3','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','libvstd.rlib','vstd.vir','verus-root']:
            src=out/'bundle'/name;dst=profile/name
            if dst.exists() and digest(dst)==digest(src):continue
            if dst.exists():dst.unlink()
            shutil.copy2(src,dst)
            require(digest(dst)==digest(src),'private test profile '+name)
        sys.exit(run(out,a.label or 'focused-tests',[TC/'bin/cargo','test','--offline','--locked','-p','rust_verify_test','--test','local_checked_owned_cell','--','--test-threads=1']))
if __name__=='__main__':main()
