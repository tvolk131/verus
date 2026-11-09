"""One bounded native-source Infer cycle; authored namespace lemmas remain explicit."""

import argparse
import copy
import hashlib
import json
from pathlib import Path
import re
from types import SimpleNamespace

import check_fedimint_prefix_boundary as boundary
from infer_eval_support import ArtifactGuard, tool_artifacts
from infer_transfer_identity import CACHE_VERSION, ENGINE_IDENTITY as IDENTITY, INTERFACE_POLICY as POLICY
from workflow_runtime import (artifact, bind_execution, digest, environment as shared_environment,
    execute, identical, read_json, validate_command as shared_command, validate_outcome, validate_timed)


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "evidence/transfer-fedimint-prefix-infer79"
OBLIGATIONS = ("soundness", "maximality", "exactness")
require = boundary.require
write_json = boundary.write_json
sha256 = boundary.sha256


def render(pieces, goals=True):
    source = "use vstd::prelude::*;\nuse std::fmt::Debug;\nuse std::ops::Range;\nverus! {\n"
    source += "\n".join(pieces[name] for name in ("token", "database_record", "transaction_record")) + "\n"
    for owner, names in (("PrefixDatabase", ("database_key",)),
                         ("PrefixDatabaseTransaction", ("transaction_key", "transaction_range"))):
        source += "impl<Inner" + (": Debug" if owner == "PrefixDatabase" else "") + "> " + owner + "<Inner> {\n"
        source += "\n".join("#[verifier::infer]\n" + pieces[name] for name in names) + "\n}\n"
    source += (EVIDENCE / "wrappers.verus.rs").read_text() + "\n"
    if goals:
        source += (boundary.EVIDENCE / "prefix.spec.rs").read_text() + "\n"
        source += (boundary.EVIDENCE / "goals.verus.rs").read_text().replace("pub fn ", "fn ") + "\n"
        source += (EVIDENCE / "endpoint-goal.verus.rs").read_text() + "\n"
    return source + "}\n"


def correspondence(source, pieces):
    for name in ("database_record", "transaction_record", "token"):
        require(source.count(pieces[name]) == 1, "native declaration changed: " + name)
    result = {}
    for name, (owner, method) in boundary.NATIVE.items():
        start = source.index("> " + owner + "<Inner> {")
        end = source.find("\nimpl<", start)
        selected = boundary.item(source[start:None if end < 0 else end], "    fn " + method + "(")
        require(selected == pieces[name], "native item bytes changed: " + name)
        require("#[verifier::infer]\n" + selected in source, "native inference annotation missing")
        result[name] = dict(item_sha256=sha256(selected.encode()),
                            body_sha256=sha256(boundary.body(selected).encode()), byte_identical=True)
    return result


def command(verus, source, sysroot, crate, vir, rlib, imports=()):
    result = [str(verus), str(source), "--sysroot", str(sysroot), "--crate-type", "lib",
              "--crate-name", crate, "--edition=2021", "--infer-goals", "--no-cheating",
              "--output-json", "--num-threads", "1", "--compile", "--export", str(vir), "-o", str(rlib)]
    for name, provider_vir, provider_rlib in imports:
        result += ["--import", name + "=" + str(provider_vir), "--extern", name + "=" + str(provider_rlib)]
    return result


def validate_command(actual, *arguments):
    require(type(actual) is list and all(type(value) is str for value in actual), "command type")
    shared_command(actual, command(*arguments), "unrecorded, joined, duplicated, or scope-changing command options")


