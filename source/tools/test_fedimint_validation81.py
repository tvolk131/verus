"""Exercise the integrated acceptance path, including isolated removable guards."""

import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_fedimint_validation81 as fee


def fixture(directory, crate="fee_provider"):
    selected = json.loads((Path(__file__).parent / "infer_fee_validation81/native-items.json").read_text())
    functions = [dict(function="fee_provider::" + name, mode="exact",
        required_checks_passed=True, exported=not name.endswith("::absolute_fee"),
        contract=dict(requires="true"), metrics=dict(cache="miss", cache_policy="candidate_cache", solver_micros=1),
        proofs={check: dict(status="proved", required=True)
            for check in ("soundness", "maximality", "exactness")}) for name in sorted(fee.EXPECTED_INFERRED)] if crate == "fee_provider" else []
    native = [dict(function=name, **fields) for name, fields in sorted(fee.expected_native_scope(crate).items())]
    bodies, verified = (31, 55) if crate == "fee_provider" else (4, 4)
    data = {"verification-results": dict(success=True, verified=verified, errors=0,
        **{"is-verifying-entire-crate": True, "encountered-error":False, "encountered-vir-error":False}),
        "verus-infer": dict(schema_version=1, crate_success=True, interface_written=True, functions=functions,
            imported_inference_dependencies=[], verification_scope=dict(native_collection_completed=True,
                complete_selected_scope_verified=True, checked_exec_bodies=bodies, checked_proof_bodies=0,
                native_functions=native))}
    artifact = directory / "provider.vir"
    artifact.write_bytes(b"fresh")
    return dict(data=data, artifact=artifact, returncode=0, category="positive", stderr="",
        phase="cold", count=len(functions), bodies=bodies, verified=verified, crate=crate, selected=selected,
        source=fee.provider_source(selected) if functions else fee.client_source(), dependency=None)


