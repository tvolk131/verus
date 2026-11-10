#!/usr/bin/env python3
"""Read-only review audit; writes compact evidence here, never proof inputs.

Run after the separate, normal check.py replay has completed. No controller
imports, mutation controls, compiler invocations, or other checkout reads.
"""
import hashlib
import json
import re
from pathlib import Path

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[1]
LANE = ROOT / 'source/tools/research/native_mir_lrust_refinement'
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
WORKER = Path('/tmp/native-mir-lrust-c6116d49-trial018')
REPLAY = Path('/tmp/native-mir-lrust-review-a13f0551-replay')
FE = 'Coq.Logic.FunctionalExtensionality.functional_extensionality_dep'
EQ = 'Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq'


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def read(p):
    return json.loads(Path(p).read_bytes())


def need(ok, msg):
    if not ok:
        raise ValueError(msg)


def pinned(p, expected):
    need(sha(p) == expected, 'hash mismatch: ' + str(p))


def write(name, data):
    (OUT / name).write_text(json.dumps(data, indent=2) + '\n')


def local_input(p):
    # Authenticate recorded repo inputs against THIS prepared checkout only.
    if p.startswith('/home/dev/.t3/worktrees/verus/'):
        parts = Path(p).parts
        return ROOT.joinpath(*parts[7:])
    need(p.startswith('/tmp/'), 'unrecognized input root: ' + p)
    need('parent-native-primitives' not in p, 'live output forbidden')
    return Path(p)


def inventory(d, expected):
    text = (d / 'assumptions.stdout').read_text()
    sections = re.findall(r'^BEGIN (\S+)\n(.*?)^END (\S+)\s*$', text, re.M | re.S)
    need([x for x, _, _ in sections] == list(expected) == [x for _, _, x in sections], 'inventory markers')
    aliases = {'functional_extensionality_dep': FE,
               'FunctionalExtensionality.functional_extensionality_dep': FE}
    actual = {}
    for name, body, _ in sections:
        if body.strip() == 'Closed under the global context':
            actual[name] = []
        else:
            need(body.startswith('Axioms:\n'), 'unexpected assumptions format')
            names = re.findall(r'^(\S+)\s*:', body.split('\n', 1)[1], re.M)
            actual[name] = [aliases.get(x, x) for x in names]
            need(actual[name] == [FE], 'unexpected assumptions')
    need(actual == expected, 'assumptions differ')
    return actual


def run_audit(d, expected):
    r = read(d / 'receipt.json')
    need(r['success'] is True and r['model_proofs_kernel_checked'] is True, 'unfinished/failed replay')
    need(r['publication'] is False and r['source_entry_proved'] is False and r['native_effects_discharged'] == 0, 'scope changed')
    stages = ['compile-fragment', 'compile-simulation', 'compile-physical', 'compile-reified', 'assumptions', 'kernel']
    need([c['label'] for c in r['commands']] == stages, 'command inventory')
    need(all(c['exit_code'] == 0 for c in r['commands']), 'failed command')
    for p, facts in r['inputs'].items():
        pinned(local_input(p), facts['sha256'])
    for p, facts in r['proof_artifacts'].items():
        need(Path(p).parent == d / 'proofs', 'foreign proof product')
        pinned(p, facts['sha256'])
    pinned(d / 'freeze.json', sha(LANE / 'final-freeze.json'))
    for n in ['fragment', 'simulation', 'physical', 'reified']:
        pinned(d / 'proofs' / (n + '.v'), sha(LANE / 'candidates' / (n + '.v')))
    inventory(d, expected['assumptions'])
    need(r['assumptions'] == expected['assumptions'], 'receipt assumptions mismatch')
    need(r['kernel_axioms'] == [FE, EQ], 'kernel axioms changed')
    need(r['source_controls'] == expected['source_controls'] and len(r['source_controls']) == 7, 'controls changed')
    need(read(d / 'source-check.json') == read(LANE / 'source-coverage.json'), 'coverage mismatch')
    need(read(d / 'source-controls.json') == r['source_controls'], 'control log mismatch')
    need(r['constant_count'] == 121 and r['program_obligations'] == 0, 'declaration count')
    kernel = (d / 'kernel.stderr').read_text()
    need(kernel.count('CONTEXT SUMMARY') == 1, 'kernel summary missing')
    for s in ['Theory: Set is predicative', 'Theory: Rewrite rules are not allowed',
              'Constants/Inductives relying on type-in-type: <none>',
              'Constants/Inductives relying on unsafe (co)fixpoints: <none>',
              'Inductives whose positivity is assumed: <none>']:
        need(s in kernel, 'unsafe kernel context')
    logs = {p.name: sha(p) for p in sorted(d.iterdir()) if p.suffix in ['.stdout', '.stderr', '.json']}
    objects = {p.name: sha(p) for p in sorted((d / 'proofs').glob('*.vo'))}
    need(objects == expected['proof_objects'], 'five proof objects differ')
    return dict(path=str(d), receipt_sha256=sha(d / 'receipt.json'),
                authenticated_inputs=len(r['inputs']), proof_artifacts=len(r['proof_artifacts']),
                objects=objects, logs_and_json_sha256=logs, commands=r['commands'])


