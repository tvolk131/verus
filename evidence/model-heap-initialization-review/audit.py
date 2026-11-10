#!/usr/bin/env python3
"""Read-only authentication of completed review/worker/parent artifacts.

Writes only this directory's compact receipt. Does not run refusal mutations,
read other checkouts, build the foundation, or repeat the proof replay.
"""
import hashlib
import json
import re
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
LANE = ROOT / 'source/tools/research/native_heap_initialization'
FRESH = Path('/tmp/model-heap-initialization-review-9f91fd58-normal')
INSPECT = Path('/tmp/model-heap-initialization-review-9f91fd58-inspection')


def need(test, message):
    if not test:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load(path, expected=None):
    if expected is not None:
        need(sha(path) == expected, 'digest mismatch: ' + str(path))
    return json.loads(Path(path).read_bytes())


def bound(path):
    return dict(path=str(path), sha256=sha(path))


freeze = load(LANE / 'final-freeze.json')
results = load(LANE / 'results.json')
foundation = load(ROOT / 'source/tools/research/mutable_buffer_goal/foundation-baseline.json')
gmp = foundation['toolchain']['GMP_packages']
need(sha(gmp['path']) == gmp['sha256'], 'reviewed GMP environment manifest')
for rel, expected in {**freeze['files'], **results['files']}.items():
    need(sha(ROOT / rel) == expected, 'current frozen input: ' + rel)
expected_names = [m + '.' + n for m, ns in freeze['constants'].items() for n in ns]
declarations = []
for module, ns in freeze['constants'].items():
    src = LANE / 'candidates' / (module + '.v')
    found = re.findall(r'^\s*(Definition|Fixpoint|Lemma|Theorem)\s+(\w+)', src.read_text(), re.M)
    need([n for _, n in found] == ns, 'source declarations: ' + module)
    declarations += [(module + '.' + n, kind) for kind, n in found]
need(len(declarations) == 29, 'declaration count')
need(sum(k in ('Definition', 'Fixpoint') for _, k in declarations) == 4, 'definition count')
printed = (INSPECT / 'inspect.stdout').read_text()
object_names = re.findall(r'^\s+(?:Definition|Parameter) (\w+) :', printed, re.M)
need(object_names == [n.split('.')[1] for n in expected_names], 'compiled module inventory')

fresh = load(FRESH / 'receipt.json')
fresh_logs = {p.name: sha(p) for p in FRESH.iterdir() if p.suffix in ('.stdout', '.stderr')}
need(fresh['inputs'][gmp['path']]['sha256'] == gmp['sha256'], 'fresh recorded GMP manifest')
need(fresh['success'] and fresh['model_proofs_kernel_checked'], 'fresh replay failed')
need(list(fresh['assumptions']) == expected_names, 'fresh assumption inventory')
need(all(a == [] for a in fresh['assumptions'].values()), 'new assumption')
need(fresh['constant_count'] == 29, 'receipt count')
need([c['label'] for c in fresh['commands']] == [
    'compile-initialization', 'compile-partition', 'compile-controls', 'assumptions', 'kernel'],
    'unexpected replay commands')
need(all(c['exit_code'] == 0 for c in fresh['commands']), 'command failure')
need(fresh['commands'][-1]['argv'][2:4] == ['-silent', '-o'], 'recursive check options')
fresh_objects = {Path(p).name: v['sha256'] for p, v in fresh['proof_artifacts'].items()
                 if p.endswith('.vo')}