def environment_for(output, solver, sysroot):
    for name in ("home", "cache", "cargo-home", "rustup-home", "tmp"):
        (output / name).mkdir()
    return shared_environment(dict(HOME=str(output / "home"), XDG_CACHE_HOME=str(output / "cache"),
        CARGO_HOME=str(output / "cargo-home"), RUSTUP_HOME=str(output / "rustup-home"),
        CARGO_TARGET_DIR=str(output / "unused-target"), TMPDIR=str(output / "tmp"),
        GIT_OPTIONAL_LOCKS="0", CARGO_NET_OFFLINE="true", PYTHONDONTWRITEBYTECODE="1",
        VERUS_USE_RUSTUP="0", VERUS_Z3_PATH=str(solver), DYLD_LIBRARY_PATH=str(sysroot / "lib")))


def bound_case(directory, argv, source, text, environment, inputs):
    require(argv[1] == str(source) and source in inputs, "command/source split")
    require(digest(source) == sha256(text.encode()), "checked source changed during run")
    result = bind_execution(directory, argv, source.parent, environment, inputs=inputs)
    data = read_json(directory / "stdout")
    return data, SimpleNamespace(returncode=result["returncode"],
        stdout=(directory / "stdout").read_bytes(), stderr=(directory / "stderr").read_bytes()), result["seconds"]


def exact(data, expected_contracts, cache=None, exported=True):
    inference = data["verus-infer"]
    require(type(inference.get("schema_version")) is int and inference["schema_version"] == 1, "report schema")
    require(inference.get("generation_failure") is None, "generation failure")
    require(type(inference.get("requested_functions")) is int
            and inference["requested_functions"] == len(expected_contracts), "requested function count")
    require(type(inference.get("generated_functions")) is int
            and inference["generated_functions"] == len(expected_contracts), "generated function count")
    functions = inference.get("functions")
    require(type(functions) is list and len(functions) == len(expected_contracts), "function rows")
    observed = {}
    for function in functions:
        name = function["function"].split("::", 1)[-1]
        require(name not in observed, "duplicate inferred function")
        observed[name] = function["contract"]
        require(function.get("mode") == "exact" and function.get("required_checks_passed") is True, "exact inference gates")
        public = name in ("key_contents", "range_contents", "composed")
        require(function.get("exported") is (exported and public), "function publication")
        if exported:
            require(function.get("available_to_same_crate_callers") is True, "same-crate availability")
        for obligation in OBLIGATIONS:
            proof = function["proofs"][obligation]
            require(proof.get("status") == "proved" and proof.get("required") is True, "required proof: " + obligation)
        metrics = function["metrics"]
        require(type(metrics.get("solver_micros")) is int and metrics["solver_micros"] > 0, "fresh solver work")
        require(metrics.get("cache_policy") == "regenerated_state", "owned candidates must regenerate")
        if cache is not None:
            require(metrics.get("cache") == cache, "cache phase")
    require(identical(observed, expected_contracts), "exact generated contract differs")
    return observed


def scope(data, expected, source_path):
    actual = data["verus-infer"]["verification_scope"]
    target = copy.deepcopy(expected)
    for row in target["native_functions"]:
        row["source"] = row["source"].replace("{source}", str(source_path))
    require(identical(actual, target), "full selected-body verification scope differs")


def positive(data, code, expected_contracts, expected_scope, source_path, cache=None):
    results, inference, selected_scope = validate_outcome(data, code, True, errors=0, verified=expected_scope["verified"])
    require(inference.get("status") == "verified" and inference.get("crate_success") is True, "positive inference status")
    require(inference.get("interface_written") is True and inference.get("failed_functions") == [], "positive publication")
    exact(data, expected_contracts, cache)
    models = inference.get("trusted_models")
    require(type(models) is list and models, "trusted standard models missing")
    for model in models:
        require(type(model.get("function")) is str and model["function"].startswith(("core::", "alloc::", "vstd::", "verus_builtin::")), "trusted application summary")
        require(type(model.get("fingerprint")) is str and re.fullmatch(r"[0-9a-f]{64}", model["fingerprint"]), "trusted model fingerprint")
    selected = {key: value for key, value in expected_scope.items() if key != "verified"}
    scope(data, selected, source_path)