def main():
    expected = read(LANE / 'results.json')
    pinned(WORKER / 'receipt.json', 'e98d875a3062053a87e0030d9ee9ec71536a58a942f5ddf182a1cbd2162f3998')
    pinned(LANE / 'final-freeze.json', expected['final_freeze_sha256'])
    pinned(WORKER / 'kernel.stderr', expected['kernel']['sha256'])
    frozen = read(LANE / 'final-freeze.json')
    for p, digest in frozen['files'].items():
        pinned(ROOT / p, digest)
    assumptions = expected['assumptions']
    need(list(assumptions) == [m + '.' + n for m, ns in frozen['constants'].items() for n in ns], 'freeze inventory')
    need(len(assumptions) == 121 and sum(not v for v in assumptions.values()) == 119, 'assumption counts')
    generated = [n for n in assumptions if re.search(r'_(rect|ind|rec|sind)$', n)]
    need(len(generated) == 24, 'eliminator count')
    for module, frozen_names in frozen['constants'].items():
        text = (LANE / 'candidates' / (module + '.v')).read_text()
        explicit = re.findall(r'^(?:Definition|Fixpoint|Inductive|Lemma|Theorem|Corollary)\s+(\w+)', text, re.M)
        explicit += re.findall(r'^\|\s+(\w+)', text, re.M)
        implicit = [n.split('.', 1)[1] for n in generated if n.startswith(module + '.')]
        need(set(explicit + implicit) == set(frozen_names) and len(explicit + implicit) == len(frozen_names), 'source declaration coverage')
        need(not re.search(r'\b(?:Axiom|Admitted|admit|Parameter|Program|Obligation|Abort|vm_cast_no_check|native_compute)\b', text), 'unchecked source construct')
    need([n for n, v in assumptions.items() if v] == [
        'physical.offset_then_original_read_resolve', 'physical.interior128_offset_read_adequacy'], 'FE inventory')
    source = Path('/tmp/native-semantic-bridge-e4a8dcab/source-v1')
    binding = read(ROOT / 'source/tools/research/native_semantic_bridge/source-bindings.json')
    source_hashes = {'receipt.json': binding['receipt_sha256'], **binding['outputs']}
    for p, digest in source_hashes.items():
        pinned(source / p, digest)
    need(read(source / 'receipt.json')['success'] is True, 'source receipt failed')
    native, borrow = read(source / 'drop-native.json'), read(source / 'drop-borrowck.json')
    fixture = ROOT / binding['source_fixture']
    need(sha(fixture) == native['input_sha256'] == borrow['source_sha256'], 'source fixture mismatch')
    trials = []
    development = read(LANE / 'development.json')
    for t in development['trials']:
        d = Path(t['path'])
        pinned(d / 'receipt.json', t['receipt_sha256'])
        pinned(d / 'freeze.json', t['freeze_sha256'])
        need(read(d / 'receipt.json')['success'] is t['success'], 'trial status mismatch')
        for n, digest in t['sources'].items():
            pinned(d / 'proofs' / n, digest)
        trials.append(dict(trial=t['trial'], path=str(d), success=t['success'], receipt_sha256=t['receipt_sha256']))
    need(sum(not t['success'] for t in trials) == 16, 'development count')
    worker = run_audit(WORKER, expected)
    replay = run_audit(REPLAY, expected)
    need({k: v for k, v in worker['logs_and_json_sha256'].items() if k != 'receipt.json'} ==
         {k: v for k, v in replay['logs_and_json_sha256'].items() if k != 'receipt.json'},
         'deterministic logs or source outputs differ')
    ready_path = BASE / 'rocq-toolchain/baseline-ready.json'
    ready = read(ready_path)
    baseline = read(ready['receipt'])
    need(ready['baseline_kernel_checked'] is True and baseline['success'] is True, 'foundation status')
    for c in baseline['commands']:
        if c['label'] in ['prover-version', 'build-baseline', 'kernel-check-baseline']:
            need(c['returncode'] == 0, 'foundation stage failed')
            pinned(c['log'], c['log_sha256'])
    version_command = next(c for c in baseline['commands'] if c['label'] == 'prover-version')
    need('version 8.20.0' in Path(version_command['log']).read_text(), 'prover version')
    build = Path(ready['build'])
    for p, digest in baseline['copied_sources'].items():
        pinned(build / p, digest)
    object_map = {str(p.relative_to(build)): sha(p) for p in (build / 'src').rglob('*.vo')}
    map_digest = hashlib.sha256(json.dumps(object_map, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    baseline_binding = read(ROOT / 'source/tools/research/native_semantic_bridge/baseline-bindings.json')
    need(len(baseline['copied_sources']) == 523 and len(object_map) == 284, 'foundation counts')
    need(map_digest == baseline_binding['compiled_module_map_sha256'], 'foundation object map')
    gmp = BASE / 'rocq-toolchain/gmp-private/manifest.json'
    pinned(gmp, '332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37')
    audit = dict(scope='MODEL prerequisite only; actual Rust/native correspondences 0',
        prepared_checkout='a13f0551ebd18cc2a7dd2b642c6b2f04547597c6', reviewed_lane='a0d77659',
        reviewed_lane_files={str(p.relative_to(ROOT)): sha(p) for p in sorted(LANE.rglob('*'))
                             if p.is_file() and p.suffix in ['.v', '.py', '.md', '.json']},
        frozen_files=frozen['files'], source_payloads=dict(path=str(source), sha256=source_hashes),
        foundation=dict(ready_sha256=sha(ready_path), receipt_path=ready['receipt'],
            receipt_sha256=sha(ready['receipt']), source_commit=ready['source_commit'],
            sources=523, objects=284, object_map_sha256=map_digest, gmp_manifest_sha256=sha(gmp),
            tools=baseline_binding['tools']),
        worker=worker, replay=replay, development_trials=trials,
        filesystem_mutation_experiments=False, production_publication=False, goal_complete=False)
    write('assumptions.json', dict(declarations=121, closed=119, generated_eliminators=generated,
                                 program_obligations=0, assumptions=assumptions, recursive_kernel_axioms=[FE, EQ]))
    write('audit.json', audit)
    print(json.dumps(dict(authenticated=True, worker_objects_equal_replay=True,
                         declarations=121, closed=119, generated_eliminators=len(generated))))


if __name__ == '__main__':
    main()
