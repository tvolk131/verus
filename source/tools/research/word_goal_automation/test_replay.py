#!/usr/bin/env python3
"""Small replay guard tests. Solver outcomes belong to the full replay."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('automation_replay', HERE/'replay.py')
replay = importlib.util.module_from_spec(spec)
spec.loader.exec_module(replay)

class Guards(unittest.TestCase):
    def test_frozen_fixture_and_semantic_fidelity(self):
        replay.fidelity()

    def test_changed_state_update_rejected(self):
        import sys
        sys.path.insert(0, str(replay.HISTORY))
        from replay import fidelity
        with tempfile.TemporaryDirectory() as d:
            path = Path(d)/'changed.rs'
            path.write_text((HERE/'automatic.rs').read_text().replace('group |=', 'group ^='))
            with self.assertRaisesRegex(ValueError, 'original executable body lost'):
                fidelity(path)

    def test_changed_frozen_fixture_rejected(self):
        with patch.object(replay, 'sha', return_value='changed'):
            with self.assertRaisesRegex(ValueError, 'frozen fixture changed'):
                replay.fidelity()

    def checked_run(self):
        return dict(returncode=0, stderr='', stdout=json.dumps({
            'verification-results': {'success': True, 'is-verifying-entire-crate': True},
            'verus-infer': {
                'verification_scope': {'complete_selected_scope_verified': True},
                'functions': [{'mode': 'exact', 'required_checks_passed': True,
                    'proofs': {p: {'required': True, 'status': 'proved'}
                               for p in ('soundness', 'maximality', 'exactness')}}]}}))

    def test_missing_mandatory_proof_rejected(self):
        result = self.checked_run()
        data = json.loads(result['stdout'])
        del data['verus-infer']['functions'][0]['proofs']['exactness']
        result['stdout'] = json.dumps(data)
        with self.assertRaisesRegex(ValueError, 'mandatory exact inference checks'):
            replay.check_run(result, True)

    def test_incomplete_scope_rejected(self):
        result = self.checked_run()
        data = json.loads(result['stdout'])
        data['verus-infer']['verification_scope']['complete_selected_scope_verified'] = False
        result['stdout'] = json.dumps(data)
        with self.assertRaisesRegex(ValueError, 'incomplete verification scope'):
            replay.check_run(result, True)

    def test_failed_proof_cannot_count_as_success(self):
        result = self.checked_run()
        result['returncode'] = 1
        with self.assertRaisesRegex(ValueError, 'unexpected verification outcome'):
            replay.check_run(result, True)

    def test_expected_negative_requires_its_diagnostic(self):
        result = self.checked_run()
        data = json.loads(result['stdout'])
        data['verification-results']['success'] = False
        result.update(returncode=1, stdout=json.dumps(data))
        with self.assertRaisesRegex(ValueError, 'expected diagnostic missing'):
            replay.check_run(result, False, 'postcondition not satisfied')

    def test_checked_run_accepted(self):
        replay.check_run(self.checked_run(), True)

if __name__ == '__main__':
    unittest.main()
