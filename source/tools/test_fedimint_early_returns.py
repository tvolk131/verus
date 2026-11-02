"""Independent source/domain checks for the early-return replay extension."""

import unittest

from check_fedimint_early_returns import POLICY, item, lower, proof_source, regress


SOURCE = """impl InterceptPaymentRequest {
    pub fn incoming_circuit(&self) -> Option<(u64, u64)> {
        let circuit = (self.incoming_chan_id, self.htlc_id);
        (circuit != NO_INCOMING_CIRCUIT).then_some(circuit)
    }
    pub fn lnv2_blocks_to_claim_deadline(&self, current_block_height: Option<u32>) -> u32 {
        if self.incoming_circuit().is_some() {
            return 0;
        }

        current_block_height.map_or(0, |height| self.expiry.saturating_sub(height))
    }
}"""


class EarlyReturnTests(unittest.TestCase):
    def test_extraction_preserves_original_body_and_rejects_truncation(self):
        self.assertEqual(item("prefix\n" + SOURCE + "\nsuffix", "impl InterceptPaymentRequest {"), SOURCE)
        with self.assertRaises(ValueError):
            item(SOURCE[:-1], "impl InterceptPaymentRequest {")

    def test_desugarings_preserve_return_and_fail_on_source_drift(self):
        body = lower(SOURCE)
        for method in ("then_some", "map_or", "is_some", "NO_INCOMING_CIRCUIT"):
            self.assertNotIn(method, body)
        self.assertIn("return 0;", body)
        self.assertIn("self.expiry.saturating_sub(height)", body)
        for drift in (SOURCE + SOURCE, SOURCE.replace("then_some", "changed")):
            with self.assertRaises(ValueError):
                lower(drift)

    def test_both_methods_are_inferred_with_an_unrestricted_authored_wrapper(self):
        body = lower(SOURCE)
        checked = proof_source("", body)
        self.assertEqual(checked.count("#[verifier::infer]"), 2)
        self.assertEqual(checked.count("ensures"), 1)
        self.assertIn(POLICY, checked)
        self.assertNotIn("requires", checked)
        self.assertIn("incoming_chan_id: u64", checked)
        self.assertIn("htlc_id: u64", checked)
        self.assertIn("expiry: u32", checked)
        self.assertIn("height: Option<u32>", checked)
        self.assertIn("current as int + 2", checked)
        self.assertIn("==> blocks >= 2", checked)
        self.assertEqual(checked.count("return 0;"), 1)

    def test_mutant_deletes_only_the_origin_guard(self):
        body = lower(SOURCE)
        guard = "        if match self.incoming_circuit() { Some(_) => true, None => false } {\n            return 0;\n        }\n\n"
        self.assertEqual(regress(body), body.replace(guard, ""))
        self.assertIn("saturating_sub", regress(body))
        with self.assertRaises(ValueError):
            regress(regress(body))


if __name__ == "__main__":
    unittest.main()
