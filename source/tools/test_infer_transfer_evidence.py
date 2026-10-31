"""Offline capture checks, discovered by the existing infer-tooling CI job."""

import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import check_infer_transfer_evidence as checker


class TransferEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "capture"
        shutil.copytree(checker.DEFAULT_ROOT, self.root)

    def change_json(self, relative, change):
        path = self.root / relative
        data = json.loads(path.read_text())
        change(data)
        path.write_text(json.dumps(data))

    def test_frozen_capture_passes_without_writes_or_subprocesses(self):
        before = {path.relative_to(self.root): checker.digest(path)
                  for path in self.root.rglob("*") if path.is_file()}
        with patch("subprocess.run", side_effect=AssertionError("must stay offline")):
            result = checker.check_capture(self.root)
        self.assertEqual(result["command_receipts"], 62)
        self.assertEqual(result["application_frontend_rejections"], 16)
        after = {path.relative_to(self.root): checker.digest(path)
                 for path in self.root.rglob("*") if path.is_file()}
        self.assertEqual(before, after)

    def test_changed_file_is_rejected(self):
        path = self.root / "raw/native-tests.stdout"
        path.write_bytes(path.read_bytes().replace(b"3 passed", b"0 passed"))
        with self.assertRaisesRegex(ValueError, "hash mismatch"):
            checker.check_capture(self.root)

    def test_missing_file_is_rejected(self):
        (self.root / "upstream/LICENSE").unlink()
        with self.assertRaises(OSError):
            checker.check_capture(self.root)

    def test_extra_file_is_rejected(self):
        (self.root / "unrecorded.txt").write_text("not frozen")
        with self.assertRaisesRegex(ValueError, "unexpected capture files"):
            checker.check_capture(self.root)

    def test_symlink_is_rejected(self):
        original = self.root / "upstream/LICENSE"
        copy = Path(self.temporary.name) / "license-copy"
        shutil.copyfile(original, copy)
        original.unlink()
        original.symlink_to(copy)
        with self.assertRaisesRegex(ValueError, "symlink"):
            checker.check_capture(self.root)

    def test_rehashed_index_cannot_redefine_capture(self):
        path = self.root / "README.md"
        path.write_bytes(path.read_bytes() + b"\nNot the historical report.\n")

        def update_index(index):
            for entry in index["entries"]:
                if entry["path"] == "README.md":
                    entry.update(bytes=path.stat().st_size, sha256=checker.digest(path))

        self.change_json("evidence-index.json", update_index)
        with self.assertRaisesRegex(ValueError, "frozen index changed"):
            checker.check_capture(self.root)

    def test_receipt_output_hash_is_checked(self):
        (self.root / "raw/native-tests.stdout").write_text("forged result")
        with self.assertRaisesRegex(ValueError, "stdout receipt mismatch"):
            checker.check_receipts(self.root)

    def test_incomplete_command_is_rejected(self):
        self.change_json("raw/native-tests.json", lambda receipt: receipt.update(exit_status=None))
        with self.assertRaisesRegex(ValueError, "incomplete command"):
            checker.check_receipts(self.root)

    def test_upstream_snapshot_is_checked_against_archive(self):
        path = self.root / "upstream/LICENSE"
        path.write_text("a different license")
        with self.assertRaisesRegex(ValueError, "upstream snapshot mismatch"):
            checker.check_source(self.root)

    def test_false_proof_success_cannot_replace_frontend_failure(self):
        self.change_json("raw/ordinary-plain-cold.stdout",
                         lambda report: report["verification-results"].update(success=True))
        with self.assertRaisesRegex(ValueError, "differs from raw report"):
            checker.check_outcomes(self.root)

    def test_optional_sanity_obligation_is_rejected(self):
        def make_optional(report):
            report["verus-infer"]["functions"][0]["proofs"]["exactness"]["required"] = False

        self.change_json("raw/engine-sanity-warm.stdout", make_optional)
        with self.assertRaisesRegex(ValueError, "fresh required proofs"):
            checker.check_outcomes(self.root)

    def test_optimized_python_still_fails_explicitly(self):
        self.change_json("evidence-index.json", lambda index: index.update(schema=999))
        process = subprocess.run([sys.executable, "-B", "-O", checker.__file__, "--root", str(self.root)],
                                 capture_output=True, text=True, check=False)
        self.assertEqual(process.returncode, 1)
        self.assertIn("frozen index changed", process.stderr)

    def test_separate_publication_replay_receipts(self):
        replay = checker.DEFAULT_ROOT.with_name("verus-infer-transfer-v1-publication-replay")
        self.assertEqual(checker.check_publication_replay(replay, self.root), 16)

    def test_publication_replay_cannot_claim_different_binary_hashes(self):
        replay = Path(self.temporary.name) / "replay"
        shutil.copytree(checker.DEFAULT_ROOT.with_name("verus-infer-transfer-v1-publication-replay"), replay)
        path = replay / "replay.json"
        receipt = json.loads(path.read_text())
        receipt["artifact_sha256"]["verus"] = "0" * 64
        path.write_text(json.dumps(receipt))
        with self.assertRaisesRegex(ValueError, "artifact_sha256 differs"):
            checker.check_publication_replay(replay, self.root)


if __name__ == "__main__":
    unittest.main()
