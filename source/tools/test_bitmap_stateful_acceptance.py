"""Synthetic receipt consistency tests; no verifier execution is claimed here."""

import ast
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import types
import unittest
from unittest.mock import patch

import bitmap_stateful_acceptance as acceptance


def report_fixture():
    body_rows = [dict(function="fixture::" + name, selected=True,
                      body_status="checked_exec_body", automatically_derived=False,
                      inference_candidate=name == "helper", ordinary_contract=name != "helper")
                 for name in ("helper", "resize", "client")]
    helper = dict(function="fixture::helper", mode="exact", required_checks_passed=True,
                  available_to_same_crate_callers=True, exported=False,
                  contract=dict(requires="true", ensures="result == input / 64"),
                  metrics=dict(solver_micros=100, cache="miss"),
                  proofs={name: dict(required=True, status="proved")
                          for name in ("soundness", "maximality", "exactness")})
    document = {"verification-results": dict(success=True, verified=7, errors=0,
                **{"encountered-error": False, "encountered-vir-error": False,
                   "is-verifying-entire-crate": True}),
                "verus-infer": dict(crate_success=True, interface_written=True,
                    requested_functions=1, generated_functions=1,
                    generation_failure=None, failed_functions=[], functions=[helper],
                    imported_inference_dependencies=[], trusted_models=[],
                    verification_scope=dict(native_collection_completed=True,
                        complete_selected_scope_verified=True, native_functions=body_rows,
                        checked_exec_bodies=3, checked_proof_bodies=0))}
    expected = dict(verified=7, interface_written=True,
                    bodies={row["function"]: copy.deepcopy(row) for row in body_rows},
                    helpers={"fixture::helper": dict(contract=copy.deepcopy(helper["contract"]),
                                                    exported=False, cache="miss")},
                    dependencies={}, trusted_models=[])
    return document, expected


def bundle_fixture(root):
    document, expected_report = report_fixture()
    contents = {"input": b"fn synthetic_unit_fixture() {}\n",
                "stdout": json.dumps(document).encode(), "stderr": b"",
                "library": b"synthetic library, not executable proof evidence",
                "interface": b"synthetic interface, not a certificate"}
    records = {}
    for name, payload in contents.items():
        path = root / name
        path.write_bytes(payload)
        records[name] = dict(path=str(path), sha256=hashlib.sha256(payload).hexdigest())
    arguments = ["/fixture/rustup", "run", "/fixture/sysroot", "/fixture/verus",
                 records["input"]["path"], "--sysroot", "/fixture/sysroot",
                 "--compile", "--no-cheating", "--output-json", "--crate-type=lib",
                 "--num-threads", "1", "-o", records["library"]["path"],
                 "--export", records["interface"]["path"]]
    run = dict(argv=arguments, returncode=0, artifacts=records)
    expected = dict(argv=copy.deepcopy(arguments), artifacts=copy.deepcopy(records),
                    report=expected_report)
    return run, expected


def store_report(run, expected, document):
    payload = json.dumps(document).encode()
    Path(run["artifacts"]["stdout"]["path"]).write_bytes(payload)
    digest = hashlib.sha256(payload).hexdigest()
    run["artifacts"]["stdout"]["sha256"] = digest
    expected["artifacts"]["stdout"]["sha256"] = digest


