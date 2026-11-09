#!/usr/bin/env python3
"""Small replay guard tests. Solver outcomes belong to the full replay."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('automation_replay', HERE/'replay.py')
replay = importlib.util.module_from_spec(spec)
spec.loader.exec_module(replay)

class Guards(unittest.TestCase):
    def test_frozen_fixture_and_semantic_fidelity(self):
        replay.fidelity()

    def test_changed_state_update_rejected(self):
        import sys
        sys.path.insert(0, str(replay.HISTORY))
        from replay import fidelity
        with tempfile.TemporaryDirectory() as d:
            path = Path(d)/'changed.rs'
            path.write_text((HERE/'automatic.rs').read_text().replace('group |=', 'group ^='))
            with self.assertRaisesRegex(ValueError, 'original executable body lost'):
                fidelity(path)

if __name__ == '__main__':
    unittest.main()
