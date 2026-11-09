"""Bounded, explicitly scaffolded proof of the pinned redb slot selector."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

from check_redb_page_size import item
from infer_eval_support import digest
from infer_experiment import LoggedCommands, positive_timeout, replay_receipt, require, require_exact
from infer_transfer_identity import CACHE_VERSION, ENGINE_IDENTITY, INTERFACE_POLICY, INTERFACE_SCHEMA


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "evidence/transfer-redb-slot-selection"
LIVE_EVIDENCE = ROOT / "evidence/transfer-two-project-integration82-final2"
PIN = "c82ab865bb362bc4af067d332adc44978ff64813"
CHECKPOINT = "3b05c1912b9d7314ad00c832c4065f4362562e02"
BASELINE_PATCH = "545e330ee87d539aa727154a6121f251f557a18867be62517a9867592a372e12"
BASELINE = Path("/private/tmp/verus-cycle77-review-fixes.XCkOUU")
HEADER = "src/tree_store/page_store/header.rs"
TRACKER = "src/transaction_tracker.rs"
BASE = "src/tree_store/page_store/base.rs"
BTREE = "src/tree_store/btree_base.rs"
ERROR = "src/error.rs"
MARKERS = {
    "UnrepairedDatabaseHeader": (HEADER, "pub(super) struct UnrepairedDatabaseHeader"),
    "DatabaseHeader": (HEADER, "pub(super) struct DatabaseHeader"),
    "TransactionHeader": (HEADER, "pub(super) struct TransactionHeader"),
    "PageNumber": (BASE, "pub(crate) struct PageNumber"),
    "BtreeHeader": (BTREE, "pub(crate) struct BtreeHeader"),
    "StorageError": (ERROR, "pub enum StorageError"),
    "select_primary_slot": (HEADER, "    fn select_primary_slot("),
    "primary_slot": (HEADER, "    pub(super) fn primary_slot("),
    "secondary_slot": (HEADER, "    pub(super) fn secondary_slot("),
    "swap_primary_slot": (HEADER, "    pub(super) fn swap_primary_slot("),
}


def sha256(content):
    return hashlib.sha256(content).hexdigest()


def body(source):
    return source[source.index("{"):]


def extract(application):
    ledger = read_json(EVIDENCE / "source-ledger.json")
    require(ledger["pin"] == PIN, "source ledger pin changed")
    for relative, expected in ledger["sources"].items():
        require(digest(application / relative) == expected, "pinned source changed: " + relative)
    for relative, declaration in ((HEADER, "const TRANSACTION_SIZE: usize = 128;"),
                                  (BTREE, "pub(super) type Checksum = u128;"),
                                  ("src/lib.rs", "pub type Result<T = (), E = StorageError> = core::result::Result<T, E>;")):
        require(declaration in (application / relative).read_text(), "native scaffold alias/constant changed")
    pieces = {name: item((application / relative).read_text(), marker)
              for name, (relative, marker) in MARKERS.items()}
    tracker = (application / TRACKER).read_text()
    declaration = "#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]\npub(crate) struct TransactionId(u64);"
    require(declaration in tracker, "native TransactionId declaration changed")
    pieces["TransactionId"] = declaration
    for name, piece in pieces.items():
        require(sha256(piece.encode()) == ledger["items"][name]["sha256"], "native item changed: " + name)
        require((EVIDENCE / "native-items" / (name + ".rs")).read_text() == piece + "\n",
                "frozen native item changed: " + name)
    return pieces


def render(pieces, full_errors=False):
    contracts = json.loads((EVIDENCE / "contracts.json").read_text())
    transaction = pieces["TransactionId"].replace("pub(crate)", "pub").replace("TransactionId(u64)", "TransactionId(pub u64)")
    error = pieces["StorageError"] if full_errors else "pub enum StorageError { Corrupted(String) }"
    source = ("use vstd::prelude::*;\nuse std::{io, panic};\n"
              "#[verus_verify(infer_derive(PartialEq, PartialOrd, Ord))]\n"
              + transaction + "\nverus! {\n"
              "pub type Checksum = u128;\nconst TRANSACTION_SIZE: usize = 128;\n"
              + error + "\n"
              "pub type Result<T = (), E = StorageError> = core::result::Result<T, E>;\n")
    for name in ("PageNumber", "BtreeHeader", "TransactionHeader", "DatabaseHeader", "UnrepairedDatabaseHeader"):
        declaration = pieces[name].replace("pub(crate)", "pub").replace("pub(super)", "pub")
        source += re.sub(r"(?m)^(    )([a-z_]+):", r"\1pub \2:", declaration) + "\n"
    specification = (EVIDENCE / "selection.spec.rs").read_text()
    if full_errors:
        specification = specification.replace('} else { "Both commit slots are corrupted"@ },',
            '} else { "Both commit slots are corrupted"@ },\n            Err(_) => false,')
    source += specification + "\n"
    for owner, names in (("DatabaseHeader", ("primary_slot", "secondary_slot", "swap_primary_slot")),
                         ("UnrepairedDatabaseHeader", ("select_primary_slot",))):
        source += "impl " + owner + " {\n"
        for name in names:
            source += contracts[name] + body(pieces[name]) + "\n"
        source += "}\n"
    return source + (EVIDENCE / "goals.verus.rs").read_text() + "}\n"


def native_harness(pieces):
    source = ("use std::{io, panic};\n"
              "type Checksum = u128;\nconst TRANSACTION_SIZE: usize = 128;\n"
              "pub type Result<T = (), E = StorageError> = core::result::Result<T, E>;\n")
    source += pieces["TransactionId"].replace("pub(crate)", "pub") + "\n"
    for name in ("PageNumber", "BtreeHeader", "TransactionHeader", "DatabaseHeader",
                 "UnrepairedDatabaseHeader", "StorageError"):
        if name in ("PageNumber", "BtreeHeader"):
            source += "#[derive(Debug, Copy, Clone, Eq, PartialEq)]\n"
        elif name in ("TransactionHeader", "DatabaseHeader"):
            source += "#[derive(Clone)]\n"
        elif name == "StorageError":
            source += "#[derive(Debug)]\n#[non_exhaustive]\n"
        source += pieces[name].replace("pub(crate)", "pub").replace("pub(super)", "pub") + "\n"
    for owner, names in (("DatabaseHeader", ("primary_slot", "secondary_slot", "swap_primary_slot")),
                         ("UnrepairedDatabaseHeader", ("select_primary_slot",))):
        source += "impl " + owner + " {\n"
        for name in names:
            source += pieces[name].replace("pub(super)", "pub") + "\n"
        source += "}\n"
    return source + (EVIDENCE / "native_tests.rs").read_text()


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON key: " + key)
        result[key] = value
    return result


def read_json(file_path):
    return json.loads(Path(file_path).read_text(), object_pairs_hook=unique_object)


def write_json(file_path, value):
    Path(file_path).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def environment_for(output, verifier):
    environment = {key: value for key, value in os.environ.items()
                   if not key.startswith("GIT_") and key not in (
                       "RUSTFLAGS", "RUSTDOCFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER",
                       "VERUS_INFER_CACHE", "VERUS_EXTRA_ARGS")}
    environment.update(GIT_OPTIONAL_LOCKS="0", PYTHONDONTWRITEBYTECODE="1", CARGO_NET_OFFLINE="true",
                       CARGO_HOME=str(output / "unused-cargo-home"),
                       CARGO_TARGET_DIR=str(output / "unused-cargo-target"),
                       VERUS_Z3_PATH=str(verifier.parent / "z3"))
    return environment


def audit_baseline(baseline, application, environment):
    manifest = read_json(baseline / "handoff/manifest.json")
    require(manifest["checkpoint"] == CHECKPOINT and manifest["identity"] == list(ENGINE_IDENTITY)
            and manifest["patch_sha256"] == BASELINE_PATCH, "baseline identity changed")
    require(digest(baseline / "handoff/combined.apply_patch") == BASELINE_PATCH, "baseline patch changed")
    result = {"ledger_hashes": {file_path.name: digest(file_path)
                               for file_path in sorted((baseline / "handoff").glob("*.json"))}}
    for name, prefix in (("file-hashes", ROOT), ("source-hashes", ROOT),
                         ("rust-build-input-hashes", ROOT), ("runtime-hashes", baseline / "tools77"),
                         ("toolchain-hashes", Path("/")), ("guarded-inputs", Path("/"))):
        ledger = read_json(baseline / "handoff" / (name + ".json"))
        changed = [relative for relative, expected in ledger.items()
                   if not (prefix / relative).is_file() or digest(prefix / relative) != expected]
        require(not changed, "snapshot mismatch: " + name + ": " + str(changed[:5]))
        result[name] = {"count": len(ledger), "mismatches": changed}
    for name, entry in read_json(baseline / "handoff/native-hashes.json").items():
        native = Path(entry["root"])
        require(name != "redb" or native.resolve() == application.resolve(), "redb root differs from native ledger")
        for relative, expected in entry["sources"].items():
            require(digest(native / relative) == expected, "native snapshot mismatch: " + relative)
        head = subprocess.check_output(["git", "-C", str(native), "rev-parse", "HEAD"], env=environment, text=True).strip()
        clean = subprocess.check_output(["git", "-C", str(native), "status", "--porcelain=v1", "--untracked-files=all"],
                                        env=environment, text=True) == ""
        require(head == entry["pin"] and clean, "native checkout pin/state changed: " + name)
        result["native-" + name] = {"count": len(entry["sources"]), "head": head, "clean": clean}
    for relative, entry in read_json(baseline / "handoff/inventory.json").items():
        require(digest(ROOT / relative) == entry["candidate_sha256"], "baseline inventory changed")
        try:
            original = subprocess.check_output(["git", "show", "HEAD:" + relative], cwd=ROOT,
                                               env=environment, stderr=subprocess.DEVNULL)
        except subprocess.CalledProcessError:
            require(entry["head_sha256"] is None, "checkpoint inventory source missing")
        else:
            require(sha256(original) == entry["head_sha256"], "checkpoint inventory hash changed")
    for relative, entry in read_json(baseline / "handoff/correction-delta.json").items():
        require(digest(ROOT / relative) == entry["after"], "review correction snapshot changed")
    result["head"] = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, env=environment, text=True).strip()
    result["branch"] = subprocess.check_output(["git", "symbolic-ref", "--short", "HEAD"], cwd=ROOT, env=environment, text=True).strip()
    result["refs"] = subprocess.check_output(["git", "show-ref"], cwd=ROOT, env=environment, text=True)
    require(result["head"] == CHECKPOINT == read_json(baseline / "handoff/git-state.json")["head"], "checkpoint changed")
    return result


SCOPE_KEYS = ("function", "rust_function", "automatically_derived", "selected", "body_status",
              "inference_candidate", "ordinary_contract")


def scope_rows(data):
    rows = data["verus-infer"]["verification_scope"]["native_functions"]
    require(isinstance(rows, list) and all(isinstance(row, dict) for row in rows), "missing native scope")
    require(len({row["function"] for row in rows}) == len(rows), "duplicate native scope identity")
    for row in rows:
        require(all(type(row.get(key)) is bool for key in
                    ("automatically_derived", "selected", "inference_candidate", "ordinary_contract")),
                "untyped native scope flags")
    return sorted([{key: row[key] for key in SCOPE_KEYS} for row in rows], key=lambda row: row["function"])


def validate_positive(data, code, stderr, artifact, library, source, crate, expected):
    require(type(code) is int and code == 0, "positive compiler failed")
    results, inference = data["verification-results"], data["verus-infer"]
    require(results.get("success") is True and results.get("encountered-error") is False
            and results.get("encountered-vir-error") is False
            and results.get("is-verifying-entire-crate") is True, "positive is not a successful whole-crate proof")
    require(type(results.get("verified")) is int and results["verified"] == expected["verified"]
            and type(results.get("errors")) is int and results["errors"] == 0, "positive proof counts differ")
    require(inference.get("crate_success") is True and inference.get("interface_written") is True
            and inference.get("status") == "verified", "positive publication incomplete")
    scope = inference["verification_scope"]
    require(scope.get("native_collection_completed") is True and scope.get("complete_selected_scope_verified") is True,
            "positive native scope incomplete")
    for key in ("checked_exec_bodies", "checked_proof_bodies"):
        require(type(scope.get(key)) is int and scope[key] == expected[key], "positive checked body counts differ")
    require(scope_rows(data) == expected["native_functions"], "positive native body identities or flags differ")
    checked = [row for row in scope["native_functions"] if row["body_status"].startswith("checked_")]
    require(all(row["source"].startswith(str(source) + ":") for row in checked), "checked bodies came from another source")
    require(sum(row["body_status"] == "checked_exec_body" for row in checked) == expected["checked_exec_bodies"]
            and sum(row["body_status"] == "checked_proof_body" for row in checked) == expected["checked_proof_bodies"],
            "checked scope rows disagree with counts")
    functions = inference.get("functions")
    require(isinstance(functions, list) and len(functions) == 1, "unexpected inference function scope")
    function = functions[0]
    require(function.get("function") == crate + expected["inferred_suffix"]
            and function.get("contract") == expected["contract"], "inferred identity or contract changed")
    require_exact(functions)
    require(function.get("required_checks_passed") is True and function.get("exported") is True
            and function.get("available_to_same_crate_callers") is True, "untyped inferred acceptance flags")
    for obligation in ("soundness", "maximality", "exactness"):
        require(function["proofs"][obligation].get("required") is True
                and function["proofs"][obligation].get("status") == "proved", "incomplete inferred proof")
    work = function["metrics"].get("solver_micros")
    require(type(work) is int and work > 0, "no fresh required-check solver work")
    require(artifact.is_file() and artifact.stat().st_size > 0 and library.is_file()
            and library.stat().st_size > 0, "missing fresh export or erased compiled library")
    require(not any(marker in stderr.lower() for marker in ("panicked at", "resource limit", "timed out", "out of memory")),
            "positive contains resource or runtime failure")
    return function


def validate_negative(data, code, stderr, artifact, category, diagnostic=None):
    require(type(code) is int and code != 0, "negative unexpectedly succeeded")
    results, inference = data["verification-results"], data["verus-infer"]
    require(results.get("success") is False and inference.get("crate_success") is False, "negative success flags differ")
    require(not any(marker in stderr.lower() for marker in
                    ("panicked at", "resource limit", "timed out", "out of memory", "killed by signal")),
            "negative failed for resource/runtime reason")
    if category in ("semantic", "safety", "legacy"):
        require(results.get("encountered-vir-error") is False and results.get("is-verifying-entire-crate") is True,
                "negative did not reach whole-crate proof")
        require(type(results.get("errors")) is int and results["errors"] == 1, "negative proof count changed")
        expected_verified = {"semantic": 22, "safety": 23, "legacy": 0}[category]
        require(type(results.get("verified")) is int and results["verified"] == expected_verified,
                "negative verified count changed")
        marker = "precondition not satisfied" if category == "safety" else "postcondition not satisfied"
        require(marker in stderr, "negative missed expected obligation")
        if category != "legacy":
            other = "postcondition not satisfied" if category == "safety" else "precondition not satisfied"
            require(other not in stderr and "precondition not met" not in stderr, "negative failed a different obligation")
        else:
            require(inference.get("functions") == [] and inference.get("interface_written") is False,
                    "legacy control unexpectedly used inference-aware publication")
    else:
        require(results.get("encountered-vir-error") is True, "expected translation/import rejection")
        require(type(diagnostic) is str and diagnostic in stderr, "wrong frontend/import diagnostic")
    if category != "legacy":
        require(inference.get("interface_written") is False and not artifact.exists(),
                "inference-aware failed run retained or published seeded export")
    return results


def metadata(raw):
    require(raw.endswith(b"VRINFER1"), "missing inference certificate trailer")
    offset = raw.rindex(b'{"schema":')
    return offset, json.loads(raw[offset:-8], object_pairs_hook=unique_object)


def validate_certificate(raw):
    offset, certificate = metadata(raw)
    expected = read_json(LIVE_EVIDENCE / "redb-certificate-expectations.json")
    require(type(certificate["schema"]) is int and certificate["schema"] == INTERFACE_SCHEMA
            and certificate["policy"] == INTERFACE_POLICY, "certificate identity changed")
    require(certificate["producer"] == expected["producer"], "certificate producer changed")
    require(all(certificate.get(key) is True for key in ("soundness", "maximality", "exactness")), "certificate proofs incomplete")
    require(certificate["proofs"] == expected["proofs"]
            and len(certificate["proofs"]) == 1
            and all(certificate["proofs"][0][1].get(key) is True
                    for key in ("soundness", "maximality", "exactness")), "exported inference proof scope changed")
    require(certificate["dependencies"] == [], "unexpected provider inference dependencies")
    require(isinstance(certificate["summaries"], str) and re.fullmatch(r"[0-9a-f]{64}", certificate["summaries"]),
            "malformed exported summary fingerprint")
    models = certificate["trusted_models"]
    require(len(models) == expected["trusted_model_fingerprints"]
            and sha256(json.dumps(models, sort_keys=True).encode()) == expected["trusted_models_sha256"]
            and all(entry[0]["path"]["krate"] in ("Core", "Alloc", "Vstd") for entry in models),
            "standard-model inventory changed or an application model appeared")
    crate_id = certificate["proofs"][0][0]["path"]["krate"]["Id"][1]
    require(type(crate_id) is int, "untyped provider crate identity")
    dependency = [{"crate_id": crate_id, "fingerprint": sha256(raw[offset:-8])}]
    return certificate, dependency


def mutate_once(source, original, changed):
    require(source.count(original) == 1, "control mutation is not unique")
    return source.replace(original, changed, 1)


def controls(source):
    final_return = "\n        Ok(true)\n    }"
    return {
        "wrong_kept_flag": (mutate_once(source, final_return, "\n        Ok(false)\n    }"), "semantic"),
        "ignore_two_phase": (mutate_once(source,
            "if self.inner.two_phase_commit {\n            if self.primary_corrupted {",
            "if false {\n            if self.primary_corrupted {"), "semantic"),
        "equal_counts_as_newer": (mutate_once(source,
            "self.inner.secondary_slot().transaction_id > self.inner.primary_slot().transaction_id;",
            "self.inner.secondary_slot().transaction_id >= self.inner.primary_slot().transaction_id;"), "semantic"),
        "trust_corrupt_secondary": (mutate_once(source, "if secondary_newer && !self.secondary_corrupted {",
                                               "if secondary_newer {"), "semantic"),
        "break_state_frame": (mutate_once(source, "self.primary_slot ^= 1;",
            "self.primary_slot ^= 1;\n        self.recovery_required = !self.recovery_required;"), "semantic"),
        "missing_representation": (source.rsplit("}", 1)[0]
            + "pub fn missing_representation(header: &mut UnrepairedDatabaseHeader) {\n"
              "    let _result = header.select_primary_slot();\n}\n}\n", "safety"),
    }


def replay(args):
    output, verifier = args.output.resolve(), args.verus.resolve()
    require(not output.exists(), "replay output must be fresh")
    require(verifier == (args.baseline / "tools77/verus").resolve(), "only sealed tools77 may run")
    output.mkdir(parents=True)
    environment = environment_for(output, verifier)
    receipt = {"status": "incomplete", "commands": [], "cases": [], "success": False,
               "claim": "conditional reduced-error scaffold with unchanged source bodies; full native selection blocked",
               "identity": list(ENGINE_IDENTITY), "python_optimized": not __debug__,
               "baseline_independently_accepted": False, "native_selection_proved": False}
    run = LoggedCommands(output, ROOT, receipt["commands"], timeout=args.timeout)
    with replay_receipt(output, receipt):
        receipt["baseline_before"] = audit_baseline(args.baseline, args.application, environment)
        pieces = extract(args.application)
        receipt["source_ledger"] = read_json(EVIDENCE / "source-ledger.json")
        checked_inputs = [Path(__file__), Path(__file__).with_name("test_redb_slot_selection.py")]
        checked_inputs += [EVIDENCE / name for name in ("contracts.json", "selection.spec.rs", "goals.verus.rs",
            "consumer.verus.rs", "native_tests.rs", "source-ledger.json", "expected-scope.json", "certificate-expectations.json")]
        checked_inputs += sorted((EVIDENCE / "native-items").glob("*.rs"))
        checked_inputs += [LIVE_EVIDENCE / "redb-certificate-expectations.json"]
        receipt["input_sha256"] = {str(file_path.relative_to(ROOT)): digest(file_path) for file_path in checked_inputs}
        expected = read_json(EVIDENCE / "expected-scope.json")
        source = render(pieces)
        source_path = output / "provider.rs"
        source_path.write_text(source)
        consumer = output / "consumer.rs"
        consumer.write_text((EVIDENCE / "consumer.verus.rs").read_text())
        receipt["body_correspondence"] = {}
        for name in ("primary_slot", "secondary_slot", "swap_primary_slot", "select_primary_slot"):
            checked_body = body(item(source, "    pub fn " + name + "("))
            require(checked_body == body(pieces[name]), "checked named source body differs: " + name)
            receipt["body_correspondence"][name] = {"native_body_sha256": sha256(body(pieces[name]).encode()),
                "checked_body_sha256": sha256(checked_body.encode()), "byte_identical": True}
        receipt["authored_body_sha256"] = {}
        for relative, names in (("selection.spec.rs", ("slot_bits::two_slot_toggle",)),
            ("goals.verus.rs", ("two_phase_goal", "both_corrupt_goal", "fallback_goal", "newer_secondary_goal",
                                "equal_or_stale_goal", "corrupt_secondary_goal", "selected_observation_goal")),
            ("consumer.verus.rs", ("ordering", "imported_selection", "imported_order_goal"))):
            text = (EVIDENCE / relative).read_text()
            for name in names:
                marker = "pub broadcast proof fn " if "::" in name else "pub fn "
                checked_body = body(item(text, marker + name.split("::")[-1] + "("))
                receipt["authored_body_sha256"][name] = sha256(checked_body.encode())

        def invoke(label, file_path, crate="provider", category="positive", debug=True, dependency=None,
                   seed=None, diagnostic=None, cache=None):
            case_output = output / label
            case_output.mkdir()
            artifact, library = case_output / (crate + ".vir"), case_output / ("lib" + crate + ".rlib")
            require(not artifact.exists() and not library.exists(), "nonfresh case outputs")
            if seed is not None:
                artifact.write_bytes(seed)
            environment["VERUS_INFER_CACHE"] = str(cache or case_output / "cache")
            command = [str(verifier), str(file_path), "--crate-type", "lib", "--crate-name", crate,
                       "--edition=2021", "--no-cheating", "--output-json", "--num-threads", "1",
                       "--compile", "--export", str(artifact), "-o", str(library),
                       "-C", "debug-assertions=" + ("yes" if debug else "no")]
            if dependency is not None:
                vir, rlib = dependency
                imported = {str(file_path): digest(file_path) for file_path in dependency}
                command += ["--extern", "provider=" + str(rlib), "--import", "provider=" + str(vir)]
            code = run(label, command, environment)
            stdout, stderr = output / (label + ".stdout"), output / (label + ".stderr")
            data = read_json(stdout)
            row = {"label": label, "category": category, "returncode": code,
                   "source": str(file_path), "source_sha256": digest(file_path),
                   "stdout_sha256": digest(stdout), "stderr_sha256": digest(stderr),
                   "artifact": str(artifact), "artifact_exists": artifact.exists(),
                   "artifact_sha256": digest(artifact) if artifact.is_file() else None,
                   "library_sha256": digest(library) if library.is_file() else None,
                   "seeded_export_sha256": sha256(seed) if seed is not None else None,
                   "default_erasure": True, "no_cheating": True, "report": data}
            if dependency is not None:
                require(imported == {str(file_path): digest(file_path) for file_path in dependency},
                        "import artifacts changed during verification")
                row["imports"] = imported
            receipt["cases"].append(row)
            if category == "positive":
                validate_positive(data, code, stderr.read_text(), artifact, library, file_path, crate, expected[crate])
            else:
                validate_negative(data, code, stderr.read_text(), artifact, category, diagnostic)
                if category == "legacy":
                    require(artifact.exists() and row["artifact_sha256"] != row["seeded_export_sha256"]
                            and not artifact.read_bytes().endswith(b"VRINFER1"), "legacy export observation changed")
                    row["legacy_observation"] = "Failed non-Infer run replaced the seeded export with a new legacy artifact; no cleanup guarantee"
            return artifact, library, data

        for mode, debug in (("debug", True), ("release", False)):
            cache = output / ("provider-cache-" + mode)
            provider = None
            for phase in ("cold", "repeat"):
                artifact, library, data = invoke("provider_" + mode + "_" + phase, source_path,
                                               debug=debug, cache=cache)
                function = data["verus-infer"]["functions"][0]
                require(function["metrics"].get("cache_policy") == "candidate_cache"
                        and function["metrics"].get("cache") == "miss",
                        "candidate cache phase changed")
                certificate, dependency_identity = validate_certificate(artifact.read_bytes())
                provider = artifact, library
            seed = provider[0].read_bytes()
            _artifact, _library, data = invoke("consumer_" + mode, consumer, "consumer", debug=debug,
                                             dependency=provider)
            require(data["verus-infer"]["imported_inference_dependencies"] == dependency_identity,
                    "imported certificate fingerprint differs from checked provider")
            for label, (mutant, category) in controls(source).items():
                file_path = output / (label + "_" + mode + ".rs")
                file_path.write_text(mutant)
                invoke(label + "_" + mode, file_path, category=category, debug=debug, seed=seed)
            offset, certificate = metadata(seed)
            for fault in ("policy", "proof", "summaries", "trusted_models"):
                changed = json.loads(json.dumps(certificate))
                if fault == "policy":
                    changed["policy"] = "invalid-policy"
                    diagnostic = "incompatible inference schema, safety policy, or producer"
                elif fault == "proof":
                    changed["exactness"] = False
                    diagnostic = "incomplete inference proof guarantees"
                elif fault == "summaries":
                    changed["summaries"] = "invalid-summary"
                    diagnostic = "inferred contract fingerprint mismatch"
                else:
                    changed["trusted_models"][0][1] = "invalid-model"
                    diagnostic = "stale or missing trusted-model fingerprint"
                dependency = output / ("dependency_" + mode + "_" + fault + ".vir")
                dependency.write_bytes(seed[:offset] + json.dumps(changed, separators=(",", ":")).encode() + b"VRINFER1")
                invoke("import_" + mode + "_" + fault, consumer, "consumer", category="import", debug=debug,
                       dependency=(dependency, provider[1]), seed=seed, diagnostic=diagnostic)
            legacy = output / ("legacy_" + mode + ".rs")
            legacy.write_text("use vstd::prelude::*; verus! {pub fn false_goal() -> (result: bool) ensures result, { false }}\n")
            invoke("legacy_noninfer_" + mode, legacy, "legacy", category="legacy", debug=debug, seed=seed)

        full = output / "full_native_error_probe.rs"
        full.write_text(render(pieces, full_errors=True))
        _artifact, _library, data = invoke("full_native_error_blocker", full, category="frontend",
            diagnostic="`core::io::error::Error` is not supported")
        require("`core::panic::location::Location` is not supported"
                in (output / "full_native_error_blocker.stderr").read_text(), "native Location blocker changed")
        compiler_paths = [Path(file_path) for file_path in read_json(args.baseline / "handoff/toolchain-hashes.json")
                          if file_path.endswith("/bin/rustc")]
        require(len(compiler_paths) == 1, "ambiguous sealed native Rust compiler")
        native = output / "native_full_error.rs"
        native.write_text(native_harness(pieces))
        for mode, debug in (("debug", True), ("release", False)):
            binary = output / ("native_" + mode)
            require(run("native_compile_" + mode, [str(compiler_paths[0]), "--edition=2021", "--test", "-C", "opt-level=2",
                "-C", "debug-assertions=" + ("yes" if debug else "no"), str(native), "-o", str(binary)], environment) == 0,
                "native test harness did not compile")
            require(run("native_test_" + mode, [str(binary), "--test-threads=1", "--nocapture"], environment) == 0,
                    "native test harness failed")
            stdout = (output / ("native_test_" + mode + ".stdout")).read_text()
            require("native full-error-enum cases: 1600" in stdout and "1 passed; 0 failed" in stdout,
                    "native test count changed")
        receipt["native_finite_tests"] = {"cases_per_build": 1600, "builds": ["debug", "release"],
            "formal_refinement_proved": False, "source_sha256": digest(native)}
        for mode in ("debug", "release"):
            cache_entries = list((output / ("provider-cache-" + mode)).glob("*.json"))
            require(len(cache_entries) == 2 and all(read_json(entry)["version"] == CACHE_VERSION for entry in cache_entries),
                    "candidate cache identity changed")
        receipt["cache_version"] = CACHE_VERSION
        required_labels = {"full_native_error_blocker"}
        for mode in ("debug", "release"):
            required_labels.update({"provider_" + mode + "_cold", "provider_" + mode + "_repeat", "consumer_" + mode,
                                    "legacy_noninfer_" + mode})
            required_labels.update(label + "_" + mode for label in ("wrong_kept_flag", "ignore_two_phase",
                "equal_counts_as_newer", "trust_corrupt_secondary", "break_state_frame", "missing_representation"))
            required_labels.update("import_" + mode + "_" + fault for fault in ("policy", "proof", "summaries", "trusted_models"))
        require(len(receipt["cases"]) == len(required_labels)
                and {row["label"] for row in receipt["cases"]} == required_labels,
                "incomplete or duplicate bounded experiment cases")
        for relative, expected_hash in receipt["input_sha256"].items():
            require(digest(ROOT / relative) == expected_hash, "proof/replay input changed during execution")
        receipt["baseline_after"] = audit_baseline(args.baseline, args.application, environment)
        require(receipt["baseline_after"] == receipt["baseline_before"], "frozen baseline or refs changed")
        receipt["status"] = "bounded_scaffold_verified_native_declaration_blocked"
        write_json(output / "artifacts.json", {str(file_path.relative_to(output)): digest(file_path)
                   for file_path in sorted(output.rglob("*")) if file_path.is_file()})
    print(json.dumps({"receipt": str(output / "receipt.json"), "status": receipt["status"],
                      "cases": len(receipt["cases"])}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--application", type=Path, required=True)
    parser.add_argument("--verus", type=Path, required=True)
    parser.add_argument("--baseline", type=Path, default=BASELINE)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=positive_timeout, default=180)
    replay(parser.parse_args())


if __name__ == "__main__":
    main()
