"""Keep the shared receipt expectation bound to the Rust engine declarations."""

from pathlib import Path
import re
import unittest

from infer_transfer_identity import (
    CACHE_VERSION, ENGINE_IDENTITY, INTERFACE_POLICY, INTERFACE_SCHEMA,
)


ROOT = Path(__file__).resolve().parents[2]


class TransferIdentityTests(unittest.TestCase):
    def test_cache_version_matches_the_engine(self):
        source = (ROOT / "source/vir/src/infer/registry.rs").read_text()
        self.assertEqual(re.findall(r'^const VERSION: &str = "([^"\n]+)";$', source, re.MULTILINE),
                         [CACHE_VERSION])

    def test_certificate_identity_matches_the_engine(self):
        source = (ROOT / "source/rust_verify/src/import_export.rs").read_text()
        self.assertEqual(re.findall(r'^const INFER_POLICY: &str = "([^"\n]+)";$', source, re.MULTILINE),
                         [INTERFACE_POLICY])
        self.assertEqual(re.findall(r'^const INFER_SCHEMA: u32 = ([0-9]+);$', source, re.MULTILINE),
                         [str(INTERFACE_SCHEMA)])

    def test_numeric_receipt_identity_matches_the_checked_strings(self):
        cache = re.fullmatch(r"verus-infer-v([0-9]+):.+", CACHE_VERSION)
        policy = re.fullmatch(r"candidate-bound-evidence-v([0-9]+)", INTERFACE_POLICY)
        self.assertIsNotNone(cache)
        self.assertIsNotNone(policy)
        self.assertEqual(ENGINE_IDENTITY, (int(cache[1]), int(policy[1]), INTERFACE_SCHEMA))


if __name__ == "__main__":
    unittest.main()
