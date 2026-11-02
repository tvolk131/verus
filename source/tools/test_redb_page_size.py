import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_redb_page_size as page


class RedbPageSizeTests(unittest.TestCase):
    def test_canonical_domain_has_no_extra_size_or_record_restriction(self):
        canonical = page.CALLERS.split("fn full_domain")[0]
        requires = canonical.split("requires", 1)[1].split("ensures", 1)[0]
        self.assertEqual(requires.strip(), "page_order <= 20,")
        self.assertIn("result <= 4503599626321920", canonical)
        self.assertIn("result <= 1048576 * page_size as int", canonical)
        self.assertIn("page_order < 64", page.CALLERS)
        for forbidden in ("assume(", "external_body", "admit(", "axiom"):
            self.assertNotIn(forbidden, page.BOUND + page.CALLERS)

    def test_source_rejection_precedes_writing_fixtures(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / page.BASE
            source.parent.mkdir(parents=True)
            source.write_text("altered source")
            with self.assertRaisesRegex(ValueError, "pinned application source changed"):
                page.extract(root, root)
            self.assertFalse((root / "page_size.rs").exists())

    def test_extraction_and_independent_negative_inputs(self):
        declaration = "pub(crate) struct PageNumber { pub(crate) region:u32, pub(crate) page_index:u32, pub(crate) page_order:u8 }"
        method = "    pub(crate) fn page_size_bytes(&self, page_size:u32)->u64 { let pages = 1u64 << self.page_order; pages * u64::from(page_size) }"
        source = declaration + "\nimpl PageNumber {\n" + method + "\n}\n"
        hashes = {name: hashlib.sha256((contents + "\n").encode()).hexdigest()
                  for name, contents in (("PageNumber", declaration), ("page_size_bytes", method))}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            original = root / page.BASE
            original.parent.mkdir(parents=True)
            original.write_text(source)
            with patch.object(page, "SOURCE_HASH", page.digest(original)), patch.object(page, "ITEM_HASHES", hashes):
                provenance = page.extract(root, root)
            positive = (root / "page_size.rs").read_text()
            self.assertIn(method, positive)
            self.assertIn(declaration, positive)
            for name in ("page_size", "page_size_provider", "page_size_client"):
                unaided = (root / (name + ".rs")).read_text()
                self.assertNotIn("proof {", unaided)
                self.assertNotIn("page_size_bound", unaided)
                self.assertNotIn("by(bit_vector)", unaided)
                self.assertNotIn("by(nonlinear_arith)", unaided)
            self.assertIn(page.BOUND, (root / "page_size_authored.rs").read_text())
            self.assertIn(page.BOUND, (root / "false_property.rs").read_text())
            self.assertEqual(provenance["page_size_bytes"]["line"], 3)
            self.assertIn("page_order:64}.page_size_bytes(0)", (root / "shift64_zero.rs").read_text())
            self.assertIn("page_order:255}.page_size_bytes(0)", (root / "shift255_zero.rs").read_text())
            self.assertIn("requires page_order>=64", (root / "shift_all_invalid_zero.rs").read_text())
            self.assertIn("page_order:63}.page_size_bytes(2)", (root / "multiply_overflow.rs").read_text())
            self.assertIn("by(bit_vector)", (root / "multiply_overflow.rs").read_text())
            self.assertIn("ensures result !=", (root / "false_property.rs").read_text())
            self.assertEqual((root / "native_mutation.rs").read_text(),
                             (root / "native.rs").read_text().replace("let pages = 1u64", "let pages = 2u64"))
            self.assertIn("relay(&page_size_provider::PageNumber", (root / "page_size_client.rs").read_text())
            self.assertEqual(original.read_text(), source)

    def test_frontend_and_resource_failures_are_not_semantic_controls(self):
        data = {"verification-results": {"success": False},
                "verus-infer": {"functions": [{"required_checks_passed": True}]}}
        self.assertEqual(page.classify(data, "precondition not satisfied", 1), "caller_obligation_failure")
        self.assertEqual(page.classify(data, "postcondition not satisfied", 1), "semantic_goal_failure")
        data["verification-results"]["encountered-vir-error"] = True
        self.assertEqual(page.classify(data, "precondition not satisfied", 1), "frontend_rejection")
        self.assertEqual(page.classify(data, "resource limit; precondition not satisfied", 1), "resource_or_internal_failure")
        data["verification-results"]["encountered-vir-error"] = False
        data["verus-infer"]["functions"] = []
        self.assertEqual(page.classify(data, "postcondition not satisfied", 1), "proof_blocked")


if __name__ == "__main__":
    unittest.main()