class Acceptance(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.directory = Path(self.temporary.name)

    def tearDown(self):
        self.temporary.cleanup()

    def isolated(self, guard, change, diagnostic=None, crate="fee_provider"):
        outcome = fixture(self.directory, crate)
        change(outcome)
        with self.assertRaisesRegex(ValueError, diagnostic or ".*"):
            fee.validate_outcome(outcome)
        with patch.object(fee, guard, lambda outcome: None):
            fee.validate_outcome(outcome)

    def test_untampered_accepts(self):
        fee.validate_outcome(fixture(self.directory))
        fee.validate_outcome(fixture(self.directory, "fee_client"))

    def test_complete_scope_has_all_native_and_authored_bodies(self):
        for crate, total, checked in (("fee_provider", 52, 31), ("fee_client", 4, 4)):
            scope = fixture(self.directory, crate)["data"]["verus-infer"]["verification_scope"]
            self.assertEqual(len(scope["native_functions"]), total)
            self.assertEqual(sum(entry["body_status"] == "checked_exec_body" for entry in scope["native_functions"]), checked)

    def test_missing_duplicate_unexpected_and_empty_body_lists_reject(self):
        def change(outcome, fault):
            scope = outcome["data"]["verus-infer"]["verification_scope"]
            entries = scope["native_functions"]
            if fault == "empty":
                scope["native_functions"] = []
            elif fault == "missing":
                entries[:] = [entry for entry in entries if entry["function"] != outcome["crate"] + "::PaymentFee::checked_add"]
            elif fault == "duplicate":
                entries.append(dict(entries[0]))
            else:
                entries[0]["function"] = outcome["crate"] + "::unexpected"
        for crate, faults in (("fee_provider", ("missing", "duplicate", "unexpected", "empty")),
                ("fee_client", ("duplicate", "unexpected", "empty"))):
            for fault in faults:
                with self.subTest(crate=crate, fault=fault):
                    self.isolated("guard_scope", lambda outcome: change(outcome, fault), "^native_body_set_guard$", crate)

    def test_native_helper_and_consumer_body_classifications_are_pinned(self):
        def change(outcome, field, value):
            suffix = "PaymentFee::checked_add" if outcome["crate"] == "fee_provider" else "imported_amount"
            entry = next(entry for entry in outcome["data"]["verus-infer"]["verification_scope"]["native_functions"]
                if entry["function"].endswith("::" + suffix))
            entry[field] = value
        for crate in ("fee_provider", "fee_client"):
            for field, value in (("selected", False), ("selected", 1), ("body_status", "outside_selected_scope"),
                    ("automatically_derived", True), ("inference_candidate", 1), ("ordinary_contract", 1)):
                with self.subTest(crate=crate, field=field, value=value):
                    self.isolated("guard_scope", lambda outcome: change(outcome, field, value),
                        "^native_body_metadata_guard$", crate)

    def test_scope_totals_are_recomputed_from_the_complete_entries(self):
        for field, value in (("checked_exec_bodies", 30), ("checked_proof_bodies", 1)):
            self.isolated("guard_scope", lambda outcome: outcome["data"]["verus-infer"]["verification_scope"].update({field: value}),
                "^native_scope_count_guard$")

    def test_positive_error_states_are_strictly_false(self):
        for field in ("encountered-error", "encountered-vir-error"):
            for value in (True, 0, 1, None, "false", "missing"):
                def change(outcome):
                    result = outcome["data"]["verification-results"]
                    if value == "missing":
                        result.pop(field)
                    else:
                        result[field] = value
                with self.subTest(field=field, value=value):
                    self.isolated("guard_classification", change, "^positive_error_state_guard$")

    def test_positive_report_schema_is_exact(self):
        for value in (True, False, 999, "1", None):
            self.isolated("guard_classification", lambda outcome: outcome["data"]["verus-infer"].update(schema_version=value),
                "^positive_report_schema_guard$")

    def test_positive_report_cannot_declare_both_verifier_errors(self):
        self.isolated("guard_classification", lambda outcome: outcome["data"]["verification-results"].update(
            {"encountered-error": True, "encountered-vir-error": True}), "^positive_error_state_guard$")

    def test_native_body_guard_is_independently_necessary(self):
        self.isolated("guard_native", lambda outcome: outcome.update(source=outcome["source"] + "\n"))

    def test_native_item_ledger_is_not_caller_supplied_truth(self):
        def change(outcome):
            outcome["selected"]["Amount::checked_add"] += "\n"
            outcome["source"] = fee.provider_source(outcome["selected"])
        self.isolated("guard_native", change)

    def test_full_domain_guard_is_independently_necessary(self):
        def change(outcome):
            target = next(function for function in outcome["data"]["verus-infer"]["functions"]
                if function["function"].endswith("Amount::checked_add"))
            target["contract"]["requires"] = "self.msats==0"
        self.isolated("guard_domains", change)

    def test_required_check_guard_is_independently_necessary(self):
        for check in ("soundness", "maximality", "exactness"):
            for fault in ("missing", "failed", "not_required"):
                def change(outcome):
                    proofs = outcome["data"]["verus-infer"]["functions"][0]["proofs"]
                    if fault == "missing":
                        proofs.pop(check)
                    elif fault == "failed":
                        proofs[check]["status"] = "incomplete"
                    else:
                        proofs[check]["required"] = False
                with self.subTest(check=check, fault=fault):
                    self.isolated("guard_required", change)

    def test_scope_and_actual_derive_guard_are_independently_necessary(self):
        for field, value in (("checked_exec_bodies", 30), ("complete_selected_scope_verified", False)):
            self.isolated("guard_scope", lambda outcome: outcome["data"]["verus-infer"]["verification_scope"].update({field:value}))
        self.isolated("guard_scope", lambda outcome: outcome["data"]["verus-infer"]["verification_scope"]["native_functions"][0].update(body_status="assumed"))

    def test_fresh_work_and_cache_phase_are_independently_necessary(self):
        self.isolated("guard_work", lambda outcome: outcome["data"]["verus-infer"]["functions"][0]["metrics"].update(solver_micros=0))
        self.isolated("guard_work", lambda outcome: outcome["data"]["verus-infer"]["functions"][0]["metrics"].update(cache="hit"))

    def test_publication_guard_is_independently_necessary(self):
        self.isolated("guard_publication", lambda outcome: outcome["artifact"].unlink())
        self.isolated("guard_publication", lambda outcome: outcome["data"]["verus-infer"].update(interface_written=False))

    def test_import_guard_is_independently_necessary(self):
        self.isolated("guard_import", lambda outcome: outcome["data"]["verus-infer"]["imported_inference_dependencies"].append(dict(fingerprint="0" * 64)))

    def test_import_fingerprint_and_native_artifact_are_bound(self):
        for fault in ("fingerprint", "native", "policy"):
            with self.subTest(fault=fault):
                outcome = fixture(self.directory)
                provider = self.directory / "dependency"
                provider.mkdir(exist_ok=True)
                cert = dict(schema=38, policy=fee.POLICY, proofs=[], summaries="0" * 64)
                blob = json.dumps(cert).encode() + b"VRINFER1"
                (provider / "provider.vir").write_bytes(blob)
                (provider / "libprovider.rlib").write_bytes(b"native")
                _, fingerprint = fee.metadata(blob)
                outcome["dependency"] = dict(folder=provider, fingerprint=fingerprint,
                    native_sha256=fee.sha(b"native"))
                outcome["data"]["verus-infer"]["imported_inference_dependencies"] = [dict(fingerprint=fingerprint)]
                fee.validate_outcome(outcome)
                if fault == "fingerprint":
                    outcome["data"]["verus-infer"]["imported_inference_dependencies"][0]["fingerprint"] = "0" * 64
                elif fault == "native":
                    (provider / "libprovider.rlib").write_bytes(b"changed-native")
                else:
                    cert["policy"] = "stale-policy"
                    (provider / "provider.vir").write_bytes(json.dumps(cert).encode() + b"VRINFER1")
                with self.assertRaises(ValueError):
                    fee.validate_outcome(outcome)
                with patch.object(fee, "guard_import", lambda outcome: None):
                    fee.validate_outcome(outcome)

    def test_classification_guard_is_independently_necessary(self):
        outcome = fixture(self.directory)
        outcome.update(category="semantic", returncode=1, stderr="precondition not satisfied")
        outcome["artifact"].unlink()
        outcome["data"]["verification-results"].update(success=False, errors=1)
        inference = outcome["data"]["verus-infer"]
        inference.update(crate_success=False, interface_written=False, status="caller_proof_failed")
        for function in inference["functions"]:
            function["exported"] = False
        with self.assertRaises(ValueError):
            fee.validate_outcome(outcome)
        with patch.object(fee, "guard_classification", lambda outcome: None):
            fee.validate_outcome(outcome)

    def test_private_absolute_fee_cannot_be_exported(self):
        def change(outcome):
            next(function for function in outcome["data"]["verus-infer"]["functions"]
                if function["function"].endswith("::absolute_fee"))["exported"] = True
        self.isolated("guard_publication", change)


if __name__ == "__main__":
    unittest.main()
