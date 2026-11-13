"""Replay full-declaration redb selection with two authored opaque datatype boundaries."""

import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

import check_redb_slot_selection as original
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import LoggedCommands, positive_timeout, replay_receipt, require
from infer_transfer_identity import CACHE_VERSION, ENGINE_IDENTITY, INTERFACE_POLICY, INTERFACE_SCHEMA, require_no_foreign_dependencies


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "source/tools/fixtures/transfers/transfer-redb-slot-selection-opaque78"
FIXTURES = original.LIVE_EVIDENCE / "redb-opaque-fixtures.json"
METHODS = ("primary_slot", "secondary_slot", "swap_primary_slot", "select_primary_slot")
RECORDS = ("PageNumber", "BtreeHeader", "TransactionHeader", "DatabaseHeader", "UnrepairedDatabaseHeader")
PROOFS = ("soundness", "maximality", "exactness")
FAULTS = ("policy", "schema", "producer", "proof", "row_proof", "summaries", "trusted_models", "truncated")


def fixtures():
    data = original.read_json(FIXTURES)
    require(type(data["identity"]) is list and all(type(value) is int for value in data["identity"])
            and data["identity"] == list(ENGINE_IDENTITY), "engine identity changed")
    for relative, expected in data["base_inputs"].items():
        require(digest(ROOT / relative) == expected, "portable base fixture/helper changed: " + relative)
    require(len(data["expected"]["provider"]["native_functions"]) == 43, "full provider scope must have 43 rows")
    return data


def render(pieces, opaque=True):
    source = original.render(pieces, full_errors=True)
    source = original.mutate_once(source, "pub enum StorageError", "#[derive(Debug)]\n#[non_exhaustive]\npub enum StorageError")
    if opaque:
        source = original.mutate_once(source, "use std::{io, panic};\n",
            "use std::{io, panic};\n" + fixtures()["opaque_declarations"])
    return source


def source_fidelity(application, pieces, source):
    require(source == render(pieces), "full declaration/contract/body source changed")
    ledger = original.read_json(original.EVIDENCE / "source-ledger.json")
    for name, entry in ledger["items"].items():
        lines = (application / entry["source"]).read_text().splitlines()
        require("\n".join(lines[entry["line"] - 1:]).startswith(pieces[name]), "native item line changed: " + name)
    attributes = "#[derive(Debug)]\n#[non_exhaustive]\n" + pieces["StorageError"]
    require(attributes in (application / original.ERROR).read_text() and attributes in source,
            "full StorageError declaration or native attributes changed")
    result = {"declarations": {}, "bodies": {}}
    for name in RECORDS:
        native = pieces[name].replace("pub(crate)", "pub").replace("pub(super)", "pub")
        adapted = re.sub(r"(?m)^(    )([a-z_]+):", r"\1pub \2:", native)
        require(adapted in source, "record fields or types changed: " + name)
        result["declarations"][name] = {"native_sha256": original.sha256(pieces[name].encode()),
            "adapted_sha256": original.sha256(adapted.encode())}
    result["declarations"]["StorageError"] = {"attributed_sha256": original.sha256(attributes.encode()),
        "complete_enum_byte_identical": True}
    for name in METHODS:
        checked = original.body(original.item(source, "    pub fn " + name + "("))
        require(checked == original.body(pieces[name]), "native method body changed: " + name)
        result["bodies"][name] = original.sha256(checked.encode())
    result["source_sha256"] = original.sha256(source.encode())
    return result


def sysroot_for(explicit=None):
    if explicit is not None:
        root = explicit.resolve()
    else:
        home = Path(os.environ.get("RUSTUP_HOME", Path.home() / ".rustup"))
        candidates = sorted(path for path in (home / "toolchains").glob("1.99.1*") if path.is_dir())
        require(len(candidates) == 1, "provide --sysroot for the compatible installed Rust toolchain")
        root = candidates[0].resolve()
    require((root / "bin/rustc").is_file(), "installed Rust sysroot missing")
    return root


