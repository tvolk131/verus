"""The maintained verifier inputs retain their archived source bytes."""
import hashlib
from pathlib import Path
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[2]
FIXTURES = ROOT / 'source/rust_verify_test/fixtures/infer'


class FixtureTests(unittest.TestCase):
    def test_provenance_and_contents(self):
        manifest = tomllib.loads((FIXTURES / 'provenance.toml').read_text())
        self.assertEqual(manifest['schema_version'], 1)
        self.assertEqual(len(manifest['archive_commit']), 40)
        seen = set()
        for fixture in manifest['fixtures']:
            with self.subTest(path=fixture['path']):
                relative = Path(fixture['path'])
                self.assertFalse(relative.is_absolute())
                self.assertNotIn('..', relative.parts)
                self.assertNotIn(relative, seen)
                seen.add(relative)
                self.assertTrue(fixture['origin'])
                data = (FIXTURES / relative).read_bytes()
                self.assertEqual(hashlib.sha256(data).hexdigest(), fixture['sha256'])
        self.assertEqual(seen, {p.relative_to(FIXTURES) for p in FIXTURES.rglob('*.rs')})


if __name__ == '__main__':
    unittest.main()
