#!/usr/bin/env python3
"""Fresh, full-crate replay. Never builds or writes application checkouts."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
HISTORY = HERE.parent / 'redb_single_word_set'
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, tool_artifacts
from infer_experiment import (LoggedCommands, replay_receipt, require,
                              require_exact, require_publication, validate_run)

def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()

def fidelity():
    for manifest in ('freeze.json', 'reduction-freeze.json'):
        for name, digest in json.loads((HERE/manifest).read_text())['files'].items():
            require(sha(HERE/name) == digest, f'frozen fixture changed: {name}')
    sys.path.insert(0, str(HISTORY))
    from replay import fidelity as historical_fidelity
    historical_fidelity(HERE / 'automatic.rs')
    historical_fidelity(HERE / 'automatic_alias.rs')
    historical_fidelity(HERE / 'automatic_inferred_mask.rs')
    original = (HISTORY / 'baseline.rs').read_text()
    adapted = original.replace('(word & (1u64 << (j as u64))) != 0', 'vstd::word::member(word, j)')
    adapted = adapted.replace('Seq::new(len as nat, |j: int| member(word, j))', 'vstd::word::view(word, len as nat)')
    require(adapted == (HERE / 'automatic.rs').read_text(), 'executable/semantic drift')
    aliased = adapted.replace('pub open spec fn member(word: u64, j: int) -> bool {\n    vstd::word::member(word, j)\n}', 'pub use vstd::word::member;')
    require(aliased == (HERE/'automatic_alias.rs').read_text(), 'alias changed semantics/body')
    mask_contract = '    fn select_mask(bit: usize) -> (r: u64)\n        requires bit < 64,\n        ensures r == 1u64 << (bit as u64),'
    inferred = aliased.replace(mask_contract, '    #[verifier::infer]\n    fn select_mask(bit: usize) -> (r: u64)')
    require(inferred == (HERE/'automatic_inferred_mask.rs').read_text(), 'inferred-mask fixture drift')
    for name in ('automatic.rs', 'automatic_alias.rs', 'automatic_inferred_mask.rs', 'permission.rs', 'consumer.rs'):
        s = (HERE / name).read_text()
        require(not any(x in s for x in ('proof {', 'let ghost', 'word_laws', 'lemma_', 'assume(', 'admit(', 'external_body')),
                f'local proof assistance or trust added: {name}')

def check_run(result, success, expected_error=None):
    data = validate_run(result, success, allow_frontend=expected_error == 'Verus Infer unsupported')
    verification = data['verification-results']
    info = data.get('verus-infer', {})
    if expected_error:
        require(expected_error in result['stderr'], 'expected diagnostic missing')
    if expected_error != 'Verus Infer unsupported':
        require(not verification.get('encountered-vir-error') and verification.get('is-verifying-entire-crate'),
                'frontend failure or partial verification')
    if expected_error == 'postcondition not satisfied':
        require(not any(x in result['stderr'] for x in ('precondition not satisfied', 'assertion failed')),
                'mixed failure kinds')
    if success:
        require(info['verification_scope']['complete_selected_scope_verified'], 'incomplete verification scope')
        require_exact(info['functions'])
    return data

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--verus', type=Path, default=ROOT / 'source/target-verus/release/verus')
    a = p.parse_args()
    fidelity()
    out = Path(tempfile.mkdtemp(prefix='word-goals-', dir=a.output.resolve()))
    os.chmod(out, 0o700)
    receipt = {'output': str(out), 'runs': [], 'commands': [], 'sources': {}, 'tools': {}}
    with replay_receipt(out, receipt):
        replay(a, out, receipt)
    print(out/'receipt.json')

def replay(a, out, receipt):
    env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
               RUSTUP_TOOLCHAIN='1.99.1', VERUS_Z3_PATH=str(ROOT / 'source/z3'),
               VERUS_INFER_CACHE=str(out/'cache'))
    for folder in (HERE, ROOT / 'source/vir/src/infer', ROOT / 'source/vstd'):
        for f in folder.glob('*.rs'):
            receipt['sources'][str(f.relative_to(ROOT))] = sha(f)
    for f in (ROOT/'source/vir/src/infer.rs', ROOT/'source/rust_verify/src/infer_report.rs'):
        receipt['sources'][str(f.relative_to(ROOT))] = sha(f)
    guard = ArtifactGuard(tool_artifacts(a.verus.resolve(), ROOT / 'source/z3', extra=[
        *(ROOT / f for f in receipt['sources']), Path(__file__),
        ROOT/'source/tools/infer_experiment.py', ROOT/'source/tools/infer_eval_support.py']))
    receipt['tools'] = guard.initial
    invoke = LoggedCommands(out, out, receipt['commands'], timeout=240, guard=guard,
                            status_key='returncode')
    def run(label, text, success, opts=(), expected_error=None, source_name=None):
        source = out / ((source_name or label) + '.rs')
        source.write_text(text)
        command = [str(a.verus.resolve()), str(source), '--no-cheating', '--output-json',
                   '--multiple-errors', '20', *map(str, opts)]
        rc = invoke(label, command, env)
        result = dict(returncode=rc, stdout=(out/(label+'.stdout')).read_text(),
                      stderr=(out/(label+'.stderr')).read_text())
        d = json.loads(result['stdout'])
        v = d['verification-results']
        info = d.get('verus-infer', {})
        rec = dict(label=label, command=command, returncode=rc,
                   source_sha256=sha(source), stdout_sha256=sha(out/(label+'.stdout')),
                   stderr_sha256=sha(out/(label+'.stderr')), verification=v,
                   assistance=info.get('goal_assistance'), functions=info.get('functions'),
                   verification_scope=info.get('verification_scope'),
                   interface_written=info.get('interface_written'),
                   imported_inference_dependencies=info.get('imported_inference_dependencies'))
        receipt['runs'].append(rec)
        (out/'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
        check_run(result, success, expected_error)
        print(label, v.get('verified'), v.get('errors'), flush=True)
        return d
    automatic = (HERE/'automatic_alias.rs').read_text()
    permission = (HERE/'permission.rs').read_text()
    goals = ['--infer-goals']
    run('historical_baseline', (HISTORY/'baseline.rs').read_text(), False, goals, 'postcondition not satisfied')
    run('representation_equivalence', (HERE/'representation_equivalence.rs').read_text(), True, goals)
    run('forwarding_wrapper', (HERE/'automatic.rs').read_text(), False, goals, 'postcondition not satisfied')
    d = run('automatic', automatic, True, goals)
    require(any(x['strategy'] == 'word_view_broadcast' for x in d['verus-infer']['goal_assistance']),
            'word assistance was not selected')
    run('disabled', automatic, False, (), 'postcondition not satisfied')
    run('permission', permission, True, goals)
    run('automatic_inferred_mask', (HERE/'automatic_inferred_mask.rs').read_text(), True, goals)
    run('xor', permission.replace('rights | (1u64', 'rights ^ (1u64'), False, goals, 'postcondition not satisfied')
    run('stateful_xor', automatic.replace('group |=', 'group ^='), False, goals, 'postcondition not satisfied')
    run('padding_negative', automatic.replace('forall|j: int| self.len <= j < 64 ==> member(self.data@[0], j)', 'true'), False, goals, 'postcondition not satisfied')
    run('return_negative', automatic.replace('group == u64::MAX', 'group != u64::MAX'), False, goals, 'postcondition not satisfied')
    run('count_negative', permission.replace('{ 0nat } else { 1nat }', '{ 1nat } else { 0nat }'), False, goals, 'postcondition not satisfied')
    run('helper_tuple', (HERE/'helper_tuple.rs').read_text(), False, goals, 'Verus Infer unsupported')
    run('helper_shift', (HERE/'helper_shift.rs').read_text(), True, goals)
    provider = (HERE/'provider.rs').read_text()
    for label in ('provider_cold', 'provider_warm'):
        d = run(label, provider, True, [*goals, '--crate-type=lib', '--crate-name=provider', '--compile',
                                  '--export', out/'provider.vir', '-o', out/'libprovider.rlib'], source_name='provider')
        require(d['verus-infer']['functions'][0]['metrics']['cache'] == ('hit' if label.endswith('warm') else 'miss'),
                f'{label}: incorrect cache provenance')
        require_publication(d, True, out/'provider.vir')
    d = run('consumer', (HERE/'consumer.rs').read_text(), True,
        [*goals, '--extern', 'provider='+str(out/'libprovider.rlib'), '--import', 'provider='+str(out/'provider.vir')])
    require(d['verus-infer']['imported_inference_dependencies'], 'missing checked dependency identity')
    # A failed original goal must prevent publishing an otherwise valid inferred interface.
    bad = provider + '\nverus! { fn false_goal() { assert(false); } }\n'
    failed_export = out/'failed.vir'
    failed_export.write_bytes((out/'provider.vir').read_bytes())
    d = run('publication_negative', bad, False, [*goals, '--crate-type=lib', '--crate-name=failed',
        '--export', failed_export], 'assertion failed')
    require_publication(d, False, failed_export)
    guard.check()
    receipt['artifacts_after'] = guard.snapshot()

if __name__ == '__main__':
    main()
