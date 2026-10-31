#!/usr/bin/env python3
"""Replay runner regressions using disposable fake engines, never real applications."""
import ast
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest

from infer_eval_support import ArtifactChanged, ArtifactGuard, tool_artifacts
from infer_experiment import LoggedCommands


TOOLS = Path(__file__).resolve().parent
HEX = ("check_hex_adoption", "check_hex_iterator", "check_hex_vec_adoption")
RUNNERS = ("check_native_adoption", *HEX, "check_hex_vec_phase3")
SUPPORT = ("infer_experiment", "infer_eval_support")
CHECKED = (
    "Char::decode_nibble", "iter::hex_chars_to_byte", "iter::HexToBytesIter::new",
    "iter::HexToBytesIter::new_unchecked", "iter::HexToBytesIter::from_pairs",
    "iter::HexToBytesIter::next", "iter::HexToBytesIter::nth",
    "iter::HexDigitsIter::new_unchecked", "iter::HexDigitsIter::next",
    "iter::HexDigitsIter::nth", "iter::HexDigitsIter::size_hint",
    "iter::pair_decoding_goal", "iter::forward_step_goal", "iter::string_forward_goal",
    "iter::string_next_goal", "iter::original_error_location_goal",
    "iter::string_exhaustion_goal", "iter::saturated_error_position_goal",
    "iter::runtime_length_goal", "iter::forward_drain_domain_goal",
    "iter::HexToBytesIter::drain_to_vec", "iter::HexToBytesIter::size_hint", "decode_to_vec",
)
FAKE_COMMAND = r'''
import json, os, sys, time
from pathlib import Path
args = sys.argv[1:]
name = Path(sys.argv[0]).name
failure = os.environ.get("FAKE_FAILURE", "")
with Path(os.environ["FAKE_COMMAND_LOG"]).open("a") as stream:
    stream.write(json.dumps([name, *args]) + "\n")
if name == "git":
    print("fake-native-commit"); sys.exit(0)
if name == "rustc":
    print("fake-rustc-version"); sys.exit(0)
if name == "cargo":
    action = "clean" if "clean" in args else "native"
    print(action, flush=True)
    if failure == action + "_timeout": time.sleep(30)
    sys.exit(7 if failure == action else 0)
artifact = Path(args[args.index("--export") + 1]) if "--export" in args else None
label = artifact.stem if artifact else "native_proof"
if failure == "mutation_timeout" and label == "wrong_table":
    Path(os.environ["FAKE_READY"]).write_text(str(os.getpid()))
    print("mutation started", flush=True); time.sleep(30)
if failure == "malformed":
    print("not a report"); sys.exit(0)
if failure == "artifact":
    Path(__file__).with_name("rust_verify").write_text("changed tool")
if failure == "restoration" and label == "wrong_table":
    source = Path(args[0]); source.unlink(); source.mkdir()
    source.with_name("iter.rs").write_text("changed second source")
features = args[args.index("--features") + 1] if "--features" in args else ""
if name == "cargo-verus" and "--verify-function" in args:
    print("Partial verification must use `cargo verus focus`", file=sys.stderr); sys.exit(1)
success = not features and "--no-verify" not in args and "--verify-function" not in args
if artifact:
    success = label in ("fresh", "restored")
if artifact and (success or failure == "publication"):
    artifact.write_bytes(b"fake interface")
proofs = {key: dict(required=True, status="proved") for key in ("soundness", "maximality", "exactness")}
function = dict(function="hex_conservative::iter::hex_chars_to_byte", mode="exact",
                required_checks_passed=True, proofs=proofs)
if failure == "proof": function["proofs"]["soundness"]["status"] = "failed"
if failure == "candidate" and label == "wrong_table": function["required_checks_passed"] = False
scope = dict(native_collection_completed=True, complete_selected_scope_verified=success,
             checked_exec_bodies=20 if success else 0,
             native_functions=[dict(function="hex_conservative::" + item, body_status="checked_exec_body")
                               for item in json.loads(os.environ["FAKE_CHECKED"])])
if name == "cargo-verus":
    scope["native_functions"] = [dict(selected=False, rust_function=item) for item in ("Sum", "Encode")]
if failure == "scope": scope["complete_selected_scope_verified"] = False
inference = dict(crate_success=success, interface_written=success, verification_scope=scope,
                 functions=[function] * (10 if name == "cargo-verus" else 1),
                 sequence_pattern_assistance=[{}],
                 array_callback_assistance=[{}, {}], type_invariant_entry_assistance=[{}, {}, {}, {}])
if label == "blocked":
    diagnostics = ["::as_mut_ptr` is not supported", "`core::ptr::write` is not supported",
                   "::add` is not supported", "::set_len` is not supported", "aborting"]
    if failure == "diagnostics": diagnostics.pop()
    for diagnostic in diagnostics: print("error: " + diagnostic, file=sys.stderr)
elif not success:
    print("postcondition not satisfied; cannot use function; derived trait request requires one checked implementation; requires full-crate verification", file=sys.stderr)
print(json.dumps({"verification-results": {"success": success}, "verus-infer": inference}))
sys.exit(0 if success else 1)
'''


