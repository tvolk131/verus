"""A bounded insert-key link, not verification of async dispatch or a backend."""

import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "evidence/transfer-fedimint-insert-link80a"
NATIVE = Path("/private/tmp/verus-fedimint-early-returns-jyfO2M/fedimint")
RUNTIME = Path("/private/tmp/verus-prefix-infer79-impl.Ks6IeN/delivery/runtime78")
SYSROOT = Path("/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin")
BASELINE = Path("/private/tmp/verus-prefix-infer79.1eVEuO/final-package")
PIN = "b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236"
DATABASE = "fedimint-core/src/db/mod.rs"
OBLIGATIONS = ("soundness", "maximality", "exactness")
TIMINGS = ("generation_micros", "solver_micros", "total_micros")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(content):
    return hashlib.sha256(content).hexdigest()


def digest(path):
    return sha256(Path(path).read_bytes())


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON field: " + key)
        result[key] = value
    return result


def read_json(path):
    return json.loads(Path(path).read_text(), object_pairs_hook=unique_object)


def write_json(path, value):
    Path(path).write_text(json.dumps(value, sort_keys=True, indent=2) + "\n")


def identical(left, right):
    if type(left) is not type(right):
        return False
    if type(left) is dict:
        return left.keys() == right.keys() and all(identical(left[key], right[key]) for key in left)
    if type(left) is list:
        return len(left) == len(right) and all(identical(first, second) for first, second in zip(left, right))
    return left == right


def environment(output):
    result = {key: value for key, value in os.environ.items()
              if not key.startswith(("GIT_", "CARGO_", "RUST", "VERUS_", "VARGO"))}
    result.update(GIT_OPTIONAL_LOCKS="0", VERUS_Z3_PATH=str(RUNTIME / "z3"),
                  RUSTC=str(SYSROOT / "bin/rustc"), RUSTUP_TOOLCHAIN=str(SYSROOT),
                  DYLD_LIBRARY_PATH=str(SYSROOT / "lib"), PYTHONDONTWRITEBYTECODE="1",
                  VERUS_INFER_CACHE=str(output / "infer-cache"), TMPDIR=str(output / "tmp"))
    return result


def native_state(env):
    result = {}
    for name, arguments in (("head", ["rev-parse", "HEAD"]), ("status", ["status", "--porcelain"])):
        run = subprocess.run(["git", "--no-optional-locks", "-C", str(NATIVE)] + arguments,
                             env=env, capture_output=True, text=True, check=True)
        result[name] = run.stdout.strip()
    require(result == {"head": PIN, "status": ""}, "native checkout is not the clean pin")
    return result


def item(source, marker):
    start = source.index(marker)
    opening = source.index("{", start)
    depth = 1
    ending = opening + 1
    while depth:
        require(ending < len(source), "unterminated source item")
        depth += (source[ending] == "{") - (source[ending] == "}")
        ending += 1
    return source[start:ending]


def extract():
    source = (NATIVE / DATABASE).read_text()
    transaction = source[source.index("impl<Inner> PrefixDatabaseTransaction<Inner>"):]
    core = source[source.index("impl<Inner> IDatabaseTransactionOpsCore for PrefixDatabaseTransaction<Inner>"):]
    trait = source[source.index("pub trait IDatabaseTransactionOpsCore: MaybeSend") :]
    marker = "    async fn raw_insert_bytes("
    start = trait.index(marker)
    declaration = trait[start:trait.index(";", start) + 1]
    return {
        "record": item(source, "struct PrefixDatabaseTransaction<Inner>"),
        "token": "pub struct GlobalDBTxAccessToken(u32);",
        "key": item(transaction, "    fn get_full_key("),
        "insert": item(core, marker),
        "declaration": declaration,
    }


