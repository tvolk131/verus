"""Read-only, standard-library audit of the frozen transfer capture (not a proof)."""

import argparse
import datetime
import hashlib
import json
import math
from pathlib import Path, PurePosixPath
import sys
import tarfile


DEFAULT_ROOT = Path(__file__).resolve().parents[2] / "evidence/verus-infer-transfer-v1"
INDEX_SHA256 = "f18a65b05415b158dfe60e984baa958231d906e4ef802821d73ed875cab08c28"
ENGINE_BASELINE = "45f93e51f4d0c0f1b71ef645932d54a7862ec983"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_json(path):
    return json.loads(path.read_bytes())


def check_index(root):
    require(digest(root / "evidence-index.json") == INDEX_SHA256, "frozen index changed")
    index = read_json(root / "evidence-index.json")
    expected = {"evidence-index.json"}
    total_bytes = 0
    for entry in index["entries"]:
        relative = PurePosixPath(entry["path"])
        require(not relative.is_absolute() and ".." not in relative.parts,
                "unsafe index path")
        require(str(relative) not in expected, "duplicate index path")
        expected.add(str(relative))
        path = root / relative
        require(not path.is_symlink(), f"symlink: {relative}")
        require(path.stat().st_size == entry["bytes"], f"size mismatch: {relative}")
        require(digest(path) == entry["sha256"], f"hash mismatch: {relative}")
        total_bytes += entry["bytes"]
    paths = list(root.rglob("*"))
    require(not root.is_symlink() and not any(path.is_symlink() for path in paths),
            "symlinks are not frozen evidence")
    actual = {path.relative_to(root).as_posix() for path in paths if path.is_file()}
    require(actual == expected, "missing or unexpected capture files")
    require(index["schema"] == 1 and index["self_excluded"] is True, "invalid index schema")
    require(len(expected) - 1 == index["file_count"] == 226, "invalid index count")
    require(total_bytes == index["total_bytes"] == 603898, "invalid index byte total")
    return index


def check_receipts(root, expected_count=62, selection_hash=None):
    if selection_hash is None:
        selection_hash = digest(root / "selection.json")
    receipts = sorted(path for path in root.rglob("*.json") if path.parent.name == "raw")
    require(len(receipts) == expected_count, f"expected {expected_count} command receipts")
    for path in receipts:
        receipt = read_json(path)
        require(receipt["label"] == path.stem, f"receipt label mismatch: {path}")
        require(receipt["selection_sha256"] == selection_hash, f"selection mismatch: {path}")
        require(type(receipt["exit_status"]) is int, f"incomplete command: {path}")
        require(not receipt.get("timed_out") and not receipt.get("launch_error"),
                f"command did not finish: {path}")
        require(math.isfinite(receipt["elapsed_seconds"]) and receipt["elapsed_seconds"] >= 0,
                f"invalid elapsed time: {path}")
        require(receipt["argv"] and receipt["cwd"] and receipt["environment"],
                f"missing command provenance: {path}")
        started = datetime.datetime.fromisoformat(receipt["started_utc"])
        require(started.tzinfo is not None, f"missing receipt timezone: {path}")
        for stream in ("stdout", "stderr"):
            require(digest(path.with_suffix("." + stream)) == receipt[stream + "_sha256"],
                    f"{stream} receipt mismatch: {path}")
    return len(receipts)


def check_source(root):
    selection = read_json(root / "selection.json")
    upstream = selection["upstream"]
    require(selection["engine_baseline"] == ENGINE_BASELINE, "engine baseline changed")
    archive = root / "upstream/unsigned-varint-0.8.0.crate"
    require(digest(archive) == upstream["archive_sha256"], "upstream archive hash mismatch")
    with tarfile.open(archive, "r:gz") as package:
        for relative, expected in upstream["snapshot_sha256"].items():
            snapshot = root / "upstream" / relative
            require(digest(snapshot) == expected, f"upstream snapshot mismatch: {relative}")
            if relative.endswith(".crate"):
                continue
            with package.extractfile("unsigned-varint-0.8.0/" + relative) as member:
                require(member.read() == snapshot.read_bytes(), f"archive member mismatch: {relative}")
    original = (root / "original-selected.rs").read_text()
    require(digest(root / "original-selected.rs") == upstream["original_selected_sha256"],
            "selected original changed")
    macro, remainder = original.split("\n\npub fn u64(", 1)
    function, constant = ("pub fn u64(" + remainder).split("\n\nconst U64_LEN", 1)
    source = (root / "upstream/src/encode.rs").read_text()
    require(macro in source and function in source and "const U64_LEN" + constant in source,
            "selected items differ from upstream")
    for path in (root / "fixtures").glob("*.rs"):
        fixture = path.read_text()
        require(macro in fixture and function in fixture and "const U64_LEN" + constant.rstrip() in fixture,
                f"original executable items changed: {path}")
        require("requires" not in fixture, f"narrowed input domain: {path}")
    frozen = read_json(root / "artifacts-before-proofs.json")
    require(frozen["selection_sha256"] == digest(root / "selection.json"), "selection pin mismatch")
    audit = read_json(root / "audit.json")
    hashes = {name: artifact["sha256"] for name, artifact in frozen["artifacts"].items()}
    require(audit["artifacts_unchanged_sha256"] == hashes, "historical artifact receipt mismatch")
    first_trial = read_json(root / "raw/ordinary-plain-cold.json")["started_utc"]
    require(selection["frozen_at_utc"] < frozen["recorded_utc"] < first_trial,
            "selection/artifact freeze does not predate trials")
    return hashes