class ReplaySupportTests(unittest.TestCase):
    def test_acceptance_paths_have_no_runtime_assertions(self):
        for name in (*RUNNERS, *SUPPORT):
            with self.subTest(module=name):
                tree = ast.parse((TOOLS / (name + ".py")).read_text())
                self.assertFalse(any(isinstance(node, ast.Assert) for node in ast.walk(tree)))

    def test_portable_macro_discovery_and_missing_artifacts(self):
        for prefix, extension in (("lib", ".so"), ("lib", ".dylib"),
                                  ("lib", ".dll"), ("", ".dll")):
            with self.subTest(prefix=prefix, extension=extension), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                for name in ("verus", "z3", "rust_verify", "vstd.vir", "libvstd.rlib", "libverus_builtin.rlib"):
                    (root / name).write_bytes(b"stable")
                macros = [root / (prefix + stem + extension) for stem in
                          ("verus_builtin_macros", "verus_state_machines_macros")]
                for path in macros:
                    path.write_bytes(b"macro")
                paths = tool_artifacts(root / "verus", root / "z3")
                self.assertTrue(all(path in paths for path in macros))
                guard = ArtifactGuard(paths)
                guard.check()
                macros[1].unlink()
                with self.assertRaises(ArtifactChanged):
                    guard.check()
                with self.assertRaises(ArtifactChanged):
                    ArtifactGuard(tool_artifacts(root / "verus", root / "z3"))

    @unittest.skipUnless(os.name == "posix", "POSIX nested process-group interruption")
    def test_timeout_allows_nested_runner_to_reap_children_and_save_failure_receipt(self):
        child = """import os,sys,time
from pathlib import Path
root=Path(sys.argv[1]);(root/'pid').write_text(str(os.getpid()))
try:
    print('child started',flush=True);time.sleep(30)
finally:
    (root/'cleaned').write_text('child cleanup ran')
"""
        worker = f"""import os,sys
from pathlib import Path
sys.path.insert(0,sys.argv[1])
from infer_experiment import LoggedCommands,replay_receipt
root=Path(sys.argv[2]);receipt={{'commands':[]}}
with replay_receipt(root,receipt):
    LoggedCommands(root,root,receipt['commands'],timeout=20)('nested',
        [sys.executable,'-c',{child!r},str(root)],dict(os.environ))
"""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            commands = []
            with self.assertRaises(subprocess.TimeoutExpired):
                LoggedCommands(root, root, commands, timeout=1)("outer",
                    [sys.executable, "-B", "-O", "-c", worker, str(TOOLS), tmp], dict(os.environ))
            self.assertTrue((root / "cleaned").is_file())
            with self.assertRaises(ProcessLookupError):
                os.kill(int((root / "pid").read_text()), 0)
            receipt = json.loads((root / "receipt.json").read_text())
            self.assertFalse(receipt["success"])
            self.assertEqual(receipt["failure"]["type"], "KeyboardInterrupt")
            self.assertTrue(receipt["commands"][0]["interrupted"])
            self.assertTrue(commands[0]["timed_out"])


