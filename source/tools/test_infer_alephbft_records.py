"""Ensure record experiments preserve source and distinguish coverage from soundness."""
import copy
import json
import unittest

from eval_verus_infer_alephbft_counting import remove_update_block
from eval_verus_infer_alephbft_parents import item
from eval_verus_infer_alephbft_records import add_mutation, infer_add, require_domain_rejection, validate
from test_infer_alephbft_counting import SOURCE


class RecordExperimentTests(unittest.TestCase):
    def source(self):
        return SOURCE.replace("    pub fn add(&mut self)",
            "    #[cfg_attr(verus_keep_ghost, verus_spec(requires old(self).valid(), ensures final(self).valid()))]\n"
            "    pub fn add(&mut self, creator:usize,payload:T,round:u16)") + "\n}\n\n#[cfg(test)]\nmod tests {}\n"

    def test_add_body_is_unchanged_and_both_harnesses_are_explicit(self):
        source = self.source(); inferred = infer_add(source)
        self.assertEqual(item(remove_update_block(source), "pub fn add(&mut self")[2],
                         item(inferred, "pub fn add(&mut self")[2])
        self.assertIn('verus_infer)', inferred)
        self.assertIn("pub fn check_add_invariant", inferred)
        self.assertIn("pub fn check_add_behavior", inferred)
        self.assertNotIn("collection_ops", inferred)
        self.assertEqual(inferred.count("self.add(creator, payload, round);"), 2)

    def test_mutation_changes_only_the_method_and_requires_one_target(self):
        source = infer_add(self.source()).replace("keep_constructor();", "keep_constructor(); keep_update();")
        mutated = add_mutation(source, "keep_update();", "wrong_update();")
        self.assertEqual(mutated.count("wrong_update();"), 1)
        self.assertIn("keep_constructor(); keep_update();", mutated)
        with self.assertRaises(ValueError): add_mutation(source, "missing();", "wrong();")

    def report(self):
        exact = dict(mode="exact", summary_kind="value", required_checks_passed=True,
            proofs={p:dict(status="proved",required=True) for p in ("soundness","maximality","exactness")})
        state = dict(mode="exact", summary_kind="record_state", required_checks_passed=True,
            metrics=dict(cache="miss",cache_policy="regenerated_state"),
            proofs={p:dict(status="proved",required=True) for p in ("soundness","maximality","exactness")})
        return {"verification-results":dict(success=False),"verus-infer":dict(functions=[exact,copy.deepcopy(exact),state],failed_functions=[])}

    def run_report(self,data,stderr=""):
        return dict(stdout=json.dumps(data),stderr=stderr,returncode=1)

    def test_semantic_controls_require_checked_candidates_and_honest_state_precision(self):
        validate(self.run_report(self.report()),False,1)
        for proof,status in (("soundness","not_proved"),("maximality","not_established"),("exactness","not_established")):
            bad = self.report(); bad["verus-infer"]["functions"][2]["proofs"][proof]["status"] = status
            with self.assertRaises(ValueError): validate(self.run_report(bad),False,1)

    def test_sound_comparison_keeps_state_precision_optional_and_checked(self):
        data = self.report(); state = data["verus-infer"]["functions"][2]
        state["mode"] = "sound"
        for p in ("maximality", "exactness"): state["proofs"][p]["required"] = False
        validate(self.run_report(data),False,1,state_mode="sound")
        with self.assertRaises(ValueError): validate(self.run_report(data),False,1)
        state["proofs"]["exactness"]["status"] = "not_established"
        with self.assertRaises(ValueError): validate(self.run_report(data),False,1,state_mode="sound")

    def test_frontend_and_resource_failures_are_not_semantic_controls(self):
        bad = self.report(); bad["verification-results"]["encountered-vir-error"] = True
        with self.assertRaises(ValueError): validate(self.run_report(bad),False,1)
        for text in ("panicked at checker.rs","resource limit exceeded","solver timed out"):
            with self.assertRaises(ValueError): validate(self.run_report(self.report(),text),False,1)

    def test_behavior_oracle_failure_alone_does_not_establish_failed_domain_coverage(self):
        data = self.report(); data["verus-infer"]["failed_functions"] = ["test::check_add_behavior"]
        with self.assertRaises(ValueError): require_domain_rejection(data,"precondition not satisfied")
        data["verus-infer"]["failed_functions"].append("test::check_add_invariant")
        require_domain_rejection(data,"precondition not satisfied")
        with self.assertRaises(ValueError): require_domain_rejection(data,"postcondition not satisfied")


if __name__ == "__main__": unittest.main()
