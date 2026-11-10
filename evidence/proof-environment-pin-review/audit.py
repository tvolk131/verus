"""Read-only authentication of named completed evidence; never runs a controller."""
import ast
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
RESEARCH = ROOT / 'source/tools/research'
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
BRIDGE = RESEARCH / 'native_semantic_bridge'
HEAP = RESEARCH / 'native_heap_initialization'
ENTRY = RESEARCH / 'native_source_entry'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load(path):
    return json.loads(Path(path).read_bytes())


def authenticate(path, expected):
    require(sha(path) == expected, 'identity mismatch: ' + str(path))
    return load(path)


trusted_path = RESEARCH / 'mutable_buffer_goal/foundation-baseline.json'
trusted = load(trusted_path)
heap_parent = load(RESEARCH / 'mutable_buffer_goal/parent-heap-initialization.json')
bridge_parent = load(RESEARCH / 'mutable_buffer_goal/parent-semantic-foundation.json')
bindings = load(BRIDGE / 'baseline-bindings.json')
manifest = Path(trusted['toolchain']['GMP_packages']['path'])
pin = trusted['toolchain']['GMP_packages']['sha256']
authenticate(manifest, pin)
ready_path = Path(trusted['readiness']['path'])
ready = authenticate(ready_path, trusted['readiness']['sha256'])
foundation = authenticate(ready['receipt'], trusted['receipt']['sha256'])
require(foundation['success'] is True and len(foundation['copied_sources']) == 523,
        'foundation receipt failed/incomplete')
wrapper = Path(ready['runner'])
require(sha(wrapper) == bindings['tools']['runner'], 'wrapper identity')
require("b=Path('" + str(ready_path.parent) + "')" in wrapper.read_text()
        and "(b/'gmp-private/manifest.json').read_text()" in wrapper.read_text(),
        'wrapper manifest source')
entry_freeze = load(ENTRY / 'entry-gmp-pin-freeze.json')
paths = [ready_path.parent / 'gmp-private/manifest.json',
         Path(entry_freeze['ready']).parent / 'gmp-private/manifest.json',
         BASE / 'rocq-toolchain/gmp-private/manifest.json']
require(all(p.resolve() == manifest.resolve() for p in paths), 'caller manifest paths')

