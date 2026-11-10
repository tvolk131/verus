#!/usr/bin/env python3
"""Frozen-input canonical ordinary-goal replay; no builds or application writes."""
import argparse
import json
import os
from pathlib import Path
import re
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import (LoggedCommands, replay_receipt, require, require_exact,
    require_failure_kind, require_function_coverage, require_publication, validate_goal_run, validate_run)
from fidelity import fidelity
from copy_bundle import ENGINE_SOURCES

HELPER = '\nverus! { #[verifier::infer] pub fn identity(n: u64) -> u64 { n } }\n'


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--bundle', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=True)
    out = Path(tempfile.mkdtemp(prefix='canonical-', dir=a.output.resolve()))
    receipt = dict(output=str(out), commands=[], runs=[])
    with replay_receipt(out, receipt):
        replay(a, out, receipt)
    print(out / 'receipt.json')


def replay(a, out, receipt):
    for f in HERE.glob('freeze-*.json'):
        for name, sha in json.loads(f.read_text())['files'].items():
            require(digest(HERE / name) == sha, 'prior freeze changed: ' + name)
    for n in ('004', '006', '007', '008'):
        fidelity(HERE / f'candidate-{n}.rs')
    receipt['sources'] = {str(f.relative_to(ROOT)): digest(f) for f in
        [*HERE.glob('*.py'), *HERE.glob('*.rs'), *[ROOT / n for n in ENGINE_SOURCES]]}
    guard = ArtifactGuard(tool_artifacts(a.bundle / 'verus', a.bundle / 'z3',
        extra=[ROOT / n for n in receipt['sources']]))
    receipt['artifacts'] = guard.initial
    env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
        RUSTUP_TOOLCHAIN='1.99.1', VERUS_Z3_PATH=str(a.bundle / 'z3'),
        VERUS_INFER_CACHE=str(out / 'cache'))
    invoke = LoggedCommands(out, out, receipt['commands'], timeout=180, guard=guard,
        status_key='returncode')

    def run(label, text, success, *, goals=True, helpers=(), proofs=(), execs=('u64',),
            kind=None, diagnostic_only=False, opts=(), assistance=None, rejection=None, source_name=None, poisoned_helper=False, diagnostic_kind=None):
        source = out / ((source_name or label) + '.rs')
        source.write_text(text)
        frozen = dict(label=label, source_sha256=digest(source), expected_helpers=list(helpers),
            expected_execs=list(execs), expected_proofs=list(proofs), success=success,
            expected_failure_kind=kind, diagnostic_only=diagnostic_only, assistance=assistance, rejection=rejection, poisoned_helper=poisoned_helper, diagnostic_kind=diagnostic_kind)
        if poisoned_helper:
            frozen['poisoned_cache_hashes'] = {str(f.name):digest(f) for f in (out/'cache').glob('*.json')}
        (out / (label + '.freeze.json')).write_text(json.dumps(frozen, indent=2) + '\n')
        # Each candidate is immutable and hashed before the trial, including variants.
        source_guard = ArtifactGuard([source])
        command = [str(a.bundle / 'verus'), str(source), '--crate-type=lib', '--no-cheating',
            '--output-json', '--multiple-errors', '20', *(['--infer-goals'] if goals else []),
            *map(str, opts)]
        rc = invoke(label, command, env)
        source_guard.check()
        result = dict(returncode=rc, stdout=(out / (label + '.stdout')).read_text(),
            stderr=(out / (label + '.stderr')).read_text())
        data = json.loads(result['stdout'])
        record = dict(**frozen, verification=data.get('verification-results'),
            stdout_sha256=digest(out / (label + '.stdout')),
            stderr_sha256=digest(out / (label + '.stderr')),
            inference=data.get('verus-infer'), failures=[])
        receipt['runs'].append(record)
        # Require helper identities and all three precision checks before property gates.
        functions = data.get('verus-infer', {}).get('functions', [])
        require_function_coverage(functions, set(helpers))
        if poisoned_helper:
            require(not success and len(functions)==1 and functions[0].get('required_checks_passed') is False
                and all(functions[0].get('proofs',{}).get(k,{}).get('status')=='not_proved'
                    and functions[0]['proofs'][k].get('required') is True for k in ('soundness','exactness')),
                'intentional helper poison did not fail mandatory checks')
        else:
            require_exact(functions)
        if success or kind:
            validate_goal_run(result, success, expected_functions=set(helpers),
                exec_bodies=set(execs), proof_bodies=set(proofs), failure_kind=kind)
        else:
            validate_run(result, False, allow_frontend=rejection is not None)
        v = data['verification-results']
        if rejection:
            require(rejection in result['stderr'], 'expected intentional rejection missing')
            require(not any(k in result['stderr'] for k in ('postcondition not satisfied',
                'precondition not satisfied', 'assertion failed')), 'poison rejection mixed with semantic failure')
        else:
            require(v.get('is-verifying-entire-crate') and not v.get('encountered-vir-error'),
                'frontend/partial run is not semantic evidence')
        require(not re.search(r'error\[E[0-9]+\]', result['stderr']), 'Rust frontend error')
        if diagnostic_kind:
            require_failure_kind(result['stderr'], diagnostic_kind)
            require(not any(k in result['stderr'] for k in ('invariant not satisfied',
                'possible arithmetic underflow/overflow')), 'mixed diagnostic-only failure')
            require(v.get('errors')==1, 'diagnostic-only control has unexpected error count')
            selected=[f for f in data['verus-infer']['verification_scope']['native_functions']
                if f['body_status']=='selected_body_not_certified_by_successful_full_run']
            require_function_coverage(selected,set(execs)|set(proofs))
        if kind:
            require_failure_kind(result['stderr'], kind)
            require('invariant not satisfied' not in result['stderr'] and
                    'loop invariant not satisfied' not in result['stderr'], 'mixed semantic failure')
        if success:
            scope = data['verus-infer']['verification_scope']
            require(scope['complete_selected_scope_verified'], 'incomplete checked scope')
            for status, expected in [('checked_exec_body', set(execs)),
                                     ('checked_proof_body', set(proofs))]:
                names = [f['function'].rsplit('::', 1)[-1] for f in scope['native_functions']
                    if f['body_status'] == status]
                require(len(names) == len(set(names)) and set(names) == expected,
                    f'{label}: unexpected checked local bodies {status}: {names}')
        decisions = data.get('verus-infer', {}).get('assistance_explanations', {}).get('decisions', [])
        applied = any(d.get('profile') == 'mutable_buffer_u64_radix128' and
            d.get('status') == 'applied' for d in decisions)
        if assistance is not None:
            require(applied == assistance, 'assistance selection mismatch: ' + label)
        print(label, v.get('verified'), v.get('errors'), flush=True)
        return data

    manual = (HERE / 'candidate-004.rs').read_text()
    automatic = (HERE / 'candidate-008.rs').read_text()
    run('ordinary_canonical', manual, True, goals=False,
        proofs=('radix_positive', 'quotient_step', 'byte_step'), assistance=False)
    for label in ('canonical_cold', 'canonical_warm'):
        run(label, automatic, True, assistance=True, source_name='canonical')
    run('disabled', automatic, False, goals=False, diagnostic_only=True, assistance=False)
    length = automatic.replace('canonical(number, encoded@)', '1 <= encoded@.len() <= 10').replace(
        'pub fn u64(', 'pub fn safety_length(')
    run('safety_length', length, True, execs=('safety_length',), assistance=True)
    healthy = automatic + HELPER
    run('healthy_exact_helper', healthy, True, helpers=('identity',),
        execs=('u64', 'identity'), assistance=True)
    # Only the requested output goal changes: generated proposals stay healthy.
    bad = healthy + '''
verus! { fn false_goal(n: u64, b: &mut [u8;10]) -> (r: &[u8])
    ensures r@.len() == 0,
{ u64(n,b) } }
'''
    run('false_return_length', bad, False, helpers=('identity',),
        kind='postcondition not satisfied', execs=('u64', 'identity', 'false_goal'), assistance=True)
    for label, goal in [('false_payload', 'r@[0] == 255'),
                        ('false_continuation', 'r@[r@.len() - 1] >= 128'),
                        ('false_minimal_length', 'r@.len() == 10')]:
        caller = healthy + '\nverus! { fn false_goal(n:u64,b:&mut[u8;10])->(r:&[u8]) ensures ' + goal + ', {u64(n,b)} }\n'
        run(label, caller, False, helpers=('identity',),
            kind='postcondition not satisfied', execs=('u64', 'identity', 'false_goal'), assistance=True)
    for label, old, new in [
        ('wrong_shift', 'n >>= 7;', 'n >>= 6;'),
        ('wrong_break', 'if n == 0', 'if n == 1'),
        ('wrong_counter', 'i += 1', 'i += 2'),
        ('wrong_payload', '*b = n as u8 | 0x80;', '*b = 0x80;'),
        ('wrong_continuation', '*b &= 0x7f;', '*b &= 0xff;'),
        ('step8', 'n >>= 7;', 'n >>= 8;'),
        ('capacity9', 'U64_LEN: usize = 10', 'U64_LEN: usize = 9'),
        ('width32', 'number: u64', 'number: u32'),
    ]:
        run(label, automatic.replace(old, new), False, diagnostic_only=True, assistance=False)
    generic_radix = automatic.replace('pub open spec fn radix_power(exponent:',
        'pub open spec fn radix_power<T>(exponent:').replace('radix_power(', 'radix_power::<u8>(').replace(
        '128 * radix_power::<u8>', '128 * radix_power::<T>')
    run('generic_radix', generic_radix, False, diagnostic_only=True, assistance=False)
    run('wrong_return_range', healthy.replace('&$buf[0..=i]', '&$buf[0..i]'), False,
        helpers=('identity',), diagnostic_only=True, assistance=True,
        diagnostic_kind='postcondition not satisfied', execs=('u64','identity'))
    provider_opts = ['--crate-name=provider', '--compile', '--export', out / 'provider.vir']
    for label in ('provider_cold', 'provider_warm'):
        d = run(label, healthy, True, helpers=('identity',), execs=('u64', 'identity'),
            opts=provider_opts, assistance=True, source_name='provider')
        require_publication(d, True, out / 'provider.vir')
        require(d['verus-infer']['functions'][0]['metrics']['cache'] ==
                ('hit' if label.endswith('warm') else 'miss'), 'incorrect helper cache provenance')
    # A typed cached candidate poison must fail helper checks; it is NOT an
    # ordinary semantic control and never passes the healthy-helper gate.
    cache_files = [f for f in (out / 'cache').glob('*.json')
                   if json.loads(f.read_text()).get('post') == {'Binary': [{'Eq':'Spec'}, {'Param':1}, {'Param':0}]}]
    require(len(cache_files)==7, 'unexpected explicit identity cache inventory')
    saved_cache = {f: f.read_bytes() for f in cache_files}
    for f in cache_files:
        cached = json.loads(f.read_text())
        cached['post'] = {'Binary': [{'Eq':'Spec'}, {'Param':1}, {'BitNot':[{'Width':64}, {'Param':0}]}]}
        f.write_text(json.dumps(cached))
    original_provider = (out / 'provider.vir').read_bytes()
    try:
        d = run('poisoned_helper', healthy, False, helpers=('identity',),
                opts=provider_opts, assistance=True, poisoned_helper=True, source_name='provider')
        require(d['verus-infer']['functions'][0]['metrics']['cache']=='hit', 'poisoned candidate was not consumed')
        require_publication(d, False, out / 'provider.vir')
    finally:
        (out / 'provider.vir').write_bytes(original_provider)
        for f, original in saved_cache.items():
            f.write_bytes(original)
    consumer = '''use vstd::prelude::*;verus! {
        fn caller(n:u64, b:&mut [u8;10]) { let r=provider::u64(n,b); assert(provider::canonical(n,r@)); }
    }'''
    run('consumer', consumer, True, execs=('caller',), opts=['--extern',
        'provider=' + str(out / 'libprovider.rlib'), '--import',
        'provider=' + str(out / 'provider.vir')], assistance=False)
    # Intentional metadata poison is distinct from an ordinary semantic negative.
    artifact = out / 'provider.vir'
    saved = artifact.read_bytes()
    position = saved.rindex(b'{"schema":')
    metadata = json.loads(saved[position:-8])
    metadata['policy'] = 'intentionally-invalid-buffer-proof-policy'
    artifact.write_bytes(saved[:position] + json.dumps(metadata, separators=(',', ':')).encode() + b'VRINFER1')
    try:
        run('metadata_poison', consumer, False, rejection='rebuild the library',
            opts=['--extern', 'provider=' + str(out / 'libprovider.rlib'), '--import',
                  'provider=' + str(artifact)])
    finally:
        artifact.write_bytes(saved)
    failed = out / 'failed.vir'
    failed.write_bytes((out / 'provider.vir').read_bytes())
    d = run('seeded_failed_export', bad, False, helpers=('identity',),
        kind='postcondition not satisfied', execs=('u64', 'identity', 'false_goal'), opts=['--crate-name=failed', '--export', failed],
        assistance=True)
    require_publication(d, False, failed)
    guard.check()
    receipt['artifacts_after'] = guard.snapshot()

if __name__ == '__main__':
    main()