def environment_for(output, solver, sysroot):
    environment = {key: value for key, value in os.environ.items()
                   if key in ("PATH", "SYSTEMROOT", "WINDIR", "LANG", "LC_ALL")}
    for name in ("home", "cargo", "rustup", "tmp", "xdg-cache", "xdg-config", "xdg-data"):
        (output / name).mkdir()
    environment.update(HOME=str(output / "home"), CARGO_HOME=str(output / "cargo"),
        CARGO_TARGET_DIR=str(output / "unused-target"), CARGO_NET_OFFLINE="true",
        RUSTUP_HOME=str(output / "rustup"), RUSTUP_TOOLCHAIN="1.99.1", VERUS_USE_RUSTUP="0",
        VERUS_IN_VARGO="1", VERUS_Z3_PATH=str(solver), GIT_OPTIONAL_LOCKS="0",
        PYTHONDONTWRITEBYTECODE="1", TMPDIR=str(output / "tmp"),
        XDG_CACHE_HOME=str(output / "xdg-cache"), XDG_CONFIG_HOME=str(output / "xdg-config"),
        XDG_DATA_HOME=str(output / "xdg-data"),
        PATH=str(sysroot / "bin") + os.pathsep + environment.get("PATH", ""),
        DYLD_LIBRARY_PATH=str(sysroot / "lib"), LD_LIBRARY_PATH=str(sysroot / "lib"))
    return environment


def native_state(application, environment):
    def git(*arguments):
        return subprocess.check_output(["git", "-c", "core.fsmonitor=false", *arguments],
            cwd=application, env=environment)
    require(git("rev-parse", "HEAD").decode().strip() == original.PIN, "native pin changed")
    require(not git("status", "--porcelain=v1", "--untracked-files=all"), "native checkout is dirty")
    hashes = {}
    for entry in git("ls-tree", "-rz", "--full-tree", "HEAD").split(b"\0"):
        if not entry:
            continue
        metadata, relative = entry.split(b"\t", 1)
        mode, kind, expected = metadata.split()
        require(kind == b"blob" and mode in (b"100644", b"100755"), "unsupported native tree entry")
        path = application / relative.decode()
        require(path.is_file() and not path.is_symlink(), "missing native source")
        raw = path.read_bytes()
        actual = hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()
        require(actual == expected.decode(), "native source differs from pinned blob: " + str(path))
        hashes[relative.decode()] = original.sha256(raw)
    require(len(hashes) == 132, "pinned complete native tree size changed")
    return {"pin": original.PIN, "clean": True, "sources": hashes}


def guarded_paths(verifier, solver, sysroot, application, native):
    paths = [Path(__file__), FIXTURES, ROOT / "rust-toolchain.toml", sysroot / "bin/rustc"]
    paths += [application / relative for relative in native["sources"]]
    for directory in (ROOT / "source", ROOT / "dependencies", sysroot / "lib"):
        paths += [path for path in directory.rglob("*") if path.is_file()
                  and "__pycache__" not in path.parts and "target" not in path.parts]
    paths += [ROOT / relative for relative in fixtures()["base_inputs"]]
    paths += [path for path in verifier.parent.iterdir() if path.is_file()]
    return tool_artifacts(verifier, solver, paths)


def audit_snapshot(snapshot, verifier):
    result = {}
    for name, prefix in (("file-hashes.json", ROOT), ("source-hashes.json", ROOT),
            ("rust-build-input-hashes.json", ROOT), ("runtime-hashes.json", verifier.parent),
            ("toolchain-hashes.json", Path("/")), ("guarded-inputs.json", Path("/"))):
        ledger = original.read_json(snapshot / name)
        require(all((prefix / relative).is_file() and digest(prefix / relative) == expected
                    for relative, expected in ledger.items()), "optional snapshot ledger differs: " + name)
        result[name] = {"count": len(ledger), "sha256": digest(snapshot / name)}
    for name, entry in original.read_json(snapshot / "native-hashes.json").items():
        require(all(digest(Path(entry["root"]) / relative) == expected
            for relative, expected in entry["sources"].items()), "optional native snapshot changed: " + name)
        result[name] = len(entry["sources"])
    return result


