"""Receipt rejection tests for the explicitly authored MODEL-ONLY proof."""

import copy
import json
from pathlib import Path
import unittest

import check_fedimint_atomic_replay as model


SOURCE = Path("/private/tmp/fresh-atomic/model.rs")


def positive_fixture():
    rows = [dict(function=name, rust_function=name.split("::")[-1], body_status=status,
        source=str(SOURCE) + ":1:1: 1:10", selected=True, automatically_derived=False,
        inference_candidate=False, ordinary_contract=status != "specification_definition_or_declaration")
        for name, status in sorted(model.expected_rows().items())]
    return {"verification-results": dict(success=True, verified=18, errors=0,
        **{"encountered-error": False, "encountered-vir-error": False, "is-verifying-entire-crate": True}),
        "verus-infer": dict(status="no_inference", crate_success=True, interface_written=False,
            requested_functions=0, generated_functions=0, functions=[], failed_functions=[],
            imported_inference_dependencies=[], trusted_models=[], metrics=dict(total_micros=1),
            verification_scope=dict(native_collection_completed=True, complete_selected_scope_verified=True,
                checked_exec_bodies=1, checked_proof_bodies=16, native_functions=rows))}


class FedimintAtomicReplayTests(unittest.TestCase):
    def rejects(self, mutate):
        fixture = positive_fixture()
        mutate(fixture)
        with self.assertRaises((ValueError, KeyError, TypeError)):
            model.validate_positive(fixture, 0, SOURCE)

    def test_valid_complete_model_scope(self):
        accepted = model.validate_positive(positive_fixture(), 0, SOURCE)
        self.assertEqual(accepted["checked_exec_bodies"], 1)
        self.assertEqual(accepted["checked_proof_bodies"], 16)
        self.assertIs(accepted["inference"], False)

    def test_nonzero_returncode_rejects(self):
        with self.assertRaises(ValueError):
            model.validate_positive(positive_fixture(), 1, SOURCE)

    def test_boolean_returncode_rejects(self):
        with self.assertRaises(ValueError):
            model.validate_positive(positive_fixture(), False, SOURCE)

    def test_positive_result_flags_are_exact_booleans(self):
        for field in ("success", "encountered-error", "encountered-vir-error", "is-verifying-entire-crate"):
            for value in (None, 0, 1, "true", []):
                with self.subTest(field=field, value=value):
                    self.rejects(lambda data: data["verification-results"].__setitem__(field, value))

    def test_nonzero_or_missing_errors_reject(self):
        for value in (1, -1, False, None, "0"):
            with self.subTest(value=value):
                self.rejects(lambda data: data["verification-results"].__setitem__("errors", value))
        self.rejects(lambda data: data["verification-results"].pop("errors"))

    def test_exact_verified_count_rejects_partial_fake_or_typed_counts(self):
        for value in (17, 19, True, 18.0, "18", None):
            with self.subTest(value=value):
                self.rejects(lambda data: data["verification-results"].__setitem__("verified", value))

    def test_scope_flags_are_exact_booleans(self):
        for field in ("native_collection_completed", "complete_selected_scope_verified"):
            for value in (False, 1, None, "true"):
                with self.subTest(field=field, value=value):
                    self.rejects(lambda data: data["verus-infer"]["verification_scope"].__setitem__(field, value))

    def test_scope_counts_are_exact_integers(self):
        for field, expected in (("checked_exec_bodies", 1), ("checked_proof_bodies", 16)):
            for value in (False, True, expected - 1, expected + 1, str(expected), float(expected)):
                with self.subTest(field=field, value=value):
                    self.rejects(lambda data: data["verus-infer"]["verification_scope"].__setitem__(field, value))

    def test_missing_scope_rejects(self):
        self.rejects(lambda data: data["verus-infer"].pop("verification_scope"))

    def test_missing_body_rejects(self):
        self.rejects(lambda data: data["verus-infer"]["verification_scope"]["native_functions"].pop())

    def test_duplicate_body_rejects(self):
        self.rejects(lambda data: data["verus-infer"]["verification_scope"]["native_functions"].append(
            copy.deepcopy(data["verus-infer"]["verification_scope"]["native_functions"][0])))

    def test_consistent_identity_rename_rejects(self):
        def rename(data):
            for row in data["verus-infer"]["verification_scope"]["native_functions"]:
                row["function"] = row["function"].replace(model.CRATE, "native_async_engine")
        self.rejects(rename)

    def test_fake_checked_body_status_rejects(self):
        self.rejects(lambda data: data["verus-infer"]["verification_scope"]["native_functions"][0].__setitem__(
            "body_status", "checked_exec_body"))

    def test_unselected_derived_inferred_or_malformed_contract_rows_reject(self):
        for field in ("selected", "automatically_derived", "inference_candidate", "ordinary_contract"):
            for value in (None, 0, 1, "false"):
                with self.subTest(field=field, value=value):
                    self.rejects(lambda data: data["verus-infer"]["verification_scope"]["native_functions"][0].__setitem__(
                        field, value))
        self.rejects(lambda data: data["verus-infer"]["verification_scope"]["native_functions"][0].__setitem__(
            "selected", False))

    def test_source_identity_rejects(self):
        self.rejects(lambda data: data["verus-infer"]["verification_scope"]["native_functions"][0].__setitem__(
            "source", "/native/engine.rs:974:1"))

    def test_rust_function_identity_rejects(self):
        self.rejects(lambda data: data["verus-infer"]["verification_scope"]["native_functions"][0].__setitem__(
            "rust_function", "process_consensus_item_attempt"))

    def test_inference_and_publication_misclassification_rejects(self):
        for field, value in (("status", "proved"), ("crate_success", 1), ("interface_written", True),
                             ("requested_functions", True), ("generated_functions", 1)):
            with self.subTest(field=field):
                self.rejects(lambda data: data["verus-infer"].__setitem__(field, value))

    def test_imports_trusted_models_and_inference_rows_reject(self):
        for field in ("functions", "failed_functions", "imported_inference_dependencies", "trusted_models"):
            with self.subTest(field=field):
                self.rejects(lambda data: data["verus-infer"].__setitem__(field, [dict(fake=True)]))

    def test_positive_work_is_integer_and_nonzero(self):
        for value in (False, True, 0, -1, 1.0, "1", None):
            with self.subTest(value=value):
                self.rejects(lambda data: data["verus-infer"]["metrics"].__setitem__("total_micros", value))

    def test_frozen_definitions_and_body_inventory(self):
        frozen = json.loads((model.EVIDENCE / "scope.json").read_text())
        self.assertEqual(frozen["model_sha256"], model.digest(model.EVIDENCE / "model.rs"))
        self.assertEqual(frozen["functions"], model.model_inventory((model.EVIDENCE / "model.rs").read_text()))
        self.assertEqual(len(frozen["functions"]), 32)

    def test_model_has_no_trusted_application_escape(self):
        source = (model.EVIDENCE / "model.rs").read_text()
        for forbidden in ("external_body", "assume(", "admit(", "#[verifier::infer]", "unimplemented!"):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, source)

    def test_added_unknown_model_function_rejects_inventory(self):
        source = (model.EVIDENCE / "model.rs").read_text()
        with self.assertRaises(ValueError):
            model.model_inventory(model.appended_source(source, "pub proof fn unknown() {}"))

    def test_negative_goal_insertion_preserves_existing_definitions(self):
        source = (model.EVIDENCE / "model.rs").read_text()
        for label, goal in model.NEGATIVE_GOALS.items():
            with self.subTest(label=label):
                changed = model.appended_source(source, goal)
                self.assertIn(source.rsplit("\n}", 1)[0], changed)
                self.assertEqual(changed.count(goal), 1)

    def test_frontend_or_resource_error_is_not_semantic_evidence(self):
        data = positive_fixture()
        data["verification-results"].update(success=False, errors=1)
        data["verus-infer"]["crate_success"] = False
        data["verus-infer"]["status"] = "caller_proof_failed"
        for stderr in ("syntax error", "postcondition not satisfied; panicked at solver",
                       "postcondition not satisfied; resource limit"):
            with self.subTest(stderr=stderr):
                with self.assertRaises(ValueError):
                    model.validate_negative(data, 1, stderr)
        data["verification-results"]["encountered-vir-error"] = True
        with self.assertRaises(ValueError):
            model.validate_negative(data, 1, "postcondition not satisfied")

    def test_expected_semantic_and_safety_failures_accept(self):
        data = positive_fixture()
        data["verification-results"].update(success=False, errors=1)
        data["verus-infer"]["crate_success"] = False
        data["verus-infer"]["status"] = "caller_proof_failed"
        self.assertEqual(model.validate_negative(data, 1, "postcondition not satisfied")["failure"], "semantic")
        self.assertEqual(model.validate_negative(data, 1, "possible arithmetic overflow", safety=True)["failure"], "safety")

    def test_git_environment_is_sanitized_and_scratch_scoped(self):
        environment = model.sanitized_environment(Path("/private/tmp/atomic-fresh"), Path("/sealed/tools77"))
        self.assertEqual({name for name in environment if name.startswith("GIT_")}, {"GIT_OPTIONAL_LOCKS"})
        self.assertEqual(environment["GIT_OPTIONAL_LOCKS"], "0")
        self.assertEqual(environment["CARGO_HOME"], "/private/tmp/atomic-fresh/unused-cargo-home")

    def test_existing_output_rejects_without_writes(self):
        with self.assertRaisesRegex(ValueError, "fresh"):
            model.replay(Path("/sealed"), model.EVIDENCE)


if __name__ == "__main__":
    unittest.main()
