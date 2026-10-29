"""Keep native error payloads, assertion, and propagation source-connected."""
import unittest

from eval_verus_infer_alephbft_validation import validation_api, infer_member_count, native_models, construction_api, native_construction_runtime, full_validation_api, parent_validation_api, outer_validation_api, FIXTURES, RUNTIME
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
    def test_outer_body_is_preserved_with_explicit_policy_and_authentication(self):
        validator = '''impl<H: Hasher, D: Data, S: Signature> From<SignatureError<FullUnit<H, D>, S>> for ValidationError<H, D, S> {
    fn from(se: SignatureError<FullUnit<H, D>, S>) -> Self { ValidationError::WrongSignature(se.unchecked) }
}
impl<K: Keychain> Validator<K> {
 pub fn new(session_id:SessionId,keychain:K,max_round:Round)->Self{Self{session_id,keychain,max_round}}
 pub fn validate_unit<H: Hasher, D: Data>(&self,uu:UncheckedSignedUnit<H,D,K::Signature>)-> Result<H, D, K> {
  let su=uu.check(&self.keychain)?;
  if su.as_signable().session_id()!=self.session_id{return Err(ValidationError::WrongSession(su.as_signable().clone()));}
  self.validate_unit_parents(su)
 }
}'''
        signature = 'pub struct SignatureError<T:Signable,S:Signature>{pub unchecked:UncheckedSigned<T,S>,}'
        generated = outer_validation_api((FIXTURES/'parent_validation_models.rs').read_text(),validator,signature)
        original = item(validator, 'pub fn validate_unit<')[2]
        outer = item(generated, 'pub fn validate_unit<')[2]
        self.assertEqual(original.split('{',1)[1],outer.split('{',1)[1])
        self.assertIn('requires self.outer_domain(uu)',outer)
        self.assertIn('ensures self.outer_policy(uu,result)',outer)
        self.assertNotIn('proof {',generated)
        self.assertIn('value==su',generated)
        self.assertIn('original==uu',generated)
        self.assertIn('authenticated::<H,D>',generated)
        self.assertEqual(generated.count('#[verifier::infer]'),4)
        with self.assertRaises(ValueError):
            outer_validation_api((FIXTURES/'parent_validation_models.rs').read_text(),validator,signature.replace('pub unchecked:', 'pub extra:u8,pub unchecked:'))

    def test_full_validation_preserves_both_original_bodies(self):
        original = SOURCE.replace('pub fn n_members', '''
        fn validate_initial_round(&self) -> Result<(), Error<H>> {
            let count = self.parents().count();
            if count > 0 { return Err(Error::RoundZeroWithSomeParents(NodeCount(count))); }
            Ok(())
        }
        pub fn validate(&self, coord: UnitCoord) -> Result<(), Error<H>> {
            match coord.round { 0 => self.validate_initial_round(), _ => self.validate_non_initial_round(coord) }
        }
        pub fn n_members''')
        generated = full_validation_api(original)
        self.assertEqual(generated.count('#[verifier::infer]'), 2)
        self.assertEqual(item(original, 'fn validate_initial_round(')[2], item(generated, 'fn validate_initial_round(')[2])
        self.assertEqual(item(original, 'pub fn validate(')[2],
            item(generated, 'pub fn validate(')[2])
        self.assertIn('coord.parts().0.0 < parents@.len()', generated)
        self.assertNotIn('construction_rounds(parents).len()', generated)

    def test_parent_validator_and_accessors_are_source_connected(self):
        units = """impl UnitCoord {
         pub fn new(round:Round,creator:NodeIndex)->Self{Self{round,creator}}
         pub fn creator(&self)->NodeIndex{self.creator}
         pub fn round(&self)->Round{self.round}
        }
        impl<H: Hasher> PreUnit<H> {
         pub(crate) fn new(creator:NodeIndex,round:Round,control_hash:ControlHash<H>)->Self{Self{coord:UnitCoord::new(round,creator),control_hash}}
         pub(crate) fn n_members(&self)->NodeCount{self.control_hash.n_members()}
         pub(crate) fn creator(&self)->NodeIndex{self.coord.creator()}
         pub(crate) fn round(&self)->Round{self.coord.round()}
         pub(crate) fn control_hash(&self)->&ControlHash<H>{&self.control_hash}
        }
        impl<H: Hasher, D: Data> FullUnit<H, D> {
         pub(crate) fn as_pre_unit(&self)->&PreUnit<H>{&self.pre_unit}
        }"""
        signature = """pub struct UncheckedSigned<T:Signable,S:Signature>{signable:T,signature:S,}
        pub struct Signed<T:Signable+Index,K:Keychain>{unchecked:UncheckedSigned<T,K::Signature>,}
        impl<T: Signable + Index, K: Keychain> Signed<T, K> {
         pub fn as_signable(&self)->&T{&self.unchecked.signable}
        }"""
        validator = """pub enum ValidationError<H:Hasher,D:Data,S:Signature>{WrongNumberOfMembers(PreUnit<H>),Other(S,D)}
        pub struct Validator<K:Keychain>{keychain:K}
        impl<K: Keychain> Validator<K> {
         fn validate_unit_parents<H:Hasher,D:Data>(&self,su:SignedUnit<H,D,K>)->Result<H,D,K>{
          let pre_unit=su.as_signable().as_pre_unit();
          pre_unit.control_hash.validate(UnitCoord::new(pre_unit.round(),pre_unit.creator()))
           .map_err(|e|ValidationError::ParentValidationFailed(pre_unit.clone(),e))?;
          Ok(su)
         }
        }"""
        api = units.replace('D: Data', 'D: CloneView')
        generated = parent_validation_api(api, units, validator, signature)
        self.assertEqual(generated.count('#[verifier::infer]'), 11)
        for marker in ('fn validate_unit_parents<', 'pub fn as_signable(', 'pub(crate) fn as_pre_unit('):
            original = validator if 'validate_unit' in marker else signature if 'signable' in marker else units
            self.assertEqual(item(generated, marker)[2], item(original, marker)[2])
        self.assertIn('value==su', generated)
        self.assertIn('copy@==pre@', generated)
        self.assertIn('|| su.pre_value().0.0<su.pre_value().2.0.len()', generated)
        with self.assertRaises(ValueError):
            parent_validation_api(api, units, validator, signature.replace('signature:S,', 'signature:S,extra:u8,'))

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

    def test_native_constructor_and_hash_bodies_are_preserved(self):
        original = SOURCE.replace('pub fn n_members', '''pub fn new(parents_with_rounds_and_hashes: &NodeMap<(H::Hash, Round)>) -> Self {
            let mut parents_with_rounds = NodeMap::with_size(parents_with_rounds_and_hashes.size());
            for (parent_index, (_, parent_round)) in parents_with_rounds_and_hashes.iter() {
                parents_with_rounds.insert(parent_index, *parent_round);
            }
            ControlHash { parents: parents_with_rounds, combined_hash: Self::create_control_hash(parents_with_rounds_and_hashes) }
        }
        pub fn create_control_hash(parent_map: &NodeMap<(H::Hash, Round)>) -> H::Hash { parent_map.using_encoded(H::hash) }
        pub fn n_members''')
        generated = construction_api(original)
        self.assertIn(item(original, 'pub fn new(')[2], generated)
        self.assertEqual(item(original, 'pub fn create_control_hash(')[2].split('{', 1)[1],
            item(generated, 'pub fn create_control_hash(')[2].split('{', 1)[1])
        native = native_construction_runtime(RUNTIME.read_text())
        self.assertIn('ControlHash::<Hasher64>::new(&hashes)', native)
        self.assertIn('NATIVE_CONSTRUCTORS=378;EMPTY=1', native)
        with self.assertRaises(ValueError):
            native_construction_runtime('fn changed_corpus() {}')

    def test_factory_proof_mode_keeps_original_chains_without_handwritten_proofs(self):
        node = 'pub fn iter(&self) { self.0.iter().enumerate().filter_map(|(idx, maybe_value)| Some((NodeIndex(idx), maybe_value.as_ref()?))) }'
        control = 'pub fn parents(&self) { self.parents.iter().map(|(node_index, &round)| UnitCoord::new(round, node_index)) }'
        generated = native_models(node, control, factory_proofs=True)
        self.assertEqual(item(generated, 'pub fn iter(')[2], node)
        self.assertEqual(item(generated, 'pub fn parents(')[2], control)
        self.assertEqual(generated.count('iterator_via_view('), 2)
        self.assertNotIn('proof {', generated)
        self.assertNotIn('filter_map_postcondition', generated)
        self.assertNotIn('-> (result: impl Iterator', generated)
        with self.assertRaises(ValueError):
            native_models(node.replace('NodeIndex(idx)', 'NodeIndex(0)'), control, True)

    def test_changed_native_pipeline_is_not_silently_adapted(self):
        for old, new in [('parent.round == round - 1', 'parent.round < round - 1'),
                         ('unit_coord.round > round - 1', 'unit_coord.round >= round - 1')]:
            with self.assertRaises(ValueError):
                validation_api(SOURCE.replace(old, new))


if __name__ == '__main__':
    unittest.main()
