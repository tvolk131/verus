#!/usr/bin/env python3
"""Check real derives, explicit unused scope and failed-call/publication controls.

Build the matching release compiler, macros and vstd first. All commands are
offline and retain their exact stdout/stderr and structured receipts.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time


def reports(text):
    decoder = json.JSONDecoder()
    results, offset = [], 0
    while True:
        opening = text.find("{", offset)
        if opening < 0:
            return results
        try:
            value, length = decoder.raw_decode(text[opening:])
        except ValueError:
            offset = opening + 1
            continue
        offset = opening + length
        if isinstance(value, dict) and "verification-results" in value:
            results.append(value)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verus-source", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--native-toolchain", default="1.85.1")
    args = parser.parse_args()
    source = args.verus_source.resolve()
    fixture = Path(__file__).resolve().parent / "native_adoption"
    manifest = fixture / "Cargo.toml"
    binary = source / "target-verus/release"
    for name in ["cargo-verus", "verus", "rust_verify", "vstd.vir"]:
        if not (binary / name).is_file():
            parser.error(f"Build the matching release compiler/macros/vstd first: {binary / name}")
    args.output.mkdir(parents=True, exist_ok=True)
    out = Path(tempfile.mkdtemp(prefix="run-", dir=args.output.resolve()))
    channel = re.search(r'channel\s*=\s*"([^"]+)"', (source.parent / "rust-toolchain.toml").read_text()).group(1)
    patch = "patch.crates-io.vstd.path=" + json.dumps(str(source / "vstd"))
    env = dict(os.environ, RUSTUP_TOOLCHAIN=channel,
               CARGO_TARGET_DIR=str(out / "proof-target"), VERUS_Z3_PATH=str(source / "z3"),
               VERUS_INFER_CACHE=str(out / "candidate-cache"))
    env["PATH"] = str(binary) + os.pathsep + env.get("PATH", "")
    receipt = {
        "engine_sha256": {name: digest(binary / name) for name in ["cargo-verus", "verus", "rust_verify", "vstd.vir", "libvstd.rlib"]},
        "fixture_sha256": {str(p.relative_to(fixture)): digest(p) for p in [manifest, fixture / "Cargo.lock", fixture / "src/lib.rs"]},
        "verification_toolchain": channel, "native_toolchain": args.native_toolchain,
        "runner_sha256": digest(Path(__file__).resolve()), "commands": [], "cases": [],
    }

    def run(label, command, environment=env):
        started = time.monotonic()
        with (out / f"{label}.stdout").open("w") as stdout, (out / f"{label}.stderr").open("w") as stderr:
            # Avoid inheriting source/.cargo's compiler-development flags merely
            # because this independent example lives below the compiler tree.
            code = subprocess.run(command, cwd=source.parent, env=environment, stdout=stdout, stderr=stderr).returncode
        receipt["commands"].append({"label": label, "argv": command, "exit_status": code,
                                    "seconds": time.monotonic() - started})
        return code

    cases = [
        ("fresh", [], [], True), ("repeat", [], [], True),
        ("false_goal", ["wrong-goal"], [], False),
        ("excluded_sum", ["use-excluded-sum"], [], False),
        ("excluded_codec", ["use-excluded-codec"], [], False),
        ("excluded_request", ["exclude-requested-into"], [], False),
        ("no_verify", [], ["--no-verify"], False),
        ("filtered", [], ["--verify-root", "--verify-function", "unwrap"], False),
        ("restored", [], [], True),
    ]
    try:
        for label, features, flags, success in cases:
            clean = ["cargo", "--config", patch, "clean", "--offline", "--manifest-path", str(manifest), "-p", "verus-infer-native-adoption"]
            assert run(label + "_clean", clean) == 0
            command = [str(binary / "cargo-verus"), "verus", "verify", "--manifest-path", str(manifest),
                       "--config", patch, "--offline", "--locked", "-p", "verus-infer-native-adoption"]
            if features:
                command += ["--features", ",".join(features)]
            command += ["--fwd-verus-args-to", "roots", "--", "--infer-goals", "--no-cheating", "--output-json", *flags]
            code = run(label, command)
            parsed = reports((out / f"{label}.stdout").read_text())
            if label == "filtered" and not parsed:
                # Cargo rejects partial verification at its CLI boundary before
                # starting rust_verify. Direct compiler gates are tested too.
                assert code != 0 and "Partial verification must use `cargo verus focus`" in (out / f"{label}.stderr").read_text(), (label, out)
                receipt["cases"].append({"case": label, "expected_success": False,
                                         "stage": "cargo_cli", "verifier_started": False})
                continue
            assert len(parsed) == 1, (label, out)
            result = parsed[0]["verification-results"]
            inference = parsed[0]["verus-infer"]
            scope = inference["verification_scope"]
            assert (code == 0) == success and result.get("success", False) == success, (label, out)
            assert inference["crate_success"] == success, (label, out)
            assert inference["interface_written"] == success, (label, out)
            assert scope["native_collection_completed"], (label, out)
            assert scope["complete_selected_scope_verified"] == success, (label, out)
            assert any(not f["selected"] and "Sum" in f["rust_function"] for f in scope["native_functions"])
            assert any(not f["selected"] and "Encode" in f["rust_function"] for f in scope["native_functions"])
            if success or label == "false_goal":
                functions = inference["functions"]
                assert len(functions) == 10 and all(f["required_checks_passed"] and f["mode"] == "exact"
                    and all(f["proofs"][p]["required"] and f["proofs"][p]["status"] == "proved"
                            for p in ["soundness", "maximality", "exactness"]) for f in functions), (label, out)
            if not success:
                assert scope["checked_exec_bodies"] == 0, (label, out)
            if label in ["excluded_sum", "excluded_codec"]:
                assert "cannot use function" in (out / f"{label}.stderr").read_text(), (label, out)
            if label == "excluded_request":
                assert "derived trait request requires one checked implementation" in (out / f"{label}.stderr").read_text(), (label, out)
            receipt["cases"].append({"case": label, "expected_success": success, "verification": result,
                                     "inference": inference})
        native_env = dict(env, RUSTUP_TOOLCHAIN=args.native_toolchain, CARGO_TARGET_DIR=str(out / "native-target"),
                          CARGO_ENCODED_RUSTFLAGS="")
        assert run("native", ["cargo", "--config", patch, "test", "--offline", "--locked", "--manifest-path", str(manifest)], native_env) == 0
        receipt["success"] = True
    finally:
        (out / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(out)
    print("Real derives, ten exact helpers, scope/call/publication controls and native erasure passed")


if __name__ == "__main__":
    main()
