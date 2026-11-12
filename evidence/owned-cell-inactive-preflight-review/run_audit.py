#!/usr/bin/env python3
"""Independent reject-only replay; immutable inputs, existing execution helpers."""
import json, os, sys, time
from pathlib import Path
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
TOOLS = ROOT / 'frozen/source/tools'
sys.path.insert(0, str(TOOLS))
from infer_experiment import LoggedCommands, replay_receipt, require, validate_run
from infer_eval_support import ArtifactGuard, digest
TC = Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
BUNDLE = ROOT / 'private-runtime'
OUT = ROOT / 'runs'
REQUEST = '--experimental-owned-unsafe-cell'
GOOD = 'use vstd::prelude::*; verus! {pub fn scalar(v:u64)->(r:u64) ensures r==v, {v}}'
FROZEN = (TOOLS / 'research/redb_crash_protocol/native_state_link/owned_goals.rs').read_text()

def main():
    OUT.mkdir(exist_ok=False)
    auth = json.loads((ROOT/'AUTHENTICATION.json').read_text())
    for n,h in auth['tools_before_and_after_private_copy'].items():
        require(digest(BUNDLE/n)==h, 'private tool '+n)
    guard = ArtifactGuard([ROOT/'run_audit.py', ROOT/'AUTHENTICATION.json',
        *[BUNDLE/n for n in auth['tools_before_and_after_private_copy']],
        *TOOLS.glob('*.py'), *[p for p in (ROOT/'frozen').rglob('*') if p.is_file()]])
    receipt = {'kind':'INDEPENDENT_INACTIVE_PREFLIGHT_REPLAY', 'commands':[], 'cases':[],
        'source_commit':auth['source_commit'], 'native_corruptions_independently_run':False}
    e = dict(os.environ)
    for k in list(e):
        if k.startswith(('VERUS_', '__VERUS_', 'CARGO_')) or k in ('RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER'):
            del e[k]
    e.update(PATH='/home/dev/.cargo/bin:'+str(TC/'bin')+':/usr/bin:/bin', RUSTUP_HOME='/home/dev/.rustup',
        RUSTUP_TOOLCHAIN='1.99.1', RUSTC_BOOTSTRAP='1', LD_LIBRARY_PATH=str(TC/'lib'),
        VERUS_ROOT=str(BUNDLE), VERUS_Z3_PATH=str(BUNDLE/'z3'), RUST_MIN_STACK='20971520',
        CARGO_NET_OFFLINE='true', CARGO_HOME=str(OUT/'cargo-home'),
        XDG_CACHE_HOME=str(OUT/'xdg-cache'))
    with replay_receipt(OUT, receipt):
        guard.check()
        log = LoggedCommands(OUT, ROOT, receipt['commands'], timeout=90)
        def case(label, source, opts=(), expected=False, reason=None, cargo=False,
                 explicit=False, parse=False, poison=False, deletion_error=False):
            d=OUT/label; d.mkdir(); (d/'tmp').mkdir()
            src=d/'case.rs'; src.write_text(source)
            env=dict(e, VERUS_INFER_CACHE=str(d/'infer-cache'), TMPDIR=str(d/'tmp'))
            exp=d/'explicit.vir'; carg=d/'libcargo.vir'; paths=[]
            argv=[str(BUNDLE/('rust_verify' if cargo else 'verus'))]
            if cargo:
                argv+=['rustc']
                env.update(__VERUS_DRIVER_VIA_CARGO__='1', CARGO_PKG_NAME='owned_cell_preflight',
                    CARGO_PKG_VERSION='0.0.0', CARGO_MANIFEST_DIR='/owned-cell-preflight-fixture',
                    __VERUS_DRIVER_VERIFY_owned_cell_preflight_ignored='unused')
                env['__VERUS_DRIVER_VERIFY_owned_cell_preflight-0.0.0-af08914f15a5']='1'
                argv+=['--emit=metadata']
                paths.append(carg)
            argv += [str(src),'--crate-name=case','--crate-type=lib','--edition=2024',
                '-o', str(d/('libcargo.rmeta' if cargo else 'libcase.rlib')),
                '--output-json', *opts]
            if explicit:
                argv+=['--export', str(exp)]; paths.append(exp)
            for p in paths: p.write_bytes(b'private stale metadata sentinel')
            if deletion_error:
                exp.unlink(); exp.mkdir()
            if poison:
                bad=d/'poison.vir'; bad.write_bytes(b'not a VIR file')
                argv+=['--import','poison='+str(bad)]
                # Existing invalid cache must not matter before request refusal.
                (d/'infer-cache').mkdir(); (d/'infer-cache/poison').write_bytes(b'not a cache entry')
            code=log(label,argv,env)
            stdout=(OUT/(label+'.stdout')).read_text(); stderr=(OUT/(label+'.stderr')).read_text()
            require((code==0)==expected, label+' exit')
            require(not any(x in stderr.lower() for x in ['panicked at','internal compiler error','timed out','resource limit']), label+' panic/resource')
            if reason: require(reason in stderr, label+' missing reason: '+stderr)
            data=None
            if not parse and not deletion_error:
                data=validate_run(dict(stdout=stdout,stderr=stderr,returncode=code),expected,
                    allow_frontend=bool(reason and 'postcondition not satisfied' not in reason))
                if expected:
                    vr=data['verification-results']
                    require(vr.get('is-verifying-entire-crate') is True and vr.get('errors')==0, label+' full checks')
                    require((d/'libcase.rlib').is_file(),label+' compile')
                if REQUEST in opts:
                    inf=data.get('verus-infer',{})
                    require(not inf.get('functions'),label+' no helpers')
                    require(inf.get('interface_written',False) is False,label+' no interface')
            for p in paths:
                require((p.is_dir() if deletion_error and p==exp else not p.exists()),label+' stale output '+str(p))
            row={'label':label,'expected_success':expected,'exit_code':code,'expected_reason':reason,
                'argv':argv,'source_sha256':digest(src),'stdout_sha256':digest(OUT/(label+'.stdout')),
                'stderr_sha256':digest(OUT/(label+'.stderr')),'selected_output_states':{p.name:('directory' if p.is_dir() else 'absent') for p in paths},
                'verification_results':data.get('verification-results') if data else None,
                'source_path':str(src),'fresh_private_cache':env['VERUS_INFER_CACHE']}
            receipt['cases'].append(row)
            print(label,code,'PASS',flush=True)
        for infer in [False,True]:
            mode='infer' if infer else 'ordinary'; inf=['--infer-goals'] if infer else []
            case(mode+'-default-good',GOOD,['--no-cheating','--compile',*inf],True)
            case(mode+'-default-false',GOOD.replace('r==v','r!=v'),['--no-cheating','--compile',*inf],reason='postcondition not satisfied')
            for repeat in range(2):
                case(mode+'-frozen-'+str(repeat),FROZEN,[REQUEST,'--no-cheating','--compile',*inf],reason='experimental owned UnsafeCell: inactive:')
            for label,extra in [('no-verify',['--no-verify']),('no-lifetime',['--no-lifetime']),
                ('no-trait',['--no-trait-conflicts']),('no-erasure',['--no-erasure-check']),
                ('root',['--verify-root']),('module',['--verify-module','some_module']),
                ('only-module',['--verify-only-module','some_module']),
                ('function',['--verify-root','--verify-function','owned_roundtrip'])]:
                case(mode+'-'+label,FROZEN,[REQUEST,'--no-cheating',*inf,*extra],reason='requires --no-cheating and full body/lifetime/trait/erasure')
            case(mode+'-no-cheating',FROZEN,[REQUEST,*inf],reason='requires --no-cheating')
            case(mode+'-zero-helper-export',GOOD,[REQUEST,'--no-cheating',*inf],reason='local-only; selected VIR export is unsupported',explicit=True)
            for cargo,explicit,label in [(True,True,'dual'),(True,False,'cargo-only')]:
                for malformed in [False,True]:
                    case(mode+'-'+label+('-parse' if malformed else '-refusal'),
                        'pub fn broken( {' if malformed else 'pub fn scalar()->u64 {0}',
                        [REQUEST,'--no-cheating',*inf],reason='error' if malformed else 'local-only; selected VIR export is unsupported',
                        cargo=cargo,explicit=explicit,parse=malformed)
            case(mode+'-before-import-cache',GOOD,[REQUEST,'--no-cheating',*inf],reason='experimental owned UnsafeCell: inactive:',poison=True)
            case(mode+'-deletion-error','pub fn scalar()->u64 {0}',[REQUEST,'--no-cheating',*inf],
                reason='invalidate requested export',cargo=True,explicit=True,deletion_error=True)
        guard.check(); receipt['guard_passed']=True
        receipt['command_seconds_sum']=sum(x['seconds'] for x in receipt['commands'])
        receipt['all_cases_passed']=True
if __name__=='__main__': main()
