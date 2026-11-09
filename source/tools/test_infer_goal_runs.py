"""Goal evidence must cover named bodies as well as inferred helper proofs."""
import json
from pathlib import Path
import tempfile
import unittest

from infer_experiment import validate_goal_run


class GoalRunTests(unittest.TestCase):
    def report(self, success=True, helper=False):
        functions = []
        names = ['encode']
        if helper:
            names.append('helper')
            functions.append(dict(function='crate::helper', mode='exact', required_checks_passed=True,
                proofs={p: dict(required=True, status='proved')
                        for p in ('soundness', 'maximality', 'exactness')}))
        bodies = [dict(function='crate::'+name, selected=True,
                       body_status='checked_exec_body' if success else
                       'selected_body_not_certified_by_successful_full_run') for name in names]
        return {
            'verification-results': dict(success=success, errors=0 if success else 1,
                                         **{'is-verifying-entire-crate': True}),
            'verus-infer': dict(crate_success=success, interface_written=success, functions=functions,
                verification_scope=dict(native_collection_completed=True,
                    complete_selected_scope_verified=success,
                    checked_exec_bodies=len(bodies) if success else 0,
                    checked_proof_bodies=0, native_functions=bodies)),
        }

    def check(self, data, expected=True, helpers=(), **options):
        run = dict(returncode=0 if expected else 1, stdout=json.dumps(data),
                   stderr='' if expected else 'postcondition not satisfied')
        run.update(options.pop('run_changes', {}))
        return validate_goal_run(run, expected, expected_functions=set(helpers),
            exec_bodies={'encode'} | set(helpers), proof_bodies=set(),
            failure_kind=None if expected else 'postcondition not satisfied', **options)

    def test_ordinary_zero_helper_goal_and_semantic_control(self):
        self.check(self.report())
        self.check(self.report(False), False)

    def test_inferred_helpers_are_checked_on_both_outcomes(self):
        for positive in (True, False):
            data = self.report(positive, helper=True)
            self.check(data, positive, helpers={'helper'})
            for bad in ([], [dict(data['verus-infer']['functions'][0], required_checks_passed=False)]):
                data['verus-infer']['functions'] = bad
                with self.subTest(positive=positive, bad=bad), self.assertRaises(ValueError):
                    self.check(data, positive, helpers={'helper'})

    def test_missing_duplicate_and_wrong_checked_bodies_are_rejected(self):
        for names in ([], ['encode', 'encode'], ['wrong']):
            data = self.report()
            data['verus-infer']['verification_scope']['native_functions'] = [
                dict(function=name, selected=True, body_status='checked_exec_body') for name in names]
            with self.subTest(names=names), self.assertRaisesRegex(ValueError, 'coverage mismatch'):
                self.check(data)

    def test_body_selection_mode_count_and_complete_scope_are_required(self):
        for kind in ('selection', 'mode', 'count', 'complete', 'collection'):
            data = self.report()
            scope = data['verus-infer']['verification_scope']
            if kind == 'selection': scope['native_functions'][0]['selected'] = False
            elif kind == 'mode': scope['native_functions'][0]['body_status'] = 'unverified_body_with_contract'
            elif kind == 'count': scope['checked_exec_bodies'] = 0
            elif kind == 'complete': scope['complete_selected_scope_verified'] = False
            else: scope['native_collection_completed'] = False
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                self.check(data)

    def test_declared_proof_bodies_must_be_checked_too(self):
        data = self.report()
        scope = data['verus-infer']['verification_scope']
        scope['native_functions'].append(dict(function='crate::lemma', selected=True,
                                              body_status='checked_proof_body'))
        scope['checked_proof_bodies'] = 1
        run = dict(returncode=0, stdout=json.dumps(data), stderr='')
        validate_goal_run(run, True, expected_functions=set(), exec_bodies={'encode'},
                          proof_bodies={'lemma'})
        with self.assertRaisesRegex(ValueError, 'coverage mismatch'):
            validate_goal_run(run, True, expected_functions=set(), exec_bodies={'encode'},
                              proof_bodies=set())

    def test_negative_must_select_the_expected_body(self):
        data = self.report(False)
        data['verus-infer']['verification_scope']['native_functions'][0]['function'] = 'other'
        with self.assertRaisesRegex(ValueError, 'coverage mismatch'):
            self.check(data, False)

    def test_extra_unverified_local_contract_is_not_body_coverage(self):
        for status in ('unverified_body_with_contract', 'external_specification', 'interface_without_checked_body'):
            data = self.report()
            data['verus-infer']['verification_scope']['native_functions'].append(
                dict(function='unchecked', selected=True, body_status=status))
            with self.subTest(status=status), self.assertRaisesRegex(ValueError, 'no checked body'):
                self.check(data)

    def test_frontend_resource_mixed_or_filtered_failure_is_not_a_semantic_control(self):
        for kind in ('frontend', 'resource', 'mixed', 'filtered', 'zero_errors', 'arithmetic', 'invariant'):
            data = self.report(False)
            changes = {}
            if kind == 'frontend': data['verification-results']['encountered-vir-error'] = True
            elif kind == 'resource': changes['stderr'] = 'postcondition not satisfied: resource limit'
            elif kind == 'mixed': changes['stderr'] = 'postcondition not satisfied\nprecondition not satisfied'
            elif kind == 'filtered': data['verification-results']['is-verifying-entire-crate'] = False
            elif kind == 'arithmetic': changes['stderr'] = 'postcondition not satisfied\npossible arithmetic underflow/overflow'
            elif kind == 'invariant': changes['stderr'] = 'postcondition not satisfied\ninvariant not satisfied'
            else: data['verification-results']['errors'] = 0
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                self.check(data, False, run_changes=changes)

    def test_publication_requires_the_real_export_and_removes_a_stale_negative(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory)/'goal.vir'
            with self.assertRaises(ValueError): self.check(self.report(), artifact=artifact)
            artifact.write_bytes(b'checked export')
            self.check(self.report(), artifact=artifact)
            with self.assertRaises(ValueError): self.check(self.report(False), False, artifact=artifact)
            artifact.unlink()
            self.check(self.report(False), False, artifact=artifact)

    def test_no_empty_or_ambiguous_inventory_and_no_unclassified_negative(self):
        data = self.report()
        run = dict(returncode=0, stdout=json.dumps(data), stderr='')
        for ex, pr in ((set(), set()), ({'encode'}, {'encode'})):
            with self.subTest(exec=ex, proof=pr), self.assertRaises(ValueError):
                validate_goal_run(run, True, expected_functions=set(), exec_bodies=ex, proof_bodies=pr)
        with self.assertRaises(ValueError):
            validate_goal_run(dict(run, returncode=1), False, expected_functions=set(),
                              exec_bodies={'encode'}, proof_bodies=set())


if __name__ == '__main__':
    unittest.main()
