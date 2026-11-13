import json
from pathlib import Path
import tempfile
import unittest

from audit import audit, tokens
from controls import BRIDGE_CASES, IMPORT_CASES, materialize

ROOT = Path(__file__).resolve().parents[3]
FIXTURES = Path(__file__).resolve().parent
NATIVE = Path('/private/tmp/verus-transfer-redb-miuzm2w3/redb/src/tree_store/page_store/bitmap.rs')


class Reference88Tests(unittest.TestCase):
    def test_exact_nine_body_erasure(self):
        result = audit(NATIVE.read_text(), (FIXTURES / 'reference.rs').read_text())
        self.assertTrue(result['all_nine_bodies'])
        self.assertTrue(result['exact_struct'])
        self.assertEqual(len(result['bodies']), 9)

    def test_erasure_detects_executable_mutation(self):
        source = (FIXTURES / 'reference.rs').read_text().replace('*slot |= mask;', '*slot &= mask;')
        result = audit(NATIVE.read_text(), source)
        self.assertFalse(result['bodies']['resize']['equal'])

    def test_erasure_detects_native_assertion_loss(self):
        source = (FIXTURES / 'reference.rs').read_text().replace('assert!(bit < self.len);', '')
        result = audit(NATIVE.read_text(), source)
        self.assertFalse(result['bodies']['get']['equal'])

    def test_only_u32_bridge_outside_numeric_macro(self):
        source = (ROOT / 'source/vstd/std_specs/num.rs').read_text()
        prefix, macro = source.split('macro_rules! num_specs', 1)
        self.assertEqual(prefix.count('assume_specification[u32::div_ceil]'), 1)
        self.assertNotIn('div_ceil', macro)
        self.assertIn('requires y != 0', prefix)
        self.assertIn('(x as int + y as int - 1) / (y as int)', prefix)

    def test_application_has_no_new_trust(self):
        source = (FIXTURES / 'reference.rs').read_text()
        for forbidden in ['assume', 'admit', 'external_body', 'assume_specification', 'infer']:
            self.assertNotIn(forbidden, tokens(source))

    def test_controls_frozen_deterministically(self):
        with tempfile.TemporaryDirectory() as directory:
            first = materialize(Path(directory) / 'first')
            second = materialize(Path(directory) / 'second')
            self.assertEqual(len(first), len(BRIDGE_CASES) + len(IMPORT_CASES))
            for name in first:
                self.assertEqual(first[name]['sha256'], second[name]['sha256'])
                self.assertTrue(first[name]['diagnostic'])
            frozen = json.loads((Path(directory) / 'first/frozen-expectations.json').read_text())
            self.assertEqual(frozen, first)


if __name__ == '__main__':
    unittest.main()
