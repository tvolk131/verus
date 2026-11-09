"""Synthetic gate mutations; these tests do not establish application theorems."""

import copy
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_redb_slot_selection_opaque as opaque
from infer_eval_support import ArtifactChanged, ArtifactGuard


def positive(source, expectation="provider"):
    expected = opaque.fixtures()["expected"][expectation]
    crate = "provider" if expectation == "provider" else "consumer"
    rows = copy.deepcopy(expected["native_functions"])
    for row in rows:
        row["source"] = str(source) + ":1:1: 1:2"
    return {"verification-results": {"success": True, "encountered-error": False,
        "encountered-vir-error": False, "is-verifying-entire-crate": True,
        "verified": expected["verified"], "errors": 0}, "verus-infer": {
        "schema_version": 1, "crate_success": True, "interface_written": True, "status": "verified",
        "verification_scope": {"native_collection_completed": True, "complete_selected_scope_verified": True,
            "checked_exec_bodies": expected["checked_exec_bodies"],
            "checked_proof_bodies": expected["checked_proof_bodies"], "native_functions": rows},
        "functions": [{"function": crate + expected["inferred_suffix"], "mode": "exact",
            "contract": copy.deepcopy(expected["contract"]), "required_checks_passed": True,
            "exported": True, "available_to_same_crate_callers": True,
            "proofs": {name: {"required": True, "status": "proved"} for name in opaque.PROOFS},
            "metrics": {"solver_micros": 1, "cache": "miss", "cache_policy": "candidate_cache"}}]}}


