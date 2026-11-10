"""Regression checks for source-review findings; not theorem-proof evidence."""
import copy
import json
from pathlib import Path
import sys
import subprocess
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from acceptance_inputs import accept_model_result, recorded_model_sources
from infer_eval_support import ArtifactChanged, ArtifactGuard, digest, support_artifacts
from proof_checks import NAMES

RECEIPT = Path('/tmp/verus-infer-buffer-cycle-20261109/semantic-guard-validation-1efbd004/proofs/receipt.json')


class RecordedSources(unittest.TestCase):
    def setUp(self):
        self.receipt = json.loads(RECEIPT.read_text())

    def test_complete_real_receipt(self):
        expected = {m: digest(HERE/'candidates/loan-v12'/(m+'.v')) for m in NAMES}
        self.assertEqual(recorded_model_sources(self.receipt), expected)

    def test_missing_reordered_or_duplicate_modules(self):
        for modules in ([], list(NAMES)[:-1], list(reversed(NAMES)), [*NAMES, 'raw_view']):
            r = copy.deepcopy(self.receipt);r['modules'] = modules
            with self.subTest(modules=modules), self.assertRaises(ValueError):
                recorded_model_sources(r)
        del self.receipt['proof_terms']['raw_view']
        with self.assertRaises(ValueError):
            recorded_model_sources(self.receipt)

    def test_missing_and_duplicate_source_identities(self):
        name = next(n for n in self.receipt['proof_artifacts'] if Path(n).name == 'raw_view.v')
        r = copy.deepcopy(self.receipt);del r['proof_artifacts'][name]
        with self.assertRaises(ValueError):
            recorded_model_sources(r)
        self.receipt['proof_artifacts']['another/raw_view.v'] = self.receipt['proof_artifacts'][name]
        with self.assertRaises(ValueError):
            recorded_model_sources(self.receipt)

    def test_malformed_hash_identity(self):
        name = next(n for n in self.receipt['proof_artifacts'] if Path(n).name == 'raw_view.v')
        for value in ('', 'x'*64, 0):
            r = copy.deepcopy(self.receipt);r['proof_artifacts'][name]['sha256'] = value
            with self.subTest(value=value), self.assertRaises(ValueError):
                recorded_model_sources(r)


class FinalStatus(unittest.TestCase):
    def test_failed_final_guard_keeps_false_status(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp)/'input';path.write_text('before')
            guard = ArtifactGuard([path]);path.write_text('after')
            receipt = dict(model_proofs_kernel_checked=False, publication=False)
            with self.assertRaises(ArtifactChanged):
                accept_model_result(receipt, dict(model_proofs_kernel_checked=True), [guard])
            self.assertEqual(receipt, dict(model_proofs_kernel_checked=False, publication=False))

    def test_all_final_guards_must_pass(self):
        with tempfile.TemporaryDirectory() as temp:
            a, b = Path(temp)/'a', Path(temp)/'b'
            a.write_text('a');b.write_text('b')
            guards = [ArtifactGuard([a]), ArtifactGuard([b])];b.write_text('changed')
            receipt = dict(model_proofs_kernel_checked=False)
            with self.assertRaises(ArtifactChanged):
                accept_model_result(receipt, dict(model_proofs_kernel_checked=True), guards)
            self.assertIs(receipt['model_proofs_kernel_checked'], False)

    def test_success_and_missing_guard(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp)/'input';path.write_text('stable')
            receipt = dict(model_proofs_kernel_checked=False)
            with self.assertRaises(ValueError):
                accept_model_result(receipt, dict(model_proofs_kernel_checked=True), [])
            self.assertIs(receipt['model_proofs_kernel_checked'], False)
            accept_model_result(receipt, dict(model_proofs_kernel_checked=True), [ArtifactGuard([path])])
            self.assertIs(receipt['model_proofs_kernel_checked'], True)


class RequiredSupport(unittest.TestCase):
    def test_checker_refuses_each_omitted_support_before_dependency_setup(self):
        root = HERE.parents[3]
        frozen = json.loads((HERE/'model-source-status-freeze.json').read_text())
        with tempfile.TemporaryDirectory() as temp:
            for i, support in enumerate(support_artifacts()):
                selected = copy.deepcopy(frozen)
                del selected['files'][str(support.relative_to(root))]
                path = Path(temp)/f'freeze-{i}.json'
                path.write_text(json.dumps(selected))
                out = Path(temp)/f'out-{i}'
                cmd = [sys.executable, '-B', *(['-O'] if sys.flags.optimize else []),
                       str(HERE/'check.py'), '--freeze', str(path), '--ready',
                       str(Path(temp)/'unused-ready.json'), '--output', str(out)]
                result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
                receipt = json.loads((out/'receipt.json').read_text())
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertIn('incomplete frozen trust inputs', receipt['failure']['message'])
                self.assertEqual(receipt['commands'], [])
                self.assertIs(receipt['model_proofs_kernel_checked'], False)
                self.assertIs(receipt['success'], False)
                self.assertIs(receipt['publication'], False)


if __name__ == '__main__':
    unittest.main()