def certificate(blob):
    require(blob.endswith(b"VRINFER1"), "missing certificate")
    for start in range(len(blob) - 8):
        if blob[start:start + 10] != b'{"schema":':
            continue
        try:
            content = blob[start:-8]
            value = json.loads(content, object_pairs_hook=boundary.unique_object)
        except (ValueError, UnicodeError):
            continue
        return value, sha256(content)
    raise ValueError("malformed certificate")


def check_certificate(blob):
    value, fingerprint = certificate(blob)
    require(type(value.get("schema")) is int and value["schema"] == IDENTITY[2], "interface schema")
    require(value.get("policy") == POLICY and type(value.get("producer")) is str and value["producer"], "interface policy/producer")
    require(all(value.get(key) is True for key in OBLIGATIONS), "certificate proof guarantees")
    require(type(value.get("proofs")) is list and value["proofs"], "certificate proofs")
    for proof in value["proofs"]:
        require(type(proof) is list and len(proof) == 2
                and type(proof[1]) is dict and all(proof[1].get(key) is True for key in OBLIGATIONS), "certificate function guarantees")
    require(type(value.get("summaries")) is str and re.fullmatch(r"[0-9a-f]{64}", value["summaries"]), "summary fingerprint")
    require(type(value.get("trusted_models")) is list and value["trusted_models"], "trusted model inventory")
    model_names = set()
    for entry in value["trusted_models"]:
        require(type(entry) is list and len(entry) == 2 and type(entry[0]) is dict, "trusted model entry")
        path = entry[0].get("path", {})
        require(path.get("krate") in ("Core", "Alloc", "Vstd", "Builtin"), "trusted application model in certificate")
        require(type(path.get("segments")) is list and path["segments"]
                and all(type(segment) is str and segment for segment in path["segments"]), "trusted model path")
        name = json.dumps(path, sort_keys=True)
        require(name not in model_names, "duplicate trusted model")
        model_names.add(name)
        require(type(entry[1]) is str and re.fullmatch(r"[0-9a-f]{64}", entry[1]), "trusted model certificate fingerprint")
    return value, fingerprint


def imported(data, vir, rlib, provider_certificate, fingerprint, argv):
    identities = {row[0]["path"]["krate"]["Id"][1] for row in provider_certificate["proofs"]}
    require(len(identities) == 1 and all(type(value) is int for value in identities), "provider crate identity")
    expected = [dict(crate_id=next(iter(identities)), fingerprint=fingerprint)]
    require(identical(data["verus-infer"].get("imported_inference_dependencies"), expected), "imported certificate binding")
    require(argv.count("--import") == 1 and argv.count("--extern") == 1, "both IMPORT kinds required")
    require(argv[argv.index("--import") + 1] == "prefix_provider=" + str(vir), "VIR IMPORT path")
    require(argv[argv.index("--extern") + 1] == "prefix_provider=" + str(rlib), "rlib IMPORT path")
    require(vir.is_file() and rlib.is_file(), "import artifacts missing")
    return dict(vir_sha256=digest(vir), rlib_sha256=digest(rlib), certificate_sha256=fingerprint,
                imported_inference_dependencies=expected)


