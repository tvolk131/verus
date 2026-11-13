"""Synthetic integrated-gate tests, never substituted for verifier evidence."""

import ast
import copy
from dataclasses import replace
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_redb_stateful as checker


def synthetic_report(expected):
    bodies = [dict(function=name, **wanted) for name, wanted in expected["bodies"].items()]
    helpers = [dict(function=name, mode="exact", required_checks_passed=True,
                    available_to_same_crate_callers=True, exported=wanted["exported"],
                    contract=copy.deepcopy(wanted["contract"]),
                    proofs={key: dict(required=True, status="proved")
                            for key in ("soundness", "maximality", "exactness")},
                    metrics=dict(solver_micros=1, cache=wanted["cache"]))
               for name, wanted in expected["helpers"].items()]
    return {"verification-results": dict(success=True, errors=0, verified=expected["verified"],
                **{"encountered-error": False, "encountered-vir-error": False,
                   "is-verifying-entire-crate": True}),
            "verus-infer": dict(crate_success=True, interface_written=expected["interface_written"], generation_failure=None,
                failed_functions=[], functions=helpers, requested_functions=len(helpers),
                generated_functions=len(helpers), imported_inference_dependencies=[],
                trusted_models=copy.deepcopy(expected["trusted_models"]),
                verification_scope=dict(native_collection_completed=True, complete_selected_scope_verified=True,
                    native_functions=bodies,
                    checked_exec_bodies=sum(body["body_status"] == "checked_exec_body" for body in bodies),
                    checked_proof_bodies=sum(body["body_status"] == "checked_proof_body" for body in bodies)))}


class IntegratedAcceptanceTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.repo = self.root / "repo"
        engine = self.repo / "source/engine"
        engine.mkdir(parents=True)
        (engine / "Cargo.toml").write_text('[package]\nname = "synthetic"\nversion = "0.0.0"\n')
        (engine / "lib.rs").write_text("synthetic test source, not executable proof evidence\n")
        (self.repo / "source/Cargo.toml").write_text('[workspace]\nmembers = ["engine"]\n')
        (self.repo / "source/Cargo.lock").write_text("synthetic lock\n")
        (self.repo / "source/vstd").mkdir()
        (self.repo / "source/.cargo").mkdir()
        (self.repo / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.99.1"\n')
        native = self.root / "native" / checker.NATIVE_FILE
        native.parent.mkdir(parents=True)
        native.write_text("synthetic native pin; not a redb checkout\n")
        self.native_patch = patch.object(checker, "NATIVE_SHA256", checker.digest(native))
        self.native_patch.start()
        self.addCleanup(self.native_patch.stop)
        self.git_patch = patch.object(checker, "git_read", side_effect=self.clean_git)
        self.git_patch.start()
        self.addCleanup(self.git_patch.stop)
        runtime = self.root / "runtime"
        runtime.write_bytes(b"synthetic runtime binding, not an executable")
        self.context = dict(repo=str(self.repo), source_manifest=checker.source_manifest(self.repo),
                            native=checker.native_binding(self.root / "native"), guarded=[checker.artifact(runtime)],
                            execution=dict(rustup=str(self.root / "rustup"), runtime=str(self.root / "runtime_dir"),
                                           sysroot=str(self.root / checker.TOOLCHAIN),
                                           output=str(self.root / "replay"), ready=True),
                            applications={}, trusted_models={})
        selected = [Path(self.context["execution"]["rustup"]),
                    Path(self.context["execution"]["sysroot"]) / "bin/rustc"]
        selected += [Path(self.context["execution"]["runtime"]) / name for name in checker.RUNTIME_FILES]
        for path in selected:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"synthetic selected execution input, not a real executable")
            self.context["guarded"].append(checker.artifact(path))
        self.report_expected = json.loads((checker.FIXTURES / "expected.json").read_text())["transition"]
        self.document = synthetic_report(self.report_expected)
        self.context["applications"]["stateful_transition"] = dict(
            source_sha256=checker.hashlib.sha256(b"synthetic input").hexdigest(), enabled=True)
        self.context["trusted_models"] = {row["function"]: row.get("fingerprint", "b" * 64)
                                          for row in self.report_expected["trusted_models"]}
        self.paths = checker.application_paths(self.context, "stateful_transition")
        for name, path in self.paths.items():
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(("synthetic " + name).encode())
        self.paths["stderr"].write_bytes(b"")
        self.argv = checker.application_command(self.context, "stateful_transition")
        self.run = dict(argv=self.argv, returncode=0, timeout=False,
                        **{name: str(self.paths[name]) for name in ("stdout", "stderr", "library", "interface")})
        self.expected = dict(case="stateful_transition", argv=list(self.argv), report=self.report_expected)
        self.refresh()

    @staticmethod
    def clean_git(checkout, *arguments):
        return checker.NATIVE_REVISION if arguments == ("rev-parse", "HEAD") else ""

    def refresh(self):
        self.paths["stdout"].write_text(json.dumps(self.document))
        self.run["artifacts"] = {name: checker.artifact(path) for name, path in self.paths.items()}
        for name in ("input", "stdout", "stderr"):
            self.run[name + "_artifact"] = copy.deepcopy(self.run["artifacts"][name])
        self.expected["artifacts"] = copy.deepcopy(self.run["artifacts"])

    def accept(self, imports=None):
        return checker.accept_positive(self.run, self.expected, self.context, imports)

    def rejects_document(self, mutate):
        baseline = copy.deepcopy(self.document)
        mutate(self.document)
        self.refresh()
        with self.assertRaises(checker.ReceiptError):
            self.accept()
        self.document = baseline
        self.refresh()

    def test_integrated_positive(self):
        self.assertEqual(self.accept(), self.document)

    def test_malformed_and_wrong_totals(self):
        for field, value in (("verified", True), ("verified", 19), ("verified", "18"),
                             ("errors", False), ("errors", 1), ("success", 1)):
            with self.subTest(field=field, value=value):
                self.rejects_document(lambda document: document["verification-results"].update({field: value}))

    def test_positive_failure_reports(self):
        for field, value in (("crate_success", False), ("generation_failure", "failed"),
                             ("failed_functions", ["stateful_transition::native_step"]),
                             ("interface_written", False), ("requested_functions", True)):
            with self.subTest(field=field):
                self.rejects_document(lambda document: document["verus-infer"].update({field: value}))

    def test_complete_exec_and_proof_body_scope(self):
        for field, value in (("native_collection_completed", False), ("complete_selected_scope_verified", False),
                             ("checked_exec_bodies", True), ("checked_proof_bodies", 0),
                             ("native_functions", []), ("native_functions", "not rows")):
            with self.subTest(field=field):
                self.rejects_document(lambda document: document["verus-infer"]["verification_scope"].update({field: value}))

    def test_body_scope_substitution_and_duplicates(self):
        for field, value in (("body_status", "external_body"), ("selected", False),
                             ("ordinary_contract", True), ("inference_candidate", False),
                             ("automatically_derived", True), ("function", "substituted::resize")):
            with self.subTest(field=field):
                self.rejects_document(lambda document: document["verus-infer"]["verification_scope"]
                                      ["native_functions"][0].update({field: value}))
        self.rejects_document(lambda document: document["verus-infer"]["verification_scope"]
                              ["native_functions"].append(document["verus-infer"]["verification_scope"]["native_functions"][0]))

    def test_helper_proofs_and_trust_not_just_query_count(self):
        self.rejects_document(lambda document: document["verus-infer"]["functions"][0]["proofs"]
                              ["exactness"].update(status="failed"))
        self.rejects_document(lambda document: document["verus-infer"].update(trusted_models=[]))
        self.rejects_document(lambda document: document["verus-infer"]["functions"][0]["metrics"].update(cache="hit"))

    def test_missing_each_artifact(self):
        for name, path in self.paths.items():
            with self.subTest(artifact=name):
                content = path.read_bytes()
                path.unlink()
                with self.assertRaises(checker.ReceiptError):
                    self.accept()
                path.write_bytes(content)

    def test_modified_input_and_runtime(self):
        self.paths["input"].write_text("changed input")
        with self.assertRaises(checker.ReceiptError):
            self.accept()
        self.paths["input"].write_text("synthetic input")
        Path(self.context["guarded"][0]["path"]).write_text("changed runtime")
        with self.assertRaises(checker.ReceiptError):
            self.accept()

    def test_post_execution_artifact_rebinding_is_rejected(self):
        self.paths["input"].write_text("substituted after execution")
        late = checker.artifact(self.paths["input"])
        self.run["artifacts"]["input"] = late
        self.expected["artifacts"]["input"] = late
        with self.assertRaisesRegex(checker.ReceiptError, "execution artifact binding"):
            self.accept()

    def test_compiler_source_drift_and_new_source(self):
        source = self.repo / "source/engine/lib.rs"
        original = source.read_bytes()
        source.write_text("compiler drift")
        with self.assertRaises(checker.ReceiptError):
            self.accept()
        source.write_bytes(original)
        (source.parent / "new.rs").write_text("new compiler input")
        with self.assertRaises(checker.ReceiptError):
            self.accept()

    def test_wrong_native_source(self):
        (self.root / "native" / checker.NATIVE_FILE).write_text("wrong native source")
        with self.assertRaisesRegex(checker.ReceiptError, "native bitmap"):
            self.accept()

    def test_wrong_native_revision(self):
        with patch.object(checker, "git_read", return_value="0" * 40):
            with self.assertRaisesRegex(checker.ReceiptError, "native revision"):
                self.accept()

    def test_dirty_native_pin(self):
        def dirty(checkout, *arguments):
            return checker.NATIVE_REVISION if arguments == ("rev-parse", "HEAD") else " M bitmap.rs"
        with patch.object(checker, "git_read", side_effect=dirty):
            with self.assertRaisesRegex(checker.ReceiptError, "dirty native"):
                self.accept()

    def test_mandatory_arguments_and_exact_argv(self):
        for flag in ("--compile", "--no-cheating", "--output-json"):
            with self.subTest(flag=flag):
                altered = copy.deepcopy(self.run)
                altered["argv"].remove(flag)
                with self.assertRaises(checker.ReceiptError):
                    checker.accept_positive(altered, self.expected, self.context)

    def test_import_kind_binding_even_when_expected_argv_is_replaced(self):
        imports = {"provider": {name: checker.artifact(self.paths[name]) for name in ("library", "interface")}}
        correct = ["--import", "provider=" + str(self.paths["interface"]),
                   "--extern", "provider=" + str(self.paths["library"])]
        self.run["argv"] = self.argv + correct
        self.expected["argv"] = list(self.run["argv"])
        self.accept(imports)
        for bad in (["--extern", correct[1], "--import", correct[3]],
                    ["--import=" + correct[1], "--extern", correct[3]],
                    ["--import", correct[1], "--extern", correct[3], "--extern", correct[3]]):
            with self.subTest(arguments=bad):
                self.run["argv"] = self.argv + bad
                self.expected["argv"] = list(self.run["argv"])
                with self.assertRaises(checker.ReceiptError):
                    self.accept(imports)

    def test_imported_fingerprint_is_checked(self):
        self.rejects_document(lambda document: document["verus-infer"].update(
            imported_inference_dependencies=[dict(crate_id=1, fingerprint="0" * 64)]))

    def test_current_certificate_model_binding_then_integrated_acceptance(self):
        certificate = dict(schema=checker.INTERFACE_SCHEMA, policy=checker.INTERFACE_POLICY,
            producer="stateful-source:" + checker.TOOLCHAIN + ":source-synthetic",
            soundness=True, maximality=True, exactness=True, dependencies=[],
            proofs=[[dict(path=dict(krate=dict(Id=["stateful_transition", 1]), segments=["bits_in_range"])),
                     dict(soundness=True, maximality=True, exactness=True)]],
            trusted_models=[[dict(path=dict(krate="Core", segments=["panicking", "panic"])), "a" * 64]])
        bound = checker.bind_certificate(self.report_expected, certificate, "source-synthetic")
        self.expected["report"] = bound
        self.document = synthetic_report(bound)
        self.refresh()
        self.accept()
        self.rejects_document(lambda document: document["verus-infer"]["trusted_models"][0]
                              .update(fingerprint="b" * 64))
        for mutation in (lambda cert: cert.update(producer="handpicked-old-producer"),
                         lambda cert: cert.update(schema=False),
                         lambda cert: cert["proofs"][0][1].update(exactness=False),
                         lambda cert: cert["trusted_models"][0][0]["path"].update(segments=["unapproved"]),
                         lambda cert: cert.update(dependencies=[dict(crate_id=5, fingerprint="b" * 64)])):
            altered = copy.deepcopy(certificate)
            mutation(altered)
            with self.assertRaises(checker.ReceiptError):
                checker.bind_certificate(self.report_expected, altered, "source-synthetic")

    def test_default_exit_does_not_claim_complete_resize(self):
        receipt = dict(status="transition_access_established_resize_incomplete", full_resize_complete=False)
        for extra, wanted in (([], 2), (["--transition-only"], 0)):
            with patch.object(checker, "Replay"), patch.object(checker, "replay", return_value=receipt), \
                    patch("sys.stdout", new_callable=io.StringIO):
                actual = checker.main(["--native-checkout", str(self.root / "native"),
                                       "--output", str(self.root / "not-created"), *extra])
                self.assertEqual(actual, wanted)

    def test_full_source_drift_hits_integrated_gate(self):
        snapshot = {"source/engine/lib.rs": checker.digest(self.repo / "source/engine/lib.rs")}
        self.context["full_source_manifest"] = snapshot
        with patch.object(checker, "full_source_manifest", return_value=dict(snapshot)):
            self.accept()
        with patch.object(checker, "full_source_manifest", return_value={}):
            with self.assertRaisesRegex(checker.ReceiptError, "full source drift"):
                self.accept()

    def test_access_export_remains_ordinary_not_certified_inference(self):
        expected = json.loads((checker.FIXTURES / "access_expected.json").read_text())["access"]
        self.assertIs(expected["interface_written"], False)
        self.document = synthetic_report(expected)
        self.expected["report"] = expected
        self.refresh()
        self.accept()
        self.rejects_document(lambda document: document["verus-infer"].update(interface_written=True))

    def test_full_source_inventory_hashes_noncompiler_files(self):
        inventory = "rust-toolchain.toml\0source/engine/lib.rs\0"
        with patch.object(checker, "git_read", return_value=inventory):
            before = checker.full_source_manifest(self.repo)
            self.assertEqual(len(before), 2)
            (self.repo / "rust-toolchain.toml").write_text("changed noncompiler input")
            self.assertNotEqual(before, checker.full_source_manifest(self.repo))

    def blocker_fixture(self):
        return self.negative_fixture("required_words_blocker")

    def test_expected_frontend_blocker_has_no_theorem_credit(self):
        run, document = self.blocker_fixture()
        expected = checker.EXPECTED_FRONTEND_BLOCKER
        self.assertEqual(checker.accept_frontend_blocker(run, expected, self.context), document)
        for mutation in (lambda doc: doc["verification-results"].update(verified=1),
                         lambda doc: doc["verification-results"].update(**{"encountered-vir-error": False}),
                         lambda doc: doc["verus-infer"]["verification_scope"]["native_functions"][0]
                         .update(body_status="checked_exec_body")):
            altered = copy.deepcopy(document)
            mutation(altered)
            self.paths["stdout"].write_text(json.dumps(altered))
            run["stdout_artifact"] = checker.artifact(self.paths["stdout"])
            with self.assertRaises(checker.ReceiptError):
                checker.accept_frontend_blocker(run, expected, self.context)

    def test_blocker_wrong_diagnostic_and_published_vir_rejected(self):
        run, document = self.blocker_fixture()
        self.paths["stderr"].write_text("error: unrelated unsupported function")
        run["stderr_artifact"] = checker.artifact(self.paths["stderr"])
        with self.assertRaises(checker.ReceiptError):
            checker.accept_frontend_blocker(run, checker.EXPECTED_FRONTEND_BLOCKER, self.context)
        self.paths["stderr"].write_text(checker.EXPECTED_FRONTEND_BLOCKER["diagnostic"])
        run["stderr_artifact"] = checker.artifact(self.paths["stderr"])
        self.paths["interface"].write_text("unacceptable frontend export")
        with self.assertRaises(checker.ReceiptError):
            checker.accept_frontend_blocker(run, checker.EXPECTED_FRONTEND_BLOCKER, self.context)

    def test_access_native_body_bytes_cannot_change(self):
        reference = (checker.FIXTURES / "access_composition.rs").read_text()
        required = (checker.FIXTURES / "required_words_native.rs").read_text()
        native = reference.replace("impl U64GroupedBitmap {", "impl U64GroupedBitmap {" +
                                   checker.braced(required, "impl U64GroupedBitmap"))
        self.assertEqual(checker.connect_access(native, reference, required)["checked_access_bodies"], 7)
        changed = native.replace("group |= Self::select_mask", "group &= Self::select_mask")
        with self.assertRaisesRegex(checker.ReceiptError, "native access body bytes"):
            checker.connect_access(changed, reference, required)

    def negative_fixture(self, case="wrong_branch"):
        expected = checker.failure_expectation(case)
        self.context["applications"][case] = dict(
            source_sha256=checker.hashlib.sha256(b"synthetic input").hexdigest(),
            enabled=checker.application_sources()[case]["enabled"])
        self.paths = checker.application_paths(self.context, case)
        for name in ("input", "stdout", "stderr"):
            self.paths[name].parent.mkdir(parents=True, exist_ok=True)
        self.paths["input"].write_bytes(b"synthetic input")
        inference = json.loads(json.dumps(expected["inference"]).replace("@input", str(self.paths["input"])))
        inference["metrics"]["total_micros"] = 1
        for helper in inference["functions"]:
            helper["candidate_id"] = "a" * 64
            helper["metrics"]["solver_micros"] = 1
        for model in inference["trusted_models"]:
            model["fingerprint"] = self.context["trusted_models"][model["function"]]
        document = {"verification-results": copy.deepcopy(expected["verification"]), "verus-infer": inference}
        self.failure_bodies = {body["function"]: body for body in inference["verification_scope"]["native_functions"]}
        self.paths["stdout"].write_text(json.dumps(document))
        self.paths["stderr"].write_text("\n".join("error: " + diagnostic["message"] + "\n" +
            "\n".join(" --> " + location.replace("@input", str(self.paths["input"]))
                      for location in diagnostic["locations"]) for diagnostic in expected["diagnostics"]))
        run = dict(argv=checker.application_command(self.context, case),
                   returncode=1, timeout=False, stdout=str(self.paths["stdout"]),
                   stderr=str(self.paths["stderr"]), library=str(self.paths["library"]),
                   interface=str(self.paths["interface"]))
        for name in ("input", "stdout", "stderr"):
            run[name + "_artifact"] = checker.artifact(self.paths[name])
        return run, document

    def test_actual_verification_failure_gate(self):
        run, document = self.negative_fixture()
        self.assertEqual(checker.accept_negative(run, "semantic", self.context,
                                                self.failure_bodies), document)
        for mutation in (lambda doc: doc["verification-results"].update(success=True),
                         lambda doc: doc["verification-results"].update(verified=True),
                         lambda doc: doc["verification-results"].update(errors=0),
                         lambda doc: doc["verus-infer"].update(interface_written=True),
                         lambda doc: doc["verus-infer"]["verification_scope"].update(native_functions=[])):
            altered = copy.deepcopy(document)
            mutation(altered)
            self.paths["stdout"].write_text(json.dumps(altered))
            run["stdout_artifact"] = checker.artifact(self.paths["stdout"])
            with self.assertRaises(checker.ReceiptError):
                checker.accept_negative(run, "semantic", self.context, self.failure_bodies)

    def test_negative_timeout_panic_and_stale_publication(self):
        run, document = self.negative_fixture()
        run["timeout"] = True
        with self.assertRaises(checker.ReceiptError):
            checker.accept_negative(run, "semantic", self.context, self.failure_bodies)
        run["timeout"] = False
        self.paths["stderr"].write_text("panicked at assertion failed")
        run["stderr_artifact"] = checker.artifact(self.paths["stderr"])
        with self.assertRaises(checker.ReceiptError):
            checker.accept_negative(run, "semantic", self.context, self.failure_bodies)
        self.paths["stderr"].write_text("error: assertion failed")
        run["stderr_artifact"] = checker.artifact(self.paths["stderr"])
        self.paths["interface"].write_text("stale interface")
        with self.assertRaises(checker.ReceiptError):
            checker.accept_negative(run, "semantic", self.context, self.failure_bodies)

    def accept_failure_fixture(self, run, blocker):
        if blocker:
            return checker.accept_frontend_blocker(run, checker.EXPECTED_FRONTEND_BLOCKER, self.context)
        return checker.accept_negative(run, "semantic", self.context, self.failure_bodies)

    def test_failure_consumed_paths_and_decoy_bindings(self):
        for blocker in (False, True):
            run, document = self.blocker_fixture() if blocker else self.negative_fixture()
            self.assertEqual(self.accept_failure_fixture(run, blocker), document)
            for name in ("stdout", "stderr"):
                decoy = self.root / (name + "-decoy")
                decoy.write_bytes(self.paths[name].read_bytes())
                changed = copy.deepcopy(run)
                changed[name] = str(decoy)
                with self.assertRaises(checker.ReceiptError):
                    self.accept_failure_fixture(changed, blocker)
                changed[name + "_artifact"] = checker.artifact(decoy)
                with self.assertRaises(checker.ReceiptError):
                    self.accept_failure_fixture(changed, blocker)
            decoy = self.root / "input-decoy"
            decoy.write_text("unrelated non-Rust input")
            changed = copy.deepcopy(run)
            changed["input_artifact"] = checker.artifact(decoy)
            with self.assertRaises(checker.ReceiptError):
                self.accept_failure_fixture(changed, blocker)

    def test_failure_source_cannot_be_rebound_after_execution(self):
        run, _ = self.negative_fixture()
        self.paths["input"].write_text("unrelated assertion failure")
        run["input_artifact"] = checker.artifact(self.paths["input"])
        with self.assertRaisesRegex(checker.ReceiptError, "registered application input"):
            self.accept_failure_fixture(run, False)

    def test_all_result_classes_bind_runtime_and_command_context(self):
        mutations = [lambda argv: [], lambda argv: argv + ["--verify-function", "only_one"],
                     lambda argv: argv + ["--import=other=unbound.vir"],
                     lambda argv: argv[:3] + ["/unbound/verus"] + argv[4:],
                     lambda argv: argv[:argv.index("--sysroot") + 1] + ["/unbound/sysroot"] +
                                  argv[argv.index("--sysroot") + 2:]]
        for flag in ("--compile", "--no-cheating", "--output-json", "--num-threads", "--crate-type=lib"):
            mutations.append(lambda argv, flag=flag: [argument for argument in argv if argument != flag])
        for mutation in mutations:
            changed = copy.deepcopy(self.run)
            wanted = copy.deepcopy(self.expected)
            changed["argv"] = mutation(list(changed["argv"]))
            wanted["argv"] = list(changed["argv"])
            with self.assertRaisesRegex(checker.ReceiptError, "application command binding"):
                checker.accept_positive(changed, wanted, self.context)
        for blocker in (False, True):
            run, _ = self.blocker_fixture() if blocker else self.negative_fixture()
            for mutation in mutations:
                changed = copy.deepcopy(run)
                changed["argv"] = mutation(list(changed["argv"]))
                with self.assertRaisesRegex(checker.ReceiptError, "application command binding"):
                    self.accept_failure_fixture(changed, blocker)

    def test_context_cannot_select_an_unguarded_runtime(self):
        context = copy.deepcopy(self.context)
        context["execution"]["runtime"] = "/unbound/runtime"
        changed = copy.deepcopy(self.run)
        wanted = copy.deepcopy(self.expected)
        changed["argv"] = checker.application_command(context, "stateful_transition")
        wanted["argv"] = list(changed["argv"])
        with self.assertRaisesRegex(checker.ReceiptError, "selected execution inputs"):
            checker.accept_positive(changed, wanted, context)

    def test_failure_totals_and_complete_inference_schema(self):
        mutations = [lambda document: document["verification-results"].update(verified=999999),
                     lambda document: document["verification-results"].update(errors=999999),
                     lambda document: document["verus-infer"].update(functions=None),
                     lambda document: document["verus-infer"].update(generation_failure="unrelated failure"),
                     lambda document: document["verus-infer"].update(requested_functions=999999),
                     lambda document: document["verus-infer"].update(generated_functions=True),
                     lambda document: document["verus-infer"].update(failed_functions=[]),
                     lambda document: document["verus-infer"].update(imported_inference_dependencies=[{}]),
                     lambda document: document["verus-infer"].update(trusted_models=[{}]),
                     lambda document: document["verus-infer"].pop("generation_failure"),
                     lambda document: document["verus-infer"].pop("functions")]
        for blocker in (False, True):
            run, document = self.blocker_fixture() if blocker else self.negative_fixture()
            for mutation in mutations:
                changed = copy.deepcopy(document)
                mutation(changed)
                self.paths["stdout"].write_text(json.dumps(changed))
                run["stdout_artifact"] = checker.artifact(self.paths["stdout"])
                if changed == document:
                    continue
                with self.assertRaises(checker.ReceiptError):
                    self.accept_failure_fixture(run, blocker)

    def test_failure_intended_diagnostics_not_just_error_category(self):
        for blocker in (False, True):
            run, _ = self.blocker_fixture() if blocker else self.negative_fixture()
            original = self.paths["stderr"].read_text()
            for changed in ("error: unrelated verification failure\n", original.replace(":9:21", ":9:22")
                            if blocker else original.replace(":71:9", ":71:8"),
                            original + "\nerror: unrelated verification failure\n"):
                self.paths["stderr"].write_text(changed)
                run["stderr_artifact"] = checker.artifact(self.paths["stderr"])
                with self.assertRaisesRegex(checker.ReceiptError, "failure intended diagnostics"):
                    self.accept_failure_fixture(run, blocker)

    def test_failure_dynamic_fields_keep_their_validation(self):
        run, document = self.negative_fixture()
        with self.assertRaisesRegex(checker.ReceiptError, "negative registered kind"):
            checker.accept_negative(run, "frame", self.context, self.failure_bodies)
        mutations = [lambda report: report["verus-infer"]["metrics"].update(total_micros=0),
                     lambda report: report["verus-infer"]["functions"][0]["metrics"].update(solver_micros=0),
                     lambda report: report["verus-infer"]["functions"][0].update(candidate_id="not-an-identity"),
                     lambda report: report["verus-infer"]["trusted_models"][0].update(fingerprint="0" * 64)]
        for mutation in mutations:
            changed = copy.deepcopy(document)
            mutation(changed)
            self.paths["stdout"].write_text(json.dumps(changed))
            run["stdout_artifact"] = checker.artifact(self.paths["stdout"])
            with self.assertRaises(checker.ReceiptError):
                self.accept_failure_fixture(run, False)

    def test_dangling_failure_outputs_are_not_absent(self):
        for blocker in (False, True):
            run, _ = self.blocker_fixture() if blocker else self.negative_fixture()
            for name in ("interface", "library"):
                self.paths[name].symlink_to(self.root / "missing-output-target")
                with self.assertRaisesRegex(checker.ReceiptError, "failure published " + name):
                    self.accept_failure_fixture(run, blocker)
                self.paths[name].unlink()

    def test_transition_not_accepted_as_complete_resize(self):
        provider = checker.CompleteResizeProvider(checker.FIXTURES / "transition.rs",
            self.report_expected, "stateful_transition::resize", lambda source: source)
        with self.assertRaisesRegex(checker.ReceiptError, "complete resize exact body scope"):
            checker.connect_complete(provider, "not native resize")
        self.assertNotIn("transition", checker.COMPLETE_RESIZE_PROVIDERS)

    def test_registered_reference_has_full_ordinary_scope(self):
        provider = checker.COMPLETE_RESIZE_PROVIDERS["reference88"]
        expected = provider.expected_report
        self.assertFalse(provider.enabled)
        self.assertFalse(expected["interface_written"])
        self.assertEqual(expected["helpers"], {})
        self.assertEqual(expected["trusted_models"], [])
        self.assertEqual(expected["verified"], 22)
        self.assertEqual(sum(row["body_status"] == "checked_exec_body"
                             for row in expected["bodies"].values()), 13)
        self.assertEqual(sum(row["body_status"] == "checked_proof_body"
                             for row in expected["bodies"].values()), 3)
        for name in ("required_words", "resize", "get", "set", "clear"):
            self.assertIn("stateful_resize::U64GroupedBitmap::" + name, expected["bodies"])

    def test_complete_reference_report_uses_integrated_scope_gates(self):
        self.report_expected = checker.reference88_report("reference", "stateful_transition")
        self.expected["report"] = self.report_expected
        self.document = synthetic_report(self.report_expected)
        self.refresh()
        self.accept()
        self.rejects_document(lambda document: document["verus-infer"]["verification_scope"]
                              .update(checked_exec_bodies=999))
        self.rejects_document(lambda document: document["verus-infer"]["verification_scope"]
                              ["native_functions"].pop())
        self.rejects_document(lambda document: document["verus-infer"]
                              .update(interface_written=True))

    def test_complete_projection_checks_non_resize_bodies_and_struct(self):
        from stateful_bitmap_reference88.audit import tokens, extract, erase, METHODS
        provider = checker.COMPLETE_RESIZE_PROVIDERS["reference88"]
        source = provider.source.read_text()
        items = tokens(source)
        implementation = extract(items, ["impl", "U64GroupedBitmap"])
        native = "pub(crate) struct U64GroupedBitmap { " + " ".join(
            extract(items, ["struct", "U64GroupedBitmap"])) + " }\n"
        native += "fn bits_in_range() { " + " ".join(erase(extract(items,
            ["fn", "bits_in_range", "("]))[0]) + " }\nimpl U64GroupedBitmap {\n"
        for name in METHODS:
            prefix = "pub " if name == "resize" else ""
            signature = "fn resize(&mut self, new_len: u32, full: bool)" if name == "resize" else "fn " + name + "()"
            native += prefix + signature + " { " + " ".join(
                erase(extract(implementation, ["fn", name, "("]))[0]) + " }\n"
        native += "}\n"
        result = checker.connect_complete(provider, native)
        self.assertTrue(result["full_native_projection"]["all_nine_bodies"])
        self.assertEqual(len(result["full_native_projection"]["bodies"]), 9)
        for changed in (source.replace("group |= Self::select_mask(bit_index);",
                                       "group &= Self::select_mask(bit_index);"),
                        source.replace("struct U64GroupedBitmap {\n    len: u32,",
                                       "struct U64GroupedBitmap {\n    len: u64,", 1)):
            self.assertNotEqual(changed, source)
            candidate = self.root / "changed-reference.rs"
            candidate.write_text(changed)
            with self.assertRaisesRegex(checker.ReceiptError, "nine-body native projection"):
                checker.connect_complete(replace(provider, source=candidate), native)

    def test_ordinary_import_application_does_not_bind_inference_certificate(self):
        case = "stateful_resize_consumer"
        source = checker.reference88_import_source()
        expected = checker.reference88_report("import", case)
        self.context["applications"][case] = dict(
            source_sha256=checker.hashlib.sha256(source.encode()).hexdigest(), enabled=False)
        runner = object.__new__(checker.Replay)
        runner.context = self.context
        runner.output = Path(self.context["execution"]["output"])
        runner.runtime = Path(self.context["execution"]["runtime"])
        runner.rustup = Path(self.context["execution"]["rustup"])
        runner.sysroot = Path(self.context["execution"]["sysroot"])
        runner.environment = {}
        runner.runs = []
        runner.producer = "source-synthetic"
        imports = {"stateful_resize": {name: checker.artifact(self.paths[name])
                                       for name in ("interface", "library")}}

        def execute(name, argv, category, environment):
            self.assertEqual(category, "application")
            self.assertNotIn("VSTD_KIND", environment)
            paths = checker.application_paths(self.context, name)
            paths["stdout"].parent.mkdir(parents=True)
            paths["stdout"].write_text(json.dumps(synthetic_report(expected)))
            paths["stderr"].write_text("")
            for kind in ("library", "interface"):
                paths[kind].write_bytes(b"synthetic ordinary artifact, not proof evidence")
            runner.runs.append({})
            return dict(argv=argv, returncode=0, timeout=False,
                        stdout=str(paths["stdout"]), stderr=str(paths["stderr"]),
                        stdout_artifact=checker.artifact(paths["stdout"]),
                        stderr_artifact=checker.artifact(paths["stderr"]))

        with patch.object(runner, "execute", side_effect=execute), patch(
                "infer_transfer_receipts.metadata", side_effect=AssertionError("ordinary import requested certificate")):
            accepted = runner.application(case, source, expected, enabled=False, imports=imports)
        self.assertNotIn("certificate", accepted)
        self.assertEqual(accepted["report"]["verification-results"]["verified"], 4)

    def test_complete_provider_cli_can_exit_zero_only_with_complete_receipt(self):
        receipt = dict(status="complete_resize_established", full_resize_complete=True)
        with patch.object(checker, "Replay"), patch.object(checker, "replay", return_value=receipt) as replay, \
                patch("sys.stdout", new_callable=io.StringIO):
            actual = checker.main(["--native-checkout", str(self.root / "native"),
                                   "--output", str(self.root / "not-created"), "--resize-provider", "reference88"])
        self.assertEqual(actual, 0)
        self.assertIs(replay.call_args.args[1], checker.COMPLETE_RESIZE_PROVIDERS["reference88"])

    def test_bridge_scope_retains_assertion_only_extremes_and_no_inference(self):
        expected = checker.reference88_report("bridge", "stateful_bridge")
        self.assertFalse(expected["bodies"]["stateful_bridge::extremes"]["ordinary_contract"])
        self.assertTrue(expected["bodies"]["stateful_bridge::exact"]["ordinary_contract"])
        self.assertFalse(checker.application_sources()["stateful_bridge"]["enabled"])
        self.assertFalse(checker.application_sources()["required_words"]["enabled"])

    def test_production_has_no_python_assert_gates(self):
        tree = ast.parse(Path(checker.__file__).read_text())
        self.assertFalse(any(isinstance(node, ast.Assert) for node in ast.walk(tree)))

    def test_source_derived_producer_changes(self):
        before = checker.identity(checker.source_manifest(self.repo))
        (self.repo / "source/engine/lib.rs").write_text("changed compiler bytes")
        self.assertNotEqual(before, checker.identity(checker.source_manifest(self.repo)))


if __name__ == "__main__":
    unittest.main()
