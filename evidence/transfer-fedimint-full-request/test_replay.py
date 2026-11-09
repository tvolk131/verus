"""Application fidelity checks run without building the engine or dependencies."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('full_request', Path(__file__).with_name('run.py'))
r = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(r)


class FrozenTargetTests(unittest.TestCase):
    def setUp(self):
        self.original = (r.HERE / 'original.rs').read_text()
        self.impl = r.native.item(self.original, 'impl InterceptPaymentRequest {')

    def test_seven_fields_real_type_and_exact_method_items(self):
        generated = r.source(self.impl)
        self.assertIn('pub payment_hash: sha256::Hash,', generated)
        struct = r.native.item(generated, 'pub struct InterceptPaymentRequest {')
        self.assertEqual(7, struct.count('    pub '))
        for marker in r.native.METHOD_HASHES:
            self.assertEqual(r.native.item(generated, marker), r.native.item(self.original, marker))
        self.assertNotIn('assume_specification', generated)
        self.assertEqual(2, generated.count('#[verifier::infer]'))

    def test_only_guard_deletion_is_authored_mutation(self):
        mutant = r.native.regress(self.impl)
        self.assertEqual(self.impl, mutant.replace('        current_block_height.map_or',
                        r.native.GUARD + '        current_block_height.map_or', 1))
        self.assertIn('self.expiry.saturating_sub(height)', mutant)

    def test_opaque_declaration_can_be_removed_independently(self):
        bare = r.source(self.impl, opaque=False)
        self.assertNotIn('external_type_specification', bare)
        self.assertEqual(r.source(self.impl), bare.replace(r.native.CONSTANT, r.OPAQUE + r.native.CONSTANT))

    def test_drift_in_a_method_fails_source_guard(self):
        with tempfile.TemporaryDirectory() as name:
            app = Path(name)
            for path in r.native.SOURCE_HASHES:
                dest = app / path
                dest.parent.mkdir(parents=True)
                dest.write_text('drift')
            with self.assertRaises(ValueError):
                r.check_source(app)

    def test_dependency_checksum_drift_rejected(self):
        with tempfile.TemporaryDirectory() as name:
            app = Path(name)
            (app / 'Cargo.lock').write_bytes((r.HERE / 'native/Cargo.lock').read_bytes())
            self.assertEqual(21, len(r.check_dependencies(app, app / 'Cargo.lock')))
            lock = app / 'mutant.lock'
            lock.write_text((app / 'Cargo.lock').read_text().replace(
                'bb18c03d0db0247e147a21a6faafd5a7eb851c743db062de72018b6b7e8e4d16', '0' * 64))
            with self.assertRaises(ValueError):
                r.check_dependencies(app, lock)


if __name__ == '__main__':
    unittest.main()
