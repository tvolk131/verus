"""Replay experimental resource artifact/driver barriers, not native proofs."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import time


SOURCE = Path(__file__).resolve().parents[1]
PROFILES = ("default-off", "feature-on")
TESTS = (
    "ordinary_exports_roundtrip_through_explicit_and_cargo_imports",
    "serialized_markers_are_rejected_before_import_filters_and_metadata",
    "marked_exports_are_rejected_before_private_pruning_and_verified_metadata",
    "stale_export_invalidation_covers_direct_and_cargo_paths",
    "actual_driver_rejects_marked_imports_without_panicking_or_leaving_stale_exports",
)


def command_plan(profiles):
    commands = []
    for profile in profiles:
        feature = ["--features", "vir/experimental-native-resource"] if profile == "feature-on" else []
        common = ["cargo", "--offline", "--locked"]
        commands.append((f"{profile}-build", [common[0], "build", *common[1:], "-p", "rust_verify",
                                             "-p", "verus_builtin", *feature]))
        commands.append((f"{profile}-tests", [common[0], "test", *common[1:], "-p", "rust_verify",
                                             "--lib", *feature, "native_resource_boundary_tests",
                                             "--", "--include-ignored", "--nocapture", "--test-threads=1"]))
    return commands


def source_identity():
    paths = [SOURCE / "Cargo.toml", SOURCE / "Cargo.lock", SOURCE / ".cargo/config.toml",
             SOURCE.parent / "rust-toolchain.toml", Path(__file__).resolve()]
    for directory in ("rust_verify", "vir", "air", "builtin"):
        paths.extend((SOURCE / directory).rglob("*.rs"))
        paths.append(SOURCE / directory / "Cargo.toml")
    return {str(path.relative_to(SOURCE.parent)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(set(paths))}


def checked_test_log(text):
    expected = {f"import_export::native_resource_boundary_tests::{name}" for name in TESTS}
    completed = set()
    pending = None
    summary_seen = False
    for line in text.splitlines():
        if line.startswith("test result:"):
            summary = re.fullmatch(r"test result: ok\. ([0-9]+) passed; 0 failed; 0 ignored;.*", line)
            if summary_seen or pending is not None or completed != expected \
                    or summary is None or int(summary.group(1)) != len(expected):
                return False
            summary_seen = True
            continue
        started = re.fullmatch(r"test (\S+) \.\.\.(?: (.*))?", line)
        if started is not None:
            name, output = started.groups()
            if summary_seen or pending is not None or name not in expected or name in completed:
                return False
            if output == "ok":
                completed.add(name)
            elif output == "FAILED" or output is not None and output.startswith("ignored"):
                return False
            else:
                pending = name
        elif pending is not None and (line in ("ok", "FAILED") or line.startswith("ignored")):
            if line != "ok":
                return False
            completed.add(pending)
            pending = None
    return summary_seen and pending is None and completed == expected


def execute(label, argv, output, environment, timeout):
    log = output / f"{label}.log"
    started = time.monotonic()
    timed_out = False
    with log.open("xb") as stream:
        process = subprocess.Popen(argv, cwd=SOURCE, env=environment, stdout=stream,
                                   stderr=subprocess.STDOUT, start_new_session=os.name == "posix")
        try:
            exit_code = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            if os.name == "posix":
                os.killpg(process.pid, signal.SIGKILL)
            else:
                process.kill()
            exit_code = process.wait()
    raw = log.read_bytes()
    matched_tests = checked_test_log(raw.decode("utf-8", errors="replace")) if label.endswith("-tests") else None
    return {"label": label, "argv": argv, "cwd": str(SOURCE), "exit_code": exit_code,
            "timed_out": timed_out, "seconds": time.monotonic() - started,
            "log": log.name, "log_sha256": hashlib.sha256(raw).hexdigest(),
            "expected_tests_passed": matched_tests,
            "success": exit_code == 0 and not timed_out and matched_tests is not False}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path, help="new evidence directory; must not exist")
    parser.add_argument("--profile", choices=PROFILES, action="append")
    parser.add_argument("--timeout", type=int, default=1800, help="seconds per Cargo command")
    args = parser.parse_args(argv)
    if args.timeout <= 0:
        parser.error("--timeout must be positive")
    profiles = list(dict.fromkeys(args.profile or PROFILES))
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    environment = os.environ.copy()
    environment["CARGO_TARGET_DIR"] = str(output / "target")
    environment["CARGO_BUILD_BUILD_DIR"] = str(output / "target")
    environment["CARGO_BUILD_JOBS"] = "2"
    environment["VERUS_NATIVE_RESOURCE_TEST_DRIVER"] = str(output / "target/debug/rust_verify")
    environment["VERUS_NATIVE_RESOURCE_TEST_BUILTIN"] = str(output / "target/debug/libverus_builtin.rlib")
    before = source_identity()
    receipt = {
        "schema": "native-resource-boundary-replay-v1", "profiles": profiles,
        "classification": "engineering boundary regressions, not native ownership or application proofs",
        "identity_scope": "selected source/configuration inputs, not complete compiler/runtime provenance",
        "assumptions_adopted": False, "certification_policy_changed": False,
        "environment": {name: environment.get(name) for name in
                        ("CARGO_TARGET_DIR", "CARGO_BUILD_BUILD_DIR", "CARGO_BUILD_JOBS", "RUSTFLAGS",
                         "CARGO_ENCODED_RUSTFLAGS", "RUSTC_BOOTSTRAP", "VERUS_NATIVE_RESOURCE_TEST_DRIVER",
                         "VERUS_NATIVE_RESOURCE_TEST_BUILTIN")},
        "inputs": before, "commands": [], "success": False,
    }
    try:
        for label, command in command_plan(profiles):
            if shutil.disk_usage(output).free < 16 * 1024 ** 3:
                raise RuntimeError("less than 16 GiB free; refusing another build/test command")
            print(f"{label}: {' '.join(command)}", flush=True)
            result = execute(label, command, output, environment, args.timeout)
            receipt["commands"].append(result)
            if not result["success"]:
                break
        after = source_identity()
        receipt["input_drift"] = sorted(name for name in before.keys() | after.keys()
                                       if before.get(name) != after.get(name))
        receipt["success"] = len(receipt["commands"]) == 2 * len(profiles) \
            and all(command["success"] for command in receipt["commands"]) and not receipt["input_drift"]
    except Exception as error:
        receipt["error"] = f"{type(error).__name__}: {error}"
    finally:
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({"success": receipt["success"], "receipt": str(output / "receipt.json")}))
    return 0 if receipt["success"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