def failed(data, code, stderr, artifact, category, diagnostic, expected_contracts=None,
           failed_function="prefix_provider::semantic_goal"):
    require(type(code) is int and code != 0, "negative exit code")
    require(not any(word in stderr.lower() for word in ("panicked at", "resource limit", "timed out", "out of memory")), "nonsemantic resource failure")
    results, inference = data["verification-results"], data["verus-infer"]
    require(results.get("success") is False and results.get("encountered-error") is True, "negative result")
    require(inference.get("crate_success") is False and inference.get("interface_written") is False, "negative publication")
    require(not artifact.exists(), "seeded inference-aware failed export retained")
    require(diagnostic in stderr, "wrong failure diagnostic")
    if category in ("semantic", "safety"):
        require(results.get("encountered-vir-error") is False, "frontend decline is not a semantic proof")
        require(type(results.get("errors")) is int and results["errors"] == 1, "negative proof error count")
        require(inference.get("status") == "caller_proof_failed", "negative proof classification")
        require(inference.get("failed_functions") == [failed_function], "intended failed function")
        exact(data, expected_contracts, exported=False)
    elif category == "malformed":
        require(results.get("encountered-vir-error") is False and inference.get("status") == "verification_failed", "malformed Rust classification")
        require(inference.get("failed_functions") == [], "malformed input has proof failures")
        require(type(results.get("verified")) is int and results["verified"] == 0
                and type(results.get("errors")) is int and results["errors"] == 0
                and inference.get("generated_functions") == 0, "malformed input reached verification")
    else:
        require(results.get("encountered-vir-error") is True, "frontend/import classification")
        require(inference.get("failed_functions") == [], "frontend has proof failures")
    return dict(category=category, diagnostic=diagnostic, failed_export_absent=True)


def failure_location(stderr, source, source_path, function, diagnostic):
    marker = "fn " + function.split("::")[-1] + "("
    if marker not in source:
        marker = marker[:-1] + "<"
    selected = boundary.item(source, marker)
    start = source.index(marker)
    first = source.count("\n", 0, start) + 1
    last = first + selected.count("\n")
    matches = re.findall(r"error: " + re.escape(diagnostic) + r"\n\s*--> ([^\n]+):(\d+):(\d+)", stderr)
    require(len(matches) == 1 and matches[0][0] == str(source_path)
            and first <= int(matches[0][1]) <= last, "failure outside intended authored goal")
    return dict(function=function, source=str(source_path), line=int(matches[0][1]))


def native_mutants(pieces):
    for label, name, old, new in (
        ("omit_database_prefix", "database_key", "self.prefix.clone()", "Vec::new()"),
        ("omit_transaction_prefix", "transaction_key", "self.prefix.clone()", "Vec::new()"),
        ("duplicate_suffix", "database_key", "full_key.extend_from_slice(key);", "full_key.extend_from_slice(key);\n        full_key.extend_from_slice(key);"),
        ("endpoint_is_start", "transaction_range", "self.get_full_key(range.end)", "self.get_full_key(range.start)"),
    ):
        changed = dict(pieces)
        require(changed[name].count(old) == 1, "mutation anchor")
        changed[name] = changed[name].replace(old, new)
        yield label, changed


def mutant_goal(label):
    if label == "endpoint_is_start":
        return """fn semantic_goal<Inner>(database: &PrefixDatabaseTransaction<Inner>, bounds: Range<&[u8]>)
    -> (result: Range<Vec<u8>>)
    ensures result.start@ == database.prefix@ + bounds.start@,
        result.end@ == database.prefix@ + bounds.end@,
{ database.get_full_range(bounds) }
"""
    owner = "PrefixDatabaseTransaction" if label == "omit_transaction_prefix" else "PrefixDatabase"
    bound = "" if owner == "PrefixDatabaseTransaction" else ": Debug"
    return f"""fn semantic_goal<Inner{bound}>(database: &{owner}<Inner>, key: &[u8])
    -> (result: Vec<u8>)
    ensures result@ == database.prefix@ + key@,
{{ database.get_full_key(key) }}
"""


