"""Source-fidelity and control-classification tests for the address-range replay."""

import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_redb_address_range as address


class AddressRangeTests(unittest.TestCase):
    def test_changed_source_is_rejected_before_fixture_writes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root/address.BASE
            source.parent.mkdir(parents=True)
            source.write_text("not the pinned source")
            with self.assertRaisesRegex(ValueError, "pinned application source changed"):
                address.extract(root, root)
            self.assertFalse((root/"original.rs").exists())

    def test_extraction_preserves_fields_bodies_and_range_constructor(self):
        pieces = {
            "PageNumber": "pub(crate) struct PageNumber {pub(crate) region:u32,pub(crate) page_index:u32,pub(crate) page_order:u8}",
            "page_size_bytes": "    pub(crate) fn page_size_bytes(&self,page_size:u32)->u64 {let pages=1u64<<self.page_order;pages*u64::from(page_size)}",
            "address_range": "    pub(crate) fn address_range(&self,offset:u64,size:u64,start:u64,page_size:u32)->Range<u64> {debug_assert!(start<size);let end=start+self.page_size_bytes(page_size);start..end}",
        }
        text = pieces["PageNumber"]+"\nimpl PageNumber {\n"+pieces["page_size_bytes"]+"\n"+pieces["address_range"]+"\n}\n"
        hashes = {name:hashlib.sha256((source+"\n").encode()).hexdigest() for name,source in pieces.items()}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root/address.BASE
            source.parent.mkdir(parents=True)
            source.write_text(text)
            with patch.object(address, "SOURCE_HASH", address.digest(source)), patch.object(address, "ITEM_HASHES", hashes):
                provenance = address.extract(root, root)
            self.assertEqual(source.read_text(), text)
            for name, original in pieces.items():
                self.assertEqual(provenance[name]["sha256"], hashes[name])
                self.assertIn(original, (root/"original.rs").read_text())
                self.assertIn(original, (root/"boundaries.rs").read_text())
                self.assertIn(original.replace("pub(crate)", "pub"), (root/"provider.rs").read_text())
            for name in ("original", "boundaries", "client", "release_domain"):
                fixture = (root/(name+".rs")).read_text()
                for forbidden in ("proof {", "by(bit_vector)", "by(nonlinear_arith)", "assume(", "external_body"):
                    self.assertNotIn(forbidden, fixture)
            self.assertIn("start..end", (root/"original.rs").read_text())
            self.assertIn("relay(&page,", (root/"client.rs").read_text())
            self.assertNotEqual((root/"release_domain.rs").read_text(), (root/"original.rs").read_text())
            self.assertIn("result.end<=4096", (root/"false_containment_auxiliary.rs").read_text())

    def test_safety_controls_cover_eager_and_every_intermediate_operation(self):
        self.assertEqual(address.SAFETY["shift64_zero"][2], 64)
        self.assertEqual(address.SAFETY["shift255_zero"][2], 255)
        self.assertEqual(address.SAFETY["shift64_zero"][-1], 0)
        self.assertEqual(address.SAFETY["shift255_zero"][-1], 0)
        self.assertEqual(len(address.SAFETY), 11)
        for name in ("page_multiply_overflow", "index_multiply_overflow", "regional_add_overflow",
                     "region_multiply_overflow", "base_add_overflow", "start_add_overflow", "end_add_overflow"):
            self.assertIn(name, address.SAFETY)

    def test_weak_guard_does_not_imply_end_containment(self):
        self.assertLess(4090, 4096)
        self.assertEqual(4090+4096, 8186)
        self.assertGreater(8186, 4096)
        self.assertIn("result.end==8186", address.BOUNDARIES)
        self.assertNotIn("order<=20", address.CALLERS)
        self.assertIn("order<64", address.GENERIC)
        self.assertIn("+((1u64 << order) as int * size as int)<=region_size", address.GENERIC)

    def test_auxiliary_and_resource_failures_are_not_semantic_controls(self):
        report = {"verification-results":{"success":False},
                  "verus-infer":{"functions":[{"required_checks_passed":True}]}}
        self.assertEqual(address.classify(report, "postcondition not satisfied", 1), "semantic_goal_failure")
        self.assertEqual(address.classify(report, "precondition not satisfied", 1), "caller_obligation_failure")
        self.assertEqual(address.classify(report, "bitvector assertion not satisfied", 1), "proof_blocked")
        self.assertEqual(address.classify(report, "Resource limit exceeded", 1), "resource_or_internal_failure")


if __name__ == "__main__":
    unittest.main()