class OpaqueGates(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.directory = Path(self.temporary.name)
        self.source = self.directory / "fixture.rs"
        self.artifact = self.directory / "fixture.vir"
        self.library = self.directory / "libfixture.rlib"
        self.source.write_text(opaque.render_from_fixtures())
        self.artifact.write_bytes(b"synthetic, not verified VIR")
        self.library.write_bytes(b"synthetic, not compiled Rust")

    def tearDown(self):
        self.temporary.cleanup()

    def accept(self, data, expectation="provider", code=0, stderr=""):
        crate = "provider" if expectation == "provider" else "consumer"
        return opaque.validate_positive(data, code, stderr, self.artifact, self.library,
                                        self.source, crate, expectation)

    def reject(self, mutation):
        data = positive(self.source)
        mutation(data)
        with self.assertRaises((ValueError, KeyError, TypeError)):
            self.accept(data)

    def test_exact_provider_consumer_and_boundary_counts(self):
        for name in ("provider", "consumer", "boundary_identity", "boundary_implicit_drop"):
            self.accept(positive(self.source, name), name)
        rows = opaque.fixtures()["expected"]["provider"]["native_functions"]
        self.assertEqual(len(rows), 43)
        self.assertEqual(sum(row["body_status"] == "checked_exec_body" for row in rows), 15)
        self.assertEqual(sum(row["body_status"] == "checked_proof_body" for row in rows), 1)

    def test_typed_counts_and_completion(self):
        for group, keys in (("verification-results", ("verified", "errors")),
                ("verification_scope", ("checked_exec_bodies", "checked_proof_bodies"))):
            for key in keys:
                for value in (True, False, -1, 999, "0", None, 0.0):
                    def mutate(data):
                        target = data[group] if group == "verification-results" else data["verus-infer"][group]
                        target[key] = value
                    with self.subTest(group=group, key=key, value=value):
                        self.reject(mutate)
        for key in ("native_collection_completed", "complete_selected_scope_verified"):
            for value in (False, 1, "true", None):
                self.reject(lambda data: data["verus-infer"]["verification_scope"].update({key: value}))

    def test_typed_success_and_inference_proofs(self):
        for group, keys in (("verification-results", ("success", "is-verifying-entire-crate")),
                ("verus-infer", ("crate_success", "interface_written"))):
            for key in keys:
                for value in (False, 1, "true", None):
                    self.reject(lambda data: data[group].update({key: value}))
        for key in ("encountered-error", "encountered-vir-error"):
            self.reject(lambda data: data["verification-results"].update({key: True}))
        for key in ("required_checks_passed", "exported", "available_to_same_crate_callers"):
            for value in (False, 1, "true"):
                self.reject(lambda data: data["verus-infer"]["functions"][0].update({key: value}))
        for name in opaque.PROOFS:
            for value in (False, 1, "true"):
                self.reject(lambda data: data["verus-infer"]["functions"][0]["proofs"][name].update(required=value))
            self.reject(lambda data: data["verus-infer"]["functions"][0]["proofs"][name].update(status="skipped"))

    def test_exact_whole_scope_and_strict_flags(self):
        def rows(data):
            return data["verus-infer"]["verification_scope"]["native_functions"]
        self.reject(lambda data: rows(data).pop())
        self.reject(lambda data: rows(data).append(copy.deepcopy(rows(data)[0])))
        for key in ("automatically_derived", "selected", "inference_candidate", "ordinary_contract"):
            for value in (0, 1, "false", None):
                self.reject(lambda data: rows(data)[0].update({key: value}))
        for suffix in ("arrow_Io_0", "arrow_LockPoisoned_0", "arrow_ValueTooLarge_0"):
            self.reject(lambda data: next(row for row in rows(data) if row["function"].endswith(suffix)).update(
                body_status="checked_exec_body"))
        self.reject(lambda data: next(row for row in rows(data) if row["function"].startswith("<StorageError")).update(
            selected=True, body_status="checked_exec_body"))
        self.reject(lambda data: rows(data)[0].update(function="unrelated", rust_function="unrelated"))

    def test_all_scope_source_locations_and_solver_work(self):
        self.reject(lambda data: data["verus-infer"]["verification_scope"]["native_functions"][0].update(source="other.rs:1"))
        for value in (0, -1, True, "1", None, 1.0):
            self.reject(lambda data: data["verus-infer"]["functions"][0]["metrics"].update(solver_micros=value))
        for key, value in (("cache", "hit"), ("cache_policy", "proof_cache")):
            self.reject(lambda data: data["verus-infer"]["functions"][0]["metrics"].update({key: value}))
        self.reject(lambda data: data["verus-infer"].update(schema_version=True))

    def test_inferred_identity_contract_and_output_presence(self):
        self.reject(lambda data: data["verus-infer"]["functions"][0].update(function="provider::selection"))
        self.reject(lambda data: data["verus-infer"]["functions"][0].update(contract={"requires": "false"}))
        self.reject(lambda data: data["verus-infer"].update(functions=[]))
        self.library.unlink()
        with self.assertRaises(ValueError):
            self.accept(positive(self.source))
        self.library.write_bytes(b"fixture")
        self.artifact.unlink()
        with self.assertRaises(ValueError):
            self.accept(positive(self.source))

    def test_runtime_failures_and_process_status(self):
        for code in (1, True):
            with self.assertRaises(ValueError):
                self.accept(positive(self.source), code=code)
        for marker in ("panicked at", "resource limit", "timed out", "out of memory"):
            with self.assertRaises(ValueError):
                self.accept(positive(self.source), stderr=marker)

    def certificate(self):
        return opaque.original.read_json(opaque.original.LIVE_EVIDENCE / "redb-provider-certificate.json")

    def test_current_fixture_identity_rejects_archived_and_untyped_versions(self):
        baseline = opaque.fixtures()
        current = list(opaque.ENGINE_IDENTITY)
        untyped_cache = [float(current[0]), *current[1:]]
        untyped_schema = [*current[:2], float(current[2])]
        for identity in ([78, 72, 38], untyped_cache, untyped_schema, None):
            changed = copy.deepcopy(baseline)
            changed["identity"] = identity
            with self.subTest(identity=identity), patch.object(opaque.original, "read_json", return_value=changed):
                with self.assertRaisesRegex(ValueError, "engine identity changed"):
                    opaque.fixtures()

    def test_current_certificate_does_not_accept_archived_metadata(self):
        archived = opaque.original.read_json(opaque.original.EVIDENCE / "provider-certificate.json")
        with self.assertRaisesRegex(ValueError, "certificate identity changed"):
            opaque.validate_certificate(self.encoded(archived))

    def encoded(self, certificate):
        ordered = {"schema": certificate["schema"], **certificate}
        return b"synthetic-not-VIR" + json.dumps(ordered, separators=(",", ":")).encode() + b"VRINFER1"

    def test_certificate_fixture_and_all_control_mutations(self):
        raw = self.encoded(self.certificate())
        certificate, dependency = opaque.validate_certificate(raw)
        self.assertIs(type(dependency[0]["crate_id"]), int)
        for fault in opaque.FAULTS:
            corrupted, diagnostic = opaque.corrupt_certificate(raw, fault)
            self.assertTrue(diagnostic)
            with self.assertRaises((ValueError, KeyError, json.JSONDecodeError)):
                opaque.validate_certificate(corrupted)

    def test_certificate_typed_flags_and_model_inventory(self):
        mutations = [("schema", True), ("schema", 37), ("policy", "old"), ("producer", "other"),
            ("dependencies", [{"crate_id": 1, "fingerprint": "fake"}]), ("summaries", "invalid")]
        for key, value in mutations:
            certificate = self.certificate()
            certificate[key] = value
            with self.assertRaises(ValueError):
                opaque.validate_certificate(self.encoded(certificate))
        for name in opaque.PROOFS:
            for value in (False, 1, "true"):
                for nested in (False, True):
                    certificate = self.certificate()
                    target = certificate["proofs"][0][1] if nested else certificate
                    target[name] = value
                    with self.assertRaises(ValueError):
                        opaque.validate_certificate(self.encoded(certificate))
        for change in (lambda cert: cert["trusted_models"].pop(),
                lambda cert: cert["trusted_models"][0][0]["path"].update(krate="Application"),
                lambda cert: cert["proofs"][0][0]["path"].update(segments=["fake"])):
            certificate = self.certificate()
            change(certificate)
            with self.assertRaises(ValueError):
                opaque.validate_certificate(self.encoded(certificate))

    def test_duplicate_json_and_trailer(self):
        for raw in (b'{"schema":38,"schema":38}VRINFER1', b'{"schema":38}'):
            with self.assertRaises(ValueError):
                opaque.validate_certificate(raw)

    def test_consumer_certificate_dependency_fingerprint(self):
        provider = self.certificate()
        _certificate, dependency = opaque.validate_certificate(self.encoded(provider))
        consumer = copy.deepcopy(provider)
        consumer["proofs"][0][0]["path"].update(krate={"Id": ["consumer", 42]}, segments=["ordering"])
        consumer["dependencies"] = copy.deepcopy(dependency)
        opaque.validate_certificate(self.encoded(consumer), "consumer", dependency)
        consumer["dependencies"][0]["fingerprint"] = "0" * 64
        with self.assertRaises(ValueError):
            opaque.validate_certificate(self.encoded(consumer), "consumer", dependency)

    def test_seeded_semantic_and_representation_failures(self):
        for category, count, diagnostic in (("semantic", 22, "postcondition not satisfied"),
                                            ("safety", 23, "precondition not satisfied")):
            data = positive(self.source)
            data["verification-results"].update(success=False, verified=count, errors=1)
            data["verus-infer"].update(crate_success=False, interface_written=False)
            self.artifact.write_bytes(b"seed")
            with self.assertRaises(ValueError):
                opaque.validate_negative(data, 1, diagnostic, self.artifact, category, None)
            self.artifact.unlink()
            opaque.validate_negative(data, 1, diagnostic, self.artifact, category, None)
            for key, value in (("verified", True), ("verified", count - 1), ("errors", True), ("errors", 0)):
                changed = copy.deepcopy(data)
                changed["verification-results"][key] = value
                with self.assertRaises(ValueError):
                    opaque.validate_negative(changed, 1, diagnostic, self.artifact, category, None)

    def test_lifetime_failure_is_not_a_proof_or_drop_claim(self):
        data = positive(self.source)
        data["verification-results"].update(success=False, verified=0, errors=0)
        data["verification-results"]["encountered-error"] = True
        data["verus-infer"].update(crate_success=False, interface_written=False)
        diagnostic = "lifetime may not live long enough"
        with self.assertRaises(ValueError):
            opaque.validate_negative(data, 1, diagnostic, self.artifact, "lifetime", diagnostic)
        self.artifact.unlink()
        opaque.validate_negative(data, 1, diagnostic, self.artifact, "lifetime", diagnostic)
        for key, value in (("verified", 1), ("verified", False), ("errors", False), ("encountered-vir-error", True)):
            changed = copy.deepcopy(data)
            changed["verification-results"][key] = value
            with self.assertRaises(ValueError):
                opaque.validate_negative(changed, 1, diagnostic, self.artifact, "lifetime", diagnostic)

    def binding_row(self):
        stdout, stderr = self.directory / "stdout", self.directory / "stderr"
        data = positive(self.source)
        stdout.write_text(json.dumps(data))
        stderr.write_text("")
        dependency = self.directory / "provider.vir"
        dependency_library = self.directory / "libprovider.rlib"
        dependency.write_bytes(b"synthetic dependency")
        dependency_library.write_bytes(b"synthetic dependency library")
        row = {"label": "provider_debug_cold", "report": data, "no_cheating": True, "default_erasure": True,
            "command": ["verus", str(self.source), "--sysroot", "sysroot", "--crate-name", "provider",
                "--no-cheating", "--compile", "--output-json", "--export", str(self.artifact), "-o", str(self.library)],
            "imports": {str(dependency): opaque.digest(dependency),
                        str(dependency_library): opaque.digest(dependency_library)}}
        row["command"] += ["--import", "provider=" + str(dependency),
                           "--extern", "provider=" + str(dependency_library)]
        for key, path in (("source", self.source), ("stdout", stdout), ("stderr", stderr),
                          ("artifact", self.artifact), ("library", self.library)):
            row[key], row[key + "_sha256"] = str(path), opaque.digest(path)
        return row

    def test_artifact_and_command_bindings(self):
        row = self.binding_row()
        opaque.validate_bindings(row)
        for key in ("source_sha256", "stdout_sha256", "stderr_sha256", "artifact_sha256", "library_sha256"):
            changed = copy.deepcopy(row)
            changed[key] = "0" * 64
            with self.assertRaises(ValueError):
                opaque.validate_bindings(changed)
        for flag in ("--no-verify", "--verify-function=one", "--no-lifetime"):
            changed = copy.deepcopy(row)
            changed["command"].append(flag)
            with self.assertRaises(ValueError):
                opaque.validate_bindings(changed)
        dependency = self.directory / "provider.vir"
        dependency.write_bytes(b"changed import")
        with self.assertRaises(ValueError):
            opaque.validate_bindings(row)
        dependency.write_bytes(b"synthetic dependency")
        self.source.write_text(self.source.read_text().replace("ValueTooLarge(usize)", "ValueTooLarge(u8)"))
        row["source_sha256"] = opaque.digest(self.source)
        with self.assertRaises(ValueError):
            opaque.validate_bindings(row)

    def test_import_flags_are_bound_to_artifact_kinds(self):
        row = self.binding_row()
        opaque.validate_bindings(row)
        imported = row["command"].index("--import") + 1
        external = row["command"].index("--extern") + 1
        changed = copy.deepcopy(row)
        changed["command"][imported], changed["command"][external] = (
            changed["command"][external], changed["command"][imported])
        with self.assertRaisesRegex(ValueError, "command import binding changed: --import"):
            opaque.validate_bindings(changed)
        changed = copy.deepcopy(row)
        changed["command"][external] = changed["command"][imported]
        with self.assertRaisesRegex(ValueError, "command import binding changed: --extern"):
            opaque.validate_bindings(changed)

    def test_import_inventory_requires_one_interface_and_library(self):
        row = self.binding_row()
        for suffix in (".vir", ".rlib"):
            changed = copy.deepcopy(row)
            changed["imports"] = {path: expected for path, expected in changed["imports"].items()
                                  if Path(path).suffix == suffix}
            with self.assertRaisesRegex(ValueError, "certified import requires exactly one VIR and one rlib"):
                opaque.validate_bindings(changed)
        for suffix in (".vir", ".txt"):
            extra = self.directory / ("extra" + suffix)
            extra.write_bytes(b"unrelated dependency")
            changed = copy.deepcopy(row)
            changed["imports"][str(extra)] = opaque.digest(extra)
            with self.assertRaisesRegex(ValueError, "certified import requires exactly one VIR and one rlib"):
                opaque.validate_bindings(changed)

    def test_import_arguments_require_names_and_separators(self):
        row = self.binding_row()
        for flag in ("--import", "--extern"):
            index = row["command"].index(flag) + 1
            path = row["command"][index].partition("=")[2]
            for argument in (path, "=" + path):
                changed = copy.deepcopy(row)
                changed["command"][index] = argument
                with self.assertRaisesRegex(ValueError, "command import binding changed: " + flag):
                    opaque.validate_bindings(changed)

    def test_unrecorded_import_flags_reject(self):
        row = self.binding_row()
        row["imports"] = {}
        with self.assertRaisesRegex(ValueError, "unbound import command flags"):
            opaque.validate_bindings(row)

    def test_joined_import_duplicates_reject(self):
        row = self.binding_row()
        for flag in ("--import", "--extern"):
            argument = row["command"][row["command"].index(flag) + 1]
            changed = copy.deepcopy(row)
            changed["command"].append(flag + "=" + argument)
            with self.assertRaisesRegex(ValueError, "noncanonical import command flags"):
                opaque.validate_bindings(changed)

    def test_unrecorded_joined_import_flags_reject(self):
        row = self.binding_row()
        arguments = {flag: row["command"][row["command"].index(flag) + 1]
                     for flag in ("--import", "--extern")}
        row["command"] = row["command"][:-4]
        row["imports"] = {}
        opaque.validate_bindings(row)
        for flag, argument in arguments.items():
            changed = copy.deepcopy(row)
            changed["command"].append(flag + "=" + argument)
            with self.assertRaisesRegex(ValueError, "noncanonical import command flags"):
                opaque.validate_bindings(changed)

    def test_guard_rejects_same_size_replacement(self):
        self.source.write_bytes(b"first")
        guard = ArtifactGuard([self.source])
        self.source.write_bytes(b"other")
        with self.assertRaises(ArtifactChanged):
            guard.check()

    def test_full_enum_fields_bodies_and_no_behavior_assumptions(self):
        source = opaque.render_from_fixtures()
        pieces = {name: (opaque.original.EVIDENCE / "native-items" / (name + ".rs")).read_text().rstrip("\n")
                  for name in opaque.original.read_json(opaque.original.EVIDENCE / "source-ledger.json")["items"]}
        self.assertIn("#[derive(Debug)]\n#[non_exhaustive]\n" + pieces["StorageError"], source)
        self.assertEqual(source.count("external_type_specification"), 2)
        self.assertEqual(source.count("external_body"), 2)
        self.assertNotIn("assume_specification", source)
        self.assertNotIn("assume(", source)
        for name in opaque.METHODS:
            self.assertEqual(opaque.original.body(opaque.original.item(source, "    pub fn " + name + "(")),
                             opaque.original.body(pieces[name]))
        self.assertIn("pub struct TransactionId(pub u64)", source)
        self.assertIn("self.primary_slot < 2", source)
        self.assertIn("pub corrupt_bytes: Option<[u8; TRANSACTION_SIZE]>", source)

    def test_exact_matrix_and_source_controls(self):
        labels = opaque.required_labels()
        self.assertEqual(len(labels), 47)
        for label in labels:
            self.assertTrue(opaque.expected_source(label))
        self.assertNotIn("external_type_specification", opaque.expected_source("original_frontend_blocker"))
        for bad in ({"cases": [], "commands": []}, {"cases": [{"label": label} for label in labels] * 2}):
            with self.assertRaises(ValueError):
                opaque.validate_matrix(bad)
        with self.assertRaises(ValueError):
            opaque.expected_source("unexpected")

    def test_fidelity_rejects_reduced_enum_narrowed_domain_and_changed_frame(self):
        ledger = opaque.original.read_json(opaque.original.EVIDENCE / "source-ledger.json")
        pieces = {name: (opaque.original.EVIDENCE / "native-items" / (name + ".rs")).read_text().rstrip("\n")
                  for name in ledger["items"]}
        files = {}
        for name, entry in ledger["items"].items():
            lines = files.setdefault(entry["source"], [""] * 800)
            selected = pieces[name].splitlines()
            start = entry["line"] - 1
            lines[start:start + len(selected)] = selected
            if name == "StorageError":
                lines[start - 2:start] = ["#[derive(Debug)]", "#[non_exhaustive]"]
        native = self.directory / "synthetic-native"
        for relative, lines in files.items():
            path = native / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("\n".join(lines))
        source = opaque.render_from_fixtures()
        fidelity = opaque.source_fidelity(native, pieces, source)
        self.assertEqual(len(fidelity["bodies"]), 4)
        self.assertEqual(len(fidelity["declarations"]), 6)
        for old, new in (("Io(io::Error),", ""), ("TransactionId(pub u64)", "TransactionId(pub u32)"),
                ("pub full_regions: u32,", "pub full_regions: u8,"),
                ("self.primary_slot ^= 1;", "self.primary_slot = 0;"), ("#[non_exhaustive]", "")):
            self.assertIn(old, source)
            with self.assertRaises(ValueError):
                opaque.source_fidelity(native, pieces, source.replace(old, new, 1))

    def test_private_environment_and_no_old_snapshot_dependency(self):
        out = self.directory / "environment"
        out.mkdir()
        inherited = {"GIT_DIR": "/wrong", "GIT_WORK_TREE": "/wrong", "GIT_CONFIG_COUNT": "1",
            "VERUS_EXTRA_ARGS": "--no-verify", "VERUS_USE_RUSTUP": "1", "RUSTFLAGS": "unsafe",
            "CARGO_HOME": "/shared", "RUSTUP_HOME": "/shared"}
        with patch.dict(os.environ, inherited):
            env = opaque.environment_for(out, Path("/solver"), Path("/sysroot"))
        self.assertEqual({key for key in env if key.startswith("GIT_")}, {"GIT_OPTIONAL_LOCKS"})
        for key in ("HOME", "CARGO_HOME", "RUSTUP_HOME", "TMPDIR", "XDG_CACHE_HOME"):
            self.assertTrue(Path(env[key]).is_relative_to(out))
        self.assertEqual(env["VERUS_USE_RUSTUP"], "0")
        self.assertNotIn("VERUS_EXTRA_ARGS", env)
        self.assertNotIn("RUSTFLAGS", env)
        with patch.object(opaque.original, "audit_baseline", side_effect=AssertionError("forbidden")):
            self.assertTrue(opaque.render_from_fixtures())
        self.assertNotIn("/private/tmp/", Path(opaque.__file__).read_text())

    def test_cache_string_identity_and_exact_count(self):
        entries = [self.directory / "first.json", self.directory / "second.json"]
        for path in entries:
            path.write_text(json.dumps({"version": opaque.CACHE_VERSION}))
        opaque.validate_cache(entries)
        for value in (77, True, "verus-infer-v76", None):
            entries[0].write_text(json.dumps({"version": value}))
            with self.assertRaises(ValueError):
                opaque.validate_cache(entries)
        with self.assertRaises(ValueError):
            opaque.validate_cache([])


if __name__ == "__main__":
    unittest.main()
