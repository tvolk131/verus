"""Borrowed Result source scope and complete-certificate rejection controls."""
import copy
import json
import unittest
from eval_verus_infer_alephbft_selection import selection_core, validate_selection, without_index_guard
from eval_verus_infer_alephbft_parents import item
import test_infer_alephbft_private as private_tests

class SelectionExperimentTests(unittest.TestCase):
    def source(self):
        source=private_tests.PrivateExperimentTests().source()
        start,end,_=item(source,'pub fn prospective_parents(')
        method='''pub fn prospective_parents(&self, creator: usize) -> Result<&ParentSlots<T>, SelectionError> {
        if self.direct_parents < consensus_threshold(self.candidates.len()) {
            return Err(SelectionError::NotEnoughParents);
        }
        if creator >= self.candidates.len() {return Err(SelectionError::MissingOwnParent);}
        match self.candidates[creator] {
            Some((_, r)) if u32::from(r) + 1 == self.for_round => Ok(&self.candidates),
            _ => Err(SelectionError::MissingOwnParent),
        }
    }'''
        # The mock core's next proof block follows the actual cfg convention.
        attr=source.rindex('    #[cfg_attr(verus_keep_ghost, verus_spec(',0,start)
        contract='    #[cfg_attr(verus_keep_ghost, verus_spec(result =>\n        ensures true,\n    ))]\n    '
        source=source[:attr]+contract+method+source[end:]
        return source.replace('        proof! {\n            assert(candidates@','        #[cfg(verus_keep_ghost)]\n        proof! {\n            assert(candidates@')

    def test_direct_query_keeps_branch_order_and_explicitly_adapts_slice_return(self):
        original=self.source();generated=selection_core(original)
        old=item(original,'pub fn prospective_parents(')[2];new=item(generated,'pub fn prospective_parents(')[2]
        self.assertEqual(old[old.index('{'):],new[new.index('{'):])
        self.assertIn('Result<&Vec<Option<(T, u16)>>',new)
        self.assertIn('pub fn check_selection(',generated)
        self.assertIn('result matches Err(SelectionError::NotEnoughParents)',generated)
        self.assertIn('result matches Err(SelectionError::MissingOwnParent)',generated)
        self.assertEqual(generated.count('pub fn prospective_parents('),1)

    def test_unguarded_index_control_preserves_other_branches(self):
        source=selection_core(self.source());changed=without_index_guard(source)
        method=item(changed,'pub fn prospective_parents(')[2]
        self.assertNotIn('if creator >=',method)
        self.assertIn('return Err(SelectionError::NotEnoughParents)',method)
        self.assertIn('Some((_, r)) if',method)
        with self.assertRaises(ValueError):without_index_guard(changed)

    def report(self):
        f=dict(mode='exact',summary_kind='owned_value',required_checks_passed=True,
            proofs={p:dict(status='proved',required=True) for p in ('soundness','maximality','exactness')})
        return {'verification-results':dict(success=False),'verus-infer':dict(functions=[f,copy.deepcopy(f),copy.deepcopy(f)],failed_functions=['test::check_selection'])}
    def run_report(self,data,stderr='postcondition not satisfied'):
        return dict(stdout=json.dumps(data),stderr=stderr,returncode=1)
    def test_semantic_controls_require_three_exact_structured_summaries(self):
        validate_selection(self.run_report(self.report()),False,3,True)
        for proof in ('soundness','maximality','exactness'):
            data=self.report();data['verus-infer']['functions'][2]['proofs'][proof]['status']='not_established'
            with self.assertRaises(ValueError):validate_selection(self.run_report(data),False,3,True)
        for mutation in ('kind','goal','frontend'):
            data=self.report()
            if mutation=='kind':data['verus-infer']['functions'][2]['summary_kind']='value'
            if mutation=='goal':data['verus-infer']['failed_functions']=['test::unrelated']
            if mutation=='frontend':data['verification-results']['encountered-vir-error']=True
            with self.assertRaises(ValueError):validate_selection(self.run_report(data),False,3,True)
    def test_resource_and_frontend_errors_cannot_count_as_semantic_evidence(self):
        for stderr in ('panicked at checker','resource limit','timed out','type mismatch'):
            with self.assertRaises(ValueError):validate_selection(self.run_report(self.report(),stderr),False,3,True)

if __name__=='__main__':unittest.main()
