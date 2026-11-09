#!/usr/bin/env python3
"""Demonstrate report-validation gaps without invoking or replacing a compiler."""
import copy
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('pair_replay', ROOT / 'source/tools/research/scalar_tuple_output/replay.py')
replay = importlib.util.module_from_spec(spec)
spec.loader.exec_module(replay)

BASE = {
    'verification-results': {'success': True, 'is-verifying-entire-crate': True},
    'verus-infer': {'verification_scope': {'complete_selected_scope_verified': True},
                   'functions': []},
}


def main():
    results = []
    for label, data, ok, diagnostic in [
        ('positive_empty_inferred_functions', copy.deepcopy(BASE), True, None),
        ('semantic_negative_failed_helper_obligations', copy.deepcopy(BASE), False, 'postcondition not satisfied'),
        ('semantic_negative_mixed_precondition_failure', copy.deepcopy(BASE), False, 'postcondition not satisfied'),
    ]:
        stderr = ''
        if not ok:
            data['verification-results']['success'] = False
            data['verus-infer']['verification_scope']['complete_selected_scope_verified'] = False
            data['verus-infer']['functions'] = [{
                'function': 'U64GroupedBitmap::data_index_of', 'mode': 'exact', 'required_checks_passed': False,
                'proofs': {p: {'required': True, 'status': 'failed'}
                           for p in ('soundness', 'maximality', 'exactness')},
            }]
            stderr = diagnostic
        if 'mixed' in label:
            stderr += '\nprecondition not satisfied\n'
            function = data['verus-infer']['functions'][0]
            function['required_checks_passed'] = True
            for proof in function['proofs'].values():
                proof['status'] = 'proved'
        run = dict(returncode=0 if ok else 1, stdout=json.dumps(data), stderr=stderr)
        try:
            replay.check_run(run, ok, diagnostic)
        except Exception as error:
            results.append(dict(label=label, accepted=False, error=str(error)))
        else:
            results.append(dict(label=label, accepted=True))
    print(json.dumps(results, indent=2))


if __name__ == '__main__':
    main()
