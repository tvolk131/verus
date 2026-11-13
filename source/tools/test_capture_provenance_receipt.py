"""Hermetic receipt tests: Python subprocesses and fake compiler/artifact bytes only."""

import copy
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import threading
import unittest
from unittest import mock

import capture_provenance_receipt as provenance


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class ProvenanceReceiptTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="receipt-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.sources = self.root / "inputs"
        self.sources.mkdir()
        (self.sources / "engine.rs").write_text("fake source, never compiled\n")
        self.compiler = self.root / "fake-compiler"
        self.compiler.write_bytes(b"not an executable compiler\n")
        self.artifact = self.root / "artifact.bin"
        self.output = self.root / "receipt"
        self.executable = Path(sys.executable).resolve()
        self.plan = {
            "schema": provenance.PLAN_SCHEMA, "cwd": str(self.root),
            "argv": [str(self.executable), "-I", "-S", "-c",
                     f"from pathlib import Path; Path({str(self.artifact)!r}).write_bytes(b'fake artifact')"],
            "executable_sha256": digest(self.executable),
            "compiler": {"path": str(self.compiler), "sha256": digest(self.compiler)},
            "inventory": {"root": str(self.root), "files": [], "trees": ["inputs"],
                          "expected_sha256": {"inputs/engine.rs": digest(self.sources / "engine.rs")}},
            "outputs": [str(self.artifact)], "timeout_seconds": 5,
        }

    def run_capture(self, code=None):
        if code is not None:
            self.plan["argv"][-1] = code
        receipt = provenance.capture(self.plan, self.output)
        self.assertEqual(receipt, json.loads((self.output / "receipt.json").read_text()))
        return receipt

    def assert_refused(self, receipt, fragment=None, started=False):
        self.assertFalse(receipt["success"])
        self.assertEqual(receipt["command"]["started"], started)
        self.assertTrue(receipt["errors"])
        if fragment is not None:
            self.assertIn(fragment, "\n".join(receipt["errors"]))

    def test_success_records_separate_identities_without_correspondence_claim(self):
        receipt = self.run_capture()
        self.assertTrue(receipt["success"], receipt["errors"])
        self.assertEqual(receipt["inputs_before"], receipt["inputs_after"])
        self.assertEqual(receipt["compiler_before"]["sha256"], digest(self.compiler))
        self.assertEqual(receipt["executable_before"]["sha256"], digest(self.executable))
        self.assertEqual(receipt["outputs"][str(self.artifact)]["sha256"], digest(self.artifact))
        self.assertEqual(receipt["input_drift"], [])
        self.assertGreater(receipt["command"]["launch_time_ns"], 0)
        self.assertEqual(receipt["invocation_kind"], "other-command")
        self.assertFalse(receipt["compiler_directly_named_as_executable"])
        self.assertTrue(all(value is False for value in receipt["establishes"].values()))

    def test_selected_scope_excludes_unselected_files_honestly(self):
        outside = self.root / "unselected.rs"
        outside.write_text("before")
        receipt = self.run_capture(self.plan["argv"][-1] + f"; Path({str(outside)!r}).write_text('after')")
        self.assertTrue(receipt["success"], receipt["errors"])
        self.assertEqual(set(receipt["inputs_before"]), {"inputs/engine.rs"})
        self.assertIn("declared", receipt["inventory_scope"])
        self.assertFalse(receipt["establishes"]["runtime_inventory_completeness"])

    def test_explicit_file_selection_is_supported(self):
        self.plan["inventory"].update(files=["inputs/engine.rs"], trees=[])
        self.assertTrue(self.run_capture()["success"])

    def test_missing_output_fails_after_zero_exit(self):
        receipt = self.run_capture("pass")
        self.assert_refused(receipt, "output capture failed", started=True)
        self.assertEqual(receipt["command"]["exit_code"], 0)

    def test_stale_output_timestamp_fails(self):
        receipt = self.run_capture(self.plan["argv"][-1] +
                                  f"; import os; os.utime({str(self.artifact)!r}, ns=(1, 1))")
        self.assert_refused(receipt, "stale output", started=True)

    def test_seeded_output_is_preserved_and_command_not_started(self):
        self.artifact.write_bytes(b"seeded")
        receipt = self.run_capture()
        self.assert_refused(receipt, "seeded output")
        self.assertEqual(self.artifact.read_bytes(), b"seeded")
        self.assertFalse((self.output / "command.log").exists())

    def test_dangling_seeded_symlink_is_refused(self):
        self.artifact.symlink_to(self.root / "missing")
        self.assert_refused(self.run_capture(), "seeded output")

    def test_all_outputs_checked_before_command(self):
        seeded = self.root / "second.bin"
        seeded.write_bytes(b"seeded")
        self.plan["outputs"].append(str(seeded))
        self.assert_refused(self.run_capture(), "seeded output")
        self.assertFalse(self.artifact.exists())

    def test_output_aliasing_input_is_refused(self):
        self.plan["outputs"] = [str(self.compiler)]
        self.assert_refused(self.run_capture(), "overlaps")

    def test_source_drift_content_is_retained_and_refused(self):
        changed = self.sources / "engine.rs"
        receipt = self.run_capture(self.plan["argv"][-1] + f"; Path({str(changed)!r}).write_text('changed')")
        self.assert_refused(receipt, "selected inputs drifted", started=True)
        self.assertEqual(receipt["input_drift"], ["inputs/engine.rs"])

    def test_recursive_inventory_addition_is_drift(self):
        added = self.sources / "added.rs"
        receipt = self.run_capture(self.plan["argv"][-1] + f"; Path({str(added)!r}).write_text('added')")
        self.assert_refused(receipt, "selected inputs drifted", started=True)
        self.assertEqual(receipt["input_drift"], ["inputs/added.rs"])

    def test_recursive_inventory_removal_is_drift(self):
        removed = self.sources / "engine.rs"
        receipt = self.run_capture(self.plan["argv"][-1] + f"; Path({str(removed)!r}).unlink()")
        self.assert_refused(receipt, "selected inputs drifted", started=True)
        self.assertEqual(receipt["input_drift"], ["inputs/engine.rs"])

    def test_explicit_file_removal_fails_rescan(self):
        self.plan["inventory"].update(files=["inputs/engine.rs"], trees=[])
        removed = self.sources / "engine.rs"
        receipt = self.run_capture(self.plan["argv"][-1] + f"; Path({str(removed)!r}).unlink()")
        self.assert_refused(receipt, "input rescan failed", started=True)

    def test_initial_missing_or_stale_source_refuses_before_launch(self):
        (self.sources / "engine.rs").write_text("stale initial source")
        self.assert_refused(self.run_capture(), "initial selected-input mismatch")

    def test_initial_inventory_omission_refuses_before_launch(self):
        (self.sources / "not-pinned.rs").write_text("unlisted")
        self.assert_refused(self.run_capture(), "initial selected-input mismatch")

    def test_initial_missing_tree_refuses_before_launch(self):
        self.plan["inventory"]["trees"] = ["missing-tree"]
        self.assert_refused(self.run_capture())

    def test_source_symlink_is_refused(self):
        (self.sources / "alias.rs").symlink_to(self.compiler)
        self.assert_refused(self.run_capture(), "symlink")

    def test_source_tree_symlink_is_refused(self):
        (self.sources / "linked-directory").symlink_to(self.root, target_is_directory=True)
        self.assert_refused(self.run_capture(), "symlink")

    def test_unreadable_selected_subtree_cannot_be_silently_omitted(self):
        hidden = self.sources / "unreadable"
        hidden.mkdir()
        (hidden / "unpinned.rs").write_text("must not be silently omitted")
        original_scandir = os.scandir

        def guarded_scandir(directory):
            if os.fspath(directory) == str(hidden):
                raise PermissionError("synthetic denied selected directory")
            return original_scandir(directory)

        with mock.patch.object(os, "scandir", side_effect=guarded_scandir):
            self.assert_refused(self.run_capture(), "PermissionError")

    def test_output_symlink_and_directory_are_not_regular_artifacts(self):
        for kind in ("symlink", "directory"):
            with self.subTest(kind=kind):
                artifact = self.root / kind
                self.plan["outputs"] = [str(artifact)]
                self.output = self.root / ("receipt-" + kind)
                operation = f"Path({str(artifact)!r}).mkdir()" if kind == "directory" else \
                    f"Path({str(artifact)!r}).symlink_to({str(self.compiler)!r})"
                receipt = self.run_capture("from pathlib import Path; " + operation)
                self.assert_refused(receipt, "output capture failed", started=True)

    def test_command_failure_retains_logs_outputs_and_source_drift(self):
        changed = self.sources / "engine.rs"
        receipt = self.run_capture(self.plan["argv"][-1] +
            f"; Path({str(changed)!r}).write_text('changed'); print('harmless failure'); raise SystemExit(7)")
        self.assert_refused(receipt, "command failed", started=True)
        self.assertEqual(receipt["command"]["exit_code"], 7)
        self.assertIn("inputs/engine.rs", receipt["input_drift"])
        self.assertIn(str(self.artifact), receipt["outputs"])
        self.assertIn("harmless failure", (self.output / "command.log").read_text())

    def test_timeout_retains_partial_output_and_fails(self):
        self.plan["timeout_seconds"] = 0.2
        receipt = self.run_capture(self.plan["argv"][-1] + "; import time; time.sleep(10)")
        self.assert_refused(receipt, "timed out", started=True)
        self.assertTrue(receipt["command"]["timed_out"])
        self.assertNotEqual(receipt["command"]["exit_code"], 0)

    def test_nonexecutable_spawn_error_is_retained(self):
        self.plan["argv"][0] = str(self.compiler)
        self.plan["executable_sha256"] = digest(self.compiler)
        self.assert_refused(self.run_capture(), "PermissionError")

    def test_compiler_hash_mismatch_refuses_before_launch(self):
        self.plan["compiler"]["sha256"] = "0" * 64
        self.assert_refused(self.run_capture(), "compiler identity mismatch")

    def test_missing_declared_compiler_refuses_before_launch(self):
        self.compiler.unlink()
        self.assert_refused(self.run_capture(), "FileNotFoundError")

    def test_missing_executable_refuses_before_launch(self):
        self.plan["argv"][0] = str(self.root / "missing-runner")
        self.assert_refused(self.run_capture(), "FileNotFoundError")

    def test_compiler_drift_refuses_after_launch(self):
        receipt = self.run_capture(self.plan["argv"][-1] +
                                  f"; Path({str(self.compiler)!r}).write_bytes(b'drift')")
        self.assert_refused(receipt, "compiler identity drift", started=True)

    def test_executable_hash_mismatch_refuses_before_launch(self):
        self.plan["executable_sha256"] = "0" * 64
        self.assert_refused(self.run_capture(), "executable identity mismatch")

    def test_compiler_identity_can_be_absent_without_inventing_one(self):
        self.plan["compiler"] = None
        receipt = self.run_capture()
        self.assertTrue(receipt["success"])
        self.assertIsNone(receipt["compiler_before"])
        self.assertIsNone(receipt["compiler_after"])

    def test_environment_secrets_are_not_inherited_or_recorded(self):
        code = self.plan["argv"][-1] + "; import os; assert 'PROVENANCE_TEST_TOKEN' not in os.environ"
        with mock.patch.dict(os.environ, {"PROVENANCE_TEST_TOKEN": "fake-secret-canary"}):
            receipt = self.run_capture(code)
        self.assertTrue(receipt["success"], receipt["errors"])
        self.assertEqual(receipt["environment"], {"LANG": "C", "LC_ALL": "C", "TZ": "UTC"})
        self.assertNotIn("fake-secret-canary", (self.output / "receipt.json").read_text())
        self.assertNotIn("fake-secret-canary", (self.output / "command.log").read_text())

    def test_vstd_only_is_never_classified_as_engine_build(self):
        for executable in ("rust_verify", "cargo"):
            argv = ["/fake/" + executable, "build", "-p", "rust_verify", "--is-vstd", "--compile"]
            self.assertEqual(provenance.invocation_kind(argv), "vstd-only-command-shape")
        self.assertEqual(provenance.invocation_kind(
            ["/fake/cargo", "build", "--offline", "--locked", "-p", "rust_verify"]),
            "engine-build-command-shape")
        self.assertEqual(provenance.invocation_kind(
            ["/fake/cargo", "test", "-p", "rust_verify"]), "other-command")

    def test_classifier_does_not_promote_arbitrary_build_labels(self):
        for argv in (["/fake/runner", "engine-build"],
                     ["/fake/cargo", "build", "-p", "vstd"],
                     ["/fake/cargo", "build"],
                     ["/fake/runner", "--compile"]):
            with self.subTest(argv=argv):
                self.assertEqual(provenance.invocation_kind(argv), "other-command")

    def test_engine_shape_is_not_compiler_build_or_source_attestation(self):
        for selector in (["-p", "rust_verify"], ["--package", "rust_verify"],
                         ["--package=rust_verify"]):
            with self.subTest(selector=selector):
                self.assertEqual(provenance.invocation_kind(["/fake/cargo", "build", *selector]),
                                 "engine-build-command-shape")
        self.assertNotIn("compiler_builds", self.run_capture())

    def test_symlinked_log_cannot_result_in_success_or_drop_failure_receipt(self):
        log = self.output / "command.log"
        receipt = self.run_capture(self.plan["argv"][-1] +
            f"; Path({str(log)!r}).unlink(); Path({str(log)!r}).symlink_to({str(self.compiler)!r})")
        self.assert_refused(receipt, "log capture failed", started=True)

    def test_missing_log_cannot_result_in_success(self):
        log = self.output / "command.log"
        receipt = self.run_capture(self.plan["argv"][-1] + f"; Path({str(log)!r}).unlink()")
        self.assert_refused(receipt, "log capture failed", started=True)

    @unittest.skipUnless(os.name == "posix", "SIGINT process-group control requires POSIX")
    def test_keyboard_interrupt_records_failure_and_reaps_child(self):
        original_popen = subprocess.Popen
        processes = []

        def tracked_popen(*arguments, **keywords):
            process = original_popen(*arguments, **keywords)
            processes.append(process)
            return process

        self.plan["argv"][-1] += "; import time; time.sleep(10)"
        timer = threading.Timer(0.4, lambda: os.kill(os.getpid(), signal.SIGINT))
        timer.daemon = True
        timer.start()
        try:
            with mock.patch.object(provenance.subprocess, "Popen", side_effect=tracked_popen):
                with self.assertRaises(KeyboardInterrupt):
                    self.run_capture()
            receipt = json.loads((self.output / "receipt.json").read_text())
            self.assert_refused(receipt, "command interrupted: KeyboardInterrupt", started=True)
            self.assertTrue(receipt["command"]["interrupted"])
            self.assertIsNotNone(receipt["command"]["exit_code"])
            self.assertIsNotNone(processes[0].poll())
        finally:
            timer.cancel()
            timer.join(timeout=2)
            for process in processes:
                if process.poll() is None:
                    os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=2)

    def test_system_exit_records_failure_reaps_child_and_preserves_exit_code(self):
        original_popen = subprocess.Popen
        processes = []

        def interrupted_popen(*arguments, **keywords):
            process = original_popen(*arguments, **keywords)
            original_wait = process.wait

            def interrupted_wait(*wait_arguments, **wait_keywords):
                process.wait = original_wait
                raise SystemExit(23)

            process.wait = interrupted_wait
            processes.append(process)
            return process

        self.plan["argv"][-1] += "; import time; time.sleep(10)"
        try:
            with mock.patch.object(provenance.subprocess, "Popen", side_effect=interrupted_popen):
                with self.assertRaises(SystemExit) as interruption:
                    self.run_capture()
            self.assertEqual(interruption.exception.code, 23)
            receipt = json.loads((self.output / "receipt.json").read_text())
            self.assert_refused(receipt, "command interrupted: SystemExit", started=True)
            self.assertTrue(receipt["command"]["interrupted"])
            self.assertIsNotNone(receipt["command"]["exit_code"])
            self.assertIsNotNone(processes[0].poll())
        finally:
            for process in processes:
                if process.poll() is None:
                    process.kill()
                process.wait(timeout=2)

    @unittest.skipUnless(os.name == "posix", "process-group timeout control requires POSIX")
    def test_timeout_kills_waiting_child_in_same_process_group(self):
        marker = self.root / "child-marker"
        child = f"import time; from pathlib import Path; time.sleep(0.8); Path({str(marker)!r}).write_text('escaped')"
        self.plan["timeout_seconds"] = 0.2
        receipt = self.run_capture(self.plan["argv"][-1] +
            f"; import subprocess, sys, time; subprocess.Popen([sys.executable, '-I', '-S', '-c', {child!r}]); time.sleep(10)")
        self.assert_refused(receipt, "timed out", started=True)
        import time
        time.sleep(1)
        self.assertFalse(marker.exists())

    def test_existing_receipt_directory_is_never_reused(self):
        self.output.mkdir()
        (self.output / "sentinel").write_text("preserve")
        with self.assertRaises(FileExistsError):
            provenance.capture(self.plan, self.output)
        self.assertEqual((self.output / "sentinel").read_text(), "preserve")

    def test_malformed_plans_raise_before_creating_receipt_or_running_command(self):
        mutations = [
            lambda plan: plan.update(schema="unknown"),
            lambda plan: plan.update(extra=True),
            lambda plan: plan.pop("compiler"),
            lambda plan: plan.update(argv=[]),
            lambda plan: plan.update(argv="/bin/echo"),
            lambda plan: plan.update(argv=["relative"]),
            lambda plan: plan.update(argv=[str(self.executable), True]),
            lambda plan: plan.update(argv=[str(self.executable), "bad\0argument"]),
            lambda plan: plan.update(timeout_seconds=True),
            lambda plan: plan.update(timeout_seconds=0),
            lambda plan: plan.update(timeout_seconds=float("nan")),
            lambda plan: plan.update(timeout_seconds=float("inf")),
            lambda plan: plan.update(timeout_seconds=10 ** 1000),
            lambda plan: plan.update(outputs=[]),
            lambda plan: plan.update(outputs=[str(self.artifact)] * 2),
            lambda plan: plan.update(environment={"TOKEN": "must reject"}),
            lambda plan: plan["compiler"].update(extra="bad"),
            lambda plan: plan["inventory"].update(files=["../escape"]),
            lambda plan: plan["inventory"].update(files=["."]),
            lambda plan: plan["inventory"].update(trees=["inputs", "inputs"]),
            lambda plan: plan["inventory"].update(files=[], trees=[]),
            lambda plan: plan["inventory"].update(expected_sha256={}),
            lambda plan: plan["inventory"].update(expected_sha256={"/absolute": "0" * 64}),
            lambda plan: plan["inventory"].update(expected_sha256={"inputs/engine.rs": "NOT-A-HASH"}),
        ]
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                plan = copy.deepcopy(self.plan)
                mutate(plan)
                with self.assertRaises(ValueError):
                    provenance.capture(plan, self.output)
                self.assertFalse(self.output.exists())
                self.assertFalse(self.artifact.exists())
        for value in (None, [], True, "json-not-an-object"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                provenance.capture(value, self.output)


if __name__ == "__main__":
    unittest.main()