historical = []
parent = load(ROOT / 'source/tools/research/mutable_buffer_goal/parent-heap-initialization.json')
for label, item in [
    ('worker-first', results['first_successful_trial']),
    ('worker-final', results['final_replay']),
    ('parent-normal', dict(receipt=parent['parent_replay']['path'],
                           receipt_sha256=parent['parent_replay']['sha256']))]:
    path = Path(item['receipt'])
    receipt = load(path, item['receipt_sha256'])
    need(receipt['inputs'][gmp['path']]['sha256'] == gmp['sha256'], label + ' recorded GMP manifest')
    need(receipt['success'] and receipt['model_proofs_kernel_checked'], label + ' failed')
    need(receipt['assumptions'] == fresh['assumptions'], label + ' assumptions')
    need(receipt['kernel_axioms'] == fresh['kernel_axioms'], label + ' dependencies')
    objects = {Path(p).name: v['sha256'] for p, v in receipt['proof_artifacts'].items()
               if p.endswith('.vo')}
    need(objects == fresh_objects, label + ' object mismatch')
    for p, v in receipt['proof_artifacts'].items():
        need(sha(p) == v['sha256'], label + ' retained product: ' + p)
    trial_freeze_path = LANE / ('trial-016.json' if label == 'worker-first' else 'final-freeze.json')
    trial_freeze = load(trial_freeze_path)
    need(sha(path.parent / 'freeze.json') == sha(trial_freeze_path), label + ' retained freeze')
    changes = {rel for rel in freeze['files'] if freeze['files'][rel] != trial_freeze['files'][rel]}
    need(changes == ({'source/tools/research/native_heap_initialization/check.py'}
                     if label == 'worker-first' else set()), label + ' historical input differences')
    for rel, expected in trial_freeze['files'].items():
        matches = [v['sha256'] for p, v in receipt['inputs'].items() if p.endswith('/' + rel)]
        need(matches == [expected], label + ' recorded input mismatch: ' + rel)
    for p, expected in item.get('logs', {}).items():
        need(sha(p) == expected, label + ' log mismatch: ' + p)
    need({n: sha(path.parent / n) for n in fresh_logs} == fresh_logs,
         label + ' logs differ from fresh replay')
    historical.append(dict(label=label, **bound(path), objects_match=True,
                           freeze=bound(trial_freeze_path),
                           controller_sha256=trial_freeze['files']['source/tools/research/native_heap_initialization/check.py']))

controls = results['provenance_controls']
control_receipt = load(controls['receipt'], controls['receipt_sha256'])
need(control_receipt['success'] and control_receipt['semantic_negatives'] == 0, 'control status')
need(control_receipt['results'] == controls['refusals'], 'control manifest')
need(len(control_receipt['commands']) == len(controls['refusals']) == 6, 'control coverage')
for command, item in zip(control_receipt['commands'], controls['refusals']):
    need(command['label'] == item['case'] and command['exit_code'] != 0, 'control command')
    reject_path = Path(command['argv'][-1]) / 'receipt.json'
    rejection = load(reject_path, item['receipt_sha256'])
    need(rejection['success'] is False and rejection['commands'] == [], 'not pre-prover refusal')
    need(rejection['failure'] == item['failure'], 'refusal reason')

failed_path = LANE / 'failed-trials.json'
failed = load(failed_path)
need(failed['semantic_negatives'] == 0 and len(failed['trials']) == 15, 'trial classification')
for item in failed['trials']:
    load(ROOT / item['freeze'], item['freeze_sha256'])
    trial = load(item['receipt'], item['receipt_sha256'])
    need(trial['success'] is False and trial['failure'] == item['failure'], 'failed trial receipt')
    for p, expected in {**item['copied_sources'], **item['logs']}.items():
        need(sha(p) == expected, 'failed trial artifact: ' + p)
historical_controller = failed['historical_controller_v2']
need(sha(historical_controller['path']) == historical_controller['sha256'], 'historical controller')

inspection = load(INSPECT / 'receipt.json')
need(inspection['exit_code'] == 0, 'compiled signature inspection failed')
for name, expected in inspection['files'].items():
    need(sha(INSPECT / name) == expected, 'inspection log')
