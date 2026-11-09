import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_fedimint_insert_link80a as link


def corruptions(value, path=()):
    if type(value) is dict:
        yield path, []
        yield path + ("extra",), dict(value, unexpected_field=None)
        for key, entry in value.items():
            changed = dict(value)
            del changed[key]
            yield path + (key, "missing"), changed
            for location, replacement in corruptions(entry, path + (key,)):
                changed = dict(value)
                changed[key] = replacement
                yield location, changed
    elif type(value) is list:
        yield path, {}
        yield path + ("extra",), value + [None]
        for index, entry in enumerate(value):
            yield path + (index, "missing"), value[:index] + value[index + 1:]
            for location, replacement in corruptions(entry, path + (index,)):
                changed = list(value)
                changed[index] = replacement
                yield location, changed
    elif type(value) is bool:
        yield path, not value
        yield path, int(value)
    elif type(value) is int:
        yield path, value + 1
        yield path, bool(value)
    elif type(value) is str:
        yield path, value + "!"
        yield path, None
    else:
        yield path, False


def materialize(value, source, identities):
    if type(value) is dict:
        result = {}
        for key, entry in value.items():
            if key == "candidate_id":
                result[key] = identities[entry]
            else:
                result[key] = materialize(entry, source, identities)
        return result
    if type(value) is list:
        return [materialize(entry, source, identities) for entry in value]
    if type(value) is str:
        return value.replace("{source}", str(source)).replace("{provider-certificate}", "a" * 64)
    return value


