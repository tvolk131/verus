"""Keep wrapper source adaptations and checked outcomes explicit."""
import json
import unittest

from eval_verus_infer_alephbft_parents import item
from eval_verus_infer_alephbft_wrappers import api_mutation, node_map_source, validate, wrap_collector
import test_infer_alephbft_records as record_tests

NODE = '''
pub struct NodeMap<T>(Vec<Option<T>>);
impl<T> NodeMap<T> {
    pub fn size(&self) -> NodeCount { self.0.len().into() }
    pub fn get(&self, node_id: NodeIndex) -> Option<&T> { self.0[node_id.0].as_ref() }
    pub fn insert(&mut self, node_id: NodeIndex, value: T) { self.0[node_id.0] = Some(value) }
}
'''


class WrapperExperimentTests(unittest.TestCase):
    def test_original_lookup_and_insert_bodies_are_preserved(self):
        source = node_map_source(NODE)
        for name in ('get', 'insert'):
            self.assertEqual(item(source, f'pub fn {name}(')[2], item(NODE, f'pub fn {name}(')[2])
        self.assertIn('NodeCount(self.0.len())', source)
        self.assertEqual(source.count('verus_infer'), 3)
        self.assertNotIn('T: Copy', source)
        self.assertIn('self.0.len().into()', node_map_source(NODE, constructor=False))

    def test_api_mutations_are_scoped_and_require_unique_targets(self):
        source = node_map_source(NODE)
        mutated = api_mutation(source, 'get', 'self.0[node_id.0]', 'self.0[0]')
        self.assertEqual(item(source, 'pub fn insert(')[2], item(mutated, 'pub fn insert(')[2])
        with self.assertRaises(ValueError): api_mutation(source, 'get', 'missing', 'replacement')

    def test_collector_keeps_supplied_invariants_and_the_separate_behavior_oracle(self):
        source = record_tests.RecordExperimentTests().source().replace('keep_update();', '''
            if creator >= self.candidates.len() {return;}
            let replace = match self.candidates[creator] {None=>true,Some((_, previous_round))=>previous_round < round};
            if replace {self.candidates[creator] = Some((payload, round));}
        ''')
        wrapped = wrap_collector(source)
        method = item(wrapped, 'pub fn add(&mut self')[2]
        self.assertIn('self.candidates.size().0', method)
        self.assertIn('self.candidates.get(NodeIndex(creator))', method)
        self.assertIn('self.candidates.insert(NodeIndex(creator), (payload, round))', method)
        self.assertIn('*previous_round < round', method)
        self.assertIn('check_add_invariant', wrapped)
        self.assertIn('check_add_behavior', wrapped)
        self.assertIn('library_count_agrees', wrapped)

    def report(self):
        functions = []
        for kind in ('value', 'value', 'record_view', 'record_view', 'record_state', 'record_state'):
            functions.append(dict(mode='exact', summary_kind=kind, required_checks_passed=True,
                proofs={p: dict(status='proved', required=True) for p in ('soundness', 'maximality', 'exactness')},
                metrics=dict(cache='miss', cache_policy='regenerated_state' if kind != 'value' else 'candidate_cache')))
        return {'verification-results': dict(success=False), 'verus-infer': dict(functions=functions)}

    def run_report(self, data, stderr=''):
        return dict(stdout=json.dumps(data), stderr=stderr, returncode=1)

    def test_semantic_controls_require_all_six_exact_candidates(self):
        validate(self.run_report(self.report()), False)
        for i in range(6):
            for proof in ('soundness', 'maximality', 'exactness'):
                data = self.report(); data['verus-infer']['functions'][i]['proofs'][proof]['status'] = 'not_established'
                with self.assertRaises(ValueError): validate(self.run_report(data), False)
        data = self.report(); data['verus-infer']['functions'][2]['mode'] = 'sound'
        with self.assertRaises(ValueError): validate(self.run_report(data), False)

    def test_frame_query_counts_and_regeneration_are_required(self):
        for change in ('drop', 'cache', 'kind'):
            data = self.report()
            if change == 'drop': data['verus-infer']['functions'].pop()
            elif change == 'cache': data['verus-infer']['functions'][3]['metrics']['cache'] = 'hit'
            else: data['verus-infer']['functions'][3]['summary_kind'] = 'value'
            with self.assertRaises(ValueError): validate(self.run_report(data), False)

    def test_frontend_and_resource_failures_are_not_semantic_rejections(self):
        data = self.report(); data['verification-results']['encountered-vir-error'] = True
        with self.assertRaises(ValueError): validate(self.run_report(data), False)
        for stderr in ('panicked at checker', 'resource limit exceeded', 'timed out'):
            with self.assertRaises(ValueError): validate(self.run_report(self.report(), stderr), False)


if __name__ == '__main__': unittest.main()