def report_mutations():
    return {
        "report.success": lambda result, infer: result.update(success=False),
        "report.no-errors": lambda result, infer: result.update(**{"encountered-error": True}),
        "report.no-frontend-errors": lambda result, infer: result.update(**{"encountered-vir-error": True}),
        "report.entire-crate": lambda result, infer: result.update(**{"is-verifying-entire-crate": False}),
        "report.error-count": lambda result, infer: result.update(errors=1),
        "report.verified-count": lambda result, infer: result.update(verified=1),
        "infer.crate-success": lambda result, infer: infer.update(crate_success=False),
        "infer.publication": lambda result, infer: infer.update(interface_written=False),
        "infer.generation-failure": lambda result, infer: infer.update(generation_failure="failed"),
        "infer.failed-functions": lambda result, infer: infer.update(failed_functions=["fixture::resize"]),
        "infer.helper-set": lambda result, infer: (
            infer["functions"].append(dict(infer["functions"][0], function="other::helper")),
            infer.update(requested_functions=2, generated_functions=2)),
        "infer.requested-count": lambda result, infer: infer.update(requested_functions=2),
        "infer.generated-count": lambda result, infer: infer.update(generated_functions=0),
        "helper.exact-mode": lambda result, infer: infer["functions"][0].update(mode="sound"),
        "helper.required-checks": lambda result, infer: infer["functions"][0].update(required_checks_passed=False),
        "helper.available": lambda result, infer: infer["functions"][0].update(available_to_same_crate_callers=False),
        "helper.export-status": lambda result, infer: infer["functions"][0].update(exported=True),
        "helper.contract": lambda result, infer: infer["functions"][0]["contract"].update(requires="false"),
        "helper.proof-soundness": lambda result, infer: infer["functions"][0]["proofs"]["soundness"].update(required=False),
        "helper.proof-maximality": lambda result, infer: infer["functions"][0]["proofs"]["maximality"].update(status="skipped"),
        "helper.proof-exactness": lambda result, infer: infer["functions"][0]["proofs"]["exactness"].update(status="failed"),
        "helper.fresh-work": lambda result, infer: infer["functions"][0]["metrics"].update(solver_micros=0),
        "helper.cache-state": lambda result, infer: infer["functions"][0]["metrics"].update(cache="hit"),
        "scope.collection-complete": lambda result, infer: infer["verification_scope"].update(native_collection_completed=False),
        "scope.verification-complete": lambda result, infer: infer["verification_scope"].update(complete_selected_scope_verified=False),
        "scope.body-set": lambda result, infer: infer["verification_scope"]["native_functions"].append(
            dict(infer["verification_scope"]["native_functions"][0],
                 function="other::unchecked", body_status="excluded", selected=False)),
        "scope.body-status": lambda result, infer: (
            infer["verification_scope"]["native_functions"][1].update(body_status="checked_proof_body"),
            infer["verification_scope"].update(checked_exec_bodies=2, checked_proof_bodies=1)),
        "scope.body-selected": lambda result, infer: infer["verification_scope"]["native_functions"][1].update(selected=False),
        "scope.body-inference": lambda result, infer: infer["verification_scope"]["native_functions"][1].update(inference_candidate=True),
        "scope.body-contract": lambda result, infer: infer["verification_scope"]["native_functions"][1].update(ordinary_contract=False),
        "scope.body-derived": lambda result, infer: infer["verification_scope"]["native_functions"][1].update(automatically_derived=True),
        "scope.exec-count": lambda result, infer: infer["verification_scope"].update(checked_exec_bodies=100),
        "scope.proof-count": lambda result, infer: infer["verification_scope"].update(checked_proof_bodies=1),
        "import.binding": lambda result, infer: infer.update(imported_inference_dependencies=[dict(crate_id=2, fingerprint="a" * 64)]),
        "infer.trusted-models": lambda result, infer: infer.update(trusted_models=["unlisted"]),
    }


