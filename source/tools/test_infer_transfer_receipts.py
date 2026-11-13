"""Synthetic adversarial tests for the new independent acceptance boundaries."""

import copy
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

from infer_transfer_identity import INTERFACE_POLICY, INTERFACE_SCHEMA
import check_infer_transfers as transfers
import infer_transfer_dependencies as dependencies
import infer_transfer_receipts as acceptance
from infer_eval_support import ArtifactGuard, digest
from test_infer_transfers import proof_report, scenario_receipt
from test_fedimint_full_request import fixture, FIXTURES


class PolynomialTests(unittest.TestCase):
    def test_nia_diagnostic_is_required_and_is_not_an_authored_counterexample(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            verifier = root / "verus"
            path, receipt = scenario_receipt(root / "run", "redb", verifier)
            label = "false_polynomial"
            receipt["transfer"]["polynomial_auxiliary_negative_cases"] = [label]
            entry = copy.deepcopy(receipt["commands"][-1])
            entry.update(label=label)
            entry["argv"][-1] = str(path.parent / (label + ".vir"))
            receipt["commands"].append(entry)
            report = proof_report(False)
            stdout, stderr = path.parent / (label + ".stdout"), path.parent / (label + ".stderr")
            stdout.write_text(json.dumps(report))
            diagnostic = "Verus Infer: polynomial proposal not satisfied"
            stderr.write_text(diagnostic)
            path.write_text(json.dumps(receipt))
            summary = transfers.check_receipt(path, "redb", verifier)
            self.assertEqual(summary["proofs"][-1]["failure_kind"], "polynomial_auxiliary")
            for invalid in ("postcondition not satisfied", "bitvector assertion not satisfied",
                            diagnostic + "; precondition not satisfied", diagnostic + "; postcondition not satisfied",
                            diagnostic + "; resource limit exceeded", diagnostic + "; possible arithmetic underflow/overflow"):
                stderr.write_text(invalid)
                with self.subTest(diagnostic=invalid), self.assertRaises(ValueError):
                    transfers.check_receipt(path, "redb", verifier)
            stderr.write_text(diagnostic)
            for mutate in (lambda data: data["verus-infer"].update(status="verification_failed"),
                           lambda data: data["verification-results"].update(errors=0),
                           lambda data: data["verification-results"].update(**{"is-verifying-entire-crate": False}),
                           lambda data: data["verus-infer"]["functions"][0].update(required_checks_passed="true")):
                changed = copy.deepcopy(report)
                mutate(changed)
                stdout.write_text(json.dumps(changed))
                with self.subTest(mutation=mutate), self.assertRaises(ValueError):
                    transfers.check_receipt(path, "redb", verifier)
            stdout.write_text(json.dumps(report))
            receipt["transfer"]["semantic_negative_cases"].append(label)
            path.write_text(json.dumps(receipt))
            with self.assertRaises(ValueError):
                transfers.check_receipt(path, "redb", verifier)


def scope(name, positive):
    methods = ["deadline_policy", "deadline_boundary", "deadline_monotonicity", "deadline_irrelevance"]
    if name != "client":
        methods += ["InterceptPaymentRequest::" + method for method in acceptance.CONTRACTS]
    if name in ("overflow", "panic"):
        methods.append("overflow" if name == "overflow" else "original_panic")
    functions = [dict(rust_function=method, selected=True, automatically_derived=False,
                      body_status="checked_exec_body" if positive else "selected_body_not_certified_by_successful_full_run")
                 for method in methods]
    if name != "client":
        functions.append(dict(rust_function="VERUS_UNERASED_PROXY__NO_INCOMING_CIRCUIT", selected=True,
                              automatically_derived=False, body_status="selected_item_without_translated_body"))
    return dict(native_collection_completed=True, complete_selected_scope_verified=positive,
                native_functions=functions, checked_exec_bodies=len(methods) if positive else 0,
                checked_proof_bodies=0)


class FullRequestTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="full acceptance ")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.verifier, self.solver = self.root / "verus", self.root / "z3"
        self.bitcoin = self.root / "dependency-target/release/deps/libbitcoin-test.rlib"
        self.bitcoin.parent.mkdir(parents=True)
        self.bitcoin.write_text("real dependency stand-in for synthetic harness testing only")
        self.path = self.root / "receipt.json"
        self.contracts = {name: acceptance.full.text_digest(name) for name in acceptance.CONTRACTS}
        self.addCleanup(patch.stopall)
        patch.object(acceptance, "CONTRACTS", self.contracts).start()
        patch.object(acceptance, "REGRESSION_CONTRACT", acceptance.full.text_digest("regressed")).start()
        self.sources = {name: "synthetic " + name for name in
                        ["provider", "client", "regression", "false_policy", "overflow", "panic", *acceptance.FRONTEND]}
        for name, source in self.sources.items():
            (self.root / (name + ".rs")).write_text(source)
        patch.object(acceptance, "full_inputs", return_value=self.sources).start()
        patch.object(acceptance, "full_dependencies", return_value=self.bitcoin).start()
        patch.object(acceptance, "engine_ledger").start()
        self.snapshot_checks = patch.object(acceptance, "snapshot").start()
        patch.object(acceptance, "tool_artifacts", return_value=[]).start()
        certificate = dict(schema=INTERFACE_SCHEMA, foreign_dependencies=[], policy=INTERFACE_POLICY, soundness=True, maximality=True,
            exactness=True, summaries="f" * 64, dependencies=[], trusted_models=[[{}, "f" * 64]],
            proofs=[[dict(path=dict(krate=dict(Id=["full_provider", 7]), segments=[name])),
                     dict(soundness=True, maximality=True, exactness=True)] for name in self.contracts])
        self.blob = b"synthetic API\x00" + json.dumps(certificate, separators=(",", ":")).encode() + b"VRINFER1"
        (self.root / "libfull_provider.rlib").write_text("synthetic checked provider")
        for label in ("provider_cold", "provider_warm"):
            (self.root / (label + ".vir")).write_bytes(self.blob)
        (self.root / "native.rlib").write_text("synthetic native typing")
        self.receipt = dict(success=True, commands=[], cases=[], items={}, dependency=dict(packages=[]),
            engine_source_sha256={}, artifacts_before={}, artifacts_after={},
            provider_artifacts_before={str(self.root / "libfull_provider.rlib"): {}, str(self.root / "provider_warm.vir"): {}},
            provider_artifacts_after={}, native_rlib_sha256=digest(self.root / "native.rlib"),
            dependency_sources_before={},
            funding_caller=dict(status="blocked"), transfer=dict(
                scenario="fedimint-full-request", application_commit=acceptance.full.APPLICATION,
                status="proved", source_fidelity=True, trust=["explicit opaque trust"], limitations=["not funding safety"],
                positive_cases=["provider_cold", "provider_warm", "client_cold", "client_warm"],
                semantic_negative_cases=["regression_cold", "false_policy_cold"],
                safety_negative_cases=["overflow_cold", "panic_cold"],
                frontend_negative_cases=[name + "_cold" for name in acceptance.FRONTEND] +
                    ["client_metadata_" + fault for fault in acceptance.IMPORT_FAULTS], stale_export_rejected=True))
        self.setup_command("rustc", ["rustup", "which", "--toolchain", "1.99.1", "rustc"], "/synthetic/rustc")
        self.setup_command("application_head", ["git", "-C", str(self.root), "rev-parse", "HEAD"], acceptance.full.APPLICATION)
        dependency_arguments = ["--extern", "bitcoin=" + str(self.bitcoin), "-L", "dependency=" + str(self.bitcoin.parent)]
        self.setup_command("native_build", ["/synthetic/rustc", str(self.root / "native.rs"), "--edition=2021",
            "--crate-type=lib", "-C", "overflow-checks=yes", "-o", str(self.root / "native.rlib"), *dependency_arguments])
        for name in ("provider", "client"):
            for phase in ("cold", "warm"):
                self.add_case(name, phase, "positive")
        for name in ("regression", "false_policy", "overflow", "panic"):
            self.add_case(name, "cold", "semantic" if name in ("regression", "false_policy") else "safety")
        for name in acceptance.FRONTEND:
            self.add_case(name, "cold", "frontend")
        for fault in acceptance.IMPORT_FAULTS:
            def edit(value):
                if fault == "policy": value["policy"] = "wrong-policy"
                elif fault == "proof": value["exactness"] = False
                elif fault == "summaries": value["summaries"] = "0" * 64
                elif fault == "dependencies": value["dependencies"] = [dict(crate_id=0, fingerprint="0" * 64)]
                else: value["trusted_models"][0][1] = "0" * 64
            (self.root / ("bad-" + fault + ".vir")).write_bytes(self.blob[:-8] if fault == "missing" else
                acceptance.full.replace_metadata(self.blob, edit))
            self.add_case("client", "metadata_" + fault, "frontend")

    def setup_command(self, label, argv, stdout=""):
        self.receipt["commands"].append(dict(label=label, argv=argv, exit_code=0, cwd=str(self.root)))
        (self.root / (label + ".stdout")).write_text(stdout)
        (self.root / (label + ".stderr")).write_text("")

    def add_case(self, name, phase, category, tool=None):
        label, positive = name + "_" + phase, category == "positive"
        source_name = "provider" if name == "baseline_opaque" else name
        artifact, source = self.root / (label + ".vir"), self.root / (source_name + ".rs")
        argv = [str(tool or self.verifier), str(source), "--crate-type=lib", "--edition=2021", "--crate-name",
                "full_" + name, "--no-cheating", "--output-json", "--num-threads", "1", "--export", str(artifact),
                "--extern", "bitcoin=" + str(self.bitcoin), "-L", "dependency=" + str(self.bitcoin.parent)]
        if name == "client":
            interface = self.root / ("bad-" + phase.removeprefix("metadata_") + ".vir") if phase.startswith("metadata_") else self.root / "provider_warm.vir"
            argv += ["--extern", "full_provider=" + str(self.root / "libfull_provider.rlib"), "--import", "full_provider=" + str(interface),
                     "-L", "dependency=" + str(self.root)]
        if name == "provider":
            argv += ["--compile", "-o", str(self.root / "libfull_provider.rlib")]
        data = proof_report(positive)
        inference, result = data["verus-infer"], data["verification-results"]
        inference.update(requested_functions=0 if name == "client" else 2, generated_functions=0 if name == "client" else 2,
            failed_functions=[] if positive else ["full_" + name + "::deadline_policy"], verification_scope=scope(name, positive))
        inference["functions"] = []
        if name != "client":
            for method in self.contracts:
                function = copy.deepcopy(proof_report(True)["verus-infer"]["functions"][0])
                function.update(function="full_" + name + "::InterceptPaymentRequest::" + method,
                    exported=positive, available_to_same_crate_callers=True,
                    contract=dict(requires="true", ensures="regressed" if name == "regression" and method != "incoming_circuit" else method),
                    metrics=dict(cache="miss", cache_policy="regenerated_state"))
                inference["functions"].append(function)
        if name == "client" and positive:
            inference["status"] = "no_inference"
            inference["imported_inference_dependencies"] = [dict(crate_id=7, fingerprint=acceptance.metadata(self.blob)[1])]
        result["verified"] = 4 if name == "client" else 10 if category == "semantic" else 11
        stderr = ""
        if category == "semantic":
            stderr = "postcondition not satisfied; request.incoming_chan_id != 0 || request.htlc_id != 0"
        if category == "safety":
            stderr = "possible arithmetic underflow/overflow" if name == "overflow" else 'precondition not satisfied; panic!("safety control"); false,'
        if category == "frontend":
            result.update(**{"encountered-error": True, "encountered-vir-error": True})
            stderr = acceptance.FRONTEND.get(name, "Verus Infer: " + acceptance.IMPORT_DIAGNOSTICS.get(phase.removeprefix("metadata_"), "unsupported"))
            if name == "baseline_opaque":
                stderr = "Verus Infer unsupported"
        if positive:
            artifact.write_bytes(self.blob)
        self.setup_command(label, argv, json.dumps(data))
        self.receipt["commands"][-1].update(exit_code=0 if positive else 1, environment=dict(VERUS_Z3_PATH=str(self.solver)))
        (self.root / (label + ".stderr")).write_text(stderr)
        self.receipt["cases"].append(dict(name=name, phase=phase, category=category,
            seeded_export_sha256=None if positive else digest(self.root / "provider_cold.vir"),
            export_sha256=digest(artifact) if positive else None, source_sha256=digest(source),
            verification=result, inference=inference))

    def check(self):
        self.path.write_text(json.dumps(self.receipt))
        return transfers.check_receipt(self.path, "fedimint-full-request", self.verifier)

    def row(self, label):
        return next(row for row in self.receipt["cases"] if row["name"] + "_" + row["phase"] == label)

    def mutate(self, label, edit):
        stdout = self.root / (label + ".stdout")
        data = json.loads(stdout.read_text())
        edit(data)
        stdout.write_text(json.dumps(data))
        row = self.row(label)
        row.update(verification=data["verification-results"], inference=data["verus-infer"])

    def test_accepts_all_23_current_controls_with_import_only_clients(self):
        summary = self.check()
        self.assertEqual(len(summary["proofs"]), 8)
        self.assertEqual(len(summary["frontend_controls"]), 15)
        self.assertEqual(summary["baseline_comparisons"], [])
        self.assertEqual(summary["proofs"][2]["inference_checks"], [])
        self.assertEqual(summary["proofs"][-2]["failure_kind"], "arithmetic")
        required = {Path(path) for call in self.snapshot_checks.call_args_list
                    for path in (call.args[2] if len(call.args) > 2 else call.kwargs.get("required", []))}
        self.assertTrue({self.root / (name + ".source.rs") for name in acceptance.full.CALLERS} <= required)
        self.assertFalse({self.root / (name + ".rs") for name in acceptance.full.CALLERS} & required)

    def test_missing_duplicate_and_reclassified_cases_fail_closed(self):
        original = copy.deepcopy(self.receipt)
        for edit in (lambda receipt: receipt["cases"].pop(),
                     lambda receipt: receipt["cases"].append(receipt["cases"][0]),
                     lambda receipt: receipt["transfer"]["positive_cases"].append("provider_cold"),
                     lambda receipt: receipt["transfer"]["frontend_negative_cases"].pop(),
                     lambda receipt: receipt["cases"][-1].update(category="semantic"),
                     lambda receipt: receipt["commands"].append(receipt["commands"][-1]),
                     lambda receipt: receipt["transfer"].update(auxiliary_negative_cases=["overflow_cold"])):
            self.receipt = copy.deepcopy(original)
            edit(self.receipt)
            with self.subTest(edit=edit), self.assertRaises(ValueError):
                self.check()

    def test_exactness_domains_equations_exports_caches_and_scope_cannot_be_faked(self):
        original = copy.deepcopy(self.receipt)
        edits = [lambda data: data["verus-infer"]["functions"].pop(),
                 lambda data: data["verus-infer"]["functions"][0].update(function="wrong::helper"),
                 lambda data: data["verus-infer"]["functions"][0].update(required_checks_passed="true"),
                 lambda data: data["verus-infer"]["functions"][0]["contract"].update(requires="x > 0"),
                 lambda data: data["verus-infer"]["functions"][0]["contract"].update(ensures="false"),
                 lambda data: data["verus-infer"]["functions"][0]["metrics"].update(cache="hit"),
                 lambda data: data["verus-infer"]["functions"][0].update(exported=1),
                 lambda data: data["verus-infer"]["verification_scope"].update(complete_selected_scope_verified=1),
                 lambda data: data["verus-infer"]["verification_scope"]["native_functions"][0].update(body_status="skipped"),
                 lambda data: data["verification-results"].update(verified=10),
                 lambda data: data["verification-results"].update(errors=False),
                 lambda data: data["verification-results"].update(success=1)]
        for obligation in ("soundness", "maximality", "exactness"):
            edits.append(lambda data, key=obligation: data["verus-infer"]["functions"][1]["proofs"][key].update(status="skipped"))
        for edit in edits:
            self.receipt = copy.deepcopy(original)
            (self.root / "provider_cold.stdout").write_text(json.dumps(dict(
                **{"verification-results": self.row("provider_cold")["verification"], "verus-infer": self.row("provider_cold")["inference"]})))
            self.mutate("provider_cold", edit)
            with self.subTest(edit=edit), self.assertRaises(ValueError):
                self.check()

    def test_import_clients_require_zero_locals_and_matching_provider_dependencies(self):
        for edit in (lambda data: data["verus-infer"].update(imported_inference_dependencies=[]),
                     lambda data: data["verus-infer"]["imported_inference_dependencies"][0].update(fingerprint="0" * 64),
                     lambda data: data["verus-infer"].update(requested_functions=False),
                     lambda data: data["verus-infer"].update(functions=[proof_report(True)["verus-infer"]["functions"][0]]),
                     lambda data: data["verification-results"].update(verified=3)):
            original = copy.deepcopy(self.row("client_cold"))
            stdout = (self.root / "client_cold.stdout").read_text()
            self.mutate("client_cold", edit)
            with self.subTest(edit=edit), self.assertRaises(ValueError):
                self.check()
            self.row("client_cold").update(original)
            (self.root / "client_cold.stdout").write_text(stdout)

    def test_authored_semantics_safety_and_frontend_diagnostics_are_disjoint(self):
        for label, invalid in (("regression_cold", "precondition not satisfied"),
                               ("false_policy_cold", "Verus Infer: polynomial proposal not satisfied"),
                               ("overflow_cold", "postcondition not satisfied"),
                               ("panic_cold", "possible arithmetic underflow/overflow"),
                               ("client_metadata_policy", "Verus Infer: inferred contract fingerprint mismatch"),
                               ("opaque_access_cold", "resource limit; must remain unused"),
                               ("malformed_cold", "blocks == true; postcondition not satisfied")):
            stderr = self.root / (label + ".stderr")
            original = stderr.read_text()
            stderr.write_text(invalid)
            with self.subTest(label=label), self.assertRaises(ValueError):
                self.check()
            stderr.write_text(original)

    def test_nonzero_exits_seed_binding_absence_and_raw_report_agreement_are_mandatory(self):
        original = copy.deepcopy(self.receipt)
        for edit in (lambda receipt: self.row("opaque_access_cold").update(seeded_export_sha256="0" * 64),
                     lambda receipt: self.row("overflow_cold")["verification"].update(verified=1),
                     lambda receipt: acceptance.command(receipt, "opaque_access_cold").update(exit_code=0),
                     lambda receipt: acceptance.command(receipt, "overflow_cold").update(exit_code=True),
                     lambda receipt: acceptance.command(receipt, "provider_warm")["argv"].append("--no-verify"),
                     lambda receipt: acceptance.command(receipt, "client_warm")["argv"].append("--no-cheating=false"),
                     lambda receipt: acceptance.command(receipt, "provider_cold")["argv"].__setitem__(-1, "substitute.rlib")):
            self.receipt = copy.deepcopy(original)
            edit(self.receipt)
            with self.subTest(edit=edit), self.assertRaises(ValueError):
                self.check()
        self.receipt = original
        artifact = self.root / "panic_cold.vir"
        artifact.write_bytes(self.blob)
        with self.assertRaises(ValueError):
            self.check()
        artifact.unlink()
        artifact.symlink_to(self.root / "absent")
        with self.assertRaises(ValueError):
            self.check()

    def test_corrupted_import_must_be_the_declared_exact_mutation(self):
        path = self.root / "bad-summaries.vir"
        path.write_bytes(self.blob)
        with self.assertRaisesRegex(ValueError, "corrupted-import"):
            self.check()

    def test_optional_baseline_controls_stay_separate_and_cannot_replace_current_cases(self):
        baseline = self.root / "baseline-verus"
        self.receipt.update(baseline_artifacts_before={}, baseline_artifacts_after={})
        self.add_case("unsupported", "baseline", "frontend", baseline)
        self.receipt["transfer"]["frontend_negative_cases"].append("unsupported_baseline")
        with self.assertRaisesRegex(ValueError, "incomplete optional baseline"):
            self.check()
        self.add_case("baseline_opaque", "baseline", "frontend", baseline)
        self.receipt["transfer"]["frontend_negative_cases"].append("baseline_opaque_baseline")
        summary = self.check()
        self.assertEqual(len(summary["baseline_comparisons"]), 2)
        self.assertEqual(len(summary["frontend_controls"]), 15)
        self.receipt["cases"].remove(self.row("unsupported_cold"))
        with self.assertRaises(ValueError):
            self.check()


