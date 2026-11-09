import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from infer_transfer_identity import ENGINE_IDENTITY, INTERFACE_POLICY, INTERFACE_SCHEMA
import check_redb_bitmap_state as bitmap


def pieces():
    return {name: (bitmap.EVIDENCE / (name + ".source.rs")).read_text() for name in bitmap.ITEM_HASHES}


def positive_report():
    executions = {"U64GroupedBitmap::" + name for name in
                  ("get", "set", "clear", "data_index_of", "select_mask")}
    proofs = {"word_facts::set_bit_frame", "word_facts::clear_bit_frame"}
    native = [dict(function="provider::" + name, body_status=status, selected=True,
                   automatically_derived=False, inference_candidate=name.endswith("::select_mask"),
                   ordinary_contract=not name.endswith("::select_mask"))
              for names, status in ((executions, "checked_exec_body"), (proofs, "checked_proof_body"))
              for name in sorted(names)]
    return {"verification-results": {"success": True, "verified": 23, "errors": 0,
                                     "is-verifying-entire-crate": True},
            "verus-infer": {"crate_success": True, "interface_written": True,
                            "verification_scope": dict(native_collection_completed=True,
                                complete_selected_scope_verified=True, checked_exec_bodies=5,
                                checked_proof_bodies=2, native_functions=native),
                            "functions": [{"function": "provider::U64GroupedBitmap::select_mask",
                                           "mode": "exact", "required_checks_passed": True,
                                           "exported": True, "contract": dict(requires="((bit as u64) <= 63)",
                                               ensures="result == ((1 as u64) << (bit as u64))"),
                                           "proofs": {name: {"status": "proved", "required": True}
                                                      for name in ("soundness", "maximality", "exactness")},
                                           "metrics": {"solver_micros": 1}}]}}


def certificate_blob(**changes):
    certificate = dict(schema=INTERFACE_SCHEMA, policy=INTERFACE_POLICY, soundness=True,
                       maximality=True, exactness=True, trusted_models=[],
                       proofs=[[dict(path=dict(krate=dict(Id=["provider", 23]))),
                                dict(soundness=True, maximality=True, exactness=True)]])
    certificate.update(changes)
    return b"fixture" + json.dumps(certificate, separators=(",", ":")).encode() + b"VRINFER1"


def complete_cases():
    result = []
    for name, classification in bitmap.expected_cases().items():
        success = classification == "proved"
        result.append(dict(label=name, classification=classification, expected=classification,
                           artifact_exists=success, stale_export_seeded=not success,
                           export_sha256="a" * 64 if success else None,
                           seeded_export_sha256=None if success else "b" * 64))
    return result