checks = dict(reviewed_head=subprocess.check_output(
    ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    manifest=dict(path=str(manifest), sha256=pin),
    foundation_metadata=dict(path=str(trusted_path.relative_to(ROOT)), sha256=sha(trusted_path)),
    ready_sha256=sha(ready_path), runner_sha256=sha(wrapper),
    foundation_receipt_sha256=sha(ready['receipt']), freezes={}, receipts={}, syntax=[])

freeze_paths = [BRIDGE / 'gmp-pin-freeze.json', ENTRY / 'entry-gmp-pin-freeze.json',
                HEAP / 'gmp-pin-freeze.json', HEAP / 'controls-gmp-pin-v2-freeze.json']
mandatory = {'source/tools/infer_eval_support.py', 'source/tools/infer_experiment.py',
             str((BRIDGE / 'acceptance_inputs.py').relative_to(ROOT)),
             str((BRIDGE / 'proof_checks.py').relative_to(ROOT)),
             str(trusted_path.relative_to(ROOT))}
for path in freeze_paths:
    f = load(path)
    require(f['before_first_trial'] is True and mandatory <= set(f['files']), 'mandatory inputs')
    for name, expected in f['files'].items():
        require(sha(ROOT / name) == expected, 'active freeze mismatch: ' + name)
    for name, expected in f.get('private_inputs', {}).items():
        require(sha(name) == expected, 'private freeze mismatch: ' + name)
    checks['freezes'][str(path.relative_to(ROOT))] = dict(
        sha256=sha(path), public_count=len(f['files']),
        private_count=len(f.get('private_inputs', {})), files=f['files'])
proof_freeze = load(HEAP / 'gmp-pin-freeze.json')
control_freeze = load(HEAP / 'controls-gmp-pin-v2-freeze.json')
require(set(control_freeze['files']) == set(proof_freeze['files']) | {
    str((HEAP / 'gmp-pin-freeze.json').relative_to(ROOT)),
    str((HEAP / 'guard_controls.py').relative_to(ROOT))}, 'control input coverage')

changed = ['native_semantic_bridge/acceptance_inputs.py', 'native_semantic_bridge/check.py',
           'native_source_entry/entry_audit.py', 'native_heap_initialization/check.py',
           'native_heap_initialization/guard_controls.py',
           'native_semantic_bridge/test_acceptance_inputs.py']
for name in changed + ['native_semantic_bridge/test_proof_checks.py',
                       'native_semantic_bridge/proof_checks.py',
                       'native_semantic_bridge/source_map.py',
                       '../infer_eval_support.py', '../infer_experiment.py']:
    path = RESEARCH / name
    ast.parse(path.read_text(), filename=str(path))
    checks['syntax'].append(dict(path=str(path.resolve().relative_to(ROOT)), sha256=sha(path)))
git_checks = [
    ['git', 'diff', '--exit-code', 'fa63c130', 'HEAD', '--',
     *[str((RESEARCH / n).relative_to(ROOT)) for n in changed]],
    ['git', 'diff', '--exit-code', 'fa63c130^', 'fa63c130', '--', '*.v'],
    ['git', 'diff', '--exit-code', '6deca82c', 'HEAD', '--',
     *[str((RESEARCH / n / 'candidates').relative_to(ROOT)) for n in
       ('native_semantic_bridge', 'native_source_entry', 'native_heap_initialization')]]]
for command in git_checks:
    subprocess.run(command, cwd=ROOT, check=True, capture_output=True)
checks['git_checks'] = git_checks


def receipt(path, expected, snapshot=True):
    path = Path(path)
    r = authenticate(path, expected)
    require(r['success'] is True and r['publication'] is False, 'receipt status')
    info = dict(sha256=sha(path), commands=[], retained_products={})
    for command in r['commands']:
        require(command.get('exit_code', command.get('returncode')) == 0, 'failed command')
        if 'log' in command:
            require(Path(command['log']).is_relative_to(path.parent), 'command log path')
            require(sha(command['log']) == command['log_sha256'], 'recorded command log')
            logs = {Path(command['log']).name: command['log_sha256']}
        else:
            logs = {suffix: sha(path.parent / (command['label'] + '.' + suffix))
                    for suffix in ('stdout', 'stderr')}
        info['commands'].append(dict(label=command['label'], logs=logs))
    if snapshot:
        artifacts = r.get('inputs', r.get('artifacts'))
        for p, value in [(manifest, pin), (trusted_path, sha(trusted_path)),
                         (ready_path, sha(ready_path)), (wrapper, sha(wrapper))]:
            if p.is_relative_to(ROOT):
                matches = [v for n, v in artifacts.items()
                           if n.endswith('/' + str(p.relative_to(ROOT)))]
            else:
                matches = [artifacts.get(str(p))]
            require(len(matches) == 1 and matches[0]['sha256'] == value, 'retained pin identity')
        info['manifest_sha256'] = artifacts[str(manifest)]['sha256']
    for name, info_record in r.get('proof_artifacts', {}).items():
        # Read only completed private products; never source paths from another checkout.
        product = Path(name)
        require(product.is_relative_to(path.parent), 'product outside completed output')
        require(sha(product) == info_record['sha256'], 'retained proof product identity')
        info['retained_products'][product.name] = info_record['sha256']
    checks['receipts'][str(path)] = info
    return r


fix = heap_parent['environment_pin_fix']
heap = receipt(fix['positive_receipt'], fix['positive_receipt_sha256'])
require(heap['constant_count'] == len(heap['assumptions']) == 29
        and all(v == [] for v in heap['assumptions'].values()), '29 closed constants')
require({Path(p).name: v['sha256'] for p, v in heap['proof_artifacts'].items()
         if p.endswith('.vo')} == heap_parent['proof_objects'], 'original heap objects')
require(heap['source_entry_proved'] is False and heap['native_effects_discharged'] == 0,
        'heap native status')
require(heap['model_proofs_kernel_checked'] is True, 'heap kernel status')
for name, info in heap['proof_artifacts'].items():
    if name.endswith('.v') and Path(name).name != 'Assumptions.v':
        require(sha(HEAP / 'candidates' / Path(name).name) == info['sha256'], 'heap proof source')

batch_pin = bridge_parent['model_source_status_fix']['positive_matching_replay']
batch = receipt(batch_pin['path'], batch_pin['sha256'], snapshot=False)
require(batch['artifacts'] == batch['artifacts_after'], 'historical batch stability')
require(all(batch[k] == v for k, v in dict(unit_tests=19, optimized_unit_tests=19,
        source_controls=12, readiness_controls=4, acceptance_controls=3, source_races=10,
        named_theorems=18, byte_type_obligations=6, assumption_constants=25,
        source_entry_constants=42).items()), 'batch inventory')
require(batch['native_bridge_proved'] is False and batch['goal_complete'] is False, 'batch limits')
checks['receipts'][batch_pin['path']]['counts'] = {k: batch[k] for k in (
    'unit_tests', 'optimized_unit_tests', 'source_controls', 'readiness_controls',
    'acceptance_controls', 'source_races', 'named_theorems', 'byte_type_obligations',
    'assumption_constants', 'source_entry_constants')}
for record in bridge_parent['environment_pin_fix']['completed_model_and_inspection_checks']:
    r = receipt(record['path'], record['sha256'])
    require(r['native_effects_discharged'] == 0, 'native effects promoted')
    if 'proof_terms' in r:
        require(r['model_proofs_kernel_checked'] is True and r['native_bridge_proved'] is False
                and len(r['assumption_inventory']) == 25, 'model receipt status')
        require(r['source_connection_proved'] is False and
                r['original_array_future_import_proved'] is False, 'model import status')
        for module, value in r['proof_terms'].items():
            original = bridge_parent['parent_object_and_source_hash_audit'][module]
            require(value == original['sha256'], 'original model object')
            require(sha(BRIDGE / 'candidates/loan-v12' / (module + '.v')) ==
                    original['source_sha256'], 'original model source')
    else:
        require(r['native_input_proved'] is False and r['inspected_constants'] == 42,
                'inspection status')
        for name, expected in r['outputs'].items():
            require(Path(name).is_relative_to(Path(record['path']).parent), 'inspection output path')
            require(sha(name) == expected, 'inspection retained output identity')
        checks['receipts'][record['path']]['inspection_outputs_authenticated'] = len(r['outputs'])

old_model = Path(entry_freeze['model_receipt'])
receipt(old_model, entry_freeze['private_inputs'][str(old_model)])
failed_path = Path(fix['relative_path_setup_failure_retained'])
failed = load(failed_path)
require(failed['success'] is False and failed['commands'] == [] and
        failed['semantic_negatives'] == 0 and 'not in the subpath' in failed['failure']['message'],
        'retained setup failure classification')
checks['setup_failure'] = dict(path=str(failed_path), sha256=sha(failed_path),
                              failure=failed['failure'], commands=[], semantic_negatives=0)
checks['test_logs'] = {name: sha(Path(__file__).parent / name)
                      for name in ('pure-normal.log', 'pure-optimized.log')}
print(json.dumps(checks, indent=2))
