"""Preserve original size bodies and distinguish wrapper boundary controls."""
import json
import unittest

from eval_verus_infer_alephbft_traits import wrapper_api, wrapper_collector, operator_control, validate_stage
from eval_verus_infer_alephbft_private import private_api
from eval_verus_infer_alephbft_parents import item
from test_infer_alephbft_wrappers import NODE


class WrapperTraitExperimentTests(unittest.TestCase):
    def test_original_size_body_is_restored_with_verified_wrapper_adapters(self):
        source = wrapper_api(private_api(NODE, "// threshold"), NODE)
        for name in ("size", "get", "insert"):
            self.assertEqual(item(source, f"pub fn {name}(")[2], item(NODE, f"pub fn {name}(")[2])
        self.assertIn("impl From<usize> for NodeCount", source)
        self.assertIn("impl core::ops::AddAssign for NodeCount", source)
        self.assertIn("Explicit verified equivalents", source)
        with self.assertRaises(ValueError): wrapper_api(source.replace("self.0.len().into()", "0.into()"), NODE)

    def test_counter_adaptation_keeps_the_production_increment_and_scalar_observations(self):
        source = """use super::{NodeMap, NodeIndex, Hasher, Unit};
struct Collector {direct_parents: usize,}
impl Collector {
 spec fn count(&self)->usize {self.direct_parents}
 spec fn valid(&self)->bool {self.direct_parents <= 4}
 fn add(&mut self) {self.direct_parents += 1;}
}"""
        updated = wrapper_collector(source)
        self.assertIn("direct_parents: NodeCount", updated)
        self.assertIn("{self.direct_parents.0}", updated)
        self.assertIn("self.direct_parents += NodeCount(1);", updated)
        self.assertNotIn("self.direct_parents.0.0", updated)
        self.assertIn("NodeIndex, NodeCount, Hasher", updated)
        with self.assertRaises(ValueError): wrapper_collector(source.replace("+= 1", "+= 2"))

    def test_operator_controls_expose_the_intended_differences(self):
        source = wrapper_api(private_api(NODE, "// threshold"), NODE)
        for case, expected in (("restrictive-operator", "usize::MAX as int / 2"),
                               ("weak-operator", "->bool { true }"),
                               ("wrong-operator-model", "&NodeCount(self.0)"),
                               ("replacement-operator", "self.0 = rhs.0;")):
            updated = operator_control(source, case)
            self.assertIn(expected, updated)
            self.assertEqual(updated.count("impl core::ops::AddAssign for NodeCount"), 1)
            self.assertIn("impl From<usize> for NodeCount", updated)
            self.assertNotIn("external_body", updated)

    def test_six_provider_candidates_are_required(self):
        f = dict(mode="exact", required_checks_passed=True,
                 proofs={p: dict(required=True, status="proved") for p in ("soundness", "maximality", "exactness")})
        kinds = ("record_state", "record_state", "record_view", "record_view", "value", "value")
        data = {"verification-results": {"success": False}, "verus-infer": {"functions": [dict(f, summary_kind=k) for k in kinds]}}
        run = dict(returncode=1, stdout=json.dumps(data), stderr="")
        validate_stage(run, False, 6, "provider")
        data["verus-infer"]["functions"].pop()
        run["stdout"] = json.dumps(data)
        with self.assertRaises(ValueError): validate_stage(run, False, 6, "provider")


if __name__ == "__main__": unittest.main()
