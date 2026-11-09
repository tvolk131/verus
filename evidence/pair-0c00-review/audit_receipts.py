#!/usr/bin/env python3
"""Check fresh replay coverage independently of the scenario validators."""
import argparse
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_experiment import require, require_exact

PAIR_NAMES = {'data_index_of', 'signed', 'slice_pair', 'arithmetic', 'composed',
              'branch', 'ignored_left', 'ignored_right'}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('receipt', type=Path)
    a = p.parse_args()
    receipt = json.loads(a.receipt.read_text())
    require(receipt['success'] is True, 'replay did not finish successfully')
    require(len(receipt['runs']) == 48, 'missing replay runs')
    summaries = []
    for row in receipt['runs']:
        label = row['label']
        data = row['result']
        verification = data['verification-results']
        info = data.get('verus-infer', {})
        functions = info.get('functions', [])
        names = [f['function'].rsplit('::', 1)[-1] for f in functions]
        require(len(names) == len(set(names)), label + ': duplicate helpers')
        if label in ['pairs_portable', 'model_32', 'model_64'] or label.startswith('unsafe_'):
            expected = PAIR_NAMES
        elif label.startswith('stateful'):
            expected = {'data_index_of', 'select_mask'}
        elif label in ['consumer', 'bridge_provider']:
            expected = {'composed'}
        elif (label.startswith(('provider_', 'cache_', 'changed_'))
              or label in ['restore_provider', 'failed_export', 'no_verify_export']):
            expected = {'split'}
        else:
            expected = set()
        require(set(names) == expected, label + ': helper coverage mismatch')
        # Every ordinary negative still has complete exact helper checks.
        if label not in ['cache_swapped', 'no_verify_export']:
            require_exact(functions)
        stderr = (a.receipt.parent / (label + '.stderr')).read_text()
        if label.startswith('stateful') or label in ['historical_baseline', 'forwarding_wrapper']:
            if verification.get('success') is False:
                require('postcondition not satisfied' in stderr, label)
                require('precondition not satisfied' not in stderr and 'assertion failed' not in stderr,
                        label + ': mixed semantic failure')
        if label.startswith('unsafe_'):
            require('precondition not satisfied' in stderr, label)
            require('postcondition not satisfied' not in stderr and 'assertion failed' not in stderr,
                    label + ': mixed semantic failure')
        summaries.append(dict(label=label, helpers=names,
            verification=verification, required_passed=[f['required_checks_passed'] for f in functions]))
    print(json.dumps(summaries, indent=2))


if __name__ == '__main__':
    main()
