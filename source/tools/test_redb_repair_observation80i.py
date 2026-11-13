"""Targeted fail-closed checks against an actual fresh cycle80i receipt."""

import copy
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import check_redb_repair_observation80i as experiment


class ReceiptGuards(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        receipt_path = os.environ.get("REDB_OBSERVATION80I_RECEIPT")
        if receipt_path is None:
            raise unittest.SkipTest("Set REDB_OBSERVATION80I_RECEIPT to a fresh replay receipt")
        cls.receipt = experiment.read(Path(receipt_path))
        cls.row = next(row for row in cls.receipt["cases"] if row["category"] == "positive")
        cls.scope = experiment.read(experiment.EVIDENCE / "scope.json")

    def positive(self, report, code=0):
        experiment.validate_positive(report, code, Path(self.row["source"]), self.scope)

    def report_reject(self, guard, mutate):
        report = copy.deepcopy(self.row["report"])
        mutate(report)
        with self.assertRaisesRegex(ValueError, "^" + guard + "$"):
            self.positive(report)

    def binding_reject(self, guard, mutate):
        row = copy.deepcopy(self.row)
        mutate(row)
        with self.assertRaisesRegex(ValueError, "^" + guard + "$"):
            experiment.validate_bindings(row)

    def test_genuine_receipt(self):
        self.assertIs(self.receipt["success"], True)
        self.assertEqual(len(self.receipt["cases"]), 28)
        for row in self.receipt["cases"]:
            experiment.validate_bindings(row)
            if row["category"] == "positive":
                experiment.validate_positive(row["report"], row["returncode"], Path(row["source"]), self.scope)
            else:
                experiment.validate_negative(row)
        experiment.validate_matrix(self.receipt, Path(self.receipt["application"]))

    def test_exit_status(self):
        for code in (True, False, 1, "0"):
            with self.subTest(code=code), self.assertRaisesRegex(ValueError, "^exit_status$"):
                self.positive(self.row["report"], code)

    def test_success_flags(self):
        for key, value in (("success", 1), ("encountered-error", 0),
                ("encountered-vir-error", 0), ("is-verifying-entire-crate", 1)):
            with self.subTest(key=key):
                self.report_reject("success_flags", lambda report: report["verification-results"].update({key: value}))

    def test_proof_counts(self):
        for key, value in (("verified", True), ("verified", 14), ("errors", False), ("errors", 1)):
            with self.subTest(key=key, value=value):
                self.report_reject("proof_counts", lambda report: report["verification-results"].update({key: value}))

    def test_publication_state(self):
        for key, value in (("crate_success", 1), ("status", "verified"), ("interface_written", 0)):
            with self.subTest(key=key):
                self.report_reject("publication_state", lambda report: report["verus-infer"].update({key: value}))

    def test_report_schema(self):
        self.report_reject("report_schema", lambda report: report["verus-infer"].update(schema_version=True))

    def test_inference_scope(self):
        for key, value in (("requested_functions", False), ("generated_functions", 1),
                ("functions", [{}]), ("imported_inference_dependencies", [{}])):
            with self.subTest(key=key):
                self.report_reject("inference_scope", lambda report: report["verus-infer"].update({key: value}))

    def test_scope_completion(self):
        for key in ("native_collection_completed", "complete_selected_scope_verified"):
            with self.subTest(key=key):
                self.report_reject("scope_completion", lambda report: report["verus-infer"]["verification_scope"].update({key: 1}))

    def test_body_counts(self):
        for key, value in (("checked_exec_bodies", 10), ("checked_proof_bodies", True)):
            with self.subTest(key=key):
                self.report_reject("body_counts", lambda report: report["verus-infer"]["verification_scope"].update({key: value}))

    def test_scope_identities(self):
        for mutation in ("drop", "add", "rename", "status"):
            def mutate(report):
                rows = report["verus-infer"]["verification_scope"]["native_functions"]
                if mutation == "drop":
                    rows.pop()
                elif mutation == "add":
                    rows.append(copy.deepcopy(rows[-1]))
                elif mutation == "rename":
                    rows[-1]["function"] += "_fake"
                else:
                    rows[-1]["body_status"] = "outside_selected_scope"
            with self.subTest(mutation=mutation):
                self.report_reject("scope_identities", mutate)

    def test_scope_types(self):
        def mutate(report):
            report["verus-infer"]["verification_scope"]["native_functions"][0]["selected"] = 0
        self.report_reject("scope_types", mutate)

    def test_scope_sources(self):
        def mutate(report):
            report["verus-infer"]["verification_scope"]["native_functions"][-1]["source"] = "another.rs:1:1"
        self.report_reject("scope_sources", mutate)

    def test_file_hashes(self):
        for key in ("source", "stdout", "stderr"):
            with self.subTest(key=key):
                self.binding_reject(key + "_hash", lambda row: row.update({key + "_sha256": "0" * 64}))

    def test_report_binding(self):
        self.binding_reject("report_binding", lambda row: row["report"]["verification-results"].update(verified=999))

    def test_argv_source(self):
        self.binding_reject("argv_source", lambda row: row["command"].__setitem__(1, "/other.rs"))

    def test_import_options_separated_joined_and_crosswired(self):
        for arguments in (("--import", "p=a.vir"), ("--extern", "p=a.rlib"),
                ("--import=p=a.vir",), ("--extern=p=a.rlib",),
                ("--import", "p=a.rlib", "--extern", "p=a.vir"),
                ("--import=p=a.rlib", "--extern=p=a.vir")):
            with self.subTest(arguments=arguments):
                self.binding_reject("argv_imports", lambda row: row["command"].extend(arguments))

    def test_export_options(self):
        for arguments in (("--export", "a.vir"), ("--export=a.vir",)):
            with self.subTest(arguments=arguments):
                self.binding_reject("argv_export", lambda row: row["command"].extend(arguments))

    def test_scope_bypasses(self):
        for flag in ("--no-verify", "--verify-only-module=x", "--no-lifetime", "--no-auto-recommends-check"):
            with self.subTest(flag=flag):
                self.binding_reject("argv_scope", lambda row: row["command"].append(flag))

    def test_flag_cardinality(self):
        for flag in ("--no-cheating", "--compile", "--output-json", "--num-threads", "--crate-name", "--sysroot", "-o", "-C"):
            for duplicate in (False, True):
                def mutate(row):
                    if duplicate:
                        row["command"].append(flag)
                    else:
                        row["command"].remove(flag)
                with self.subTest(flag=flag, duplicate=duplicate):
                    self.binding_reject("argv_flags", mutate)

    def test_argument_bindings(self):
        for flag, guard, value in (("-o", "argv_library", "other.rlib"),
                ("--crate-name", "argv_crate", "other"), ("--num-threads", "argv_threads", "2"),
                ("-C", "argv_mode", "debug-assertions=wrong")):
            def mutate(row):
                row["command"][row["command"].index(flag) + 1] = value
            with self.subTest(flag=flag):
                self.binding_reject(guard, mutate)

    def test_default_erasure(self):
        for key in ("no_cheating", "default_erasure"):
            with self.subTest(key=key):
                self.binding_reject("erasure_flags", lambda row: row.update({key: 1}))

    def test_exact_argv(self):
        self.binding_reject("argv_exact", lambda row: row["command"].append("--triggers-mode=silent"))

    def test_library_binding(self):
        self.binding_reject("library_binding", lambda row: row.update(library_sha256="0" * 64))

    def test_negative_exit(self):
        row = copy.deepcopy(next(row for row in self.receipt["cases"] if row["category"] == "proof"))
        row["returncode"] = False
        with self.assertRaisesRegex(ValueError, "^negative_exit$"):
            experiment.validate_negative(row)

    def test_negative_success(self):
        row = copy.deepcopy(next(row for row in self.receipt["cases"] if row["category"] == "proof"))
        row["report"]["verification-results"]["success"] = 0
        with self.assertRaisesRegex(ValueError, "^negative_success$"):
            experiment.validate_negative(row)

    def test_negative_publication(self):
        row = copy.deepcopy(next(row for row in self.receipt["cases"] if row["category"] == "proof"))
        row["report"]["verus-infer"]["interface_written"] = 0
        with self.assertRaisesRegex(ValueError, "^negative_publication$"):
            experiment.validate_negative(row)

    def test_negative_diagnostic(self):
        row = copy.deepcopy(next(row for row in self.receipt["cases"] if row["category"] == "proof"))
        row["stderr"] = self.row["stderr"]
        with self.assertRaisesRegex(ValueError, "^negative_diagnostic$"):
            experiment.validate_negative(row)

    def test_negative_proof(self):
        row = copy.deepcopy(next(row for row in self.receipt["cases"] if row["category"] == "proof"))
        row["report"]["verification-results"]["errors"] = True
        with self.assertRaisesRegex(ValueError, "^negative_proof$"):
            experiment.validate_negative(row)

    def test_negative_additional_guards(self):
        source = next(row for row in self.receipt["cases"] if row["category"] == "proof")
        mutations = {
            "negative_classification": lambda row: row.update(returncode=2),
            "negative_counts": lambda row: row["report"]["verification-results"].update(verified=999),
            "negative_schema": lambda row: row["report"]["verus-infer"].update(schema_version=True),
            "negative_scope": lambda row: row["report"]["verus-infer"]["verification_scope"].update(checked_exec_bodies=True),
            "negative_library": lambda row: row.update(library=self.row["library"], library_sha256=self.row["library_sha256"]),
        }
        for guard, mutate in mutations.items():
            row = copy.deepcopy(source)
            mutate(row)
            with self.subTest(guard=guard), self.assertRaisesRegex(ValueError, "^" + guard + "$"):
                experiment.validate_negative(row)

    def test_endian_boundary_diagnostic(self):
        row = copy.deepcopy(next(row for row in self.receipt["cases"] if row["category"] == "endian"))
        row["stderr"] = next(row["stderr"] for row in self.receipt["cases"] if row["category"] == "frontend")
        with self.assertRaisesRegex(ValueError, "^endian_boundary$"):
            experiment.validate_negative(row)

    def test_matrix_guards(self):
        mutations = {
            "matrix_success": lambda receipt: receipt.update(success=1),
            "matrix_metadata": lambda receipt: receipt.update(durability_proved=True),
            "matrix_labels": lambda receipt: receipt["cases"].pop(),
            "matrix_commands": lambda receipt: receipt["commands"][0]["argv"].append("--extra"),
            "matrix_classification": lambda receipt: receipt["cases"][0].update(category="proof"),
            "matrix_source": lambda receipt: receipt["cases"][0].update(source_sha256="0" * 64),
        }
        for guard, mutate in mutations.items():
            receipt = copy.deepcopy(self.receipt)
            mutate(receipt)
            with self.subTest(guard=guard), self.assertRaisesRegex(ValueError, "^" + guard + "$"):
                experiment.validate_matrix(receipt, Path(self.receipt["application"]))

    def test_positive_library_required(self):
        with tempfile.TemporaryDirectory(prefix="redb-observation80i-missing-") as directory:
            receipt = copy.deepcopy(self.receipt)
            row = next(row for row in receipt["cases"] if row["label"] == "positive_no")
            row["library"] = str(Path(directory) / "missing.rlib")
            row["library_sha256"] = None
            row["command"][row["command"].index("-o") + 1] = row["library"]
            command = next(command for command in receipt["commands"] if command["label"] == row["label"])
            command["argv"] = list(row["command"])
            with self.assertRaisesRegex(ValueError, "^positive_library$"):
                experiment.validate_matrix(receipt, Path(receipt["application"]))

    def test_native_metadata(self):
        for key, value in (("tests", True), ("tests", 0), ("gate_inputs", False), ("gate_inputs", 0)):
            receipt = copy.deepcopy(self.receipt)
            receipt["native_controls"][key] = value
            with self.subTest(key=key, value=value), self.assertRaisesRegex(ValueError, "^native_metadata$"):
                experiment.validate_native_controls(receipt, Path(receipt["application"]))

    def test_native_source_binding(self):
        receipt = copy.deepcopy(self.receipt)
        receipt["native_controls"]["source_sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "^native_source_binding$"):
            experiment.validate_native_controls(receipt, Path(receipt["application"]))

    def test_native_binary_binding(self):
        receipt = copy.deepcopy(self.receipt)
        receipt["native_controls"]["binary_sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "^native_binary_binding$"):
            experiment.validate_native_controls(receipt, Path(receipt["application"]))

    def test_native_command_binding(self):
        for offset in (-2, -1):
            receipt = copy.deepcopy(self.receipt)
            receipt["commands"][offset]["argv"].append("--unrecorded")
            with self.subTest(offset=offset), self.assertRaisesRegex(ValueError, "^native_argv$"):
                experiment.validate_native_controls(receipt, Path(receipt["application"]))
        receipt = copy.deepcopy(self.receipt)
        receipt["commands"][-1]["cwd"] += "/wrong"
        with self.assertRaisesRegex(ValueError, "^native_argv$"):
            experiment.validate_native_controls(receipt, Path(receipt["application"]))

    def test_native_exit_status(self):
        for offset in (-2, -1):
            for value in (True, False, 1, "0"):
                receipt = copy.deepcopy(self.receipt)
                receipt["commands"][offset]["exit_code"] = value
                with self.subTest(offset=offset, value=value), self.assertRaisesRegex(ValueError, "^native_exit_status$"):
                    experiment.validate_native_controls(receipt, Path(receipt["application"]))

    def test_native_log_fields(self):
        receipt = copy.deepcopy(self.receipt)
        receipt["native_controls"]["logs"]["unrecorded.stdout"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "^native_log_fields$"):
            experiment.validate_native_controls(receipt, Path(receipt["application"]))

    def test_native_log_binding(self):
        for name in self.receipt["native_controls"]["logs"]:
            receipt = copy.deepcopy(self.receipt)
            receipt["native_controls"]["logs"][name] = "0" * 64
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, "^native_log_binding$"):
                experiment.validate_native_controls(receipt, Path(receipt["application"]))

    def test_native_result_counts(self):
        for text in ("3 passed; 0 failed", "test result: ok. 0 passed; 0 failed; 3 ignored;",
                "test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out;"):
            with self.subTest(text=text), self.assertRaisesRegex(ValueError, "^native_test_counts$"):
                experiment.validate_native_counts(text)

    def test_matrix_checks_native_controls(self):
        receipt = copy.deepcopy(self.receipt)
        receipt["native_controls"]["tests"] = 0
        with self.assertRaisesRegex(ValueError, "^native_metadata$"):
            experiment.validate_matrix(receipt, Path(receipt["application"]))

    def test_ledger_guards(self):
        with tempfile.TemporaryDirectory(prefix="redb-observation80i-guard-") as name:
            root = Path(name)
            (root / "data").write_text("real input")
            (root / "ledger.json").write_text(json.dumps({"data": experiment.digest(root / "data")}))
            actual = experiment.digest(root / "ledger.json")
            for guard, expected, count in (("ledger_identity", "0" * 64, 1),
                    ("ledger_count", actual, 2), ("ledger_contents", actual, 1)):
                if guard == "ledger_contents":
                    (root / "data").write_text("changed input")
                with self.subTest(guard=guard), mock.patch.dict(experiment.LEDGERS, {"ledger.json": (expected, count)}):
                    with self.assertRaisesRegex(ValueError, "^" + guard + "$"):
                        experiment.audit_ledger(root, "ledger.json", root)

    def test_native_source_guards(self):
        application = Path(self.receipt["application"])
        source = experiment.render(application)
        mutations = {
            "native_gate_body": source.replace("if self.primary_corrupted {", "if !self.primary_corrupted {"),
            "native_match_arms": source.replace("Err(err) => Err(err),", "Err(err) => Ok(false),"),
            "record_fidelity": source.replace("pub primary_slot: usize,", "pub primary_slot: u64,"),
            "full_error": source.replace("#[non_exhaustive]", ""),
        }
        for guard, contents in mutations.items():
            with self.subTest(guard=guard), self.assertRaisesRegex(ValueError, "^" + guard + "$"):
                experiment.source_fidelity(application, contents)
        for guard, values in (("native_header", ["0" * 64]), ("native_db", [experiment.HEADER_HASH, "0" * 64])):
            with self.subTest(guard=guard), mock.patch.object(experiment, "digest", side_effect=values):
                with self.assertRaisesRegex(ValueError, "^" + guard + "$"):
                    experiment.extract(application)


if __name__ == "__main__":
    unittest.main()
