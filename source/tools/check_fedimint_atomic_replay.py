"""MODEL-ONLY conditional transaction/replay proof; no native async refinement."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time

from infer_transfer_identity import CACHE_VERSION, ENGINE_IDENTITY, INTERFACE_POLICY, INTERFACE_SCHEMA


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "source/tools/fixtures/transfers/transfer-fedimint-atomic-replay"
CHECKPOINT = "3b05c1912b9d7314ad00c832c4065f4362562e02"
PATCH_SHA256 = "545e330ee87d539aa727154a6121f251f557a18867be62517a9867592a372e12"
PIN = "b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236"
CRATE = "fedimint_atomic_replay"
EXECUTIONS = {"replay_decision"}
PROOFS = {
    "initial_invariant", "start_preserves", "staging_preserves", "abort_preserves",
    "commit_preserves", "crash_preserves", "step_preserves", "reachable_invariant",
    "recovered_pairing", "precommit_crash", "all_or_none_commit", "matching_replay_no_effect",
    "conflicting_replay_rejected", "lost_ack_retry", "non_atomic_counterexamples",
    "missing_dedup_counterexample",
}
SPECIFICATIONS = {
    "decision", "paired", "valid_state", "initial", "start", "stage_effects", "stage_marker",
    "abort", "commit", "crash", "step", "reachable", "non_atomic_marker", "non_atomic_effects",
    "without_dedup",
}
SOURCE_HASHES = {
    "fedimint-server/src/consensus/engine.rs": "dffbb300941944f3360ec15377f86f6befd203319de321b52c7e924f901323d1",
    "fedimint-server/src/consensus/db.rs": "b1e896c16e027af6fa6fb01188e918df29480a3d44be0a909717008f3bd1ed05",
    "fedimint-core/src/db/mod.rs": "a44f6b54cae3033e1a5896c09bdff9f737dd1af7419ac14d406bf0ed2043377b",
    "fedimint-rocksdb/src/lib.rs": "5c46f4346f5c869b22f4ae8894c93178323f9fc42e459ec8ce415f53b1e4cf25",
    "fedimint-core/src/session_outcome.rs": "8a41d99350451f9fc5609a661d513069bdc751a38b003097026ededbac8cbdba",
}
NEGATIVE_GOALS = {
    "marker_only": """
pub proof fn false_marker_only()
    ensures paired(crash(non_atomic_marker(start(initial(0, 0), Payload { item: 1, peer: 2 }))).disk),
{}
""",
    "effects_only": """
pub proof fn false_effects_only()
    ensures paired(crash(non_atomic_effects(start(initial(0, 0), Payload { item: 1, peer: 2 }))).disk),
{}
""",
    "missing_dedup": """
pub proof fn false_missing_dedup()
    ensures ({
        let request = Payload { item: 1, peer: 2 };
        let ready = stage_marker(stage_effects(start(initial(0, 0), request)));
        let first = crash(commit(ready, true, false));
        let replay = stage_marker(stage_effects(without_dedup(first, request)));
        commit(replay, true, true).disk.effects.len() == 1
    }),
{}
""",
    "different_peer": """
pub proof fn false_different_peer()
    ensures ({
        let ready = stage_marker(stage_effects(start(initial(0, 0), Payload { item: 1, peer: 2 })));
        start(commit(ready, true, false), Payload { item: 1, peer: 3 }).reply == 1
    }),
{}
""",
    "different_item": """
pub proof fn false_different_item()
    ensures ({
        let ready = stage_marker(stage_effects(start(initial(0, 0), Payload { item: 1, peer: 2 })));
        start(commit(ready, true, false), Payload { item: 2, peer: 2 }).reply == 1
    }),
{}
""",
    "precommit_leak": """
pub proof fn false_precommit_leak()
    ensures crash(stage_marker(stage_effects(start(initial(0, 0), Payload { item: 1, peer: 2 })))).disk.marker is Some,
{}
""",
    "lost_ack_reexec": """
pub proof fn false_lost_ack_reexec()
    ensures ({
        let request = Payload { item: 1, peer: 2 };
        let ready = stage_marker(stage_effects(start(initial(0, 0), request)));
        start(crash(commit(ready, true, false)), request).disk.effects.len() == 2
    }),
{}
""",
    "safety_overflow": """
