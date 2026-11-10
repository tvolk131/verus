#!/usr/bin/env python3
"""Read-only authentication of exact reviewed inputs; writes only this evidence directory.

Does not import the candidate controller or shared helpers, run artifact controls,
inspect sibling checkouts, or read the parent's replay output.
Run after the ordinary replay has completed: python3 -B evidence/.../audit.py
"""
import hashlib
import json
import re
from pathlib import Path

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[1]
LANE = ROOT / 'source/tools/research/model_state_observation'
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
WORKER = BASE / 'model-observation-trial017'
BUILD = BASE / 'verus-belt-build-115a618d'
HEAP = BASE / 'gmp-pin-heap-positive'


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read(path):
    return json.loads(Path(path).read_bytes())


def require(condition, description):
    if not condition:
        raise RuntimeError(description)


checks = []

def pin(path, expected, label):
    actual = sha(path)
    require(actual == expected, label)
    checks.append(dict(label=label, path=str(path), sha256=actual))


results = read(LANE / 'results.json')
freeze = read(LANE / 'final-freeze.json')
pin(WORKER / 'receipt.json', '7d2620147db8641c926692404be19cfbdaf5431d39bbac99f8d38bee0635aef1', 'user-specified worker receipt')
require(results['receipt']['sha256'] == checks[-1]['sha256'], 'results receipt pin')
pin(LANE / 'final-freeze.json', results['freeze']['sha256'], 'final freeze')
pin(WORKER / 'freeze.json', results['freeze']['sha256'], 'worker retained freeze')
for path, expected in freeze['files'].items():
    pin(ROOT / path, expected, 'frozen current input')
worker = read(WORKER / 'receipt.json')
require(worker['success'] is True and worker['model_proofs_kernel_checked'] is True, 'worker completed successfully')
for name, expected in results['proof_artifacts'].items():
    pin(WORKER / 'proofs' / name, expected, 'worker proof source/object')
    require(worker['proof_artifacts'][str(WORKER / 'proofs' / name)]['sha256'] == expected, 'worker artifact receipt')
for name, expected in results['logs'].items():
    pin(WORKER / name, expected, 'worker log')
for name in ['check.py', 'FROZEN.md']:
    pin(WORKER / name, sha(LANE / name), 'retained controller/document')

trusted = read(ROOT / 'source/tools/research/mutable_buffer_goal/foundation-baseline.json')
binding = read(ROOT / 'source/tools/research/native_semantic_bridge/baseline-bindings.json')
foundation_pin = read(ROOT / 'source/tools/research/mutable_buffer_goal/foundation-pin.json')
for key in ['readiness', 'receipt']:
    pin(Path(trusted[key]['path']), trusted[key]['sha256'], 'authenticated foundation ' + key)
ready = read(trusted['readiness']['path'])
baseline = read(trusted['receipt']['path'])
require(Path(ready['receipt']) == Path(trusted['receipt']['path']), 'ready selects authenticated durable receipt')
require(baseline['success'] is True and ready['baseline_kernel_checked'] is True, 'successful durable baseline and ready')
require(ready['source_commit'] == binding['source_commit'] == foundation_pin['commit'] == '115a618dd35ff8337112cf2081a7f5391d3973d5', 'same pinned source commit')
require(Path(ready['build']) == BUILD, 'same build')
for label in ['prover-version', 'build-baseline', 'kernel-check-baseline']:
    require(any(c['label'] == label and c['returncode'] == 0 for c in baseline['commands']), 'durable stage ' + label)
require(len(baseline['copied_sources']) == 523, '523 sources')
for path, expected in baseline['copied_sources'].items():
    pin(BUILD / path, expected, 'foundation source')
