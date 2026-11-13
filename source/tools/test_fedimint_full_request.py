"""Offline tests for full-request extraction, provenance and controls."""

import json
from pathlib import Path
import unittest

import check_fedimint_full_request as full
from infer_experiment import require_exact, validate_run


FIXTURES = Path(__file__).resolve().parents[2] / "source/tools/fixtures/transfers/transfer-fedimint-full-request"


def fixture():
    request = (FIXTURES / "request.source.rs").read_text().rstrip("\n")
    helpers = (FIXTURES / "helpers.source.rs").read_text().rstrip("\n")
    derive = "#[derive(Debug, Serialize, Deserialize, Clone)]"
    return full.extract_request(derive + "\n" + request + "\n" + helpers)


class FullRequestTests(unittest.TestCase):
    def test_frozen_items_and_all_seven_native_domains(self):
        constant, definition, implementation, _ = fixture()
        proof = full.proof_source(constant, definition, implementation)
        self.assertEqual(full.text_digest(definition), full.REQUEST_HASH)
        self.assertEqual(full.text_digest(implementation), full.IMPL_HASH)
        for field in full.FIELDS:
            self.assertIn("pub " + field + ",", proof)
        for marker, expected in full.METHOD_HASHES.items():
            self.assertEqual(full.text_digest(full.item(proof, marker)), expected)
        self.assertIn(full.CONSTANT, proof)
        self.assertNotIn("requires", proof)
        self.assertNotIn("assume", proof)
        self.assertNotIn("external_body]\n    pub fn", proof)
        self.assertEqual(proof.count("#[verifier::infer]"), 2)

    def test_original_items_fail_closed_on_source_or_type_changes(self):
        constant, definition, implementation, derive = fixture()
        for request in (definition.replace("incoming_amount_msat: u64", "incoming_amount_msat: u32"),
                        definition.replace("    pub payment_hash: sha256::Hash,\n", ""),
                        definition.replace("Option<u64>", "Option<u32>")):
            with self.assertRaises(ValueError):
                full.extract_request(derive + "\n" + request + "\n" + constant + "\n" + implementation)
        with self.assertRaises(ValueError):
            full.extract_request(derive + "\n" + definition + "\n" + constant + "\n" + implementation[:-1])

    def test_policy_boundary_monotonicity_and_unread_field_independence_are_authored(self):
        for policy in (full.POLICY, full.FOLLOWUP, full.IRRELEVANCE):
            self.assertNotIn("requires", policy)
            self.assertNotIn("assume", policy)
        self.assertIn("==> blocks >= 2", full.POLICY)
        self.assertIn("earlier <= later ==> budgets.1 <= budgets.0", full.FOLLOWUP)
        self.assertNotIn("payment_hash", full.IRRELEVANCE)
        self.assertNotIn("incoming_amount_msat", full.IRRELEVANCE)

    def test_mutant_removes_exactly_the_original_guard(self):
        constant, definition, implementation, _ = fixture()
        mutant = full.regress(implementation)
        self.assertEqual(mutant, implementation.replace(full.GUARD, "", 1))
        proof = full.proof_source(constant, definition, mutant)
        self.assertIn(full.POLICY, proof)
        self.assertIn("self.expiry.saturating_sub(height)", proof)
        self.assertEqual(full.item(proof, "pub struct InterceptPaymentRequest {"), definition)
        with self.assertRaises(ValueError):
            full.regress(mutant)

    def test_opaque_correspondence_asserts_identity_only(self):
        self.assertEqual(full.OPAQUE, "#[verifier::external_type_specification]\n"
            "#[verifier::external_body]\npub struct ExSha256Hash(sha256::Hash);\n")
        for forbidden in ("assume", "ensures", "spec fn", "axiom", "[u8", "impl", "where"):
            self.assertNotIn(forbidden, full.OPAQUE)

    def test_invariant_control_is_legal_private_shape_and_preserves_method_bodies(self):
        constant, definition, implementation, _ = fixture()
        source = full.invariant_control(constant, definition, implementation)
        self.assertIn("mod guarded", source)
        self.assertIn("#[verifier::type_invariant]", source)
        self.assertNotIn("pub payment_hash", source)
        self.assertIn("payment_hash: sha256::Hash", source)
        for marker in full.METHOD_HASHES:
            self.assertEqual(full.item(source, marker), full.item(implementation, marker))

    def test_clean_pin_and_whole_input_hash_checks(self):
        full.check_application_pin(full.APPLICATION, "", full.INPUT_HASHES)
        for head, status, hashes in [("0" * 40, "", full.INPUT_HASHES),
                (full.APPLICATION, " M unrelated.rs", full.INPUT_HASHES),
                (full.APPLICATION, "", {**full.INPUT_HASHES, full.CLIENT: "0" * 64})]:
            with self.assertRaises(ValueError):
                full.check_application_pin(head, status, hashes)

    def test_registry_pins_include_sources_checksums_and_both_real_crates(self):
        header = 'version = 4\n'
        records = []
        for name, version in [("bitcoin", "0.32.8"), ("bitcoin_hashes", "0.14.0")]:
            records.append('[[package]]\nname = "' + name + '"\nversion = "' + version +
                '"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "' + "a" * 64 + '"\n')
        lock = header + "\n".join(records)
        self.assertEqual(len(full.pinned_dependencies(lock, lock)), 2)
        for wrong in (lock.replace('0.32.8', '0.32.9'), lock.replace('a' * 64, 'b' * 64, 1),
                      lock.replace('registry+https:', 'git+https:'), lock + records[0]):
            with self.assertRaises(ValueError):
                full.pinned_dependencies(lock, wrong)

    def test_metadata_edits_preserve_serialized_api_prefix(self):
        metadata = dict(schema=38, policy="candidate-bound-evidence-v65", proofs=[], summaries="abc")
        prefix = b"\x00binary{\x80prefix"
        blob = prefix + json.dumps(metadata).encode() + b"VRINFER1"
        changed = full.replace_metadata(blob, lambda m: m.update(policy="wrong-policy"))
        self.assertTrue(changed.startswith(prefix))
        self.assertTrue(changed.endswith(b"VRINFER1"))
        self.assertIn(b'"policy":"wrong-policy"', changed)
        for invalid in (blob[:-8], b"garbageVRINFER1"):
            with self.assertRaises(ValueError):
                full.replace_metadata(invalid, lambda m: None)

    def test_preserved_caller_items_match_the_pin(self):
        for name, (_, marker, expected) in full.CALLERS.items():
            text = (FIXTURES / (name + ".source.rs")).read_text().rstrip("\n")
            self.assertEqual(full.text_digest(full.item(text, marker)), expected)
        caller = (FIXTURES / "try_handle_lightning_payment_lnv2.source.rs").read_text()
        self.assertEqual(caller.count("htlc_request.incoming_amount_msat"), 2)
        self.assertNotIn("htlc_request.amount_msat", caller)
        self.assertIn(".await?", caller)

    def test_frontend_and_auxiliary_failure_are_not_semantic_counterexamples(self):
        run = dict(returncode=1, stdout=json.dumps({"verification-results": {
            "encountered-vir-error": True}}), stderr="unsupported syntax")
        with self.assertRaises(ValueError):
            validate_run(run, False)
        with self.assertRaises(ValueError):
            require_exact([dict(mode="exact", required_checks_passed=False)])
        run["stderr"] = "resource limit"
        with self.assertRaises(ValueError):
            validate_run(run, False, allow_frontend=True)


if __name__ == "__main__":
    unittest.main()
