"""Source identity, scope and evidence controls for the trait getter checkpoint."""
import json
import unittest

from eval_verus_infer_alephbft_parents import item
from eval_verus_infer_alephbft_traits import getter_adapter, generic_collector, check_domain, unsupported_getter, validate_stage
from eval_verus_infer_alephbft_private import update_only
import test_infer_alephbft_private as private_tests

UNITS = """
impl UnitCoord {
    pub fn creator(&self) -> NodeIndex { self.creator }
    pub fn round(&self) -> Round { self.round }
}
pub trait Unit: Clone {
    type Hasher: Hasher;
    fn hash(&self) -> <Self::Hasher as Hasher>::Hash;
    fn coord(&self) -> UnitCoord;
    fn creator(&self) -> NodeIndex { self.coord().creator() }
    fn round(&self) -> Round { self.coord().round() }
}
"""


class TraitExperimentTests(unittest.TestCase):
    def test_supplied_contracts_keep_original_default_and_coordinate_bodies(self):
        adapter = getter_adapter(UNITS)
        unit = item(UNITS, "pub trait Unit:")[2]
        adapted_unit = item(adapter, "pub trait Unit {")[2]
        coord = item(UNITS, "impl UnitCoord")[2]
        adapted_coord = item(adapter, "impl UnitCoord")[2]
        for name in ("creator", "round"):
            for old, new, marker in ((unit, adapted_unit, "fn "), (coord, adapted_coord, "pub fn ")):
                a = item(old, marker + name + "(")[2]; b = item(new, marker + name + "(")[2]
                self.assertEqual(a[a.index("{"):], b[b.index("{"):])
        self.assertIn("<Self::Hasher as Hasher>::Hash", adapter)
        self.assertIn("closed spec fn hash_value", adapter)
        self.assertIn("pub closed spec fn parts", adapter)
        self.assertNotIn("external_body", adapter)

    def test_changed_getter_layout_is_rejected(self):
        with self.assertRaises(ValueError): getter_adapter(UNITS.replace("-> NodeIndex {", "-> usize {"))

    def test_generic_adapter_retains_the_direct_update_and_explicit_oracles(self):
        source = ("pub struct Collector<T: Copy> {candidates: Vec<Option<(T, u16)>>,}\n"
                  "impl<T: Copy> Collector<T> {\n" + private_tests.PrivateExperimentTests().source())
        source = source.replace("payload:T", "payload: T").replace("ensures final(self).valid()",
            "ensures final(self).valid(), final(self).round()==old(self).round(), final(self).parents()[creator as int]==Some((payload,round))")
        old = item(update_only(source), "pub fn add(&mut self")[2]
        updated = generic_collector(source)
        new = item(updated, "pub fn add(&mut self")[2]
        self.assertEqual(old[old.index("{"):], new[new.index("{"):])
        self.assertIn("Collector<H: Hasher>", updated)
        self.assertIn("payload: H::Hash", updated)
        self.assertIn("add_unit<U: Unit<Hasher = H>>", updated)
        self.assertIn("check_unit_invariant", updated)
        self.assertIn("check_unit_behavior", updated)
        self.assertIn("unit.hash_value()", updated)
        self.assertIn("old(self).round()", updated)
        self.assertNotIn(".unit.round_value()", updated)

    def test_domain_rejection_must_name_the_new_unit_harness(self):
        data = {"verus-infer": {"failed_functions": ["test::check_add_invariant"]}}
        with self.assertRaises(ValueError): check_domain(data, "precondition not satisfied")
        data["verus-infer"]["failed_functions"] = ["test::check_unit_invariant"]
        check_domain(data, "precondition not satisfied")
        with self.assertRaises(ValueError): check_domain(data, "postcondition not satisfied")

    def test_unsupported_contracts_are_distinct_from_resource_or_semantic_failures(self):
        data = {"verification-results": {"success": False}, "verus-infer": {"status": "unsupported"}}
        run = dict(returncode=1, stdout=json.dumps(data), stderr="trait inference requires a certified precondition-free getter")
        unsupported_getter(run)
        for error in ("panicked at compiler", "resource limit", "timed out", "postcondition not satisfied"):
            bad = dict(run, stderr=error)
            with self.assertRaises(ValueError): unsupported_getter(bad)

    def test_provider_and_consumer_require_their_actual_exact_candidates(self):
        f = dict(mode="exact", required_checks_passed=True,
                 proofs={p: dict(required=True, status="proved") for p in ("soundness", "maximality", "exactness")})
        data = {"verification-results": {"success": False}, "verus-infer": {"functions": [dict(f,summary_kind=k) for k in ("record_state","record_state","value")]}}
        run = dict(returncode=1, stdout=json.dumps(data), stderr="")
        validate_stage(run, False, 3, "consumer")
        with self.assertRaises(ValueError): validate_stage(run, False, 3, "provider")
        data["verus-infer"]["functions"][0]["proofs"]["exactness"]["status"] = "not_established"
        run["stdout"] = json.dumps(data)
        with self.assertRaises(ValueError): validate_stage(run, False, 3, "consumer")


if __name__ == "__main__": unittest.main()
