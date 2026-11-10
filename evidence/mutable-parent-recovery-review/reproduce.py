#!/usr/bin/env python3
"""Read-only identity audit plus optional normal five-constant proof replay.
Run with python3 -B. Never follow historical checkout paths in receipts.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
HERE = ROOT / 'source/tools/research/model_mutable_writer'
EVIDENCE = ROOT / 'evidence/mutable-parent-recovery-review'
BASE = '9b759adb4516e88ecc51ecf407d32b95d1a25dda'
TMP = Path('/tmp/verus-infer-buffer-cycle-20261109')
NEST = Path('/tmp/model-nested-borrow-context-trial-053')
BUILD = TMP / 'verus-belt-build-115a618d'
OFFICIAL = '115a618dd35ff8337112cf2081a7f5391d3973d5'
GMP = '332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37'
sys.path.insert(0, str(ROOT/'source/tools/research/native_semantic_bridge'))
from proof_checks import parse_axioms, parse_kernel_summary

EXPECTED = {
    'resource_protocol': ['shorter_run_resources'],
    'child': ['bytes_vector_mapsto', 'byte_child_transform'],
    'initialized': ['initialized_nested_run', 'initialized_nested_payload_run'],
}
FE = 'Coq.Logic.FunctionalExtensionality.functional_extensionality_dep'
EQ = 'Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq'
ASSUMPTIONS = {m+'.'+n: [FE] if m == 'child' else [FE, EQ]
               for m, ns in EXPECTED.items() for n in ns}


def need(b, why):
    if not b:
        raise ValueError(why)


def sha(p):
    h = hashlib.sha256()
    with Path(p).open('rb') as f:
        for b in iter(lambda: f.read(1048576), b''):
            h.update(b)
    return h.hexdigest()


def read(p):
    return json.loads(Path(p).read_text())


def pin(p, expected):
    need(sha(p) == expected, 'identity mismatch: '+str(p))


def repo_input(name):
    # This is string rebasing, never resolve/read an archived checkout path.
    if name.startswith('source/'):
        return ROOT / name
    need('/source/' in name, 'unrecognized repository-relative input')
    return ROOT / ('source/' + name.split('/source/', 1)[1])


def artifacts(receipt, directory):
    result = {}
    for name, facts in receipt['proof_artifacts'].items():
        p = directory / 'proofs' / Path(name).name
        need(p.suffix in ('.v', '.vo'), 'unexpected proof artifact')
        pin(p, facts['sha256'])
        result[p.name] = facts['sha256']
    return result


def commands(receipt, modules):
    expected = ['compile-'+m for m in modules] + ['assumptions', 'kernel']
    need([c['label'] for c in receipt['commands']] == expected, 'command inventory')
    for c in receipt['commands']:
        need(c['exit_code'] == 0, 'unsuccessful proof command')
        argv = c['argv']
        need(argv[0] == str(TMP/'rocq-toolchain/run-rocq'), 'runner')
        need(not any(x in ('-admit', '-norec', '-unsafe', '-vos', '-vok', '-quick', '-vio',
                          '-allow-sprop', '-type-in-type', '-indices-matter') for x in argv),
             'unsupported flag')
        if c['label'] == 'kernel':
            need(Path(argv[1]).name == 'coqchk' and argv[2:4] == ['-silent', '-o'], 'recursive kernel flags')
            start = 4
            while start < len(argv) and argv[start] in ('-Q', '-R'):
                start += 3
            need(argv[start:] == ['ModelMutableWriter.'+m for m in modules] +
                 ['ModelMutableWriter.Assumptions'], 'kernel module inventory or excluded dependency')
        else:
            need(Path(argv[1]).name == 'coqc' and argv[2] == '-q', 'normal compilation')
    need(receipt['success'] is True and receipt['model_proofs_kernel_checked'] is True,
         'incomplete replay')
    need(receipt['publication'] is False and receipt['native_effects_discharged'] == 0,
         'scope flags')


def logs(receipt, directory):
    log = (directory/'assumptions.stdout').read_text()
    sections = re.findall(r'^BEGIN (\S+)\n(.*?)^END (\S+)\s*$', log, re.M|re.S)
    expected = receipt['assumptions']
    need([n for n,_,_ in sections] == list(expected) == [n for _,_,n in sections], 'assumption markers')
    need(len(re.findall(r'^BEGIN ', log, re.M)) == len(re.findall(r'^END ', log, re.M)) == len(expected), 'marker count')
    need({n:parse_axioms(body) for n,body,_ in sections} == expected, 'actual assumptions log')
    kernel = parse_kernel_summary((directory/'kernel.stderr').read_text())
    need(set(kernel) == {FE, EQ} == set(receipt['kernel_axioms']), 'actual kernel context')
    return {p.name:sha(p) for p in sorted(directory.iterdir()) if p.suffix in ('.stdout', '.stderr')}


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--replay', type=Path, help='NEW private output; runs committed normal checker')
    p.add_argument('--receipt', type=Path, help='completed private review receipt')
    a = p.parse_args()
    need((a.replay is None) != (a.receipt is None), 'choose --replay or --receipt')
    if a.replay is not None:
        need(a.replay.is_absolute() and str(a.replay).startswith('/tmp/'), 'private output required')
        subprocess.run([sys.executable, '-B', str(HERE/'check.py'), '--freeze',
                        str(HERE/'trial008-freeze.json'), '--output', str(a.replay)],
                       check=True, stdout=sys.stderr, cwd=ROOT)
        receipt_path = a.replay/'receipt.json'
    else:
        receipt_path = a.receipt
    need(receipt_path.is_absolute() and str(receipt_path).startswith('/tmp/'), 'private receipt required')
    freeze = read(HERE/'trial008-freeze.json')
    need(freeze['before_first_trial'] is True and freeze['constants'] == EXPECTED, 'exact freeze')
    for name, expected in freeze['files'].items():
        pin(repo_input(name), expected)
    need({p.stem for p in (HERE/'candidates').glob('*.v')} == set(EXPECTED), 'source modules')
    declarations = {}
    for m, names in EXPECTED.items():
        src = (HERE/'candidates'/(m+'.v')).read_text()
        declarations[m] = re.findall(r'^\s*(?:Definition|Fixpoint|Lemma|Theorem|Corollary)\s+(\w+)', src, re.M)
        need(declarations[m] == names, 'declarations')
        need(not re.search(r'\b(Axiom|Parameter|Admitted|admit|Abort|Program|Obligation|Conjecture|Variable|Hypothesis|Unset|native_compute|vm_cast_no_check)\b', src), 'unexpected helper/unsafe construct')
    # Explicitly read/authenticate both foundation gates; labels alone do not suffice.
    trusted = read(ROOT/'source/tools/research/mutable_buffer_goal/foundation-baseline.json')
    binding = read(ROOT/'source/tools/research/native_semantic_bridge/baseline-bindings.json')
    foundation_pin = read(ROOT/'source/tools/research/mutable_buffer_goal/foundation-pin.json')
    ready_path = TMP/'rocq-toolchain/baseline-ready.json'
    pin(ready_path, trusted['readiness']['sha256'])
    ready = read(ready_path)
    baseline_path = Path(ready['receipt'])
    need(baseline_path == TMP/'rocq-toolchain/durable-build-2/receipt.json', 'baseline path')
    pin(baseline_path, trusted['receipt']['sha256'])
    baseline = read(baseline_path)
    need(ready['baseline_kernel_checked'] is True and baseline['success'] is True, 'both foundation gates')
    need(ready['source_commit'] == binding['source_commit'] == foundation_pin['commit'] == OFFICIAL, 'official commit')
    need(Path(ready['build']) == BUILD, 'build path')
    need({'prover-version', 'build-baseline', 'kernel-check-baseline'} <=
         {c['label'] for c in baseline['commands'] if c['returncode'] == 0}, 'foundation commands')
    need(len(baseline['copied_sources']) == trusted['copied_source_files'] == 523, '523 sources')
    for name, expected in baseline['copied_sources'].items():
        pin(BUILD/name, expected)
    pin(BUILD/'_CoqProject', foundation_pin['files']['_CoqProject'])
    obj = {str(p.relative_to(BUILD)): sha(p) for p in (BUILD/'src').rglob('*.vo')}
    need(len(obj) == binding['compiled_module_count'] == 284, '284 objects')
    need(hashlib.sha256(json.dumps(obj, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
         == binding['compiled_module_map_sha256'], 'foundation object map')
    pin(TMP/'rocq-toolchain/gmp-private/manifest.json', GMP)
    need(trusted['toolchain']['GMP_packages']['sha256'] == GMP, 'GMP pin')
    for name in ('coqc', 'coqchk', 'runner', 'opam'):
        tool = Path(ready['switch'])/'_opam/bin'/name if name in ('coqc', 'coqchk') else Path(ready[name])
        pin(tool, binding['tools'][name])
    prerequisites = [
        (TMP/'parent-heap-init-3d799b38', 'mutable_buffer_goal/parent-heap-initialization.json', 'parent_replay'),
        (TMP/'semantic-guard-validation-1efbd004/proofs', 'mutable_buffer_goal/parent-semantic-foundation.json', 'review_fix_positive_receipt'),
        (TMP/'parent-model-input-94b87742/proofs', 'mutable_buffer_goal/parent-model-input-context.json', 'proof_receipt'),
        (TMP/'model-observation-trial017', 'model_state_observation/results.json', 'receipt'),
        (NEST, 'model_nested_borrow_context/results.json', 'receipt'),
    ]
    deps = {}
    for directory, record, field in prerequisites:
        expected = read(ROOT/'source/tools/research'/record)[field]
        need(Path(expected['path']) == directory/'receipt.json', 'prerequisite receipt path')
        pin(directory/'receipt.json', expected['sha256'])
        r = read(directory/'receipt.json')
        need(r['success'] is True and r['model_proofs_kernel_checked'] is True, 'prerequisite complete')
        arts = artifacts(r, directory)
        family = record.split('/')[0]
        family = {'mutable_buffer_goal/parent-heap-initialization.json': 'native_heap_initialization',
                  'mutable_buffer_goal/parent-semantic-foundation.json': 'native_semantic_bridge',
                  'mutable_buffer_goal/parent-model-input-context.json': 'model_input_context'}.get(record, family)
        candidate = ROOT/'source/tools/research'/family/'candidates'
        if family == 'native_semantic_bridge':
            candidate /= 'loan-v12'
        for name, identity in arts.items():
            if name.endswith('.v') and name not in ('Assumptions.v', 'AssumptionInventory.v'):
                pin(candidate/name, identity)
        deps[str(directory)] = dict(receipt_sha256=sha(directory/'receipt.json'), artifacts=arts,
                                   repository_sources_match=True)
    # Accepted historical records are read only. Never follow their checkout paths.
    old = {}
    for record, modules in [('protocol-results.json', ['resource_protocol']),
                            ('child-results.json', ['resource_protocol', 'child']),
                            ('recovery-results.json', list(EXPECTED))]:
        rec = read(HERE/record)
        rp = Path(rec['receipt']['path'])
        need(rp.is_relative_to(TMP), 'accepted receipt location')
        pin(rp, rec['receipt']['sha256'])
        r = read(rp)
        commands(r, modules)
        need(r['constant_count'] == rec['constant_count'] == sum(len(EXPECTED[m]) for m in modules), 'historical constant count')
        need(r['assumptions'] == rec['assumptions'] == {n:xs for n,xs in ASSUMPTIONS.items() if n.split('.')[0] in modules}, 'historical assumptions')
        arts = artifacts(r, rp.parent)
        need({n:h for n,h in arts.items() if n.endswith('.vo')} == rec['proof_objects'], 'historical object map')
        for m in modules:
            pin(HERE/'candidates'/(m+'.v'), arts[m+'.v'])
        freeze_inputs = [name for name in r['inputs'] if name.endswith('-freeze.json')]
        need(len(freeze_inputs) == 1, 'historical freeze input')
        old_freeze = repo_input(freeze_inputs[0])
        pin(old_freeze, r['inputs'][freeze_inputs[0]]['sha256'])
        pin(rp.parent/'freeze.json', sha(old_freeze))
        old[record] = dict(receipt_sha256=sha(rp), artifacts=arts, logs=logs(r, rp.parent),
                           freeze_sha256=sha(old_freeze))
    recovery = read(Path(read(HERE/'recovery-results.json')['receipt']['path']))
    # Full 887-input accepted recovery identity comparison, rebasing source paths.
    allowed_tmp = [BUILD, TMP/'rocq-toolchain', *[d for d,_,_ in prerequisites]]
    for name, facts in recovery['inputs'].items():
        if name.startswith('/tmp/'):
            q = Path(name)
            need(any(q.is_relative_to(d) for d in allowed_tmp), 'unexpected historical temp input')
        else:
            q = repo_input(name)
        pin(q, facts['sha256'])
    fresh = read(receipt_path)
    commands(fresh, list(EXPECTED))
    need(fresh['constant_count'] == 5 and fresh['assumptions'] == ASSUMPTIONS, 'fresh five assumptions')
    fresh_arts = artifacts(fresh, receipt_path.parent)
    need(fresh_arts == old['recovery-results.json']['artifacts'], 'fresh source/object identities differ')
    # Reparse actual private logs, do not rely on booleans in any receipt.
    fresh_logs = logs(fresh, receipt_path.parent)
    kernel = fresh['kernel_axioms']
    pin(receipt_path.parent/'freeze.json', sha(HERE/'trial008-freeze.json'))
    for name, facts in fresh['inputs'].items():
        q = Path(name)
        need(q.is_relative_to(ROOT) or any(q.is_relative_to(d) for d in allowed_tmp), 'unexpected fresh input')
        pin(q, facts['sha256'])
    changed = subprocess.check_output(['git', 'diff', BASE, '--name-only'], cwd=ROOT, text=True).splitlines()
    untracked = subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard'], cwd=ROOT, text=True).splitlines()
    need(all(n.startswith('evidence/mutable-parent-recovery-review/') for n in changed + untracked), 'out-of-scope change')
    result = dict(schema=1, review_base=BASE, declarations=declarations, new_helpers=0,
        new_program_obligations=0, assumptions=ASSUMPTIONS,
        freeze_sha256=sha(HERE/'trial008-freeze.json'), foundation=dict(
            source_commit=OFFICIAL, sources=523, objects=284, readiness_sha256=sha(ready_path),
            receipt_sha256=sha(baseline_path), object_map_sha256=binding['compiled_module_map_sha256'],
            GMP_manifest_sha256=GMP, tools=binding['tools']), prerequisites=deps,
        accepted_checkpoints=old, accepted_recovery_inputs_authenticated=len(recovery['inputs']),
        fresh_replay=dict(path=str(receipt_path), sha256=sha(receipt_path),
            normal_compilation=True, full_recursive_coqchk=True, constant_count=5,
            source_and_object_map_matches_trial008=True, artifacts=fresh_arts,
            commands=[dict(label=c['label'], exit_code=c['exit_code'], seconds=c['seconds']) for c in fresh['commands']],
            logs=fresh_logs,
            inputs_authenticated=len(fresh['inputs']), kernel_axioms=kernel),
        native_correspondences=0, native_operations_open=44, native_phases_open=3,
        native_effects_open=101, HMAC='1/1 open', publication=False, goal_complete=False)
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