def validate_positive(data, code, stderr, artifact, library, source, crate, expectation=None):
    function = original.validate_positive(data, code, stderr, artifact, library, source, crate,
                                         fixtures()["expected"][expectation or crate])
    inference = data["verus-infer"]
    require(type(inference.get("schema_version")) is int and inference["schema_version"] == 1,
            "untyped report schema")
    for row in inference["verification_scope"]["native_functions"]:
        require(row["source"].startswith(str(source) + ":"), "scope row belongs to another source")
    require(function["metrics"].get("cache") == "miss"
            and function["metrics"].get("cache_policy") == "candidate_cache", "fresh inference work missing")
    return function


def validate_certificate(raw, crate="provider", dependencies=None):
    offset, certificate = original.metadata(raw)
    expected = fixtures()["certificate"]
    require(type(certificate.get("schema")) is int and certificate["schema"] == INTERFACE_SCHEMA
            and certificate.get("policy") == INTERFACE_POLICY, "certificate identity changed")
    require_no_foreign_dependencies(certificate)
    require(certificate.get("producer") == expected["producer"], "certificate producer changed")
    require(all(certificate.get(key) is True for key in PROOFS), "certificate guarantees incomplete")
    proofs = certificate["proofs"]
    require(type(proofs) is list and len(proofs) == 1, "certificate proof scope changed")
    require(all(proofs[0][1].get(key) is True for key in PROOFS), "certificate row proof flags untyped")
    identity = proofs[0][0]["path"]["krate"]["Id"]
    require(type(identity) is list and len(identity) == 2 and identity[0] == crate
            and type(identity[1]) is int, "certificate crate identity changed")
    if crate == "provider":
        require(proofs == expected["proofs"], "provider comparison proof identity changed")
    else:
        require(proofs[0][0]["path"]["segments"] == ["ordering"], "consumer inference identity changed")
    require(certificate["dependencies"] == (dependencies or []), "certificate dependency binding changed")
    require(type(certificate["summaries"]) is str and re.fullmatch(r"[0-9a-f]{64}", certificate["summaries"]),
            "certificate summary fingerprint malformed")
    models = certificate["trusted_models"]
    require(type(models) is list and len(models) == expected["trusted_model_fingerprints"]
            and original.sha256(json.dumps(models, sort_keys=True).encode()) == expected["trusted_models_sha256"]
            and all(entry[0]["path"]["krate"] in ("Core", "Alloc", "Vstd") for entry in models),
            "standard-model inventory changed or application theorem assumed")
    return certificate, [{"crate_id": identity[1], "fingerprint": original.sha256(raw[offset:-8])}]


def corrupt_certificate(raw, fault):
    offset, certificate = original.metadata(raw)
    changed = copy.deepcopy(certificate)
    diagnostics = {
        "policy": "incompatible inference schema, safety policy, or producer",
        "schema": "incompatible inference schema, safety policy, or producer",
        "producer": "incompatible inference schema, safety policy, or producer",
        "proof": "incomplete inference proof guarantees",
        "row_proof": "incomplete inference proof guarantees",
        "summaries": "inferred contract fingerprint mismatch",
        "trusted_models": "stale or missing trusted-model fingerprint",
        "truncated": "invalid inference proof metadata",
    }
    if fault == "truncated":
        return raw[:offset] + b'{"schema":' + b"VRINFER1", diagnostics[fault]
    if fault in ("policy", "producer"):
        changed[fault] = "invalid"
    elif fault == "schema":
        changed["schema"] = 0
    elif fault == "proof":
        changed["exactness"] = False
    elif fault == "row_proof":
        changed["proofs"][0][1]["soundness"] = False
    elif fault == "summaries":
        changed["summaries"] = "invalid-summary"
    elif fault == "trusted_models":
        changed["trusted_models"][0][1] = "invalid-model"
    else:
        raise ValueError("unknown certificate fault")
    return raw[:offset] + json.dumps(changed, separators=(",", ":")).encode() + b"VRINFER1", diagnostics[fault]


