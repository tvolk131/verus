"""Regression checks for shared evidence handling, including real subprocesses."""
import copy
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest

from infer_eval_support import digest, support_artifacts
from infer_experiment import (
    LoggedCommands, require_exact, require_publication, snapshot_inputs, validate_run,
)


class ExperimentTests(unittest.TestCase):
    def run_result(self, **changes):
        result = dict(returncode=1, stdout=json.dumps({
            "verification-results": {"success": False},
        }), stderr="postcondition not satisfied")
        return dict(result, **changes)

    def test_semantic_failure_cannot_be_a_frontend_or_resource_failure(self):
        validate_run(self.run_result(), False)
        for message in ("Panicked at checker", "Resource limit exceeded", "solver timed out"):
            with self.subTest(message=message), self.assertRaises(ValueError):
                validate_run(self.run_result(stderr=message), False)
        frontend = self.run_result(stdout=json.dumps({
            "verification-results": {"success": False, "encountered-vir-error": True},
        }))
        with self.assertRaises(ValueError):
            validate_run(frontend, False)
        validate_run(frontend, False, allow_frontend=True)
        early = self.run_result(stdout=json.dumps({
            "verification-results": {"encountered-vir-error": True},
        }))
        validate_run(early, False, allow_frontend=True)
        with self.assertRaises(ValueError):
            validate_run(early, False)
        with self.assertRaises(ValueError):
            validate_run(self.run_result(stdout=json.dumps({"verification-results": {}})),
                         False, allow_frontend=True)
        with self.assertRaises(ValueError):
            validate_run(self.run_result(returncode=0), False)

    def test_exact_checks_do_not_accept_a_missing_optional_or_failed_obligation(self):
        candidate = dict(mode="exact", required_checks_passed=True, proofs={
            p: dict(status="proved", required=True)
            for p in ("soundness", "maximality", "exactness")
        })
        require_exact([candidate])
        for obligation in candidate["proofs"]:
            for field, value in (("status", "incomplete"), ("required", False)):
                bad = copy.deepcopy(candidate)
                bad["proofs"][obligation][field] = value
                with self.subTest(obligation=obligation, field=field), self.assertRaises(ValueError):
                    require_exact([candidate, bad])
            bad = copy.deepcopy(candidate)
            del bad["proofs"][obligation]
            with self.assertRaises(ValueError):
                require_exact([bad])

    def test_publication_requires_both_report_agreement_and_the_actual_artifact(self):
        with tempfile.TemporaryDirectory() as tmp:
            artifact = Path(tmp) / "interface.vir"
            data = {"verification-results": {"success": False}, "verus-infer": {
                "crate_success": False, "interface_written": False,
            }}
            require_publication(data, False, artifact)
            artifact.write_bytes(b"stale")
            with self.assertRaises(ValueError):
                require_publication(data, False, artifact)
            data["verification-results"]["success"] = True
            data["verus-infer"] = dict(crate_success=True, interface_written=True)
            require_publication(data, True, artifact)
            data["verus-infer"]["interface_written"] = False
            with self.assertRaises(ValueError):
                require_publication(data, True, artifact)

    def test_snapshot_preserves_bytes_paths_and_hashes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "source").mkdir()
            (root / "source/input.rs").write_bytes(b"first\r\nsecond\x00")
            receipt = snapshot_inputs(root, root / "snapshot", ["source/input.rs"])
            self.assertEqual((root / "snapshot/source/input.rs").read_bytes(),
                             (root / "source/input.rs").read_bytes())
            self.assertEqual(receipt["source_hashes"],
                             {"source/input.rs": digest(root / "source/input.rs")})
            with self.assertRaises(ValueError):
                snapshot_inputs(root, root / "snapshot", ["../outside"])
            self.assertTrue(all(p.is_file() for p in support_artifacts()))

    def test_logged_commands_preserve_exit_status_streams_and_receipt(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            commands = []
            run = LoggedCommands(root, root, commands)
            command = [sys.executable, "-c",
                       "import sys;print('output');print('error',file=sys.stderr);sys.exit(3)"]
            self.assertEqual(run("negative", command, dict(os.environ)), 3)
            self.assertEqual((root / "negative.stdout").read_text(), "output\n")
            self.assertEqual((root / "negative.stderr").read_text(), "error\n")
            self.assertEqual(commands[0]["argv"], command)
            self.assertEqual(commands[0]["exit_code"], 3)
            run.timeout = .2
            with self.assertRaises(subprocess.TimeoutExpired):
                run("timeout", [sys.executable, "-c",
                    "import time;print('started',flush=True);time.sleep(30)"], dict(os.environ))
            self.assertIn("started", (root / "timeout.stdout").read_text())
            self.assertEqual(len(commands), 2)
            self.assertIsNone(commands[1]["exit_code"])
            self.assertTrue(commands[1]["timed_out"])

    @unittest.skipUnless(os.name == "posix", "POSIX process-group interruption")
    def test_interrupt_reaps_the_compiler_and_preserves_logs_and_receipt(self):
        child = """import os,sys,time
from pathlib import Path
print('started',flush=True)
ready=Path(sys.argv[1]);pending=ready.with_suffix('.tmp')
pending.write_text(str(os.getpid()));pending.replace(ready)
time.sleep(30)
"""
        worker = f"""import json,os,sys
from pathlib import Path
sys.path.insert(0,sys.argv[1])
from infer_experiment import LoggedCommands
root=Path(sys.argv[2]);commands=[]
try:
    LoggedCommands(root,root,commands)('interrupt',
        [sys.executable,'-c',{child!r},str(root/'ready')],dict(os.environ))
except KeyboardInterrupt:
    (root/'receipt.json').write_text(json.dumps(commands))
    sys.exit(130)
"""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            process = subprocess.Popen(
                [sys.executable, "-c", worker, str(Path(__file__).parent), tmp],
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
                start_new_session=True,
            )
            try:
                deadline = time.monotonic() + 10
                while not (root / "ready").exists() and time.monotonic() < deadline:
                    time.sleep(.01)
                self.assertTrue((root / "ready").exists(), "compiler child did not start")
                child_pid = int((root / "ready").read_text())
                process.send_signal(signal.SIGINT)
                stdout, stderr = process.communicate(timeout=10)
                self.assertEqual(process.returncode, 130, stdout + stderr)
                with self.assertRaises(ProcessLookupError):
                    os.kill(child_pid, 0)
                self.assertIn("started", (root / "interrupt.stdout").read_text())
                receipt = json.loads((root / "receipt.json").read_text())
                self.assertTrue(receipt[0]["interrupted"])
                self.assertIsNone(receipt[0]["exit_code"])
            finally:
                if process.poll() is None:
                    process.kill()
                    process.communicate()
                if (root / "ready").exists():
                    try:
                        os.kill(int((root / "ready").read_text()), signal.SIGKILL)
                    except ProcessLookupError:
                        pass


if __name__ == "__main__":
    unittest.main()
