import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys


ROOT = Path(__file__).resolve().parent


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--engine-root", type=Path, default=ROOT.parents[1])
    parser.add_argument("--allow-rebuilt-artifacts", action="store_true")
    arguments = parser.parse_args()
    engine = arguments.engine_root.resolve()
    output = arguments.output.resolve()
    selection = json.loads((ROOT / "selection.json").read_text())
    difference = subprocess.check_output([
        "git", "diff", selection["engine_baseline"], "--", "source", "dependencies", "tools", "rust-toolchain.toml"
    ], cwd=engine, text=True)
    assert difference == "", "Engine source differs from the frozen baseline"
    frozen = json.loads((ROOT / "artifacts-before-proofs.json").read_text())
    observed = {name: digest(engine / "source/target-verus/release" / name)
                for name in frozen["artifacts"]}
    mismatches = [name for name, value in observed.items()
                  if value != frozen["artifacts"][name]["sha256"]]
    if mismatches and not arguments.allow_rebuilt_artifacts:
        raise SystemExit("Artifact hashes differ; independently rebuild the unchanged baseline and explicitly use --allow-rebuilt-artifacts to record a distinct replay: " + str(mismatches))
    output.mkdir(parents=True, exist_ok=False)
    for relative in ("fixtures", "upstream"):
        shutil.copytree(ROOT / relative, output / relative)
    for relative in ("run.py", "selection.json", "original-selected.rs", "native_tests.rs"):
        shutil.copyfile(ROOT / relative, output / relative)
    original_engine = Path(json.loads((ROOT / "raw/engine-build-offline.json").read_text())["cwd"]).parent
    original_experiment = original_engine / "evidence/verus-infer-transfer-v1"
    native = output / "native"
    native.mkdir()

    def relocate(value):
        value = value.replace("/private/tmp/", str(native) + "/")
        value = value.replace(str(original_experiment), str(output))
        value = value.replace(str(original_engine), str(engine))
        value = value.replace("evidence/verus-infer-transfer-v1/", str(output) + "/")
        return value

    labels = ["native-upstream-build-corrected", "native-tests-build-corrected", "native-tests",
              "native-upstream-release-build", "native-release-tests-build", "native-release-tests"]
    labels += [row["label"] for row in json.loads((ROOT / "constant-enclosed-trials.json").read_text())]
    labels += ["engine-sanity-cold", "engine-sanity-warm"]
    results = []
    for label in labels:
        template = json.loads((ROOT / "raw" / (label + ".json")).read_text())
        environment = dict(os.environ)
        for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_TARGET_DIR"):
            environment.pop(key, None)
        environment.update(RUSTUP_TOOLCHAIN="1.99.1", RUST_MIN_STACK="20971520",
                           VERUS_Z3_PATH=str(engine / "source/z3"))
        if "VERUS_INFER_CACHE" in template["environment"]:
            environment["VERUS_INFER_CACHE"] = relocate(template["environment"]["VERUS_INFER_CACHE"])
            Path(environment["VERUS_INFER_CACHE"]).parent.mkdir(parents=True, exist_ok=True)
        command = [relocate(value) for value in template["argv"]]
        process = subprocess.run([sys.executable, str(output / "run.py"), "--cwd", str(engine),
                                  "--timeout", "180", label, "--", *command], env=environment,
                                 stdout=subprocess.DEVNULL, check=False)
        validated = process.returncode == template["exit_status"]
        if "constant-enclosed-" in label:
            try:
                report = json.loads((output / "raw" / (label + ".stdout")).read_text())
                diagnostic = (output / "raw" / (label + ".stderr")).read_text()
                validated = validated and report["verification-results"]["encountered-vir-error"]
                validated = validated and report["verus-infer"]["verification_scope"]["checked_exec_bodies"] == 0
                validated = validated and "`for` loops produced by a macro expansion" in diagnostic
            except (KeyError, json.JSONDecodeError):
                validated = False
        if label.startswith("engine-sanity-"):
            try:
                report = json.loads((output / "raw" / (label + ".stdout")).read_text())
                validated = validated and report["verification-results"]["success"]
                functions = report["verus-infer"]["functions"]
                validated = validated and len(functions) == 3 and all(
                    function["required_checks_passed"] and all(
                        proof["required"] and proof["status"] == "proved"
                        for proof in function["proofs"].values()) for function in functions)
            except (KeyError, json.JSONDecodeError):
                validated = False
        results.append({"label": label, "expected_exit_status": template["exit_status"],
                        "observed_exit_status": process.returncode, "validated": bool(validated)})
        print(label, process.returncode)
    receipt = {"engine_root": str(engine), "source_baseline": selection["engine_baseline"],
               "artifact_sha256": observed, "artifact_differences_from_frozen_build": mismatches,
               "z3_sha256": digest(engine / "source/z3"), "commands": results,
               "limit": "This replays captured commands, not the original clean build. Rebuilt artifacts are distinct evidence; source equivalence alone does not authenticate their build provenance."}
    (output / "replay.json").write_text(json.dumps(receipt, indent=2) + "\n")
    if not all(row["validated"] for row in results):
        raise SystemExit("Replay outcomes differ; inspect the new raw logs")


if __name__ == "__main__":
    main()