def frontend_controls():
    return {
        "non_u8": ("""use vstd::prelude::*;
verus! { #[verifier::infer] fn unsupported(prefix: &Vec<u16>, key: &[u16]) -> Vec<u16> {
    let mut result = prefix.clone(); result.extend_from_slice(key); result
} }
""", "unsupported owned input/output leaf"),
        "generic_clone": ("""use vstd::prelude::*;
verus! { #[verifier::infer] fn unsupported<T: Clone>(prefix: &Vec<T>) -> Vec<T> {
    prefix.clone()
} }
""", "generic Clone does not establish value equality"),
        "custom_clone": ("""use vstd::prelude::*;
verus! {
struct Custom { value: u8 }
impl Clone for Custom { fn clone(&self) -> Self { Self { value: self.value + 1 } } }
#[verifier::infer] fn unsupported(prefix: &Vec<Custom>) -> Vec<Custom> {
    prefix.clone()
} }
""", "generic Clone does not establish value equality"),
        "nonlocal": ("""use vstd::prelude::*;
verus! { struct State { data: Vec<u8> }
#[verifier::infer] fn unsupported(state: &mut State, key: &[u8]) { state.data.extend_from_slice(key); }
}
""", "Verus Infer unsupported"),
        "lookalike": ("""use vstd::prelude::*;
verus! {
struct Fake { data: Vec<u8> }
impl Fake { fn extend_from_slice(&mut self, key: &[u8]) {} }
#[verifier::infer] fn unsupported(prefix: &Vec<u8>, key: &[u8]) -> Vec<u8> {
    let mut fake = Fake { data: prefix.clone() }; fake.extend_from_slice(key); fake.data
} }
""", "calls require an acyclic same-crate inferred callee"),
        "effectful_argument": ("""use vstd::prelude::*;
verus! {
fn effect(key: &[u8]) -> &[u8] { key }
#[verifier::infer] fn unsupported(prefix: &Vec<u8>, key: &[u8]) -> Vec<u8> {
    let mut result = prefix.clone(); result.extend_from_slice(effect(key)); result
} }
""", "calls require an acyclic same-crate inferred callee"),
    }


def replace_certificate(blob, value):
    certificate(blob)
    start = blob.rfind(b'{"schema":')
    require(start >= 0, "certificate start")
    return blob[:start] + json.dumps(value, separators=(",", ":")).encode() + b"VRINFER1"


def identity_sources():
    registry = (ROOT / "source/vir/src/infer/registry.rs").read_text()
    interfaces = (ROOT / "source/rust_verify/src/import_export.rs").read_text()
    require('const VERSION: &str = "' + CACHE_VERSION + '";' in registry, "current cache identity")
    require('const INFER_POLICY: &str = "' + POLICY + '";' in interfaces, "current interface policy")
    require("const INFER_SCHEMA: u32 = 38;" in interfaces, "current interface schema")


def completion(cases, baseline):
    expected = {name: "positive" for name in ("cold", "warm", "consumer", "poisoned_cache",
                "restore_provider", "final_provider", "final_consumer")}
    for name in ("omit_database_prefix", "omit_transaction_prefix", "duplicate_suffix", "endpoint_is_start"):
        expected[name + "_exact"] = "positive"
        expected[name + "_semantic"] = "semantic"
    expected.update(false_goal="semantic", unsafe_index="safety", malformed_goal="malformed")
    expected.update({name: "frontend" for name in frontend_controls()})
    expected.update({"metadata_" + name: "import" for name in ("policy", "proof", "summaries", "trusted_models", "missing")})
    if baseline:
        expected["baseline"] = "frontend"
    require(type(cases) is list and len(cases) == len(expected), "complete replay case count")
    require({case["label"]: case["category"] for case in cases} == expected, "missing, duplicate, or misclassified cases")
    for case in cases:
        if case["category"] != "positive" and case["label"] != "baseline":
            require(type(case.get("seeded_export_sha256")) is str
                    and re.fullmatch(r"[0-9a-f]{64}", case["seeded_export_sha256"]), "negative export was not seeded")
            require(case.get("export_sha256") is None and case.get("failed_export_absent") is True, "negative export survived")