def validate_extraction(pieces):
    ledger = read_json(EVIDENCE / "source-ledger.json")
    require(ledger["pin"] == PIN, "source pin")
    for path, expected in ledger["files"].items():
        require((NATIVE / path).is_file() and digest(NATIVE / path) == expected, "native source hash: " + path)
    require(identical({name: sha256(value.encode()) for name, value in pieces.items()}, ledger["items"]),
            "complete extracted items")
    require(pieces["token"] in (NATIVE / DATABASE).read_text(), "native token")


def provider_source(pieces):
    source = "use vstd::prelude::*;\nverus! {\n" + pieces["token"] + "\n" + pieces["record"]
    source += "\nimpl<Inner> PrefixDatabaseTransaction<Inner> {\n#[verifier::infer]\n" + pieces["key"] + "\n}\n"
    source += """
#[verifier::infer]
pub fn key_contents(prefix: &Vec<u8>, key: &[u8]) -> Vec<u8> {
    let database = PrefixDatabaseTransaction {
        inner: 0u8,
        global_dbtx_access_token: None,
        prefix: prefix.clone(),
    };
    database.get_full_key(key)
}
"""
    return source + (EVIDENCE / "footprint.verus.rs").read_text() + "\n}\n"


def native_source(pieces):
    source = (EVIDENCE / "native-harness.rs.in").read_text()
    for name in ("record", "key", "declaration", "insert"):
        source = source.replace("@@" + name.upper() + "@@", pieces[name])
        require(source.count(pieces[name]) == 1, "native extraction occurrence: " + name)
    require("@@" not in source, "unexpanded native source")
    return source


def async_source(pieces):
    return ("use vstd::prelude::*;\nverus! {\ntype DatabaseResult<Value> = Result<Value, u8>;\n"
            + pieces["token"] + "\n" + pieces["record"]
            + "\nimpl<Inner> PrefixDatabaseTransaction<Inner> {\n#[verifier::infer]\n" + pieces["key"]
            + "\n}\npub trait MaybeSend {}\npub trait IDatabaseTransactionOpsCore: MaybeSend {\n"
            + pieces["declaration"] + "\n}\nimpl<Inner> IDatabaseTransactionOpsCore for PrefixDatabaseTransaction<Inner>\n"
            + "where Inner: IDatabaseTransactionOpsCore, {\n" + pieces["insert"] + "\n}\n}\n")


def command(source, output, crate, imports=()):
    result = [str(RUNTIME / "verus"), str(source), "--sysroot", str(SYSROOT), "--crate-type", "lib",
              "--crate-name", crate, "--edition=2021", "--infer-goals", "--no-cheating", "--output-json",
              "--num-threads", "1", "--compile", "--export", str(output / (crate + ".vir")),
              "-o", str(output / ("lib" + crate + ".rlib"))]
    for kind, name, path in imports:
        result += [kind, name + "=" + str(path)]
    return result


def invoke(output, label, source, crate, env, imports=()):
    directory = output / label
    directory.mkdir()
    path = directory / "source.rs"
    path.write_text(source)
    argv = command(path, directory, crate, imports)
    run = subprocess.run(argv, env=env, capture_output=True, timeout=180)
    (directory / "stdout.json").write_bytes(run.stdout)
    (directory / "stderr.txt").write_bytes(run.stderr)
    write_json(directory / "invocation.json", {"argv": argv, "returncode": run.returncode})
    print(label, run.returncode, run.stderr.decode(errors="replace")[-1000:], flush=True)
    return run, directory


def normalize(value, source, candidates=None):
    if type(value) is dict:
        result = {}
        for key, entry in value.items():
            if key in TIMINGS:
                continue
            if key == "candidate_id":
                require(type(entry) is str and candidates is not None and entry in candidates, "candidate digest binding")
                result[key] = candidates[entry]
                continue
            if key == "source":
                require(type(entry) is str and entry.startswith(str(source) + ":"), "source location binding")
            result[key] = normalize(entry, source, candidates)
        return result
    if type(value) is list:
        return [normalize(entry, source, candidates) for entry in value]
    if type(value) is str:
        return value.replace(str(source), "{source}")
    return value


