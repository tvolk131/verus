"""Failure injection for the AlephBFT experiment's outcome classification."""
import copy
import json
import unittest

from eval_verus_infer_alephbft import native_errors, validate


class AlephPilotTests(unittest.TestCase):
    def report(self):
        return {
            "verification-results": {"success": False, "encountered-vir-error": False},
            "verus-infer": {"functions": [{
                "required_checks_passed": True, "mode": "exact", "contract": {"requires": "true"},
                "proofs": {p: {"status": "proved", "required": True}
                           for p in ("soundness", "maximality", "exactness")},
            }]},
        }

    def run_report(self, data, stderr=""):
        return {"stdout": json.dumps(data), "stderr": stderr, "returncode": 1}

    def test_negative_control_requires_verified_unrestricted_callee(self):
        data = self.report()
        validate(self.run_report(data), False, True)
        for change in ("domain", "proof", "required"):
            invalid = copy.deepcopy(data)
            function = invalid["verus-infer"]["functions"][0]
            if change == "domain":
                function["contract"]["requires"] = "false"
            elif change == "proof":
                function["proofs"]["soundness"]["status"] = "not_proved"
            else:
                function["proofs"]["exactness"]["required"] = False
            with self.assertRaises(ValueError):
                validate(self.run_report(invalid), False, True)

    def test_frontend_errors_and_timeouts_are_not_semantic_rejections(self):
        data = self.report()
        for stderr in ("Resource limit exceeded", "solver timed out"):
            with self.assertRaises(ValueError):
                validate(self.run_report(data, stderr), False, True)
        data["verification-results"]["encountered-vir-error"] = True
        with self.assertRaises(ValueError):
            validate(self.run_report(data), False, True)

    def test_native_baseline_comparison_includes_unnumbered_errors(self):
        original = "error[E0412]: missing type\n --> collection.rs:1:2\nerror: missing parameters\n --> collection.rs:3:4\n"
        errors = native_errors(original)
        self.assertEqual(len(errors["messages"]), 2)
        self.assertNotEqual(errors, native_errors(original + "error: additional failure\n"))


if __name__ == "__main__":
    unittest.main()
