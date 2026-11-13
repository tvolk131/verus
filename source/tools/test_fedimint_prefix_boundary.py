"""Focused fail-closed reader and exact-source adaptation tests."""

import copy
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import check_fedimint_prefix_boundary as boundary


SOURCE = Path("/private/tmp/portable-prefix-test/source.rs")
PIECES = boundary.read_json(boundary.EVIDENCE / "native-items.json")
SCOPES = boundary.read_json(boundary.EVIDENCE / "expected-scope.json")


def positive_fixture(crate="prefix_provider"):
    source = boundary.render(PIECES) if crate == "prefix_provider" else (boundary.EVIDENCE / "consumer.verus.rs").read_text()
    expected = SCOPES[crate]
    rows, unused = boundary.expected_scope(source, SOURCE, crate, expected)
    data = {"verification-results": dict(success=True, verified=expected["verified"], errors=0,
        **{"encountered-error": False, "encountered-vir-error": False, "is-verifying-entire-crate": True}),
        "verus-infer": dict(schema_version=1, functions=[], failed_functions=[], trusted_models=[],
            imported_inference_dependencies=[], interface_written=False, status="no_inference", crate_success=True,
            generation_failure=None, requested_functions=0, generated_functions=0,
            verification_scope=dict(native_collection_completed=True, complete_selected_scope_verified=True,
                checked_exec_bodies=expected["checked_exec_bodies"], checked_proof_bodies=expected["checked_proof_bodies"],
                native_functions=rows))}
    return data, source, expected


def failure_fixture(safety=False):
    data, source, expected = positive_fixture()
    label, source, failed, actual_safety = next(control for control in boundary.controls(source, PIECES)
        if control[3] == safety)
    data["verification-results"].update(success=False, verified=10 if safety else 9, errors=1,
        **{"encountered-error": True})
    data["verus-infer"].update(status="caller_proof_failed", crate_success=False,
        failed_functions=["prefix_provider::" + failed])
    marker = "fn " + failed.split("::")[-1]
    start = source.index(marker)
    line = source.count("\n", 0, start) + 1
    diagnostic = "possible arithmetic underflow/overflow" if safety else "postcondition not satisfied"
    stderr = f"error: {diagnostic}\n  --> {SOURCE}:{line}:5\n"
    return data, source, failed, stderr


