import hashlib
from pathlib import Path
import unittest

from goal_scope import check_goal_snapshot, goal_scope_sha256, GOAL_REQUIREMENTS_SHA256, START, END


class GoalScopeTests(unittest.TestCase):
    def setUp(self):
        self.original = (Path(__file__).parent / 'TARGET.md').read_bytes()

    def check(self, data, pin=GOAL_REQUIREMENTS_SHA256):
        return check_goal_snapshot(data, hashlib.sha256(data).hexdigest(), pin)

    def test_current_requirements(self):
        self.assertEqual(self.check(self.original), GOAL_REQUIREMENTS_SHA256)

    def test_progress_edits_preserve_requirements(self):
        self.assertEqual(self.check(b'New progress\n' + self.original + b'\nNew coordination note'),
                         GOAL_REQUIREMENTS_SHA256)

    def test_domain_mutation_rejected(self):
        data = self.original.replace(b'u64 and arbitrary initialized', b'u32 and arbitrary initialized')
        self.assertNotEqual(data, self.original)
        with self.assertRaises(ValueError):
            self.check(data)

    def test_completion_mutation_rejected(self):
        data = self.original.replace(b'1. The full frozen canonical theorem', b'1. The partial frozen canonical theorem')
        self.assertNotEqual(data, self.original)
        with self.assertRaises(ValueError):
            self.check(data)

    def test_missing_or_wrong_pin_rejected(self):
        for pin in [None, '', '0' * 64]:
            with self.subTest(pin=pin), self.assertRaises(ValueError):
                self.check(self.original, pin)

    def test_missing_duplicate_and_reversed_boundaries_rejected(self):
        for data in [self.original.replace(START, b''), self.original.replace(END, b''),
                     START + self.original, self.original + END, END + START]:
            with self.subTest(data=data[:30]), self.assertRaises(ValueError):
                goal_scope_sha256(data)

    def test_retained_snapshot_identity_required(self):
        with self.assertRaises(ValueError):
            check_goal_snapshot(b'Progress\n' + self.original,
                                hashlib.sha256(self.original).hexdigest(), GOAL_REQUIREMENTS_SHA256)


if __name__ == '__main__':
    unittest.main()