def check_outcomes(root):
    for filename, prefix in (("trials.json", ""), ("constant-enclosed-trials.json", "constant-enclosed-")):
        trials = read_json(root / filename)
        labels = {prefix + fixture + "-" + assistance + "-" + temperature
                  for fixture in ("ordinary", "inferred")
                  for assistance in ("plain", "assisted") for temperature in ("cold", "warm")}
        require(len(trials) == 8 and {trial["label"] for trial in trials} == labels,
                f"incomplete trial matrix: {filename}")
        for trial in trials:
            label = trial["label"]
            receipt = read_json(root / "raw" / (label + ".json"))
            report = read_json(root / "raw" / (label + ".stdout"))
            inference = report["verus-infer"]
            result = report["verification-results"]
            require(receipt["exit_status"] == trial["exit_status"] == 1,
                    f"application exit mismatch: {label}")
            require(trial["verification_results"] == result and trial["inference"] == inference,
                    f"trial summary differs from raw report: {label}")
            require(result["success"] is False and result["encountered-vir-error"] is True,
                    f"not a frontend rejection: {label}")
            require(inference["status"] == "verification_failed" and inference["crate_success"] is False,
                    f"unexpected application success: {label}")
            require(inference["functions"] == [] and inference["generated_functions"] == 0
                    and inference["interface_written"] is False, f"candidate/export exists: {label}")
            scope = inference["verification_scope"]
            require(scope["checked_exec_bodies"] == scope["checked_proof_bodies"] == 0
                    and report["times-ms"]["total-verify"] == report["times-ms"]["air"]["total"] == 0,
                    f"unexpected application proof work: {label}")
            assistance = inference["assistance_explanations"]
            require(assistance["enabled"] == trial["assisted"]
                    and assistance["preparation_started"] is False and assistance["decisions"] == [],
                    f"unexpected assistance work: {label}")
            require(trial["cache_before"] == trial["cache_after"] == {}
                    and trial["export_exists_after"] is False, f"cache/export mismatch: {label}")
            require(all(flag in receipt["argv"] for flag in ("--no-cheating", "--output-json", "--time", "--export"))
                    and ("--infer-goals" in receipt["argv"]) == trial["assisted"],
                    f"trial command flags mismatch: {label}")
            diagnostic = (root / "raw" / (label + ".stderr")).read_text()
            require("`for` loops produced by a macro expansion" in diagnostic
                    and ("U64_LEN` which is ignored" in diagnostic) == (prefix == ""),
                    f"wrong frontend blocker: {label}")
    for capture in (root, root / "replay-corrected"):
        for label in ("native-tests", "native-release-tests"):
            require(read_json(capture / "raw" / (label + ".json"))["exit_status"] == 0
                    and "3 passed; 0 failed" in (capture / "raw" / (label + ".stdout")).read_text(),
                    f"native tests did not pass: {capture}/{label}")
        contracts = []
        for temperature, cache in (("cold", "miss"), ("warm", "hit")):
            label = "engine-sanity-" + temperature
            report = read_json(capture / "raw" / (label + ".stdout"))
            require(read_json(capture / "raw" / (label + ".json"))["exit_status"] == 0
                    and report["verification-results"]["success"] is True
                    and report["verification-results"]["verified"] == 9, "engine sanity failed")
            functions = report["verus-infer"]["functions"]
            require(len(functions) == 3, "incomplete sanity functions")
            for function in functions:
                require(function["required_checks_passed"] is True
                        and set(function["proofs"]) == {"soundness", "maximality", "exactness"}
                        and all(proof["required"] is True and proof["status"] == "proved"
                                for proof in function["proofs"].values())
                        and function["metrics"]["cache"] == cache and function["metrics"]["solver_micros"] > 0,
                        "sanity lacks fresh required proofs")
            contracts.append({function["function"]: function["contract"] for function in functions})
        require(contracts[0] == contracts[1], "cold/warm sanity contracts differ")
    replay = read_json(root / "replay-corrected/replay.json")
    require(len(replay["commands"]) == 16 and all(row["validated"] is True
            and row["expected_exit_status"] == row["observed_exit_status"] for row in replay["commands"]),
            "historical corrected replay failed")
    require(replay["source_baseline"] == ENGINE_BASELINE
            and replay["artifact_differences_from_frozen_build"] == [], "historical replay provenance differs")
    require(replay["artifact_sha256"] == check_source(root), "historical replay artifact hashes differ")


def check_capture(root):
    index = check_index(root)
    receipts = check_receipts(root)
    check_outcomes(root)
    return {"indexed_files": index["file_count"], "indexed_bytes": index["total_bytes"],
            "command_receipts": receipts, "application_frontend_rejections": 16,
            "limit": "Offline hash/log consistency only; no application proof or current binary authentication."}


def check_publication_replay(root, capture):
    count = check_receipts(root, 16, digest(capture / "selection.json"))
    replay = read_json(root / "replay.json")
    historical = read_json(capture / "replay-corrected/replay.json")
    require(replay["commands"] == historical["commands"], "publication replay outcomes differ")
    for field in ("source_baseline", "artifact_sha256", "z3_sha256", "artifact_differences_from_frozen_build"):
        require(replay[field] == historical[field], f"publication replay {field} differs")
    for row in replay["commands"]:
        receipt = read_json(root / "raw" / (row["label"] + ".json"))
        require(receipt["exit_status"] == row["observed_exit_status"], "publication replay status mismatch")
    return count


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    parser.add_argument("--replay-root", type=Path, help="also check the separate publication replay receipts")
    arguments = parser.parse_args()
    try:
        result = check_capture(arguments.root)
        if arguments.replay_root is not None:
            result["publication_replay_receipts"] = check_publication_replay(arguments.replay_root, arguments.root)
    except (OSError, ValueError, KeyError, TypeError, tarfile.TarError) as error:
        print(f"transfer evidence check failed: {error}", file=sys.stderr)
        return 1
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