def validate_cache(entries):
    require(len(entries) == 2 and all(type(original.read_json(path)["version"]) is str
        and original.read_json(path)["version"] == CACHE_VERSION for path in entries), "cache identity/count changed")


def validate_bindings(row):
    for key in ("source", "stdout", "stderr"):
        require(digest(Path(row[key])) == row[key + "_sha256"], "case input/log binding changed: " + key)
    require(original.read_json(row["stdout"]) == row["report"], "case report is not the actual compiler output")
    for key in ("artifact", "library"):
        path = Path(row[key])
        expected = row[key + "_sha256"]
        require((path.is_file() and digest(path) == expected) if expected is not None else not path.exists(),
                "case output binding changed: " + key)
    for path, expected in row.get("imports", {}).items():
        require(digest(Path(path)) == expected, "certified import bytes changed")
    require(row.get("no_cheating") is True and row.get("default_erasure") is True,
            "proof flags changed")
    require(row["source_sha256"] == original.sha256(expected_source(row["label"]).encode()),
            "source is not the exact authored/native fixture for this case")
    command = row["command"]
    for flag in ("--no-cheating", "--compile", "--output-json", "--export", "-o", "--crate-name", "--sysroot"):
        require(command.count(flag) == 1, "missing/duplicate proof command flag: " + flag)
    require(command[1] == row["source"] and command[command.index("--export") + 1] == row["artifact"]
            and command[command.index("-o") + 1] == row["library"], "command source/output binding changed")
    require(not any(argument.startswith(("--no-verify", "--verify-", "--no-lifetime", "--no-auto-recommends"))
                    for argument in command), "proof scope/erasure bypass in command")
    require(not any(argument.startswith(("--import=", "--extern=")) for argument in command),
            "noncanonical import command flags")
    if row["imports"]:
        require(command.count("--import") == command.count("--extern") == 1, "missing certified import flags")
        interfaces = [path for path in row["imports"] if Path(path).suffix == ".vir"]
        libraries = [path for path in row["imports"] if Path(path).suffix == ".rlib"]
        require(len(row["imports"]) == 2 and len(interfaces) == len(libraries) == 1,
                "certified import requires exactly one VIR and one rlib")
        for flag, expected in (("--import", interfaces[0]), ("--extern", libraries[0])):
            name, separator, path = command[command.index(flag) + 1].partition("=")
            require(bool(name) and separator == "=" and path == expected,
                    "command import binding changed: " + flag)
    else:
        require("--import" not in command and "--extern" not in command, "unbound import command flags")


def expectation_for(label):
    if label.startswith("boundary_identity_"):
        return "boundary_identity"
    if label.startswith("boundary_implicit_drop_"):
        return "boundary_implicit_drop"
    return "consumer" if label.startswith("consumer_") else "provider"


def validate_negative(data, code, stderr, artifact, category, diagnostic):
    if category != "lifetime":
        return original.validate_negative(data, code, stderr, artifact, category, diagnostic)
    require(type(code) is int and code != 0 and diagnostic in stderr, "lifetime escape did not reject")
    results, inference = data["verification-results"], data["verus-infer"]
    require(results.get("success") is False and results.get("encountered-error") is True
            and results.get("encountered-vir-error") is False
            and results.get("is-verifying-entire-crate") is True, "wrong lifetime boundary rejection")
    require(all(type(results.get(key)) is int and results[key] == 0 for key in ("verified", "errors")),
            "lifetime rejection must not claim SMT proof work")
    require(inference.get("crate_success") is False and inference.get("interface_written") is False
            and not artifact.exists(), "lifetime failure retained seeded export")
    require(not any(marker in stderr.lower() for marker in
        ("panicked at", "resource limit", "timed out", "out of memory", "killed by signal")), "runtime failure is not lifetime evidence")


