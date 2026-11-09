#!/usr/bin/env python3
"""Independent pair attacks; raw sources/logs stay in a new private directory."""
import argparse
import copy
import importlib.util
import json
import os
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require, require_exact, require_publication, validate_run

PRE = 'use vstd::prelude::*; verus! {\n'
SPLIT = PRE + '#[verifier::infer] pub fn split(x:u8)->(u8,u8){(x+1,x/2)} }'


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--verus', type=Path, required=True)
    p.add_argument('--solver', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    out = a.output.resolve()
    out.mkdir(mode=0o700, parents=True, exist_ok=False)
    env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
               RUSTUP_TOOLCHAIN='1.99.1', VERUS_Z3_PATH=str(a.solver.resolve()),
               VERUS_INFER_CACHE=str(out / 'cache'))
    guard = ArtifactGuard(tool_artifacts(a.verus.resolve(), a.solver.resolve(), extra=[Path(__file__)]))
    receipt = dict(commands=[], runs=[], tools=guard.initial)
    invoke = LoggedCommands(out, out, receipt['commands'], timeout=240, guard=guard)

    def run(label, source, *, name=None, opts=(), ok=True, diag=None, count=None, semantic=True):
        name = name or label
        path = out / (name + '.rs')
        path.write_text(source)
        (out / (label + '.source.rs')).write_text(source)
        artifact = out / (name + '.vir')
        cmd = [str(a.verus.resolve()), str(path), '--no-cheating', '--output-json',
               '--multiple-errors', '20', '--crate-type=lib', '--crate-name', name,
               '--export', str(artifact), '-L', 'dependency=' + str(out), *map(str, opts)]
        rc = invoke(label, cmd, env)
        result = dict(returncode=rc, stdout=(out / (label + '.stdout')).read_text(),
                      stderr=(out / (label + '.stderr')).read_text())
        data = json.loads(result['stdout'])
        receipt['runs'].append(dict(label=label, command=cmd, returncode=rc,
            source_sha256=digest(path), stdout_sha256=digest(out / (label + '.stdout')),
            stderr_sha256=digest(out / (label + '.stderr')), data=data,
            export_exists=artifact.exists()))
        validate_run(result, ok, allow_frontend=not semantic)
        if diag:
            require(diag in result['stderr'], label + ': missing diagnostic')
        if ok:
            require(data['verification-results']['is-verifying-entire-crate'] is True, label)
            require(data['verus-infer']['verification_scope']['complete_selected_scope_verified'] is True, label)
            require_exact(data['verus-infer']['functions'])
        if count is not None:
            require(len(data['verus-infer']['functions']) == count, label + ': function coverage')
        require_publication(data, ok, artifact)
        print(label, rc, flush=True)
        return data

    with replay_receipt(out, receipt):
        # Different widths, booleans, negative truncation, casts, pair-valued
        # conditionals and substitution through multiple calls and projections.
        src = PRE
        types = ['u8', 'u16', 'u32', 'u64', 'u128', 'usize', 'i8', 'i16', 'i32', 'i64', 'i128', 'isize']
        for typ in types:
            src += f'#[verifier::infer] fn f_{typ}(x:{typ})->({typ},bool){{(x+1,x==0)}}\n'
        src += '''
#[verifier::infer] fn sign(x:i16,y:i16)->(i16,i16){(x/y,x%y)}
#[verifier::infer] fn cast(x:i16)->(u8,i8){(x as u8,x as i8)}
#[verifier::infer] fn narrow(x:u128)->(usize,isize){(x as usize,x as isize)}
#[verifier::infer] fn flags(x:bool,y:bool)->(bool,bool){(x&&!y,x||y)}
#[verifier::infer] fn pairs(x:u8,y:u8,b:bool)->(u8,u8){
    let p=if b {(x+1,y/x)} else {(y+1,x/y)};
    let q=(p.1,p.0); (q.0+1,q.1)
}
#[verifier::infer] fn compose(x:u8)->(u8,u8){
    let p=pairs(x,10,true); let q=pairs(p.1,10,false); (p.0,q.1)
}
fn caller(){
    let p=sign(-7,3);assert(p.0 == -2 && p.1 == -1);
    let p=sign(7,-3);assert(p.0 == -2 && p.1 == 1);
    // Ordinary SMT clips retain range/identity facts; prove wrapping ground
    // arithmetic in the existing bit-vector backend before using it here.
    assert((-129i16 as u8)==127 && (-129i16 as i8)==127) by(bit_vector);
    assert((511i16 as u8)==255 && (511i16 as i8) == -1) by(bit_vector);
    let p=cast(-129);assert(p.0==127 && p.1==127);
    let p=cast(511);assert(p.0==255 && p.1 == -1);
    let p=flags(true,false);assert(p.0 && p.1);
    let p=compose(2);assert(p.0==6 && p.1==11);
}
'''
        for typ in types:
            src += f'fn check_{typ}(){{let p=f_{typ}({typ}::MAX-1);assert(p.0=={typ}::MAX && !p.1);}}\n'
        src += '}'
        run('mixed_scalars', src, count=18)
        for bits in [32, 64]:
            body = f'''fn arch() requires arch_word_bits()=={bits}, {{
                let p=narrow(18446744073709551615u128);
                assert(p.0 == (18446744073709551615u128 as usize));
                assert(p.1 == (18446744073709551615u128 as isize));
                assert(usize::MAX as int == {2**bits-1});
                let q=narrow(127);assert(q.0==127 && q.1==127);
            }}'''
            run('narrow_' + str(bits), src[:-1] + body + '}', count=18)
        for label, body, call in [
            ('discard_call', 'let p=split(x);p.1', 'f(255)'),
            ('discard_branch', 'let p=if b {(x+1,0u8)} else {(0u8,x/y)};p.0', 'f(1,0,false)'),
            ('discard_signed', 'let p=(x/y,0i8);p.1', 'f(i8::MIN,-1)'),
        ]:
            sig = {'discard_call':'x:u8', 'discard_branch':'x:u8,y:u8,b:bool', 'discard_signed':'x:i8,y:i8'}[label]
            typ = 'i8' if label == 'discard_signed' else 'u8'
            unit = SPLIT[:-1] + f'#[verifier::infer] fn f({sig})->{typ}{{{body}}} fn bad(){{let _={call};}} }}'
            run(label, unit, ok=False, diag='precondition not satisfied', count=2)
        for label, signature, body in [
            ('ref_pair', 'x:&(u8,u8)->u8', 'x.0'),
            ('enum_pair', 'x:u8->Option<(u8,u8)>', 'Some((x,x))'),
            ('mutable_pair', 'x:u8->u8', 'let mut p=(x,x);p.0=x+1;p.0'),
            ('destructure', 'x:u8->u8', 'let (a,b)=(x,x);a+b'),
        ]:
            params, ret = signature.split('->')
            run(label, PRE+f'#[verifier::infer] fn f({params})->{ret}{{{body}}} }}',
                ok=False, semantic=False, diag='Verus Infer unsupported')

        # Same source identity on every candidate attack. A hit is still a fresh
        # three-obligation check; malformed formulas may honestly regenerate.
        env['VERUS_INFER_CACHE'] = str(out / 'provider-cache')
        run('cache_seed', SPLIT, name='provider', count=1)
        files = list((out / 'provider-cache').glob('*.json'))
        selected = [f for f in files if 'Pair' in json.dumps(json.loads(f.read_text())['post'])
                    and json.loads(f.read_text())['post'].get('Binary', [None, None])[1] == {'Param': 1}]
        # Select by the current provider's miss-written entry using parameter and
        # component shape, then ensure exactly one, rather than poisoning fixtures.
        selected = [f for f in selected if json.loads(f.read_text())['post']['Binary'][2].get('Pair', [None])[0] == [{'Unsigned': 8}, {'Unsigned': 8}]]
        require(len(selected) == 1, 'ambiguous provider cache')
        cache = selected[0]
        original = cache.read_bytes()
        candidate = json.loads(original)
        for kind in ['restrictive', 'narrow_domain', 'overpermissive', 'swapped', 'missing', 'projection', 'wrong_scalar',
                     'nested_type', 'bad_index', 'result_pre', 'oversize', 'deep', 'wrong_version']:
            c = copy.deepcopy(candidate)
            expected_hit = kind in ['restrictive', 'narrow_domain', 'overpermissive', 'swapped']
            if kind == 'restrictive': c['pre'] = {'Bool': False}
            elif kind == 'narrow_domain': c['pre'] = {'Binary': [{'Inequality': 'Lt'}, {'Param': 0}, {'Int': [1, [254]]}]}
            elif kind == 'overpermissive': c['pre'] = {'Bool': True}
            elif kind == 'swapped': c['post']['Binary'][2]['Pair'][1].reverse()
            elif kind == 'missing': c['post']['Binary'][2]['Pair'][1].pop()
            elif kind == 'projection': c['post'] = {'Binary': [{'Eq': 'Spec'}, {'PairField': [0, {'Param': 1}]}, {'Param': 0}]}
            elif kind == 'wrong_scalar': c['post']['Binary'][2]['Pair'][0][0] = {'Signed': 8}
            elif kind == 'nested_type': c['post']['Binary'][2]['Pair'][0][0] = {'Option': {'Unsigned': 8}}
            elif kind == 'bad_index': c['post']['Binary'][2]['Pair'][1][0] = {'PairField': [2, {'Param': 1}]}
            elif kind == 'result_pre': c['pre'] = {'Binary': [{'Eq': 'Spec'}, {'Param': 1}, {'Param': 1}]}
            elif kind == 'wrong_version': c['version'] = 'verus-infer-v69:checked-goals-shared-option:4096'
            elif kind == 'deep':
                for _ in range(150): c['pre'] = {'Not': c['pre']}
            cache.write_bytes(b' ' * (2 * 1024 * 1024 + 1) if kind == 'oversize' else json.dumps(c).encode())
            # Reseed physical output: failure must remove an already-valid export.
            data = run('poison_' + kind, SPLIT, name='provider', ok=not expected_hit,
                       diag=('safe-domain maximality could not be proved' if kind in ['restrictive', 'narrow_domain'] else
                             'possible arithmetic' if kind == 'overpermissive' else 'postcondition not satisfied') if expected_hit else None, count=1)
            require(data['verus-infer']['functions'][0]['metrics']['cache'] == ('hit' if expected_hit else 'miss'), kind)
            if expected_hit:
                proofs = data['verus-infer']['functions'][0]['proofs']
                failed = 'maximality' if kind in ['restrictive', 'narrow_domain'] else 'soundness' if kind == 'overpermissive' else 'exactness'
                require(proofs[failed]['status'] != 'proved', kind + ': wrong failed obligation')
            cache.write_bytes(original)
            run('restore_' + kind, SPLIT, name='provider', count=1)
        guard.check()
        receipt['tools_after'] = guard.snapshot()


if __name__ == '__main__':
    main()
