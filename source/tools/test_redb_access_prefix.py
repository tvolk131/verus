"""Offline extraction, full-domain and non-semantic diagnostic controls."""
import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_redb_access_prefix as access


class AccessPrefixTests(unittest.TestCase):
    def test_full_checked_domain_keeps_pointer_representability(self):
        self.assertEqual(len(access.REQUIREMENTS), 10)
        self.assertIn("order<64", access.REQUIREMENTS)
        self.assertEqual(access.REQUIREMENTS[-1], access.COUNT+"<=usize::MAX")
        source=access.caller()+access.relationship()
        for term in ["header as int +", "index as int *", "region as int *", "<region_size", "+"+access.COUNT+"<=u64::MAX"]:
            self.assertIn(term, source)
        for forbidden in ["order<=20", "order <= 20", "proof {", "by(bit_vector)", "by(nonlinear_arith)", "assume("]:
            self.assertNotIn(forbidden, source)

    def test_range_length_is_authored_not_a_trusted_application_summary(self):
        source=access.relationship()
        self.assertIn("result.0 as int==result.1.end as int-result.1.start as int", source)
        self.assertIn(".access_length(", source)
        self.assertIn(".address_range(size.into(),", source)
        self.assertNotIn("#[verifier::infer]", source)
        self.assertNotIn("external_body", source)

    def test_changed_base_pin_fails_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            source=root/access.BASE
            source.parent.mkdir(parents=True)
            source.write_text("changed")
            with self.assertRaisesRegex(ValueError,"base.rs pin changed"):
                access.extract(root,root)

    def test_extract_selects_transactional_memory_not_forwarding_methods(self):
        parts={
            "PageNumber":"pub(crate) struct PageNumber {pub(crate) region:u32,pub(crate) page_index:u32,pub(crate) page_order:u8}",
            "page_size_bytes":"    pub(crate) fn page_size_bytes(&self,page_size:u32)->u64 {let pages=1u64<<self.page_order;pages*u64::from(page_size)}",
            "address_range":"    pub(crate) fn address_range(&self,offset:u64,size:u64,start:u64,page_size:u32)->Range<u64> {debug_assert!(start<size);let end=start+self.page_size_bytes(page_size);start..end}",
        }
        base=parts["PageNumber"]+"\nimpl PageNumber {\n"+parts["page_size_bytes"]+"\n"+parts["address_range"]+"\n}\n"
        marker="    pub(crate) fn get_page(&self, page_number: PageNumber, hint: PageHint) -> Result<PageImpl> {\n"
        prefix="        let range = page_number.address_range(self.page_size.into(),self.region_size,self.region_header_with_padding_size,self.page_size);\n        let len: usize = (range.end - range.start).try_into().unwrap();\n"
        method=marker+"        Self::check_page_order(page_number)?;\n"+prefix+"        let mem = self.storage.read(range.start,len,hint)?;\n        construct(mem)\n    }"
        manager="impl Forward {\n"+marker+"self.mem.get_page(page_number,hint)\n}}\nimpl TransactionalMemory {\n"+method+"\n}\n"
        hashes={name:hashlib.sha256((piece+"\n").encode()).hexdigest() for name,piece in parts.items()}
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            for name,text in [(access.BASE,base),(access.MANAGER,manager)]:
                path=root/name
                path.parent.mkdir(parents=True,exist_ok=True)
                path.write_text(text)
            with patch.object(access,"SOURCE_HASH",access.digest(root/access.BASE)), patch.object(access,"ITEM_HASHES",hashes), patch.object(access,"MANAGER_HASH",access.digest(root/access.MANAGER)), patch.object(access,"PREFIX_HASH",hashlib.sha256(prefix.encode()).hexdigest()):
                source,receipt=access.extract(root,root)
            self.assertEqual((root/"get_page_prefix.original.rs").read_text(),prefix)
            self.assertEqual((root/"get_page.original.rs").read_text(),method+"\n")
            self.assertEqual((root/access.BASE).read_text(),base)
            self.assertEqual((root/access.MANAGER).read_text(),manager)
            for piece in parts.values():
                self.assertIn(piece,source)
            self.assertIn(prefix,source)
            self.assertNotIn("storage.read",source)
            self.assertNotIn("check_page_order",source)
            self.assertIn("append len return",receipt["adaptation"])

    def test_semantic_safety_auxiliary_and_frontend_failures_are_distinct(self):
        data={"verification-results":{},"verus-infer":{"status":"caller_proof_failed"}}
        for diagnostic,expected in [
            ("postcondition not satisfied","semantic_goal_failure"),
            ("precondition not satisfied","safety_failure"),
            ("Verus Infer: polynomial proposal not satisfied","auxiliary_theorem_failure"),
            ("Verus Infer: bit-vector proposal not satisfied","auxiliary_theorem_failure"),
            ("error: bitvector assertion not satisfied","auxiliary_theorem_failure"),
            ("resource limit","resource_or_runtime_failure"),
            ("timed out","resource_or_runtime_failure"),
        ]:
            self.assertEqual(access.classify(data,diagnostic,1),expected)
        self.assertEqual(access.classify({"verification-results":{"encountered-vir-error":True}},"unsupported",1),"frontend_rejection")

    def test_mixed_auxiliary_authored_diagnostics_are_not_semantic_evidence(self):
        for auxiliary in ("Verus Infer: polynomial proposal not satisfied",
                "Verus Infer: bit-vector proposal not satisfied", "error: bitvector assertion not satisfied"):
            for diagnostic in ("precondition not satisfied", "postcondition not satisfied"):
                with self.subTest(auxiliary=auxiliary, diagnostic=diagnostic):
                    self.assertEqual(access.classify({"verification-results":{}},
                        auxiliary+"\n"+diagnostic,1),"mixed_failure")

    def test_native_oracle_keeps_into_and_try_into_syntax(self):
        original="fn access_length(){let offset:u64=number.into();let len:usize=number.try_into().unwrap();}"
        native=access.native(original)
        self.assertIn(original,native)
        self.assertIn("for order in 0u8..64",native)
        self.assertIn("usize::BITS",native)
        self.assertIn("u32::try_from(count)",native)


if __name__=="__main__":
    unittest.main()