class EvidencePrimitiveTests(unittest.TestCase):
    def test_duplicate_json_keys_are_rejected_even_when_values_agree(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "receipt.json"
            path.write_text('{"success": true, "success": true}')
            with self.assertRaises(ValueError):
                acceptance.read_json(path)

    def test_artifact_snapshots_recheck_real_bytes_and_missing_guard_entries(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary).resolve() / "input"
            path.write_text("original")
            before = ArtifactGuard([path]).initial
            acceptance.snapshot(before, copy.deepcopy(before), [path])
            for required in ([path.parent / "omitted"],):
                with self.assertRaises(ValueError):
                    acceptance.snapshot(before, before, required)
            path.write_text("tampered")
            with self.assertRaises(ValueError):
                acceptance.snapshot(before, before, [path])

    def test_registry_archive_checksums_source_bytes_extra_files_links_and_duplicates(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source = root / "src/package-1.0"
            source.mkdir(parents=True)
            (source / "Cargo.toml").write_text("original")
            archive = root / "package-1.0.crate"
            def pack(extra=None):
                with tarfile.open(archive, "w:gz") as stream:
                    stream.add(source / "Cargo.toml", arcname="package-1.0/Cargo.toml")
                    if extra is not None:
                        stream.addfile(extra, io.BytesIO(b"x") if extra.isfile() else None)
            pack()
            expected = digest(archive)
            self.assertEqual(len(dependencies.archive_sources(archive, source, expected)), 2)
            with self.assertRaises(ValueError):
                dependencies.archive_sources(archive, source, "0" * 64)
            (source / "Cargo.toml").write_text("tampered")
            with self.assertRaises(ValueError):
                dependencies.archive_sources(archive, source, expected)
            (source / "Cargo.toml").write_text("original")
            (source / "extra.rs").write_text("unexpected build input")
            with self.assertRaises(ValueError):
                dependencies.archive_sources(archive, source, expected)
            (source / "extra.rs").unlink()
            for name, kind in (("package-1.0/link", tarfile.SYMTYPE), ("../escape", tarfile.REGTYPE),
                               ("package-1.0/Cargo.toml", tarfile.REGTYPE)):
                member = tarfile.TarInfo(name)
                member.type, member.size = kind, 1 if kind == tarfile.REGTYPE else 0
                pack(member)
                with self.subTest(name=name), self.assertRaises(ValueError):
                    dependencies.archive_sources(archive, source, digest(archive))

    def test_locked_graph_has_all_21_exact_application_pins(self):
        original = dependencies.LOCK.read_text()
        selected = dependencies.packages(original)
        self.assertEqual(len(selected), 21)
        for wrong in (original.replace('version = "0.32.8"', 'version = "0.32.9"'),
                      original.replace(selected[0]["checksum"], "0" * 64),
                      original + original.split("[[package]]")[1].join(["[[package]]", ""])):
            with self.subTest(lock=wrong[:60]), self.assertRaises(ValueError):
                dependencies.packages(wrong)


class FullSourceInventoryTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="source inventory ")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.application = self.root / "application"
        self.directory = self.root / "replay"
        self.directory.mkdir()
        constant, definition, implementation, derive = fixture()
        contents = {acceptance.full.LIGHTNING: derive + "\n" + definition + "\n" + constant + "\n" + implementation,
                    acceptance.full.CLIENT: "const LNV2_CLAIM_DEADLINE_MARGIN: u32 = 2;\n",
                    acceptance.full.SERVER: "", "Cargo.lock": dependencies.LOCK.read_text()}
        fixtures = {}
        for name, (relative, marker, expected) in acceptance.full.CALLERS.items():
            text = acceptance.full.item((FIXTURES / (name + ".source.rs")).read_text(), marker)
            contents[relative] += "\n" + text
            (self.directory / (name + ".source.rs")).write_text(text)
            fixtures[name] = dict(source=relative, marker=marker, sha256=expected)
        hashes = {}
        for relative, text in contents.items():
            for base in (self.application, self.directory / "application-inputs"):
                path = base / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text)
            hashes[relative] = digest(self.application / relative)
        self.addCleanup(patch.stopall)
        patch.object(acceptance.full, "INPUT_HASHES", hashes).start()
        self.receipt = dict(source_sha256=hashes, commands=[], transfer=dict(field_count=7,
            trust=["Explicit audited external-type correspondence for bitcoin_hashes::sha256::Hash"]),
            items=dict(request=acceptance.full.REQUEST_HASH, implementation=acceptance.full.IMPL_HASH,
                methods=acceptance.full.METHOD_HASHES, constant=acceptance.full.text_digest(constant),
                omitted_derive=acceptance.full.text_digest(derive), opaque_correspondence=acceptance.full.text_digest(acceptance.full.OPAQUE)),
            funding_caller=dict(status="blocked", fixtures=fixtures, first_blocker="An async persistent lookup is unproved"))
        self.receipt["adaptations"] = [
            dict(change="Omit the adjacent Debug/Serialize/Deserialize/Clone derive only", original=derive,
                 before_sha256=acceptance.full.text_digest(derive), after_sha256=acceptance.full.text_digest("")),
            dict(change="Add one Infer attribute before each complete original method item",
                 before_sha256=acceptance.full.text_digest(implementation), after_sha256=acceptance.full.text_digest(
                     implementation.replace("    pub fn ", "    #[verifier::infer]\n    pub fn "))),
            dict(change="Wrap exact request, constant and annotated impl in verus! and import real sha256 type",
                 request_sha256=acceptance.full.REQUEST_HASH),
            dict(change="Add explicit opaque external-type identity correspondence; no SHA/layout/trait laws",
                 sha256=acceptance.full.text_digest(acceptance.full.OPAQUE))]
        for label, arguments, text in (("application_head", ["rev-parse", "HEAD"], acceptance.full.APPLICATION),
                ("application_clean", ["status", "--porcelain"], ""), ("application_clean_after", ["status", "--porcelain"], "")):
            self.receipt["commands"].append(dict(label=label, argv=["git", "-C", str(self.application), *arguments], exit_code=0))
            (self.directory / (label + ".stdout")).write_text(text)
        for name, text in acceptance.full_sources(constant, definition, implementation).items():
            (self.directory / (name + ".rs")).write_text(text)
        (self.directory / "native.rs").write_text("use bitcoin::hashes::sha256;\n" + constant + "\n" + definition + "\n" + implementation + "\n")

    def check(self):
        return acceptance.full_inputs(self.directory, self.receipt)

    def test_seven_real_fields_complete_items_explicit_correspondence_and_blocked_trace(self):
        self.assertEqual(len(self.check()), 15)
        self.assertEqual(self.receipt["funding_caller"]["status"], "blocked")

    def test_source_identity_adaptations_correspondence_and_funding_status_are_not_flags(self):
        original = copy.deepcopy(self.receipt)
        for mutation in (lambda receipt: receipt["items"].update(request="0" * 64),
                         lambda receipt: receipt["items"].update(methods={}),
                         lambda receipt: receipt["items"].update(opaque_correspondence="0" * 64),
                         lambda receipt: receipt.update(adaptations=[]),
                         lambda receipt: receipt["transfer"].update(field_count=True),
                         lambda receipt: receipt["transfer"].update(trust=["implicit external types"]),
                         lambda receipt: receipt["funding_caller"].update(status="proved"),
                         lambda receipt: receipt["funding_caller"].update(first_blocker=""),
                         lambda receipt: receipt["funding_caller"]["fixtures"].popitem()):
            self.receipt = copy.deepcopy(original)
            mutation(self.receipt)
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                self.check()

    def test_tampered_or_missing_proof_application_or_caller_sources_fail_closed(self):
        for path in (self.directory / "provider.rs", self.directory / "regression.rs", self.directory / "client.rs",
                     self.directory / "native.rs", self.directory / "relay_incoming_htlc.source.rs",
                     self.application / acceptance.full.LIGHTNING):
            original = path.read_bytes()
            path.write_bytes(original + b"\ntampered")
            with self.subTest(path=path), self.assertRaises(ValueError):
                self.check()
            path.write_bytes(original)


if __name__ == "__main__":
    unittest.main()
