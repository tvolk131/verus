"""Keep private-crate scope and exact-certificate controls explicit."""
import json
import unittest

from eval_verus_infer_alephbft_parents import item
from eval_verus_infer_alephbft_private import private_api, update_only, validate
from eval_verus_infer_alephbft_wrappers import wrap_collector
import test_infer_alephbft_records as record_tests
from test_infer_alephbft_wrappers import NODE


class PrivateExperimentTests(unittest.TestCase):
    def test_provider_keeps_original_lookup_and_insertion_and_supplies_a_closed_view(self):
        source = private_api(NODE, "// threshold body")
        for name in ("get", "insert"):
            self.assertEqual(item(source, f"pub fn {name}(")[2], item(NODE, f"pub fn {name}(")[2])
        self.assertIn("closed spec fn view", source)
        self.assertIn("{ self.0@ }", source)
        self.assertIn("NodeCount(self.0.len())", source)
        self.assertNotIn("T: Copy", source)
        self.assertNotIn("pub Vec", source)

    def source(self):
        source = record_tests.RecordExperimentTests().source()
        for name in ("new", "next"):
            source = source.replace(f"    pub fn {name}(",
                "    #[cfg_attr(verus_keep_ghost, verus_spec(ensures true))]\n"
                f"    pub fn {name}(")
        source = source.replace("\n}\n\n#[cfg(test)]", """
    #[cfg_attr(verus_keep_ghost, verus_spec(ensures true))]
    pub fn prospective_parents(&self) { self.candidates.len(); }
}

#[cfg(test)]""")
        return source

    def test_update_scope_keeps_body_invariant_equivalence_and_oracle(self):
        original = self.source(); source = update_only(original)
        self.assertEqual(item(source, "pub fn add(&mut self")[2],
                         item(wrap_collector(original), "pub fn add(&mut self")[2])
        for name in ("new", "next", "prospective_parents"):
            self.assertNotIn(f"pub fn {name}(", source)
        for name in ("check_add_invariant", "check_add_behavior", "library_count_agrees"):
            self.assertIn(name, source)
        self.assertEqual(source.count("let _ = self.candidates.size();"), 2)
        self.assertNotIn("self.candidates.0", source)

    def run_report(self):
        f = dict(mode="exact", required_checks_passed=True,
                 proofs={p: dict(required=True, status="proved")
                         for p in ("soundness", "maximality", "exactness")})
        return dict(returncode=1, stderr="", stdout=json.dumps({
            "verification-results": dict(success=False), "verus-infer": dict(functions=[f])}))

    def test_negative_controls_require_all_exact_obligations(self):
        validate(self.run_report(), False, 1)
        for proof in ("soundness", "maximality", "exactness"):
            run = self.run_report(); data = json.loads(run["stdout"])
            data["verus-infer"]["functions"][0]["proofs"][proof]["status"] = "not_established"
            run["stdout"] = json.dumps(data)
            with self.assertRaises(ValueError): validate(run, False, 1)
        with self.assertRaises(ValueError): validate(self.run_report(), False, 2)

    def test_unsupported_view_requires_its_own_diagnostic(self):
        run = self.run_report()
        with self.assertRaises(ValueError): validate(run, False, 0, unsupported=True)
        run["stderr"] = "record state requires visible transparent record fields"
        validate(run, False, 0, unsupported=True)
        for error in ("panicked at checker", "resource limit exceeded", "timed out"):
            run["stderr"] = error
            with self.assertRaises(ValueError): validate(run, False, 0, unsupported=True)


if __name__ == "__main__": unittest.main()
