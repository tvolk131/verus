import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parent
REPOSITORY = ROOT.parents[1]


def files(directory):
    return {str(path.relative_to(directory)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(directory.rglob("*")) if path.is_file()}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture-prefix", default="")
    arguments = parser.parse_args()
    prefix = arguments.fixture_prefix
    result_path = ROOT / (prefix + "trials.json")
    assert not result_path.exists()
    work = ROOT / "work"
    work.mkdir(exist_ok=True)
    results = []
    artifact_receipt = ROOT / "artifacts-before-proofs.json"
    assert artifact_receipt.exists()
    for fixture in ("ordinary", "inferred"):
        for assisted in (False, True):
            variant = prefix + fixture + ("-assisted" if assisted else "-plain")
            directory = work / variant
            directory.mkdir(exist_ok=False)
            cache = directory / "cache"
            export = directory / "result.vir"
            environment = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.1",
                               RUST_MIN_STACK="20971520",
                               VERUS_Z3_PATH=str(REPOSITORY / "source/z3"),
                               VERUS_INFER_CACHE=str(cache))
            for name in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"):
                environment.pop(name, None)
            command = [str(REPOSITORY / "source/target-verus/release/verus"),
                       str(ROOT / "fixtures" / (prefix + fixture + ".rs")),
                       "--crate-type=lib", "--crate-name=uv_transfer",
                       "--no-cheating", "--output-json", "--time",
                       "--export", str(export)]
            if assisted:
                command.append("--infer-goals")
            for temperature in ("cold", "warm"):
                label = variant + "-" + temperature
                before = files(cache)
                invocation = [sys.executable, str(ROOT / "run.py"), "--timeout", "180", label, "--", *command]
                subprocess.run(invocation, cwd=REPOSITORY, env=environment,
                               stdout=subprocess.DEVNULL, check=False)
                receipt = json.loads((ROOT / "raw" / (label + ".json")).read_text())
                try:
                    report = json.loads((ROOT / "raw" / (label + ".stdout")).read_text())
                except json.JSONDecodeError:
                    report = None
                results.append({"label": label, "fixture": fixture, "assisted": assisted,
                                "temperature": temperature, "cache_before": before,
                                "cache_after": files(cache), "export_exists_after": export.exists(),
                                "exit_status": receipt["exit_status"],
                                "elapsed_seconds": receipt["elapsed_seconds"],
                                "verification_results": report.get("verification-results") if report else None,
                                "inference": report.get("verus-infer") if report else None})
                print(label, "exit", receipt["exit_status"], "seconds", receipt["elapsed_seconds"])
    result_path.write_text(json.dumps(results, indent=2) + "\n")


if __name__ == "__main__":
    main()
