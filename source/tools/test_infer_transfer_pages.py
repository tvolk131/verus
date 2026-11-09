"""Independent end-to-end synthetic receipt tests for the relational page slice."""

import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from infer_transfer_identity import INTERFACE_POLICY, INTERFACE_SCHEMA
import check_infer_transfers as transfers
import infer_transfer_receipts as acceptance
from infer_eval_support import ArtifactGuard, digest
from test_infer_transfers import proof_report


class PageReceiptTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="page acceptance ")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.application, self.output = self.root / "application", self.root / "replay"
        self.output.mkdir()
        self.verifier = self.root / "verus"
        parts = dict(PageNumber="pub(crate) struct PageNumber {pub(crate) region:u32,pub(crate) page_index:u32,pub(crate) page_order:u8}",
            page_size_bytes="    pub(crate) fn page_size_bytes(&self,size:u32)->u64 {let pages=1u64<<self.page_order;pages*u64::from(size)}",
            address_range="    pub(crate) fn address_range(&self,offset:u64,size:u64,start:u64,page_size:u32)->Range<u64> {debug_assert!(start<size);let end=start+self.page_size_bytes(page_size);start..end}")
        original = self.application / acceptance.pages.BASE
        original.parent.mkdir(parents=True)
        original.write_text(parts["PageNumber"] + "\nimpl PageNumber {\n" + parts["page_size_bytes"] + "\n" + parts["address_range"] + "\n}\n")
        hashes = {name: acceptance.full.text_digest(text + "\n") for name, text in parts.items()}
        self.addCleanup(patch.stopall)
        patch.object(acceptance.pages, "SOURCE_HASH", digest(original)).start()
        patch.object(acceptance.pages, "ITEM_HASHES", hashes).start()
        patch.object(acceptance, "engine_ledger").start()
        original_snapshot = acceptance.snapshot
        def snapshot(before, after=None, required=()):
            if self.output / "libprovider.rlib" in required:
                original_snapshot(before, after, required)
        self.snapshot_checks = patch.object(acceptance, "snapshot", side_effect=snapshot).start()
        self.contracts = {method: dict(requires="safe domain of " + method, ensures="exact result of " + method)
                          for method in acceptance.PAGE_CONTRACTS}
        patch.object(acceptance, "PAGE_CONTRACTS", {method: {key: acceptance.full.text_digest(value)
            for key, value in contract.items()} for method, contract in self.contracts.items()}).start()
        extraction = acceptance.pages.extract(self.application, self.output)
        (self.output / "inputs/source").mkdir(parents=True)
        (self.output / "inputs/source/engine.rs").write_text("synthetic source")
        self.receipt = dict(success=True, commands=[], cases=[], extraction=extraction,
            artifacts={}, source_hashes={"source/engine.rs": digest(self.output / "inputs/source/engine.rs")},
            fixture_hashes={path.name: digest(path) for path in self.output.glob("*.rs")},
            transfer=dict(scenario="redb_page_relations", application_commit=acceptance.pages.APPLICATION,
                source_fidelity=True, status="proved", stale_export_rejected=True, trust=["ordinary backend"],
                limitations=["not an allocator theorem"],
                positive_cases=["relations_cold", "relations_warm", "relations_poisoned_cache", "zero_ordinary",
                    "witnesses_checked", "provider_cold", "client_cold", "client_warm"],
                semantic_negative_cases=[name + "_negative" for name in acceptance.pages.NEGATIVES],
                safety_negative_cases=[name + "_negative" for name in acceptance.pages.SAFETY],
                polynomial_auxiliary_negative_cases=["false_polynomial_negative"],
                metadata_negative_cases=["client_metadata_" + fault for fault in ("policy", "exactness", "summaries", "trusted_model")]))
        proof = dict(soundness=True, maximality=True, exactness=True)
        certificate = dict(schema=INTERFACE_SCHEMA, policy=INTERFACE_POLICY, **proof,
            proofs=[[dict(path=dict(krate=dict(Id=["provider", 17]), segments=["impl&%0", method])), proof]
                    for method in ("page_size_bytes", "address_range")],
            summaries="f" * 64, trusted_models=[[{}, "f" * 64]])
        self.blob = b"synthetic\x00" + json.dumps(certificate, separators=(",", ":")).encode() + b"VRINFER1"
        self.imported = [dict(crate_id=17, fingerprint=acceptance.metadata(self.blob)[1])]
        for field in ("positive_cases", "semantic_negative_cases", "safety_negative_cases",
                      "polynomial_auxiliary_negative_cases", "metadata_negative_cases"):
            for label in self.receipt["transfer"][field]:
                self.add_case(label, field)
        (self.output / "relations-cache").mkdir()
        (self.output / "relations-cache/poisoned.json").write_text(json.dumps(dict(version="verus-infer-v70", pre=True, post=False)))
        for fault in ("policy", "exactness", "summaries", "trusted_model"):
            def edit(value):
                if fault == "policy": value["policy"] = "candidate-bound-evidence-v0"
                elif fault == "exactness": value["exactness"] = False
                elif fault == "summaries": value["summaries"] = "0" * 64
                else: value["trusted_models"][0][1] = "0" * 64
            (self.output / ("provider_poison_" + fault + ".vir")).write_bytes(acceptance.full.replace_metadata(self.blob, edit))
        (self.output / "libprovider.rlib").write_text("synthetic native provider artifact")
        self.guard_provider()
        for label in ("application_before_pin", "application_before_clean", "engine_files",
                      "application_after_pin", "application_after_clean"):
            if label == "engine_files":
                arguments = ["git", "ls-files", "--cached", "--others", "--exclude-standard", "--",
                             "source", "dependencies", "rust-toolchain.toml"]
                stdout = "\n".join(self.receipt["source_hashes"]) + "\n"
            else:
                pin = label.endswith("_pin")
                arguments = ["git", "-C", str(self.application), *(
                    ["rev-parse", "HEAD"] if pin else ["status", "--porcelain", "--untracked-files=all"])]
                stdout = acceptance.pages.APPLICATION + "\n" if pin else ""
            self.receipt["commands"].append(dict(label=label, argv=arguments, exit_code=0, cwd=str(transfers.ROOT)))
            for suffix in ("stdout", "stderr"):
                (self.output / (label + "." + suffix)).write_text(stdout if suffix == "stdout" else "")

    def guard_provider(self):
        guard = ArtifactGuard([self.output / "provider_cold.vir", self.output / "libprovider.rlib"])
        self.receipt.update(provider_artifacts_before=guard.initial, provider_artifacts_after=guard.snapshot())

    def add_case(self, label, field):
        name = "client" if label.startswith("client_") else "relations" if label == "relations_poisoned_cache" else label.rsplit("_", 1)[0]
        artifact, source = self.output / (label + ".vir"), self.output / (name + ".rs")
        positive, metadata = field == "positive_cases", field == "metadata_negative_cases"
        report = proof_report(positive)
        if name == "client":
            methods = ["client::relay"]
        else:
            methods = [name + "::PageNumber::" + method for method in ("page_size_bytes", "address_range")]
        functions = []
        for method in methods:
            function = copy.deepcopy(report["verus-infer"]["functions"][0])
            function["function"] = method
            function.update(contract=self.contracts[method.split("::", 1)[1]],
                exported=positive and name in ("provider", "client"),
                available_to_same_crate_callers=True, metrics=dict(cache="miss", cache_policy="regenerated_state"))
            function["candidate_id"] = acceptance.full.text_digest(method)
            function["dependencies"] = [dict(function="provider::impl&%0::address_range",
                candidate_id=acceptance.full.text_digest("provider::PageNumber::address_range"), mode="exact")] if name == "client" else []
            functions.append(function)
        report["verus-infer"]["functions"] = [] if metadata else functions
        bodies = {"strict_start", "injective_start", "ordered_nonoverlap", "nonoverlap_all_sizes"}
        verified = 24
        if name == "client":
            bodies |= {"relay"}
            verified = 21
        elif name == "zero":
            bodies, verified = {"zero_size"}, 7
        elif name == "witnesses":
            bodies, verified = {"zero_witness", "mixed_witness"}, 22
        elif name == "provider":
            bodies, verified = set(), 6
        elif name in acceptance.pages.NEGATIVES:
            bodies, verified = {"zero_witness", "mixed_witness", "wrong"}, 22
        elif name in acceptance.pages.SAFETY:
            bodies, verified = {"rejected"}, 6
        elif name == "false_polynomial":
            bodies, verified = {"wrong"}, 9
        if name != "client":
            bodies |= {"PageNumber::address_range", "PageNumber::page_size_bytes"}
        report["verification-results"]["verified"] = verified
        report["verus-infer"].update(requested_functions=len(functions), generated_functions=len(functions),
            generation_failure=None, imported_inference_dependencies=self.imported if name == "client" else [],
            failed_functions=[] if positive or metadata else [name + "::" + ("rejected" if name in acceptance.pages.SAFETY else "wrong")],
            verification_scope=dict(native_collection_completed=True, complete_selected_scope_verified=positive,
                checked_exec_bodies=len(bodies) if positive else 0, checked_proof_bodies=0,
                native_functions=[dict(rust_function=body, selected=True, automatically_derived=False,
                    body_status="checked_exec_body" if positive else "selected_body_not_certified_by_successful_full_run")
                    for body in sorted(bodies)]))
        classification = {"positive_cases": "proved", "semantic_negative_cases": "semantic_goal_failure",
            "safety_negative_cases": "caller_obligation_failure", "polynomial_auxiliary_negative_cases": "auxiliary_theorem_failure",
            "metadata_negative_cases": "frontend_rejection"}[field]
        diagnostic = {"positive_cases": "", "semantic_negative_cases": "postcondition not satisfied",
            "safety_negative_cases": "precondition not satisfied",
            "polynomial_auxiliary_negative_cases": "Verus Infer: polynomial proposal not satisfied"}.get(field, "")
        if metadata:
            report["verification-results"].update(**{"encountered-error": True, "encountered-vir-error": True})
            fault = label.removeprefix("client_metadata_")
            diagnostic = "Verus Infer: " + acceptance.IMPORT_DIAGNOSTICS[{"exactness": "proof", "trusted_model": "trusted_models"}.get(fault, fault)]
        if positive:
            artifact.write_bytes(self.blob)
        argv = [str(self.verifier), str(source), "--crate-type", "lib", "--crate-name", name, "--no-cheating",
            "--output-json", "--compile", "--export", str(artifact), "-o", str(self.output / ("lib" + name + ".rlib")),
            "-C", "debug-assertions=yes"]
        if name == "client":
            interface = self.output / ("provider_poison_" + label.removeprefix("client_metadata_") + ".vir") if metadata else self.output / "provider_cold.vir"
            argv += ["--extern", "provider=" + str(self.output / "libprovider.rlib"), "--import", "provider=" + str(interface)]
        if label != "zero_ordinary" and field != "safety_negative_cases":
            argv.append("--infer-goals")
        self.receipt["commands"].append(dict(label=label, argv=argv, cwd=str(self.output), exit_code=0 if positive else 1,
                                             environment=dict(VERUS_Z3_PATH=str(self.root / "z3"))))
        (self.output / (label + ".stdout")).write_text(json.dumps(report))
        (self.output / (label + ".stderr")).write_text(diagnostic)
        self.receipt["cases"].append(dict(label=label, classification=classification, report=report, artifact=str(artifact),
                                          artifact_exists=positive, stale_export_seeded=not positive))

    def check(self):
        path = self.output / "receipt.json"
        path.write_text(json.dumps(self.receipt))
        return transfers.check_receipt(path, "redb_page_relations", self.verifier)

    def mutate(self, label, edit):
        row = next(row for row in self.receipt["cases"] if row["label"] == label)
        stdout = self.output / (label + ".stdout")
        report = json.loads(stdout.read_text())
        edit(report)
        stdout.write_text(json.dumps(report))
        row["report"] = report

    def test_counts_polynomial_separately_from_semantic_safety_and_metadata(self):
        summary = self.check()
        self.assertEqual(len(summary["proofs"]), 23)
        self.assertEqual(summary["polynomial_auxiliary_count"], 1)
        self.assertEqual(summary["semantic_negative_count"], 3)
        self.assertEqual(summary["safety_negative_count"], 11)
        self.assertEqual(len(summary["metadata_controls"]), 4)

    def test_every_control_group_is_required_and_distinct(self):
        original = copy.deepcopy(self.receipt)
        for field in ("positive_cases", "semantic_negative_cases", "safety_negative_cases",
                      "polynomial_auxiliary_negative_cases", "metadata_negative_cases"):
            self.receipt = copy.deepcopy(original)
            self.receipt["transfer"][field].pop()
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.check()
        self.receipt = copy.deepcopy(original)
        self.receipt["cases"].append(copy.deepcopy(self.receipt["cases"][-1]))
        with self.assertRaises(ValueError):
            self.check()

    def test_frontend_actual_exit_seeded_absence_and_exact_corruption_are_checked(self):
        entry = acceptance.command(self.receipt, "client_metadata_policy")
        entry["exit_code"] = 0
        with self.assertRaises(ValueError):
            self.check()
        entry["exit_code"] = 1
        (self.output / "client_metadata_policy.vir").write_bytes(self.blob)
        with self.assertRaises(ValueError):
            self.check()
        (self.output / "client_metadata_policy.vir").unlink()
        (self.output / "provider_poison_policy.vir").write_bytes(self.blob)
        with self.assertRaises(ValueError):
            self.check()

    def test_source_and_debug_guard_cannot_be_changed_by_claiming_fidelity(self):
        entry = acceptance.command(self.receipt, "relations_cold")
        position = entry["argv"].index("debug-assertions=yes")
        entry["argv"][position] = "debug-assertions=no"
        with self.assertRaises(ValueError):
            self.check()
        entry["argv"][position] = "debug-assertions=yes"
        (self.output / "relations.rs").write_text("assume(false)")
        with self.assertRaises(ValueError):
            self.check()

    def test_every_proof_rejects_narrowed_domains_wrong_equations_and_counts(self):
        for row in self.receipt["cases"]:
            if row["label"].startswith("client_metadata_"):
                continue
            label, original = row["label"], copy.deepcopy(row["report"])
            edits = [lambda data: data["verification-results"].update(verified=1),
                     lambda data: data["verification-results"].update(errors=True),
                     lambda data: data["verus-infer"].update(requested_functions=True),
                     lambda data: data["verus-infer"].update(generated_functions=0),
                     lambda data: data["verus-infer"].update(generation_failure="unsupported"),
                     lambda data: data["verus-infer"].update(failed_functions=["unrelated::body"]),
                     lambda data: data["verus-infer"].update(status="no_inference")]
            if row["classification"] != "proved":
                edits.append(lambda data: data["verification-results"].update(errors=2))
            for index in range(len(original["verus-infer"]["functions"])):
                for key, value in (("requires", "false"), ("ensures", "true")):
                    edits.append(lambda data, index=index, key=key, value=value:
                        data["verus-infer"]["functions"][index]["contract"].update({key: value}))
                edits.extend([
                    lambda data, index=index: data["verus-infer"]["functions"][index].update(exported=1),
                    lambda data, index=index: data["verus-infer"]["functions"][index].update(available_to_same_crate_callers=False),
                    lambda data, index=index: data["verus-infer"]["functions"][index]["metrics"].update(cache="hit"),
                ])
            for edit in edits:
                with self.subTest(label=label, edit=edit):
                    self.mutate(label, edit)
                    with self.assertRaises(ValueError):
                        self.check()
                    self.mutate(label, lambda report: report.update(copy.deepcopy(original)))

    def test_every_proof_requires_complete_genuine_native_body_scope(self):
        edits = [lambda data: data["verus-infer"].pop("verification_scope"),
                 lambda data: data["verus-infer"]["verification_scope"].update(native_collection_completed=False),
                 lambda data: data["verus-infer"]["verification_scope"].update(complete_selected_scope_verified=1),
                 lambda data: data["verus-infer"]["verification_scope"].update(checked_exec_bodies=True),
                 lambda data: data["verus-infer"]["verification_scope"].update(checked_proof_bodies=True),
                 lambda data: data["verus-infer"]["verification_scope"].update(checked_exec_bodies=999),
                 lambda data: data["verus-infer"]["verification_scope"].update(checked_proof_bodies=1),
                 lambda data: data["verus-infer"]["verification_scope"]["native_functions"].pop(),
                 lambda data: data["verus-infer"]["verification_scope"]["native_functions"].append(
                     copy.deepcopy(data["verus-infer"]["verification_scope"]["native_functions"][0]))]
        for key, value in (("body_status", "external_body"), ("selected", False),
                           ("automatically_derived", True), ("rust_function", "fake_body")):
            edits.append(lambda data, key=key, value=value:
                data["verus-infer"]["verification_scope"]["native_functions"][0].update({key: value}))
        for row in self.receipt["cases"]:
            if row["label"].startswith("client_metadata_"):
                continue
            label, original = row["label"], copy.deepcopy(row["report"])
            for edit in edits:
                with self.subTest(label=label, edit=edit):
                    self.mutate(label, edit)
                    with self.assertRaises(ValueError):
                        self.check()
                    self.mutate(label, lambda report: report.update(copy.deepcopy(original)))

    def test_integer_count_errors_fail_at_the_exact_count_check(self):
        controls = [
            ("false_strict_zero_negative", lambda data: data["verification-results"].update(errors=2),
             "unexpected relational-page proof counts"),
            ("relations_cold", lambda data: data["verus-infer"]["verification_scope"].update(checked_exec_bodies=7),
             "unexpected relational checked-body counts"),
            ("relations_cold", lambda data: data["verus-infer"]["verification_scope"].update(checked_proof_bodies=1),
             "unexpected relational checked-body counts"),
        ]
        for label, edit, diagnostic in controls:
            original = copy.deepcopy(next(row["report"] for row in self.receipt["cases"] if row["label"] == label))
            self.mutate(label, edit)
            with self.subTest(label=label, edit=edit), self.assertRaisesRegex(ValueError, diagnostic):
                self.check()
            self.mutate(label, lambda report: report.update(copy.deepcopy(original)))
        self.check()

    def test_provider_pair_must_be_guarded_before_and_after_client_proofs(self):
        original = copy.deepcopy(self.receipt)
        for field in ("provider_artifacts_before", "provider_artifacts_after"):
            for name in ("libprovider.rlib", "provider_cold.vir"):
                with self.subTest(field=field, name=name):
                    self.receipt[field].pop(str(self.output / name))
                    with self.assertRaises(ValueError):
                        self.check()
                    self.receipt = copy.deepcopy(original)
        for name in ("libprovider.rlib", "provider_cold.vir"):
            artifact = self.output / name
            contents = artifact.read_bytes()
            artifact.write_bytes(b"tampered artifact")
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, "guarded input drift"):
                self.check()
            artifact.write_bytes(contents)
            self.guard_provider()
        self.check()

    def test_clients_bind_imports_to_the_guarded_provider_certificate(self):
        for label in ("client_cold", "client_warm"):
            original = copy.deepcopy(next(row["report"] for row in self.receipt["cases"] if row["label"] == label))
            for imports in ([], [dict(crate_id=18, fingerprint=self.imported[0]["fingerprint"])],
                            [dict(crate_id=17, fingerprint="0" * 64)], self.imported * 2):
                self.mutate(label, lambda data: data["verus-infer"].update(imported_inference_dependencies=imports))
                with self.subTest(label=label, imports=imports), self.assertRaisesRegex(ValueError, "unbound relational imported equations"):
                    self.check()
                self.mutate(label, lambda report: report.update(copy.deepcopy(original)))

    def test_provider_guard_rejects_extra_paths(self):
        extra = self.output / "unrelated.rlib"
        extra.write_text("not a provider artifact")
        guard = ArtifactGuard([self.output / "provider_cold.vir", self.output / "libprovider.rlib", extra])
        self.receipt.update(provider_artifacts_before=guard.initial, provider_artifacts_after=guard.snapshot())
        with self.assertRaisesRegex(ValueError, "unexpected relational provider guard scope"):
            self.check()

    def test_relay_dependency_matches_the_provider_helper(self):
        edits = [lambda value: value.update(candidate_id="0" * 64),
                 lambda value: value.update(candidate_id=acceptance.full.text_digest("provider::PageNumber::page_size_bytes")),
                 lambda value: value.update(function="provider::impl&%0::page_size_bytes"),
                 lambda value: value.update(mode="bounded")]
        for label in ("client_cold", "client_warm"):
            original = copy.deepcopy(next(row["report"] for row in self.receipt["cases"] if row["label"] == label))
            for edit in edits:
                self.mutate(label, lambda data: edit(data["verus-infer"]["functions"][0]["dependencies"][0]))
                with self.subTest(label=label, edit=edit), self.assertRaisesRegex(ValueError, "unbound relational relay dependency"):
                    self.check()
                self.mutate(label, lambda report: report.update(copy.deepcopy(original)))
            for dependencies in ([], None, original["verus-infer"]["functions"][0]["dependencies"] * 2):
                self.mutate(label, lambda data: data["verus-infer"]["functions"][0].update(dependencies=dependencies))
                with self.subTest(label=label, dependencies=dependencies), self.assertRaisesRegex(ValueError, "unbound relational relay dependency"):
                    self.check()
                self.mutate(label, lambda report: report.update(copy.deepcopy(original)))
        self.check()

    def test_provider_certificate_requires_both_exact_helper_proofs(self):
        edits = [(lambda value: value.update(schema=True), "relational provider certificate is not fully checked"),
                 (lambda value: value.update(schema=37), "relational provider certificate is not fully checked"),
                 (lambda value: value.update(policy="wrong"), "relational provider certificate is not fully checked"),
                 (lambda value: value.update(exactness=False), "relational provider certificate is not fully checked"),
                 (lambda value: value["proofs"].pop(), "relational provider certificate is not fully checked"),
                 (lambda value: value["proofs"][0][1].update(maximality=False), "relational provider omitted exact helper checks"),
                 (lambda value: value["proofs"][0][0]["path"].update(segments=["fake"]), "relational provider omitted exact helper checks"),
                 (lambda value: value["proofs"][0][0]["path"]["krate"].update(Id=["other", 18]), "ambiguous relational provider crate identity")]
        for edit, diagnostic in edits:
            blob = acceptance.full.replace_metadata(self.blob, edit)
            (self.output / "provider_cold.vir").write_bytes(blob)
            self.guard_provider()
            for label in ("client_cold", "client_warm"):
                imported = [dict(crate_id=17, fingerprint=acceptance.metadata(blob)[1])]
                self.mutate(label, lambda data: data["verus-infer"].update(imported_inference_dependencies=imported))
            for fault in ("policy", "exactness", "summaries", "trusted_model"):
                def corrupt(value):
                    if fault == "policy": value["policy"] = "candidate-bound-evidence-v0"
                    elif fault == "exactness": value["exactness"] = False
                    elif fault == "summaries": value["summaries"] = "0" * 64
                    else: value["trusted_models"][0][1] = "0" * 64
                (self.output / ("provider_poison_" + fault + ".vir")).write_bytes(acceptance.full.replace_metadata(blob, corrupt))
            with self.subTest(edit=edit), self.assertRaisesRegex(ValueError, diagnostic):
                self.check()
        (self.output / "provider_cold.vir").write_bytes(self.blob)
        self.guard_provider()

    def test_every_prerequisite_must_exist_and_succeed(self):
        original = copy.deepcopy(self.receipt)
        labels = {row["label"] for row in self.receipt["cases"]}
        for entry in self.receipt["commands"]:
            if entry["label"] in labels:
                continue
            for exit_code in (17, True, -1, None):
                with self.subTest(label=entry["label"], exit_code=exit_code):
                    entry["exit_code"] = exit_code
                    with self.assertRaises(ValueError):
                        self.check()
            entry["exit_code"] = 0
        for entry in original["commands"]:
            if entry["label"] in labels:
                continue
            self.receipt["commands"] = [row for row in original["commands"] if row["label"] != entry["label"]]
            with self.subTest(missing=entry["label"]), self.assertRaises(ValueError):
                self.check()
        self.receipt = original

    def test_application_prerequisites_pin_commands_and_outputs(self):
        for phase in ("before", "after"):
            for suffix in ("pin", "clean"):
                label = "application_" + phase + "_" + suffix
                entry = acceptance.command(self.receipt, label)
                original = copy.deepcopy(entry)
                for change in (dict(argv=["true"]), dict(argv=["git", "-C", "/another/application", *entry["argv"][3:]]),
                               dict(cwd="/another/engine")):
                    entry.update(change)
                    with self.subTest(label=label, change=change), self.assertRaisesRegex(ValueError, "relational application inspection"):
                        self.check()
                    entry.update(copy.deepcopy(original))
                stdout = self.output / (label + ".stdout")
                text = stdout.read_text()
                stdout.write_text("0" * 40 + "\n" if suffix == "pin" else " M file.rs\n?? extra.rs\n")
                with self.subTest(label=label, output=True), self.assertRaisesRegex(ValueError, "relational application inspection"):
                    self.check()
                stdout.write_text(text)
        self.check()

    def test_engine_enumeration_matches_the_full_source_ledger(self):
        entry = acceptance.command(self.receipt, "engine_files")
        original = copy.deepcopy(entry)
        for change in (dict(argv=["true"]), dict(cwd="/another/engine")):
            entry.update(change)
            with self.subTest(change=change), self.assertRaisesRegex(ValueError, "relational engine source enumeration"):
                self.check()
            entry.update(copy.deepcopy(original))
        stdout = self.output / "engine_files.stdout"
        text = stdout.read_text()
        for changed in ("", "source/unrelated.rs\n", text * 2, text + "source/extra.rs\n"):
            stdout.write_text(changed)
            with self.subTest(output=changed), self.assertRaisesRegex(ValueError, "relational engine source enumeration"):
                self.check()
        stdout.write_text(text)
        self.check()


if __name__ == "__main__":
    unittest.main()
