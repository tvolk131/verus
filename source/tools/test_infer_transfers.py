"""Independent acceptance tests for the combined transfer regression entry point."""

import argparse
from contextlib import redirect_stdout
import copy
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import check_infer_transfers as transfers


def proof_report(positive):
    return {
        "verification-results": {
            "success": positive, "verified": 8 if positive else 7,
            "errors": 0 if positive else 1, "is-verifying-entire-crate": True,
            "encountered-vir-error": False,
        },
        "verus-infer": {
            "status": "verified" if positive else "caller_proof_failed",
            "crate_success": positive, "interface_written": positive,
            "functions": [{
                "function": "application::helper", "mode": "exact",
                "required_checks_passed": True,
                "proofs": {name: {"status": "proved", "required": True}
                           for name in ("soundness", "maximality", "exactness")},
            }],
        },
    }


def scenario_receipt(directory, scenario, verifier, *, status="proved"):
    directory.mkdir(parents=True, exist_ok=True)
    receipt = {
        "success": True, "commands": [],
        "transfer": {
            "scenario": scenario, "application_commit": transfers.SCENARIOS[scenario]["commit"],
            "status": status, "source_fidelity": True,
            "positive_cases": ["positive_cold", "positive_warm"],
            "semantic_negative_cases": ["mutant"], "stale_export_rejected": True,
            "limitations": ["Extracted helper bodies, not whole-application verification."],
            "trust": ["Bundled library specifications and ordinary verifier backend."],
        },
    }
    if status == "blocked":
        receipt["transfer"]["blocker"] = "unsupported original-source operation"
    for label, positive in (("positive_cold", True), ("positive_warm", True), ("mutant", False)):
        artifact = directory / (label + ".vir")
        if positive:
            artifact.write_text("interface")
        (directory / (label + ".stdout")).write_text(json.dumps(proof_report(positive)))
        (directory / (label + ".stderr")).write_text("" if positive else "postcondition not satisfied")
        receipt["commands"].append({
            "label": label, "cwd": str(directory), "exit_code": 0 if positive else 1,
            "argv": [str(verifier), "input.rs", "--no-cheating", "--output-json",
                     "--export", str(artifact)],
        })
    path = directory / "receipt.json"
    path.write_text(json.dumps(receipt))
    return path, receipt


class ReceiptTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="transfer tests ")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.verifier = self.root / "verus"
        self.path, self.receipt = scenario_receipt(self.root / "run", "redb", self.verifier)

    def check(self):
        self.path.write_text(json.dumps(self.receipt))
        return transfers.check_receipt(self.path, "redb", self.verifier)

    def mutate_report(self, label, mutate):
        path = self.path.parent / (label + ".stdout")
        report = json.loads(path.read_text())
        mutate(report)
        path.write_text(json.dumps(report))

    def test_accepts_fresh_exact_proofs_and_semantic_control(self):
        summary = self.check()
        self.assertEqual(summary["status"], "proved")
        self.assertEqual(len(summary["proofs"]), 3)
        self.assertEqual(summary["proofs"][-1]["errors"], 1)
        self.assertTrue(summary["stale_export_rejected"])

    def test_controller_success_is_not_application_proof(self):
        self.receipt["transfer"].update(status="blocked", blocker="unsupported unwrap")
        summary = self.check()
        self.assertEqual(summary["status"], "blocked")
        self.assertNotIn("proofs", summary)

    def test_auxiliary_controls_cannot_replace_semantic_or_safety_controls(self):
        label = "false_auxiliary"
        self.receipt["transfer"]["auxiliary_negative_cases"] = [label]
        command = copy.deepcopy(self.receipt["commands"][-1])
        command["label"] = label
        command["argv"][-1] = str(self.path.parent / (label+".vir"))
        self.receipt["commands"].append(command)
        report = proof_report(False)
        report["verus-infer"]["status"] = "verification_failed"
        (self.path.parent / (label+".stdout")).write_text(json.dumps(report))
        stderr = self.path.parent / (label+".stderr")
        stderr.write_text("bitvector assertion not satisfied")
        self.assertEqual(self.check()["proofs"][-1]["failure_kind"], "auxiliary")
        for diagnostic in ("postcondition not satisfied", "precondition not satisfied",
                           "bitvector assertion not satisfied: Resource limit exceeded"):
            stderr.write_text(diagnostic)
            with self.assertRaises(ValueError):
                self.check()

    def test_safety_controls_are_separate_from_semantic_postconditions(self):
        label = "unsafe_shift"
        self.receipt["transfer"]["safety_negative_cases"] = [label]
        command = copy.deepcopy(self.receipt["commands"][-1])
        command["label"] = label
        command["argv"][-1] = str(self.path.parent / (label + ".vir"))
        self.receipt["commands"].append(command)
        (self.path.parent / (label + ".stdout")).write_text(json.dumps(proof_report(False)))
        stderr = self.path.parent / (label + ".stderr")
        stderr.write_text("precondition not satisfied")
        summary = self.check()
        self.assertEqual(summary["proofs"][-1]["failure_kind"], "precondition")
        for diagnostic in ("postcondition not satisfied", "resource limit; precondition not satisfied",
                           "precondition not satisfied; postcondition not satisfied"):
            stderr.write_text(diagnostic)
            with self.subTest(diagnostic=diagnostic), self.assertRaises(ValueError):
                self.check()
        stderr.write_text("precondition not satisfied")
        self.receipt["transfer"]["semantic_negative_cases"] = []
        with self.assertRaises(ValueError):
            self.check()

    def test_rejects_unfinished_controller_wrong_pin_and_missing_scope(self):
        original = copy.deepcopy(self.receipt)
        changes = [lambda receipt: receipt.update(success=False),
                   lambda receipt: receipt.update(success=1),
                   lambda receipt: receipt["transfer"].update(application_commit="wrong"),
                   lambda receipt: receipt["transfer"].update(scenario="fedimint"),
                   lambda receipt: receipt["transfer"].update(source_fidelity=False),
                   lambda receipt: receipt["transfer"].update(trust=[]),
                   lambda receipt: receipt["transfer"].update(limitations=[]),
                   lambda receipt: receipt["transfer"].update(status="unknown"),
                   lambda receipt: receipt["transfer"].update(status="blocked", blocker="")]
        for change in changes:
            with self.subTest(change=change):
                self.receipt = copy.deepcopy(original)
                change(self.receipt)
                with self.assertRaises(ValueError):
                    self.check()

    def test_rejects_missing_duplicate_or_malformed_proof_commands(self):
        original = copy.deepcopy(self.receipt)
        changes = [lambda receipt: receipt.update(commands=[]),
                   lambda receipt: receipt["commands"].append(receipt["commands"][0]),
                   lambda receipt: receipt["transfer"].update(positive_cases=[]),
                   lambda receipt: receipt["transfer"].update(semantic_negative_cases=[]),
                   lambda receipt: receipt["transfer"].update(semantic_negative_cases=["positive_cold"]),
                   lambda receipt: receipt["transfer"].update(positive_cases=["../escape"]),
                   lambda receipt: receipt["transfer"].update(stale_export_rejected=False),
                   lambda receipt: receipt["commands"][0].update(exit_code=None),
                   lambda receipt: receipt["commands"][0].update(exit_code=True)]
        for change in changes:
            with self.subTest(change=change):
                self.receipt = copy.deepcopy(original)
                change(self.receipt)
                with self.assertRaises(ValueError):
                    self.check()

    def test_requires_all_fresh_exact_checks_and_nonempty_function_set(self):
        original = (self.path.parent / "positive_cold.stdout").read_text()
        for obligation in ("soundness", "maximality", "exactness"):
            with self.subTest(obligation=obligation):
                self.mutate_report("positive_cold", lambda report: report["verus-infer"]["functions"][0]
                                   ["proofs"][obligation].update(status="cached"))
                with self.assertRaises(ValueError):
                    self.check()
                (self.path.parent / "positive_cold.stdout").write_text(original)
        self.mutate_report("positive_cold", lambda report: report["verus-infer"].update(functions=[]))
        with self.assertRaises(ValueError):
            self.check()

    def test_rejects_frontend_failure_timeout_and_wrong_negative_failure(self):
        original = (self.path.parent / "mutant.stdout").read_text()
        for change in (
            lambda report: report["verification-results"].update(**{"encountered-vir-error": True}),
            lambda report: report["verification-results"].update(errors=0),
            lambda report: report["verus-infer"].update(status="generation_failed"),
            lambda report: report["verification-results"].update(**{"is-verifying-entire-crate": False}),
        ):
            with self.subTest(change=change):
                self.mutate_report("mutant", change)
                with self.assertRaises(ValueError):
                    self.check()
                (self.path.parent / "mutant.stdout").write_text(original)
        for stderr in ("resource limit", "timed out", "panicked at", "precondition not satisfied"):
            with self.subTest(stderr=stderr):
                (self.path.parent / "mutant.stderr").write_text(stderr)
                with self.assertRaises(ValueError):
                    self.check()

    def test_rejects_wrong_verifier_and_disabled_checks(self):
        original = copy.deepcopy(self.receipt)
        for argv in (["other-verifier", "--no-cheating", "--output-json", "--export", "out.vir"],
                     [str(self.verifier), "--output-json", "--export", "out.vir"],
                     [*original["commands"][0]["argv"], "--no-verify"],
                     [*original["commands"][0]["argv"], "--no-lifetime"]):
            self.receipt = copy.deepcopy(original)
            self.receipt["commands"][0]["argv"] = argv
            with self.subTest(argv=argv), self.assertRaises(ValueError):
                self.check()

    def test_rejects_missing_positive_or_remaining_negative_export(self):
        positive = self.path.parent / "positive_cold.vir"
        positive.unlink()
        with self.assertRaises(ValueError):
            self.check()
        positive.write_text("interface")
        (self.path.parent / "mutant.vir").write_text("stale interface")
        with self.assertRaises(ValueError):
            self.check()

    def test_receipt_and_log_paths_cannot_escape_fresh_output(self):
        output = self.root / "output"
        output.mkdir()
        for text in ("", str(self.path.parent), str(output)):
            with self.subTest(text=text), self.assertRaises(ValueError):
                transfers.receipt_from_stdout(text, output)
        for relative in ("../receipt.json", str(self.path)):
            with self.subTest(relative=relative), self.assertRaises(ValueError):
                transfers.contained_file(self.path.parent, relative)
        (output / "escape").symlink_to(self.path.parent, target_is_directory=True)
        with self.assertRaises(ValueError):
            transfers.receipt_from_stdout(str(output / "escape"), output)
        (self.path.parent / "positive_cold.stdout").unlink()
        (self.path.parent / "positive_cold.stdout").symlink_to(self.path)
        with self.assertRaises((ValueError, KeyError)):
            self.check()