class BitmapStateTests(unittest.TestCase):
    def test_five_original_method_bodies_survive_contract_insertion_exactly(self):
        source = bitmap.provider_source(pieces())
        contracts = json.loads((bitmap.EVIDENCE / "contracts.json").read_text())
        for name in bitmap.SIGNATURES:
            original = pieces()[name]
            if name == "select_mask":
                self.assertIn(original.replace("    fn ", "    pub fn ", 1), source)
            else:
                self.assertIn(contracts[name] + bitmap.body(original), source)
        self.assertEqual(source.count("#[verifier::infer]"), 1)
        self.assertIn('assert!(bit < self.len, "{bit} must be less than {}", self.len);', source)

    def test_all_six_item_hashes_bind_exact_checked_in_bytes(self):
        self.assertEqual(len(pieces()), 6)
        for name, original in pieces().items():
            self.assertEqual(hashlib.sha256(original.encode()).hexdigest(), bitmap.ITEM_HASHES[name])

    def test_extraction_reports_each_executable_item_and_body_hash(self):
        originals = pieces()
        text = originals["U64GroupedBitmap"] + "impl U64GroupedBitmap {\n"
        text += "\n".join(originals[name] for name in bitmap.SIGNATURES) + "}\n"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / bitmap.BITMAP
            source.parent.mkdir(parents=True)
            source.write_text(text)
            output = root / "output"
            output.mkdir()
            with patch.object(bitmap, "SOURCE_HASH", bitmap.digest(source)):
                extracted, provenance = bitmap.extract(root, output)
            self.assertEqual(extracted, originals)
            self.assertEqual(set(provenance["items"]), set(originals))
            for name, original in originals.items():
                self.assertEqual(provenance["items"][name]["body_sha256"],
                                 hashlib.sha256(bitmap.body(original).encode()).hexdigest())

    def test_source_and_item_tampering_fail_before_evidence_publication(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / bitmap.BITMAP
            source.parent.mkdir(parents=True)
            source.write_text("changed")
            with self.assertRaisesRegex(ValueError, "pinned bitmap.rs changed"):
                bitmap.extract(root, root)
            originals = pieces()
            text = originals["U64GroupedBitmap"] + "impl U64GroupedBitmap {\n"
            text += "\n".join(originals[name] for name in bitmap.SIGNATURES) + "}\n"
            source.write_text(text.replace("group |=", "group ^="))
            with patch.object(bitmap, "SOURCE_HASH", bitmap.digest(source)):
                with self.assertRaisesRegex(ValueError, "pinned executable extraction changed: set"):
                    bitmap.extract(root, root)

    def test_representation_allows_spare_words_not_only_exact_capacity(self):
        abstraction = (bitmap.EVIDENCE / "abstraction.verus.rs").read_text()
        self.assertIn("self.len as int <= 64 * self.data@.len()", abstraction)
        self.assertIn("index / 64", abstraction)
        self.assertIn("index % 64", abstraction)
        for length in (0, 1, 63, 64, 65, 2**32 - 1):
            required = (length + 63) // 64
            for words in (0, max(0, required - 1), required, required + 1):
                self.assertEqual(length <= 64 * words, words >= required)

    def test_high_level_contracts_cover_every_live_bit_and_stored_word_flag(self):
        contracts = json.loads((bitmap.EVIDENCE / "contracts.json").read_text())
        for name in ("set", "clear"):
            self.assertIn("old(self).valid()", contracts[name])
            self.assertIn("bit < old(self).len", contracts[name])
            self.assertIn("0 <= query < final(self).len", contracts[name])
            self.assertIn("final(self).data@ == old(self).data@.update", contracts[name])
            self.assertNotIn("bit < 64", contracts[name])
            self.assertNotIn("requires false", contracts[name])
        self.assertIn("result == (final(self).data@[bit as int / 64] == u64::MAX)", contracts["set"])

    def test_author_proof_assistance_is_explicit_checked_and_not_trusted(self):
        facts = (bitmap.EVIDENCE / "word_facts.verus.rs").read_text()
        self.assertEqual(facts.count("pub broadcast proof fn"), 2)
        self.assertEqual(facts.count("by(bit_vector)"), 2)
        for forbidden in ("external_body", "assume(", "admit(", "axiom"):
            self.assertNotIn(forbidden, facts)

    def test_native_runner_contains_exact_methods_and_independent_oracle(self):
        source = bitmap.native_source(pieces())
        for original in pieces().values():
            self.assertIn(original, source)
        self.assertNotIn("verifier::infer", source)
        self.assertIn("/ (1u64 << (observed % 64))) % 2", source)
        self.assertIn("cfg!(debug_assertions)", source)
        self.assertIn("1_228_980", source)

    def test_environment_removes_git_context_wrappers_and_shares_no_cargo_state(self):
        with patch.dict(os.environ, {"GIT_DIR": "bad", "GIT_WORK_TREE": "bad", "GIT_CONFIG_COUNT": "1",
                                     "RUSTC_WRAPPER": "bad", "RUSTFLAGS": "bad"}):
            environment = bitmap.environment_for(Path("/private/tmp/private-run"), Path("/private/tmp/z3"))
        self.assertEqual({key for key in environment if key.startswith("GIT_")}, {"GIT_OPTIONAL_LOCKS"})
        self.assertEqual(environment["GIT_OPTIONAL_LOCKS"], "0")
        self.assertEqual(environment["CARGO_BUILD_JOBS"], "1")
        self.assertEqual(environment["CARGO_HOME"], "/private/tmp/private-run/cargo-home")
        self.assertEqual(environment["CARGO_TARGET_DIR"], "/private/tmp/private-run/cargo-target")
        self.assertNotIn("RUSTC_WRAPPER", environment)
        self.assertNotIn("RUSTFLAGS", environment)

    def test_fresh_outputs_and_dangling_symlinks_are_not_reused(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            artifact, library = root / "export.vir", root / "lib.rlib"
            bitmap.fresh_artifacts(artifact, library)
            library.write_bytes(b"old")
            with self.assertRaisesRegex(ValueError, "stale"):
                bitmap.fresh_artifacts(artifact, library)
            library.unlink()
            artifact.symlink_to(root / "missing")
            with self.assertRaisesRegex(ValueError, "stale"):
                bitmap.fresh_artifacts(artifact, library)

    def test_negative_seeding_is_distinct_and_byte_exact(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            artifact, library = root / "export.vir", root / "lib.rlib"
            bitmap.fresh_artifacts(artifact, library, b"old checked interface")
            self.assertEqual(artifact.read_bytes(), b"old checked interface")
            self.assertFalse(library.exists())
            with self.assertRaisesRegex(ValueError, "paths must differ"):
                bitmap.fresh_artifacts(library, library)

    def test_diagnostic_classifications_keep_trust_boundaries_separate(self):
        report = {"verification-results": {}, "verus-infer": {}}
        for diagnostic, classification in [
                ("postcondition not satisfied", "semantic_goal_failure"),
                ("precondition not satisfied", "safety_failure"),
                ("error[E0499]: cannot borrow again more than once", "aliasing_frontend_rejection"),
                ("bitvector assertion not satisfied", "authored_lemma_failure"),
                ("Resource limit (rlimit) exceeded", "resource_or_runtime_failure")]:
            self.assertEqual(bitmap.classify(report, diagnostic, 1), classification)
        report["verus-infer"]["status"] = "invalid_import"
        self.assertEqual(bitmap.classify(report, "policy", 1), "import_rejection")
        report["verus-infer"]["status"] = "verification_failed"
        report["verification-results"]["encountered-vir-error"] = True
        for diagnostic in bitmap.METADATA_DIAGNOSTICS.values():
            self.assertEqual(bitmap.classify(report, diagnostic, 1), "import_rejection")

    def test_positive_gate_requires_all_three_checked_obligations(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "export.vir"
            artifact.write_bytes(b"new")
            bitmap.check_case(positive_report(), "", 0, "proved", artifact, 1)
            for name in ("soundness", "maximality", "exactness"):
                for field, value in (("status", "not_established"), ("required", False)):
                    data = positive_report()
                    data["verus-infer"]["functions"][0]["proofs"][name][field] = value
                    with self.assertRaises(ValueError):
                        bitmap.check_case(data, "", 0, "proved", artifact, 1)

    def test_positive_gate_requires_scope_solver_work_entire_crate_and_publication(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "export.vir"
            artifact.write_bytes(b"new")
            variants = []
            for key in ("required_checks_passed", "mode"):
                data = positive_report()
                data["verus-infer"]["functions"][0][key] = False
                variants.append(data)
            data = positive_report()
            data["verus-infer"]["functions"][0]["metrics"]["solver_micros"] = 0
            variants.append(data)
            for section, field in (("verification-results", "is-verifying-entire-crate"),
                                   ("verus-infer", "interface_written"), ("verus-infer", "crate_success")):
                data = positive_report()
                data[section][field] = False
                variants.append(data)
            for data in variants:
                with self.assertRaises(ValueError):
                    bitmap.check_case(data, "", 0, "proved", artifact, 1)
            with self.assertRaisesRegex(ValueError, "inference scope"):
                bitmap.check_case(positive_report(), "", 0, "proved", artifact, 0)
            artifact.unlink()
            with self.assertRaisesRegex(ValueError, "publication"):
                bitmap.check_case(positive_report(), "", 0, "proved", artifact, 1)

    def test_positive_scope_rejects_missing_partial_or_forged_native_proofs(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "export.vir"
            artifact.write_bytes(b"new")
            variants = []
            missing = positive_report()
            del missing["verus-infer"]["verification_scope"]
            variants.append(missing)
            for field, values in {
                    "native_collection_completed": (False, 1),
                    "complete_selected_scope_verified": (False, 1),
                    "checked_exec_bodies": (0, 4, 6, True),
                    "checked_proof_bodies": (0, 1, 3, True),
                    "native_functions": ([], None)}.items():
                for value in values:
                    changed = positive_report()
                    changed["verus-infer"]["verification_scope"][field] = value
                    variants.append(changed)
            for field, values in {"verified": (0, 22, 24, True), "errors": (1, False)}.items():
                for value in values:
                    changed = positive_report()
                    changed["verification-results"][field] = value
                    variants.append(changed)
            for index in range(7):
                for field, value in (("function", "unrelated::helper"), ("selected", False),
                        ("automatically_derived", True), ("body_status", "external_body"),
                        ("inference_candidate", "false"), ("ordinary_contract", 1)):
                    changed = positive_report()
                    changed["verus-infer"]["verification_scope"]["native_functions"][index][field] = value
                    variants.append(changed)
            for changed in variants:
                with self.subTest(changed=changed), self.assertRaises(ValueError):
                    bitmap.check_case(changed, "", 0, "proved", artifact, 1)

    def test_positive_identity_domain_and_booleans_are_strictly_bound(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "export.vir"
            artifact.write_bytes(b"new")
            for field, values in {"function": ("unrelated::helper", "client_debug::relay"),
                    "contract": (dict(requires="false"), dict(requires="true")),
                    "required_checks_passed": ("false", 1), "exported": (False, 1)}.items():
                for value in values:
                    changed = positive_report()
                    changed["verus-infer"]["functions"][0][field] = value
                    with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                        bitmap.check_case(changed, "", 0, "proved", artifact, 1)
            for section, field in (("verification-results", "success"),
                    ("verus-infer", "crate_success"), ("verus-infer", "interface_written")):
                changed = positive_report()
                changed[section][field] = 1
                with self.subTest(field=field), self.assertRaises(ValueError):
                    bitmap.check_case(changed, "", 0, "proved", artifact, 1)
            changed = positive_report()
            changed["verus-infer"]["functions"][0]["metrics"]["solver_micros"] = True
            with self.assertRaisesRegex(ValueError, "solver work"):
                bitmap.check_case(changed, "", 0, "proved", artifact, 1)

    def test_semantic_control_cannot_be_replaced_by_safety_or_frontend_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "export.vir"
            for diagnostic in ("precondition not satisfied", "error[E0499]: cannot borrow more than once", "panicked at bug"):
                with self.assertRaises(ValueError):
                    bitmap.check_case(positive_report(), diagnostic, 1, "semantic_goal_failure", artifact, 1)

    def test_provider_certificate_observes_identity_exactness_model_scope_and_fingerprint(self):
        blob = certificate_blob()
        certificate, binding = bitmap.provider_certificate(blob)
        self.assertEqual(certificate["schema"], 38)
        footer, decoded = bitmap.metadata(blob)
        self.assertEqual(decoded, certificate)
        self.assertEqual(binding, [dict(crate_id=23, fingerprint=hashlib.sha256(blob[footer:-8]).hexdigest())])
        for change in (dict(schema=37), dict(policy="stale"), dict(exactness=False), dict(trusted_models=[["new", "model"]]), dict(proofs=[])):
            with self.assertRaises(ValueError):
                bitmap.provider_certificate(certificate_blob(**change))

    def test_provider_rejects_missing_trailer_and_incomplete_leaf_proofs(self):
        with self.assertRaises(ValueError):
            bitmap.provider_certificate(certificate_blob()[:-8])
        footer, certificate = bitmap.metadata(certificate_blob())
        certificate["proofs"][0][1]["maximality"] = False
        with self.assertRaisesRegex(ValueError, "incomplete exported proof"):
            bitmap.provider_certificate(certificate_blob(**certificate))

    def test_import_binding_must_match_checked_warm_provider(self):
        data = {"verus-infer": {"imported_inference_dependencies": [dict(crate_id=23, fingerprint="old")]}}
        with self.assertRaisesRegex(ValueError, "fingerprint"):
            bitmap.check_client_import(data, [dict(crate_id=23, fingerprint="new")])

    def test_corrupt_import_controls_keep_valid_metadata_shapes_and_original_fixture(self):
        footer, original = bitmap.metadata(certificate_blob(summaries="correct"))
        saved = copy.deepcopy(original)
        fields = dict(stale_policy="policy", false_exactness="exactness", wrong_summary="summaries", wrong_model="trusted_models")
        for fault, field in fields.items():
            changed = bitmap.corrupt_certificate(original, fault)
            self.assertEqual(original, saved)
            self.assertNotEqual(changed[field], original[field])
            self.assertEqual({key: value for key, value in changed.items() if key != field},
                             {key: value for key, value in original.items() if key != field})
            if fault == "wrong_model":
                self.assertEqual(changed[field][0][0], original["proofs"][0][0])
                self.assertIsInstance(changed[field][0][0], dict)
        with self.assertRaisesRegex(ValueError, "unknown import fault"):
            bitmap.corrupt_certificate(original, "unrecognized")

    def test_completion_requires_all_54_unique_cases_and_44_removed_stale_exports(self):
        cases = complete_cases()
        bitmap.check_completion(cases)
        self.assertEqual(len(cases), 54)
        self.assertEqual(sum(case["stale_export_seeded"] for case in cases), 44)
        for index in range(len(cases)):
            with self.assertRaises(ValueError):
                bitmap.check_completion(cases[:index] + cases[index + 1:])
        with self.assertRaises(ValueError):
            bitmap.check_completion(cases[:-1] + [cases[0]])

    def test_completion_rejects_publication_seed_classification_and_hash_drift(self):
        for key, value in (("classification", "proved"), ("expected", "proved"),
                           ("artifact_exists", True), ("stale_export_seeded", False),
                           ("export_sha256", "a" * 64), ("seeded_export_sha256", None)):
            cases = copy.deepcopy(complete_cases())
            negative = next(case for case in cases if case["expected"] != "proved")
            negative[key] = value
            with self.assertRaises(ValueError):
                bitmap.check_completion(cases)

    def test_no_erasure_bypass_or_new_engine_identity_is_enabled(self):
        source = Path(bitmap.__file__).read_text()
        self.assertNotIn("--no-erasure-check", source)
        self.assertIn('"--no-cheating"', source)
        self.assertIn('"--num-threads", "1"', source)
        self.assertEqual(bitmap.IDENTITY, list(ENGINE_IDENTITY))

    def test_optimized_python_still_rejects_invalid_completion(self):
        command = [sys.executable, "-B", "-O", "-c", "import check_redb_bitmap_state as b; b.check_completion([])"]
        environment = dict(os.environ, PYTHONPATH=str(Path(bitmap.__file__).parent), PYTHONDONTWRITEBYTECODE="1")
        result = subprocess.run(command, env=environment, capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing or duplicate replay cases", result.stderr)


if __name__ == "__main__":
    unittest.main()