class StatefulAcceptanceTests(unittest.TestCase):
    def test_complete_synthetic_positive_runs_through_composed_gate(self):
        with tempfile.TemporaryDirectory() as directory:
            run, expected = bundle_fixture(Path(directory))
            self.assertEqual(acceptance.validate_positive_run(run, expected), report_fixture()[0])

    def test_report_guards_reject_through_composed_gate_at_intended_checks(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name, mutate in report_mutations().items():
                with self.subTest(guard=name):
                    run, expected = bundle_fixture(root)
                    document, unused = report_fixture()
                    mutate(document["verification-results"], document["verus-infer"])
                    store_report(run, expected, document)
                    with self.assertRaisesRegex(acceptance.ReceiptError, "^" + name + "$"):
                        acceptance.validate_positive_run(run, expected)

    def test_bool_and_noninteger_counters_do_not_pass_integer_checks(self):
        document, expected = report_fixture()
        fields = ((document["verification-results"], "errors", "report.error-count"),
                  (document["verification-results"], "verified", "report.verified-count"),
                  (document["verus-infer"], "requested_functions", "infer.requested-count"),
                  (document["verus-infer"], "generated_functions", "infer.generated-count"),
                  (document["verus-infer"]["verification_scope"], "checked_exec_bodies", "scope.exec-count"),
                  (document["verus-infer"]["verification_scope"], "checked_proof_bodies", "scope.proof-count"))
        for owner, field, name in fields:
            original = owner[field]
            for value in (True, False, "0", 0.0, None, [], {}):
                with self.subTest(field=field, value=value):
                    owner[field] = value
                    with self.assertRaisesRegex(acceptance.ReceiptError, "^" + name + "$"):
                        acceptance.validate_positive_report(document, expected)
            owner[field] = original

    def test_scope_requires_complete_unique_exact_body_set(self):
        for change in ("missing", "duplicate", "additional", "malformed"):
            document, expected = report_fixture()
            bodies = document["verus-infer"]["verification_scope"]["native_functions"]
            if change == "missing":
                bodies.pop()
            elif change == "duplicate":
                bodies.append(copy.deepcopy(bodies[0]))
            elif change == "additional":
                bodies.append(dict(bodies[0], function="fixture::extra"))
            else:
                bodies[0] = None
            with self.subTest(change=change):
                with self.assertRaises(acceptance.ReceiptError):
                    acceptance.validate_positive_report(document, expected)

    def test_all_boolean_proof_and_body_fields_require_real_booleans(self):
        document, expected = report_fixture()
        inference = document["verus-infer"]
        entries = [(document["verification-results"], field) for field in
                   ("success", "encountered-error", "encountered-vir-error", "is-verifying-entire-crate")]
        entries += [(inference, field) for field in ("crate_success", "interface_written")]
        entries += [(inference["functions"][0], field) for field in
                    ("required_checks_passed", "available_to_same_crate_callers", "exported")]
        entries += [(inference["verification_scope"], field) for field in
                    ("native_collection_completed", "complete_selected_scope_verified")]
        entries += [(inference["verification_scope"]["native_functions"][0], field) for field in
                    ("selected", "inference_candidate", "ordinary_contract", "automatically_derived")]
        entries += [(inference["functions"][0]["proofs"][field], "required") for field in
                    ("soundness", "maximality", "exactness")]
        for owner, field in entries:
            original = owner[field]
            for value in (int(original), str(original), None):
                with self.subTest(field=field, value=value):
                    owner[field] = value
                    with self.assertRaises(acceptance.ReceiptError):
                        acceptance.validate_positive_report(document, expected)
            owner[field] = original

    def test_every_helper_keeps_fresh_work_even_when_cache_hits(self):
        document, expected = report_fixture()
        helper = document["verus-infer"]["functions"][0]
        helper["metrics"]["cache"] = "hit"
        expected["helpers"][helper["function"]]["cache"] = "hit"
        acceptance.validate_positive_report(document, expected)
        for work in (0, -1, True, "100", None):
            helper["metrics"]["solver_micros"] = work
            with self.subTest(work=work):
                with self.assertRaisesRegex(acceptance.ReceiptError, "^helper.fresh-work$"):
                    acceptance.validate_positive_report(document, expected)

    def test_import_fingerprint_identity_and_uniqueness(self):
        document, expected = report_fixture()
        inference = document["verus-infer"]
        fingerprint = "a" * 64
        expected["dependencies"] = {17: fingerprint}
        inference["imported_inference_dependencies"] = [dict(crate_id=17, fingerprint=fingerprint)]
        acceptance.validate_positive_report(document, expected)
        for dependencies in ([], [dict(crate_id=17, fingerprint="b" * 64)],
                             [dict(crate_id=True, fingerprint=fingerprint)],
                             [dict(crate_id=17, fingerprint=fingerprint)] * 2):
            inference["imported_inference_dependencies"] = dependencies
            with self.subTest(dependencies=dependencies):
                with self.assertRaises(acceptance.ReceiptError):
                    acceptance.validate_positive_report(document, expected)

    def test_native_artifacts_must_exist_be_nonempty_and_match_pinned_hash(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ("input", "library", "interface"):
                for mutation in ("missing", "empty", "junk", "symlink"):
                    run, expected = bundle_fixture(root)
                    path = Path(run["artifacts"][name]["path"])
                    if mutation == "missing":
                        path.unlink()
                    elif mutation == "empty":
                        path.write_bytes(b"")
                    elif mutation == "junk":
                        path.write_bytes(b"replaced")
                    else:
                        target = root / (name + "-target")
                        target.write_bytes(path.read_bytes())
                        path.unlink()
                        path.symlink_to(target)
                    with self.subTest(artifact=name, mutation=mutation):
                        with self.assertRaises(acceptance.ReceiptError):
                            acceptance.validate_positive_run(run, expected)
                    if path.is_symlink():
                        path.unlink()

    def test_failed_native_command_cannot_be_reported_as_positive(self):
        with tempfile.TemporaryDirectory() as directory:
            for status in (1, 17, True, False, 0.0, "0", None):
                run, expected = bundle_fixture(Path(directory))
                run["returncode"] = status
                with self.subTest(status=status):
                    with self.assertRaisesRegex(acceptance.ReceiptError, "^run.exit-status$"):
                        acceptance.validate_positive_run(run, expected)

    def test_argv_is_pinned_including_crosswires_joined_and_extra_flags(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for change in ("true", "swap", "joined", "no-checks", "extra-import"):
                run, expected = bundle_fixture(root)
                if change == "true":
                    run["argv"] = ["true"]
                elif change == "swap":
                    arguments = run["argv"]
                    output = arguments.index("-o") + 1
                    interface = arguments.index("--export") + 1
                    arguments[output], arguments[interface] = arguments[interface], arguments[output]
                elif change == "joined":
                    run["argv"].append("--import=other=junk.vir")
                elif change == "no-checks":
                    run["argv"].remove("--no-cheating")
                else:
                    run["argv"].extend(["--extern", "other=junk.rlib"])
                with self.subTest(change=change):
                    with self.assertRaisesRegex(acceptance.ReceiptError, "^command.binding$"):
                        acceptance.validate_positive_run(run, expected)

    def test_stdout_and_stderr_are_actual_hash_bound_artifacts(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ("stdout", "stderr"):
                run, expected = bundle_fixture(root)
                Path(run["artifacts"][name]["path"]).write_bytes(b"modified")
                with self.subTest(artifact=name):
                    with self.assertRaisesRegex(acceptance.ReceiptError, "^artifact.hash:" + name + "$"):
                        acceptance.validate_positive_run(run, expected)

    def test_retained_snapshots_keep_actual_command_output_paths_separate(self):
        with tempfile.TemporaryDirectory() as directory:
            run, expected = bundle_fixture(Path(directory))
            for name in ("library", "interface"):
                original = run["artifacts"][name]["path"]
                snapshot = Path(directory) / (name + "-snapshot")
                Path(original).rename(snapshot)
                for record in (run["artifacts"][name], expected["artifacts"][name]):
                    record.update(path=str(snapshot), produced_path=original)
            acceptance.validate_positive_run(run, expected)
            expected["artifacts"]["interface"]["produced_path"] = run["artifacts"]["library"]["produced_path"]
            run["artifacts"]["interface"]["produced_path"] = expected["artifacts"]["interface"]["produced_path"]
            with self.assertRaisesRegex(acceptance.ReceiptError, "^command.option:--export$"):
                acceptance.validate_positive_run(run, expected)

    def test_invalid_report_json_is_a_named_rejection_not_a_setup_error(self):
        with tempfile.TemporaryDirectory() as directory:
            run, expected = bundle_fixture(Path(directory))
            payload = b"{invalid"
            Path(run["artifacts"]["stdout"]["path"]).write_bytes(payload)
            for record in (run["artifacts"]["stdout"], expected["artifacts"]["stdout"]):
                record["sha256"] = hashlib.sha256(payload).hexdigest()
            with self.assertRaisesRegex(acceptance.ReceiptError, "^report.json$"):
                acceptance.validate_positive_run(run, expected)

    def test_isolated_guard_deletions_are_detected_without_incidental_errors(self):
        source = Path(acceptance.__file__).read_text()
        for name, mutate in report_mutations().items():
            with self.subTest(guard=name):
                tree = ast.parse(source)
                changed = 0
                for node in ast.walk(tree):
                    if not isinstance(node, ast.Call) or not isinstance(node.func, ast.Name):
                        continue
                    if (node.func.id == "require" and len(node.args) == 2
                            and ((isinstance(node.args[1], ast.Constant) and node.args[1].value == name)
                                 or (name.startswith("helper.proof-")
                                     and ast.unparse(node.args[1]) == "'helper.proof-' + obligation"))):
                        node.args[0] = ast.Constant(value=True)
                        changed += 1
                    if (node.func.id == "integer" and len(node.args) >= 2
                            and isinstance(node.args[1], ast.Constant) and node.args[1].value == name):
                        node.args[0] = ast.Constant(value=1)
                        changed += 1
                self.assertEqual(changed, 1, name)
                ast.fix_missing_locations(tree)
                mutant = types.ModuleType("bitmap_acceptance_mutant")
                exec(compile(tree, acceptance.__file__, "exec"), mutant.__dict__)
                document, expected = report_fixture()
                mutate(document["verification-results"], document["verus-infer"])
                mutant.validate_positive_report(document, expected)
                with self.assertRaisesRegex(acceptance.ReceiptError, "^" + name + "$"):
                    acceptance.validate_positive_report(document, expected)
                test = StatefulAcceptanceTests(
                    "test_report_guards_reject_through_composed_gate_at_intended_checks")
                result = unittest.TestResult()
                with patch.dict(globals(), acceptance=mutant):
                    test.run(result)
                expected_failures = 3 if name.startswith("helper.proof-") else 1
                self.assertEqual(len(result.failures), expected_failures, name)
                self.assertEqual(result.errors, [], name)


if __name__ == "__main__":
    unittest.main()
