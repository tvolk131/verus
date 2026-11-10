#!/usr/bin/env python3
"""Read-only independent authentication; writes only the requested compact JSON.

Does not invoke a prover, alter inputs, run artifact controls, or inspect jobs.
Run the frozen controller separately, then pass its completed private output.
All checks use explicit exceptions (also effective under python -O).
"""
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PKG = ROOT / 'source/tools/research/model_input_observation'
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
ACCEPTED = BASE / 'input-observation-trial005'
RECEIPT_HASH = '3ad9535801ca112a729115ce69eb3c7f51f8307c8852510db0b4ba288b001635'
OBJECTS = {
    'setup.vo': 'a0b1362457f25ecf7dac43ab00cc6ad4db05700ed63d88ae2745b9fbd949a9af',
    'Assumptions.vo': '3d88feb0ef7ee52347ed343c2e84418b6896d419f5b26ffcf26176d64eeff037',
}
CONSTANTS = ['raw_model_context_wp', 'raw_model_context_observed',
             'raw_model_read_outside_observed']
FE = 'Coq.Logic.FunctionalExtensionality.functional_extensionality_dep'
EQ = 'Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq'


def need(b, why):
    if not b:
        raise ValueError(why)


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def load(p):
    return json.loads(Path(p).read_bytes())


def identity(p):
    return {'path': str(p), 'sha256': sha(p)}


def canonical_hash(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True,
                                    separators=(',', ':')).encode()).hexdigest()


def artifacts(receipt, directory):
    result = {}
    for name, facts in receipt['proof_artifacts'].items():
        p = directory / 'proofs' / Path(name).name
        need(sha(p) == facts['sha256'], 'artifact mismatch: ' + str(p))
        result[p.name] = facts['sha256']
    return result


