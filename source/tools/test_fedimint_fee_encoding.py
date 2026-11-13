"""Fail-closed extraction and classification tests, without compiler subprocesses."""

import json
import copy
from contextlib import ExitStack
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch

from infer_transfer_identity import CACHE_VERSION, ENGINE_IDENTITY, INTERFACE_POLICY, INTERFACE_SCHEMA
import check_fedimint_fee_encoding as fee


FIXTURE = Path(__file__).resolve().parents[2] / "source/tools/fixtures/transfers/transfer-fedimint-fee-encoding/helpers.source.rs"


class FeeEncodingTests(unittest.TestCase):
    def positive_result(self):
        return {
            "verification-results": {"success": True, "verified": 11, "errors": 0,
                "is-verifying-entire-crate": True},
            "verus-infer": {"crate_success": True, "interface_written": True,
                "verification_scope": {"native_collection_completed": True,
                    "complete_selected_scope_verified": True,
                    "checked_exec_bodies": 7, "checked_proof_bodies": 0},
                "functions": [{
                "mode": "exact", "required_checks_passed": True, "exported": True,
                "metrics": {"cache": "miss", "solver_micros": 1000},
                "contract": {"requires": "true"}, "proofs": {name: {"status": "proved", "required": True}
                    for name in ("soundness", "maximality", "exactness")}}
                    for _index in range(2)]}}

    def negative_result(self):
        data = self.positive_result()
        data["verification-results"].update(success=False, verified=6, errors=1)
        data["verus-infer"].update(crate_success=False, interface_written=False,
            status="caller_proof_failed")
        for function in data["verus-infer"]["functions"]:
            function["exported"] = False
        return data

    def private_positive_result(self):
        data = self.positive_result()
        data["verification-results"]["verified"] = 3
        scope = data["verus-infer"]["verification_scope"]
        scope["checked_exec_bodies"] = 1
        helper = "fee_payment_fee::PaymentFee::absolute_fee"
        scope["native_functions"] = [dict(function=helper, body_status="checked_exec_body",
            selected=True, automatically_derived=False, inference_candidate=True, ordinary_contract=False)]
        data["verus-infer"]["functions"] = data["verus-infer"]["functions"][:1]
        data["verus-infer"]["functions"][0].update(function=helper, exported=False,
            contract=dict(requires="(1000000 != 0)"))
        return data

    def invocation(self, data, diagnostic="", returncode=None):
        return dict(returncode=int(not data["verification-results"].get("success"))
            if returncode is None else returncode, stdout=json.dumps(data), stderr=diagnostic)

    def test_complete_native_items_and_domain_are_unchanged(self):
        helpers = fee.extract_helpers(FIXTURE.read_text())
        proof = fee.proof_source(helpers)
        self.assertEqual(proof.count("#[verifier::infer]"), 2)
        for marker, expected in fee.ITEM_HASHES.items():
            self.assertEqual(fee.text_digest(fee.item(proof, marker)), expected)
        for forbidden in ("requires", "assume", "external_body", "axiom", "sha256"):
            self.assertNotIn(forbidden, proof)

    def test_changed_body_type_or_visibility_fails_extraction(self):
        source = FIXTURE.read_text()
        for changed in (source.replace("u64::MAX - fee_msats", "fee_msats"),
                        source.replace("fee_msats: u64", "fee_msats: u32"),
                        source.replace("pub fn fee_from", "fn fee_from")):
            with self.assertRaises(ValueError):
                fee.extract_helpers(changed)

    def test_clean_pin_and_complete_input_guards(self):
        fee.check_pin(fee.APPLICATION, "", fee.SOURCE_HASHES)
        for head, status, hashes in (("0" * 40, "", fee.SOURCE_HASHES),
                (fee.APPLICATION, "?? unrelated.rs", fee.SOURCE_HASHES),
                (fee.APPLICATION, "", {**fee.SOURCE_HASHES, fee.CONTRACTS: "0" * 64}),
                (fee.APPLICATION, "", {key: value for key, value in fee.SOURCE_HASHES.items()
                    if key != fee.RECEIVE})):
            with self.assertRaises(ValueError):
                fee.check_pin(head, status, hashes)

    def test_mutant_is_exactly_one_total_decoder_change(self):
        helpers = fee.extract_helpers(FIXTURE.read_text())
        changed = fee.mutant(helpers)
        encoder = "pub fn fee_encoded_expiration("
        self.assertEqual(fee.item(changed, encoder), fee.item(helpers, encoder))
        self.assertEqual(changed, helpers.replace("u64::MAX - expiration",
            "u64::MAX.saturating_sub(expiration).saturating_sub(1)", 1))
        with self.assertRaises(ValueError):
            fee.mutant(changed)

    def test_authored_goals_include_real_temporal_boundary_not_unconditional_future(self):
        self.assertEqual(fee.GOALS.count("pub fn "), 5)
        self.assertIn("(fee as int) + (now as int) < u64::MAX", fee.GOALS)
        self.assertIn("encoded.1 == 0", fee.GOALS)
        self.assertIn("(left <= right) == (encoded.0 >= encoded.1)", fee.GOALS)
        self.assertIn("(left == right) == (encoded.0 == encoded.1)", fee.GOALS)
        self.assertNotIn("requires", fee.GOALS)
        self.assertNotIn("ensures expiration > now", fee.GOALS)

    def test_before_and_after_only_add_existing_infer_attributes(self):
        helpers = fee.extract_helpers(FIXTURE.read_text())
        before = fee.proof_source(helpers, fee.ROUNDTRIP, inferred=False)
        after = fee.proof_source(helpers, fee.ROUNDTRIP)
        self.assertEqual(after.replace("#[verifier::infer]\n", ""), before)

    def test_frontend_failure_never_counts_as_semantic_counterexample(self):
        run = dict(returncode=1, stderr="unsupported frontend", stdout=json.dumps({
            "verification-results": {"encountered-error": True, "encountered-vir-error": True},
            "verus-infer": {"crate_success": False, "interface_written": False}}))
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "absent.vir"
            with self.assertRaises(ValueError):
                fee.validate_case(run, "semantic", artifact, 0)
            fee.validate_case(run, "frontend", artifact, None)

    def test_marked_frontend_error_is_not_an_exact_semantic_failure(self):
        data = self.negative_result()
        data["verification-results"]["encountered-vir-error"] = True
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, "frontend failure is not a semantic"):
                fee.validate_case(self.invocation(data, "postcondition not satisfied"),
                    "semantic", Path(directory) / "absent.vir", 2)

    def test_required_checks_domain_and_scope_cannot_be_removed(self):
        baseline = self.positive_result()
        variants = []
        for obligation in ("soundness", "maximality", "exactness"):
            for field, value in (("status", "incomplete"), ("required", False)):
                changed = copy.deepcopy(baseline)
                changed["verus-infer"]["functions"][0]["proofs"][obligation][field] = value
                variants.append(changed)
        for key, value in (("required_checks_passed", False), ("mode", "approximate"), ("exported", False)):
            changed = copy.deepcopy(baseline)
            changed["verus-infer"]["functions"][0][key] = value
            variants.append(changed)
        changed = copy.deepcopy(baseline)
        changed["verus-infer"]["functions"][0]["contract"]["requires"] = "fee < 100"
        variants.append(changed)
        changed = copy.deepcopy(baseline)
        changed["verification-results"]["is-verifying-entire-crate"] = False
        variants.append(changed)
        for key in ("native_collection_completed", "complete_selected_scope_verified"):
            changed = copy.deepcopy(baseline)
            changed["verus-infer"]["verification_scope"][key] = False
            variants.append(changed)
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "positive.vir"
            artifact.write_bytes(b"seed")
            fee.validate_case(dict(returncode=0, stdout=json.dumps(baseline), stderr=""),
                "positive", artifact, 2)
            for data in variants:
                with self.subTest(data=data), self.assertRaises(ValueError):
                    fee.validate_case(dict(returncode=0, stdout=json.dumps(data), stderr=""),
                        "positive", artifact, 2)

    def test_inference_function_count_is_exact(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "positive.vir"
            artifact.write_bytes(b"seed")
            for count in (0, 1, 3):
                data = self.positive_result()
                data["verus-infer"]["functions"] = [data["verus-infer"]["functions"][0]] * count
                with self.subTest(count=count), self.assertRaisesRegex(ValueError, "wrong local inference scope"):
                    fee.validate_case(self.invocation(data), "positive", artifact, 2)

    def test_positive_proof_and_body_counts_are_exact(self):
        baseline = self.positive_result()
        for group, key, values in (("verification-results", "verified", (1, 10, True)),
                ("verification-results", "errors", (1, False)),
                ("verification_scope", "checked_exec_bodies", (1, 6, True)),
                ("verification_scope", "checked_proof_bodies", (1, False))):
            for value in values:
                data = copy.deepcopy(baseline)
                target = data[group] if group == "verification-results" else data["verus-infer"][group]
                target[key] = value
                with tempfile.TemporaryDirectory() as directory:
                    artifact = Path(directory) / "positive.vir"
                    artifact.write_bytes(b"seed")
                    with self.subTest(key=key, value=value), self.assertRaisesRegex(ValueError, "counts changed"):
                        fee.validate_case(self.invocation(data), "positive", artifact, 2)

    def test_before_and_semantic_failures_require_one_postcondition_error(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "absent.vir"
            for category in ("before", "semantic"):
                baseline = self.negative_result()
                fee.validate_case(self.invocation(baseline, "postcondition not satisfied"),
                    category, artifact, 2)
                for errors in (0, 2):
                    changed = copy.deepcopy(baseline)
                    changed["verification-results"]["errors"] = errors
                    with self.subTest(category=category, errors=errors), self.assertRaises(ValueError):
                        fee.validate_case(self.invocation(changed, "postcondition not satisfied"),
                            category, artifact, 2)
                for diagnostic in ("", "precondition not satisfied",
                        "postcondition not satisfied; precondition not satisfied"):
                    with self.subTest(category=category, diagnostic=diagnostic), self.assertRaises(ValueError):
                        fee.validate_case(self.invocation(baseline, diagnostic), category, artifact, 2)

    def test_safety_failure_and_requested_diagnostic_are_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "absent.vir"
            baseline = self.negative_result()
            for errors in (0, 2):
                changed = copy.deepcopy(baseline)
                changed["verification-results"]["errors"] = errors
                with self.subTest(errors=errors), self.assertRaisesRegex(ValueError, "exactly one obligation"):
                    fee.validate_case(self.invocation(changed, "overflow"), "safety", artifact, 2)
            with self.assertRaisesRegex(ValueError, "missing expected diagnostic"):
                fee.validate_case(self.invocation(baseline, "different error"), "safety", artifact, 2,
                    marker="possible arithmetic underflow/overflow")

    def test_stale_export_cannot_survive_frontend_failure(self):
        data = {"verification-results": {"encountered-vir-error": True},
            "verus-infer": {"crate_success": False, "interface_written": False}}
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "stale.vir"
            artifact.write_bytes(b"seed")
            with self.assertRaises(ValueError):
                fee.validate_case(dict(returncode=1, stdout=json.dumps(data), stderr="unsupported"),
                    "frontend", artifact, None)

    def test_resource_exhaustion_is_not_a_semantic_result(self):
        data = {"verification-results": {"success": False, "is-verifying-entire-crate": True},
            "verus-infer": {"crate_success": False, "interface_written": False,
                "status": "required_proof_incomplete", "functions": [{
                    "exported": False, "required_checks_passed": False,
                    "proofs": {"soundness": {"required": True, "status": "incomplete", "reason": "resource_limit"}}}
                    for _index in range(2)]}}
        run = dict(returncode=1, stdout=json.dumps(data), stderr="resource limit")
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "absent.vir"
            fee.validate_case(run, "budget", artifact, None)
            with self.assertRaises(ValueError):
                fee.validate_case(run, "semantic", artifact, None)
            for replacement in ("check_not_run", "frontend_error", None):
                changed = copy.deepcopy(data)
                for function in changed["verus-infer"]["functions"]:
                    function["proofs"]["soundness"]["reason"] = replacement
                with self.assertRaises(ValueError):
                    fee.validate_case(dict(run, stdout=json.dumps(changed)), "budget", artifact, None)

    def test_budget_category_rejects_unrelated_failures_and_wrong_scope(self):
        data = self.negative_result()
        data["verus-infer"].update(status="required_proof_incomplete")
        for function in data["verus-infer"]["functions"]:
            function.update(required_checks_passed=False, proofs={"soundness": {
                "required": True, "status": "incomplete", "reason": "resource_limit"}})
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "absent.vir"
            fee.validate_case(self.invocation(data, "resource limit"), "budget", artifact, None)
            with self.assertRaisesRegex(ValueError, "budget failure diagnostic missing"):
                fee.validate_case(self.invocation(data, "different failure"), "budget", artifact, None)
            for diagnostic in ("resource limit; panicked at internal error", "resource limit; timed out"):
                with self.subTest(diagnostic=diagnostic), self.assertRaisesRegex(ValueError, "unrelated reason"):
                    fee.validate_case(self.invocation(data, diagnostic), "budget", artifact, None)
            changed = copy.deepcopy(data)
            changed["verification-results"]["encountered-vir-error"] = True
            with self.assertRaisesRegex(ValueError, "unrelated reason"):
                fee.validate_case(self.invocation(changed, "resource limit"), "budget", artifact, None)
            for count in (0, 1, 3):
                changed = copy.deepcopy(data)
                changed["verus-infer"]["functions"] = [data["verus-infer"]["functions"][0]] * count
                with self.subTest(count=count), self.assertRaisesRegex(ValueError, "did not block inference"):
                    fee.validate_case(self.invocation(changed, "resource limit"), "budget", artifact, None)

    def test_provider_certificate_requires_every_helper_check_without_extra_trust(self):
        certificate = dict(schema=INTERFACE_SCHEMA, policy=INTERFACE_POLICY,
            soundness=True, maximality=True, exactness=True, summaries="0" * 64,
            dependencies=[], trusted_models=[], proofs=[[
                {"path": {"segments": [name]}},
                {key: True for key in ("soundness", "maximality", "exactness")}]
                for name in ("fee_encoded_expiration", "fee_from_expiration")])

        def blob(data):
            return json.dumps(data).encode() + b"VRINFER1"

        self.assertEqual(fee.provider_certificate(blob(certificate))[0], certificate)
        variants = []
        for key in ("soundness", "maximality", "exactness"):
            for value in (False, 1):
                changed = copy.deepcopy(certificate)
                changed[key] = value
                variants.append(changed)
                for index in range(2):
                    changed = copy.deepcopy(certificate)
                    changed["proofs"][index][1][key] = value
                    variants.append(changed)
        for key, value in (("schema", 0), ("policy", "unapproved"), ("dependencies", ["extra"]),
                ("trusted_models", ["extra"]), ("proofs", certificate["proofs"][:1])):
            changed = copy.deepcopy(certificate)
            changed[key] = value
            variants.append(changed)
        changed = copy.deepcopy(certificate)
        changed["proofs"][0][0]["path"]["segments"] = ["different_helper"]
        variants.append(changed)
        for changed in variants:
            with self.subTest(certificate=changed), self.assertRaisesRegex(ValueError, "certificate"):
                fee.provider_certificate(blob(changed))


class FeeReplayTests(unittest.TestCase):
    positive_result = FeeEncodingTests.positive_result
    negative_result = FeeEncodingTests.negative_result
    private_positive_result = FeeEncodingTests.private_positive_result
    invocation = FeeEncodingTests.invocation

    def test_provider_phase_requires_miss_hit_and_fresh_solver_checks(self):
        for phase, cache in (("cold", "miss"), ("warm", "hit")):
            data = self.positive_result()
            for function in data["verus-infer"]["functions"]:
                function["metrics"]["cache"] = cache
            fee.check_provider_phase(data, phase)
            changed = copy.deepcopy(data)
            changed["verus-infer"]["functions"][0]["metrics"]["cache"] = "wrong"
            with self.assertRaisesRegex(ValueError, "cache behavior"):
                fee.check_provider_phase(changed, phase)
            for value in (0, -1, None, True):
                changed = copy.deepcopy(data)
                changed["verus-infer"]["functions"][0]["metrics"]["solver_micros"] = value
                with self.subTest(phase=phase, value=value), self.assertRaisesRegex(ValueError, "solver work"):
                    fee.check_provider_phase(changed, phase)
        with self.assertRaisesRegex(ValueError, "unknown provider phase"):
            fee.check_provider_phase(self.positive_result(), "other")

    def test_provider_contracts_and_unique_crate_identity_are_required(self):
        cold, warm = self.positive_result(), self.positive_result()
        fee.check_provider_contracts(cold, warm)
        warm["verus-infer"]["functions"][0]["contract"]["ensures"] = "false"
        with self.assertRaisesRegex(ValueError, "warm exact contract changed"):
            fee.check_provider_contracts(cold, warm)
        certificate = {"proofs": [[{"path": {"krate": {"Id": ["fee_provider", 7]}}}, {}]
            for _index in range(2)]}
        self.assertEqual(fee.certificate_import(certificate, "warm"), [dict(crate_id=7, fingerprint="warm")])
        for identity in (8, True):
            changed = copy.deepcopy(certificate)
            changed["proofs"][1][0]["path"]["krate"]["Id"][1] = identity
            with self.subTest(identity=identity), self.assertRaisesRegex(ValueError, "crate identity"):
                fee.certificate_import(changed, "warm")

    def test_real_encode_and_decode_call_paths_are_required(self):
        recurring = "fee_encoded_expiration(routing_info.receive_fee.fee(amount).msats)"
        receive = "Amount::from_msats(fee_from_expiration("
        fee.check_call_paths(recurring, receive)
        with self.assertRaisesRegex(ValueError, "encoding call path"):
            fee.check_call_paths(recurring.replace("amount", "other"), receive)
        with self.assertRaisesRegex(ValueError, "decoding call path"):
            fee.check_call_paths(recurring, receive.replace("Amount", "Other"))

    def test_setup_exit_is_checked_before_reading_output(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            (output / "pin.stdout").write_text("  pinned  \n")
            self.assertEqual(fee.setup_output(Mock(return_value=0), "pin", ["true"], {}, output), "pinned")
            for code in (1, 17, None, False):
                with self.subTest(code=code), self.assertRaisesRegex(ValueError, "setup failed: pin"):
                    fee.setup_output(Mock(return_value=code), "pin", ["true"], {}, output)

    def test_completion_requires_clean_inputs_guards_and_all_seeded_removals(self):
        cases = [dict(name=name, category=category,
            seeded_export_sha256=None if category == "positive" else "a" * 64,
            export_sha256="b" * 64 if category == "positive" else None)
            for name, category in fee.EXPECTED_CASES.items()]
        guard, imported = Mock(), Mock()
        fee.check_completion("", cases, guard, imported)
        guard.check.assert_called_once_with()
        imported.check.assert_called_once_with()
        with self.assertRaisesRegex(ValueError, "became dirty"):
            fee.check_completion("?? changed.rs", cases, Mock(), Mock())
        for selected in (0, 1):
            guards = [Mock(), Mock()]
            guards[selected].check.side_effect = RuntimeError("changed input")
            with self.subTest(selected=selected), self.assertRaisesRegex(RuntimeError, "changed input"):
                fee.check_completion("", cases, *guards)
        for key, value in (("seeded_export_sha256", None), ("seeded_export_sha256", ""),
                ("export_sha256", "b" * 64)):
            changed = copy.deepcopy(cases)
            next(case for case in changed if case["category"] != "positive")[key] = value
            with self.subTest(key=key, value=value), self.assertRaisesRegex(ValueError, "stale interface"):
                fee.check_completion("", changed, Mock(), Mock())
        with self.assertRaisesRegex(ValueError, "replay cases"):
            fee.check_completion("", cases[:-1], Mock(), Mock())
        for change in ("duplicate", "misclassified"):
            changed = copy.deepcopy(cases)
            if change == "duplicate":
                changed[-1] = copy.deepcopy(changed[0])
            else:
                next(case for case in changed if case["name"] == "payment_fee_cold")["category"] = "frontend"
            with self.subTest(change=change), self.assertRaisesRegex(ValueError, "replay cases"):
                fee.check_completion("", changed, Mock(), Mock())

    def test_previous_division_blocker_is_now_an_exact_private_positive(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "private.vir"
            artifact.write_bytes(b"fresh")
            data = self.private_positive_result()
            fee.validate_case(self.invocation(data), "positive", artifact, 1)
            for field, value in (("exported", True), ("contract", dict(requires="false")),
                    ("contract", dict(requires="(0 != 0)"))):
                changed = copy.deepcopy(data)
                changed["verus-infer"]["functions"][0][field] = value
                with self.subTest(field=field, value=value), self.assertRaisesRegex(ValueError, "domain narrowed"):
                    fee.validate_case(self.invocation(changed), "positive", artifact, 1)

    def test_private_positive_requires_exact_helper_and_checked_native_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "private.vir"
            artifact.write_bytes(b"fresh")
            for target in ("inferred", "native", "both"):
                changed = self.private_positive_result()
                if target in ("inferred", "both"):
                    changed["verus-infer"]["functions"][0]["function"] = "unrelated::helper"
                if target in ("native", "both"):
                    changed["verus-infer"]["verification_scope"]["native_functions"][0]["function"] = "unrelated::helper"
                with self.subTest(target=target), self.assertRaisesRegex(ValueError, "helper identity"):
                    fee.validate_case(self.invocation(changed), "positive", artifact, 1)
            for field, value in (("selected", False), ("automatically_derived", True),
                    ("inference_candidate", 1), ("ordinary_contract", 0),
                    ("body_status", "unchecked")):
                changed = self.private_positive_result()
                changed["verus-infer"]["verification_scope"]["native_functions"][0][field] = value
                with self.subTest(field=field), self.assertRaisesRegex(ValueError, "helper identity"):
                    fee.validate_case(self.invocation(changed), "positive", artifact, 1)
            for rows in ([], self.private_positive_result()["verus-infer"]["verification_scope"]["native_functions"] * 2):
                changed = self.private_positive_result()
                changed["verus-infer"]["verification_scope"]["native_functions"] = rows
                with self.subTest(rows=rows), self.assertRaisesRegex(ValueError, "helper identity"):
                    fee.validate_case(self.invocation(changed), "positive", artifact, 1)

    def test_runtime_guard_includes_driver_llvm_and_standard_rlibs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            compiler = root / "bin/rustc"
            paths = [compiler, root / "lib/librustc_driver-hash.dylib", root / "lib/libLLVM.dylib",
                root / "lib/rustlib/host/lib/libstd-hash.rlib", root / "lib/libother.so.1"]
            for path in paths:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"immutable")
            self.assertEqual({path.resolve() for path in fee.compiler_artifacts(compiler)},
                {path.resolve() for path in paths})
            guard = fee.ArtifactGuard(fee.compiler_artifacts(compiler))
            paths[2].write_bytes(b"changed")
            with self.assertRaisesRegex(RuntimeError, "tool inputs changed"):
                guard.check()
            paths[1].unlink()
            with self.assertRaisesRegex(ValueError, "runtime or standard libraries missing"):
                fee.compiler_artifacts(compiler)

    def test_cache_identity_is_observed_and_pinned(self):
        certificate = dict(schema=INTERFACE_SCHEMA, policy=INTERFACE_POLICY)
        version = CACHE_VERSION
        with tempfile.TemporaryDirectory() as directory:
            cache = Path(directory)
            with self.assertRaisesRegex(ValueError, "cache version"):
                fee.cache_identity(cache, certificate)
            entry = cache / "candidate.json"
            entry.write_text(json.dumps(dict(version=version)))
            observed = fee.cache_identity(cache, certificate)
            self.assertEqual(observed["engine_identity"], list(ENGINE_IDENTITY))
            self.assertEqual(observed["cache_entry_sha256"], {entry.name: fee.digest(entry)})
            for changed in ("verus-infer-v74:checked-computed-shift-goals:4096", version + "-different"):
                entry.write_text(json.dumps(dict(version=changed)))
                with self.subTest(changed=changed), self.assertRaisesRegex(ValueError, "cache version"):
                    fee.cache_identity(cache, certificate)
            entry.write_text(json.dumps(dict(version=version)))
            with self.assertRaisesRegex(ValueError, "unrecognized interface policy"):
                fee.cache_identity(cache, dict(certificate, policy="other"))

    def test_frontend_and_import_categories_cannot_hide_semantic_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "absent.vir"
            data = self.negative_result()
            with self.assertRaisesRegex(ValueError, "unknown replay category"):
                fee.validate_case(self.invocation(data), "unknown", artifact, None)
            for category, diagnostic in (("frontend", "error[E0277] recovered == true"),
                    ("import", "Verus Infer: incompatible inference schema")):
                with self.subTest(category=category), self.assertRaises(ValueError):
                    fee.validate_case(self.invocation(data, diagnostic), category, artifact, None)
            data["verification-results"].update(verified=0, errors=0, **{"encountered-error": True})
            fee.validate_case(self.invocation(data, "error[E0277]"), "frontend", artifact, None,
                marker="error[E0277]")
            with self.assertRaisesRegex(ValueError, "missing expected diagnostic"):
                fee.validate_case(self.invocation(data, "recovered == true"), "frontend", artifact,
                    None, marker="error[E0277]")

    def execute_case(self, output, data, *, name="provider", phase="cold", category="positive",
            imports=(), imported_guard=None, receipt=None, publish=True, publish_native=True, local_count=2):
        calls = []
        receipt = dict(cases=[]) if receipt is None else receipt

        def command(label, argv, environment):
            artifact = Path(argv[argv.index("--export") + 1])
            calls.append(dict(argv=argv, before=artifact.read_bytes() if artifact.exists() else None))
            if data["verification-results"]["success"]:
                if publish:
                    artifact.write_bytes(label.encode())
                if "--compile" in argv and publish_native:
                    Path(argv[argv.index("-o") + 1]).write_bytes(label.encode())
            else:
                artifact.unlink(missing_ok=True)
            (output / (label + ".stdout")).write_text(json.dumps(data))
            (output / (label + ".stderr")).write_text(
                "postcondition not satisfied" if category == "semantic" else "")
            return int(not data["verification-results"]["success"])

        fee.run_case(command, {}, output, Path("verus"), receipt, name=name, phase=phase,
            category=category, local_count=local_count, imports=imports, imported_guard=imported_guard)
        return calls, receipt

    def test_provider_exports_are_fresh_phase_specific_and_no_cheating(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            calls, _receipt = self.execute_case(output, self.positive_result())
            self.assertIsNone(calls[0]["before"])
            self.assertIn("--no-cheating", calls[0]["argv"])
            self.assertIn("--compile", calls[0]["argv"])
            warm = self.positive_result()
            for function in warm["verus-infer"]["functions"]:
                function["metrics"]["cache"] = "hit"
            with self.assertRaisesRegex(ValueError, "publication disagree"):
                self.execute_case(output, warm, phase="warm", publish=False)
            warm_calls, _receipt = self.execute_case(output, warm, phase="warm")
            self.assertEqual(calls[0]["argv"], warm_calls[0]["argv"])
            self.assertNotEqual((output / "provider_cold.vir").read_bytes(),
                (output / "provider_warm.vir").read_bytes())
            with self.assertRaisesRegex(ValueError, "positive export path already exists"):
                self.execute_case(output, warm, phase="warm")

    def test_warm_native_compilation_cannot_reuse_the_cold_rlib(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            self.execute_case(output, self.positive_result())
            warm = self.positive_result()
            for function in warm["verus-infer"]["functions"]:
                function["metrics"]["cache"] = "hit"
            with self.assertRaisesRegex(ValueError, "provider native artifact missing"):
                self.execute_case(output, warm, phase="warm", publish_native=False)
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            (output / "provider.vir").write_bytes(b"stale working export")
            with patch.object(Path, "unlink", return_value=None):
                with self.assertRaisesRegex(ValueError, "working export path was not cleared"):
                    self.execute_case(output, self.positive_result())

    def test_runner_enforces_cache_and_warm_import_binding(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            warm = self.positive_result()
            with self.assertRaisesRegex(ValueError, "cache behavior"):
                self.execute_case(output, warm, phase="warm")
            data = self.positive_result()
            data["verification-results"]["verified"] = 5
            data["verus-infer"]["verification_scope"]["checked_exec_bodies"] = 5
            data["verus-infer"]["functions"] = []
            data["verus-infer"]["imported_inference_dependencies"] = [dict(crate_id=7, fingerprint="cold")]
            with self.assertRaisesRegex(ValueError, "fingerprint differs"):
                self.execute_case(output, data, name="client", imports=["--import", "provider"],
                    local_count=0, imported_guard=Mock(),
                    receipt=dict(cases=[], provider_import=[dict(crate_id=7, fingerprint="warm")]))

    def test_failure_is_seeded_and_imports_guarded_before_and_after(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            (output / "provider_warm.vir").write_bytes(b"checked provider")
            guard = Mock()
            calls, receipt = self.execute_case(output, self.negative_result(), name="mutant",
                category="semantic", imports=["--import", "provider"], imported_guard=guard)
            self.assertEqual(calls[0]["before"], b"checked provider")
            self.assertIn("--no-cheating", calls[0]["argv"])
            self.assertEqual(guard.check.call_count, 2)
            self.assertEqual(receipt["cases"][0]["seeded_export_sha256"], fee.digest(output / "provider_warm.vir"))
            self.assertIsNone(receipt["cases"][0]["export_sha256"])
            with self.assertRaisesRegex(ValueError, "imported provider guard missing"):
                self.execute_case(output, self.negative_result(), name="missing_guard",
                    category="semantic", imports=["--import", "provider"])
            with patch.object(Path, "write_bytes", return_value=0):
                with self.assertRaisesRegex(ValueError, "failure export was not seeded"):
                    self.execute_case(output, self.negative_result(), name="missing_seed", category="semantic")


    def test_complete_replay_wires_every_audited_gate(self):
        with tempfile.TemporaryDirectory() as directory, ExitStack() as patches:
            root = Path(directory).resolve()
            output, application = root / "output", root / "application"
            output.mkdir()
            sources = {fee.CONTRACTS: FIXTURE.read_text(),
                fee.FEES: "pub struct PaymentFee { pub base: u64 }\nfn absolute_fee() {}",
                fee.AMOUNT: "pub struct Amount { pub msats: u64 }",
                fee.RECURRING: "fee_encoded_expiration(routing_info.receive_fee.fee(amount).msats)",
                fee.RECEIVE: "Amount::from_msats(fee_from_expiration("}
            for relative, contents in sources.items():
                path = application / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(contents)
            compiler = root / "toolchain/bin/rustc"
            for path in (compiler, root / "toolchain/lib/librustc_driver.dylib",
                    root / "toolchain/lib/libLLVM.dylib", root / "toolchain/lib/libstd-hash.rlib"):
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"runtime")
            receipt = dict(commands=[], cases=[])
            diagnostics = dict(schema="incompatible inference schema, safety policy, or producer",
                policy="incompatible inference schema, safety policy, or producer",
                exactness="incomplete inference proof guarantees", summaries="inferred contract fingerprint mismatch",
                dependencies="stale or missing inferred dependency fingerprint",
                trusted_models="stale or missing trusted-model fingerprint",
                missing="missing or invalid inference proof metadata")

            def execute(label, argv, environment):
                if command.guard is not None:
                    command.guard.check()
                setup = dict(application_head=fee.APPLICATION, application_clean="", application_clean_after="",
                    rustc=str(compiler), engine_files="", application_files="\n".join(sources),
                    native_build="", native_samples="")
                stderr, code = "", 0
                if label in setup:
                    stdout = setup[label]
                else:
                    self.assertIn("--no-cheating", argv)
                    artifact = Path(argv[argv.index("--export") + 1])
                    positive = label in ("provider_cold", "provider_warm", "client_cold", "client_warm", "payment_fee_cold")
                    self.assertEqual(artifact.exists(), not positive)
                    data = self.positive_result() if positive else self.negative_result()
                    if positive:
                        blob = label.encode()
                        if label.startswith("provider_"):
                            warm = label.endswith("warm")
                            for function in data["verus-infer"]["functions"]:
                                function.update(candidate_id=label)
                                function["metrics"]["cache"] = "hit" if warm else "miss"
                            certificate = dict(schema=INTERFACE_SCHEMA, policy=INTERFACE_POLICY,
                                soundness=True, maximality=True, exactness=True, summaries=("b" if warm else "a") * 64,
                                dependencies=[], trusted_models=[], proofs=[[
                                    {"path": {"segments": [name], "krate": {"Id": ["fee_provider", 7]}}},
                                    {key: True for key in ("soundness", "maximality", "exactness")}]
                                    for name in ("fee_encoded_expiration", "fee_from_expiration")])
                            blob = json.dumps(certificate).encode() + b"VRINFER1"
                            cache = Path(environment["VERUS_INFER_CACHE"])
                            cache.mkdir(exist_ok=True)
                            (cache / "candidate.json").write_text(json.dumps(dict(
                                version=CACHE_VERSION)))
                            Path(argv[argv.index("-o") + 1]).write_bytes(label.encode())
                        elif label.startswith("client_"):
                            data["verification-results"]["verified"] = 5
                            data["verus-infer"]["verification_scope"]["checked_exec_bodies"] = 5
                            data["verus-infer"]["functions"] = []
                            _certificate, fingerprint = fee.provider_certificate((output / "provider_warm.vir").read_bytes())
                            data["verus-infer"]["imported_inference_dependencies"] = [dict(crate_id=7, fingerprint=fingerprint)]
                        else:
                            data = self.private_positive_result()
                        artifact.write_bytes(blob)
                    else:
                        artifact.unlink()
                        code = 1
                        stderr = "postcondition not satisfied\nrecovered == fee\nexpiration > 0"
                        if label == "unannotated_cold":
                            data["verus-infer"]["functions"] = []
                        elif label == "overflow_cold":
                            stderr = "possible arithmetic underflow/overflow"
                        elif label == "panic_cold":
                            stderr = "precondition not satisfied"
                        elif label == "malformed_cold" or label.startswith("client_metadata_"):
                            data["verification-results"].update(verified=0, errors=0,
                                **{"encountered-error": True, "encountered-vir-error": label != "malformed_cold"})
                            data["verus-infer"]["functions"] = []
                            stderr = ("error[E0277]" if label == "malformed_cold" else
                                "Verus Infer: " + diagnostics[label.removeprefix("client_metadata_")])
                        elif label == "budget_cold":
                            data["verus-infer"]["status"] = "required_proof_incomplete"
                            for function in data["verus-infer"]["functions"]:
                                function.update(required_checks_passed=False, proofs={"soundness": dict(
                                    required=True, status="incomplete", reason="resource_limit")})
                            stderr = "resource limit"
                    stdout = json.dumps(data)
                (output / (label + ".stdout")).write_text(stdout)
                (output / (label + ".stderr")).write_text(stderr)
                if command.guard is not None:
                    command.guard.check()
                return code

            command = Mock(side_effect=execute)
            command.guard = None
            patches.enter_context(patch.object(fee, "SOURCE_HASHES", {
                relative: fee.digest(application / relative) for relative in sources}))
            patches.enter_context(patch.object(fee, "LoggedCommands", return_value=command))
            patches.enter_context(patch.object(fee, "caller_evidence", return_value={}))
            patches.enter_context(patch.object(fee, "tool_artifacts", side_effect=lambda _verifier, _solver, extra: extra))
            names = ("check_pin", "check_call_paths", "compiler_artifacts", "check_provider_contracts",
                "certificate_import", "cache_identity", "check_completion")
            gates = {name: patches.enter_context(patch.object(fee, name, wraps=getattr(fee, name))) for name in names}
            fee.replay(SimpleNamespace(application=application, verus=root / "verus", solver=root / "z3", timeout=1),
                output, receipt)
            for name, gate in gates.items():
                with self.subTest(gate=name):
                    gate.assert_called_once()
            self.assertIsNotNone(command.guard)
            self.assertEqual(receipt["status"], "proved")
            self.assertTrue(receipt["stale_exports_removed"])
            self.assertEqual(receipt["engine_identity"], list(ENGINE_IDENTITY))
            self.assertEqual(len(receipt["cases"]), 19)
            self.assertEqual(len(receipt["rust_toolchain_artifacts"]), 4)


if __name__ == "__main__":
    unittest.main()