class PrefixBoundaryTests(unittest.TestCase):
    def accept(self, data=None, crate="prefix_provider", code=0):
        fixture, source, expected = positive_fixture(crate)
        return boundary.validate_positive(fixture if data is None else data, code, source, SOURCE, crate, expected)

    def reject(self, mutate):
        data, source, expected = positive_fixture()
        mutate(data)
        with self.assertRaises((ValueError, KeyError, TypeError)):
            boundary.validate_positive(data, 0, source, SOURCE, "prefix_provider", expected)

    def test_scope_fixture_counts(self):
        self.assertEqual(len(SCOPES["prefix_provider"]["functions"]), 13)
        self.assertEqual(len(SCOPES["prefix_consumer"]["functions"]), 3)

    def test_valid_provider_and_imported_consumer(self):
        for crate in SCOPES:
            self.assertEqual(self.accept(crate=crate)["inferred_functions"], 0)

    def test_returncode_is_typed_zero(self):
        for code in (False, True, 1, None, "0", 0.0):
            with self.subTest(code=code), self.assertRaises(ValueError):
                self.accept(code=code)

    def test_all_positive_result_fields_required(self):
        data, unused, expected = positive_fixture()
        for key in data["verification-results"]:
            with self.subTest(key=key):
                self.reject(lambda value: value["verification-results"].pop(key))

    def test_positive_flags_are_exact_booleans(self):
        for key in ("success", "encountered-error", "encountered-vir-error", "is-verifying-entire-crate"):
            for wrong in (None, "true", 0, 1, []):
                with self.subTest(key=key, wrong=wrong):
                    self.reject(lambda data: data["verification-results"].__setitem__(key, wrong))
        self.reject(lambda data: data["verification-results"].__setitem__("success", False))

    def test_positive_counts_exact_and_typed(self):
        for key, correct in (("verified", 10), ("errors", 0)):
            for wrong in (None, str(correct), float(correct), False, correct + 1, correct - 1):
                with self.subTest(key=key, wrong=wrong):
                    self.reject(lambda data: data["verification-results"].__setitem__(key, wrong))

    def test_all_inference_fields_required(self):
        data, unused, expected = positive_fixture()
        for key in data["verus-infer"]:
            with self.subTest(key=key):
                self.reject(lambda value: value["verus-infer"].pop(key))

    def test_no_inference_or_application_trust(self):
        for key in ("functions", "failed_functions", "trusted_models", "imported_inference_dependencies"):
            for wrong in (None, {}, ["invented"]):
                with self.subTest(key=key, wrong=wrong):
                    self.reject(lambda data: data["verus-infer"].__setitem__(key, wrong))

    def test_ordinary_classification_required(self):
        for key, wrong in (("status", "verified"), ("crate_success", False), ("crate_success", 1),
                           ("generation_failure", "unsupported"), ("interface_written", True),
                           ("interface_written", 0), ("schema_version", True)):
            with self.subTest(key=key, wrong=wrong):
                self.reject(lambda data: data["verus-infer"].__setitem__(key, wrong))

    def test_inference_counts_exact_and_typed(self):
        for key in ("requested_functions", "generated_functions"):
            for wrong in (None, "0", 0.0, False, 1):
                with self.subTest(key=key, wrong=wrong):
                    self.reject(lambda data: data["verus-infer"].__setitem__(key, wrong))

    def test_all_scope_fields_required(self):
        data, unused, expected = positive_fixture()
        for key in data["verus-infer"]["verification_scope"]:
            with self.subTest(key=key):
                self.reject(lambda value: value["verus-infer"]["verification_scope"].pop(key))

    def test_complete_scope_flags(self):
        for key in ("native_collection_completed", "complete_selected_scope_verified"):
            for wrong in (False, None, 1, "true"):
                with self.subTest(key=key, wrong=wrong):
                    self.reject(lambda data: data["verus-infer"]["verification_scope"].__setitem__(key, wrong))

    def test_checked_body_counts(self):
        for key in ("checked_exec_bodies", "checked_proof_bodies"):
            for wrong in (0, 4, 6, 5.0, True, "5", None):
                with self.subTest(key=key, wrong=wrong):
                    self.reject(lambda data: data["verus-infer"]["verification_scope"].__setitem__(key, wrong))

    def test_each_scope_identity_field_required(self):
        for index in range(13):
            for key in boundary.ROW_KEYS:
                with self.subTest(index=index, key=key):
                    self.reject(lambda data: data["verus-infer"]["verification_scope"]["native_functions"][index].pop(key))

    def test_each_scope_identity_field_bound(self):
        for index in range(13):
            for key in boundary.ROW_KEYS:
                with self.subTest(index=index, key=key):
                    self.reject(lambda data: data["verus-infer"]["verification_scope"]["native_functions"][index].__setitem__(key, "wrong"))

    def test_boolean_scope_identity_fields_are_not_integers(self):
        for key in ("selected", "automatically_derived", "inference_candidate", "ordinary_contract"):
            self.reject(lambda data: data["verus-infer"]["verification_scope"]["native_functions"][0].__setitem__(key, 1))

    def test_same_length_duplicate_and_missing_identity(self):
        self.reject(lambda data: data["verus-infer"]["verification_scope"]["native_functions"].__setitem__(0,
            copy.deepcopy(data["verus-infer"]["verification_scope"]["native_functions"][1])))

    def test_alias_source_span_is_not_accepted(self):
        self.reject(lambda data: data["verus-infer"]["verification_scope"]["native_functions"][0].__setitem__(
            "source", str(SOURCE) + ".alias:1:1: 1:2"))

    def test_body_identities_are_hashed_and_labelled(self):
        identities = self.accept()["identities"]
        self.assertEqual(sum(row["origin"] == "native-body" for row in identities.values()), 3)
        self.assertTrue(all(len(row["body_sha256"]) == 64 for row in identities.values()))

    def test_native_bodies_exact_in_both_modes(self):
        for inference in (True, False):
            rows = boundary.correspondence(boundary.render(PIECES, inference), PIECES)
            self.assertEqual(len(rows), 3)
            self.assertTrue(all(row["byte_identical"] is True for row in rows.values()))

    def test_native_body_projection_or_rewrite_rejects(self):
        source = boundary.render(PIECES)
        for original, replacement in (("self.prefix.clone()", "Vec::new()"),
            ("full_key.extend_from_slice(key)", "full_key.extend_from_slice(&[])"),
            ("self.get_full_key(range.end)", "self.get_full_key(range.start)")):
            with self.subTest(original=original), self.assertRaises(ValueError):
                boundary.correspondence(source.replace(original, replacement, 1), PIECES)

    def test_original_record_fields_and_types_retained(self):
        source = boundary.render(PIECES)
        for name in ("database_record", "transaction_record"):
            original = PIECES[name]
            owner = "PrefixDatabase" if name == "database_record" else "PrefixDatabaseTransaction"
            selected = boundary.item(source, "pub struct " + owner + "<Inner>")
            selected = selected.removeprefix("pub ")
            for field in ("prefix", "inner", "global_dbtx_access_token"):
                selected = selected.replace("pub " + field + ":", field + ":")
            self.assertEqual(selected, original)
        self.assertIn(PIECES["token"], source)

    def test_contracts_are_not_inferred_or_unverified(self):
        source = boundary.render(PIECES)
        self.assertEqual(source.count("ensures full_key@ == prefix_view(self.prefix@, key@)"), 2)
        self.assertIn("full_range.end@ == prefix_view(self.prefix@, range.end@)", source)
        for forbidden in ("external_body", "assume(", "admit(", "#[verifier::infer]"):
            self.assertNotIn(forbidden, source)

    def test_infer_probe_has_original_three_bodies_no_contracts(self):
        source = boundary.render(PIECES, inference=True)
        self.assertEqual(source.count("#[verifier::infer]"), 3)
        self.assertNotIn("ensures", source)
        self.assertNotIn("prefix_view", source)

    def test_every_control_changes_source_and_targets_body(self):
        source = boundary.render(PIECES)
        controls = boundary.controls(source, PIECES)
        self.assertEqual(len(controls), 7)
        self.assertEqual(len({row[0] for row in controls}), 7)
        self.assertTrue(all(row[1] != source for row in controls))
        self.assertEqual(sum(row[3] for row in controls), 1)

    def test_duplicate_mutation_anchor_rejects(self):
        for source in ("", "xx"):
            with self.assertRaises(ValueError):
                boundary.mutate_once(source, "x", "y")

    def test_valid_semantic_and_safety_classifications(self):
        for safety in (False, True):
            data, source, failed, stderr = failure_fixture(safety)
            result = boundary.validate_failure(data, 1, stderr, source, SOURCE, 10 if safety else 9, failed, safety)
            self.assertEqual(result["category"], "safety" if safety else "semantic")

    def test_negative_proof_fields_required(self):
        for safety in (False, True):
            fixture, source, failed, stderr = failure_fixture(safety)
            for key in fixture["verification-results"]:
                data = copy.deepcopy(fixture)
                data["verification-results"].pop(key)
                with self.subTest(key=key, safety=safety), self.assertRaises((ValueError, KeyError)):
                    boundary.validate_failure(data, 1, stderr, source, SOURCE, 10 if safety else 9, failed, safety)

    def test_negative_wrong_count_and_type_rejects(self):
        for key in ("verified", "errors"):
            for wrong in (False, None, "1", 1.0, 20):
                data, source, failed, stderr = failure_fixture()
                data["verification-results"][key] = wrong
                with self.subTest(key=key, wrong=wrong), self.assertRaises(ValueError):
                    boundary.validate_failure(data, 1, stderr, source, SOURCE, 9, failed)

    def test_negative_generic_failure_never_semantic_credit(self):
        data, source, failed, stderr = failure_fixture()
        for wrong in ("panicked at", "timed out", "resource limit", "out of memory", "precondition not satisfied"):
            with self.subTest(wrong=wrong), self.assertRaises(ValueError):
                boundary.validate_failure(data, 1, stderr + wrong, source, SOURCE, 9, failed)

    def test_negative_wrong_function_or_source_rejects(self):
        data, source, failed, stderr = failure_fixture()
        for wrong in (stderr.replace(str(SOURCE), str(SOURCE) + ".alias"), stderr.replace(":5\n", ":5\n") + stderr,
                      "error: postcondition not satisfied\n  --> " + str(SOURCE) + ":1:1\n"):
            with self.subTest(wrong=wrong), self.assertRaises(ValueError):
                boundary.validate_failure(data, 1, wrong, source, SOURCE, 9, failed)

    def test_frontend_error_never_semantic_credit(self):
        data, source, failed, stderr = failure_fixture()
        data["verification-results"]["encountered-vir-error"] = True
        with self.assertRaises(ValueError):
            boundary.validate_failure(data, 1, stderr, source, SOURCE, 9, failed)

    def test_negative_failed_function_identity_required(self):
        fixture, source, failed, stderr = failure_fixture()
        for wrong in ([], ["other_function"], None):
            data = copy.deepcopy(fixture)
            data["verus-infer"]["failed_functions"] = wrong
            with self.subTest(wrong=wrong), self.assertRaises(ValueError):
                boundary.validate_failure(data, 1, stderr, source, SOURCE, 9, failed)

    def blocker_fixture(self):
        data, source, expected = positive_fixture()
        failure = ('Verus Infer unsupported: calls require an acyclic same-crate inferred callee or certified imported summary: '
                   'Fun(Path(alloc, ["vec" :: "impl&%3" :: "extend_from_slice"]))')
        data["verification-results"].update(success=False, **{"encountered-error": True, "encountered-vir-error": True})
        data["verus-infer"].update(status="unsupported", crate_success=False, requested_functions=3, generated_functions=0,
                                  generation_failure=failure)
        return data, "error: " + failure

    def test_valid_unsupported_infer_blocker(self):
        data, stderr = self.blocker_fixture()
        with tempfile.TemporaryDirectory() as temporary:
            result = boundary.validate_blocker(data, 1, stderr, Path(temporary) / "absent.vir")
        self.assertEqual(result["category"], "frontend-unsupported")

    def test_blocker_seeded_export_must_be_removed(self):
        data, stderr = self.blocker_fixture()
        with tempfile.TemporaryDirectory() as temporary:
            artifact = Path(temporary) / "seeded.vir"
            artifact.write_bytes(b"seed")
            with self.assertRaises(ValueError):
                boundary.validate_blocker(data, 1, stderr, artifact)

    def test_wrong_blocker_or_typed_counts_reject(self):
        fixture, stderr = self.blocker_fixture()
        with tempfile.TemporaryDirectory() as temporary:
            for key, wrong in (("requested_functions", True), ("requested_functions", 0),
                               ("generated_functions", False), ("generated_functions", 1),
                               ("generation_failure", "unrelated frontend error"), ("status", "caller_proof_failed")):
                data = copy.deepcopy(fixture)
                data["verus-infer"][key] = wrong
                with self.subTest(key=key, wrong=wrong), self.assertRaises(ValueError):
                    boundary.validate_blocker(data, 1, stderr, Path(temporary) / "absent.vir")

    def test_safety_diagnostic_not_semantic(self):
        data, source, failed, stderr = failure_fixture(True)
        with self.assertRaises(ValueError):
            boundary.validate_failure(data, 1, stderr, source, SOURCE, 10, failed, False)

    def test_duplicate_json_keys_reject_even_under_optimization(self):
        with self.assertRaises(ValueError):
            json.loads('{"success":true,"success":false}', object_pairs_hook=boundary.unique_object)

    def test_sanitized_private_environment(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            with mock.patch.dict(os.environ, {"GIT_DIR": "/hostile", "GIT_WORK_TREE": "/hostile",
                 "RUSTFLAGS": "hostile", "VERUS_EXTRA_ARGS": "--no-verify"}):
                environment = boundary.environment_for(output, Path("/solver"), Path("/sysroot"))
            self.assertEqual({key: value for key, value in environment.items() if key.startswith("GIT_")},
                             {"GIT_OPTIONAL_LOCKS": "0"})
            for name in ("HOME", "XDG_CACHE_HOME", "CARGO_HOME", "RUSTUP_HOME", "TMPDIR"):
                self.assertTrue(Path(environment[name]).is_relative_to(output))
            self.assertNotIn("RUSTFLAGS", environment)
            self.assertNotIn("VERUS_EXTRA_ARGS", environment)
            self.assertEqual(environment["VERUS_USE_RUSTUP"], "0")

    def test_no_required_historical_paths_in_runner_or_fixtures(self):
        paths = [Path(boundary.__file__), *boundary.EVIDENCE.glob("*.json"), *boundary.EVIDENCE.glob("*.rs")]
        for path in paths:
            text = path.read_text()
            for historical in ("verus-cycle77-", "verus-fedimint-early-returns-", "/.t3/worktrees/"):
                self.assertNotIn(historical, text, str(path))

    def test_current_artifact_guard_detects_input_changes(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "input"
            path.write_text("before")
            guard = boundary.ArtifactGuard([path])
            path.write_text("after")
            with self.assertRaises(RuntimeError):
                guard.check()


if __name__ == "__main__":
    unittest.main()
