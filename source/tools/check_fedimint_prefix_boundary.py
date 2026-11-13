"""Source-connected pure prefix bridge; ordinary checked contracts, NOT Infer progress."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

from infer_eval_support import ArtifactGuard, digest, run_compiler, tool_artifacts
from infer_transfer_identity import ENGINE_IDENTITY


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "source/tools/fixtures/transfers/transfer-fedimint-prefix-boundary78"
PIN = "b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236"
DATABASE = "fedimint-core/src/db/mod.rs"
NATIVE = {
    "database_key": ("PrefixDatabase", "get_full_key"),
    "transaction_key": ("PrefixDatabaseTransaction", "get_full_key"),
    "transaction_range": ("PrefixDatabaseTransaction", "get_full_range"),
}
ROW_KEYS = ("function", "rust_function", "source", "automatically_derived", "selected",
            "body_status", "inference_candidate", "ordinary_contract")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(content):
    return hashlib.sha256(content).hexdigest()


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON key: " + key)
        result[key] = value
    return result


def read_json(path):
    return json.loads(Path(path).read_text(), object_pairs_hook=unique_object)


def write_json(path, value):
    Path(path).write_text(json.dumps(value, sort_keys=True, indent=2) + "\n")


def item(source, marker):
    require(source.count(marker) == 1, "nonunique source marker: " + marker)
    start = source.index(marker)
    opening = source.index("{", start)
    depth, end = 1, opening + 1
    while depth:
        require(end < len(source), "unterminated source item")
        depth += (source[end] == "{") - (source[end] == "}")
        end += 1
    return source[start:end]


def method_section(source, owner):
    start = source.index("> " + owner + "<Inner> {")
    end = source.find("\nimpl<", start)
    return start, len(source) if end < 0 else end


def body(source):
    return source[source.index("{"):]


def native_pieces(application):
    ledger = read_json(EVIDENCE / "source-ledger.json")
    require(ledger["pin"] == PIN, "source ledger pin")
    for name, expected in ledger["sources"].items():
        require(digest(application / name) == expected, "pinned source changed: " + name)
    source = (application / DATABASE).read_text()
    transaction = source[source.index("impl<Inner> PrefixDatabaseTransaction<Inner>"):]
    pieces = {
        "database_record": item(source, "struct PrefixDatabase<Inner>"),
        "transaction_record": item(source, "struct PrefixDatabaseTransaction<Inner>"),
        "token": "pub struct GlobalDBTxAccessToken(u32);",
        "database_key": item(source[:source.index("impl<Inner> PrefixDatabaseTransaction<Inner>")], "    fn get_full_key("),
        "transaction_key": item(transaction, "    fn get_full_key("),
        "transaction_range": item(transaction, "    fn get_full_range("),
    }
    require(pieces["token"] in source, "native token representation changed")
    require(pieces == read_json(EVIDENCE / "native-items.json"), "native item bytes changed")
    require(set(ledger["items"]) == set(pieces), "source item ledger coverage")
    for name, selected in pieces.items():
        require(sha256(selected.encode()) == ledger["items"][name]["item_sha256"], "source item hash: " + name)
    return pieces


def render(pieces, inference=False):
    records = []
    for name in ("database_record", "transaction_record"):
        record = "pub " + pieces[name]
        record = re.sub(r"(?m)^(    )([a-z_]+):", r"\1pub \2:", record)
        records.append(record)
    source = ("use vstd::prelude::*;\nuse std::fmt::Debug;\nuse std::ops::Range;\nverus! {\n"
              + pieces["token"] + "\n" + "\n".join(records) + "\n")
    if not inference:
        source += (EVIDENCE / "prefix.spec.rs").read_text() + "\n"
    contracts = read_json(EVIDENCE / "contracts.json")
    for owner, names in (("PrefixDatabase", ("database_key",)),
                         ("PrefixDatabaseTransaction", ("transaction_key", "transaction_range"))):
        bound = ": Debug" if owner == "PrefixDatabase" else ""
        source += "impl<Inner" + bound + "> " + owner + "<Inner> {\n"
        for name in names:
            if inference:
                source += "#[verifier::infer]\n" + pieces[name].replace("    fn ", "    pub fn ", 1) + "\n"
            else:
                source += contracts[name] + body(pieces[name]) + "\n"
        source += "}\n"
    if not inference:
        source += (EVIDENCE / "goals.verus.rs").read_text()
    return source + "}\n"


def correspondence(source, pieces):
    result = {}
    for name, (owner, method) in NATIVE.items():
        section = source[source.index("impl<Inner" + (": Debug" if owner == "PrefixDatabase" else "") + "> " + owner + "<Inner>"):]
        if owner == "PrefixDatabase":
            section = section[:section.index("impl<Inner> PrefixDatabaseTransaction")]
        checked = body(item(section, "    pub fn " + method + "("))
        original = body(pieces[name])
        require(checked == original, "native body adaptation: " + name)
        result[name] = dict(native_body_sha256=sha256(original.encode()),
                            checked_body_sha256=sha256(checked.encode()), byte_identical=True)
    return result


def expected_scope(source, path, crate, expected):
    rows, identities = [], {}
    for name, kind in expected["functions"].items():
        short = name.split("::")[-1]
        marker = "pub " + ("open spec " if kind == "spec" else "proof " if kind == "proof" else "") + "fn " + short
        if "::" in name:
            owner = name.split("::")[0]
            section_start = source.index("> " + owner + "<Inner> {")
            section_end = source.find("\nimpl<", section_start)
            section_end = len(source) if section_end < 0 else section_end
            matches = list(re.finditer(re.escape(marker) + r"(?=[(<])", source[section_start:section_end]))
            require(len(matches) == 1, "scope source function: " + name)
            start = section_start + matches[0].start()
        else:
            matches = list(re.finditer(re.escape(marker) + r"(?=[(<])", source))
            require(len(matches) == 1, "scope source function: " + name)
            start = matches[0].start()
            section_end = len(source)
        selected = item(source[start:section_end], marker)
        signature = re.split(r"\n\s*(?:requires|ensures|decreases)\b|\{", selected, maxsplit=1)[0].rstrip()
        declaration_end = start + len(signature)
        if re.search(r"->\s*\(", signature):
            declaration_end -= 1
        line = source.count("\n", 0, start) + 1
        column = start - source.rfind("\n", 0, start)
        end_line = source.count("\n", 0, declaration_end) + 1
        end_column = declaration_end - source.rfind("\n", 0, declaration_end)
        status = {"exec": "checked_exec_body", "proof": "checked_proof_body",
                  "spec": "specification_definition_or_declaration"}[kind]
        rust_name = name.replace("::", "::<Inner>::", 1) if "::" in name else short
        row = dict(function=crate + "::" + name, rust_function=rust_name,
                   source=f"{path}:{line}:{column}: {end_line}:{end_column}", automatically_derived=False,
                   selected=True, body_status=status, inference_candidate=False, ordinary_contract=kind != "spec")
        rows.append(row)
        identities[row["function"]] = dict(kind=kind, item_sha256=sha256(selected.encode()),
            body_sha256=sha256(body(selected).encode()), origin="native-body" if "::" in name else "author-owned")
    return sorted(rows, key=lambda row: row["function"]), identities


def common(data):
    results, inference = data["verification-results"], data["verus-infer"]
    require(all(key in inference for key in ("generation_failure", "failed_functions")), "missing inference field")
    require(inference.get("schema_version") == 1 and type(inference.get("schema_version")) is int, "report schema")
    require(inference.get("functions") == []
            and inference.get("trusted_models") == [] and inference.get("imported_inference_dependencies") == [],
            "unexpected inference or trusted application summary")
    require(inference.get("interface_written") is False, "ordinary/failed publication changed")
    return results, inference


def validate_positive(data, code, source, path, crate, expected):
    require(type(code) is int and code == 0, "positive compiler failed")
    results, inference = common(data)
    for key, value in (("success", True), ("encountered-error", False), ("encountered-vir-error", False),
                       ("is-verifying-entire-crate", True)):
        require(results.get(key) is value, "positive result flag: " + key)
    for key, value in (("verified", expected["verified"]), ("errors", 0)):
        require(type(results.get(key)) is int and results[key] == value, "positive count: " + key)
    require(inference.get("status") == "no_inference" and inference.get("crate_success") is True
            and inference["generation_failure"] is None and inference["failed_functions"] == [], "ordinary positive classification")
    for key in ("requested_functions", "generated_functions"):
        require(type(inference.get(key)) is int and inference[key] == 0, "not an ordinary proof")
    scope = inference["verification_scope"]
    require(scope.get("native_collection_completed") is True
            and scope.get("complete_selected_scope_verified") is True, "incomplete selected scope")
    for key in ("checked_exec_bodies", "checked_proof_bodies"):
        require(type(scope.get(key)) is int and scope[key] == expected[key], "checked count: " + key)
    rows, identities = expected_scope(source, path, crate, expected)
    actual = scope["native_functions"]
    require(type(actual) is list and len(actual) == len(rows), "scope row count")
    for row in actual:
        for flag in ("automatically_derived", "selected", "inference_candidate", "ordinary_contract"):
            require(type(row.get(flag)) is bool, "scope flag type: " + flag)
        require(all(key in row for key in ROW_KEYS), "missing identity field")
    require(sorted([{key: row[key] for key in ROW_KEYS} for row in actual], key=lambda row: row["function"]) == rows,
            "exact source/body scope identity mismatch")
    return dict(verified=results["verified"], checked_exec_bodies=scope["checked_exec_bodies"],
                checked_proof_bodies=scope["checked_proof_bodies"], identities=identities, inferred_functions=0)


def validate_failure(data, code, stderr, source, path, expected_verified, failed_function, safety=False):
    require(type(code) is int and code != 0, "negative succeeded")
    results, inference = common(data)
    require(results.get("success") is False and results.get("encountered-error") is True
            and results.get("encountered-vir-error") is False
            and results.get("is-verifying-entire-crate") is True, "negative not a checked proof failure")
    require(type(results.get("verified")) is int and results["verified"] == expected_verified
            and type(results.get("errors")) is int and results["errors"] == 1, "negative exact work/errors")
    require(inference.get("status") == "caller_proof_failed" and inference.get("crate_success") is False,
            "negative ordinary classification")
    require(inference["failed_functions"] == ["prefix_provider::" + failed_function]
            and inference["generation_failure"] is None, "negative failed a different function")
    for key in ("requested_functions", "generated_functions"):
        require(type(inference.get(key)) is int and inference[key] == 0, "negative ordinary counts")
    diagnostic = "possible arithmetic underflow/overflow" if safety else "postcondition not satisfied"
    require(diagnostic in stderr, "negative missed intended obligation")
    require(not any(word in stderr.lower() for word in
                    ("panicked at", "timed out", "resource limit", "out of memory", "precondition not satisfied")),
            "negative has wrong failure category")
    short = failed_function.split("::")[-1]
    marker = "fn " + short
    if "::" in failed_function:
        owner = failed_function.split("::")[0]
        start, end = method_section(source, owner)
        start = source.index(marker, start)
    else:
        start = source.index(marker)
        end = len(source)
    selected = item(source[start:end], marker)
    first_line = source.count("\n", 0, start) + 1
    last_line = first_line + selected.count("\n")
    locations = re.findall(r"error: " + diagnostic + r"\n\s*--> ([^\n]+):(\d+):(\d+)", stderr)
    require(len(locations) == 1 and locations[0][0] == str(path)
            and first_line <= int(locations[0][1]) <= last_line, "failure is outside intended function")
    return dict(category="safety" if safety else "semantic", verified=expected_verified, errors=1,
                failed_function=failed_function, diagnostic=diagnostic)


def validate_blocker(data, code, stderr, artifact):
    require(type(code) is int and code != 0, "Infer unexpectedly succeeded")
    results, inference = common(data)
    require(results.get("success") is False and results.get("encountered-error") is True
            and results.get("encountered-vir-error") is True, "blocker not frontend/VIR")
    require(inference.get("status") == "unsupported" and inference.get("crate_success") is False,
            "blocker classification")
    require(inference["failed_functions"] == [], "blocker is not a frontend-only rejection")
    require(type(inference.get("requested_functions")) is int and inference["requested_functions"] == 3
            and type(inference.get("generated_functions")) is int and inference["generated_functions"] == 0,
            "blocker request counts")
    failure = inference.get("generation_failure")
    require(type(failure) is str and "extend_from_slice" in failure
            and "acyclic same-crate inferred callee or certified imported summary" in failure
            and failure in stderr, "wrong Infer blocker")
    require(not artifact.exists(), "inference-aware failure retained seeded export")
    return dict(category="frontend-unsupported", requested=3, inferred=0, generation_failure=failure,
                inference_aware_export_removed=True)


def mutate_once(source, original, replacement):
    require(source.count(original) == 1, "mutation anchor count")
    return source.replace(original, replacement, 1)


def controls(source, pieces):
    result = []
    for label, name, original, replacement in (
        ("omit_database_prefix", "database_key", "self.prefix.clone()", "Vec::new()"),
        ("reverse_concatenation", "database_key", "let mut full_key = self.prefix.clone();\n        full_key.extend_from_slice(key);",
         "let mut full_key = Vec::new();\n        full_key.extend_from_slice(key);\n        full_key.extend_from_slice(&self.prefix);"),
        ("erase_suffix", "database_key", "full_key.extend_from_slice(key);", "full_key.extend_from_slice(&[]);"),
        ("omit_transaction_prefix", "transaction_key", "self.prefix.clone()", "Vec::new()"),
        ("range_end_is_start", "transaction_range", "self.get_full_key(range.end)", "self.get_full_key(range.start)"),
    ):
        owner, method = NATIVE[name]
        section_start, section_end = method_section(source, owner)
        method_start = source.index("    pub fn " + method + "(", section_start)
        selected = item(source[method_start:section_end], "    pub fn " + method + "(")
        changed = mutate_once(selected, original, replacement)
        result.append((label, source[:method_start] + changed + source[method_start + len(selected):], owner + "::" + method, False))
    selected = item(source, "pub proof fn disjoint_namespaces(")
    changed = mutate_once(selected, "    requires prefix_incomparable(left, right),\n", "")
    result.append(("unconditional_disjointness", mutate_once(source, selected, changed), "disjoint_namespaces", False))
    overflow = "pub fn unchecked_increment(value: u64) -> u64 { value + 1 }\n"
    result.append(("unchecked_overflow", source[:-2] + overflow + "}\n", "unchecked_increment", True))
    return result


def environment_for(output, solver, sysroot):
    environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")
                   and name not in ("RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER",
                                    "RUSTC_WORKSPACE_WRAPPER", "VERUS_EXTRA_ARGS", "VERUS_INFER_CACHE")}
    for name in ("home", "cache", "cargo-home", "rustup-home", "tmp"):
        (output / name).mkdir()
    environment.update(HOME=str(output / "home"), XDG_CACHE_HOME=str(output / "cache"),
        CARGO_HOME=str(output / "cargo-home"), RUSTUP_HOME=str(output / "rustup-home"),
        CARGO_TARGET_DIR=str(output / "unused-target"), TMPDIR=str(output / "tmp"),
        GIT_OPTIONAL_LOCKS="0", CARGO_NET_OFFLINE="true", PYTHONDONTWRITEBYTECODE="1",
        VERUS_USE_RUSTUP="0", VERUS_Z3_PATH=str(solver), DYLD_LIBRARY_PATH=str(sysroot / "lib"))
    return environment


def native_state(application, environment):
    command = ["git", "-c", "core.fsmonitor=false", "-C", str(application)]
    head = subprocess.check_output(command + ["rev-parse", "HEAD"], env=environment, text=True).strip()
    status = subprocess.check_output(command + ["status", "--porcelain", "--untracked-files=all"], env=environment, text=True)
    require(head == PIN and status == "", "native application pin/cleanliness")
    return dict(pin=head, status=status)


def discover_sysroot():
    toolchain = (ROOT / "rust-toolchain.toml").read_text()
    channel = re.search(r'channel\s*=\s*"([^"]+)"', toolchain).group(1)
    home = Path(os.environ.get("RUSTUP_HOME", str(Path.home() / ".rustup")))
    candidates = sorted((home / "toolchains").glob(channel + "-*"))
    require(len(candidates) == 1, "supply --sysroot for the existing pinned Rust toolchain")
    return candidates[0].resolve()


def replay(args):
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    verifier, solver, application = args.verus.resolve(), args.solver.resolve(), args.application.resolve()
    sysroot = args.sysroot.resolve() if args.sysroot else discover_sysroot()
    environment = environment_for(output, solver, sysroot)
    receipt = dict(decision="INCOMPLETE", engine_identity=list(ENGINE_IDENTITY), pin=PIN, cases=[])
    guard = None
    try:
        require(ENGINE_IDENTITY == (77, 71, 38), "current engine identity changed")
        runtime = read_json(EVIDENCE / "runtime-hashes.json")
        for name, expected in runtime.items():
            path = solver if name == "z3" else verifier.parent / name
            require(digest(path) == expected, "not audited sealed runtime: " + name)
        pieces = native_pieces(application)
        expected = read_json(EVIDENCE / "expected-scope.json")
        fixtures = sorted(path for path in EVIDENCE.iterdir() if path.suffix in (".rs", ".json"))
        inputs = [Path(__file__), Path(__file__).with_name("test_fedimint_prefix_boundary.py"),
                  Path(__file__).with_name("infer_transfer_identity.py"), ROOT / "rust-toolchain.toml", *fixtures]
        inputs += [application / name for name in read_json(EVIDENCE / "source-ledger.json")["sources"]]
        inputs += [sysroot / "bin/rustc", *sorted((sysroot / "lib").glob("*rustc_driver*"))]
        guard = ArtifactGuard(tool_artifacts(verifier, solver, extra=inputs)
                              + [verifier.parent / name for name in runtime if name != "z3"])
        receipt["artifacts_before"] = guard.initial
        receipt["native_before"] = native_state(application, environment)
        receipt["environment"] = {name: environment[name] for name in ("HOME", "CARGO_HOME", "RUSTUP_HOME",
            "XDG_CACHE_HOME", "TMPDIR", "GIT_OPTIONAL_LOCKS", "VERUS_USE_RUSTUP", "DYLD_LIBRARY_PATH")}
        receipt["source_ledger"] = read_json(EVIDENCE / "source-ledger.json")
        source = render(pieces)
        receipt["body_correspondence"] = correspondence(source, pieces)

        def invoke(label, text, crate, category, debug=True, dependency=None, seed=None, failed=None, safety=False):
            case = output / label
            case.mkdir()
            path, artifact, library = case / "source.rs", case / (crate + ".vir"), case / ("lib" + crate + ".rlib")
            path.write_text(text)
            if seed is not None:
                artifact.write_bytes(seed)
            command = [str(verifier), str(path), "--sysroot", str(sysroot), "--crate-name", crate,
                "--crate-type", "lib", "--edition=2021", "--no-cheating", "--output-json", "--num-threads", "1",
                "--export", str(artifact), "-C", "debug-assertions=" + ("yes" if debug else "no")]
            if category == "blocker":
                command += ["--infer-goals"]
            else:
                command += ["--compile", "-o", str(library)]
            if dependency is not None:
                imported_guard = ArtifactGuard(dependency)
                command += ["--extern", "prefix_provider=" + str(dependency[1]), "--import", "prefix_provider=" + str(dependency[0])]
            environment["VERUS_INFER_CACHE"] = str(case / "infer-cache")
            write_json(case / "command.json", command)
            guard.check()
            try:
                result = run_compiler(command, environment, args.timeout, cwd=case)
            except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
                for filename, content in (("stdout.json", getattr(error, "stdout", "")),
                                          ("stderr.txt", getattr(error, "stderr", ""))):
                    if isinstance(content, bytes):
                        content = content.decode(errors="replace")
                    (case / filename).write_text(content or "")
                raise
            (case / "stdout.json").write_text(result.stdout)
            (case / "stderr.txt").write_text(result.stderr)
            guard.check()
            if dependency is not None:
                imported_guard.check()
            row = dict(label=label, category=category, returncode=result.returncode,
                source_sha256=digest(path), stdout_sha256=digest(case / "stdout.json"),
                stderr_sha256=digest(case / "stderr.txt"), command_sha256=digest(case / "command.json"),
                artifact_sha256=digest(artifact) if artifact.is_file() else None,
                library_sha256=digest(library) if library.is_file() else None,
                seed_sha256=sha256(seed) if seed is not None else None,
                path=str(case), default_erasure=True, no_cheating=True)
            receipt["cases"].append(row)
            data = read_json(case / "stdout.json")
            if category == "positive":
                row["validation"] = validate_positive(data, result.returncode, text, path, crate, expected[crate])
                require(artifact.is_file() and library.is_file(), "positive ordinary artifacts missing")
            elif category == "blocker":
                row["validation"] = validate_blocker(data, result.returncode, result.stderr, artifact)
            else:
                row["validation"] = validate_failure(data, result.returncode, result.stderr, text, path,
                    expected[crate]["verified"] if safety else expected[crate]["verified"] - 1, failed, safety)
                row["legacy_failed_export"] = artifact.is_file()
                row["seed_replaced"] = seed is not None and artifact.is_file() and digest(artifact) != sha256(seed)
            return artifact, library

        infer_source = render(pieces, inference=True)
        correspondence(infer_source, pieces)
        invoke("infer_before", infer_source, "prefix_probe", "blocker")
        for debug in (True, False):
            suffix = "debug" if debug else "release_assertions"
            provider = invoke("provider_" + suffix, source, "prefix_provider", "positive", debug=debug)
            invoke("consumer_" + suffix, (EVIDENCE / "consumer.verus.rs").read_text(),
                   "prefix_consumer", "positive", debug=debug, dependency=provider)
        seed = (output / "provider_debug/prefix_provider.vir").read_bytes()
        for label, mutant, failed, safety in controls(source, pieces):
            invoke(label, mutant, "prefix_provider", "safety" if safety else "semantic",
                   seed=seed if label == "omit_database_prefix" else None, failed=failed, safety=safety)
        invoke("infer_after_seeded", infer_source, "prefix_probe", "blocker", seed=seed)
        receipt.update(decision="CHECKED SOURCE-CONNECTED PURE PREFIX BOUNDARY; NO INFERENCE PROGRESS",
                       native_body_count=3, inferred_functions=0, whole_native_application_verified=False)
    except BaseException as error:
        receipt["failure"] = dict(type=type(error).__name__, message=str(error))
        raise
    finally:
        try:
            if guard is not None:
                receipt["artifacts_after"] = guard.snapshot()
                receipt["artifacts_unchanged"] = receipt["artifacts_after"] == guard.initial
                guard.check()
                receipt["native_after"] = native_state(application, environment)
                if "native_before" in receipt:
                    require(receipt["native_after"] == receipt["native_before"], "native state changed")
        finally:
            write_json(output / "receipt.json", receipt)
    print(json.dumps(dict(output=str(output), decision=receipt["decision"], cases=len(receipt["cases"]))))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("application", "verus", "solver", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--sysroot", type=Path, help="existing pinned Rust sysroot; discovered read-only by default")
    parser.add_argument("--timeout", type=int, default=120)
    args = parser.parse_args()
    require(args.timeout > 0, "timeout must be positive")
    replay(args)


if __name__ == "__main__":
    main()
