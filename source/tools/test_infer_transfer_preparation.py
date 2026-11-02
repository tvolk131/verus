"""Offline tests for exact-pin CI preparation and its proof/nonproof boundary."""

from contextlib import redirect_stdout
import copy
import io
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import prepare_infer_transfers as preparation
import check_infer_transfers as transfers


class PreparationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="infer preparation ")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.destination = self.root / "applications"
        self.output = self.root / "evidence"
        self.calls = []
        self.failure = None
        self.wrong_head = False
        self.wrong_root = False
        self.dirty = False

    def runner(self, output, cwd, commands, *, timeout):
        owner = self

        class FakeCommands:
            def __init__(self):
                self.output = output

            def __call__(self, label, command, environment):
                owner.calls.append((label, command,
                                    {name: value for name, value in environment.items()
                                     if name.startswith("GIT_")}, timeout))
                commands.append(dict(label=label, argv=command, cwd=str(cwd), exit_code=0))
                if owner.failure == label:
                    commands[-1]["exit_code"] = 1
                    return 1
                if command[1] == "init":
                    Path(command[2]).mkdir()
                    (Path(command[2]) / ".git").mkdir()
                text = ""
                if label.endswith("_root"):
                    text = str(owner.root / "redirected") if owner.wrong_root else command[2]
                if label.endswith("_head"):
                    scenario = label.removesuffix("_prepared_head")
                    text = "0" * 40 if owner.wrong_head else preparation.SCENARIOS[scenario]["commit"]
                if label.endswith("_status") and owner.dirty:
                    text = " M modified.rs"
                (output / (label + ".stdout")).write_text(text + "\n")
                (output / (label + ".stderr")).write_text("")
                return 0

        return FakeCommands()

    def prepare(self):
        with patch.object(preparation, "LoggedCommands", self.runner), redirect_stdout(io.StringIO()):
            preparation.prepare(self.destination, self.output, 29)

    def receipt(self):
        return json.loads((self.output / "receipt.json").read_text())

    def test_prepares_each_checkout_once_using_shared_scenario_pins(self):
        self.prepare()
        receipt = self.receipt()
        self.assertTrue(receipt["success"])
        self.assertFalse(receipt["application_proofs_proved"])
        self.assertEqual(receipt["kind"], "application_preparation")
        self.assertEqual(set(receipt["applications"]), {"fedimint", "redb"})
        fetches = [call for call in self.calls if call[0].endswith("_fetch")]
        self.assertEqual(len(fetches), 2)
        for label, command, environment, timeout in fetches:
            checkout = label.removesuffix("_fetch")
            self.assertEqual(command[-1], preparation.application_specs()[checkout]["commit"])
            self.assertEqual(command[4:7], ["--no-tags", "--depth=1", "origin"])
            self.assertEqual(environment["GIT_TERMINAL_PROMPT"], "0")
            self.assertEqual(timeout, 29)

    def test_refuses_to_overwrite_existing_application_sources(self):
        self.destination.mkdir()
        original = self.destination / "keep.rs"
        original.write_text("original")
        with self.assertRaisesRegex(ValueError, "must not already exist"):
            self.prepare()
        self.assertEqual(original.read_text(), "original")
        self.assertFalse(self.calls)
        self.assertFalse(self.receipt()["success"])

    def test_refuses_a_dangling_destination_symlink(self):
        self.destination.symlink_to(self.root / "missing", target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "must not already exist"):
            self.prepare()
        self.assertTrue(self.destination.is_symlink())
        self.assertFalse(self.calls)

    def test_git_context_and_configuration_are_not_inherited(self):
        inherited = {
            "GIT_DIR": "/unrelated/repository/.git",
            "GIT_WORK_TREE": "/unrelated/repository",
            "GIT_COMMON_DIR": "/unrelated/repository/.git",
            "GIT_INDEX_FILE": "/unrelated/index",
            "GIT_OBJECT_DIRECTORY": "/unrelated/objects",
            "GIT_CONFIG_COUNT": "1",
            "GIT_CONFIG_KEY_0": "core.worktree",
            "GIT_CONFIG_VALUE_0": "/unrelated/repository",
        }
        with patch.dict(os.environ, inherited):
            self.prepare()
        for _, _, environment, _ in self.calls:
            self.assertEqual(
                {name: value for name, value in environment.items() if name.startswith("GIT_")},
                {"GIT_TERMINAL_PROMPT": "0", "GIT_CONFIG_GLOBAL": os.devnull,
                 "GIT_CONFIG_NOSYSTEM": "1"},
            )

    def test_rejects_wrong_repository_root_before_fetching(self):
        self.wrong_root = True
        with self.assertRaisesRegex(ValueError, "root differs"):
            self.prepare()
        self.assertFalse(self.receipt()["success"])
        self.assertFalse(any(label.endswith("_fetch") for label, _, _, _ in self.calls))

    def test_repeated_output_preserves_success_receipt_and_logs(self):
        self.prepare()
        original = {path.name: path.read_bytes() for path in self.output.iterdir()}
        self.calls.clear()
        with self.assertRaisesRegex(ValueError, "evidence destination must not already exist"):
            self.prepare()
        self.assertEqual(original, {path.name: path.read_bytes() for path in self.output.iterdir()})
        self.assertFalse(self.calls)
        self.assertTrue(self.receipt()["success"])

    def test_existing_or_symlinked_evidence_cannot_redirect_pending_receipts(self):
        victim = self.root / "original.rs"
        victim.write_text("original source")
        actual = self.root / "existing_evidence"
        actual.mkdir()
        pending = actual / "receipt.json.tmp"
        pending.symlink_to(victim)
        linked = self.output
        linked.symlink_to(actual, target_is_directory=True)
        for output in [actual, linked]:
            with self.subTest(output=output):
                self.output = output
                with self.assertRaisesRegex(ValueError, "evidence destination must not already exist"):
                    self.prepare()
                self.assertEqual(victim.read_text(), "original source")
                self.assertTrue(pending.is_symlink())
                self.assertFalse((actual / "receipt.json").exists())
        self.assertFalse(self.calls)

    def test_rejects_dangling_evidence_symlink(self):
        self.output.symlink_to(self.root / "absent_evidence", target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "evidence destination must not already exist"):
            self.prepare()
        self.assertTrue(self.output.is_symlink())
        self.assertFalse((self.root / "absent_evidence").exists())
        self.assertFalse(self.calls)

    def test_rejects_overlapping_application_and_evidence_destinations(self):
        for destination, output in [
            (self.destination, self.destination / "evidence"),
            (self.output / "applications", self.output),
        ]:
            with self.subTest(destination=destination):
                with self.assertRaisesRegex(ValueError, "must be disjoint"):
                    preparation.prepare(destination, output, 29)
                self.assertFalse(destination.exists())
                self.assertFalse(output.exists())
        self.assertFalse(self.calls)

    def test_rejects_conflicting_redb_scenario_pins_before_fetching(self):
        scenarios = copy.deepcopy(preparation.SCENARIOS)
        scenarios["redb_page_size"]["commit"] = "0" * 40
        with patch.object(preparation, "SCENARIOS", scenarios):
            with self.assertRaisesRegex(ValueError, "conflicting pins"):
                self.prepare()
        self.assertFalse(self.calls)
        self.assertFalse(self.receipt()["success"])

    def test_rejects_branch_names_and_unknown_checkouts(self):
        for mutation, diagnostic in [("commit", "exact commit"), ("checkout", "unknown application")]:
            with self.subTest(mutation=mutation):
                scenarios = copy.deepcopy(preparation.SCENARIOS)
                scenarios["fedimint"][mutation] = "unrecognized"
                with patch.object(preparation, "SCENARIOS", scenarios):
                    with self.assertRaisesRegex(ValueError, diagnostic):
                        preparation.application_specs()

    def test_fetch_failure_leaves_inspectable_failure_receipt(self):
        self.failure = "fedimint_fetch"
        with self.assertRaisesRegex(ValueError, "fedimint fetch"):
            self.prepare()
        self.assertFalse(self.receipt()["success"])
        self.assertNotIn("fedimint_checkout", [call[0] for call in self.calls])
        self.assertTrue((self.destination / "fedimint").is_dir())

    def test_checks_actual_head_and_clean_status_after_checkout(self):
        for setting in ["wrong_head", "dirty"]:
            with self.subTest(setting=setting):
                setattr(self, setting, True)
                with self.assertRaisesRegex(ValueError, "wrong pin or local changes"):
                    self.prepare()
                self.assertFalse(self.receipt()["success"])
                self.assertFalse(self.receipt()["applications"])
                setattr(self, setting, False)
                self.destination = self.root / (setting + "_next")
                self.output = self.root / (setting + "_next_evidence")


