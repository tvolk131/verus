"""Keep the collection experiment's source and evidence claims narrow."""
import copy
import json
import unittest

from eval_verus_infer_alephbft_collections import extract_operations, manual_operations, validate
from eval_verus_infer_alephbft_parents import FIXTURES


class CollectionExperimentTests(unittest.TestCase):
    def report(self):
        functions = []
        for name in ("quorum", "adapter", "collection_ops::length", "collection_ops::read", "collection_ops::replace"):
            state = name.endswith("::replace")
            functions.append(dict(function=name, mode="sound" if state else "exact", required_checks_passed=True,
                metrics=dict(cache="miss", cache_policy="regenerated_template"),
                proofs=dict(soundness=dict(status="proved", required=True),
                    **{p: dict(status="not_established" if state else "proved", required=not state,
                               reason="state_precision_not_supported" if state else None)
                       for p in ("maximality", "exactness")})))
        return {"verification-results": {"success": False}, "verus-infer": {"functions": functions}}

    def run_report(self, data, stderr=""):
        return dict(stdout=json.dumps(data), stderr=stderr, returncode=1)

    def test_state_precision_is_honest_and_every_helper_is_proved(self):
        good = self.report()
        validate(self.run_report(good), False, 3)
        for index, proof, status in ((4, "exactness", "proved"), (4, "soundness", "not_proved"),
                                     (2, "maximality", "not_proved")):
            bad = copy.deepcopy(good)
            bad["verus-infer"]["functions"][index]["proofs"][proof]["status"] = status
            with self.assertRaises(ValueError): validate(self.run_report(bad), False, 3)

    def test_frontend_and_resource_failures_are_not_semantic_controls(self):
        bad = self.report(); bad["verification-results"]["encountered-vir-error"] = True
        with self.assertRaises(ValueError): validate(self.run_report(bad), False, 3)
        for text in ("panicked at checker.rs", "resource limit exceeded", "solver timed out"):
            with self.assertRaises(ValueError): validate(self.run_report(self.report(), text), False, 3)

    def test_extraction_changes_only_the_three_add_operations(self):
        source = """before
pub fn add(&mut self, creator:usize, payload:T, round:u16) {
    if creator >= self.candidates.len() { return; }
    let replace = match self.candidates[creator] { None => true, _ => false };
    if replace {
        proof! { count_update(self.candidates@); }
        self.candidates[creator] = Some((payload, round));
        self.direct_parents += 1;
    }
}
after"""
        changed = extract_operations(source)
        self.assertEqual(changed.count("crate::collection_ops::"), 3)
        self.assertIn("proof! { count_update(self.candidates@); }", changed)
        self.assertIn("self.direct_parents += 1", changed)
        self.assertTrue(changed.startswith("before\n") and changed.endswith("\nafter"))
        with self.assertRaises(ValueError): extract_operations(source.replace("self.candidates.len()", "0"))

    def test_manual_comparison_has_three_contracts_and_no_inference(self):
        manual = manual_operations((FIXTURES / "collection_ops.rs").read_text())
        self.assertEqual(manual.count("verus_spec("), 3)
        self.assertNotIn("verus_infer", manual)
        self.assertIn("old(v)@.update(i as int, value)", manual)


if __name__ == "__main__":
    unittest.main()
