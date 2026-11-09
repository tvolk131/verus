"""Fail-closed source and outcome checks, including optimized Python."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('scalar_pair_replay', HERE/'replay.py')
replay = importlib.util.module_from_spec(spec)
spec.loader.exec_module(replay)


class Guards(unittest.TestCase):
    def test_frozen_sources(self):
        replay.fidelity()

    def test_changed_frozen_source_rejected(self):
        with patch.object(replay, 'sha', return_value='changed'):
            with self.assertRaisesRegex(ValueError, 'frozen input changed'):
                replay.fidelity()

    def test_changed_stateful_body_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'changed.rs'
            path.write_text((HERE/'automatic_inferred_helpers.rs').read_text().replace('group |=', 'group ^='))
            with self.assertRaisesRegex(ValueError, 'stateful helper adaptation changed'):
                replay.fidelity(path)

    def checked(self):
        return dict(returncode=0, stderr='', stdout=json.dumps({
            'verification-results': {'success': True, 'is-verifying-entire-crate': True},
            'verus-infer': {
                'verification_scope': {'complete_selected_scope_verified': True},
                'functions': [{'function': 'probe::split', 'mode': 'exact', 'required_checks_passed': True,
                    'proofs': {p: {'required': True, 'status': 'proved'}
                               for p in ('soundness', 'maximality', 'exactness')}}]}}))

    def test_checked_outcome(self):
        replay.check_run(self.checked(), True, expected_functions={'split'})

    def test_missing_tuple_exactness_rejected(self):
        result = self.checked()
        data = json.loads(result['stdout'])
        del data['verus-infer']['functions'][0]['proofs']['exactness']
        result['stdout'] = json.dumps(data)
        with self.assertRaisesRegex(ValueError, 'mandatory exact inference checks'):
            replay.check_run(result, True, expected_functions={'split'})

    def test_incomplete_scope_rejected(self):
        result = self.checked()
        data = json.loads(result['stdout'])
        data['verus-infer']['verification_scope']['complete_selected_scope_verified'] = False
        result['stdout'] = json.dumps(data)
        with self.assertRaisesRegex(ValueError, 'incomplete verification scope'):
            replay.check_run(result, True, expected_functions={'split'})

    def test_frontend_rejection_is_not_a_semantic_negative(self):
        result = dict(returncode=1, stderr='postcondition not satisfied', stdout=json.dumps({
            'verification-results': {'success': False, 'encountered-vir-error': True}}))
        with self.assertRaisesRegex(ValueError, 'frontend failure'):
            replay.check_run(result, False, 'postcondition not satisfied', expected_functions={'split'})

    def test_unexpected_failure_rejected(self):
        result = self.checked()
        result['returncode'] = 1
        with self.assertRaisesRegex(ValueError, 'unexpected verification outcome'):
            replay.check_run(result, True, expected_functions={'split'})

    def with_helpers(self, names):
        result = self.checked()
        data = json.loads(result['stdout'])
        original = data['verus-infer']['functions'][0]
        data['verus-infer']['functions'] = [dict(original, function=name) for name in names]
        result['stdout'] = json.dumps(data)
        return result

    def test_empty_omitted_duplicate_and_wrong_helpers_rejected(self):
        expected = {'data_index_of', 'select_mask'}
        for names in ([], ['data_index_of'], ['data_index_of', 'data_index_of'], ['data_index_of', 'wrong']):
            with self.subTest(names=names), self.assertRaisesRegex(ValueError, 'coverage mismatch'):
                replay.check_run(self.with_helpers(names), True, expected_functions=expected)

    def semantic_negative(self, *, failed_helper=False, mixed=False):
        result = self.checked()
        data = json.loads(result['stdout'])
        data['verification-results']['success'] = False
        data['verus-infer']['verification_scope']['complete_selected_scope_verified'] = False
        if failed_helper:
            function = data['verus-infer']['functions'][0]
            function['required_checks_passed'] = False
            for proof in function['proofs'].values():
                proof['status'] = 'failed'
        result.update(returncode=1, stdout=json.dumps(data), stderr='postcondition not satisfied')
        if mixed:
            result['stderr'] += '\nprecondition not satisfied'
        return result

    def test_semantic_negative_rejects_failed_helper_and_mixed_failures(self):
        with self.assertRaisesRegex(ValueError, 'mandatory exact inference checks'):
            replay.check_run(self.semantic_negative(failed_helper=True), False,
                'postcondition not satisfied', expected_functions={'split'})
        with self.assertRaisesRegex(ValueError, 'mixed failure kinds'):
            replay.check_run(self.semantic_negative(mixed=True), False,
                'postcondition not satisfied', expected_functions={'split'})

    def test_ordinary_negative_retains_healthy_helpers(self):
        replay.check_run(self.semantic_negative(), False, 'postcondition not satisfied',
                         expected_functions={'split'})

    def test_cache_poison_rejection_has_an_explicit_separate_expectation(self):
        replay.check_run(self.semantic_negative(failed_helper=True), False,
            'postcondition not satisfied', expected_functions={'split'}, helper_checks='rejected')
        with self.assertRaisesRegex(ValueError, 'poisoned pair did not fail'):
            replay.check_run(self.semantic_negative(), False, 'postcondition not satisfied',
                expected_functions={'split'}, helper_checks='rejected')

    def test_unknown_scenario_cannot_default_to_zero_helpers(self):
        with self.assertRaisesRegex(ValueError, 'unknown pair replay case'):
            replay.expected_helpers('missing_case')


if __name__ == '__main__':
    unittest.main()
