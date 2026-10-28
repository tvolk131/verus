"""Keep native error payloads, assertion, and propagation source-connected."""
import unittest

from eval_verus_infer_alephbft_validation import validation_api, infer_member_count, native_models
from eval_verus_infer_alephbft_parents import item


SOURCE = '''pub enum Error<H: Hasher> { Missing(NodeIndex), Wrong(Round), Hash(H::Hash) }
impl<H: Hasher> ControlHash<H> {
fn unit_creator_is_descendant_of_previous_unit(&self, unit_coord: UnitCoord) -> Result<(), Error<H>> {
    match self.parents.get(unit_coord.creator) {
        None => Err(Error::Missing(unit_coord.creator)),
        Some(&round) => Err(Error::Wrong(round)),
    }
}
fn previous_round_have_enough_parents(&self, round: Round) -> Result<(), Error<H>> {
    let count = self
            .parents()
            .filter(|&parent| parent.round == round - 1)
            .count();
    Ok(())
}
fn check_if_parents_greater_than_previous_round(&self, round: Round) -> Result<(), Error<H>> {
    let future = self
            .parents()
            .any(|unit_coord| unit_coord.round > round - 1);
    Ok(())
}
fn validate_non_initial_round(&self, unit_coord: UnitCoord) -> Result<(), Error<H>> {
    assert!(unit_coord.round > 0, "Round must be greater than 0");
    self.unit_creator_is_descendant_of_previous_unit(unit_coord)?;
    self.previous_round_have_enough_parents(unit_coord.round)?;
    self.check_if_parents_greater_than_previous_round(unit_coord.round)?;
    Ok(())
}
pub fn n_members(&self) -> NodeCount { self.parents.size() }
}'''


class ValidationSourceTests(unittest.TestCase):
    def test_native_error_lookup_and_orchestration_are_preserved(self):
        adapted = validation_api(SOURCE)
        self.assertIn(item(SOURCE, 'pub enum Error')[2], adapted)
        for name in ('unit_creator_is_descendant_of_previous_unit', 'validate_non_initial_round'):
            self.assertEqual(item(SOURCE, 'fn '+name+'(')[2], item(adapted, 'fn '+name+'(')[2])
        self.assertIn('rounds_in_range_checked(round - 1, round - 1)', adapted)
        self.assertIn('contains_after_checked(round - 1)', adapted)
        api = 'pub fn n_members(&self)->(result:NodeCount) ensures result.0==self@.len(), {NodeCount(0)}'
        self.assertIn(item(SOURCE, 'pub fn n_members(')[2], infer_member_count(api, SOURCE))

    def test_native_mode_preserves_every_original_validation_body(self):
        native = validation_api(SOURCE, native_pipelines=True)
        for name in ('unit_creator_is_descendant_of_previous_unit',
                     'previous_round_have_enough_parents',
                     'check_if_parents_greater_than_previous_round', 'validate_non_initial_round'):
            self.assertEqual(item(SOURCE, 'fn '+name+'(')[2], item(native, 'fn '+name+'(')[2])
        node = 'pub fn iter(&self) { self.0.iter().enumerate().filter_map(|(idx, maybe_value)| Some((NodeIndex(idx), maybe_value.as_ref()?))) }'
        control = 'pub fn parents(&self) { self.parents.iter().map(|(node_index, &round)| UnitCoord::new(round, node_index)) }'
        self.assertIn('finite_native_view', native_models(node, control))
        with self.assertRaises(ValueError):
            native_models(node.replace('NodeIndex(idx)', 'NodeIndex(0)'), control)
        with self.assertRaises(ValueError):
            native_models(node, control.replace('new(round,', 'new(0,'))

    def test_changed_native_pipeline_is_not_silently_adapted(self):
        for old, new in [('parent.round == round - 1', 'parent.round < round - 1'),
                         ('unit_coord.round > round - 1', 'unit_coord.round >= round - 1')]:
            with self.assertRaises(ValueError):
                validation_api(SOURCE.replace(old, new))


if __name__ == '__main__':
    unittest.main()
