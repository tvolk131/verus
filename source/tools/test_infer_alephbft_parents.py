"""Keep unsupported source and missing proof evidence out of semantic successes."""
import copy
import json
import unittest

from eval_verus_infer_alephbft_parents import remove_constructor_loop_annotation, request_collector_inference, validate, validate_native


class ParentExperimentTests(unittest.TestCase):
    def report(self):
        return {"verification-results": {"success": False}, "verus-infer": {
            "functions": [{"mode": "exact", "required_checks_passed": True,
                           "proofs": {p: {"status": "proved", "required": True}
                                      for p in ("soundness", "maximality", "exactness")}}]}}

    def run_report(self, data, stderr=""):
        return dict(stdout=json.dumps(data), stderr=stderr, returncode=1)

    def test_semantic_control_requires_every_helper_proof(self):
        good = self.report()
        validate(self.run_report(good), False, 1)
        for proof in ("soundness", "maximality", "exactness"):
            bad = copy.deepcopy(good)
            bad["verus-infer"]["functions"][0]["proofs"][proof]["status"] = "not_proved"
            with self.assertRaises(ValueError):
                validate(self.run_report(bad), False, 1)

    def test_frontend_timeout_and_panic_are_not_semantic_controls(self):
        bad = self.report(); bad["verification-results"]["encountered-vir-error"] = True
        with self.assertRaises(ValueError):
            validate(self.run_report(bad), False, 1)
        for text in ("Resource limit exceeded", "solver timed out", "panicked at checker.rs"):
            with self.assertRaises(ValueError):
                validate(self.run_report(self.report(), text), False, 1)

    def test_unsupported_source_is_explicitly_classified(self):
        data = self.report(); data["verus-infer"]["status"] = "unsupported"
        data["verification-results"]["encountered-vir-error"] = True
        validate(self.run_report(data, "generic method"), False, 0, "generic method")
        with self.assertRaises(ValueError):
            validate(self.run_report(data, "unrelated syntax error"), False, 0, "generic method")

    def test_invariant_ablation_preserves_termination(self):
        source = """before
        #[cfg_attr(verus_keep_ghost, verus_spec(
            invariant candidates@.len() <= node_count,
            decreases node_count - candidates@.len(),
        ))]
        while candidates.len() < node_count { candidates.push(None); }
after"""
        changed = remove_constructor_loop_annotation(source)
        self.assertNotIn("invariant", changed)
        self.assertIn("decreases node_count - candidates@.len()", changed)
        self.assertTrue(changed.endswith("while candidates.len() < node_count { candidates.push(None); }\nafter"))

    def test_native_runs_must_execute_tests_and_preserve_counterexamples(self):
        good = dict(stdout="test result: ok. 1 passed; 0 failed", stderr="Running unittests", returncode=0)
        validate_native(good, True, "positive")
        for bad in (dict(good, stdout="test result: ok. 0 passed; 0 failed"),
                    dict(good, stderr="compilation failed")):
            with self.assertRaises(ValueError):
                validate_native(bad, True, "positive")
        with self.assertRaises(ValueError):
            validate_native(good, True, "original-counterexample")
        evidence = dict(good, stdout=good["stdout"] + "\nINFER_PARENT_COUNTEREXAMPLE n=128",
                        stderr=good["stderr"] + "\nattempt to multiply with overflow")
        validate_native(evidence, True, "original-counterexample")

    def test_collector_probe_removes_contract_and_ghost_calls(self):
        source = """    #[cfg_attr(verus_keep_ghost, verus_spec(requires old(self).valid()))]
    pub fn add(&mut self, n: usize) {
        if n > 0 {
            #[cfg(verus_keep_ghost)]
            proof! { count_update(n); count_bound(n); }
            self.count += 1;
        }
    }
"""
        changed = request_collector_inference(source)
        self.assertIn("verus_infer", changed)
        self.assertNotIn("verus_spec", changed)
        self.assertNotIn("proof!", changed)
        self.assertNotIn("count_update", changed)
        self.assertIn("self.count += 1", changed)


if __name__ == "__main__":
    unittest.main()
