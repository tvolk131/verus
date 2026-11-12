#!/usr/bin/env python3
"""Run only the prospectively registered native smoke cases; no proof claim."""
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


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=False)
    f = json.loads((HERE / 'SMOKE_FREEZE.json').read_text())
    r = {'kind': 'TWO_NATIVE_NO_FAULT_SMOKE_CASES_NOT_SOURCE_PROOF', 'commands': [], 'outcomes': []}
    with replay_receipt(a.output, r):
        for name, expected in f['inputs'].items():
            require(digest(Path(name)) == expected, 'frozen smoke input ' + name)
        prep = json.loads(Path(f['preparation_receipt']).read_text())
        require(prep['success'] is True, 'completed native preparation required')
        binary = Path(prep['binary'])
        require(digest(binary) == prep['binary_sha256'], 'native executable bytes')
        inputs = [Path(n) for n in f['inputs']] + [HERE / 'SMOKE_FREEZE.json', binary]
        for name, expected in prep['prepared_inputs'].items():
            require(digest(Path(name)) == expected, 'original native context ' + name)
            inputs.append(Path(name))
        guard = ArtifactGuard(inputs)
        run = LoggedCommands(a.output, a.output, r['commands'], timeout=120, guard=guard)
        for prior in ['false', 'true']:
            label = 'prior_' + prior
            code = run(label, [str(binary), prior], dict(os.environ))
            require(code == 0, 'native public-API smoke failure')
            data = json.loads((a.output / (label + '.stdout')).read_text())
            require(data['prior_requested'] is (prior == 'true') and data['prior_header_mode'] is (prior == 'true'), 'actual mode history')
            require(data['current_header_mode'] is True and data['commit_ok'] is True, 'successful requested durable commit')
            require(data['reopened_pair'] == 'new' and data['formal_source_proof'] is False, 'exact data/no proof promotion')
            require(sum(e['kind'] == 'sync_success' for e in data['events']) >= 2, 'observed two-phase barriers')
            data['configuration_match'] = bool(data['fixed_layout'])
            data['stdout_sha256'] = digest(a.output / (label + '.stdout'))
            data['stderr_sha256'] = digest(a.output / (label + '.stderr'))
            r['outcomes'].append(data)
            print(json.dumps({'prior': prior, 'reopened': data['reopened_pair'], 'events': len(data['events']), 'fixed_layout': data['fixed_layout']}), flush=True)
        r['formal_source_proof'] = False
        r['fault_schedules_executed'] = 0
        guard.check()


if __name__ == '__main__':
    main()