def replay(args):
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    verifier, solver, application, sysroot = (path.resolve() for path in (args.verus, args.solver, args.application, args.sysroot))
    environment = environment_for(output, solver, sysroot)
    environment["VERUS_INFER_CACHE"] = str(output / "candidate-cache")
    expected_contracts = read_json(EVIDENCE / "contracts.json")
    expected_scopes = read_json(EVIDENCE / "scope.json")
    receipt = dict(decision="INCOMPLETE", identity=list(IDENTITY), pin=boundary.PIN, cases=[])
    fixtures = [path for path in EVIDENCE.iterdir() if path.is_file()]
    inputs = [Path(__file__), Path(__file__).with_name("test_fedimint_prefix_infer79.py"),
              Path(__file__).with_name("workflow_runtime.py"), *fixtures]
    inputs += [ROOT / name for name in ("source/vir/src/infer/owned/byte_append.rs",
        "source/vir/src/infer/owned/models.rs", "source/vir/src/infer/owned.rs", "source/vir/src/infer.rs",
        "source/vir/src/infer/records/shape.rs", "source/vir/src/infer/registry.rs", "source/rust_verify/src/import_export.rs")]
    inputs += [application / name for name in read_json(boundary.EVIDENCE / "source-ledger.json")["sources"]]
    inputs += [boundary.EVIDENCE / name for name in ("prefix.spec.rs", "goals.verus.rs", "native-items.json", "source-ledger.json")]
    inputs += [sysroot / "bin/rustc", *[path for path in (sysroot / "lib").rglob("*") if path.is_file() and path.suffix in (".rlib", ".dylib", ".so")]]
    artifacts = tool_artifacts(verifier, solver, extra=inputs)
    if args.baseline_verus:
        artifacts += tool_artifacts(args.baseline_verus.resolve(), solver)
    guard = ArtifactGuard(artifacts)
    receipt["artifacts_before"] = guard.initial
    receipt["native_before"] = boundary.native_state(application, environment)
    pieces = boundary.native_pieces(application)
    identity_sources()
    source = render(pieces)
    receipt["correspondence"] = correspondence(source, pieces)
    source_path = output / "provider.rs"
    source_path.write_text(source)
    provider_vir, provider_rlib = output / "prefix_provider.vir", output / "libprefix_provider.rlib"

    def invoke(label, text, crate="prefix_provider", imports=(), seed=None, binary=verifier):
        path = source_path if crate == "prefix_provider" else output / "consumer.rs"
        path.write_text(text)
        (output / (label + ".rs")).write_text(text)
        vir = provider_vir if crate == "prefix_provider" else output / "prefix_consumer.vir"
        rlib = provider_rlib if crate == "prefix_provider" else output / "libprefix_consumer.rlib"
        if seed is not None:
            vir.write_bytes(seed)
        argv = command(binary, path, sysroot, crate, vir, rlib, imports)
        validate_command(argv, binary, path, sysroot, crate, vir, rlib, imports)
        guard.check()
        directory = output / "runs" / label
        bound_inputs = [path, binary, solver, *[provider for entry in imports for provider in entry[1:]]]
        execute(directory, argv, output, environment, args.timeout, inputs=bound_inputs)
        guard.check()
        timings = {}
        data, result, seconds = validate_timed(timings, bound_case, directory, argv, path, text,
                                              environment, bound_inputs)
        (output / (label + ".stdout.json")).write_bytes(result.stdout)
        (output / (label + ".stderr.txt")).write_bytes(result.stderr)
        row = dict(label=label, category="positive", command=argv, returncode=result.returncode, source_sha256=sha256(text.encode()),
                   stdout_sha256=sha256(result.stdout), stderr_sha256=sha256(result.stderr),
                   seeded_export_sha256=None if seed is None else sha256(seed),
                   export_sha256=digest(vir) if vir.is_file() else None,
                   invocation_seconds=seconds, **timings)
        receipt["cases"].append(row)
        write_json(output / "receipt.json", receipt)
        print(label, result.returncode, data["verus-infer"]["status"], flush=True)
        return data, result, row, vir

    try:
        if args.baseline_verus:
            data, run, row, artifact = invoke("baseline", source, binary=args.baseline_verus.resolve())
            row.update(validate_timed(row, failed, data, run.returncode, run.stderr.decode(), artifact, "frontend", "extend_from_slice"))
            require(data["verus-infer"].get("status") == "unsupported"
                    and data["verus-infer"].get("generated_functions") == 0, "baseline decline changed")
        for phase in ("cold", "warm"):
            data, run, row, artifact = invoke(phase, source)
            validate_timed(row, positive, data, run.returncode, expected_contracts["provider"], expected_scopes["provider"], source_path,
                     "miss")
            provider_blob = provider_vir.read_bytes()
            provider_certificate, fingerprint = check_certificate(provider_blob)
            row.update(vir_sha256=digest(provider_vir), rlib_sha256=digest(provider_rlib), certificate_sha256=fingerprint)
        imports = (("prefix_provider", provider_vir, provider_rlib),)
        consumer = (EVIDENCE / "consumer.verus.rs").read_text()
        data, run, row, artifact = invoke("consumer", consumer, "prefix_consumer", imports)
        validate_timed(row, positive, data, run.returncode, expected_contracts["consumer"], expected_scopes["consumer"], output / "consumer.rs", "miss")
        row["imports"] = imported(data, provider_vir, provider_rlib, provider_certificate, fingerprint, row["command"])
        check_certificate(artifact.read_bytes())
        seed = artifact.read_bytes()
        cache = output / "candidate-cache"
        cache.mkdir(exist_ok=True)
        poisoned = list(cache.glob("*.json"))
        write_json(output / "cache-before-poison.json", {path.name: read_json(path) for path in poisoned})
        provider_data = read_json(output / "warm.stdout.json")
        for function in provider_data["verus-infer"]["functions"]:
            path = cache / (function["candidate_id"] + ".json")
            path.write_text('{"version":"poisoned","key":"not-current","requires":false,"ensures":true}')
        receipt["cache_control"] = "Owned summaries regenerate on cold, warm, and poisoned invocations; no cache-hit or hostile-cache decoder claim."
        data, run, row, artifact = invoke("poisoned_cache", source)
        validate_timed(row, positive, data, run.returncode, expected_contracts["provider"], expected_scopes["provider"], source_path, "miss")
        provider_blob = provider_vir.read_bytes()
        for label, mutant in native_mutants(pieces):
            text = render(mutant, goals=False)
            data, run, row, artifact = invoke(label + "_exact", text)
            require(run.returncode == 0, "mutant is not exact before authored goals")
            mutant_contracts = {function["function"].split("::", 1)[-1]: function["contract"] for function in data["verus-infer"]["functions"]}
            validate_timed(row, exact, data, mutant_contracts)
            mutant_seed = artifact.read_bytes()
            text = render(mutant, goals=False)[:-2] + mutant_goal(label) + "}\n"
            data, run, row, artifact = invoke(label + "_semantic", text, seed=mutant_seed)
            row.update(validate_timed(row, failed, data, run.returncode, run.stderr.decode(), artifact, "semantic", "postcondition not satisfied", mutant_contracts))
            row["failure_location"] = failure_location(run.stderr.decode(), text, source_path, "prefix_provider::semantic_goal", "postcondition not satisfied")
        data, run, row, artifact = invoke("restore_provider", source)
        validate_timed(row, positive, data, run.returncode, expected_contracts["provider"], expected_scopes["provider"], source_path)
        provider_blob = provider_vir.read_bytes()
        for label, addition, diagnostic in (
            ("false_goal", "pub fn false_goal() ensures false, {}\n", "postcondition not satisfied"),
            ("unsafe_index", "pub fn unsafe_index(prefix:&Vec<u8>, key:&[u8]) {let values=key_contents(prefix,key); let value=values[0];}\n", "precondition not satisfied"),
        ):
            text = source[:-2] + addition + "}\n"
            data, run, row, artifact = invoke(label, text, seed=provider_blob)
            row.update(validate_timed(row, failed, data, run.returncode, run.stderr.decode(), artifact, "safety" if label == "unsafe_index" else "semantic", diagnostic, expected_contracts["provider"], "prefix_provider::" + label))
            row["failure_location"] = failure_location(run.stderr.decode(), text, source_path, "prefix_provider::" + label, diagnostic)
        for label, (text, diagnostic) in frontend_controls().items():
            data, run, row, artifact = invoke(label, text, seed=provider_blob)
            row.update(validate_timed(row, failed, data, run.returncode, run.stderr.decode(), artifact, "frontend", diagnostic))
            require(data["verus-infer"].get("status") == "unsupported", "type/proxy control not unsupported")
        malformed = source[:-2] + "fn malformed(value: u8) ensures value == true, {}\n}\n"
        data, run, row, artifact = invoke("malformed_goal", malformed, seed=provider_blob)
        row.update(validate_timed(row, failed, data, run.returncode, run.stderr.decode(), artifact, "malformed", "SpecEq<bool>` is not satisfied"))
        data, run, row, artifact = invoke("final_provider", source)
        validate_timed(row, positive, data, run.returncode, expected_contracts["provider"], expected_scopes["provider"], source_path)
        provider_blob = provider_vir.read_bytes()
        metadata, fingerprint = check_certificate(provider_blob)
        for fault, diagnostic in (
            ("policy", "incompatible inference schema, safety policy, or producer"),
            ("proof", "incomplete inference proof guarantees"),
            ("summaries", "inferred contract fingerprint mismatch"),
            ("trusted_models", "stale or missing trusted-model fingerprint"),
            ("missing", "missing or invalid inference proof metadata"),
        ):
            corrupted = copy.deepcopy(metadata)
            if fault == "policy":
                corrupted["policy"] = "candidate-bound-evidence-v71"
            elif fault == "proof":
                corrupted["proofs"][0][1]["soundness"] = False
            elif fault == "summaries":
                corrupted["summaries"] = "0" * 64
            elif fault == "trusted_models":
                corrupted["trusted_models"][0][1] = "0" * 64
            altered = replace_certificate(provider_blob, corrupted)
            if fault == "missing":
                altered = provider_blob[:provider_blob.rfind(b'{"schema":')]
            provider_vir.write_bytes(altered)
            data, run, row, artifact = invoke("metadata_" + fault, consumer, "prefix_consumer", imports, seed=seed)
            row.update(validate_timed(row, failed, data, run.returncode, run.stderr.decode(), artifact, "import", diagnostic))
            provider_vir.write_bytes(provider_blob)
        data, run, row, artifact = invoke("final_consumer", consumer, "prefix_consumer", imports)
        validate_timed(row, positive, data, run.returncode, expected_contracts["consumer"], expected_scopes["consumer"], output / "consumer.rs")
        row["imports"] = imported(data, provider_vir, provider_rlib, metadata, fingerprint, row["command"])
        completion(receipt["cases"], args.baseline_verus is not None)
        receipt["native_after"] = boundary.native_state(application, environment)
        guard.check()
        receipt["artifacts_after"] = guard.snapshot()
        receipt["inputs_unchanged"] = True
        receipt["decision"] = "PASS"
        write_json(output / "receipt.json", receipt)
    except BaseException as error:
        receipt["error"] = str(error)
        write_json(output / "receipt.json", receipt)
        raise


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verus", type=Path, required=True)
    parser.add_argument("--solver", type=Path, required=True)
    parser.add_argument("--application", type=Path, required=True)
    parser.add_argument("--sysroot", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=int, default=180)
    parser.add_argument("--baseline-verus", type=Path)
    args = parser.parse_args()
    require(args.timeout > 0, "timeout must be positive")
    replay(args)


if __name__ == "__main__":
    main()