for local, original in [('DECLARATIONS.txt', INSPECT / 'inspect.stdout'),
                        ('ASSUMPTIONS.txt', FRESH / 'assumptions.stdout'),
                        ('KERNEL.txt', FRESH / 'kernel.stderr')]:
    need(sha(HERE / local) == sha(original), 'committed log copy: ' + local)
for p, record in fresh['proof_artifacts'].items():
    need(sha(p) == record['sha256'], 'fresh product changed: ' + p)
interface = subprocess.check_output(['git', 'show', 'da164976:source/tools/research/native_heap_initialization/INTERFACE.md'], cwd=ROOT)
need(hashlib.sha256(interface).hexdigest() == sha(LANE / 'INTERFACE.md'), 'original interface')
subprocess.run(['git', 'diff', '--exit-code', '3d799b38', '--',
                'source/tools/research/native_heap_initialization',
                'source/tools/research/native_semantic_bridge/proof_checks.py'], cwd=ROOT, check=True)

receipt = dict(
    schema=1, review_base='9f91fd58', implementation='3d799b38',
    authentication_command=['python3', '-B', 'evidence/model-heap-initialization-review/audit.py'],
    authentication_code_sha256=sha(Path(__file__)),
    verdict='accept_exact_model_proofs_with_P2_controller_pin_finding',
    substantive_findings=[dict(id='C1', priority='P2',
        source='source/tools/research/native_heap_initialization/check.py:65',
        issue='GMP environment manifest is stability-guarded but not compared with its existing trusted digest',
        method='source inspection only; no substitution or concurrency experiment',
        current_and_all_four_completed_replays_match_pinned_manifest=True)],
    independently_authenticated_gmp_manifest=bound(gmp['path']),
    new_source_assumptions=False, unsafe_kernel_modes=False,
    fresh_replay=dict(**bound(FRESH / 'receipt.json'),
        invocation=['python3', '-B', 'source/tools/research/native_heap_initialization/check.py',
                    '--freeze', 'source/tools/research/native_heap_initialization/final-freeze.json',
                    '--output', str(FRESH)],
        cwd=str(ROOT), success=True,
        stages=[{k: c[k] for k in ('label', 'exit_code', 'seconds')} for c in fresh['commands']],
        logs=fresh_logs,
        full_argv_in_bound_receipt=True, proof_objects=fresh_objects,
        all_29_individually_closed=True, definitions=4, theorems=25, program_obligations=0,
        kernel_axioms=fresh['kernel_axioms']),
    compiled_signature_inspection=bound(INSPECT / 'receipt.json'),
    inventory=[dict(name=n, kind=k) for n, k in declarations],
    historical_replays_authenticated=historical,
    provenance_controls=dict(**bound(controls['receipt']), cases_authenticated=6,
                             pre_prover_refusals=True, rerun=False, semantic_negatives=0),
    development_trials=dict(**bound(failed_path), failures_authenticated=15,
                            retained_freezes_sources_logs_authenticated=True, rerun=False,
                            semantic_negatives=0),
    source_bindings={str(p.relative_to(ROOT)): sha(p) for p in [
        LANE / 'final-freeze.json', LANE / 'controls-freeze.json', LANE / 'results.json',
        ROOT / 'source/tools/research/mutable_buffer_goal/TARGET.md',
        ROOT / 'source/tools/research/mutable_buffer_goal/parent-heap-initialization.json',
        ROOT / 'source/tools/research/mutable_buffer_goal/FACT_PROVENANCE.md']},
    log_copies={p: sha(HERE / p) for p in ['DECLARATIONS.txt', 'ASSUMPTIONS.txt', 'KERNEL.txt']},
    native_source_entry_proved=False, actual_native_correspondences=0,
    minimal_native_simulations_open=44, native_effects_open=101,
    full_HMAC='inherited 1/1; not rerun', native_drop='inherited 0/1; not rerun',
    full_goal_complete=False, publication=False)
(HERE / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps(dict(success=True, verdict=receipt['verdict'], receipt=bound(HERE / 'receipt.json'))))
