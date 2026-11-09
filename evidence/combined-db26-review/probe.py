#!/usr/bin/env python3
"""Independent combined-boundary attacks. Writes only to a new private output directory."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

PRE = 'use vstd::prelude::*; verus! {\n'
VALID = PRE + 'pub mod hidden {pub fn value()->(r:u8) ensures r==0,{0}} pub fn selected(){} }'
IDENTITY = PRE + 'pub proof fn identity<T>(tracked value:T)->(tracked result:T){value} }'
DUPLICATE = IDENTITY.replace('identity', 'duplicate').replace('result:T){value}', 'result:(T,T)){(value,value)}')
WORD = PRE + '''pub fn grant(word:u64,index:u64)->(result:u64)
    requires index<64,
    ensures vstd::word::view(result,64) == vstd::word::view(word,64).update(index as int,true),
    {word | (1u64 << index)} }'''
CALLER = WORD.replace('word | (1u64 << index)', 'provider::grant(word,index)')
FOREIGN = PRE + '''
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExternalOpaque(foreign::Opaque);
pub struct Inner { pub token: foreign::Opaque, pub n:u8 }
pub struct Outer { pub inner:Inner, pub other:u64 }
impl Outer {
    #[verifier::infer]
    pub fn query(&self, x:u8)->u8 { self.inner.n.saturating_sub(x) }
}
pub fn policy(o:&Outer,x:u8)->(r:u8)
    ensures r as int == if o.inner.n >= x {o.inner.n as int-x} else {0},
{o.query(x)}
}'''


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--verus', type=Path, required=True)
    p.add_argument('--solver', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    out = a.output.resolve()
    out.mkdir(mode=0o700, parents=True, exist_ok=False)
    binary = a.verus.resolve()
    env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
               RUSTUP_TOOLCHAIN='1.99.1', VERUS_Z3_PATH=str(a.solver.resolve()),
               VERUS_INFER_CACHE=str(out/'cache'))
    receipt = dict(runs=[], bundle_sha256={f.name:sha(f.read_bytes()) for f in binary.parent.iterdir() if f.is_file()})
    flags = ['--infer-goals', '--no-cheating']

    def run(label, name, src, extra=(), deps=(), ok=True, diag=None, aware=True, compile=False, tool=None):
        path = out/(name+'.rs')
        path.write_text(src)
        (out/(label+'.source.rs')).write_text(src)
        artifact = out/(name+'.vir')
        cmd = [str(tool or binary), str(path), '--crate-type=lib', '--edition=2021', '--crate-name', name,
               '--output-json', '--export', str(artifact), '-L', 'dependency='+str(out), *extra]
        if compile:
            cmd += ['--compile', '-o', str(out/('lib'+name+'.rlib'))]
        for dep in deps:
            cmd += ['--extern', f'{dep}={out}/lib{dep}.rlib', '--import', f'{dep}={out}/{dep}.vir']
        result = subprocess.run(cmd, env=env, cwd=out, capture_output=True, timeout=180)
        (out/(label+'.stdout')).write_bytes(result.stdout)
        (out/(label+'.stderr')).write_bytes(result.stderr)
        data = json.loads(result.stdout)
        inf = data.get('verus-infer', {})
        row = dict(label=label, command=cmd, exit=result.returncode, source_sha256=sha(src.encode()),
                   stdout_sha256=sha(result.stdout), stderr_sha256=sha(result.stderr),
                   interface_exists=artifact.exists(), inference=inf,
                   verification=data['verification-results'])
        receipt['runs'].append(row)
        (out/'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
        assert (result.returncode == 0) == ok, (label, result.stderr.decode())
        assert artifact.exists() == ok, (label, 'wrong publication result')
        if aware:
            assert inf.get('interface_written') == ok, (label, inf)
        if ok and aware:
            assert artifact.read_bytes().endswith(b'VRINFER1'), label
        if diag:
            assert diag in result.stderr.decode(), (label, result.stderr.decode())
        assert not any(t in result.stderr.decode().lower() for t in ['panicked at', 'resource limit', 'timed out']), label
        print(label, result.returncode, flush=True)
        return data

    for i, bypass in enumerate([
        ['--no-verify'], ['--no-verify','--build-sst'], ['--verify-root'],
        ['--verify-root','--verify-function','selected'], ['--verify-module','hidden'],
        ['--verify-only-module','hidden'], ['--no-lifetime'], ['--no-trait-conflicts'],
    ]):
        run(f'seed_{i}', 'provider', VALID, flags)
        result = run(f'bypass_{i}', 'provider', VALID.replace('r==0','r==1'), flags+bypass,
                     ok=False, diag='requires full-crate verification')
        assert result['verus-infer']['functions'] == [], 'ordinary zero-candidate control changed'
    run('tracked_seed', 'provider', IDENTITY, flags)
    run('tracked_duplicate_normal', 'provider', DUPLICATE, flags, ok=False, diag='use of moved value')
    run('tracked_seed_again', 'provider', IDENTITY, flags)
    run('tracked_duplicate_bypass', 'provider', DUPLICATE, flags+['--no-lifetime'],
        ok=False, diag='normal lifetime/trait checks')
    run('legacy_ordinary_early_export', 'legacy', VALID.replace('r==0','r==1'),
        ['--no-verify'], aware=False)
    assert not (out/'legacy.vir').read_bytes().endswith(b'VRINFER1')

    run('word_provider', 'provider', WORD, flags, compile=True)
    original = (out/'provider.vir').read_bytes()
    start = original.rindex(b'{"schema":')
    meta = json.loads(original[start:-8])
    assert meta['schema'] == 38 and meta['policy'] == 'candidate-bound-evidence-v66'
    assert not meta['proofs'] and meta['trusted_models']
    receipt['word_metadata'] = meta
    for assisted in [False, True]:
        opts = flags if assisted else ['--no-cheating']
        run(f'word_consumer_{assisted}', 'consumer', CALLER, opts, ['provider'], compile=True)
        for bypass in ['--no-lifetime', '--no-trait-conflicts', '--no-verify']:
            run(f'consumer_seed_{assisted}_{bypass}', 'consumer', CALLER, opts, ['provider'])
            run(f'consumer_bypass_{assisted}_{bypass}', 'consumer', CALLER, opts+[bypass],
                ['provider'], ok=False, diag='requires')
        for old in [63,64,65]:
            run(f'policy_seed_{assisted}_{old}', 'consumer', CALLER, opts, ['provider'])
            bad = dict(meta, policy=f'candidate-bound-evidence-v{old}')
            (out/'provider.vir').write_bytes(original[:start]+json.dumps(bad).encode()+b'VRINFER1')
            run(f'policy_reject_{assisted}_{old}', 'consumer', CALLER, opts, ['provider'],
                ok=False, diag='incompatible inference schema')
            (out/'provider.vir').write_bytes(original)
        run(f'poison_seed_{assisted}', 'consumer', CALLER, opts, ['provider'])
        bad = json.loads(json.dumps(meta))
        bad['trusted_models'][-1][1] = 'poisoned-library-artifact'
        (out/'provider.vir').write_bytes(original[:start]+json.dumps(bad).encode()+b'VRINFER1')
        run(f'poison_reject_{assisted}', 'consumer', CALLER, opts, ['provider'],
            ok=False, diag='stale or missing trusted-model fingerprint')
        (out/'provider.vir').write_bytes(original)
    # Alter only the imported library artifact in a disposable bundle copy.
    # Bincode still decodes the library; the changed whole-artifact hash must
    # reject the previously checked provider even with its certificate intact.
    poisoned = out/'poisoned-bundle'
    shutil.copytree(binary.parent, poisoned)
    with (poisoned/'vstd.vir').open('ab') as stream:
        stream.write(b'combined-review-artifact-poison')
    run('artifact_poison_seed', 'consumer', CALLER, flags, ['provider'])
    run('artifact_poison_reject', 'consumer', CALLER, flags, ['provider'], ok=False,
        diag='stale or missing trusted-model fingerprint', tool=poisoned/'verus')
    run('word_false_seed', 'provider', WORD, flags)
    run('word_false_ordinary', 'provider', WORD+'\nverus! {fn false_goal(){assert(false);}}',
        flags, ok=False, diag='assertion failed')
    # A homonymous user module cannot trigger the vstd-identity selector.
    fake = PRE + '''pub mod word {
        use super::*;
        pub open spec fn view(w:u64,n:nat)->Seq<bool>{Seq::new(n,|i:int|false)}
        pub open spec fn member(w:u64,i:int)->bool{false}
    }
    pub fn unchanged(w:u64)->(r:u64) ensures word::view(r,64)==word::view(w,64),{w}
    }'''
    data = run('word_user_identity', 'fake', fake, flags)
    assert not any(x['strategy']=='word_view_broadcast' for x in data['verus-infer']['goal_assistance'])
    no_leak = WORD + '''\nverus! {
        proof fn without_scoped_broadcast(word:u64,index:u64)
            requires index<64,
            ensures vstd::word::view(word | (1u64 << index),64) ==
                vstd::word::view(word,64).update(index as int,true),
        {}
    }'''
    data = run('word_scoped_no_proof_leak', 'no_leak', no_leak, flags,
               ok=False, diag='postcondition not satisfied')
    selected = [x for x in data['verus-infer']['goal_assistance'] if x['strategy']=='word_view_broadcast']
    assert len(selected)==1 and selected[0]['function'].endswith('::grant'), selected

    native = out/'foreign.rs'
    native.write_text('pub struct Opaque {pub x:u8}\npub struct Token<T>(pub T);\npub struct DropToken;\nimpl Drop for DropToken {fn drop(&mut self){}}\n')
    cmd = ['rustc',str(native),'--crate-type=lib','--edition=2021','-o',str(out/'libforeign.rlib')]
    r = subprocess.run(cmd, env=env, capture_output=True, timeout=120)
    assert r.returncode == 0, r.stderr.decode()
    receipt['native_command'] = cmd
    extern = ['--extern', 'foreign='+str(out/'libforeign.rlib')]
    for phase in ['cold','warm']:
        data = run('foreign_nested_'+phase, 'packets', FOREIGN, flags+extern, compile=True)
        fs = data['verus-infer']['functions']
        assert len(fs)==1 and fs[0]['required_checks_passed']
        assert all(v['required'] and v['status']=='proved' for v in fs[0]['proofs'].values())
    consumer = PRE+'pub fn use_query(o:&packets::Outer){let r=o.query(o.inner.n); assert(r==0);} }'
    run('foreign_import', 'foreign_consumer', consumer, flags+extern, ['packets'])
    short = FOREIGN.split('pub fn policy')[0]+'}'
    cases = [
        ('mutable', short.replace('pub fn query(&self, x:u8)->u8 { self.inner.n.saturating_sub(x) }',
                                 'pub fn query(&mut self, x:u8) { self.inner.n=x; }'), 'Verus Infer unsupported'),
        ('byvalue', short.replace('query(&self,', 'query(self,'), 'Verus Infer unsupported'),
        ('sound_only', short.replace('#[verifier::infer]', '#[verifier::infer(sound)]'), 'Verus Infer unsupported'),
        ('root', PRE+'''#[verifier::external_type_specification] #[verifier::external_body]
             pub struct ExternalOpaque(foreign::Opaque);
             #[verifier::infer] pub fn root(o:&foreign::Opaque)->u8 {0} }''', 'Verus Infer unsupported'),
        ('generic', short.replace('pub struct ExternalOpaque(foreign::Opaque);',
              'pub struct ExternalOpaque<#[verifier::reject_recursive_types] T>(foreign::Token<T>);').replace('pub token: foreign::Opaque',
              'pub token: foreign::Token<u8>'), 'Verus Infer unsupported'),
        ('drop', short.replace('foreign::Opaque','foreign::DropToken'), 'Verus Infer unsupported'),
        ('wrong_identity', short.replace('ExternalOpaque(foreign::Opaque)', 'ExternalOpaque(foreign::DropToken)'), 'not supported'),
        ('false_identity', FOREIGN+'\nverus! {fn false_id(a:&Outer,b:&Outer){assert(a.inner.token==b.inner.token);}}', 'assertion failed'),
    ]
    for label, source, diag in cases:
        run('foreign_seed_'+label, 'packets', FOREIGN, flags+extern)
        run('foreign_'+label, 'packets', source, flags+extern, ok=False, diag=diag)
    combined = FOREIGN+'\n'+WORD
    data = run('combined_provider', 'provider', combined, flags+extern, compile=True)
    assert len(data['verus-infer']['functions'])==1
    assert data['verus-infer']['functions'][0]['required_checks_passed']
    assert any(x['strategy']=='word_view_broadcast' for x in data['verus-infer']['goal_assistance'])
    both = CALLER+'\n'+PRE+'pub fn use_query(o:&provider::Outer){let r=o.query(o.inner.n);assert(r==0);} }'
    run('combined_consumer', 'combined_consumer', both, flags+extern, ['provider'])
    run('combined_ordinary_failure', 'provider', combined+'\nverus! {fn false_goal(){assert(false);}}',
        flags+extern, ok=False, diag='assertion failed')
    receipt['bundle_unchanged'] = receipt['bundle_sha256'] == {f.name:sha(f.read_bytes()) for f in binary.parent.iterdir() if f.is_file()}
    assert receipt['bundle_unchanged']
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')


if __name__ == '__main__':
    main()
