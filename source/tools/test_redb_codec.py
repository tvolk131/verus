import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import check_redb_codec as codec


class RedbCodecTests(unittest.TestCase):
    def test_negative_exports_cannot_remove_positive_import_evidence(self):
        output = Path("output")
        positive = codec.export_path(output, "conversion_client", "cold", False)
        self.assertEqual(positive, codec.export_path(output, "conversion_client", "warm", False))
        for phase in ("metadata_policy", "metadata_proof", "negative"):
            negative = codec.export_path(output, "conversion_client", phase, True)
            self.assertNotEqual(negative, positive)
            self.assertEqual(negative, output / ("conversion_client_" + phase + ".vir"))

    def test_frozen_domain_keeps_original_machine_bounds(self):
        self.assertIn("region <= 0x000F_FFFF,", codec.ROUNDTRIP)
        self.assertIn("page_order <= 20,", codec.ROUNDTRIP)
        self.assertIn("page_index <= (0x000F_FFFFu32 >> page_order),", codec.ROUNDTRIP)
        self.assertNotIn("assume", codec.ROUNDTRIP)
        self.assertNotIn("external_body", codec.ROUNDTRIP)

    def test_source_hash_rejection_precedes_extraction(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / codec.BASE
            source.parent.mkdir(parents=True)
            source.write_text("altered source")
            with self.assertRaisesRegex(ValueError, "pinned application source changed"):
                codec.extract(root, root)
            self.assertFalse((root / "codec.rs").exists())

    def test_extraction_keeps_original_bodies_and_detects_mutation(self):
        source = """pub(crate) struct PageNumber {
    pub(crate) region: u32,
    pub(crate) page_index: u32,
    pub(crate) page_order: u8,
}
impl PageNumber {
    pub(crate) fn to_le_bytes(self) -> [u8; 8] { [0; 8] }
    pub(crate) fn from_le_bytes(bytes: [u8; 8]) -> Self {
        let temp = u64::from_le_bytes(bytes);
        let region = ((temp >> 20) & 0x000F_FFFF) as u32;
        Self { region, page_index: 0, page_order: 0 }
    }
}
"""
        pieces = {"PageNumber": codec.item(source, "pub(crate) struct PageNumber")}
        pieces.update({name: codec.item(source, "    pub(crate) fn " + name + "(")
                       for name in ("to_le_bytes", "from_le_bytes")})
        original_digest = codec.digest

        def test_digest(path):
            if path.name == "codec.rs":
                return codec.FROZEN_HASHES["codec.rs"]
            return original_digest(path)

        hashes = {name: hashlib.sha256((body + "\n").encode()).hexdigest()
                  for name, body in pieces.items()}
        hashes["codec.rs"] = codec.FROZEN_HASHES["codec.rs"]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / codec.BASE
            path.parent.mkdir(parents=True)
            path.write_text(source)
            with patch.object(codec, "SOURCE_HASH", original_digest(path)), \
                    patch.object(codec, "FROZEN_HASHES", hashes), \
                    patch.object(codec, "digest", test_digest):
                codec.extract(root, root)
            positive = (root / "codec.rs").read_text()
            authored = (root / "codec_authored.rs").read_text()
            mutant = (root / "decoder_mutation.rs").read_text()
            self.assertEqual(mutant, authored.replace("((temp >> 20) & 0x000F_FFFF)", "((temp >> 21) & 0x000F_FFFF)"))
            self.assertEqual((root / "decoder_mutation_unaided.rs").read_text(),
                             positive.replace("((temp >> 20) & 0x000F_FFFF)", "((temp >> 21) & 0x000F_FFFF)"))
            self.assertIn("by (bit_vector)", authored)
            for name in ("codec", "codec_complete", "codec_provider", "codec_client", "decoder_mutation_unaided"):
                unaided = (root / (name + ".rs")).read_text()
                self.assertNotIn("proof {", unaided)
                self.assertNotIn("by (bit_vector)", unaided)
                self.assertNotIn("lemma_auto", unaided)
            for body in pieces.values():
                self.assertIn(body, positive)

    def test_frontend_rejection_is_not_a_semantic_counterexample(self):
        data = {"verification-results": {"success": False, "encountered-vir-error": True},
                "verus-infer": {"functions": [{"required_checks_passed": True}]}}
        self.assertEqual(codec.classify(data, "postcondition not satisfied", 1), "frontend_rejection")
        data["verification-results"]["encountered-vir-error"] = False
        data["verification-results"]["encountered-error"] = True
        self.assertEqual(codec.classify(data, "error[E0277]: type error", 1), "frontend_rejection")
        self.assertEqual(codec.classify(data, "postcondition not satisfied", 1), "semantic_goal_failure")
        self.assertEqual(codec.classify(data, "precondition not satisfied", 1), "caller_obligation_failure")
        self.assertEqual(codec.classify(data, "resource limit; postcondition not satisfied", 1), "resource_or_internal_failure")
        data["verus-infer"]["functions"][0]["required_checks_passed"] = False
        self.assertEqual(codec.classify(data, "postcondition not satisfied", 1), "proof_blocked")


if __name__ == "__main__":
    unittest.main()
