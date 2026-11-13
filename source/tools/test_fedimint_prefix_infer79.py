"""Acceptance checks exercised from otherwise-valid fixtures, without masking."""

import copy
import json
from pathlib import Path
import tempfile
import unittest

import check_fedimint_prefix_infer79 as infer
import workflow_runtime as runtime


CONTRACTS = {"key_contents": {"requires": "true", "ensures": "result@ == prefix@ + key@"}}
PATH = Path("/private/tmp/prefix-infer79-unit/provider.rs")


def fixture():
    scope = dict(coverage="selected local Verus items; not all Rust code", native_collection_completed=True,
                 complete_selected_scope_verified=True, checked_exec_bodies=1, checked_proof_bodies=0,
                 native_functions=[dict(function="prefix_provider::key_contents", rust_function="key_contents",
                    source="{source}:1:1: 1:60", automatically_derived=False, selected=True,
                    body_status="checked_exec_body", inference_candidate=True, ordinary_contract=False)], limitations=[])
    actual_scope = copy.deepcopy(scope)
    actual_scope["native_functions"][0]["source"] = actual_scope["native_functions"][0]["source"].replace("{source}", str(PATH))
    function = dict(function="prefix_provider::key_contents", contract=copy.deepcopy(CONTRACTS["key_contents"]),
                    mode="exact", required_checks_passed=True, exported=True, available_to_same_crate_callers=True,
                    proofs={name: dict(status="proved", required=True) for name in infer.OBLIGATIONS},
                    metrics=dict(solver_micros=1, cache="miss", cache_policy="regenerated_state"))
    data = {"verification-results": dict(success=True, **{"encountered-error": False,
        "encountered-vir-error": False, "is-verifying-entire-crate": True}, errors=0, verified=3),
        "verus-infer": dict(schema_version=1, status="verified", crate_success=True, interface_written=True,
            failed_functions=[], requested_functions=1, generated_functions=1, generation_failure=None,
            functions=[function], verification_scope=actual_scope,
            trusted_models=[dict(function="vstd::seq::Seq::add", fingerprint="a" * 64)])}
    scope["verified"] = 3
    return data, scope


def certificate_fixture():
    return dict(schema=infer.IDENTITY[2], foreign_dependencies=[], policy=infer.POLICY, producer="unit-only", soundness=True, maximality=True, exactness=True,
                proofs=[[dict(path=dict(krate=dict(Id=["prefix_provider", 1]), segments=["key_contents"])),
                         dict(soundness=True, maximality=True, exactness=True)]],
                summaries="a" * 64, dependencies=[], trusted_models=[[dict(path=dict(krate="Vstd", segments=["seq", "Seq", "add"])), "b" * 64]])


def certificate_blob(value):
    return json.dumps(value, separators=(",", ":")).encode() + b"VRINFER1"