def report_shape(data, source):
    inference = copy.deepcopy(data["verus-infer"])
    candidates = {}
    for function in inference["functions"]:
        candidate = function["candidate_id"]
        require(type(candidate) is str and re.fullmatch(r"[0-9a-f]{64}", candidate) is not None, "candidate digest syntax")
        require(candidate not in candidates, "unique candidate identities")
        candidates[candidate] = function["function"]
    for dependency in inference["imported_inference_dependencies"]:
        fingerprint = dependency["fingerprint"]
        require(type(fingerprint) is str and re.fullmatch(r"[0-9a-f]{64}", fingerprint) is not None, "import fingerprint syntax")
        dependency["fingerprint"] = "{provider-certificate}"
    return normalize({"verification-results": data["verification-results"], "verus-infer": inference}, source, candidates)


def validate_shape(actual, expected):
    require(identical(actual, expected), "typed results, contracts, imports, models or complete scope differ")


def validate_command(actual, expected):
    require(identical(actual, expected), "exact command, separate import kinds and flags required")


def validate_metrics(data, positive):
    inference = data["verus-infer"]
    if positive:
        metrics = inference["metrics"]
        require(type(metrics["total_micros"]) is int and metrics["total_micros"] > 0, "positive total work")
        require(type(metrics["generation_micros"]) is int and metrics["generation_micros"] >= 0, "generation work")
        for function in inference["functions"]:
            for field in ("generation_micros", "solver_micros"):
                work = function["metrics"][field]
                require(type(work) is int and work > 0, "fresh inferred function work: " + field)


def certificate(blob):
    require(blob.endswith(b"VRINFER1"), "inference certificate trailer")
    start = blob.rfind(b'{"schema":')
    require(start >= 0, "inference certificate JSON")
    content = blob[start:-8]
    value = json.loads(content, object_pairs_hook=unique_object)
    return value, sha256(content)


def validate_artifacts(vir, rlib, expected, hashes=None, dependencies=()):
    require(vir.is_file() and not vir.is_symlink(), "regular VIR artifact required")
    require(rlib.is_file() and not rlib.is_symlink(), "regular rlib artifact required")
    require(rlib.read_bytes().startswith(b"!<arch>\n") and rlib.stat().st_size > 8, "rlib archive kind")
    metadata, fingerprint = certificate(vir.read_bytes())
    validate_shape(metadata["dependencies"], list(dependencies))
    validate_shape(certificate_shape(metadata), expected)
    actual = {"vir_sha256": digest(vir), "rlib_sha256": digest(rlib), "certificate_sha256": fingerprint}
    if hashes is not None:
        validate_shape(actual, hashes)
    return actual


def certificate_shape(metadata):
    result = copy.deepcopy(metadata)
    summaries = result["summaries"]
    require(type(summaries) is str and re.fullmatch(r"[0-9a-f]{64}", summaries) is not None, "summary digest syntax")
    result["summaries"] = "{source-dependent-summary}"
    for dependency in result["dependencies"]:
        fingerprint = dependency["fingerprint"]
        require(type(fingerprint) is str and re.fullmatch(r"[0-9a-f]{64}", fingerprint) is not None,
                "certificate dependency digest syntax")
        dependency["fingerprint"] = "{provider-certificate}"
    return result


def consumer_certificate(fixture):
    result = copy.deepcopy(fixture["certificate"])
    result["proofs"] = []
    result["dependencies"] = copy.deepcopy(fixture["reports"]["consumer"]["verus-infer"]["imported_inference_dependencies"])
    return result


def import_dependencies(vir):
    metadata, fingerprint = certificate(vir.read_bytes())
    identities = {proof[0]["path"]["krate"]["Id"][1] for proof in metadata["proofs"]}
    require(len(identities) == 1 and all(type(identity) is int for identity in identities), "import crate identity")
    return [{"crate_id": next(iter(identities)), "fingerprint": fingerprint}]


