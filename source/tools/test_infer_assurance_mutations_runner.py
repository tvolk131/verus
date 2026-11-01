import json
import subprocess
import unittest
from unittest.mock import patch

import test_infer_assurance_mutations as runner


def result_output(status="ok", passed=1, failed=0, ignored=0, started=None):
    if started is None:
        started = passed + failed + ignored
    return (f"running {started} {'test' if started == 1 else 'tests'}\n"
            f"test result: {status}. {passed} passed; {failed} failed; "
            f"{ignored} ignored; 0 measured; 0 filtered out; finished in 0.00s\n")


class MutationSitesTests(unittest.TestCase):
    def test_all_groups_have_unique_effective_sites_in_current_sources(self):
        groups = [("Acceptance", runner.POLICY_SOURCE, runner.POLICY_MUTATIONS)] + [
            (group, relative, mutations)
            for group, relative, _, mutations in runner.VIR_MUTATION_GROUPS
        ]
        for group, relative, mutations in groups:
            source = (runner.SOURCE / relative).read_text()
            names = [name for name, _, _ in mutations]
            self.assertTrue(names)
            self.assertNotIn("baseline", names)
            self.assertEqual(len(names), len(set(names)))
            for name, before, after in mutations:
                with self.subTest(group=group, mutation=name):
                    changed = runner.change(source, before, after)
                    self.assertNotEqual(changed, source)
                    self.assertEqual(source.count(before), 1)
        runner.preflight()

    def test_termination_controls_mutate_counted_and_native_branches_independently(self):
        _, relative, _, mutations = runner.VIR_MUTATION_GROUPS[0]
        source = (runner.SOURCE / relative).read_text()
        termination = [mutation for mutation in mutations if "termination measure" in mutation[0]]
        self.assertEqual(len(termination), 2)
        for name, before, after in termination:
            with self.subTest(mutation=name):
                changed = runner.change(source, before, after)
                self.assertNotIn(before, changed)
                for other_name, other_before, _ in termination:
                    if name != other_name:
                        self.assertIn(other_before, changed)

    def test_missing_ambiguous_and_noop_changes_are_rejected(self):
        for source, before, after in [
            ("", "", "replacement"),
            ("present", "missing", "replacement"),
            ("site site", "site", "replacement"),
            ("site", "site", "site"),
        ]:
            with self.subTest(source=source, before=before, after=after):
                with self.assertRaises(RuntimeError):
                    runner.change(source, before, after)

    def test_variants_start_independently_from_the_baseline(self):
        self.assertEqual(
            runner.variants("left right", [("first", "left", "A"), ("second", "right", "B")]),
            [("baseline", "left right"), ("first", "A right"), ("second", "left B")],
        )

    def test_preflight_reports_the_stale_group_and_source(self):
        with patch.object(runner, "POLICY_MUTATIONS", [("stale", "not a mutation site", "")]):
            with self.assertRaisesRegex(RuntimeError, r"Acceptance \(vir/src/infer_proof/policy.rs\)"):
                runner.preflight()


class MutationExecutionTests(unittest.TestCase):
    def run_result(self, output, returncode, should_pass):
        with patch.object(runner.subprocess, "run", return_value=subprocess.CompletedProcess(
            ["tests"], returncode, stdout=output, stderr="",
        )):
            return runner.run(["tests"], runner.ROOT, {}, should_pass=should_pass)

    def test_completed_success_and_failure_are_recognized_including_one_test(self):
        for passed, failed, should_pass in [(1, 0, True), (2, 0, True), (0, 1, False), (2, 1, False)]:
            output = result_output("ok" if should_pass else "FAILED", passed, failed)
            with self.subTest(passed=passed, failed=failed):
                self.assertEqual(self.run_result(output, 0 if should_pass else 101, should_pass), output)

    def test_compile_errors_crashes_empty_filters_and_survivors_are_not_detection(self):
        passed = result_output()
        failed = result_output("FAILED", 0, 1)
        cases = [
            ("error[E0425]: cannot find value\n", 101),
            ("running 2 tests\nerror: could not compile `vir`\n", 101),
            (passed + "error: could not compile another target\n", 101),
            ("running 2 tests\ntest some_test ... FAILED\n", -9),
            (result_output(passed=0), 101),
            (result_output(passed=0, ignored=2), 101),
            (result_output("FAILED", 0, 1, started=2), 101),
            (passed, 0),
            (passed, 101),
            (failed, 0),
            (failed, -9),
            (passed + failed, 101),
        ]
        for output, returncode in cases:
            with self.subTest(output=output, returncode=returncode):
                with self.assertRaises(RuntimeError):
                    self.run_result(output, returncode, False)

    def test_baseline_must_succeed_with_executed_tests(self):
        for output, returncode in [
            (result_output("FAILED", 0, 1), 101),
            (result_output(), 101),
            (result_output(passed=0), 0),
            (result_output(passed=0, ignored=1), 0),
        ]:
            with self.subTest(output=output, returncode=returncode):
                with self.assertRaises(RuntimeError):
                    self.run_result(output, returncode, True)

    def test_compilation_failure_is_never_a_detected_mutant(self):
        with patch.object(runner.subprocess, "run", return_value=subprocess.CompletedProcess(
            ["rustc"], 1, stdout=result_output("FAILED", 0, 1), stderr="compile error",
        )):
            with self.assertRaisesRegex(RuntimeError, "compilation failed"):
                runner.compile_tests(["rustc"], runner.ROOT, {})

    def test_cargo_only_compiles_and_selects_the_vir_test_executable(self):
        artifact = dict(reason="compiler-artifact", target=dict(name="vir"), executable="/tmp/vir-tests")
        with patch.object(runner, "compile_tests", return_value=json.dumps(artifact)) as compile_tests:
            self.assertEqual(runner.vir_test_binary(runner.SOURCE, {}), "/tmp/vir-tests")
            command = compile_tests.call_args.args[0]
            self.assertIn("--no-run", command)
            self.assertIn("--lib", command)

    def test_missing_or_ambiguous_test_executables_are_harness_errors(self):
        artifact = json.dumps(dict(reason="compiler-artifact", target=dict(name="vir"), executable="tests"))
        for output in ["", artifact + "\n" + artifact]:
            with self.subTest(output=output):
                with patch.object(runner, "compile_tests", return_value=output):
                    with self.assertRaisesRegex(RuntimeError, "Expected one VIR test executable"):
                        runner.vir_test_binary(runner.SOURCE, {})

    def test_policy_only_preflights_but_does_not_run_vir(self):
        with patch("sys.argv", ["mutations", "--policy-only"]), \
                patch.object(runner, "preflight") as preflight, \
                patch.object(runner, "policy_tests") as policy_tests, \
                patch.object(runner, "precision_tests") as precision_tests:
            runner.main()
        preflight.assert_called_once_with()
        policy_tests.assert_called_once()
        precision_tests.assert_not_called()


if __name__ == "__main__":
    unittest.main()