class PrefixInferAcceptanceTests(unittest.TestCase):
    def accept(self, data=None, code=0):
        valid, scope = fixture()
        return infer.positive(valid if data is None else data, code, CONTRACTS, scope, PATH, "miss")

    def reject(self, mutate, message):
        data, unused = fixture()
        self.accept(data)
        mutate(data)
        with self.assertRaisesRegex(ValueError, message):
            self.accept(data)

    def test_valid_positive(self):
        self.accept()

    def test_exit_code_type(self):
        for value in (False, True, 1, None, "0", 0.0):
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, "exit code"):
                self.accept(code=value)

    def test_each_result_flag_independently(self):
        for name in ("success", "encountered-error", "encountered-vir-error", "is-verifying-entire-crate"):
            for value in (None, "true", 0, 1, []):
                with self.subTest(name=name, value=value):
                    self.reject(lambda data: data["verification-results"].__setitem__(name, value), "result flag")

    def test_each_result_count_independently(self):
        for name, correct in (("verified", 3), ("errors", 0)):
            for value in (None, str(correct), float(correct), False, correct + 1):
                with self.subTest(name=name, value=value):
                    self.reject(lambda data: data["verification-results"].__setitem__(name, value), "verified count|positive errors")

    def test_status_generation_and_publication(self):
        for name, value, reason in (("status", "unsupported", "inference status"),
            ("crate_success", 1, "inference status"), ("interface_written", 1, "publication"),
            ("failed_functions", ["bad"], "publication"), ("generation_failure", "bad", "generation failure"),
            ("requested_functions", True, "requested"), ("generated_functions", True, "generated"),
            ("schema_version", True, "schema")):
            with self.subTest(name=name):
                self.reject(lambda data: data["verus-infer"].__setitem__(name, value), reason)

    def test_every_exactness_gate(self):
        for name, value, message in (("mode", "sound", "exact inference"),
            ("required_checks_passed", 1, "exact inference"), ("exported", 1, "function publication")):
            with self.subTest(name=name):
                self.reject(lambda data: data["verus-infer"]["functions"][0].__setitem__(name, value), message)
        for obligation in infer.OBLIGATIONS:
            for name, value in (("required", 1), ("status", "skipped")):
                with self.subTest(obligation=obligation, name=name):
                    self.reject(lambda data: data["verus-infer"]["functions"][0]["proofs"][obligation].__setitem__(name, value), "required proof")
        self.reject(lambda data: data["verus-infer"]["functions"][0].__setitem__("available_to_same_crate_callers", 1), "same-crate availability")

    def test_standard_trust_inventory_cannot_hide_application_summaries(self):
        self.reject(lambda data: data["verus-infer"].__setitem__("trusted_models", []), "standard models missing")
        self.reject(lambda data: data["verus-infer"]["trusted_models"][0].__setitem__("function", "prefix_provider::trusted_concat"), "trusted application summary")
        self.reject(lambda data: data["verus-infer"]["trusted_models"][0].__setitem__("fingerprint", "not-a-hash"), "model fingerprint")

    def test_complete_contract_function_set_and_fresh_work(self):
        self.reject(lambda data: data["verus-infer"]["functions"].append(data["verus-infer"]["functions"][0]), "function rows")
        self.reject(lambda data: data["verus-infer"]["functions"][0]["contract"].__setitem__("requires", "false"), "exact generated contract")
        self.reject(lambda data: data["verus-infer"]["functions"][0].__setitem__("function", "prefix_provider::range_contents"), "exact generated contract")
        for value in (0, -1, True, None, "1"):
            with self.subTest(value=value):
                self.reject(lambda data: data["verus-infer"]["functions"][0]["metrics"].__setitem__("solver_micros", value), "fresh solver work")
        self.reject(lambda data: data["verus-infer"]["functions"][0]["metrics"].__setitem__("cache", "hit"), "cache phase")
        self.reject(lambda data: data["verus-infer"]["functions"][0]["metrics"].__setitem__("cache_policy", "candidate_cache"), "must regenerate")

    def test_all_scope_fields_have_exact_typed_identity(self):
        for name in fixture()[0]["verus-infer"]["verification_scope"]:
            with self.subTest(name=name):
                self.reject(lambda data: data["verus-infer"]["verification_scope"].pop(name), "verification scope")
        row = fixture()[0]["verus-infer"]["verification_scope"]["native_functions"][0]
        for name, value in row.items():
            for changed in (None, 1 if type(value) is bool else "wrong"):
                with self.subTest(name=name, changed=changed):
                    self.reject(lambda data: data["verus-infer"]["verification_scope"]["native_functions"][0].__setitem__(name, changed), "verification scope")
        self.reject(lambda data: data["verus-infer"]["verification_scope"].__setitem__("checked_exec_bodies", True), "verification scope")

    def test_exact_commands_reject_joined_unrecorded_and_duplicate_options(self):
        arguments = (Path("/tools/verus"), PATH, Path("/sysroot"), "prefix_provider", Path("/out/api.vir"), Path("/out/libapi.rlib"))
        command = infer.command(*arguments)
        infer.validate_command(command, *arguments)
        for changed in (command + ["--no-verify"], command + ["--verify-function=key_contents"],
            command + ["--no-cheating"], [word for word in command if word != "--infer-goals"],
            command + ["--import=prefix_provider=/tmp/other.vir"]):
            with self.subTest(command=changed), self.assertRaisesRegex(ValueError, "command options"):
                infer.validate_command(changed, *arguments)

    def test_original_item_and_input_domains_unchanged(self):
        pieces = infer.read_json(infer.boundary.EVIDENCE / "native-items.json")
        source = infer.render(pieces)
        self.assertEqual(len(infer.correspondence(source, pieces)), 3)
        for name in pieces:
            altered = source.replace(pieces[name], pieces[name].replace("u32", "u64"), 1) if name == "token" else source.replace(pieces[name], pieces[name].replace("u8", "u16", 1), 1)
            if altered == source:
                continue
            with self.subTest(name=name), self.assertRaises(ValueError):
                infer.correspondence(altered, pieces)

    def test_certificate_is_required_and_duplicate_keys_rejected(self):
        for blob in (b"", b"{}VRINFER1", b'{"schema":38,"schema":38}VRINFER1'):
            with self.subTest(blob=blob), self.assertRaises(ValueError):
                infer.check_certificate(blob)

    def test_each_certificate_guarantee_from_an_otherwise_valid_certificate(self):
        valid = certificate_fixture()
        infer.check_certificate(certificate_blob(valid))
        for key, value, reason in (("schema", True, "schema"), ("schema", 39, "schema"),
            ("policy", "candidate-bound-evidence-v71", "policy"), ("producer", "", "producer"),
            ("summaries", "not-a-hash", "summary fingerprint"), ("proofs", [], "certificate proofs"),
            ("trusted_models", [], "model inventory")):
            changed = copy.deepcopy(valid)
            changed[key] = value
            with self.subTest(key=key, value=value), self.assertRaisesRegex(ValueError, reason):
                infer.check_certificate(certificate_blob(changed))
        for obligation in infer.OBLIGATIONS:
            for value in (False, 1, None):
                changed = copy.deepcopy(valid)
                changed[obligation] = value
                with self.subTest(obligation=obligation, value=value), self.assertRaisesRegex(ValueError, "proof guarantees"):
                    infer.check_certificate(certificate_blob(changed))
                changed = copy.deepcopy(valid)
                changed["proofs"][0][1][obligation] = value
                with self.subTest(function_obligation=obligation, value=value), self.assertRaisesRegex(ValueError, "function guarantees"):
                    infer.check_certificate(certificate_blob(changed))

    def test_certificate_model_identity_shape_hash_and_duplicates(self):
        valid = certificate_fixture()
        for mutation, reason in (("crate", "application model"), ("path", "model path"),
                                  ("hash", "certificate fingerprint"), ("duplicate", "duplicate trusted")):
            changed = copy.deepcopy(valid)
            if mutation == "crate":
                changed["trusted_models"][0][0]["path"]["krate"] = {"Id": ["application", 1]}
            elif mutation == "path":
                changed["trusted_models"][0][0]["path"]["segments"] = [False]
            elif mutation == "hash":
                changed["trusted_models"][0][1] = "corrupt"
            else:
                changed["trusted_models"].append(copy.deepcopy(changed["trusted_models"][0]))
            with self.subTest(mutation=mutation), self.assertRaisesRegex(ValueError, reason):
                infer.check_certificate(certificate_blob(changed))

    def test_import_kinds_certificate_and_actual_artifacts_are_bound(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            vir, rlib = root / "provider.vir", root / "libprovider.rlib"
            vir.write_bytes(b"actual-vir")
            rlib.write_bytes(b"actual-rlib")
            data = {"verus-infer": {"imported_inference_dependencies": [dict(crate_id=1, fingerprint="c" * 64)]}}
            command = ["--import", "prefix_provider=" + str(vir), "--extern", "prefix_provider=" + str(rlib)]
            accepted = infer.imported(data, vir, rlib, certificate_fixture(), "c" * 64, command)
            self.assertEqual(accepted["vir_sha256"], infer.digest(vir))
            self.assertEqual(accepted["rlib_sha256"], infer.digest(rlib))
            for changed in (["--import", "prefix_provider=" + str(vir)], command + ["--extern", "other=x"],
                            ["--import", "prefix_provider=/wrong.vir", "--extern", "prefix_provider=" + str(rlib)]):
                with self.subTest(command=changed), self.assertRaises(ValueError):
                    infer.imported(data, vir, rlib, certificate_fixture(), "c" * 64, changed)
            for wrong in (True, 2, None):
                changed = copy.deepcopy(data)
                changed["verus-infer"]["imported_inference_dependencies"][0]["crate_id"] = wrong
                with self.subTest(crate_id=wrong), self.assertRaisesRegex(ValueError, "certificate binding"):
                    infer.imported(changed, vir, rlib, certificate_fixture(), "c" * 64, command)
            rlib.unlink()
            with self.assertRaisesRegex(ValueError, "artifacts missing"):
                infer.imported(data, vir, rlib, certificate_fixture(), "c" * 64, command)

    def test_failure_location_cannot_be_an_incidental_other_goal(self):
        source = "fn semantic_goal() ensures false, {}\nfn other() {}\n"
        diagnostic = "postcondition not satisfied"
        good = "error: " + diagnostic + "\n  --> " + str(PATH) + ":1:1\n"
        infer.failure_location(good, source, PATH, "prefix_provider::semantic_goal", diagnostic)
        for changed in (good.replace(":1:1", ":2:1"), good.replace(str(PATH), "/wrong.rs"), good + good):
            with self.subTest(stderr=changed), self.assertRaisesRegex(ValueError, "outside intended"):
                infer.failure_location(changed, source, PATH, "prefix_provider::semantic_goal", diagnostic)

    def test_completion_rejects_each_missing_misclassified_or_unseeded_case(self):
        categories = {name: "positive" for name in ("cold", "warm", "consumer", "poisoned_cache",
            "restore_provider", "final_provider", "final_consumer")}
        for name in ("omit_database_prefix", "omit_transaction_prefix", "duplicate_suffix", "endpoint_is_start"):
            categories[name + "_exact"] = "positive"
            categories[name + "_semantic"] = "semantic"
        categories.update(false_goal="semantic", unsafe_index="safety", malformed_goal="malformed")
        categories.update({name: "frontend" for name in infer.frontend_controls()})
        categories.update({"metadata_" + name: "import" for name in ("policy", "proof", "summaries", "trusted_models", "missing")})
        cases = [dict(label=name, category=category, seeded_export_sha256="a" * 64,
                      export_sha256=None, failed_export_absent=True) for name, category in categories.items()]
        infer.completion(cases, False)
        infer.completion(cases + [dict(label="baseline", category="frontend")], True)
        for index, case in enumerate(cases):
            with self.subTest(label=case["label"], fault="missing"), self.assertRaisesRegex(ValueError, "case count"):
                infer.completion(cases[:index] + cases[index + 1:], False)
            changed = copy.deepcopy(cases)
            changed[index]["category"] = "wrong"
            with self.subTest(label=case["label"], fault="category"), self.assertRaisesRegex(ValueError, "misclassified"):
                infer.completion(changed, False)
            if case["category"] == "positive":
                continue
            for field, value, reason in (("seeded_export_sha256", None, "not seeded"),
                ("seeded_export_sha256", "invalid", "not seeded"), ("export_sha256", "b" * 64, "survived"),
                ("failed_export_absent", 1, "survived")):
                changed = copy.deepcopy(cases)
                changed[index][field] = value
                with self.subTest(label=case["label"], field=field, value=value), self.assertRaisesRegex(ValueError, reason):
                    infer.completion(changed, False)
        changed = copy.deepcopy(cases)
        changed[0] = copy.deepcopy(changed[1])
        with self.assertRaisesRegex(ValueError, "duplicate"):
            infer.completion(changed, False)

    def test_seeded_export_removal_is_independent_of_negative_reason(self):
        with tempfile.TemporaryDirectory() as temporary:
            artifact = Path(temporary) / "api.vir"
            artifact.write_bytes(b"prior-certified-export")
            data, unused = fixture()
            data["verification-results"].update(success=False, errors=1, **{"encountered-error": True})
            data["verus-infer"].update(crate_success=False, interface_written=False, status="caller_proof_failed", failed_functions=["prefix_provider::semantic_goal"])
            data["verus-infer"]["functions"][0]["exported"] = False
            with self.assertRaisesRegex(ValueError, "failed export retained"):
                infer.failed(data, 1, "postcondition not satisfied", artifact, "semantic", "postcondition not satisfied", CONTRACTS)
            artifact.unlink()
            infer.failed(data, 1, "postcondition not satisfied", artifact, "semantic", "postcondition not satisfied", CONTRACTS)
            data["verus-infer"]["failed_functions"] = ["prefix_provider::incidental"]
            with self.assertRaisesRegex(ValueError, "intended failed function"):
                infer.failed(data, 1, "postcondition not satisfied", artifact, "semantic", "postcondition not satisfied", CONTRACTS)
            data["verus-infer"]["failed_functions"] = ["prefix_provider::semantic_goal"]
            for value in (True, 0, "1"):
                with self.subTest(value=value), self.assertRaisesRegex(ValueError, "negative exit code"):
                    infer.failed(data, value, "postcondition not satisfied", artifact, "semantic", "postcondition not satisfied", CONTRACTS)
            data["verification-results"]["encountered-vir-error"] = True
            with self.assertRaisesRegex(ValueError, "frontend decline"):
                infer.failed(data, 1, "postcondition not satisfied", artifact, "semantic", "postcondition not satisfied", CONTRACTS)


class IntegratedPrefixRuntimeTests(unittest.TestCase):
    def case(self, root):
        source = root / 'provider.rs'
        text = 'synthetic unit fixture, not a verifier proof'
        source.write_text(text)
        executable = Path(infer.__file__)
        argv = infer.command(executable, source, root / 'sysroot', 'prefix_provider',
                             root / 'provider.vir', root / 'provider.rlib')
        directory = root / 'runs' / 'synthetic'
        directory.mkdir(parents=True)
        environment = {'TOKEN': 'synthetic'}
        inputs = [source, executable]
        data, scope = fixture()
        for row in data['verus-infer']['verification_scope']['native_functions']:
            row['source'] = row['source'].replace(str(PATH), str(source))
        runtime.record(directory / 'stdout', data)
        (directory / 'stderr').write_text('')
        runtime.record(directory / 'invocation.json', dict(argv=argv, cwd=str(root),
            environment=environment, inputs=[runtime.artifact(path) for path in inputs]))
        runtime.record(directory / 'result.json', dict(returncode=0, timeout=False, seconds=0.0,
            stdout=runtime.artifact(directory / 'stdout'), stderr=runtime.artifact(directory / 'stderr')))
        return directory, argv, source, text, environment, inputs, scope

    def accept(self, case):
        directory, argv, source, text, environment, inputs, scope = case
        infer.validate_command(argv, Path(infer.__file__), source, source.parent / 'sysroot',
            'prefix_provider', source.parent / 'provider.vir', source.parent / 'provider.rlib')
        data, result, seconds = infer.bound_case(directory, argv, source, text, environment, inputs)
        infer.positive(data, result.returncode, CONTRACTS, scope, source, 'miss')
        self.assertGreaterEqual(seconds, 0)

    def test_integrated_report_command_artifact_and_source_forgeries_reject(self):
        for mutation in ('schema', 'duplicate', 'outcome', 'command', 'artifact', 'source', 'source_split', 'environment'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                case = self.case(root)
                self.accept(case)
                directory, argv, source, text, environment, inputs, scope = case
                if mutation in ('schema', 'outcome'):
                    data = runtime.read_json(directory / 'stdout')
                    if mutation == 'schema':
                        data['verus-infer']['verification_scope'] = []
                    else:
                        data['verification-results']['success'] = 1
                    runtime.record(directory / 'stdout', data)
                elif mutation == 'duplicate':
                    original = (directory / 'stdout').read_text()
                    (directory / 'stdout').write_text('{"verification-results":{},' + original[1:])
                elif mutation == 'command':
                    invocation = runtime.read_json(directory / 'invocation.json')
                    invocation['argv'].append('--no-verify')
                    runtime.record(directory / 'invocation.json', invocation)
                elif mutation == 'source':
                    source.write_text('different bytes, coherently rehashed')
                    invocation = runtime.read_json(directory / 'invocation.json')
                    invocation['inputs'] = [runtime.artifact(path) for path in inputs]
                    runtime.record(directory / 'invocation.json', invocation)
                elif mutation == 'source_split':
                    alias = root / 'unexecuted.rs'
                    alias.write_text(text)
                    case = directory, argv, alias, text, environment, [alias, inputs[1]], scope
                elif mutation == 'environment':
                    invocation = runtime.read_json(directory / 'invocation.json')
                    invocation['environment']['RUSTFLAGS'] = '--cfg poison'
                    runtime.record(directory / 'invocation.json', invocation)
                result = runtime.read_json(directory / 'result.json')
                if mutation in ('schema', 'outcome', 'duplicate'):
                    result['stdout'] = runtime.artifact(directory / 'stdout')
                elif mutation == 'artifact':
                    other = root / 'unexecuted-report.json'
                    other.write_bytes((directory / 'stdout').read_bytes())
                    result['stdout'] = runtime.artifact(other)
                runtime.record(directory / 'result.json', result)
                with self.assertRaises(ValueError):
                    self.accept(case)


if __name__ == "__main__":
    unittest.main()