def validate_outcome(data, code, source, label, stderr, export_exists, dependencies=()):
    fixtures = read_json(EVIDENCE / "receipt-shapes.json")
    positive = label in ("provider", "warm-provider", "consumer")
    require(type(code) is int and (code == 0 if positive else code != 0), "typed compiler exit code")
    require(type(export_exists) is bool and export_exists is positive, "proof artifact publication/cleanup")
    require(label != "consumer" or type(dependencies) is list and len(dependencies) == 1, "paired provider certificate required")
    validate_shape(data["verus-infer"]["imported_inference_dependencies"], list(dependencies))
    expected = fixtures["reports"]["provider" if label == "warm-provider" else label]
    validate_shape(report_shape(data, source), expected)
    validate_metrics(data, positive)
    if not positive:
        diagnostic, function = fixtures["diagnostics"][label]
        require(type(stderr) is str and diagnostic in stderr, "expected failure category")
        if function:
            text = source.read_text()
            declaration = text.index("fn " + function + "(")
            end = declaration + len(item(text[declaration:], "fn " + function + "("))
            first_line = text.count("\n", 0, declaration) + 1
            last_line = text.count("\n", 0, end) + 1
            locations = re.findall(re.escape(str(source)) + r":(\d+):\d+", stderr)
            require(any(first_line <= int(line) <= last_line for line in locations), "failure function location")


def verifier_controls(source):
    return {
        "unprefixed-target": source.replace("let target = key_contents(prefix, key);", "let target = key_contents(&Vec::new(), key);"),
        "missing-backend-premise": source.replace(
            "requires backend_insert(before, after, prefix + key, value),",
            "requires true,", 1),
        "missing-namespace-premise": source.replace(
            "requires incomparable(prefix, peer), backend_insert(before, after, prefix + key, value),",
            "requires backend_insert(before, after, prefix + key, value),", 1),
        "unsafe-index": source.replace("let target = key_contents(prefix, key);",
                                       "let _byte = key[key.len()];\n    let target = key_contents(prefix, key);"),
    }


def native_controls(source, pieces):
    selected = pieces["insert"]
    call = "self.inner.raw_insert_bytes(&key, value).await"
    return {
        "native-unprefixed": source.replace(selected, selected.replace("self.get_full_key(key)", "key.to_vec()")),
        "native-wrong-value": source.replace(selected, selected.replace(call, "self.inner.raw_insert_bytes(&key, &[]).await")),
        "native-double-dispatch": source.replace(selected, selected.replace(call, "let _ = " + call + ";\n        " + call)),
        "native-wrong-return": source.replace(selected, selected.replace(call, call + ".map(|_| None)")),
    }


def validate_native_test(code, stdout, positive):
    require(type(positive) is bool, "typed native polarity")
    require(type(code) is int and (code == 0 if positive else code != 0), "typed native test exit")
    require(type(stdout) is str and "running 6 tests\n" in stdout, "native test selection")
    if positive:
        for name in ("arbitrary_prefixes_can_overlap", "cancellation_does_not_imply_rollback",
                     "error_result_does_not_imply_rollback", "forwarding_matrix",
                     "unconstrained_backend_can_escape_namespace", "unpolled_future_has_no_dispatch"):
            require("test " + name + " ... ok\n" in stdout, "native test: " + name)
        require("test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;" in stdout,
                "complete native test count")
    else:
        require("test forwarding_matrix ... FAILED\n" in stdout and "test result: FAILED." in stdout,
                "semantic forwarding control did not fail")


def validate_build(code, binary_exists):
    require(type(code) is int and code == 0 and binary_exists is True, "native harness compilation")


