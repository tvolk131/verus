"""Adversarial body-coverage checks; no verifier or application proof is mocked."""
import copy
import unittest
from pathlib import Path
from replay_s1 import require_native_scope

SOURCE=Path('/tmp/scope.rs')


def sample(failed=False):
    bodies=[dict(function='crate::'+name,rust_function=name,source=str(SOURCE)+':1:1',
                 selected=True,body_status=('selected_body_not_certified_by_successful_full_run'
                 if failed else 'checked_exec_body')) for name in ['A::set','B::set']]
    return {'verus-infer':{
        'verification_scope':dict(native_collection_completed=True,
            complete_selected_scope_verified=not failed,checked_exec_bodies=0 if failed else 2,
            checked_proof_bodies=0,native_functions=bodies),
        'failed_functions':['crate::A::set'] if failed else [],
        'set_pair_assistance':[]},
        'times-ms':{'smt':{'smt-run-module-times':[{'function-breakdown':[
            dict(function='crate::A::set',success=not failed),
            dict(function='crate::B::set',success=True)]}]}}}


class ScopeChecks(unittest.TestCase):
    def check(self,data,failed=False):
        require_native_scope(data,SOURCE,['A::set','B::set'],[],
                             ['A::set'] if failed else [],{})

    def test_duplicate_method_basenames_are_distinct(self):
        self.check(sample())
        self.check(sample(True),True)

    def test_incomplete_extra_duplicate_or_wrong_mode_refused(self):
        original=sample()
        for change in ['empty','missing','extra','duplicate','mode','unselected','source','count']:
            with self.subTest(change=change):
                data=copy.deepcopy(original)
                scope=data['verus-infer']['verification_scope'];b=scope['native_functions']
                if change=='empty':b.clear()
                elif change=='missing':b.pop()
                elif change=='extra':b.append(dict(b[0],rust_function='C::set',function='crate::C::set'))
                elif change=='duplicate':b[1]=copy.deepcopy(b[0])
                elif change=='mode':b[1]['body_status']='checked_proof_body'
                elif change=='unselected':b[1]['selected']=False
                elif change=='source':b[1]['source']='/tmp/other.rs:1:1'
                elif change=='count':scope['checked_exec_bodies']=0
                with self.assertRaises(ValueError):self.check(data)

    def test_same_basename_dependency_failure_is_not_intended_failure(self):
        data=sample(True)
        data['verus-infer']['failed_functions']=['crate::B::set']
        with self.assertRaises(ValueError):self.check(data,True)
        data=sample(True)
        data['times-ms']['smt']['smt-run-module-times'][0]['function-breakdown'][1]['success']=False
        with self.assertRaises(ValueError):self.check(data,True)

    def test_negative_scope_cannot_certify_bodies(self):
        data=sample(True)
        data['verus-infer']['verification_scope']['native_functions'][0]['body_status']='checked_exec_body'
        with self.assertRaises(ValueError):self.check(data,True)

    def test_pair_scope_duplicates_or_unknown_function_refused(self):
        for names in [['crate::A::set','crate::A::set'],['crate::missing']]:
            data=sample()
            data['verus-infer']['set_pair_assistance']=[dict(function=n,pairs=2) for n in names]
            with self.assertRaises(ValueError):self.check(data)


if __name__=='__main__':
    unittest.main()
