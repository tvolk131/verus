#!/usr/bin/env python3
"""Read-only authentication of completed MODEL replays; write a compact review receipt."""
import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LANE = ROOT / 'source/tools/research/model_input_context'
FE = 'Coq.Logic.FunctionalExtensionality.functional_extensionality_dep'
EQ = 'Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq'


def need(ok, message):
    if not ok:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read(path):
    return json.loads(Path(path).read_bytes())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--replay', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    results = read(LANE / 'results.json')
    worker_path = Path(results['receipt']['path'])
    need(sha(worker_path) == results['receipt']['sha256'], 'worker receipt identity')
    worker = read(worker_path)
    fresh_path = args.replay / 'receipt.json'
    fresh = read(fresh_path)
    original = read(LANE / 'final-freeze.json')
    parent = read(LANE / 'parent-freeze.json')
    changed = [n for n in original['files'] if original['files'][n] != parent['files'][n]]
    need(changed == [
        'source/tools/research/mutable_buffer_goal/parent-heap-initialization.json',
        'source/tools/research/mutable_buffer_goal/parent-semantic-foundation.json'],
        'unexpected parent reconciliation')
    need(original['constants'] == parent['constants'], 'changed declaration inventory')
    need(set(original['files']) == set(parent['files']), 'changed freeze coverage')
    for name, expected in parent['files'].items():
        need(sha(ROOT / name) == expected, 'current frozen input: ' + name)
    need(sha(worker_path.parent / 'freeze.json') == sha(LANE / 'final-freeze.json')
         == results['final_freeze_sha256'], 'worker freeze identity')
    need(sha(args.replay / 'freeze.json') == sha(LANE / 'parent-freeze.json'), 'fresh freeze identity')
    modules = results['modules']
    names = [m + '.' + n for m in modules for n in parent['constants'][m]]
    need(len(names) == len(set(names)) == 26, '26-constant inventory')
    for module in modules:
        source = LANE / 'candidates' / (module + '.v')
        text = source.read_text()
        declarations = re.findall(r'^\s*(?:Definition|Fixpoint|Lemma|Theorem|Corollary)\s+(\w+)', text, re.M)
        need(declarations == parent['constants'][module], 'declarations: ' + module)
        need(not re.search(r'\b(Program|Obligation|Axiom|Admitted|admit|Parameter|Abort|native_compute|vm_cast_no_check|Unset)\b', text),
             'unexpected source construct: ' + module)
        need(sha(source) == sha(worker_path.parent / 'proofs' / source.name)
             == sha(args.replay / 'proofs' / source.name), 'source copies: ' + module)
    commands = ['compile-' + m for m in modules] + ['assumptions', 'kernel']
    for label, receipt, directory in [('worker', worker, worker_path.parent), ('fresh', fresh, args.replay)]:
        need(receipt['success'] is True and receipt['model_proofs_kernel_checked'] is True,
             label + ' completed proof check')
        need(receipt['constant_count'] == 26, label + ' count')
        need(receipt['assumptions'] == results['assumptions'], label + ' assumptions')
        need(set(receipt['assumptions']) == set(names), label + ' assumption coverage')
        need(set(receipt['kernel_axioms']) == {FE, EQ}, label + ' kernel axioms')
        need([c['label'] for c in receipt['commands']] == commands, label + ' stages')
        for command in receipt['commands']:
            need(command['exit_code'] == 0, label + ' stage success')
            need(not {'-vos', '-vok', '-quick'} & set(command['argv']), label + ' normal compilation')
        for path, facts in receipt['proof_artifacts'].items():
            need(sha(directory / 'proofs' / Path(path).name) == facts['sha256'], label + ' product identity')
        for name, expected in results['proof_objects'].items():
            need(sha(directory / 'proofs' / name) == expected, label + ' object matches worker')
    need(sha(results['kernel_log']['path']) == results['kernel_log']['sha256'], 'worker kernel log')
    need(sha(args.replay / 'kernel.stderr') == results['kernel_log']['sha256'], 'fresh kernel summary matches')
    for path, facts in fresh['inputs'].items():
        need(sha(path) == facts['sha256'], 'fresh retained input identity: ' + path)
    closed = [n for n in names if not fresh['assumptions'][n]]
    fe = [n for n in names if fresh['assumptions'][n] == [FE]]
    eq = [n for n in names if fresh['assumptions'][n] == [EQ]]
    need((len(closed), len(fe), len(eq)) == (11, 12, 3), 'assumption groups')
    need(all(not fresh['assumptions'][n] for n in names if n.startswith('vector.')), 'vector closed')
    development = read(LANE / 'development.json')
    need(sha(development['retained_manifest']['path']) == development['retained_manifest']['sha256'],
         'historical manifest identity')
    failures = []
    for trial in development['trials']:
        n = trial['trial']
        directory = Path(development['trial_output_pattern'].format(trial=n))
        freeze = Path(development['frozen_snapshots'].format(trial=n))
        need(sha(freeze) == trial['freeze_sha256'], 'historical freeze ' + str(n))
        need(sha(directory / 'receipt.json') == trial['receipt_sha256'], 'historical receipt ' + str(n))
        receipt = read(directory / 'receipt.json')
        need(receipt['success'] == trial['success'], 'historical outcome ' + str(n))
        if not receipt['success']:
            need(receipt['model_proofs_kernel_checked'] is False, 'failure mislabelled ' + str(n))
            failures.append(n)
    need(len(development['trials']) == 32 and len(failures) == 30, 'history counts')
    summary = dict(
        schema=1, outcome='ACCEPT exact MODEL prerequisite; no mathematical/controller findings',
        checkout=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        reviewed_revision='94b87742', model_sources_and_checker_equal_worker=True,
        worker_receipt=dict(path=str(worker_path), sha256=sha(worker_path), execution='read authenticated completed receipt'),
        fresh_receipt=dict(path=str(fresh_path), sha256=sha(fresh_path), execution='executed fresh normal replay by reviewer'),
        freezes={n: sha(LANE / n) for n in ['final-freeze.json', 'parent-freeze.json']},
        parent_freeze_changes=changed, source_hashes={m: sha(LANE / 'candidates' / (m + '.v')) for m in modules},
        checker_sha256=sha(LANE / 'check.py'), constant_count=26, program_obligations=0,
        assumptions=dict(closed=closed, functional_extensionality_dep=fe, eq_rect_eq=eq),
        fresh_commands=[dict(label=c['label'], exit_code=c['exit_code'], seconds=c['seconds']) for c in fresh['commands']],
        exact_argv_and_cwd='fresh receipt commands; all prover cwd paths private to fresh replay',
        proof_objects=results['proof_objects'], all_six_objects_match_worker=True,
        kernel_summary_sha256=sha(args.replay / 'kernel.stderr'), kernel_axioms=fresh['kernel_axioms'],
        all_fresh_receipt_inputs_rehashed=True, historical_freeze_receipt_pairs_authenticated=32,
        failed_development_trials=failures, historical_trials_rerun=False,
        parent_active_replay_inspected=False, filesystem_or_source_mutation_experiments=False,
        foundation_rebuilt_or_modified=False, source_entry_proved=False,
        final_heap_writer_frame_theorem_proved=False, publication=False, full_goal_complete=False)
    args.output.write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(dict(outcome=summary['outcome'], evidence=str(args.output))))


if __name__ == '__main__':
    main()