@unittest.skipUnless(os.name == "posix", "fake executable scripts require POSIX")
class ReplayRunnerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.engine = self.root / "engine"
        self.tools = self.engine / "source/tools"
        self.tools.mkdir(parents=True)
        for name in (*RUNNERS, *SUPPORT):
            shutil.copyfile(TOOLS / (name + ".py"), self.tools / (name + ".py"))
        fixture = self.tools / "native_adoption"
        (fixture / "src").mkdir(parents=True)
        for name in ("Cargo.toml", "Cargo.lock", "src/lib.rs"):
            (fixture / name).write_text("fake fixture")
        self.binary = self.engine / "source/target-verus/release"
        self.binary.mkdir(parents=True)
        for name in ("rust_verify", "vstd.vir", "libvstd.rlib", "libverus_builtin.rlib",
                     "libverus_builtin_macros.so", "libverus_state_machines_macros.so"):
            (self.binary / name).write_bytes(b"stable")
        (self.engine / "source/z3").write_bytes(b"fake solver")
        (self.engine / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "fake"\n')
        self.commands = self.root / "commands"
        self.commands.mkdir()
        for name in ("cargo", "git", "rustc", "verus", "cargo-verus"):
            path = (self.binary if name in ("verus", "cargo-verus") else self.commands) / name
            path.write_text("#!" + sys.executable + "\n" + FAKE_COMMAND)
            path.chmod(0o755)
        self.native = self.root / "native"
        (self.native / "src").mkdir(parents=True)
        (self.native / "tests").mkdir()
        for name in ("Cargo.toml", "Cargo.lock", "Cargo-recent.lock", "VERUS_INFER.md",
                     "src/iter.rs", "src/error.rs", "tests/native.rs"):
            (self.native / name).write_bytes(b"original\r\n")
        (self.native / "src/lib.rs").write_bytes(b"t[Char::LowerA as usize] = 10\r\n")
        self.before = {str(path.relative_to(self.native)): path.read_bytes()
                       for path in self.native.rglob("*") if path.is_file()}
        self.env = dict(os.environ, PATH=str(self.commands) + os.pathsep + os.environ.get("PATH", ""),
                        FAKE_COMMAND_LOG=str(self.root / "command-log"), FAKE_CHECKED=json.dumps(CHECKED),
                        FAKE_READY=str(self.root / "ready"))

    def replay(self, name, *, failure="", optimized=True, positive=True, timeout=5):
        output = self.root / ("output-" + str(len(list(self.root.glob("output-*")))))
        command = [sys.executable, "-B", *(["-O"] if optimized else []),
                   str(self.tools / (name + ".py")), "--output", str(output), "--timeout", str(timeout)]
        if name == "check_native_adoption":
            command += ["--verus-source", str(self.engine / "source")]
        else:
            command += ["--native-root", str(self.native)]
            if positive and name in HEX:
                command += ["--positive-only"]
        result = subprocess.run(command, env=dict(self.env, FAKE_FAILURE=failure),
                                capture_output=True, text=True, timeout=20)
        if name == "check_native_adoption":
            output = next(output.glob("run-*"))
        receipt = json.loads((output / "receipt.json").read_text())
        self.assertEqual(receipt["success"], result.returncode == 0, result.stdout + result.stderr)
        return result, receipt, output

    def assert_restored(self, receipt):
        self.assertEqual(receipt["native_sources_before"], receipt["native_sources_after"])
        self.assertEqual(self.before, {str(path.relative_to(self.native)): path.read_bytes()
                                      for path in self.native.rglob("*") if path.is_file()})

    def test_success_runs_all_required_commands_with_and_without_optimization(self):
        for optimized in (False, True):
            for name in RUNNERS:
                with self.subTest(runner=name, optimized=optimized):
                    result, receipt, output = self.replay(name, optimized=optimized)
                    self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                    labels = [entry["label"] for entry in receipt["commands"]]
                    if name == "check_native_adoption":
                        self.assertEqual(len([label for label in labels if label.endswith("_clean")]), 9)
                        self.assertIn("native", labels)
                        self.assertEqual(receipt["commands"][-1]["exit_status"], 0)
                        self.assertEqual(len(receipt["cases"]), 9)
                    elif name in HEX:
                        self.assertTrue({"fresh", "restored", "native_default", "native_all",
                                         "native_no_default", "native_alloc"}.issubset(labels))
                        if name != "check_hex_adoption":
                            self.assertIn("native_msrv", labels)
                        self.assert_restored(receipt)
                    else:
                        self.assertEqual(labels, ["blocked", "restored"])
                        self.assertFalse(receipt["public_vector_verified"])
                        restored = json.loads((output / "restored-iterator/receipt.json").read_text())
                        self.assertTrue(restored["success"])

    def test_optimized_native_clean_and_tests_cannot_be_skipped(self):
        for failure in ("clean", "native"):
            with self.subTest(failure=failure):
                result, receipt, output = self.replay("check_native_adoption", failure=failure)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(receipt["commands"][-1]["exit_status"], 7)
                self.assertEqual(receipt["failure"]["type"], "ValueError")

    def test_optimized_evidence_and_native_failures_are_rejected(self):
        for name in ("check_native_adoption", *HEX):
            for failure in ("proof", "scope", "malformed", "native"):
                with self.subTest(runner=name, failure=failure):
                    result, receipt, output = self.replay(name, failure=failure)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn("failure", receipt)
                    if name in HEX:
                        self.assert_restored(receipt)

    def test_optimized_negative_controls_require_candidate_checks_and_restore_sources(self):
        for name in HEX:
            for failure in ("candidate", "publication"):
                with self.subTest(runner=name, failure=failure):
                    result, receipt, output = self.replay(name, failure=failure, positive=False)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertEqual(receipt["commands"][-1]["label"], "wrong_table")
                    self.assert_restored(receipt)

    def test_optimized_mutation_preconditions_fail_before_writing_sources(self):
        (self.native / "src/lib.rs").write_bytes(b"no mutation marker\r\n")
        self.before["src/lib.rs"] = (self.native / "src/lib.rs").read_bytes()
        for name in HEX:
            with self.subTest(runner=name):
                result, receipt, output = self.replay(name, positive=False)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(receipt["commands"][-1]["label"], "fresh")
                self.assertIn("mutation", receipt["failure"]["message"])
                self.assert_restored(receipt)

    def test_timeouts_preserve_partial_logs_failure_receipts_and_sources(self):
        for name in ("check_native_adoption", *HEX):
            with self.subTest(runner=name):
                failure = "clean_timeout" if name == "check_native_adoption" else "mutation_timeout"
                result, receipt, output = self.replay(name, failure=failure, positive=False, timeout=.3)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(receipt["failure"]["type"], "TimeoutExpired")
                entry = receipt["commands"][-1]
                self.assertTrue(entry["timed_out"])
                self.assertIn("clean" if failure == "clean_timeout" else "mutation started",
                              (output / (entry["label"] + ".stdout")).read_text())
                if name in HEX:
                    self.assert_restored(receipt)

    def test_restoration_failure_still_writes_receipt_and_restores_other_files(self):
        result, receipt, output = self.replay(HEX[0], failure="restoration", positive=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("source restoration failed", receipt["failure"]["message"])
        self.assertTrue(receipt["restoration_errors"])
        self.assertIsNone(receipt["native_sources_after"]["src/lib.rs"])
        self.assertEqual((self.native / "src/iter.rs").read_bytes(), self.before["src/iter.rs"])

    def test_artifact_mutation_aborts_before_next_command(self):
        result, receipt, output = self.replay(HEX[0], failure="artifact")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(receipt["failure"]["type"], "ArtifactChanged")
        self.assertEqual(receipt["commands"][-1]["label"], "fresh")
        self.assert_restored(receipt)

    def test_interrupt_restores_mutations_and_keeps_command_and_failure_receipts(self):
        output = self.root / "interrupted"
        command = [sys.executable, "-B", "-O", str(self.tools / (HEX[0] + ".py")),
                   "--output", str(output), "--native-root", str(self.native), "--timeout", "20"]
        process = subprocess.Popen(command, env=dict(self.env, FAKE_FAILURE="mutation_timeout"),
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
                                   start_new_session=True)
        try:
            ready = self.root / "ready"
            deadline = time.monotonic() + 10
            while not ready.exists() and process.poll() is None and time.monotonic() < deadline:
                time.sleep(.01)
            self.assertTrue(ready.exists(), "fake mutation did not start")
            process.send_signal(signal.SIGINT)
            stdout, stderr = process.communicate(timeout=10)
            self.assertNotEqual(process.returncode, 0, stdout + stderr)
            with self.assertRaises(ProcessLookupError):
                os.kill(int(ready.read_text()), 0)
            receipt = json.loads((output / "receipt.json").read_text())
            self.assertFalse(receipt["success"])
            self.assertEqual(receipt["failure"]["type"], "KeyboardInterrupt")
            self.assertTrue(receipt["commands"][-1]["interrupted"])
            self.assert_restored(receipt)
        finally:
            if process.poll() is None:
                process.kill()
                process.communicate()

    def test_missing_artifacts_have_durable_setup_failure_receipts(self):
        (self.binary / "libverus_state_machines_macros.so").unlink()
        for name in RUNNERS:
            with self.subTest(runner=name):
                result, receipt, output = self.replay(name)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(receipt["failure"]["type"], "ArtifactChanged")
                self.assertFalse(receipt["commands"])

    def test_launch_failure_keeps_logs_and_a_failed_command_receipt(self):
        (self.binary / "verus").chmod(0o644)
        result, receipt, output = self.replay(HEX[0])
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(receipt["failure"]["type"], "PermissionError")
        self.assertEqual(receipt["commands"][-1]["label"], "fresh")
        self.assertIsNone(receipt["commands"][-1]["exit_code"])
        self.assertIn("Permission denied", (output / "fresh.stderr").read_text())
        self.assert_restored(receipt)

    def test_phase3_rejects_incomplete_diagnostics_publication_and_failed_restoration(self):
        for failure in ("diagnostics", "publication", "native"):
            with self.subTest(failure=failure):
                result, receipt, output = self.replay("check_hex_vec_phase3", failure=failure)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(receipt["public_vector_verified"])
                self.assertEqual(receipt["sources_before"], receipt["sources_after"])

    def test_timeout_cli_rejects_nonpositive_and_nonfinite_values(self):
        for name in RUNNERS:
            for timeout in ("0", "-1", "nan", "inf"):
                with self.subTest(runner=name, timeout=timeout):
                    result = subprocess.run([sys.executable, "-B", "-O", str(self.tools / (name + ".py")),
                                             "--output", str(self.root / "unused"),
                                             "--native-root" if name != "check_native_adoption" else "--verus-source",
                                             str(self.native), "--timeout", timeout],
                                            capture_output=True, text=True, timeout=5)
                    self.assertEqual(result.returncode, 2, result.stderr)
        self.assertFalse((self.root / "unused").exists())


if __name__ == "__main__":
    unittest.main()