class SuiteTests(unittest.TestCase):
    def test_both_scenarios_run_and_failure_receipts_survive(self):
        for first_outcome in ("proved", "blocked", "runner_failed", "timeout"):
            with self.subTest(first_outcome=first_outcome), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                (root / "source").mkdir()
                (root / "source/tool.rs").write_text("engine")
                verifier = root / "verus"
                solver = root / "z3"
                verifier.write_text("verifier")
                solver.write_text("solver")
                args = argparse.Namespace(output=root / "output", scenario=None, verus=verifier,
                                          solver=solver, fedimint=root / "fedimint", redb=root / "redb",
                                          timeout=1, scenario_timeout=2)
                attempts = []

                class FakeGuard:
                    def __init__(self, paths):
                        self.initial = {}

                    def check(self):
                        pass

                class FakeCommands:
                    def __init__(self, output, cwd, commands, **kwargs):
                        self.output = output
                        self.commands = commands
                        self.guard = None

                    def __call__(self, label, command, environment):
                        stdout = ""
                        if label == "engine_files":
                            stdout = "source/tool.rs\n"
                        elif label.endswith("_head"):
                            stdout = transfers.SCENARIOS[label.split("_")[0]]["commit"]
                        elif label in transfers.SCENARIOS:
                            attempts.append(label)
                            if label == "fedimint" and first_outcome == "timeout":
                                raise subprocess.TimeoutExpired(command, 2)
                            if label == "fedimint" and first_outcome == "runner_failed":
                                return 1
                            destination = Path(command[command.index("--output") + 1]) / "run-fresh"
                            scenario_receipt(destination, label, verifier,
                                             status=first_outcome if label == "fedimint" else "proved")
                            stdout = str(destination)
                        (self.output / (label + ".stdout")).write_text(stdout)
                        (self.output / (label + ".stderr")).write_text("")
                        return 0

                with patch.object(transfers, "ROOT", root), \
                        patch.object(transfers, "ArtifactGuard", FakeGuard), \
                        patch.object(transfers, "tool_artifacts", return_value=[]), \
                        patch.object(transfers, "LoggedCommands", FakeCommands), redirect_stdout(io.StringIO()):
                    if first_outcome == "proved":
                        output = transfers.run_suite(args)
                    else:
                        with self.assertRaises(ValueError):
                            transfers.run_suite(args)
                        output = next(args.output.iterdir())
                receipt = json.loads((output / "receipt.json").read_text())
                self.assertEqual(attempts, list(transfers.SCENARIOS))
                self.assertEqual(receipt["success"], first_outcome == "proved")
                self.assertEqual(receipt["application_proofs_proved"], first_outcome == "proved")
                self.assertEqual(receipt["scenarios"][1]["status"], "proved")
                self.assertEqual(receipt["scenarios"][0]["status"],
                                 first_outcome if first_outcome in ("proved", "blocked") else "failed")


if __name__ == "__main__":
    unittest.main()