objects = {str(p.relative_to(BUILD)): sha(p) for p in sorted((BUILD / 'src').rglob('*.vo'))}
require(len(objects) == 284, '284 foundation objects')
object_map_hash = hashlib.sha256(json.dumps(objects, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
require(object_map_hash == binding['compiled_module_map_sha256'], 'foundation object inventory hash')
pin(BUILD / '_CoqProject', foundation_pin['files']['_CoqProject'], 'loadpaths')
for name, expected in binding['tools'].items():
    path = Path(ready['switch']) / '_opam/bin' / name if name in ['coqc', 'coqchk'] else Path(ready[name])
    pin(path, expected, 'tool ' + name)
gmp_expected = '332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37'
require(trusted['toolchain']['GMP_packages']['sha256'] == gmp_expected, 'authenticated GMP expected hash')
pin(BASE / 'rocq-toolchain/gmp-private/manifest.json', gmp_expected, 'GMP manifest')
heap_pin = read(ROOT / 'source/tools/research/mutable_buffer_goal/parent-heap-initialization.json')
pin(HEAP / 'receipt.json', heap_pin['environment_pin_fix']['positive_receipt_sha256'], 'corrected heap receipt')
heap = read(HEAP / 'receipt.json')
require(heap['success'] is True and heap['model_proofs_kernel_checked'] is True, 'heap success')
for path, facts in heap['proof_artifacts'].items():
    pin(HEAP / 'proofs' / Path(path).name, facts['sha256'], 'prerequisite source/object')

# Independent syntactic inventory of these manually reviewed, fixed sources.
# This is not a general Coq parser or validation of arbitrary future source files.
inventory = []
for module in ['observation', 'instances', 'controls']:
    path = LANE / 'candidates' / (module + '.v')
    text = path.read_text()
    stripped = re.sub(r'\(\*.*?\*\)', '', text, flags=re.S)
    entries = re.findall(r'^\s*(Definition|Fixpoint|CoFixpoint|Lemma|Theorem|Corollary|Proposition|Fact|Remark|Example|Inductive|CoInductive|Record|Class|Instance)\s+(\w+)', stripped, re.M)
    require([name for _, name in entries] == freeze['constants'][module], 'full fixed-source declaration inventory ' + module)
    require(not re.search(r'\b(Program|Obligation|Admitted|admit|Axiom|Parameter|Abort|native_compute|vm_cast_no_check)\b', stripped), 'no unreviewed declarations or incomplete proofs')
    for kind, name in entries:
        line = next(i for i, s in enumerate(text.splitlines(), 1) if re.match(r'\s*' + kind + r'\s+' + name + r'\b', s))
        inventory.append(dict(name=module + '.' + name, kind=kind, source=str(path.relative_to(ROOT)), line=line))
require(len(inventory) == 20, '20 declarations')
expected_names = [entry['name'] for entry in inventory]
for directory, receipt in [(WORKER, worker), (OUT / 'replay', read(OUT / 'replay/receipt.json'))]:
    require(receipt['success'] is True and receipt['model_proofs_kernel_checked'] is True, 'successful replay ' + str(directory))
    require(receipt['constant_count'] == 20 and list(receipt['assumptions']) == expected_names and all(not x for x in receipt['assumptions'].values()), 'all20 closed ' + str(directory))
    log = (directory / 'assumptions.stdout').read_text()
    blocks = re.findall(r'^BEGIN (\S+)\n(.*?)^END (\S+)\s*$', log, re.M | re.S)
    require([a for a, _, _ in blocks] == expected_names == [b for _, _, b in blocks] and all(body.strip() == 'Closed under the global context' for _, body, _ in blocks), 'exact closed assumption blocks')
    require(receipt['kernel_axioms'] == results['kernel_axioms'], 'inherited kernel axioms only')
    summary = (directory / 'kernel.stderr').read_text()
    require(summary.count('CONTEXT SUMMARY') == 1 and all(needle in summary for needle in ['Theory: Set is predicative', 'Theory: Rewrite rules are not allowed', 'Constants/Inductives relying on type-in-type: <none>', 'Constants/Inductives relying on unsafe (co)fixpoints: <none>', 'Inductives whose positivity is assumed: <none>']), 'safe kernel summary')
    require([c['label'] for c in receipt['commands']] == ['compile-observation', 'compile-instances', 'compile-controls', 'assumptions', 'kernel'] and all(c['exit_code'] == 0 for c in receipt['commands']), 'all five commands successful')
    for cmd in receipt['commands']:
        require(not any(flag in cmd['argv'] for flag in ['-quick', '-vos', '-vok', '-admit', '-norec']), 'normal compile / recursive kernel flags')
    for name, facts in receipt['proof_artifacts'].items():
        require(sha(directory / 'proofs' / Path(name).name) == facts['sha256'], 'retained replay artifact')

replay = read(OUT / 'replay/receipt.json')
comparisons = {}
for name, expected in results['proof_artifacts'].items():
    actual = sha(OUT / 'replay/proofs' / name)
    require(actual == expected, 'fresh byte-identical source/object ' + name)
    comparisons[name] = actual
require(replay['assumptions'] == worker['assumptions'] == results['assumptions'], 'results assumptions agree')
# Development receipts are read only at the explicitly indexed historical paths.
# Never use their failing scripts as mathematical counterexamples.
development = []
for entry in read(LANE / 'development.json'):
    directory = Path(entry['receipt']['path']).parent
    for key in ['receipt', 'freeze']:
        pin(Path(entry[key]['path']), entry[key]['sha256'], 'development ' + key)
    old = read(entry['receipt']['path'])
    require(old['success'] == entry['success'], 'development success classification')
    for name, expected in entry['sources'].items():
        pin(directory / 'proofs' / name, expected, 'development retained source')
    for name, expected in entry['logs'].items():
        pin(directory / name, expected, 'development log')
    development.append(dict(trial=entry['trial'], success=entry['success'], failure=entry['failure'], classification=entry['classification']))

# Compare only pinned identities from the worker receipt; do not open old checkout paths.
worker_inputs = {p: v['sha256'] for p, v in worker['inputs'].items()}
replay_inputs = {p: v['sha256'] for p, v in replay['inputs'].items()}
for rel in freeze['files']:
    old_values = [v for p, v in worker_inputs.items() if p.endswith('/' + rel)]
    require(old_values == [freeze['files'][rel]], 'worker frozen input identity')
    require(replay_inputs[str(ROOT / rel)] == freeze['files'][rel], 'replay frozen input identity')
for path, expected in worker_inputs.items():
    if path == str(BASE / 'model-observation-trial017-freeze.json'):
        require(expected == sha(LANE / 'final-freeze.json'), 'worker original freeze has final-freeze identity')
    elif path.startswith(str(BASE) + '/'):
        require(replay_inputs.get(path) == expected, 'same exact shared dependency ' + path)

audit = dict(scope='Exact MODEL final-state observation prerequisite only', success=True,
    reviewed_head='1d380328', worker_receipt_sha256=sha(WORKER / 'receipt.json'),
    results_sha256=sha(LANE / 'results.json'), freeze_sha256=sha(LANE / 'final-freeze.json'),
    replay_receipt_sha256=sha(OUT / 'replay/receipt.json'), foundation_source_count=523,
    foundation_object_count=284, foundation_object_map_sha256=object_map_hash,
    gmp_manifest_sha256=gmp_expected, comparisons=comparisons, inventory=inventory,
    definition_count=sum(x['kind'] == 'Definition' for x in inventory),
    proved_lemma_theorem_count=sum(x['kind'] in ['Lemma', 'Theorem'] for x in inventory),
    corollary_count=0, new_program_obligations=0, all20_closed=True,
    kernel_axioms=replay['kernel_axioms'], commands=replay['commands'],
    authenticated_artifacts=checks, development=development,
    excluded_failed_trials=[x['trial'] for x in development if not x['success']],
    publication=False, native_effects_discharged=0, whole_goal_acceptance=False)
(OUT / 'audit.json').write_text(json.dumps(audit, indent=2) + '\n')
print(json.dumps({k: audit[k] for k in ['success', 'replay_receipt_sha256', 'foundation_source_count', 'foundation_object_count', 'all20_closed', 'excluded_failed_trials']}))
