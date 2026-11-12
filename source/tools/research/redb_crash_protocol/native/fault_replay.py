#!/usr/bin/env python3
"""Prospectively frozen finite native schedules; never a source proof."""
import argparse
import json
import os
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt, require

SCENARIOS = ['write1_none', 'write1_partial', 'sync1_none', 'sync1_header',
             'sync1_all', 'sync2_none', 'sync2_all', 'publish_none',
             'publish_control', 'page1_none', 'success', 'corrupt_rewrite']


def expected_pair(prior, scenario):
    if scenario in ['success', 'sync2_all', 'publish_control']:
        return 'new'
    if scenario in ['sync1_all', 'sync2_none', 'publish_none']:
        return 'old' if prior else 'new'
    return 'old'


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--freeze', type=Path, default=HERE / 'FAULT_FREEZE.json')
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    receipt = {'kind': 'FINITE_NATIVE_FAILURE_MATRIX_NOT_SOURCE_PROOF', 'commands': [], 'outcomes': [],
               'formal_source_proof': False, 'source_coupling_proved': False}
    with replay_receipt(out, receipt):
        freeze = json.loads(args.freeze.read_text())
        for path, value in freeze['inputs'].items():
            require(digest(Path(path)) == value, 'input hash: ' + path)
        prepared = json.loads(Path(freeze['preparation_receipt']).read_text())
        require(prepared.get('success') is True, 'successful original native preparation required')
        for path, value in prepared['prepared_inputs'].items():
            require(digest(Path(path)) == value, 'prepared source changed: ' + path)
        guard = ArtifactGuard([args.freeze] + [Path(p) for p in freeze['inputs']] +
                              [Path(p) for p in prepared['prepared_inputs']])
        run = LoggedCommands(out, ROOT, receipt['commands'], timeout=120, guard=guard)
        tc = Path(freeze['toolchain'])
        env = {k: v for k, v in os.environ.items() if not k.startswith('CARGO_') and k not in
               ['RUSTFLAGS', 'RUSTDOCFLAGS', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'RUSTC', 'RUSTDOC']}
        env.update(PATH=str(tc / 'bin') + ':/usr/bin:/bin', RUSTUP_HOME='/home/dev/.rustup',
                   RUSTUP_TOOLCHAIN=str(tc), LD_LIBRARY_PATH=str(tc / 'lib'))
        binary = out / 'faults'
        command = [str(tc / 'bin/rustc'), str(HERE / 'faults.rs'), '--edition=2024', '-O',
                   '-L', 'dependency=' + freeze['dependency_dir'], '--extern',
                   'redb=' + freeze['redb_rlib'], '-o', str(binary)]
        require(run('build_fault_harness', command, env) == 0, 'fault harness compilation')
        receipt['binary_sha256'] = digest(binary)
        guard = ArtifactGuard([Path(p) for p in guard.paths] + [binary])
        run.guard = guard
        errors = []
        for prior in [False, True]:
            for scenario in SCENARIOS:
                label = str(prior).lower() + '_' + scenario
                code = run(label, [str(binary), str(prior).lower(), scenario], env)
                item = dict(label=label, exit_code=code,
                            stdout_sha256=digest(out / (label + '.stdout')),
                            stderr_sha256=digest(out / (label + '.stderr')))
                try:
                    require(code == 0, 'native execution failed')
                    observation = json.loads((out / (label + '.stdout')).read_text())
                    require(observation['prior'] is prior and observation['scenario'] == scenario,
                            'requested profile missing')
                    require(observation['formal_source_proof'] is False, 'native result scope')
                    if scenario == 'corrupt_rewrite':
                        require(observation['repair_aborted'] is True and
                                observation['raw_slot_preserved'] is True and
                                observation['header_rewrites'] > 0 and observation['repair_callbacks'] > 0,
                                'non-vacuous corrupt slot regression')
                    else:
                        require(observation['commit_ok'] is (scenario == 'success'), 'public return')
                        require(observation['injection_fired'] is (scenario != 'success'), 'injection reachability')
                        require(observation['fixed_layout'] is True, 'outside registered fixed-layout case')
                        require(observation['reopened_pair'] == expected_pair(prior, scenario),
                                'expected coherent snapshot not recovered')
                        if scenario == 'write1_partial':
                            require(observation['partial_slot_invalid'] is True, 'tear not actually detected')
                        if scenario == 'success':
                            require(sum(e['kind'] == 'sync_success' for e in observation['events']) >= 2,
                                    'two barriers not observed')
                    item['observation'] = observation
                    item['passed'] = True
                except (ValueError, KeyError, TypeError) as error:
                    item['passed'] = False
                    item['failure'] = str(error)
                    errors.append(label + ': ' + str(error))
                receipt['outcomes'].append(item)
                print(label, 'PASS' if item['passed'] else item['failure'], flush=True)
        guard.check()
        receipt['final_guard_passed'] = True
        receipt['total_cases'] = len(receipt['outcomes'])
        receipt['passed_cases'] = sum(x['passed'] for x in receipt['outcomes'])
        require(not errors, '; '.join(errors))


if __name__ == '__main__':
    main()
