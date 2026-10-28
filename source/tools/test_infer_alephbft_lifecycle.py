"""Lifecycle source preservation and negative-control evidence requirements."""
import copy
import json
import unittest

from eval_verus_infer_alephbft_lifecycle import remove_loop_specs, direct_core, lifecycle_api, mutate, validate_stage
from eval_verus_infer_alephbft_parents import item
import test_infer_alephbft_private as private_tests
from test_infer_alephbft_wrappers import NODE
from test_infer_alephbft_traits import UNITS


class LifecycleExperimentTests(unittest.TestCase):
    def test_nested_loop_annotations_and_local_proofs_are_removed_only(self):
        method='''pub fn new(n:usize)->Self {
        let mut candidates=Vec::with_capacity(n);
        #[cfg_attr(verus_keep_ghost, verus_spec(
            invariant candidates@.len()<=n, forall|i:int|0<=i<candidates@.len() ==> candidates@[i].is_none(),
            decreases n-candidates@.len(),
        ))]
        while candidates.len()<n {candidates.push(None);}
        #[cfg(verus_keep_ghost)] proof! {assert(candidates@.len()==n);}
        Self{candidates}
        }'''
        cleaned=remove_loop_specs(method)
        self.assertIn('while candidates.len()<n {candidates.push(None);}',cleaned)
        self.assertIn('let mut candidates=Vec::with_capacity(n);',cleaned)
        self.assertNotIn('invariant',cleaned)
        self.assertNotIn('proof!',cleaned)
        self.assertEqual(cleaned,remove_loop_specs(cleaned))

    def test_original_factories_keep_their_executable_bodies_and_separate_oracles(self):
        original=private_tests.PrivateExperimentTests().source()
        # The small fixture next proof block is not cfg-gated; use the supplied
        # collector's real annotation convention for this source-preservation test.
        original=original.replace('        proof! {\n            assert(candidates@', '        #[cfg(verus_keep_ghost)]\n        proof! {\n            assert(candidates@')
        generated=direct_core(original)
        for name in ('new','next'):
            expected=remove_loop_specs(item(original,f'pub fn {name}(')[2])
            actual=item(generated,f'pub fn {name}(')[2]
            self.assertEqual(''.join(expected[expected.index('{'):].split()),''.join(actual[actual.index('{'):].split()))
        self.assertIn('pub fn check_new(',generated)
        self.assertIn('pub fn check_next(',generated)
        self.assertIn('pub fn check_lifecycle(',generated)

    def test_private_constructor_and_native_derive_replace_the_copy_adapter(self):
        node=NODE.replace('\n}', '''
 pub fn with_size(len:NodeCount)->Self where T:Clone {let v=vec![None;len.into()];NodeMap(v)}
}''',1)
        generated=lifecycle_api(node,'// supplied threshold',UNITS)
        self.assertEqual(item(node,'pub fn with_size(')[2],item(generated,'pub fn with_size(')[2])
        self.assertNotIn('copy_contents',generated)
        self.assertIn('#[derive(Clone)]',generated)
        self.assertIn('verus_verify(clone_via_view)',generated)
        self.assertIn('type Hash: Copy + vstd::std_specs::clone::CloneIdentity',generated)
        self.assertNotIn('external_body',generated)

    def test_mutations_are_unique_and_scoped(self):
        source='pub fn new()->u8{1}\npub fn next()->u8{1}'
        changed=mutate(source,'new','{1}','{2}')
        self.assertEqual(item(source,'pub fn next(')[2],item(changed,'pub fn next(')[2])
        with self.assertRaises(ValueError):mutate(source,'new','missing','x')

    def report(self):
        f=dict(mode='exact',summary_kind='owned_value',required_checks_passed=True,
            proofs={p:dict(status='proved',required=True) for p in ('soundness','maximality','exactness')})
        return {'verification-results':dict(success=False),'verus-infer':dict(functions=[f,copy.deepcopy(f)],failed_functions=['test::check_new'])}

    def run_report(self,data,stderr='postcondition not satisfied'):
        return dict(returncode=1,stdout=json.dumps(data),stderr=stderr)

    def test_semantic_failures_require_complete_exact_candidates_and_named_goal(self):
        validate_stage(self.run_report(self.report()),False,2,'check_new')
        for p in ('soundness','maximality','exactness'):
            bad=self.report();bad['verus-infer']['functions'][0]['proofs'][p]['status']='not_established'
            with self.assertRaises(ValueError):validate_stage(self.run_report(bad),False,2,'check_new')
        for mutation in ('kind','goal','frontend'):
            bad=self.report()
            if mutation=='kind':bad['verus-infer']['functions'][0]['summary_kind']='value'
            if mutation=='goal':bad['verus-infer']['failed_functions']=['test::unrelated']
            if mutation=='frontend':bad['verification-results']['encountered-vir-error']=True
            with self.assertRaises(ValueError):validate_stage(self.run_report(bad),False,2,'check_new')
        for stderr in ('panicked at checker','resource limit','timed out','type mismatch'):
            with self.assertRaises(ValueError):validate_stage(self.run_report(self.report(),stderr),False,2,'check_new')

if __name__=='__main__':unittest.main()
