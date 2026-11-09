import importlib.util
from pathlib import Path
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('redb_set_replay', HERE/'replay.py')
r = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r)

class FidelityTests(unittest.TestCase):
    def test_frozen_baseline(self):
        r.fidelity(HERE/'baseline.rs')

    def test_erased_assistance_retains_body(self):
        r.fidelity(HERE/'assisted.rs')
        r.same_contracts(HERE/'baseline.rs', HERE/'assisted.rs')

    def test_state_update_change_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)/'mutated.rs'
            p.write_text((HERE/'baseline.rs').read_text().replace('group |=', 'group ^='))
            with self.assertRaisesRegex(ValueError, 'original executable body lost'):
                r.fidelity(p)

    def test_narrowed_assisted_domain_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)/'narrowed.rs'
            p.write_text((HERE/'assisted.rs').read_text().replace('bit < old(self).len,', 'bit < old(self).len, bit == 0,'))
            with self.assertRaisesRegex(ValueError, 'semantic contracts'):
                r.same_contracts(HERE/'baseline.rs', p)

    def test_erased_admission_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)/'admitted.rs'
            p.write_text((HERE/'assisted.rs').read_text().replace('proof {', 'proof { assume(false);', 1))
            with self.assertRaisesRegex(ValueError, 'forbidden trust'):
                r.fidelity(p)

if __name__ == '__main__': unittest.main()
