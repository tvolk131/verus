"""Synthetic receipt-reader regressions, not substitute application proofs."""

import copy
import json
from pathlib import Path
import tempfile
import unittest

import check_redb_slot_selection as slot


def fixture(source, crate="provider"):
    expected = slot.read_json(slot.EVIDENCE / "expected-scope.json")[crate]
    rows = copy.deepcopy(expected["native_functions"])
    for row in rows:
        row["source"] = str(source) + ":1:1: 1:2"
    return {"verification-results": {"encountered-error": False, "encountered-vir-error": False,
            "success": True, "verified": expected["verified"], "errors": 0, "is-verifying-entire-crate": True},
        "verus-infer": {"crate_success": True, "interface_written": True, "status": "verified",
            "functions": [{"function": crate + expected["inferred_suffix"], "mode": "exact",
                "required_checks_passed": True, "exported": True, "available_to_same_crate_callers": True,
                "contract": copy.deepcopy(expected["contract"]),
                "proofs": {name: {"status": "proved", "required": True}
                           for name in ("soundness", "maximality", "exactness")},
                "metrics": {"solver_micros": 1}}],
            "verification_scope": {"native_collection_completed": True,
                "complete_selected_scope_verified": True, "checked_exec_bodies": expected["checked_exec_bodies"],
                "checked_proof_bodies": expected["checked_proof_bodies"], "native_functions": rows}}}


