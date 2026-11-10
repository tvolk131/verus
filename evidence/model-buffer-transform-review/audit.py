#!/usr/bin/env python3
"""Read-only identity/log audit of the exact conditional MODEL library.

Run check.py separately in a new private output, then pass its receipt here.
This auditor runs no compiler, mutation/control experiment, or foreign-checkout
read. Historical repository paths are matched as receipt metadata only.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shlex
import subprocess

ROOT = Path(__file__).resolve().parents[2]
LIB = ROOT / 'source/tools/research/model_buffer_transform'
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
BUILD = BASE / 'verus-belt-build-115a618d'
COMMIT = '2b7092b930db4f4a779bd2a71e2eb659734bf928'
NAMES = ['byte_value', 'zvalues', 'transform_prefix', 'zip_transform',
         'transform_prefix_length', 'transform_prefix_suffix',
         'binary_operation_spec', 'wp_zip_transform']
FOUNDATION_AXIOMS = [
    'Coq.Logic.FunctionalExtensionality.functional_extensionality_dep',
    'Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq',
]


def need(ok, message):
    if not ok:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load(path):
    return json.loads(Path(path).read_bytes())


def identity(path):
    return {'path': str(path), 'sha256': sha(path)}


def recorded_suffix(receipt, suffix):
    found = [info['sha256'] for name, info in receipt['inputs'].items()
             if name.endswith('/' + suffix)]
    need(len(found) == 1, 'ambiguous/missing recorded input: ' + suffix)
    return found[0]


def check_freeze(freeze_path, receipt, copied_source):
    frozen = load(freeze_path)
    need(frozen['before_first_trial'] is True, 'not marked pretrial')
    need(frozen['constants'] == {'transform': NAMES}, 'constant inventory')
    for name, value in frozen['files'].items():
        need(recorded_suffix(receipt, name) == value, 'unbound frozen input: ' + name)
    suffix = 'source/tools/research/model_buffer_transform/' + freeze_path.name
    need(recorded_suffix(receipt, suffix) == sha(freeze_path), 'retained freeze bytes')
    need(sha(copied_source) == frozen['files'][
        'source/tools/research/model_buffer_transform/candidates/transform.v'],
        'copied historical source')
    need((Path(copied_source).parent.parent / 'freeze.json').read_bytes()
         == freeze_path.read_bytes(), 'private retained freeze bytes')
    return frozen


def check_success(receipt_path, freeze, current=False):
    receipt = load(receipt_path)
    need(receipt['success'] is True and receipt['model_proofs_kernel_checked'] is True,
         'unsuccessful replay')
    need(receipt['constant_count'] == 8, 'not eight declarations')
    expected = {'transform.' + name: [] for name in NAMES}
    need(receipt['assumptions'] == expected, 'nonclosed or incomplete assumptions')
    need(receipt['publication'] is False and receipt['source_entry_proved'] is False
         and receipt['native_effects_discharged'] == 0, 'scope flags changed')
    need(receipt['kernel_axioms'] == FOUNDATION_AXIOMS, 'kernel summary axioms')
    out = receipt_path.parent
    need((out / 'freeze.json').read_bytes() == freeze.read_bytes(), 'retained freeze')
    proof_dir = out / 'proofs'
    products = {Path(name).name: info['sha256']
                for name, info in receipt['proof_artifacts'].items()}
    need(set(products) == {'transform.v', 'Assumptions.v', 'transform.vo', 'Assumptions.vo'},
         'proof product inventory')
    for name, value in products.items():
        need(sha(proof_dir / name) == value, 'proof product changed: ' + name)
    inv = 'From ModelBufferTransform Require Import transform.\n'
    for name in expected:
        inv += (f'Goal True. idtac "BEGIN {name}". exact I. Qed.\n'
                f'Print Assumptions ModelBufferTransform.{name}.\n'
                f'Goal True. idtac "END {name}". exact I. Qed.\n')
    need((proof_dir / 'Assumptions.v').read_text() == inv, 'inventory source coverage')
    raw = (out / 'assumptions.stdout').read_text()
    sections = re.findall(r'^BEGIN (\S+)\n(.*?)^END (\S+)\s*$', raw, re.M | re.S)
    need([a for a, _, _ in sections] == list(expected)
         == [b for _, _, b in sections], 'raw assumption boundaries')
    need(all(body.strip() == 'Closed under the global context' for _, body, _ in sections),
         'raw nonclosed assumptions')
    need(len(re.findall(r'^BEGIN ', raw, re.M)) == len(re.findall(r'^END ', raw, re.M)) == 8,
         'extra assumption boundaries')
    summary = '\nCONTEXT SUMMARY\n===============\n\n'
    summary += '* Theory: Set is predicative\n  \n'
    summary += '* Theory: Rewrite rules are not allowed\n  \n* Axioms:\n'
    summary += ''.join('    ' + name + '\n' for name in FOUNDATION_AXIOMS) + '  \n'
    summary += '* Constants/Inductives relying on type-in-type: <none>\n  \n'
    summary += '* Constants/Inductives relying on unsafe (co)fixpoints: <none>\n  \n'
    summary += '* Inductives whose positivity is assumed: <none>\n  \n\n'
    normalized = lambda text: [line.strip() for line in text.splitlines() if line.strip()]
    need(normalized((out / 'kernel.stderr').read_text()) == normalized(summary),
         'unsafe/different kernel context')
    args = []
    for line in (BUILD / '_CoqProject').read_text().splitlines():
        words = shlex.split(line, comments=True)
        if words and words[0] in ['-Q', '-R']:
            need(len(words) == 3, 'loadpath syntax')
            args.extend([words[0], str(BUILD / words[1]), words[2]])
    args.extend(['-Q', str(proof_dir), 'ModelBufferTransform'])
    runner = str(BASE / 'rocq-toolchain/run-rocq')
    bin_dir = BASE / 'rocq-toolchain/switch/_opam/bin'
    commands = [
        ('compile-transform', [runner, str(bin_dir / 'coqc'), '-q', *args, str(proof_dir / 'transform.v')]),
        ('assumptions', [runner, str(bin_dir / 'coqc'), '-q', *args, str(proof_dir / 'Assumptions.v')]),
        ('kernel', [runner, str(bin_dir / 'coqchk'), '-silent', '-o', *args,
                    'ModelBufferTransform.transform', 'ModelBufferTransform.Assumptions']),
    ]
    need(len(receipt['commands']) == 3, 'extra/missing commands')
    for (label, argv), observed in zip(commands, receipt['commands']):
        need(observed['label'] == label and observed['argv'] == argv
             and observed['cwd'] == str(proof_dir) and observed['exit_code'] == 0,
             'command/loadpaths/mode mismatch: ' + label)
    for suffix in ['compile-transform.stdout', 'compile-transform.stderr',
                   'assumptions.stderr', 'kernel.stdout']:
        need((out / suffix).read_bytes() == b'', 'unexpected diagnostic: ' + suffix)
    # Never follow historical repository paths to the parent/live checkout.
    # Match their relative identity against this isolated, read-only checkout.
    for name, value in load(freeze)['files'].items():
        need(recorded_suffix(receipt, name) == value == sha(ROOT / name),
             'current mandatory input: ' + name)
    if current:
        for name, info in receipt['inputs'].items():
            need(sha(name) == info['sha256'], 'fresh guarded input changed: ' + name)
    return {
        **identity(receipt_path), 'success': True, 'assumptions': expected,
        'kernel_axioms': FOUNDATION_AXIOMS, 'products': products,
        'commands': [{'label': c['label'], 'exit_code': c['exit_code'],
                      'seconds': c['seconds']} for c in receipt['commands']],
        'log_hashes': {p.name: sha(p) for p in sorted(out.glob('*.stdout'))
                       + sorted(out.glob('*.stderr'))},
        'normal_coqc_and_recursive_coqchk': True,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--replay', type=Path, required=True)
    parser.add_argument('--summary', type=Path, required=True)
    a = parser.parse_args()
    need(not a.replay.resolve().is_relative_to(ROOT), 'raw replay must stay private')
    need(a.summary.resolve().parent == Path(__file__).resolve().parent,
         'summary belongs only in this review directory')
    tree = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', COMMIT,
                                    '--', str(LIB.relative_to(ROOT))], cwd=ROOT).decode().splitlines()
    exact = {}
    for name in tree:
        committed = subprocess.check_output(['git', 'show', COMMIT + ':' + name], cwd=ROOT)
        need((ROOT / name).read_bytes() == committed, 'reviewed commit differs: ' + name)
        exact[name] = sha(ROOT / name)
    status = load(LIB / 'STATUS.json')
    accepted_path = Path(status['receipt']['path'])
    need(sha(accepted_path) == status['receipt']['sha256']
         == '687c9f1ab9a09456cf54cecfb8a440386b48e5a425ad7d7223cc92c7f7762aa7',
         'accepted receipt authentication')
    freeze = LIB / 'checked-loop-freeze.json'
    need(freeze.read_bytes() == (LIB / 'trial007-freeze.json').read_bytes(), 'freeze alias')
    needed = {
        'source/tools/research/model_buffer_transform/' + name
        for name in ['INTERFACE.md', 'check.py', 'candidates/transform.v']
    } | {
        'source/tools/research/native_semantic_bridge/' + name
        for name in ['proof_checks.py', 'acceptance_inputs.py', 'baseline-bindings.json']
    } | {
        'source/tools/research/mutable_buffer_goal/' + name
        for name in ['foundation-pin.json', 'foundation-baseline.json']
    } | {'source/tools/infer_eval_support.py', 'source/tools/infer_experiment.py'}
    need(set(load(freeze)['files']) == needed, 'mandatory inputs omitted/extra')
    source = (LIB / 'candidates/transform.v').read_text()
    need(re.findall(r'^\s*(?:Definition|Fixpoint|Lemma|Theorem)\s+(\w+)', source, re.M)
         == NAMES, 'source declaration inventory')
    need(not re.search(r'\b(Axiom|Admitted|admit|Parameter|Abort|Program|Obligation|'
                       r'native_compute|vm_cast_no_check|Unset Guard Checking|Unset Positivity Checking)\b', source),
         'unexpected trust/Program construct')
    trials = []
    for d in load(LIB / 'development.json'):
        p = Path(d['receipt'])
        need(sha(p) == d['receipt_sha256'], 'historical receipt changed')
        rec = load(p)
        n = d['trial']
        check_freeze(LIB / f'trial{n:03d}-freeze.json', rec, Path(d['source']))
        need(rec['success'] is (n == 7), 'historical outcome')
        row = {'trial': n, **identity(p), 'freeze': identity(LIB / f'trial{n:03d}-freeze.json'),
               'copied_source': identity(Path(d['source'])), 'success': rec['success']}
        if n != 7:
            need(len(rec['commands']) == 1 and rec['commands'][0]['label'] == 'compile-transform'
                 and rec['commands'][0]['exit_code'] == 1, 'not ordinary Coq compile failure')
            need(not rec['model_proofs_kernel_checked'], 'failed trial claims checked result')
            row['diagnostic'] = identity(p.parent / 'compile-transform.stderr')
            row['classification'] = 'ordinary proof-script development; no semantic countertheorem'
        trials.append(row)
    accepted = check_success(accepted_path, freeze)
    fresh = check_success(a.replay.resolve(), freeze, current=True)
    need(accepted['products'] == fresh['products'], 'fresh source/object mismatch')
    fresh_rec = load(a.replay)
    trusted = load(ROOT / 'source/tools/research/mutable_buffer_goal/foundation-baseline.json')
    pin = load(ROOT / 'source/tools/research/mutable_buffer_goal/foundation-pin.json')
    binding = load(ROOT / 'source/tools/research/native_semantic_bridge/baseline-bindings.json')
    ready_path = BASE / 'rocq-toolchain/baseline-ready.json'
    ready = load(ready_path)
    baseline_path = Path(ready['receipt'])
    baseline = load(baseline_path)
    need(sha(ready_path) == trusted['readiness']['sha256'], 'readiness identity')
    need(sha(baseline_path) == trusted['receipt']['sha256'], 'durable build identity')
    need(ready['baseline_kernel_checked'] is True and baseline['success'] is True,
         'readiness AND successful durable build required')
    need(ready['source_commit'] == pin['commit'] == binding['source_commit']
         == '115a618dd35ff8337112cf2081a7f5391d3973d5', 'foundation commit')
    need(trusted['toolchain']['Coq'] == '8.20.0', 'Coq version')
    stages = {c['label'] for c in baseline['commands'] if c['returncode'] == 0}
    need({'prover-version', 'build-baseline', 'kernel-check-baseline'} <= stages, 'durable stages')
    for c in baseline['commands']:
        if 'log' in c:
            need(sha(c['log']) == c['log_sha256'], 'durable stage log')
    sources = baseline['copied_sources']
    need(len(sources) == trusted['copied_source_files'] == 523, '523 sources required')
    for name, expected in sources.items():
        need(sha(BUILD / name) == expected, 'foundation source identity')
        need(fresh_rec['inputs'][str(BUILD / name)]['sha256'] == expected, 'unretained foundation source')
    objects = {str(p.relative_to(BUILD)): sha(p) for p in sorted((BUILD / 'src').rglob('*.vo'))}
    objhash = hashlib.sha256(json.dumps(objects, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    need(len(objects) == binding['compiled_module_count'] == 284
         and objhash == binding['compiled_module_map_sha256'], '284 objects identity')
    for name, expected in objects.items():
        need(fresh_rec['inputs'][str(BUILD / name)]['sha256'] == expected, 'unretained foundation object')
    for name, expected in binding['tools'].items():
        path = Path(ready['switch']) / '_opam/bin' / name if name in ['coqc', 'coqchk'] else Path(ready[name])
        need(sha(path) == fresh_rec['inputs'][str(path)]['sha256'] == expected, 'tool identity')
    gmp = BASE / 'rocq-toolchain/gmp-private/manifest.json'
    need(sha(gmp) == trusted['toolchain']['GMP_packages']['sha256']
         == '332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37', 'GMP pin')
    need(sha(BUILD / '_CoqProject') == pin['files']['_CoqProject'], 'loadpaths identity')
    for name, info in load(accepted_path)['inputs'].items():
        if name.startswith(str(BASE) + '/'):
            need(sha(name) == info['sha256'], 'accepted foundation/private input changed')
    summary = {
        'reviewed_commit': COMMIT, 'review_checkout_head': subprocess.check_output(
            ['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(),
        'verdict': 'accept exact conditional MODEL law only', 'actionable_findings': [],
        'exact_commit_files': exact, 'companion_report': identity(LIB / 'REPORT.md'),
        'mandatory_input_count': len(needed), 'accepted': accepted, 'fresh': fresh,
        'fresh_matches_accepted_sources_and_both_objects': True, 'development': trials,
        'foundation': {'source_commit': pin['commit'], 'Coq': '8.20.0',
                       'readiness': identity(ready_path), 'durable_build': identity(baseline_path),
                       'readiness_and_successful_durable_build': True,
                       'GMP_manifest': identity(gmp), 'source_count': len(sources),
                       'object_count': len(objects), 'object_map_sha256': objhash,
                       'loadpaths': identity(BUILD / '_CoqProject'), 'tools': binding['tools']},
        'no_new_program_obligations': True, 'native_correspondences': 0,
        'publication': False, 'goal_complete': False,
    }
    a.summary.write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({'summary': str(a.summary), 'verdict': summary['verdict']}))


if __name__ == '__main__':
    main()
