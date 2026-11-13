"""Unit controls for the bounded bitmap mask replay and authored domain."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_redb_bitmap_range as bitmap
from infer_eval_support import ArtifactChanged


HELPER = """fn bits_in_range(lo: u32, hi: u32) -> u64 {
    debug_assert!(lo < hi && hi <= 64);
    let bits_at_or_above_lo = u64::MAX << lo;
    let bits_below_hi = u64::MAX >> (64 - hi);
    bits_at_or_above_lo & bits_below_hi
}
"""


class BitmapRangeTests(unittest.TestCase):
    def test_runner_does_not_disable_native_erasure_checks(self):
        self.assertNotIn("--no-erasure-check", Path(bitmap.__file__).read_text())

    def test_debug_and_release_keep_different_exact_domains(self):
        self.assertEqual(bitmap.requirements(True), "lo<hi,hi<=64,bit<64")
        self.assertEqual(bitmap.requirements(False), "lo<64,1<=hi,hi<=64,bit<64")
        for lo in range(64):
            for hi in range(1, 65):
                release_safe = lo < 64 and 1 <= hi <= 64
                self.assertTrue(release_safe)
                debug_safe = lo < hi and hi <= 64
                self.assertEqual(debug_safe, lo < hi)

    def test_body_is_copied_without_arithmetic_rewriting(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / bitmap.BITMAP
            source.parent.mkdir(parents=True)
            source.write_text("before\n" + HELPER + "after\n")
            output = root / "output"
            output.mkdir()
            with patch.object(bitmap, "SOURCE_HASH", bitmap.digest(source)), patch.object(
                    bitmap, "ITEM_HASH", hashlib.sha256(HELPER.encode()).hexdigest()):
                helper, receipt = bitmap.extract(root, output)
            self.assertEqual(helper, "#[verifier::infer]\npub " + HELPER)
            self.assertEqual((output / "bits_in_range.original.rs").read_text(), HELPER)
            self.assertEqual(receipt["line"], 2)
            self.assertIn("not extraction or verification of resize", receipt["update"])

    def test_changed_source_is_rejected_before_extracting(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / bitmap.BITMAP
            source.parent.mkdir(parents=True)
            source.write_text(HELPER)
            with self.assertRaisesRegex(ValueError, "pinned bitmap.rs changed"):
                bitmap.extract(root, root)
            self.assertFalse((root / "bits_in_range.original.rs").exists())

    def test_changed_helper_is_rejected_even_with_matching_file_hash(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / bitmap.BITMAP
            source.parent.mkdir(parents=True)
            source.write_text(HELPER.replace(" & bits_below_hi", " ^ bits_below_hi"))
            with patch.object(bitmap, "SOURCE_HASH", bitmap.digest(source)):
                with self.assertRaisesRegex(ValueError, "pinned interval helper changed"):
                    bitmap.extract(root, root)

    def test_authored_goals_cover_every_observed_bit_and_update_frame(self):
        self.assertIn("(lo<=bit && bit<hi)", bitmap.membership(True))
        self.assertIn("word | mask", bitmap.update(True))
        self.assertIn("word & !mask", bitmap.update(False))
        self.assertIn("else {(word>>bit)&1u64==1u64}", bitmap.update(False))
        self.assertNotIn("proof {", bitmap.membership(True) + bitmap.update(True))

    def test_native_oracle_keeps_original_shifts_and_independent_membership(self):
        source = bitmap.native("#[verifier::infer]\npub " + HELPER)
        self.assertIn(HELPER, source)
        self.assertNotIn("verifier::infer", source)
        self.assertIn("assert_eq!(((mask>>bit)&1)==1,lo<=bit && bit<hi)", source)
        self.assertIn("cfg!(debug_assertions)", source)

    def test_diagnostic_categories_are_not_interchangeable(self):
        data = {"verus-infer": {}, "verification-results": {}}
        for diagnostic, expected in [
                ("postcondition not satisfied", "semantic_goal_failure"),
                ("precondition not satisfied", "safety_failure"),
                ("Verus Infer: bit-vector proposal not satisfied", "auxiliary_theorem_failure"),
                ("error: bitvector assertion not satisfied", "auxiliary_theorem_failure"),
                ("resource limit exceeded", "resource_or_runtime_failure")]:
            self.assertEqual(bitmap.classify(data, diagnostic, 1), expected)
        self.assertEqual(bitmap.classify(data, "", 0), "proved")
        poisoned = json.loads(json.dumps(data))
        poisoned["verus-infer"]["status"] = "invalid_import"
        self.assertEqual(bitmap.classify(poisoned, "", 1), "frontend_rejection")

    def test_import_and_stale_export_fixtures_must_be_separate_and_immutable(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            dependency = root / "dependency.vir"
            dependency.write_bytes(b"fixture")
            with self.assertRaisesRegex(ValueError, "paths must differ"):
                bitmap.import_guard(dependency, dependency)
            with self.assertRaisesRegex(ValueError, "fixture is missing"):
                bitmap.import_guard(root / "output.vir", root / "missing.vir")
            guard = bitmap.import_guard(root / "output.vir", dependency)
            guard.check()
            dependency.write_bytes(b"changed")
            with self.assertRaises(ArtifactChanged):
                guard.check()

    def test_intended_metadata_and_budget_diagnostics_are_required(self):
        for diagnostic in bitmap.METADATA_DIAGNOSTICS.values():
            bitmap.failure_diagnostic("frontend_rejection", diagnostic, diagnostic)
            with self.assertRaisesRegex(ValueError, "intended rejection"):
                bitmap.failure_diagnostic("frontend_rejection", "could not open imported library", diagnostic)
        bitmap.failure_diagnostic("resource_or_runtime_failure", "Resource limit (rlimit) exceeded")
        for diagnostic in ("panicked at resource limit", "timed out", "other failure"):
            with self.assertRaisesRegex(ValueError, "not an intentional solver-budget"):
                bitmap.failure_diagnostic("resource_or_runtime_failure", diagnostic)


if __name__ == "__main__":
    unittest.main()