def validate_matrix(receipt):
    rows = receipt["cases"]
    require(len(rows) == len(required_labels()) and {row["label"] for row in rows} == required_labels(),
            "incomplete replay case matrix")
    commands = receipt["commands"]
    require(len(commands) == len(rows), "compiler command count differs")
    for row, command in zip(rows, commands):
        require(row["label"] == command["label"] and row["command"] == command["argv"]
                and type(row["returncode"]) is int and row["returncode"] == command["exit_code"],
                "compiler invocation/report binding differs")
        validate_bindings(row)
        if row["category"] == "positive":
            expectation = expectation_for(row["label"])
            crate = "provider" if expectation == "provider" else "consumer"
            validate_positive(row["report"], row["returncode"], Path(row["stderr"]).read_text(),
                Path(row["artifact"]), Path(row["library"]), Path(row["source"]), crate, expectation)
            dependency = None
            if crate == "consumer":
                imported = [Path(path) for path in row["imports"] if Path(path).suffix == ".vir"]
                require(len(imported) == 1, "consumer must bind one certified provider")
                _certificate, dependency = validate_certificate(imported[0].read_bytes())
                require(row["report"]["verus-infer"]["imported_inference_dependencies"] == dependency,
                        "consumer imported fingerprint differs")
            validate_certificate(Path(row["artifact"]).read_bytes(), crate, dependency)
        else:
            validate_negative(row["report"], row["returncode"], Path(row["stderr"]).read_text(),
                Path(row["artifact"]), row["category"], row["diagnostic"])
            if row["label"] != "original_frontend_blocker":
                require(type(row["seeded_export_sha256"]) is str and len(row["seeded_export_sha256"]) == 64,
                        "negative was not seeded")
    for mode in ("debug", "release"):
        provider = next(row for row in rows if row["label"] == "provider_" + mode + "_repeat")
        seed = Path(provider["artifact"]).read_bytes()
        for row in (entry for entry in rows if entry["label"].endswith("_" + mode)
                    or entry["label"].startswith("import_" + mode + "_")):
            if row["category"] != "positive":
                require(row["seeded_export_sha256"] == original.sha256(seed), "negative seed is not this mode's provider")
            if row["label"].startswith("import_"):
                fault = row["label"].removeprefix("import_" + mode + "_")
                expected, diagnostic = corrupt_certificate(seed, fault)
                imported = [Path(path) for path in row["imports"] if Path(path).suffix == ".vir"]
                require(len(imported) == 1 and imported[0].read_bytes() == expected
                        and row["diagnostic"] == diagnostic, "import control is not the intended certificate mutation")
            if row["imports"]:
                libraries = [path for path in row["imports"] if Path(path).suffix == ".rlib"]
                require(libraries == [provider["library"]], "import is not the exact selected provider library")


def required_labels():
    labels = {"original_frontend_blocker"}
    for mode in ("debug", "release"):
        labels.update("provider_" + mode + "_" + phase for phase in ("cold", "repeat"))
        labels.add("consumer_" + mode)
        labels.update(label + "_" + mode for label in original.controls(render_from_fixtures()))
        labels.update("import_" + mode + "_" + fault for fault in FAULTS)
        labels.update("boundary_" + name + "_" + mode for name in fixtures()["boundaries"])
    return labels


def render_from_fixtures():
    pieces = {name: (original.EVIDENCE / "native-items" / (name + ".rs")).read_text().rstrip("\n")
              for name in original.read_json(original.EVIDENCE / "source-ledger.json")["items"]}
    return render(pieces)


def expected_source(label):
    source = render_from_fixtures()
    if label.startswith("provider_"):
        return source
    consumer = (original.EVIDENCE / "consumer.verus.rs").read_text()
    if label.startswith(("consumer_", "import_")):
        return consumer
    if label.startswith("boundary_"):
        for name, body in fixtures()["boundaries"].items():
            if label in ("boundary_" + name + "_debug", "boundary_" + name + "_release"):
                return consumer.rsplit("}", 1)[0] + body + "}\n"
    if label == "original_frontend_blocker":
        return original.render({name: (original.EVIDENCE / "native-items" / (name + ".rs")).read_text().rstrip("\n")
            for name in original.read_json(original.EVIDENCE / "source-ledger.json")["items"]}, full_errors=True)
    for name, (mutant, _category) in original.controls(source).items():
        if label in (name + "_debug", name + "_release"):
            return mutant
    raise ValueError("unexpected case label")


