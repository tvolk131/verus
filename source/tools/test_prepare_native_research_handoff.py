import gzip
import hashlib
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import prepare_native_research_handoff as handoff


class NativeResearchHandoffTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / "clone"
        self.root.mkdir()
        self.inputs = self.root / "inputs"
        self.inputs.mkdir()
        self.manifest_path = self.root / handoff.MANIFEST
        self.manifest_path.parent.mkdir(parents=True)
        self.manifest = dict(format=1, inputs=[], portable_runtime_patch=False)

    def save(self):
        self.manifest_path.write_text(json.dumps(self.manifest))

    def add_input(self, name, kind, destination, content):
        path = self.inputs / name
        path.write_bytes(content)
        entry = dict(id=name, path=str(path.relative_to(self.root)), kind=kind,
                     destination=destination, bytes=len(content),
                     sha256=hashlib.sha256(content).hexdigest())
        self.manifest["inputs"].append(entry)
        return entry

    def add_tar(self, members):
        buffer = io.BytesIO()
        with tarfile.open(fileobj=buffer, mode="w:gz") as saved:
            for name, kind, content in members:
                member = tarfile.TarInfo(name)
                member.type = kind
                member.size = len(content) if kind == tarfile.REGTYPE else 0
                member.linkname = "outside" if kind in {tarfile.SYMTYPE, tarfile.LNKTYPE} else ""
                saved.addfile(member, io.BytesIO(content) if member.isfile() else None)
        return self.add_input("snapshot.tar.gz", "tar", "snapshot", buffer.getvalue())

    def test_clean_preparation_needs_only_declared_clone_inputs(self):
        self.add_tar([("source/main.rs", tarfile.REGTYPE, b"fn main() {}")])
        raw = b"retained proof artifact"
        entry = self.add_input("proof.gz", "gzip", "proof/vstd.vir", gzip.compress(raw))
        entry["uncompressed"] = dict(bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest())
        self.save()
        output = self.root.parent / "prepared"
        result = handoff.prepare(self.root, output)
        self.assertEqual((output / "proof/vstd.vir").read_bytes(), raw)
        self.assertEqual((output / "snapshot/source/main.rs").read_bytes(), b"fn main() {}")
        self.assertEqual(result["all18"], "OPEN")
        self.assertFalse(result["compiler_solver_or_application_executed"])
        self.assertEqual(json.loads((output / "PREPARED.json").read_text()), result)

    def test_missing_or_changed_input_refuses(self):
        entry = self.add_input("source.txt", "file", "source.txt", b"original")
        self.save()
        path = self.root / entry["path"]
        path.write_bytes(b"modified")
        with self.assertRaisesRegex(ValueError, "identity mismatch"):
            handoff.verify(self.root)
        path.unlink()
        with self.assertRaisesRegex(ValueError, "missing regular input"):
            handoff.verify(self.root)

    def test_input_symlink_refuses_even_when_bytes_match(self):
        entry = self.add_input("source.txt", "file", "source.txt", b"original")
        self.save()
        path = self.root / entry["path"]
        outside = self.root.parent / "outside.txt"
        outside.write_bytes(path.read_bytes())
        path.unlink()
        path.symlink_to(outside)
        with self.assertRaisesRegex(ValueError, "symlink input"):
            handoff.verify(self.root)

    def test_existing_destination_is_not_modified(self):
        self.save()
        output = self.root.parent / "prepared"
        output.mkdir()
        marker = output / "keep"
        marker.write_text("original")
        with self.assertRaisesRegex(ValueError, "must not exist"):
            handoff.prepare(self.root, output)
        self.assertEqual(marker.read_text(), "original")

    def test_unsafe_portable_paths_refuse(self):
        for value in ["", "/absolute", "../outside", "a/../b", "./a", "a//b", "a\\b", "C:/outside"]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                handoff.relative_path(value)

    def test_archive_traversal_links_and_special_files_refuse_before_output(self):
        for name, kind in [("../escape", tarfile.REGTYPE), ("/escape", tarfile.REGTYPE),
                           ("link", tarfile.SYMTYPE), ("hard", tarfile.LNKTYPE),
                           ("device", tarfile.CHRTYPE)]:
            with self.subTest(name=name):
                self.manifest["inputs"] = []
                self.add_tar([(name, kind, b"")])
                self.save()
                output = self.root.parent / "prepared"
                with self.assertRaises(ValueError):
                    handoff.prepare(self.root, output)
                self.assertFalse(output.exists())

    def test_duplicate_archive_members_refuse(self):
        self.add_tar([("same", tarfile.REGTYPE, b"first"), ("same", tarfile.REGTYPE, b"second")])
        self.save()
        with self.assertRaisesRegex(ValueError, "duplicate archive"):
            handoff.prepare(self.root, self.root.parent / "prepared")

    def test_decompressed_identity_and_size_are_checked(self):
        entry = self.add_input("proof.gz", "gzip", "proof", gzip.compress(b"real"))
        entry["uncompressed"] = dict(bytes=4, sha256=hashlib.sha256(b"fake").hexdigest())
        self.save()
        with self.assertRaisesRegex(ValueError, "identity mismatch"):
            handoff.verify(self.root)
        entry["uncompressed"]["bytes"] = 1
        self.save()
        with self.assertRaisesRegex(ValueError, "exceeds declared bound"):
            handoff.verify(self.root)

    def test_archive_and_combined_output_limits_refuse(self):
        self.add_tar([("source", tarfile.REGTYPE, b"large")])
        self.save()
        with patch.object(handoff, "MAX_OUTPUT_BYTES", 3), self.assertRaisesRegex(ValueError, "archive output bound"):
            handoff.prepare(self.root, self.root.parent / "prepared")
        self.manifest["inputs"] = []
        self.add_input("first", "file", "first", b"123")
        self.add_input("second", "file", "second", b"456")
        self.save()
        with patch.object(handoff, "MAX_OUTPUT_BYTES", 5), self.assertRaisesRegex(ValueError, "combined preparation"):
            handoff.prepare(self.root, self.root.parent / "prepared")

    def test_runtime_path_adapter_is_explicit_and_recorded(self):
        original = ("let path = " + handoff.OLD_RUNTIME + ";").encode()
        self.add_input("source.rs", "file", handoff.RUNTIME_SOURCE, original)
        self.manifest["portable_runtime_patch"] = True
        self.save()
        output = self.root.parent / "prepared"
        result = handoff.prepare(self.root, output)
        self.assertIn(handoff.NEW_RUNTIME, (output / handoff.RUNTIME_SOURCE).read_text())
        adapter = result["runtime_path_adapter"]
        self.assertEqual(adapter["before_sha256"], hashlib.sha256(original).hexdigest())
        self.assertEqual(adapter["after_sha256"], handoff.digest(output / handoff.RUNTIME_SOURCE))
        self.assertIn("UNCOMPILED_UNREVIEWED", adapter["status"])

    def test_runtime_adapter_precondition_cannot_silently_skip(self):
        self.add_input("source.rs", "file", handoff.RUNTIME_SOURCE, b"unrelated")
        self.manifest["portable_runtime_patch"] = True
        self.save()
        output = self.root.parent / "prepared"
        with self.assertRaisesRegex(ValueError, "precondition mismatch"):
            handoff.prepare(self.root, output)
        self.assertFalse((output / "PREPARED.json").exists())


if __name__ == "__main__":
    unittest.main()