class PositiveAcceptance(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.directory = Path(self.temporary.name)
        self.source = self.directory / "fixture.rs"
        self.artifact = self.directory / "fixture.vir"
        self.library = self.directory / "libfixture.rlib"
        self.artifact.write_bytes(b"unit fixture, not verified VIR")
        self.library.write_bytes(b"unit fixture, not compiled Rust")
        self.expected = slot.read_json(slot.EVIDENCE / "expected-scope.json")

    def tearDown(self):
        self.temporary.cleanup()

    def accept(self, data, crate="provider", code=0, stderr=""):
        return slot.validate_positive(data, code, stderr, self.artifact, self.library,
                                      self.source, crate, self.expected[crate])

    def reject(self, mutation):
        data = fixture(self.source)
        mutation(data)
        with self.assertRaises((ValueError, KeyError, TypeError)):
            self.accept(data)

    def test_provider_and_importer_fixtures(self):
        for crate in self.expected:
            self.accept(fixture(self.source, crate), crate)

    def test_missing_scope(self):
        self.reject(lambda data: data["verus-infer"].pop("verification_scope"))

    def test_partial_scope(self):
        self.reject(lambda data: data["verus-infer"]["verification_scope"].update(complete_selected_scope_verified=False))

    def test_missing_collection(self):
        self.reject(lambda data: data["verus-infer"]["verification_scope"].update(native_collection_completed=False))

    def test_wrong_counts_and_typed_integers(self):
        for group, key in (("verification-results", "verified"), ("verification-results", "errors")):
            for value in (True, 0.0, "0", -1, 100):
                with self.subTest(group=group, key=key, value=value):
                    self.reject(lambda data: data[group].update({key: value}))
        for key in ("checked_exec_bodies", "checked_proof_bodies"):
            for value in (True, 1.0, "1", -1, 100):
                with self.subTest(key=key, value=value):
                    self.reject(lambda data: data["verus-infer"]["verification_scope"].update({key: value}))

    def test_false_or_nonnative_success_flags(self):
        fields = (("verification-results", "success"), ("verification-results", "is-verifying-entire-crate"),
                  ("verus-infer", "crate_success"), ("verus-infer", "interface_written"))
        for group, key in fields:
            for value in (False, 1, "true", None):
                with self.subTest(group=group, key=key, value=value):
                    self.reject(lambda data: data[group].update({key: value}))

    def test_frontend_error_not_positive(self):
        for key in ("encountered-error", "encountered-vir-error"):
            self.reject(lambda data: data["verification-results"].update({key: True}))

    def test_missing_native_body(self):
        self.reject(lambda data: data["verus-infer"]["verification_scope"]["native_functions"].pop())

    def test_duplicate_native_body(self):
        self.reject(lambda data: data["verus-infer"]["verification_scope"]["native_functions"].append(
            copy.deepcopy(data["verus-infer"]["verification_scope"]["native_functions"][0])))

    def test_consistently_renamed_native_body(self):
        def rename(data):
            row = next(row for row in data["verus-infer"]["verification_scope"]["native_functions"]
                       if row["function"].endswith("::select_primary_slot"))
            row.update(function="provider::fake_selection", rust_function="fake_selection")
        self.reject(rename)

    def test_native_boolean_flags_strict(self):
        for key in ("selected", "automatically_derived", "inference_candidate", "ordinary_contract"):
            for value in (0, 1, "false", None):
                with self.subTest(key=key, value=value):
                    self.reject(lambda data: data["verus-infer"]["verification_scope"]["native_functions"][0].update({key: value}))

    def test_inference_identity_and_contract(self):
        self.reject(lambda data: data["verus-infer"]["functions"][0].update(function="provider::wrong"))
        self.reject(lambda data: data["verus-infer"]["functions"][0].update(contract={"requires": "false"}))
        self.reject(lambda data: data["verus-infer"].update(functions=[]))

    def test_required_proof_acceptance_strict(self):
        for key in ("required_checks_passed", "exported", "available_to_same_crate_callers"):
            for value in (False, 1, "true"):
                with self.subTest(key=key, value=value):
                    self.reject(lambda data: data["verus-infer"]["functions"][0].update({key: value}))
        for name in ("soundness", "maximality", "exactness"):
            for value in (False, 1):
                self.reject(lambda data: data["verus-infer"]["functions"][0]["proofs"][name].update(required=value))

    def test_solver_work_is_positive_integer(self):
        for value in (0, -1, True, 1.0, "1", None):
            self.reject(lambda data: data["verus-infer"]["functions"][0]["metrics"].update(solver_micros=value))

    def test_body_from_other_source(self):
        def change(data):
            row = next(row for row in data["verus-infer"]["verification_scope"]["native_functions"]
                       if row["body_status"] == "checked_exec_body")
            row["source"] = "other.rs:1:1: 1:2"
        self.reject(change)

    def test_missing_compiled_erasure_or_export(self):
        self.library.unlink()
        with self.assertRaises(ValueError):
            self.accept(fixture(self.source))
        self.library.write_bytes(b"fixture")
        self.artifact.unlink()
        with self.assertRaises(ValueError):
            self.accept(fixture(self.source))

    def test_process_failure_and_resource_messages(self):
        for code in (1, True):
            with self.assertRaises(ValueError):
                self.accept(fixture(self.source), code=code)
        for marker in ("panicked at", "resource limit", "timed out", "out of memory"):
            with self.assertRaises(ValueError):
                self.accept(fixture(self.source), stderr=marker)


class CertificateAcceptance(unittest.TestCase):
    def setUp(self):
        self.certificate = slot.read_json(slot.LIVE_EVIDENCE / "redb-provider-certificate.json")

    def encoded(self):
        ordered = {"schema": self.certificate["schema"], **self.certificate}
        return b"unit-fixture-not-VIR" + json.dumps(ordered, separators=(",", ":")).encode() + b"VRINFER1"

    def test_certificate_fixture(self):
        certificate, dependency = slot.validate_certificate(self.encoded())
        self.assertEqual(certificate, self.certificate)
        self.assertIs(type(dependency[0]["crate_id"]), int)

    def test_wrong_schema_policy_producer(self):
        for key, value in (("schema", True), ("schema", 37), ("policy", "old"), ("producer", "other")):
            with self.subTest(key=key):
                original = self.certificate[key]
                self.certificate[key] = value
                with self.assertRaises(ValueError):
                    slot.validate_certificate(self.encoded())
                self.certificate[key] = original

    def test_false_and_untyped_proof_flags(self):
        for key in ("soundness", "maximality", "exactness"):
            for value in (False, 1):
                self.certificate[key] = value
                with self.assertRaises(ValueError):
                    slot.validate_certificate(self.encoded())
                self.certificate[key] = True

    def test_application_model_not_admitted(self):
        self.certificate["trusted_models"][0][0]["path"]["krate"] = "Application"
        with self.assertRaises(ValueError):
            slot.validate_certificate(self.encoded())

    def test_standard_model_fingerprint_change(self):
        self.certificate["trusted_models"][0][1] = "wrong"
        with self.assertRaises(ValueError):
            slot.validate_certificate(self.encoded())

    def test_missing_exported_proofs_or_extra_dependency(self):
        self.certificate["proofs"] = []
        with self.assertRaises(ValueError):
            slot.validate_certificate(self.encoded())

    def test_wrong_exported_proof_identity_and_summary(self):
        original = copy.deepcopy(self.certificate)
        self.certificate["proofs"][0][0]["path"]["segments"][-1] = "wrong"
        with self.assertRaises(ValueError):
            slot.validate_certificate(self.encoded())
        self.certificate = original
        self.certificate["summaries"] = "wrong"
        with self.assertRaises(ValueError):
            slot.validate_certificate(self.encoded())
        self.certificate = slot.read_json(slot.EVIDENCE / "provider-certificate.json")
        self.certificate["dependencies"] = [["unproved", "summary"]]
        with self.assertRaises(ValueError):
            slot.validate_certificate(self.encoded())

    def test_duplicate_json_and_missing_trailer(self):
        with self.assertRaises(ValueError):
            slot.metadata(b'{"schema":38,"schema":38}VRINFER1')
        with self.assertRaises(ValueError):
            slot.metadata(b'{"schema":38}')


class SourceFidelity(unittest.TestCase):
    def pieces(self):
        return {file_path.stem: file_path.read_text().removesuffix("\n")
                for file_path in (slot.EVIDENCE / "native-items").glob("*.rs")}

    def test_selected_body_bytes_unchanged(self):
        pieces = self.pieces()
        source = slot.render(pieces)
        ledger = slot.read_json(slot.EVIDENCE / "source-ledger.json")
        for name in ("select_primary_slot", "primary_slot", "secondary_slot", "swap_primary_slot"):
            self.assertEqual(source.count(slot.body(pieces[name])), 1)
            self.assertEqual(slot.sha256(slot.body(pieces[name]).encode()), ledger["items"][name]["body_sha256"])

    def test_all_native_record_fields_preserved(self):
        pieces = self.pieces()
        source = slot.render(pieces)
        for name in ("PageNumber", "BtreeHeader", "TransactionHeader", "DatabaseHeader", "UnrepairedDatabaseHeader"):
            for line in pieces[name].splitlines()[1:-1]:
                if ":" in line and not line.strip().startswith("//"):
                    self.assertIn(line.strip().removeprefix("pub(crate) ").removeprefix("pub(super) "), source)

    def test_frame_mentions_every_unchanged_header_field(self):
        specification = (slot.EVIDENCE / "selection.spec.rs").read_text()
        fields = [line.strip().split(":")[0].split()[-1]
                  for line in self.pieces()["DatabaseHeader"].splitlines()[1:-1]]
        for field in fields:
            if field != "primary_slot":
                self.assertIn("after." + field + " == before." + field, specification)

    def test_full_declaration_probe_and_reduced_scaffold_are_distinct(self):
        pieces = self.pieces()
        reduced, full = slot.render(pieces), slot.render(pieces, full_errors=True)
        self.assertIn("pub enum StorageError { Corrupted(String) }", reduced)
        self.assertNotIn("Io(io::Error)", reduced)
        self.assertIn(pieces["StorageError"], full)
        self.assertIn("Err(_) => false", full)
        self.assertIn(pieces["TransactionId"].replace("pub(crate)", "pub").replace("TransactionId(u64)", "TransactionId(pub u64)"), reduced)

    def test_native_harness_uses_full_errors_and_unchanged_bodies(self):
        pieces = self.pieces()
        native = slot.native_harness(pieces)
        self.assertIn(pieces["StorageError"], native)
        self.assertNotIn("verus!", native)
        for name in ("select_primary_slot", "primary_slot", "secondary_slot", "swap_primary_slot"):
            self.assertEqual(native.count(slot.body(pieces[name])), 1)

    def test_semantic_mutations_are_named_and_unique(self):
        source = slot.render(self.pieces())
        controls = slot.controls(source)
        self.assertEqual(len(controls), 6)
        for mutant, category in controls.values():
            self.assertNotEqual(mutant, source)
            self.assertIn(category, ("semantic", "safety"))
        with self.assertRaises(ValueError):
            slot.mutate_once("twice twice", "twice", "different")

    def test_sanitized_git_and_private_cargo_context(self):
        environment = slot.environment_for(Path("/private/tmp/unit-slot"), Path("/private/tmp/tools/verus"))
        self.assertEqual({key for key in environment if key.startswith("GIT_")}, {"GIT_OPTIONAL_LOCKS"})
        self.assertEqual(environment["GIT_OPTIONAL_LOCKS"], "0")
        self.assertEqual(environment["CARGO_NET_OFFLINE"], "true")
        self.assertEqual(environment["CARGO_HOME"], "/private/tmp/unit-slot/unused-cargo-home")


class NegativeAcceptance(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.artifact = Path(self.temporary.name) / "negative.vir"
        self.data = {"verification-results": {"success": False, "encountered-vir-error": False,
            "is-verifying-entire-crate": True, "errors": 1, "verified": 22},
            "verus-infer": {"crate_success": False, "interface_written": False, "functions": []}}

    def tearDown(self):
        self.temporary.cleanup()

    def validate(self, category="semantic", stderr="postcondition not satisfied", code=1):
        self.data["verification-results"]["verified"] = {"semantic": 22, "safety": 23, "legacy": 0}[category]
        return slot.validate_negative(self.data, code, stderr, self.artifact, category)

    def test_expected_semantic_and_safety_failures(self):
        self.validate()
        self.validate("safety", "precondition not satisfied")

    def test_seeded_export_must_be_removed_only_for_infer_path(self):
        self.artifact.write_bytes(b"prior export")
        with self.assertRaises(ValueError):
            self.validate()
        self.validate("legacy")
        self.assertTrue(self.artifact.exists())

    def test_different_failure_reasons_rejected(self):
        for stderr in ("precondition not satisfied", "resource limit", "panicked at", "postcondition not satisfied timed out"):
            with self.assertRaises(ValueError):
                self.validate(stderr=stderr)

    def test_untyped_counts_and_unexpected_success(self):
        for value in (True, 1.0, "1", 0):
            self.data["verification-results"]["errors"] = value
            with self.assertRaises(ValueError):
                self.validate()
        self.data["verification-results"]["errors"] = 1
        for code in (0, True):
            with self.assertRaises(ValueError):
                self.validate(code=code)

    def test_untyped_publication_flag(self):
        self.data["verus-infer"]["interface_written"] = 0
        with self.assertRaises(ValueError):
            self.validate()


if __name__ == "__main__":
    unittest.main()