class InsertLink80aTests(unittest.TestCase):
    negatives = 0

    @classmethod
    def setUpClass(cls):
        cls.fixture = link.read_json(link.EVIDENCE / "receipt-shapes.json")
        cls.native_directory = tempfile.TemporaryDirectory(prefix="insert-link80a-unit-source-")
        cls.addClassCleanup(cls.native_directory.cleanup)
        cls.native = Path(cls.native_directory.name)
        cls.native_source = (Path(__file__).parent / "fixtures/fedimint_insert_link80a.rs").read_text()
        ledger = link.read_json(link.EVIDENCE / "source-ledger.json")
        for name in ledger["files"]:
            path = cls.native / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(cls.native_source if name == link.DATABASE else "unit fixture: " + name)
        with patch.object(link, "NATIVE", cls.native):
            cls.pieces = link.extract()
        cls.native_ledger = {
            "pin": link.PIN,
            "files": {name: link.digest(cls.native / name) for name in ledger["files"]},
            "items": ledger["items"],
        }
        if {name: link.sha256(value.encode()) for name, value in cls.pieces.items()} != ledger["items"]:
            raise AssertionError("unit excerpts differ from the pinned item ledger")

    @classmethod
    def tearDownClass(cls):
        print("INSERT_LINK80A_REJECTED_CONTROLS=" + str(cls.negatives))

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="insert-link80a-test-")
        self.directory = Path(self.temporary.name)
        self.source = self.directory / "source.rs"
        self.source.write_text(link.provider_source(self.pieces))

    def tearDown(self):
        self.temporary.cleanup()

    def reject(self, function, *arguments):
        type(self).negatives += 1
        with self.assertRaises((ValueError, KeyError, TypeError, AttributeError)):
            function(*arguments)

    def raw_report(self, label="provider"):
        shape = self.fixture["reports"][label]
        identities = {entry["function"]: link.sha256(entry["function"].encode())
                      for entry in shape["verus-infer"]["functions"]}
        data = materialize(shape, self.source, identities)
        inference = data["verus-infer"]
        inference["metrics"].update(generation_micros=1, total_micros=1)
        for function in inference["functions"]:
            function["metrics"].update(generation_micros=1, solver_micros=1)
        return data

    def test_every_locked_receipt_and_certificate_condition(self):
        targets = dict(self.fixture["reports"])
        targets.update(certificate=self.fixture["certificate"], consumer_certificate=link.consumer_certificate(self.fixture),
                       sources=self.fixture["sources"], labels=self.fixture["labels"])
        for name, expected in targets.items():
            link.validate_shape(expected, expected)
            for location, changed in corruptions(expected):
                with self.subTest(target=name, path=location):
                    self.reject(link.validate_shape, changed, expected)

    def test_outcome_checks_complete_report(self):
        data = self.raw_report()
        link.validate_outcome(data, 0, self.source, "provider", "", True)
        mutations = (
            lambda changed: changed["verification-results"].update(success=False),
            lambda changed: changed["verification-results"].update(verified=True),
            lambda changed: changed["verus-infer"]["verification_scope"].update(complete_selected_scope_verified=False),
            lambda changed: changed["verus-infer"]["functions"][0]["proofs"]["soundness"].update(required=False),
        )
        for mutation in mutations:
            changed = copy.deepcopy(data)
            mutation(changed)
            with self.subTest(mutation=mutation), self.assertRaisesRegex(ValueError,
                    "^typed results, contracts, imports, models or complete scope differ$"):
                link.validate_outcome(changed, 0, self.source, "provider", "", True)

    def test_outcome_checks_fresh_work(self):
        data = self.raw_report()
        link.validate_outcome(data, 0, self.source, "provider", "", True)
        changed = copy.deepcopy(data)
        changed["verus-infer"]["metrics"]["total_micros"] = 0
        with self.assertRaisesRegex(ValueError, "^positive total work$"):
            link.validate_outcome(changed, 0, self.source, "provider", "", True)
        changed = copy.deepcopy(data)
        changed["verus-infer"]["functions"][0]["metrics"]["solver_micros"] = 0
        with self.assertRaisesRegex(ValueError, "^fresh inferred function work: solver_micros$"):
            link.validate_outcome(changed, 0, self.source, "provider", "", True)

    def test_outcome_exit_publication_and_import_binding(self):
        for label in self.fixture["reports"]:
            positive = label in ("provider", "consumer")
            data = self.raw_report(label)
            text = (link.EVIDENCE / "consumer.verus.rs").read_text() if label == "consumer" else link.provider_source(self.pieces)
            if label in link.verifier_controls(text):
                text = link.verifier_controls(text)[label]
            if label == "async-blocker":
                text = link.async_source(self.pieces)
            self.source.write_text(text)
            diagnostic, function = self.fixture["diagnostics"].get(label, ("", ""))
            line = text.count("\n", 0, text.index("fn " + function + "(")) + 1 if function else 1
            stderr = diagnostic + "\n --> " + str(self.source) + ":" + str(line) + ":1\n"
            code = 0 if positive else 1
            dependencies = data["verus-infer"]["imported_inference_dependencies"] if label == "consumer" else ()
            args = [data, code, self.source, label, stderr, positive, dependencies]
            link.validate_outcome(*args)
            for invalid in ([True, False, 0.0, "0", 1] if positive else [0, True, "1", 1.0]):
                changed = list(args)
                changed[1] = invalid
                self.reject(link.validate_outcome, *changed)
            for invalid in (not positive, int(positive), None):
                changed = list(args)
                changed[5] = invalid
                self.reject(link.validate_outcome, *changed)
            if not positive:
                for invalid in ("", diagnostic + "\n --> elsewhere.rs:1:1\n", diagnostic + "\n --> " + str(self.source) + ":99999:1\n"):
                    if not function and invalid:
                        continue
                    changed = list(args)
                    changed[4] = invalid
                    self.reject(link.validate_outcome, *changed)
            if label == "consumer":
                for invalid in ((), [], [{"crate_id": True, "fingerprint": "a" * 64}],
                                [{"crate_id": dependencies[0]["crate_id"], "fingerprint": "b" * 64}]):
                    changed = list(args)
                    changed[6] = invalid
                    self.reject(link.validate_outcome, *changed)

    def test_dynamic_work_and_candidate_guards(self):
        original = self.raw_report()
        for container, fields in (("top", ("total_micros", "generation_micros")),
                                  ("function", ("solver_micros", "generation_micros"))):
            for field in fields:
                for invalid in (True, False, -1, 1.5, "1", None, 0):
                    if container == "top" and field == "generation_micros" and invalid == 0 and type(invalid) is int:
                        continue
                    data = copy.deepcopy(original)
                    target = data["verus-infer"]["metrics"] if container == "top" else data["verus-infer"]["functions"][0]["metrics"]
                    target[field] = invalid
                    self.reject(link.validate_metrics, data, True)
        link.validate_metrics(original, True)
        for invalid in (True, None, "0", "g" * 64, "a" * 63):
            data = copy.deepcopy(original)
            data["verus-infer"]["functions"][0]["candidate_id"] = invalid
            self.reject(link.report_shape, data, self.source)
        data = copy.deepcopy(original)
        data["verus-infer"]["functions"][1]["candidate_id"] = data["verus-infer"]["functions"][0]["candidate_id"]
        self.reject(link.report_shape, data, self.source)
        data = copy.deepcopy(original)
        data["verus-infer"]["functions"][1]["dependencies"][0]["candidate_id"] = "f" * 64
        self.reject(link.report_shape, data, self.source)
        data = copy.deepcopy(original)
        data["verus-infer"]["functions"][1]["dependencies"][0]["candidate_id"] = data["verus-infer"]["functions"][1]["candidate_id"]
        self.reject(lambda: link.validate_shape(link.report_shape(data, self.source), self.fixture["reports"]["provider"]))
        for invalid in (None, "elsewhere.rs:1:1", "{source}:11:5: 11:50"):
            data = copy.deepcopy(original)
            data["verus-infer"]["verification_scope"]["native_functions"][0]["source"] = invalid
            self.reject(link.report_shape, data, self.source)
        consumer = self.raw_report("consumer")
        for invalid in (False, "z" * 64, "a" * 63, None):
            changed = copy.deepcopy(consumer)
            changed["verus-infer"]["imported_inference_dependencies"][0]["fingerprint"] = invalid
            self.reject(link.report_shape, changed, self.source)

    def test_command_import_kinds_joined_flags_and_scope_controls(self):
        imports = (("--import", "insert_link80a", self.directory / "provider.vir"),
                   ("--extern", "insert_link80a", self.directory / "provider.rlib"))
        for selected in ((), imports):
            expected = link.command(self.source, self.directory, "insert_consumer80a", selected)
            link.validate_command(expected, expected)
            for location, changed in corruptions(expected):
                with self.subTest(path=location):
                    self.reject(link.validate_command, changed, expected)
            for extra in ("--no-verify", "--verify-function=prepare_insert", "--verify-module=root", "--rlimit=0"):
                self.reject(link.validate_command, expected + [extra], expected)
            for flag in ("--infer-goals", "--no-cheating", "--compile", "--export", "--import", "--extern", "--sysroot"):
                if flag in expected:
                    index = expected.index(flag)
                    self.reject(link.validate_command, expected[:index] + expected[index + 1:], expected)
                    self.reject(link.validate_command, expected + [flag], expected)
                    if flag in ("--import", "--extern", "--export", "--sysroot"):
                        joined = expected[:index] + [flag + "=" + expected[index + 1]] + expected[index + 2:]
                        self.reject(link.validate_command, joined, expected)
            if selected:
                swapped = ["--extern" if entry == "--import" else "--import" if entry == "--extern" else entry for entry in expected]
                self.reject(link.validate_command, swapped, expected)

    def artifacts(self):
        metadata = copy.deepcopy(self.fixture["certificate"])
        metadata["summaries"] = "a" * 64
        metadata = dict(schema=metadata.pop("schema"), **metadata)
        vir, rlib = self.directory / "provider.vir", self.directory / "provider.rlib"
        vir.write_bytes(b"unit-fixture-not-a-real-VIR" + json.dumps(metadata, separators=(",", ":")).encode() + b"VRINFER1")
        rlib.write_bytes(b"!<arch>\nunit-fixture-not-a-real-rlib")
        return vir, rlib

    def test_artifact_kinds_hashes_and_certificate_parser(self):
        vir, rlib = self.artifacts()
        expected = self.fixture["certificate"]
        hashes = link.validate_artifacts(vir, rlib, expected)
        link.validate_artifacts(vir, rlib, expected, hashes)
        for location, changed in corruptions(hashes):
            with self.subTest(path=location):
                self.reject(link.validate_artifacts, vir, rlib, expected, changed)
        self.reject(link.validate_artifacts, rlib, vir, expected)
        self.reject(link.validate_artifacts, self.directory / "absent.vir", rlib, expected)
        self.reject(link.validate_artifacts, vir, self.directory / "absent.rlib", expected)
        original_vir, original_rlib = vir.read_bytes(), rlib.read_bytes()
        for malformed in (b"", original_vir[:-1], b"VRINFER1", b'{"schema":38,"schema":38}VRINFER1', b'{"schema":VRINFER1'):
            vir.write_bytes(malformed)
            self.reject(link.validate_artifacts, vir, rlib, expected)
        vir.write_bytes(original_vir)
        for malformed in (b"", b"!<arch>\n", b"not an archive"):
            rlib.write_bytes(malformed)
            self.reject(link.validate_artifacts, vir, rlib, expected)
        rlib.write_bytes(original_rlib)
        for artifact in (vir, rlib):
            alias = self.directory / (artifact.name + ".link")
            alias.symlink_to(artifact)
            self.reject(link.validate_artifacts, alias if artifact == vir else vir,
                        alias if artifact == rlib else rlib, expected)
        for invalid in (True, "x" * 64, "a" * 63, None):
            metadata = copy.deepcopy(expected)
            metadata["summaries"] = invalid
            self.reject(link.certificate_shape, metadata)
        consumer = link.consumer_certificate(self.fixture)
        self.reject(link.validate_artifacts, vir, rlib, consumer)
        metadata = copy.deepcopy(consumer)
        metadata["summaries"] = "a" * 64
        metadata["dependencies"][0]["fingerprint"] = "b" * 64
        metadata = dict(schema=metadata.pop("schema"), **metadata)
        vir.write_bytes(b"unit-fixture" + json.dumps(metadata, separators=(",", ":")).encode() + b"VRINFER1")
        dependencies = metadata["dependencies"]
        link.validate_artifacts(vir, rlib, consumer, dependencies=dependencies)
        self.reject(link.validate_artifacts, vir, rlib, consumer)
        for invalid in (True, "a" * 63, "g" * 64, None):
            changed = copy.deepcopy(metadata)
            changed["dependencies"][0]["fingerprint"] = invalid
            self.reject(link.certificate_shape, changed)
        for changed_proofs in ([], [[{"path": {"krate": {"Id": ["insert_link80a", True]}}}, {}]]):
            changed = copy.deepcopy(self.fixture["certificate"])
            changed["proofs"] = changed_proofs
            with patch.object(link, "certificate", return_value=(changed, "a" * 64)):
                self.reject(link.import_dependencies, vir)

    def test_native_receipt_guards(self):
        names = ("arbitrary_prefixes_can_overlap", "cancellation_does_not_imply_rollback", "error_result_does_not_imply_rollback",
                 "forwarding_matrix", "unconstrained_backend_can_escape_namespace", "unpolled_future_has_no_dispatch")
        stdout = "running 6 tests\n" + "".join("test " + name + " ... ok\n" for name in names)
        stdout += "test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n"
        link.validate_native_test(0, stdout, True)
        for line in stdout.splitlines(keepends=True):
            self.reject(link.validate_native_test, 0, stdout.replace(line, ""), True)
        for invalid in (1, True, False, "0", 0.0):
            self.reject(link.validate_native_test, invalid, stdout, True)
        for invalid in (1, 0, None, "true"):
            self.reject(link.validate_native_test, 0, stdout, invalid)
        negative = "running 6 tests\ntest forwarding_matrix ... FAILED\ntest result: FAILED.\n"
        link.validate_native_test(101, negative, False)
        for invalid in (0, True, "101", 101.0):
            self.reject(link.validate_native_test, invalid, negative, False)
        for line in negative.splitlines(keepends=True):
            self.reject(link.validate_native_test, 101, negative.replace(line, ""), False)
        link.validate_build(0, True)
        for code, exists in ((True, True), (0.0, True), ("0", True), (1, True), (0, False), (0, 1), (0, None)):
            self.reject(link.validate_build, code, exists)

    def test_source_extraction_and_original_input_guards(self):
        with patch.object(link, "NATIVE", self.native):
            with patch.object(link, "read_json", return_value=self.native_ledger):
                link.validate_extraction(self.pieces)
                for name in self.pieces:
                    changed = dict(self.pieces)
                    changed[name] += " "
                    self.reject(link.validate_extraction, changed)
                self.reject(link.validate_extraction, dict(self.pieces, extra="unselected"))
                with patch.object(Path, "is_file", return_value=False):
                    self.reject(link.validate_extraction, self.pieces)
            for field in ("pin", "files", "items"):
                changed = copy.deepcopy(self.native_ledger)
                changed[field] = "wrong" if field == "pin" else {"wrong": "a" * 64}
                with patch.object(link, "read_json", return_value=changed):
                    self.reject(link.validate_extraction, self.pieces)
            for name in self.native_ledger["files"]:
                changed = copy.deepcopy(self.native_ledger)
                changed["files"][name] = "0" * 64
                with patch.object(link, "read_json", return_value=changed):
                    self.reject(link.validate_extraction, self.pieces)
        path = self.directory / "guarded"
        path.write_text("original")
        expected = {str(path): link.digest(path)}
        self.assertEqual(link.verify_hashes(expected), 1)
        for invalid in ({}, [], {str(path): True}, {str(path): "a"}, {1: "a" * 64}, {str(path): "0" * 64},
                        {str(self.directory / "missing"): "a" * 64}):
            self.reject(link.verify_hashes, invalid)
        for function in (link.original_guards, link.baseline_guards):
            with patch.object(link, "read_json", return_value={"wrong": "a" * 64}):
                self.reject(function)

    def test_json_types_and_sanitized_git_context(self):
        path = self.directory / "data.json"
        for content in ('{"success":true,"success":false}', '{"proof":{"ok":true,"ok":true}}', '{'):
            path.write_text(content)
            self.reject(link.read_json, path)
        with patch.dict(link.os.environ, {"GIT_DIR": "/hostile", "GIT_CONFIG_COUNT": "1", "CARGO_HOME": "/forbidden",
                                         "RUSTFLAGS": "--cfg bypass", "VERUS_Z3_PATH": "/hostile/z3"}):
            env = link.environment(self.directory)
        self.assertEqual({key: value for key, value in env.items() if key.startswith("GIT_")}, {"GIT_OPTIONAL_LOCKS": "0"})
        self.assertNotIn("CARGO_HOME", env)
        self.assertNotIn("RUSTFLAGS", env)
        self.assertEqual(env["VERUS_Z3_PATH"], str(link.RUNTIME / "z3"))
        for head, status in (("wrong", ""), (link.PIN, " M changed")):
            with patch.object(link.subprocess, "run", side_effect=[type("Result", (), {"stdout": head})(),
                                                                    type("Result", (), {"stdout": status})()]):
                self.reject(link.native_state, env)


if __name__ == "__main__":
    unittest.main()
