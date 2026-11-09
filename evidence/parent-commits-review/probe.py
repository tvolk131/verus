#!/usr/bin/env python3
"""Focused d0d12c6f publication/report probes; artifacts go to private --output."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

VALID = 'use vstd::prelude::*; verus! { pub mod hidden {pub fn value()->(r:u8) ensures r==0,{0}} pub fn selected(){} }'
INVALID = VALID.replace('r==0', 'r==1')
FILTERS = [('--no-verify',), ('--no-verify', '--build-sst'), ('--verify-root',),
           ('--verify-root', '--verify-function', 'selected'),
           ('--verify-module', 'hidden'), ('--verify-only-module', 'hidden'),
           ('--verify-only-module', 'hidden', '--verify-function', 'value')]


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--verus', type=Path, required=True)
    p.add_argument('--solver', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output.mkdir(mode=0o700, parents=True, exist_ok=False)
    binary, solver = a.verus.resolve(), a.solver.resolve()
    env = dict(os.environ, VERUS_Z3_PATH=str(solver), VERUS_INFER_CACHE=str(a.output / 'cache'))
    receipt = {'compiler_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
               'solver_sha256': hashlib.sha256(solver.read_bytes()).hexdigest(), 'runs': []}

    receipt['bundle_sha256'] = {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                                for p in binary.parent.iterdir()
                                if p.is_file() and (p.name in ['rust_verify', 'libvstd.rlib', 'vstd.vir',
                                    'libverus_builtin.rlib', 'libverus_builtin_macros.so',
                                    'libverus_state_machines_macros.so'])}

    def run(label, name, source, flags=(), deps=(), ok=True, diag=None, publish=None, export=True):
        src = a.output / (name + '.rs')
        vir = a.output / (name + '.vir')
        src.write_text(source)
        (a.output / (label + '.source.rs')).write_text(source)
        cmd = [str(binary), str(src), '--crate-type=lib', '--edition=2021', '--crate-name', name,
               '--output-json', '--compile', '-o', str(a.output / f'lib{name}.rlib'),
               '-L', f'dependency={a.output}', *flags]
        if export:
            cmd += ['--export', str(vir)]
        for dep in deps:
            cmd += ['--extern', f'{dep}={a.output}/lib{dep}.rlib', '--import', f'{dep}={a.output}/{dep}.vir']
        proc = subprocess.run(cmd, env=env, capture_output=True, timeout=120)
        (a.output / (label + '.stdout')).write_bytes(proc.stdout)
        (a.output / (label + '.stderr')).write_bytes(proc.stderr)
        report = json.loads(proc.stdout)
        inf = report['verus-infer']
        row = {'label': label, 'command': cmd, 'exit': proc.returncode,
               'source_sha256': hashlib.sha256(source.encode()).hexdigest(),
               'stdout_sha256': hashlib.sha256(proc.stdout).hexdigest(),
               'stderr_sha256': hashlib.sha256(proc.stderr).hexdigest(),
               'published': vir.exists(), 'interface_written': inf['interface_written'],
               'verification_results': report['verification-results'],
               'scope': inf['verification_scope']['complete_selected_scope_verified'],
               'requested': inf['requested_functions'], 'decisions': inf['assistance_explanations'],
               'functions': inf['functions']}
        receipt['runs'].append(row)
        (a.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
        assert (proc.returncode == 0) == ok, (label, proc.stderr.decode())
        if diag:
            assert diag in proc.stderr.decode(), (label, proc.stderr.decode())
        if publish is not None:
            assert vir.exists() == publish, row
            assert inf['interface_written'] == publish, row
        if not ok:
            assert not inf['crate_success'] and not row['scope'], row
        print(label, proc.returncode, 'published=' + str(vir.exists()), flush=True)
        return report

    goal = ('--infer-goals', '--no-cheating')
    for i, flags in enumerate(FILTERS):
        seed = run(f'ordinary_seed_{i}', 'provider', VALID, goal, publish=True)
        assert not seed['verus-infer']['functions']
        r = run(f'ordinary_scope_{i}', 'provider', INVALID, goal + flags, ok=False,
                diag='requires full-crate verification', publish=False)
        assert not r['verus-infer']['functions']
    run('no_export_scope', 'no_export', INVALID, goal + ('--no-verify',), ok=False,
        diag='requires full-crate verification', publish=False, export=False)
    run('no_export_valid', 'no_export', VALID, goal, publish=False, export=False)
    for i, body in enumerate(['use vstd::prelude::*; verus! {}', 'use vstd::prelude::*; verus! {pub open spec fn only_spec()->bool{true}}']):
        run(f'zero_body_seed_{i}', f'zero_body_{i}', body, goal, publish=True)
        run(f'zero_body_scope_{i}', f'zero_body_{i}', body, goal + ('--no-verify',),
            ok=False, diag='requires full-crate verification', publish=False)
    run('ordinary_full_false_seed', 'provider', VALID, goal, publish=True)
    run('ordinary_full_false', 'provider', INVALID, goal, ok=False,
        diag='postcondition not satisfied', publish=False)
    run('ordinary_full_valid', 'provider', VALID, goal, publish=True)
    metadata_path = a.output / 'provider.vir'
    original = metadata_path.read_bytes()
    start = original.rindex(b'{"schema":')
    metadata = json.loads(original[start:-8])
    assert metadata['policy'] == 'candidate-bound-evidence-v64' and metadata['schema'] == 38
    assert not metadata['proofs']

    caller = 'use vstd::prelude::*; verus! { pub fn selected(){let r=provider::hidden::value(); assert(r==0);} }'
    for assisted in [False, True]:
        flags = goal if assisted else ()
        run(f'consumer_valid_{assisted}', 'consumer', caller, flags, ('provider',), publish=True)
        for i, partial in enumerate(FILTERS):
            # Re-seed each stale output, then fail with a scope diagnostic, never a frontend error.
            run(f'consumer_seed_{assisted}_{i}', 'consumer', caller, flags, ('provider',), publish=True)
            # Module names must belong to the consumer, but scope rejection precedes name resolution.
            run(f'consumer_scope_{assisted}_{i}', 'consumer', caller, flags + partial, ('provider',),
                ok=False, diag='requires full-crate verification', publish=False)
        run(f'consumer_false_{assisted}', 'consumer', caller.replace('r==0', 'r==1'), flags,
            ('provider',), ok=False, diag='assertion failed', publish=False)

    run('transitive_middle', 'middle', caller, (), ('provider',), publish=True)
    leaf = 'use vstd::prelude::*; verus! {pub fn selected(){middle::selected();}}'
    run('transitive_leaf', 'leaf', leaf, (), ('provider', 'middle'), publish=True)
    run('transitive_partial', 'leaf', leaf, ('--no-verify',), ('provider', 'middle'),
        ok=False, diag='requires full-crate verification', publish=False)
    for assisted in [False, True]:
        for version in ['candidate-bound-evidence-v63', 'candidate-bound-evidence-v62']:
            run(f'stale_seed_{assisted}_{version[-3:]}', 'consumer', caller,
                goal if assisted else (), ('provider',), publish=True)
            changed = dict(metadata, policy=version)
            metadata_path.write_bytes(original[:start] + json.dumps(changed, separators=(',', ':')).encode() + b'VRINFER1')
            run(f'stale_{assisted}_{version[-3:]}', 'consumer', caller,
                goal if assisted else (), ('provider',), ok=False,
                diag='incompatible inference schema', publish=False)
            metadata_path.write_bytes(original)

    # Disabled/filtered legacy reports: successful command is not full verification evidence.
    for i, partial in enumerate([('--no-verify',), ('--verify-root',),
                                  ('--verify-root', '--verify-function', 'selected')]):
        r = run(f'legacy_reporting_{i}', 'legacy', INVALID, partial)
        assert 'success' not in r['verification-results']
        assert r['verification-results']['is-verifying-entire-crate'] is False
        assert r['verus-infer']['verification_scope']['complete_selected_scope_verified'] is False
        assert r['verus-infer']['assistance_explanations']['decisions'] == []
        assert not (a.output / 'legacy.vir').read_bytes().endswith(b'VRINFER1')

    transport = Path(__file__).with_name('transport.rs').read_text()
    first = None
    for phase in ['cold', 'warm']:
        r = run(f'transport_{phase}', 'transport', transport, goal, publish=True)
        explanation = r['verus-infer']['assistance_explanations']
        assert explanation['preparation_completed'] is True
        assert explanation['decisions'] == sorted(explanation['decisions'], key=lambda d: (d['profile'], d['function']))
        decision = [d for d in explanation['decisions'] if d['profile']=='pointwise_transport' and d['function'].endswith('::advance')]
        assert len(decision)==1 and decision[0]['status']=='applied', decision
        if first is not None:
            assert first == explanation
        first = explanation
    variants = [
        ('missing_goal', transport.replace('previous.valid() ==> r.valid(),', ''), True, 'skipped', 'missing_transport_goal', None),
        ('false_transition', transport.replace('epoch:previous.epoch+1', 'epoch:0'), False, 'applied', None, 'postcondition not satisfied'),
        ('unconditional', transport.replace('previous.valid() ==> r.valid()', 'r.valid()'), False, 'skipped', 'missing_transport_goal', 'postcondition not satisfied'),
        ('false_epoch', transport.replace('r@.1==previous@.1+1', 'r@.1==previous@.1'), False, 'applied', None, 'postcondition not satisfied'),
        ('choose', transport.replace('self@.0[i].unwrap().1<self@.1', 'self@.0[i].unwrap().1<(choose|epoch:u16|epoch==self@.1)'), False, 'error', 'preparation_error', 'annotation'),
    ]
    for label, source, ok, status, reason, diag in variants:
        run('transport_seed_' + label, 'transport', transport, goal, publish=True)
        r = run('transport_' + label, 'transport', source, goal, ok=ok, diag=diag, publish=ok)
        explanation = r['verus-infer']['assistance_explanations']
        d = [d for d in explanation['decisions'] if d['profile']=='pointwise_transport' and d['function'].endswith('::advance')]
        assert len(d)==1 and d[0]['status']==status and d[0]['reason']==reason, d
        if status=='error':
            assert explanation['preparation_started'] and not explanation['preparation_completed']
            assert all(d['profile']=='pointwise_transport' for d in explanation['decisions'])
    # Multi-function failure: earlier actual attempt persists, later one must be absent.
    source = variants[-1][1].replace('fn advance<', 'fn middle_error<')
    source = source.replace('fn middle_error<', 'fn aaa_before(){}\n fn middle_error<')
    source += '\nverus! {fn zzz_after(){}}\n'
    r = run('partial_order', 'transport_order', source, goal, ok=False, diag='annotation', publish=False)
    ds = r['verus-infer']['assistance_explanations']['decisions']
    assert len([d for d in ds if d['status']=='error'])==1
    assert all(d['profile']=='pointwise_transport' for d in ds)
    assert any(d['function'].endswith('::aaa_before') for d in ds)
    assert not any(d['function'].endswith('::zzz_after') for d in ds)
    run('frontend_seed', 'provider', VALID, goal, publish=True)
    r = run('frontend_error', 'provider', 'use vstd::prelude::*; verus! {pub fn wrong()->u8{false}}', goal,
            ok=False, diag='mismatched types', publish=False)
    e = r['verus-infer']['assistance_explanations']
    assert not e['preparation_started'] and not e['preparation_completed'] and not e['decisions']
    receipt['all_expectations_passed'] = True
    (a.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print('ALL EXPECTATIONS PASSED:', len(receipt['runs']), 'runs')


if __name__ == '__main__':
    main()
