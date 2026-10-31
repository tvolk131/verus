import datetime
import hashlib
import json
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parent
REPOSITORY = ROOT.parents[1]


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read_json(path):
    return json.loads(path.read_text())


def main():
    selection = read_json(ROOT / "selection.json")
    frozen = read_json(ROOT / "artifacts-before-proofs.json")
    assert digest(ROOT / "selection.json") == frozen["selection_sha256"]
    source_diff = subprocess.check_output([
        "git", "diff", selection["engine_baseline"], "--", "source", "dependencies", "tools", "rust-toolchain.toml"
    ], cwd=REPOSITORY, text=True)
    assert source_diff == ""
    for relative, expected in selection["upstream"]["snapshot_sha256"].items():
        assert digest(ROOT / "upstream" / relative) == expected
    original = (ROOT / "original-selected.rs").read_text()
    assert digest(ROOT / "original-selected.rs") == selection["upstream"]["original_selected_sha256"]
    macro, remainder = original.split("\n\npub fn u64(", 1)
    function, constant = ("pub fn u64(" + remainder).split("\n\nconst U64_LEN", 1)
    body_preservation = {}
    initial = read_json(ROOT / "pre-proof-fixtures.json")
    for relative, expected in initial.items():
        assert digest(ROOT / relative) == expected
    adapters = read_json(ROOT / "constant-enclosure-adapter.json")
    for change in adapters["changes"]:
        assert digest(ROOT / change["before"]) == change["before_sha256"]
        assert digest(ROOT / change["after"]) == change["after_sha256"]
    for path in sorted((ROOT / "fixtures").glob("*.rs")):
        source = path.read_text()
        assert macro in source and function in source
        assert "const U64_LEN" + constant.rstrip() in source
        assert "requires" not in source
        assert source.count("#[trigger]") == 1 and source.count("decreases exponent") == 1
        assert all(forbidden not in source for forbidden in (
            "external_body", "assume_specification", "assume(", "admit(",
            "assume_termination", "exec_allows_no_decreases_clause", "invariant"))
        body_preservation[str(path.relative_to(ROOT))] = {
            "original_macro_exact_substring": True, "original_u64_function_exact_substring": True,
            "original_constant_exact_substring": True, "no_requires": True, "sha256": digest(path)}
    current_artifacts = {}
    for name, expected in frozen["artifacts"].items():
        path = Path(expected["path"])
        assert digest(path) == expected["sha256"]
        current_artifacts[name] = digest(path)
    for name in ("rustc", "rustc_driver", "cargo", "z3"):
        assert digest(frozen[name]["path"]) == frozen[name]["sha256"]
    receipts = list(sorted((ROOT / "raw").glob("*.json")))
    for path in receipts:
        receipt = read_json(path)
        assert receipt["selection_sha256"] == digest(ROOT / "selection.json")
        assert digest(path.with_suffix(".stdout")) == receipt["stdout_sha256"]
        assert digest(path.with_suffix(".stderr")) == receipt["stderr_sha256"]
        assert receipt["exit_status"] is not None and receipt["elapsed_seconds"] >= 0
    trial_count = 0
    for name in ("trials.json", "constant-enclosed-trials.json"):
        for trial in read_json(ROOT / name):
            trial_count += 1
            assert trial["exit_status"] == 1
            assert trial["cache_before"] == {} and trial["cache_after"] == {}
            assert trial["export_exists_after"] is False
            report = read_json(ROOT / "raw" / (trial["label"] + ".stdout"))
            result = report["verification-results"]
            inference = report["verus-infer"]
            assert result["success"] is False and result["encountered-vir-error"] is True
            assert inference["status"] == "verification_failed" and inference["crate_success"] is False
            assert inference["functions"] == [] and inference["generated_functions"] == 0
            assert inference["interface_written"] is False
            assert inference["verification_scope"]["checked_exec_bodies"] == 0
            assert inference["verification_scope"]["checked_proof_bodies"] == 0
            assert inference["assistance_explanations"]["enabled"] == trial["assisted"]
            assert inference["assistance_explanations"]["preparation_started"] is False
            assert inference["assistance_explanations"]["decisions"] == []
            assert report["times-ms"]["total-verify"] == 0
            assert report["times-ms"]["air"]["total"] == 0
            diagnostic = (ROOT / "raw" / (trial["label"] + ".stderr")).read_text()
            assert "`for` loops produced by a macro expansion" in diagnostic
            assert ("U64_LEN` which is ignored" in diagnostic) == (name == "trials.json")
    assert trial_count == 16
    sanity = {}
    for temperature, cache in (("cold", "miss"), ("warm", "hit")):
        report = read_json(ROOT / "raw" / ("engine-sanity-" + temperature + ".stdout"))
        assert report["verification-results"]["success"] is True
        assert report["verification-results"]["verified"] == 9
        inference = report["verus-infer"]
        assert inference["interface_written"] is True and len(inference["functions"]) == 3
        sanity[temperature] = {}
        for function in inference["functions"]:
            assert function["metrics"]["cache"] == cache
            assert function["metrics"]["solver_micros"] > 0
            assert function["required_checks_passed"] is True
            assert all(proof["required"] and proof["status"] == "proved"
                       for proof in function["proofs"].values())
            sanity[temperature][function["function"]] = function["contract"]
    assert sanity["cold"] == sanity["warm"]
    for label in ("native-tests", "native-release-tests"):
        assert read_json(ROOT / "raw" / (label + ".json"))["exit_status"] == 0
        assert "3 passed; 0 failed" in (ROOT / "raw" / (label + ".stdout")).read_text()
    replay = read_json(ROOT / "replay-corrected/replay.json")
    assert len(replay["commands"]) == 16 and all(row["validated"] for row in replay["commands"])
    assert replay["artifact_differences_from_frozen_build"] == []
    started = read_json(ROOT / "raw/ordinary-plain-cold.json")["started_utc"]
    assert selection["frozen_at_utc"] < frozen["recorded_utc"] < started
    native_binaries = {path: digest(path) for path in (
        "/private/tmp/libunsigned_varint_transfer_v1.rlib",
        "/private/tmp/libunsigned_varint_transfer_v1_release.rlib",
        "/private/tmp/verus-infer-transfer-v1-native-tests",
        "/private/tmp/verus-infer-transfer-v1-native-release-tests")}
    audit = {
        "audited_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "engine_baseline": selection["engine_baseline"], "engine_source_diff": source_diff,
        "head_before_experiment_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPOSITORY, text=True).strip(),
        "selection_unchanged_sha256": digest(ROOT / "selection.json"),
        "selection_and_artifact_receipts_predate_first_application_trial": True,
        "body_preservation": body_preservation, "artifacts_unchanged_sha256": current_artifacts,
        "rustc_driver_cargo_z3_hashes_unchanged": True,
        "primary_command_receipts_validated": len(receipts),
        "application_frontend_rejections_validated": trial_count,
        "application_checked_exec_bodies": 0, "semantic_mutation_control": "not applicable: no positive application proof",
        "engine_sanity_cold_misses_warm_hits_all_fresh_required_proofs": True,
        "native_slice_cases_per_profile": 173503, "native_binaries_sha256": native_binaries,
        "corrected_replay_commands_validated": len(replay["commands"]),
        "trust_limit": "Hash/source/log consistency audit, not a proof certificate, source-to-binary formal proof or authenticated supply-chain audit.",
    }
    (ROOT / "audit.json").write_text(json.dumps(audit, indent=2) + "\n")
    entries = []
    for path in sorted(ROOT.rglob("*")):
        relative = path.relative_to(ROOT)
        if not path.is_file() or relative.parts[0] in ("work", "__pycache__") or str(relative) == "evidence-index.json":
            continue
        entries.append({"path": str(relative), "bytes": path.stat().st_size, "sha256": digest(path)})
    total_bytes = sum(row["bytes"] for row in entries)
    assert len(entries) <= 300 and total_bytes <= 5_000_000
    index = {"schema": 1, "file_count": len(entries), "total_bytes": total_bytes,
             "bound": {"files": 300, "bytes": 5_000_000},
             "self_excluded": True, "entries": entries}
    (ROOT / "evidence-index.json").write_text(json.dumps(index, indent=2) + "\n")
    print(json.dumps({"audit": "passed", "indexed_files": len(entries), "indexed_bytes": total_bytes,
                      "primary_commands": len(receipts), "application_trials": trial_count}, indent=2))


if __name__ == "__main__":
    main()