def native_run(output, label, source, env, optimization, positive):
    directory = output / label
    directory.mkdir()
    path = directory / "source.rs"
    path.write_text(source)
    binary = directory / "tests"
    argv = [str(SYSROOT / "bin/rustc"), "--edition=2021", "--test", str(path),
            "-C", "opt-level=" + str(optimization), "-o", str(binary)]
    built = subprocess.run(argv, env=env, capture_output=True, timeout=120)
    (directory / "build.stdout.txt").write_bytes(built.stdout)
    (directory / "build.stderr.txt").write_bytes(built.stderr)
    validate_build(built.returncode, binary.is_file())
    test_argv = [str(binary), "--test-threads=1", "--nocapture"]
    tested = subprocess.run(test_argv, env=env, capture_output=True, timeout=120)
    (directory / "test.stdout.txt").write_bytes(tested.stdout)
    (directory / "test.stderr.txt").write_bytes(tested.stderr)
    validate_native_test(tested.returncode, tested.stdout.decode(), positive)
    row = {"label": label, "source_sha256": digest(path), "build_command": argv,
           "test_command": test_argv, "build_returncode": built.returncode, "returncode": tested.returncode,
           "test_binary_sha256": digest(binary), "positive": positive}
    write_json(directory / "invocation.json", row)
    print(label, tested.returncode, flush=True)
    return row


def verify_hashes(expected):
    require(type(expected) is dict and bool(expected), "nonempty guarded inventory")
    for path, expected_hash in expected.items():
        require(type(path) is str and type(expected_hash) is str
                and re.fullmatch(r"[0-9a-f]{64}", expected_hash) is not None, "typed guarded digest")
        require(Path(path).is_file() and digest(path) == expected_hash, "guarded original input changed: " + path)
    return len(expected)


def original_guards():
    result = read_json(BASELINE / "guarded-inputs.json")
    require(len(result) == 10782, "accepted original-input count")
    for path in BASELINE.iterdir():
        if path.is_file():
            result[str(path)] = digest(path)
    for path, expected in read_json(BASELINE / "runtime-hashes.json").items():
        result[str(RUNTIME / path)] = expected
    for path in EVIDENCE.iterdir():
        if path.is_file():
            result[str(path)] = digest(path)
    result[str(Path(__file__).resolve())] = digest(__file__)
    test = Path(__file__).with_name("test_fedimint_insert_link80a.py")
    result[str(test.resolve())] = digest(test)
    return result


def baseline_guards():
    result = read_json(BASELINE / "file-hashes.json")
    require(len(result) == 2145, "baseline path count")
    return {str(ROOT / path): value for path, value in result.items()}