pub fn unchecked_effect_count(count: u64) -> u64 { count + 1 }
""",
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write_json(path, value):
    Path(path).write_text(json.dumps(value, sort_keys=True, indent=2) + "\n")


def expected_rows():
    return {CRATE + "::" + name: status for names, status in (
        (EXECUTIONS, "checked_exec_body"), (PROOFS, "checked_proof_body"),
        (SPECIFICATIONS, "specification_definition_or_declaration")) for name in names}


def model_inventory(source):
    matches = list(re.finditer(r"^pub (?:(open spec|proof) )?fn ([a-z_]+)\(", source, re.MULTILINE))
    inventory = {}
    for position, match in enumerate(matches):
        end = matches[position + 1].start() if position + 1 < len(matches) else source.rfind("\n}")
        declaration = source[match.start():end].strip()
        depth = 0
        body_start = None
        for offset in range(len(declaration) - 1, -1, -1):
            if declaration[offset] == "}":
                depth += 1
            elif declaration[offset] == "{":
                depth -= 1
                if depth == 0:
                    body_start = offset
                    break
        require(body_start is not None, "missing balanced function body")
        name = CRATE + "::" + match[2]
        require(name not in inventory, "duplicate model identity")
        inventory[name] = dict(kind=match[1] or "exec",
            item_sha256=hashlib.sha256(declaration.encode()).hexdigest(),
            body_sha256=hashlib.sha256(declaration[body_start:].encode()).hexdigest())
    require(set(inventory) == set(expected_rows()), "model declaration inventory changed")
    return inventory


def validate_positive(data, returncode, source_path):
    require(type(returncode) is int and returncode == 0, "positive process failed")
    result, inference = data["verification-results"], data["verus-infer"]
    for flag, expected in (("success", True), ("encountered-error", False),
                           ("encountered-vir-error", False), ("is-verifying-entire-crate", True)):
        require(result.get(flag) is expected, "invalid positive flag: " + flag)
    for field, expected in (("verified", 18), ("errors", 0)):
        require(type(result.get(field)) is int and result[field] == expected, "invalid count: " + field)
    require(inference.get("status") == "no_inference", "not an ordinary model-only run")
    require(inference.get("crate_success") is True and inference.get("interface_written") is False,
            "ordinary export reporting changed; not inference-aware publication")
    for field in ("requested_functions", "generated_functions"):
        require(type(inference.get(field)) is int and inference[field] == 0, "unexpected inference count")
    for field in ("functions", "failed_functions", "imported_inference_dependencies", "trusted_models"):
        require(inference.get(field) == [], "unexpected inference/import/trusted-model rows: " + field)
    scope = inference.get("verification_scope", {})
    for flag in ("native_collection_completed", "complete_selected_scope_verified"):
        require(scope.get(flag) is True, "incomplete whole selected model scope")
    for field, expected in (("checked_exec_bodies", 1), ("checked_proof_bodies", 16)):
        require(type(scope.get(field)) is int and scope[field] == expected, "wrong checked body count")
    rows = scope.get("native_functions")
    require(type(rows) is list and all(type(row) is dict for row in rows), "missing model scope rows")
    require(len(rows) == len(expected_rows()), "missing, duplicate, or extra model bodies")
    require({row.get("function"): row.get("body_status") for row in rows} == expected_rows(),
            "exact checked model identities/statuses differ")
    for row in rows:
        checked = row["body_status"] in ("checked_exec_body", "checked_proof_body")
        require(row.get("selected") is True and row.get("automatically_derived") is False
                and row.get("inference_candidate") is False and row.get("ordinary_contract") is checked,
                "malformed selection/inference/contract flags")
        name = row["function"].split("::")[-1]
        require(row.get("rust_function") == name, "wrong Rust function identity")
        require(type(row.get("source")) is str and row["source"].startswith(str(source_path) + ":"),
                "body source does not bind checked input")
    work = inference.get("metrics", {}).get("total_micros")
    require(type(work) is int and work > 0, "missing positive verifier work")
    return dict(verified=18, checked_exec_bodies=1, checked_proof_bodies=16,
                specification_definitions=15, whole_selected_scope=True, inference=False)


def validate_negative(data, returncode, stderr, safety=False):
    result = data["verification-results"]
    require(type(returncode) is int and returncode != 0, "negative process unexpectedly succeeded")
    require(result.get("success") is False and result.get("encountered-vir-error") is False
            and result.get("is-verifying-entire-crate") is True, "negative is not whole-scope semantic failure")
    require(type(result.get("errors")) is int and result["errors"] > 0
            and type(result.get("verified")) is int and result["verified"] > 0, "negative lacks checked work/errors")
    require("postcondition not satisfied" in stderr or (safety and "overflow" in stderr),
            "negative has wrong diagnostic")
    require(not any(token in stderr.lower() for token in
        ("panicked at", "timed out", "resource limit", "out of memory")), "resource/panic failure is not evidence")
    require(data["verus-infer"].get("crate_success") is False
            and data["verus-infer"].get("status") == "caller_proof_failed"
            and data["verus-infer"].get("functions") == []
            and type(data["verus-infer"].get("requested_functions")) is int
            and data["verus-infer"]["requested_functions"] == 0,
            "negative ordinary-proof classification changed")
    return dict(errors=result["errors"], verified=result["verified"],
                failure="safety" if safety else "semantic")


def sanitized_environment(output, runtime):
    environment = {key: value for key, value in os.environ.items()
        if not key.startswith("GIT_") and key not in
        ("RUSTFLAGS", "RUSTDOCFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER")}
    environment.update(GIT_OPTIONAL_LOCKS="0", PYTHONDONTWRITEBYTECODE="1", RUSTUP_TOOLCHAIN="1.99.1",
        VERUS_IN_VARGO="1", VERUS_Z3_PATH=str(runtime / "z3"),
        VERUS_INFER_CACHE=str(output / "unused-model-cache"), CARGO_HOME=str(output / "unused-cargo-home"),
        CARGO_TARGET_DIR=str(output / "unused-cargo-target"), TMPDIR=str(output))
    return environment


def audit_inputs(parent, environment):
    handoff = parent / "handoff"
    failures, counts, ledger_hashes = [], {}, {}
    ledgers = (("file-hashes.json", ROOT), ("source-hashes.json", ROOT),
        ("rust-build-input-hashes.json", ROOT), ("runtime-hashes.json", parent / "tools77"),
        ("toolchain-hashes.json", Path("/")), ("guarded-inputs.json", Path("/")))
    for filename, base in ledgers:
        path = handoff / filename
        entries = json.loads(path.read_text())
        ledger_hashes[filename] = digest(path)
        counts[filename] = len(entries)
        failures += [str(base / name) for name, expected in entries.items()
                     if not (base / name).is_file() or digest(base / name) != expected]
    natives = json.loads((handoff / "native-hashes.json").read_text())
    ledger_hashes["native-hashes.json"] = digest(handoff / "native-hashes.json")
    for name, native in natives.items():
        checkout = Path(native["root"])
        failures += [str(checkout / relative) for relative, expected in native["sources"].items()
                     if not (checkout / relative).is_file() or digest(checkout / relative) != expected]
        head = subprocess.check_output(["git", "-c", "core.fsmonitor=false", "rev-parse", "HEAD"],
            cwd=checkout, env=environment, text=True).strip()
        status = subprocess.check_output(["git", "-c", "core.fsmonitor=false", "status", "--porcelain",
            "--untracked-files=all"], cwd=checkout, env=environment, text=True)
        require(head == native["pin"] and not status, "native pin/cleanliness changed: " + name)
        counts[name + "_native_sources"] = len(native["sources"])
    require(natives["fedimint"]["pin"] == PIN, "unexpected Fedimint pin")
    require({name: digest(Path(natives["fedimint"]["root"]) / name) for name in SOURCE_HASHES}
            == SOURCE_HASHES, "inspected source changed")
    require(digest(handoff / "combined.apply_patch") == PATCH_SHA256, "baseline patch changed")
    head = subprocess.check_output(["git", "-c", "core.fsmonitor=false", "rev-parse", "HEAD"],
        cwd=ROOT, env=environment, text=True).strip()
    require(head == CHECKPOINT and not failures, "baseline/guarded input changed: " + repr(failures))
    require(counts == {"file-hashes.json": 1976, "source-hashes.json": 1566,
        "rust-build-input-hashes.json": 1085, "runtime-hashes.json": 9,
        "toolchain-hashes.json": 61, "guarded-inputs.json": 3395,
        "fedimint_native_sources": 1199, "redb_native_sources": 132}, "ledger scope changed")
    require(ENGINE_IDENTITY == (77, 71, 38) and INTERFACE_SCHEMA == 38
            and CACHE_VERSION == "verus-infer-v77:record-div-import:4096"
            and INTERFACE_POLICY == "candidate-bound-evidence-v71", "identity changed")
    inspected = json.loads((EVIDENCE / "inspected-dependencies.json").read_text())
    require(all(Path(name).is_file() and digest(name) == expected for name, expected in inspected.items()),
            "read-only inspected dependency source changed")
    return dict(counts=counts, ledger_sha256=ledger_hashes, native_pin=PIN,
                source_sha256=SOURCE_HASHES, identity=list(ENGINE_IDENTITY), checkpoint=head,
                frozen_patch_sha256=PATCH_SHA256, inspected_dependencies_sha256=inspected)


def appended_source(source, goal):
    prefix, suffix = source.rsplit("\n}", 1)
    return prefix + "\n" + goal + "\n}" + suffix


def replay(parent, output):
    require(not output.exists(), "output must be fresh; no artifact cleanup is promised")
    require(not output.is_relative_to(parent), "sealed parent is read-only")
    output.mkdir(parents=True, exist_ok=False)
    runtime = parent / "tools77"
    environment = sanitized_environment(output, runtime)
    before = audit_inputs(parent, environment)
    source = (EVIDENCE / "model.rs").read_text()
    inventory = model_inventory(source)
    frozen_scope = json.loads((EVIDENCE / "scope.json").read_text())
    require(frozen_scope == dict(model_sha256=digest(EVIDENCE / "model.rs"), functions=inventory),
            "frozen model/body inventory differs")
    receipt = dict(success=False, classification="CHECKED CONDITIONAL MODEL-ONLY", source_refinement=False,
        before=before, source_inventory=frozen_scope, runner_sha256=digest(Path(__file__)),
        inference=False, imports=[], cases=[], export_qualification=
        "Legacy ordinary exports only; no inference-aware publication, cache, import, or stale cleanup claim")
    write_json(output / "receipt.json", receipt)

    def run(label, text, expected, debug=True, seed=None, legacy=False):
        case = output / label
        case.mkdir(exist_ok=False)
        path = case / "model.rs"
        path.write_text(text)
        artifact, library = case / "model.vir", case / "model.rlib"
        if seed is not None:
            artifact.write_bytes(seed.read_bytes())
        seed_hash = digest(artifact) if artifact.exists() else None
        command = [str(runtime / "verus"), str(path), "--crate-type=lib", "--crate-name", CRATE,
            "--output-json", "--no-cheating", "--num-threads", "1", "--compile",
            "--export", str(artifact), "-o", str(library), "-C", "debug-assertions=" + ("yes" if debug else "no")]
        started = time.monotonic()
        completed = subprocess.run(command, cwd=case, env=environment, capture_output=True, text=True, timeout=180)
        (case / "stdout.json").write_text(completed.stdout)
        (case / "stderr.txt").write_text(completed.stderr)
        data = json.loads(completed.stdout)
        require(digest(path) == hashlib.sha256(text.encode()).hexdigest(), "checked source changed during run")
        entry = dict(label=label, argv=command, cwd=str(case), returncode=completed.returncode,
            seconds=time.monotonic() - started, source_sha256=digest(path), expected=expected,
            environment={name: environment[name] for name in ("GIT_OPTIONAL_LOCKS", "RUSTUP_TOOLCHAIN",
                "VERUS_IN_VARGO", "VERUS_Z3_PATH", "VERUS_INFER_CACHE", "CARGO_HOME", "CARGO_TARGET_DIR", "TMPDIR")},
            imports=[], default_erasure=True, no_cheating=True, seeded_export_sha256=seed_hash)
        write_json(case / "command.json", entry)
        if expected == "positive":
            require(text == source, "positive executable/specification source adapted")
            entry["acceptance"] = validate_positive(data, completed.returncode, path)
            require(artifact.is_file() and library.is_file(), "positive default-erasure compile/export missing")
        else:
            entry["acceptance"] = validate_negative(data, completed.returncode, completed.stderr,
                safety=expected == "safety")
        entry["legacy_export"] = dict(interface_written=data["verus-infer"]["interface_written"],
            exists=artifact.exists(), sha256=digest(artifact) if artifact.exists() else None)
        if legacy:
            require(seed_hash is not None and artifact.exists(), "legacy seeded observation changed")
            entry["legacy_export"]["seed_replaced_before_failed_proof"] = digest(artifact) != seed_hash
        entry["artifacts"] = {path.name: digest(path) for path in sorted(case.iterdir()) if path.is_file()}
        receipt["cases"].append(entry)
        write_json(output / "receipt.json", receipt)
        return artifact

    positive = run("positive_debug", source, "positive")
    run("positive_release", source, "positive", debug=False)
    for label, goal in NEGATIVE_GOALS.items():
        run(label, appended_source(source, goal), "safety" if label == "safety_overflow" else "semantic")
    run("legacy_seeded_failure", appended_source(source, NEGATIVE_GOALS["marker_only"]), "semantic",
        seed=positive, legacy=True)
    receipt["after"] = audit_inputs(parent, environment)
    require(receipt["before"] == receipt["after"], "input ledger changed during replay")
    require(not (output / "unused-cargo-home").exists() and not (output / "unused-cargo-target").exists(),
            "unexpected Cargo writes")
    require(len(receipt["cases"]) == 11, "incomplete fresh replay")
    receipt["success"] = True
    write_json(output / "receipt.json", receipt)
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--parent", type=Path, default=Path("/private/tmp/verus-cycle77-review-fixes.XCkOUU"))
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    receipt = replay(arguments.parent.resolve(), arguments.output.resolve())
    print(json.dumps(dict(success=receipt["success"], cases=len(receipt["cases"]),
        classification=receipt["classification"], receipt=str(arguments.output.resolve() / "receipt.json"))))


if __name__ == "__main__":
    main()
