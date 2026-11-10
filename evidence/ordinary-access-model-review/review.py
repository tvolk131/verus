#!/usr/bin/env python3
"""Read-only authentication plus fresh, private ordinary-access MODEL replay.

Run with python3 -B; output must not exist. No historical checkout path is opened.
The unchanged component checker performs the pinned baseline/dependency gates.
This independent wrapper authenticates the completed evidence, checks exhaustive
source and compiled-module inventories, and guards identities through a second
full recursive kernel check including the declaration-free inventory module.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
COMPONENT = ROOT / 'source/tools/research/model_non_atomic_access'
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
OLD = BASE / 'parent-non-atomic-access-006/proofs'
BUILD = BASE / 'verus-belt-build-115a618d'
sys.path[:0] = [str(ROOT / 'source/tools'),
               str(ROOT / 'source/tools/research/native_semantic_bridge')]
from infer_eval_support import ArtifactGuard, digest
from proof_checks import parse_axioms, parse_kernel_summary

NAMES = ['silent_step', 'sc_read_one_step', 'na_read_two_steps',
         'sc_write_one_step', 'na_write_two_steps', 'na_read_phase_changes',
         'na_write_readers_blocked', 'cell_assignment_na', 'wp_cell_assignment_na']
KINDS = ['Definition', *(['Lemma'] * 6), 'Definition', 'Lemma']


def need(test, message):
    if not test:
        raise ValueError(message)


def load(path):
    return json.loads(path.read_text())


def historical_path(label):
    # These are labels in the authenticated receipt, never checkout locations
    # to visit. Only the explicitly completed /tmp dependencies are read.
    prefix = '/home/dev/projects/verus/'
    if label.startswith(prefix):
        relative = Path(label[len(prefix):])
        need('..' not in relative.parts, 'invalid source label')
        return ROOT / relative
    p = Path(label)
    allowed = [BUILD, BASE / 'rocq-toolchain', OLD,
               BASE / 'buffer-transform-trial007',
               Path('/tmp/verus-byte-xor-trial017'),
               Path('/tmp/verus-native-byte-xor-fbb7afa6-trial008')]
    need(any(p.is_relative_to(a) for a in allowed), 'unapproved artifact: ' + label)
    return p


def strip_comments_strings(source):
    # Nested Coq comments and doubled quote escapes, without changing newlines.
    output = []
    depth = 0
    string = False
    i = 0
    while i < len(source):
        pair = source[i:i+2]
        c = source[i]
        if depth:
            if pair == '(*':
                depth += 1
                output.extend('  ')
                i += 2
                continue
            if pair == '*)':
                depth -= 1
                output.extend('  ')
                i += 2
                continue
            output.append('\n' if c == '\n' else ' ')
        elif string:
            if pair == '""':
                output.extend('  ')
                i += 2
                continue
            if c == '"':
                string = False
            output.append('\n' if c == '\n' else ' ')
        elif pair == '(*':
            depth = 1
            output.extend('  ')
            i += 2
            continue
        elif c == '"':
            string = True
            output.append(' ')
        else:
            output.append(c)
        i += 1
    need(not depth and not string, 'unterminated comment/string')
    return ''.join(output)


def assumptions(path):
    text = path.read_text()
    sections = re.findall(r'^BEGIN (\S+)\n(.*?)^END (\S+)\s*$', text, re.M | re.S)
    names = ['access.' + n for n in NAMES]
    need([a for a, _, _ in sections] == names == [b for _, _, b in sections],
         'incomplete assumption inventory')
    need(text.count('BEGIN ') == text.count('END ') == len(names), 'extra markers')
    result = {n: parse_axioms(body) for n, body, _ in sections}
    need(all(v == [] for v in result.values()), 'candidate assumption is not closed')
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    out = args.output.resolve()
    need(not out.is_relative_to(ROOT), 'raw outputs must be private')
    out.mkdir(parents=True, exist_ok=False)
    os.chmod(out, 0o700)
    summary = {'schema': 1, 'success': False, 'commands': []}
    try:
        result = load(COMPONENT / 'results.json')
        old_receipt = load(OLD / 'receipt.json')
        final = load(COMPONENT / 'final-freeze.json')
        scope = load(HERE / 'scope.json')
        need(result['receipt']['path'] == str(OLD / 'receipt.json'), 'receipt path')
        need(digest(OLD / 'receipt.json') == result['receipt']['sha256'], 'receipt hash')
        need(old_receipt['success'] is True and old_receipt['model_proofs_kernel_checked'] is True,
             'completed evidence not successful')
        need(all(c['exit_code'] == 0 for c in old_receipt['commands']), 'old command failure')
        need([c['label'] for c in old_receipt['commands']] ==
             ['compile-access', 'assumptions', 'kernel'], 'old command inventory')
        need(old_receipt['constant_count'] == result['constant_count'] == 9, 'count')
        need(final['constants'] == {'access': NAMES}, 'frozen declaration inventory')
        need(digest(COMPONENT / 'final-freeze.json') == scope['final_freeze_sha256'], 'freeze identity')
        need((OLD / 'freeze.json').read_bytes() == (COMPONENT / 'final-freeze.json').read_bytes(),
             'completed freeze differs from final freeze')
        component_files = subprocess.check_output(
            ['git', 'ls-files', str(COMPONENT.relative_to(ROOT))], cwd=ROOT, text=True).splitlines()
        for name in component_files:
            need((ROOT / name).read_bytes() == subprocess.check_output(
                ['git', 'show', '8438803e:' + name], cwd=ROOT), 'component differs from 8438803e: ' + name)
        paths = [HERE / 'review.py', HERE / 'scope.json', *[ROOT / n for n in component_files],
                 OLD / 'receipt.json', OLD / 'freeze.json']
        for category in ['inputs', 'proof_artifacts']:
            for label, facts in old_receipt[category].items():
                actual = historical_path(label)
                need(digest(actual) == facts['sha256'], 'completed input/product mismatch: ' + label)
                paths.append(actual)
        old_artifacts = {Path(n).name: v for n, v in old_receipt['proof_artifacts'].items()}
        need(set(old_artifacts) == {'access.v', 'access.vo', 'Assumptions.v', 'Assumptions.vo'},
             'completed proof artifact inventory')
        for name, expected in result['proof_objects'].items():
            need(old_artifacts[name]['sha256'] == expected, 'results object hash')
        need(old_artifacts['access.v']['sha256'] == scope['candidate_sha256'], 'source hash')
        for name in ['assumptions.stdout', 'assumptions.stderr', 'compile-access.stdout',
                     'compile-access.stderr', 'kernel.stdout', 'kernel.stderr']:
            paths.append(OLD / name)
        need(assumptions(OLD / 'assumptions.stdout') == result['assumptions'] ==
             old_receipt['assumptions'], 'completed assumption report mismatch')
        need(parse_kernel_summary((OLD / 'kernel.stderr').read_text()) == old_receipt['kernel_axioms'],
             'completed kernel summary mismatch')
        source = strip_comments_strings((COMPONENT / 'candidates/access.v').read_text())
        declarations = re.findall(r'^\s*(Definition|Lemma)\s+(\w+)', source, re.M)
        need(declarations == list(zip(KINDS, NAMES)), 'source declaration inventory')
        forbidden = r'\b(Axiom|Axioms|Parameter|Parameters|Variable|Variables|Hypothesis|Hypotheses|Conjecture|Admitted|admit|Abort|Program|Obligation|Obligations|Instance|Instances|Inductive|CoInductive|Record|Structure|Fixpoint|CoFixpoint|Theorem|Fact|Remark|Corollary|Proposition|Example|Let|native_compute|vm_cast_no_check|Unset|Add\s+LoadPath)\b'
        need(re.search(forbidden, source) is None, 'extra declaration/unsafe construct')
        need(len(re.findall(r'\bQed\.', source)) == 7, 'lemma closure inventory')
        need('Context `{!lrustGS Σ}.' in source, 'WP model context changed')
        # Snapshot prior to any proof trial; originals are hashed again through
        # final checks, and are never modified, copied back, or rebuilt.
        guard = ArtifactGuard(paths)
        summary['completed_authentication'] = {
            'receipt': result['receipt'], 'input_count': len(old_receipt['inputs']),
            'proof_artifacts': {n: f['sha256'] for n, f in old_artifacts.items()},
            'assumptions': old_receipt['assumptions'],
            'historical_source_paths': 'remapped labels into own checkout only',
            'all_recorded_inputs_and_products_match': True}
        inventory_text = ('From ModelNonAtomicAccess Require Import access.\n'
                          'Print Module ModelNonAtomicAccess.access.\n')
        for name in NAMES:
            inventory_text += 'About ModelNonAtomicAccess.access.' + name + '.\n'
        inventory_dir = out / 'inventory'
        inventory_dir.mkdir()
        inventory = inventory_dir / 'Inventory.v'
        inventory.write_text(inventory_text)
        inventory_guard = ArtifactGuard([inventory])
        (out / 'pretrial-freeze.json').write_text(json.dumps({
            'schema': 1, 'before_first_trial': True, 'input_snapshot': guard.initial,
            'supplementary_inventory_source': inventory_guard.initial,
            'exact_declarations': declarations,
            'instances': [], 'constructors': [], 'obligations': []}, indent=2) + '\n')
        summary['pretrial_freeze_sha256'] = digest(out / 'pretrial-freeze.json')
        env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')

        def run(label, argv, cwd):
            guard.check()
            inventory_guard.check()
            started = time.monotonic()
            with (out / (label + '.stdout')).open('w') as stdout, \
                    (out / (label + '.stderr')).open('w') as stderr:
                process = subprocess.run(argv, cwd=cwd, env=env, stdout=stdout,
                                         stderr=stderr, timeout=1800)
            summary['commands'].append({'label': label, 'argv': argv, 'cwd': str(cwd),
                                        'exit_code': process.returncode,
                                        'seconds': time.monotonic() - started})
            guard.check()
            inventory_guard.check()
            need(process.returncode == 0, 'setup/development/replay failure, not semantics: ' + label)

        run('fresh-frozen-replay', [sys.executable, '-B', str(COMPONENT / 'check.py'),
                                  '--freeze', str(COMPONENT / 'final-freeze.json'),
                                  '--output', str(out / 'trial001')], out)
        fresh = load(out / 'trial001/receipt.json')
        need(fresh['success'] is True and fresh['model_proofs_kernel_checked'] is True, 'fresh replay')
        need(assumptions(out / 'trial001/assumptions.stdout') == old_receipt['assumptions'] ==
             fresh['assumptions'], 'fresh all-nine assumptions differ')
        need(fresh['constant_count'] == 9, 'fresh declaration count')
        fresh_paths = [out / 'trial001/receipt.json', out / 'trial001/freeze.json']
        for label, facts in fresh['proof_artifacts'].items():
            need(digest(Path(label)) == facts['sha256'], 'fresh source/object identity')
            fresh_paths.append(Path(label))
        fresh_guard = ArtifactGuard(fresh_paths)
        compile_command = fresh['commands'][0]['argv']
        need(compile_command[2] == '-q', 'normal coqc required')
        loadpaths = compile_command[3:-1]
        run('compiled-module-inventory', [*compile_command[:3], *loadpaths,
                                          '-Q', str(inventory_dir), 'OrdinaryAccessReview', str(inventory)], out)
        module_output = (out / 'compiled-module-inventory.stdout').read_text()
        module_part = module_output.split('End', 1)[0]
        compiled = re.findall(r'^\s*(?:Definition|Parameter)\s+(\w+)\s*:', module_part, re.M)
        need(compiled == NAMES, 'compiled module inventory differs: ' + repr(compiled))
        need(not re.search(r'\b(Inductive|Record|Instance|obligation_\w*)\b', module_part),
             'extra compiled constructor/instance/obligation')
        supplementary_guard = ArtifactGuard([inventory, inventory.with_suffix('.vo')])
        # No admission, norec, exclusions, unsafe flags, or user-supplied axioms.
        kernel_command = fresh['commands'][-1]['argv']
        need(kernel_command[2:4] == ['-silent', '-o'], 'full recursive coqchk required')
        run('full-recursive-kernel-with-inventory',
            [*kernel_command[:4], *loadpaths, '-Q', str(inventory_dir), 'OrdinaryAccessReview',
             'ModelNonAtomicAccess.access', 'ModelNonAtomicAccess.Assumptions',
             'OrdinaryAccessReview.Inventory'], out)
        axioms = parse_kernel_summary((out / 'full-recursive-kernel-with-inventory.stderr').read_text())
        need(axioms == fresh['kernel_axioms'] == old_receipt['kernel_axioms'], 'kernel context changed')
        guard.check()
        fresh_guard.check()
        inventory_guard.check()
        supplementary_guard.check()
        summary.update(success=True, model_only=True, source_commit=scope['reviewed_commit'],
                       candidate_sha256=scope['candidate_sha256'],
                       final_freeze_sha256=scope['final_freeze_sha256'],
                       exact_declarations=declarations, compiled_declarations=compiled,
                       instances=[], constructors=[], obligations=[],
                       assumptions=fresh['assumptions'], kernel_axioms=axioms,
                       fresh_receipt={'path': str(out / 'trial001/receipt.json'),
                                      'sha256': digest(out / 'trial001/receipt.json')},
                       fresh_proof_artifacts={Path(p).name: f['sha256']
                                              for p, f in fresh['proof_artifacts'].items()},
                       all_final_guards_unchanged=True,
                       guarded_input_count=len(guard.initial),
                       native_correspondences=0, native_effects_open=101,
                       publication=False, goal_complete=False)
    except BaseException as error:
        summary['failure'] = {'type': type(error).__name__, 'message': str(error),
                              'classification': 'setup/development/review failure, never semantic evidence'}
        raise
    finally:
        summary['raw_outputs'] = {p.name: digest(p) for p in out.iterdir() if p.is_file()}
        (out / 'review-receipt.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({'output': str(out), 'success': True,
                      'receipt_sha256': digest(out / 'review-receipt.json')}))


if __name__ == '__main__':
    main()
