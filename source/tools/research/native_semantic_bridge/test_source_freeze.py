"""Exercise both source-audit entry points against incomplete pretrial freezes."""
import copy
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from source_map import ROOT, digest, source_audit_inputs

RAW = Path('/tmp/native-semantic-bridge-e4a8dcab/source-v1')


class SourceFreeze(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for path in source_audit_inputs():
            target = self.root/path.relative_to(ROOT)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, target)
        self.bridge = self.root/HERE.relative_to(ROOT)
        self.raw = self.root/'extraction'
        self.raw.mkdir()
        for name in ('drop-native.json', 'drop-borrowck.json', 'receipt.json'):
            shutil.copyfile(RAW/name, self.raw/name)
        self.freeze = dict(before_first_trial=True, files={
            str(p.relative_to(ROOT)): digest(p) for p in source_audit_inputs()})
        self.serial = 0

    def run_entry(self, tool, freeze, expected_failure=None):
        self.serial += 1
        path = self.root/f'freeze-{self.serial}.json'
        path.write_text(json.dumps(freeze))
        output = self.root/f'run-{self.serial}'
        result = subprocess.run([sys.executable, '-B', '-O', str(self.bridge/tool),
            '--freeze', str(path), '--extraction', str(self.raw), '--output', str(output)],
            text=True, capture_output=True, timeout=30)
        receipt = json.loads((output/'receipt.json').read_text())
        self.assertIs(receipt['publication'], False)
        self.assertIs(receipt['native_bridge_proved'], False)
        if expected_failure is None:
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIs(receipt['success'], True)
            for path in source_audit_inputs():
                self.assertIn(str(self.root/path.relative_to(ROOT)), receipt['artifacts'])
        else:
            self.assertNotEqual(result.returncode, 0)
            self.assertIs(receipt['success'], False)
            self.assertIn(expected_failure, receipt['failure']['message'])
            self.assertFalse((output/'source-obligations.json').exists())
            self.assertNotIn('healthy_audit', receipt)

    def test_complete_freeze_both_entry_points(self):
        for tool in ('source_map.py', 'audit.py'):
            with self.subTest(tool=tool):
                self.run_entry(tool, self.freeze)

    def test_each_mandatory_input_omission_rejects_before_audit(self):
        for name in self.freeze['files']:
            frozen = copy.deepcopy(self.freeze)
            del frozen['files'][name]
            for tool in ('source_map.py', 'audit.py'):
                with self.subTest(name=name, tool=tool):
                    self.run_entry(tool, frozen, 'incomplete frozen source-audit inputs')

    def test_post_freeze_synthetic_receipt_and_omitted_binding_reject(self):
        frozen = copy.deepcopy(self.freeze)
        binding = self.bridge/'source-bindings.json'
        del frozen['files'][str(binding.relative_to(self.root))]
        # The omitted freeze exists before the private binding is replaced.
        pretrial = self.root/'pretrial.json'
        pretrial.write_text(json.dumps(frozen))
        (self.raw/'receipt.json').write_text(json.dumps(dict(success=True)))
        pin = json.loads(binding.read_text())
        pin['receipt_sha256'] = digest(self.raw/'receipt.json')
        binding.write_text(json.dumps(pin))
        for tool in ('source_map.py', 'audit.py'):
            with self.subTest(tool=tool):
                self.run_entry(tool, json.loads(pretrial.read_text()),
                               'incomplete frozen source-audit inputs')
                self.run_entry(tool, self.freeze, 'frozen source changed')


if __name__ == '__main__':
    unittest.main()