class GitEnvironmentRegressionTests(unittest.TestCase):
    def git(self, arguments, cwd):
        environment = {name: value for name, value in os.environ.items()
                       if not name.startswith("GIT_")}
        environment.update(GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1",
                           GIT_TERMINAL_PROMPT="0")
        process = subprocess.run(
            ["git", "-c", "user.name=Infer fixture", "-c", "user.email=infer@example.invalid",
             *arguments], cwd=cwd, env=environment, text=True, capture_output=True,
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        return process.stdout.strip()

    def test_actual_git_ignores_inherited_context_and_preserves_existing_repository(self):
        with tempfile.TemporaryDirectory(prefix="real infer preparation ") as temporary:
            root = Path(temporary)
            repositories = {}
            pins = {}
            for name in preparation.REPOSITORIES:
                origin = root / (name + "_origin")
                origin.mkdir()
                self.git(["init"], origin)
                (origin / "fixture.rs").write_text(name)
                self.git(["add", "fixture.rs"], origin)
                self.git(["commit", "-m", "local fixture"], origin)
                pins[name] = self.git(["rev-parse", "HEAD"], origin)
                repositories[name] = origin.as_uri()
            victim = root / "unrelated_repository"
            victim.mkdir()
            self.git(["init"], victim)
            (victim / "original.rs").write_text("original source")
            self.git(["add", "original.rs"], victim)
            self.git(["commit", "-m", "preserved repository"], victim)
            original_head = self.git(["rev-parse", "HEAD"], victim)
            original_index = (victim / ".git/index").read_bytes()
            configuration = root / "untrusted.gitconfig"
            configuration.write_text("[core]\nworktree = " + str(victim) + "\n")
            inherited = {
                "GIT_DIR": str(victim / ".git"), "GIT_WORK_TREE": str(victim),
                "GIT_COMMON_DIR": str(victim / ".git"),
                "GIT_INDEX_FILE": str(victim / ".git/index"),
                "GIT_OBJECT_DIRECTORY": str(victim / ".git/objects"),
                "GIT_CONFIG_GLOBAL": str(configuration),
                "GIT_CONFIG_COUNT": "1", "GIT_CONFIG_KEY_0": "core.worktree",
                "GIT_CONFIG_VALUE_0": str(victim),
            }
            scenarios = copy.deepcopy(preparation.SCENARIOS)
            for specification in scenarios.values():
                specification["commit"] = pins[specification["checkout"]]
            destination = root / "prepared_applications"
            output = root / "evidence"
            with patch.object(preparation, "REPOSITORIES", repositories), \
                    patch.object(preparation, "SCENARIOS", scenarios), \
                    patch.object(transfers, "SCENARIOS", scenarios), \
                    patch.dict(os.environ, inherited), redirect_stdout(io.StringIO()):
                preparation.prepare(destination, output, 10)
            receipt = json.loads((output / "receipt.json").read_text())
            self.assertTrue(receipt["success"])
            self.assertFalse(receipt["application_proofs_proved"])
            for name, pin in pins.items():
                application = destination / name
                self.assertEqual(self.git(["rev-parse", "HEAD"], application), pin)
                self.assertEqual(Path(self.git(["rev-parse", "--show-toplevel"], application)).resolve(),
                                 application.resolve())
                self.assertEqual(self.git(["status", "--porcelain"], application), "")
                self.assertTrue((application / ".git").is_dir())
            self.assertEqual(self.git(["rev-parse", "HEAD"], victim), original_head)
            self.assertEqual((victim / "original.rs").read_text(), "original source")
            self.assertEqual((victim / ".git/index").read_bytes(), original_index)
            self.assertEqual(self.git(["remote"], victim), "")


class WorkflowTests(unittest.TestCase):
    def test_ci_builds_and_checks_the_selected_branch_without_a_master_fallback(self):
        path = Path(__file__).resolve().parents[2] / ".github/workflows/infer-transfers.yml"
        workflow = path.read_text()
        self.assertIn("workflow_dispatch:", workflow)
        self.assertIn("schedule:", workflow)
        self.assertNotIn("pull_request:", workflow)
        self.assertIn("ref: ${{ inputs.ref || 'verus-infer' }}", workflow)
        self.assertIn("persist-credentials: false", workflow)
        self.assertIn("contents: read", workflow)
        self.assertIn("prepare_infer_transfers.py", workflow)
        self.assertIn("check_infer_transfers.py", workflow)
        self.assertNotIn("--scenario ", workflow)
        self.assertIn("--scenario-timeout 480", workflow)
        self.assertIn("timeout-minutes: 90", workflow)
        self.assertIn("if: always()", workflow)
        self.assertIn("actions/upload-artifact@v4", workflow)
        self.assertNotIn("--no-verify", workflow)
        self.assertNotIn("continue-on-error", workflow)
        for specification in preparation.SCENARIOS.values():
            self.assertNotIn(specification["commit"], workflow)
        blocks = []
        for section in workflow.split("        run: |\n")[1:]:
            lines = []
            for line in section.splitlines():
                if line and not line.startswith("          "):
                    break
                lines.append(line[10:] if line else "")
            blocks.append("\n".join(lines))
        self.assertEqual(len(blocks), 6)
        for block in blocks:
            result = subprocess.run(["bash", "-n"], input=block, text=True,
                                    capture_output=True, check=False)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_step_deadlines_leave_time_for_artifact_upload(self):
        path = Path(__file__).resolve().parents[2] / ".github/workflows/infer-transfers.yml"
        workflow = path.read_text()
        budget = int(re.search(r"^    timeout-minutes: (\d+)$", workflow, re.MULTILINE)[1])
        deadlines = {}
        for step in workflow.split("      - name: ")[1:]:
            name = step.splitlines()[0]
            timeout = re.search(r"^        timeout-minutes: (\d+)$", step, re.MULTILINE)
            self.assertIsNotNone(timeout, name)
            deadlines[name] = int(timeout[1])
        self.assertEqual(len(deadlines), 9)
        self.assertTrue(all(timeout > 0 for timeout in deadlines.values()))
        self.assertLessEqual(sum(deadlines.values()), budget - 5)
        self.assertEqual(deadlines["Retain proof receipts and failure evidence"], 5)
        self.assertIn("timeout --kill-after=15s 3m ./tools/get-z3.sh", workflow)
        self.assertIn("timeout --kill-after=30s 13m cargo build", workflow)
        self.assertIn("timeout --kill-after=30s 6m cargo run", workflow)


if __name__ == "__main__":
    unittest.main()
