import unittest

import check_native_resource_boundaries as boundaries


class BoundaryReplayTests(unittest.TestCase):
    def passing_log(self):
        outcomes = "\n".join(
            f"test import_export::native_resource_boundary_tests::{name} ... ok"
            for name in boundaries.TESTS
        )
        return outcomes + "\ntest result: ok. 5 passed; 0 failed; 0 ignored; 1 filtered out;\n"

    def test_profiles_build_before_tests_with_explicit_offline_locked_commands(self):
        plan = boundaries.command_plan(boundaries.PROFILES)
        self.assertEqual([label for label, _ in plan], [
            "default-off-build", "default-off-tests", "feature-on-build", "feature-on-tests",
        ])
        for _, command in plan:
            self.assertEqual(command[0], "cargo")
            self.assertIn("--offline", command)
            self.assertIn("--locked", command)
        for _, command in plan[:2]:
            self.assertNotIn("--features", command)
        for _, command in plan[2:]:
            self.assertIn("vir/experimental-native-resource", command)
        for _, command in plan[1::2]:
            self.assertIn("--include-ignored", command)
            self.assertIn("--test-threads=1", command)

    def test_acceptance_requires_every_named_test_and_no_ignored_tests(self):
        passing = self.passing_log()
        self.assertTrue(boundaries.checked_test_log(passing))
        self.assertFalse(boundaries.checked_test_log(passing.replace(boundaries.TESTS[-1], "missing")))
        self.assertFalse(boundaries.checked_test_log(passing.replace("5 passed", "0 passed")))
        self.assertFalse(boundaries.checked_test_log(passing.replace("0 ignored", "1 ignored")))
        self.assertFalse(boundaries.checked_test_log(passing.replace("0 failed", "1 failed")))

    def test_names_mentioned_in_diagnostics_are_not_completed_tests(self):
        mentions = "\n".join(
            f"diagnostic: import_export::native_resource_boundary_tests::{name}"
            for name in boundaries.TESTS
        )
        summary = "\ntest result: ok. 5 passed; 0 failed; 0 ignored;\n"
        self.assertFalse(boundaries.checked_test_log(mentions + summary))
        self.assertFalse(boundaries.checked_test_log(
            self.passing_log().replace(" ... ok", "_different ... ok")
        ))

    def test_each_named_test_must_finish_successfully(self):
        for status in ("FAILED", "ignored", "ignored, reason"):
            with self.subTest(status=status):
                self.assertFalse(boundaries.checked_test_log(
                    self.passing_log().replace(" ... ok", f" ... {status}", 1)
                ))
        self.assertFalse(boundaries.checked_test_log(
            self.passing_log().replace(" ... ok", " ... diagnostic only", 1)
        ))

    def test_nocapture_output_can_separate_start_from_success(self):
        split = self.passing_log().replace(
            " ... ok", " ... driver output\nmore output\nok", 1
        )
        self.assertTrue(boundaries.checked_test_log(split))
        self.assertFalse(boundaries.checked_test_log(split.replace("\nok\n", "\n", 1)))
        self.assertFalse(boundaries.checked_test_log(split.replace("\nok\n", "\nFAILED\n", 1)))

    def test_duplicate_unknown_and_concatenated_test_runs_are_rejected(self):
        passing = self.passing_log()
        first = passing.splitlines()[0] + "\n"
        self.assertFalse(boundaries.checked_test_log(first + passing))
        self.assertFalse(boundaries.checked_test_log("test unrelated::control ... ok\n" + passing))
        self.assertFalse(boundaries.checked_test_log(passing + passing))

    def test_summary_is_required_once_after_completed_tests(self):
        passing = self.passing_log()
        outcomes, summary = passing.rsplit("test result:", 1)
        self.assertFalse(boundaries.checked_test_log(outcomes))
        self.assertFalse(boundaries.checked_test_log("test result:" + summary + outcomes))
        self.assertFalse(boundaries.checked_test_log(passing + "test result:" + summary))
        self.assertFalse(boundaries.checked_test_log(passing.replace("5 passed", "6 passed")))

    def test_identity_includes_production_guard_tests_and_configuration(self):
        identity = boundaries.source_identity()
        for name in (
            "rust-toolchain.toml", "source/Cargo.lock", "source/.cargo/config.toml",
            "source/rust_verify/src/import_export.rs", "source/rust_verify/src/verifier.rs",
            "source/vir/src/native_resource.rs",
            "source/rust_verify/src/import_export/native_resource_boundary_tests.rs",
        ):
            self.assertIn(name, identity)
            self.assertEqual(len(identity[name]), 64)


if __name__ == "__main__":
    unittest.main()
