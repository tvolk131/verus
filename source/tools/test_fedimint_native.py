"""Independent native extraction, full-domain, mutation and fail-closed tests."""

import unittest

from check_fedimint_native import (CONSTANT, FOLLOWUP, GUARD, METHOD_HASHES, POLICY,
                                   PROJECTION, extract, item, proof_source, regress)


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


class NativeTransferTests(unittest.TestCase):
    def test_infer_annotations_preserve_complete_native_methods(self):
        checked = proof_source(CONSTANT, SOURCE)
        for marker in METHOD_HASHES:
            self.assertEqual(item(checked, marker), item(SOURCE, marker))
        self.assertIn(CONSTANT, checked)
        self.assertEqual(checked.count("#[verifier::infer]"), 2)
        for native in (".then_some(circuit)", ".is_some()", ".map_or(0, |height|", "circuit != NO_INCOMING_CIRCUIT"):
            self.assertIn(native, checked)

    def test_policy_retains_entire_original_input_domain_and_coverage(self):
        checked = proof_source(CONSTANT, SOURCE)
        self.assertNotIn("requires", checked)
        self.assertIn(POLICY, checked)
        for field in ("incoming_chan_id: u64", "htlc_id: u64", "expiry: u32"):
            self.assertIn(field, PROJECTION)
        self.assertIn("height: Option<u32>", POLICY)
        self.assertIn("current as int + 2", POLICY)
        self.assertIn("==> blocks >= 2", POLICY)

    def test_mutation_deletes_only_original_forward_guard(self):
        self.assertEqual(regress(SOURCE), SOURCE.replace(GUARD, "", 1))
        self.assertIn(".map_or(0, |height| self.expiry.saturating_sub(height))", regress(SOURCE))
        with self.assertRaises(ValueError):
            regress(regress(SOURCE))

    def test_pinned_hashes_reject_wrong_or_hand_desugared_source(self):
        for changed in (SOURCE, SOURCE.replace("then_some", "then"), SOURCE + SOURCE):
            with self.assertRaises(ValueError):
                extract(CONSTANT + "\n" + changed)

    def test_bounded_followup_uses_same_body_and_no_assumptions(self):
        self.assertNotIn("requires", FOLLOWUP)
        self.assertNotIn("assume", FOLLOWUP)
        self.assertIn("earlier <= later ==> budgets.1 <= budgets.0", FOLLOWUP)
        self.assertIn("Some(request.expiry)", FOLLOWUP)
        checked = proof_source(CONSTANT, SOURCE, POLICY + FOLLOWUP)
        self.assertEqual(item(checked, "impl InterceptPaymentRequest {"),
                         item(proof_source(CONSTANT, SOURCE), "impl InterceptPaymentRequest {"))

    def test_truncated_extraction_fails(self):
        with self.assertRaises(ValueError):
            item(SOURCE[:-1], "impl InterceptPaymentRequest {")


if __name__ == "__main__":
    unittest.main()