def main():
    fresh, output = map(Path, sys.argv[1:])
    need(sha(ACCEPTED / 'receipt.json') == RECEIPT_HASH, 'trial005 receipt')
    old, new = load(ACCEPTED / 'receipt.json'), load(fresh / 'receipt.json')
    freeze = load(PKG / 'final-freeze.json')
    frozen = (PKG / 'final-freeze.json').read_bytes()
    for p in [PKG / 'trial005-freeze.json', ACCEPTED / 'freeze.json', fresh / 'freeze.json']:
        need(p.read_bytes() == frozen, 'freeze differs: ' + str(p))
    files = {str(p.relative_to(ROOT)): sha(p) for p in PKG.rglob('*') if p.is_file()}
    for name in files:
        target = subprocess.check_output(['git', 'show', 'a49847d7:' + name], cwd=ROOT)
        need((ROOT / name).read_bytes() == target, 'target differs: ' + name)
    need(freeze['constants'] == {'setup': CONSTANTS}, 'constant inventory')
    source = (PKG / 'candidates/setup.v').read_text()
    declared = re.findall(r'^\s*(?:Definition|Fixpoint|Lemma|Theorem|Corollary)\s+(\w+)', source, re.M)
    need(declared == CONSTANTS and not re.search(
        r'\b(?:Program|Obligation|Axiom|Admitted|admit|Parameter|Abort|native_compute|vm_cast_no_check)\b', source),
        'unexpected declaration/unchecked construct')
    for name, expected in freeze['files'].items():
        need(sha(ROOT / name) == expected, 'frozen input: ' + name)
    expected_assumptions = {'setup.' + n: [FE] for n in CONSTANTS}
    runs = []
    for directory, receipt in [(ACCEPTED, old), (fresh, new)]:
        need(receipt['success'] is True and receipt['model_proofs_kernel_checked'] is True,
             'incomplete replay')
        need(receipt['constant_count'] == 3 and receipt['assumptions'] == expected_assumptions,
             'theorem assumptions')
        need(receipt['kernel_axioms'] == [FE, EQ], 'recursive axiom inventory')
        need(receipt['publication'] is False and receipt['source_entry_proved'] is False and
             receipt['native_effects_discharged'] == 0, 'scope flags')
        need([c['label'] for c in receipt['commands']] == ['compile-setup', 'assumptions', 'kernel']
             and all(c['exit_code'] == 0 for c in receipt['commands']), 'prover commands')
        kernel = receipt['commands'][2]['argv']
        need('-admit' not in kernel and '-norec' not in kernel and kernel[-2:] ==
             ['ModelInputObservation.setup', 'ModelInputObservation.Assumptions'], 'recursive targets')
        objects = artifacts(receipt, directory)
        for n, expected in OBJECTS.items():
            need(objects[n] == expected, 'object differs: ' + n)
        need(objects['setup.v'] == sha(PKG / 'candidates/setup.v'), 'copied model source')
        for name, facts in receipt['inputs'].items():
            need(sha(name) == facts['sha256'], 'retained input differs: ' + name)
        runs.append(dict(receipt=identity(directory / 'receipt.json'),
                         artifacts=objects, assumptions=receipt['assumptions'],
                         kernel_axioms=receipt['kernel_axioms'], commands=receipt['commands'],
                         logs=[identity(directory / (label + suffix))
                               for label in ['compile-setup', 'assumptions', 'kernel']
                               for suffix in ['.stdout', '.stderr']]))
    # Independently authenticate all existing foundation sources/objects and tools.
    research = PKG.parent
    trusted = load(research / 'mutable_buffer_goal/foundation-baseline.json')
    binding = load(research / 'native_semantic_bridge/baseline-bindings.json')
    pin = load(research / 'mutable_buffer_goal/foundation-pin.json')
    ready_path = BASE / 'rocq-toolchain/baseline-ready.json'
    ready = load(ready_path)
    build = Path(ready['build'])
    baseline = load(ready['receipt'])
    need(sha(ready_path) == trusted['readiness']['sha256'] and
         sha(ready['receipt']) == trusted['receipt']['sha256'], 'ready/build pins')
    need(ready['baseline_kernel_checked'] is True and baseline['success'] is True and
         ready['source_commit'] == pin['commit'] == binding['source_commit'], 'foundation status')
    need(len(baseline['copied_sources']) == 523, '523 sources')
    for n, expected in baseline['copied_sources'].items():
        need(sha(build / n) == expected, 'foundation source: ' + n)
    objects = {str(p.relative_to(build)): sha(p) for p in (build / 'src').rglob('*.vo')}
    need(len(objects) == 284 and canonical_hash(objects) == binding['compiled_module_map_sha256'],
         '284 object map')
    need(sha(build / '_CoqProject') == pin['files']['_CoqProject'], 'loadpaths')
    tools = {}
    for name in ['coqc', 'coqchk', 'runner', 'opam']:
        p = Path(ready['switch']) / '_opam/bin' / name if name in ('coqc', 'coqchk') else Path(ready[name])
        need(sha(p) == binding['tools'][name], 'tool: ' + name)
        tools[name] = identity(p)
    gmp = BASE / 'rocq-toolchain/gmp-private/manifest.json'
    need(sha(gmp) == trusted['toolchain']['GMP_packages']['sha256'] ==
         '332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37', 'GMP')
    libraries = []
    library_specs = [
        (BASE / 'parent-heap-init-3d799b38', load(research / 'mutable_buffer_goal/parent-heap-initialization.json')['parent_replay']),
        (BASE / 'semantic-guard-validation-1efbd004/proofs', load(research / 'mutable_buffer_goal/parent-semantic-foundation.json')['review_fix_positive_receipt']),
        (BASE / 'parent-model-input-94b87742/proofs', load(research / 'mutable_buffer_goal/parent-model-input-context.json')['proof_receipt']),
        (BASE / 'model-observation-trial017', load(research / 'model_state_observation/results.json')['receipt']),
    ]
    for d, committed in library_specs:
        r = load(d / 'receipt.json')
        need(sha(d / 'receipt.json') == committed['sha256'] and r['success'] is True and
             r['model_proofs_kernel_checked'] is True, 'prerequisite receipt')
        libraries.append(dict(receipt=identity(d / 'receipt.json'), artifacts=artifacts(r, d)))
    history = []
    for row in load(PKG / 'development.json'):
        p = Path(row['receipt'])
        need(sha(p) == row['receipt_sha256'], 'historical receipt')
        r = load(p)
        historical = dict(receipt=identity(p), classification=row['classification'])
        if 'trial' in row:
            src = Path(row['source_copy'])
            fr = load(PKG / row['freeze'])
            need(sha(src) == fr['files']['source/tools/research/model_input_observation/candidates/setup.v'],
                 'historical source freeze')
            historical.update(source=identity(src), success=r['success'], constant_count=r.get('constant_count'))
            if not r['success']:
                historical['diagnostic'] = (p.parent / 'compile-setup.stderr').read_text()
        else:
            src = p.parent / 'read.v'
            auxiliary_freeze = p.parent / 'freeze.json'
            need(sha(src) == load(auxiliary_freeze)['source_sha256'], 'auxiliary source freeze')
            if 'source_sha256' in r:
                need(sha(src) == r['source_sha256'], 'historical auxiliary source')
            need(r['exit_code'] == 1 and r['kernel_checked'] is False, 'auxiliary classification')
            historical.update(source=identity(src), freeze=identity(auxiliary_freeze),
                              diagnostic=(p.parent / 'stderr').read_text(),
                              kernel_checked=False)
        history.append(historical)
    inspection = load(fresh / 'inspection.json')
    need(inspection['exit_code'] == 0, 'type inspection')
    for file in inspection['files']:
        need(sha(file['path']) == file['sha256'], 'type-inspection retained identity')
    result = dict(schema=1, reviewed_checkout='a13f0551ebd18cc2a7dd2b642c6b2f04547597c6',
                  target='a49847d7', verdict='accept exact MODEL prerequisite', findings=[],
                  package_files=files, freeze=identity(PKG / 'final-freeze.json'),
                  frozen_inputs=freeze['files'], program_obligations=0,
                  accepted=runs[0], fresh=runs[1], type_inspection=inspection,
                  foundation=dict(ready=identity(ready_path), receipt=identity(Path(ready['receipt'])),
                                  source_count=523, source_map_sha256=canonical_hash(baseline['copied_sources']),
                                  object_count=284, object_map_sha256=canonical_hash(objects),
                                  loadpaths=identity(build / '_CoqProject'), tools=tools, GMP=identity(gmp)),
                  libraries=libraries, historical_classification=history,
                  actual_native_correspondences=0, minimal_native_open=44, phase_open=3,
                  native_effects_open=101, canonical=[6, 0], frozen_HMAC_prefix_AND_suffix=[1, 1],
                  publication=False, goal_complete=False)
    output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(dict(verdict=result['verdict'], output=str(output), sha256=sha(output))))


if __name__ == '__main__':
    main()