def replay(args):
    output, verifier, solver = args.output.resolve(), args.verus.resolve(), args.solver.resolve()
    require(not output.exists() and not output.is_relative_to(ROOT), "output must be fresh and outside repository")
    output.mkdir(parents=True)
    receipt = {"success": False, "status": "incomplete", "commands": [], "cases": [],
        "identity": list(ENGINE_IDENTITY), "python_optimized": not __debug__,
        "claim": "original extracted native method, full enum, explicit authored opaque datatypes; conditional selection/frame only",
        "vstd_freshly_verified": False, "whole_crate_recovery_proved": False}
    guard = None
    with replay_receipt(output, receipt):
        sysroot = sysroot_for(args.sysroot)
        environment = environment_for(output, solver, sysroot)
        application = args.application.resolve()
        receipt["native_before"] = native_state(application, environment)
        if args.snapshot is not None:
            receipt["snapshot_before"] = audit_snapshot(args.snapshot, verifier)
        guard = ArtifactGuard(guarded_paths(verifier, solver, sysroot, application, receipt["native_before"]))
        original.write_json(output / "guarded-inputs-before.json", guard.initial)
        run = LoggedCommands(output, output, receipt["commands"], timeout=args.timeout)
        try:
            pieces = original.extract(application)
            source = render(pieces)
            receipt["source_fidelity"] = source_fidelity(application, pieces, source)
            source_path, consumer = output / "provider.rs", output / "consumer.rs"
            source_path.write_text(source)
            consumer.write_bytes((original.EVIDENCE / "consumer.verus.rs").read_bytes())

            def invoke(label, path, crate="provider", category="positive", debug=True,
                       dependency=None, seed=None, diagnostic=None, cache=None):
                directory = output / label
                directory.mkdir()
                artifact, library = directory / (crate + ".vir"), directory / ("lib" + crate + ".rlib")
                require(not artifact.exists() and not library.exists(), "case output not fresh")
                if seed is not None:
                    artifact.write_bytes(seed)
                inputs = ArtifactGuard([path, *(dependency or [])])
                env = dict(environment, VERUS_INFER_CACHE=str(cache or directory / "cache"))
                command = [str(verifier), str(path), "--sysroot", str(sysroot), "--crate-type", "lib",
                    "--crate-name", crate, "--edition=2021", "--no-cheating", "--output-json", "--num-threads", "1",
                    "--compile", "--export", str(artifact), "-o", str(library),
                    "-C", "debug-assertions=" + ("yes" if debug else "no")]
                imports = {str(entry): digest(entry) for entry in dependency or []}
                if dependency is not None:
                    command += ["--extern", "provider=" + str(dependency[1]), "--import", "provider=" + str(dependency[0])]
                code = run(label, command, env)
                inputs.check()
                stdout, stderr = output / (label + ".stdout"), output / (label + ".stderr")
                row = {"label": label, "category": category, "returncode": code, "command": command,
                    "source": str(path), "source_sha256": digest(path), "stdout": str(stdout),
                    "stdout_sha256": digest(stdout), "stderr": str(stderr), "stderr_sha256": digest(stderr),
                    "artifact": str(artifact), "artifact_sha256": digest(artifact) if artifact.is_file() else None,
                    "library": str(library), "library_sha256": digest(library) if library.is_file() else None,
                    "imports": imports, "seeded_export_sha256": original.sha256(seed) if seed is not None else None,
                    "default_erasure": True, "no_cheating": True, "diagnostic": diagnostic,
                    "report": original.read_json(stdout)}
                receipt["cases"].append(row)
                validate_bindings(row)
                if category == "positive":
                    validate_positive(row["report"], code, stderr.read_text(), artifact, library, path, crate,
                                      expectation_for(label))
                else:
                    validate_negative(row["report"], code, stderr.read_text(), artifact, category, diagnostic)
                return artifact, library, row["report"]

            for mode, debug in (("debug", True), ("release", False)):
                cache = output / ("provider-cache-" + mode)
                for phase in ("cold", "repeat"):
                    artifact, library, data = invoke("provider_" + mode + "_" + phase, source_path, debug=debug, cache=cache)
                    certificate, dependency_identity = validate_certificate(artifact.read_bytes())
                seed = artifact.read_bytes()
                imported_artifact, _library, data = invoke("consumer_" + mode, consumer, "consumer", debug=debug,
                                                         dependency=(artifact, library))
                require(data["verus-infer"]["imported_inference_dependencies"] == dependency_identity,
                        "fresh consumer did not bind provider certificate")
                validate_certificate(imported_artifact.read_bytes(), "consumer", dependency_identity)
                for label, (mutant, category) in original.controls(source).items():
                    path = output / (label + "_" + mode + ".rs")
                    path.write_text(mutant)
                    invoke(label + "_" + mode, path, category=category, debug=debug, seed=seed)
                for fault in FAULTS:
                    raw, diagnostic = corrupt_certificate(seed, fault)
                    dependency = output / ("dependency_" + mode + "_" + fault + ".vir")
                    dependency.write_bytes(raw)
                    invoke("import_" + mode + "_" + fault, consumer, "consumer", category="import", debug=debug,
                           dependency=(dependency, library), seed=seed, diagnostic=diagnostic)
                for name in fixtures()["boundaries"]:
                    label = "boundary_" + name + "_" + mode
                    path = output / (label + ".rs")
                    path.write_text(expected_source(label))
                    category, diagnostic = {
                        "identity": ("positive", None), "implicit_drop": ("positive", None),
                        "error_operation": ("frontend", "`core::io::error::impl&%3::raw_os_error` is not supported"),
                        "location_operation": ("frontend", "`core::panic::location::impl&%6::line` is not supported"),
                        "explicit_drop": ("frontend", "`core::mem::drop` is not supported"),
                        "lifetime_escape": ("lifetime", "lifetime may not live long enough"),
                    }[name]
                    boundary_artifact, _library, boundary_data = invoke(label, path, "consumer", category, debug,
                        dependency=(artifact, library), seed=seed if category != "positive" else None, diagnostic=diagnostic)
                    if category == "positive":
                        require(boundary_data["verus-infer"]["imported_inference_dependencies"] == dependency_identity,
                                "boundary probe imported a different provider")
                        validate_certificate(boundary_artifact.read_bytes(), "consumer", dependency_identity)
            blocker = output / "original_frontend_blocker.rs"
            blocker.write_text(original.render(pieces, full_errors=True))
            invoke("original_frontend_blocker", blocker, category="frontend",
                   diagnostic="`core::io::error::Error` is not supported")
            require("`core::panic::location::Location` is not supported" in
                    (output / "original_frontend_blocker.stderr").read_text(), "original Location blocker changed")
            validate_matrix(receipt)
            for mode in ("debug", "release"):
                entries = list((output / ("provider-cache-" + mode)).glob("*.json"))
                validate_cache(entries)
            receipt["status"] = "full_declaration_opaque_conditional_selection_verified"
        finally:
            guard.check()
            original.write_json(output / "guarded-inputs-after.json", guard.snapshot())
            receipt["native_after"] = native_state(application, environment)
            require(receipt["native_before"] == receipt["native_after"], "native source changed")
            if args.snapshot is not None:
                receipt["snapshot_after"] = audit_snapshot(args.snapshot, verifier)
                require(receipt["snapshot_before"] == receipt["snapshot_after"], "optional snapshot changed")
            original.write_json(output / "artifacts.json", {str(path.relative_to(output)): digest(path)
                for path in sorted(output.rglob("*")) if path.is_file()})
    print(json.dumps({"receipt": str(output / "receipt.json"), "status": receipt["status"], "cases": len(receipt["cases"])}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("application", "verus", "solver", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--sysroot", type=Path)
    parser.add_argument("--snapshot", type=Path, help="optional, additional historical provenance audit; never disables proof gates")
    parser.add_argument("--timeout", type=positive_timeout, default=180)
    replay(parser.parse_args())


if __name__ == "__main__":
    main()