def replay(output, optimization=0):
    output = output.resolve()
    output.mkdir()
    (output / "tmp").mkdir()
    env = environment(output)
    originals = original_guards()
    baseline = baseline_guards()
    write_json(output / "original-inputs.json", originals)
    write_json(output / "baseline-inputs.json", baseline)
    receipt = {"identity": [78, 72, 38], "scope": "conditional insert-key link; not async/backend refinement",
               "originals_before": verify_hashes(originals), "baseline_before": verify_hashes(baseline),
               "native_before": native_state(env), "cases": []}
    pieces = extract()
    validate_extraction(pieces)
    source = provider_source(pieces)
    native = native_source(pieces)
    fixture = read_json(EVIDENCE / "receipt-shapes.json")
    validate_shape({"provider": sha256(source.encode()),
                    "consumer": digest(EVIDENCE / "consumer.verus.rs"),
                    "native": sha256(native.encode()), "async-blocker": sha256(async_source(pieces).encode()),
                    "controls": {name: sha256(text.encode()) for name, text in verifier_controls(source).items()},
                    "native_controls": {name: sha256(text.encode()) for name, text in native_controls(native, pieces).items()}},
                   fixture["sources"])
    artifact_hashes = None
    provider = None
    for label, text, crate, imports in (
        ("provider", source, "insert_link80a", ()),
        ("warm-provider", source, "insert_link80a", ()),
        ("consumer", (EVIDENCE / "consumer.verus.rs").read_text(), "insert_consumer80a",
         (("--import", "insert_link80a", output / "provider/insert_link80a.vir"),
          ("--extern", "insert_link80a", output / "provider/libinsert_link80a.rlib"))),
    ):
        run, directory = invoke(output, label, text, crate, env, imports)
        path = directory / "source.rs"
        argv = read_json(directory / "invocation.json")["argv"]
        validate_command(argv, command(path, directory, crate, imports))
        vir = directory / (crate + ".vir")
        rlib = directory / ("lib" + crate + ".rlib")
        data = read_json(directory / "stdout.json")
        dependencies = import_dependencies(provider / "insert_link80a.vir") if label == "consumer" else ()
        validate_outcome(data, run.returncode, path, label, run.stderr.decode(), vir.is_file(), dependencies)
        if label != "consumer":
            hashes = validate_artifacts(vir, rlib, fixture["certificate"])
        else:
            hashes = validate_artifacts(vir, rlib, consumer_certificate(fixture), dependencies=dependencies)
            validate_artifacts(provider / "insert_link80a.vir", provider / "libinsert_link80a.rlib",
                               fixture["certificate"], artifact_hashes)
        if label == "provider":
            provider, artifact_hashes = directory, hashes
        receipt["cases"].append({"label": label, "returncode": run.returncode, "argv": argv,
                                 "source_sha256": digest(path), "artifacts": hashes})
    controls = verifier_controls(source)
    controls["async-blocker"] = async_source(pieces)
    for label, text in controls.items():
        directory = output / label
        directory.mkdir()
        path = directory / "source.rs"
        path.write_text(text)
        vir = directory / "insert_link80a.vir"
        vir.write_bytes((provider / "insert_link80a.vir").read_bytes())
        seeded = digest(vir)
        argv = command(path, directory, "insert_link80a")
        run = subprocess.run(argv, env=env, capture_output=True, timeout=180)
        (directory / "stdout.json").write_bytes(run.stdout)
        (directory / "stderr.txt").write_bytes(run.stderr)
        validate_command(argv, command(path, directory, "insert_link80a"))
        validate_outcome(read_json(directory / "stdout.json"), run.returncode, path, label,
                         run.stderr.decode(), vir.exists())
        row = {"label": label, "returncode": run.returncode, "argv": argv,
               "source_sha256": digest(path), "seeded_vir_sha256": seeded, "export_absent": not vir.exists()}
        write_json(directory / "invocation.json", row)
        receipt["cases"].append(row)
        print(label, run.returncode, flush=True)
    receipt["cases"].append(native_run(output, "native", native, env, optimization, True))
    for label, text in native_controls(native, pieces).items():
        receipt["cases"].append(native_run(output, label, text, env, optimization, False))
    receipt["originals_after"] = verify_hashes(originals)
    receipt["baseline_after"] = verify_hashes(baseline)
    receipt["native_after"] = native_state(env)
    expected_labels = ["provider", "warm-provider", "consumer"] + list(controls) + ["native"] + list(native_controls(native, pieces))
    validate_shape(expected_labels, fixture["labels"])
    validate_shape([row["label"] for row in receipt["cases"]], expected_labels)
    require(len(receipt["cases"]) == 13, "complete 13-case replay")
    receipt["logs"] = {str(path.relative_to(output)): digest(path) for path in output.rglob("*")
                       if path.is_file() and path.suffix in (".json", ".txt", ".rs") and "infer-cache" not in path.parts}
    receipt["complete"] = True
    write_json(output / "receipt.json", receipt)
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--rust-opt", type=int, choices=(0, 2), default=0)
    arguments = parser.parse_args()
    replay(arguments.output, arguments.rust_opt)


if __name__ == "__main__":
    main()
