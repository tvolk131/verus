#!/usr/bin/env python3
"""Check real derives, explicit unused scope and failed-call/publication controls.

Build the matching release compiler, macros and vstd first. All commands are
offline and retain their exact stdout/stderr and structured receipts.
"""
import argparse
import json
import os
from pathlib import Path
import re
import tempfile

from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import LoggedCommands, positive_timeout, replay_receipt, require


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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verus-source", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--native-toolchain", default="1.85.1")
    parser.add_argument("--timeout", type=positive_timeout, default=300.0,
                        help="positive per-command timeout in seconds (default: 300)")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    out = Path(tempfile.mkdtemp(prefix="run-", dir=args.output.resolve()))
    receipt = {"commands": [], "cases": []}
    with replay_receipt(out, receipt):
        replay(args, out, receipt)
    print(out)
    print("Real derives, ten exact helpers, scope/call/publication controls and native erasure passed")


def replay(args, out, receipt):
    source = args.verus_source.resolve()
    fixture = Path(__file__).resolve().parent / "native_adoption"
    manifest = fixture / "Cargo.toml"
    binary = source / "target-verus/release"
    guard = ArtifactGuard(tool_artifacts(binary / "verus", source / "z3",
                                         extra=[binary / "cargo-verus", Path(__file__).resolve()]))
    channel = re.search(r'channel\s*=\s*"([^"]+)"', (source.parent / "rust-toolchain.toml").read_text()).group(1)
    patch = "patch.crates-io.vstd.path=" + json.dumps(str(source / "vstd"))
    env = dict(os.environ, RUSTUP_TOOLCHAIN=channel,
               CARGO_TARGET_DIR=str(out / "proof-target"), VERUS_Z3_PATH=str(source / "z3"),
               VERUS_INFER_CACHE=str(out / "candidate-cache"))
    env["PATH"] = str(binary) + os.pathsep + env.get("PATH", "")
    receipt.update({
        "engine_sha256": {Path(path).name: value["sha256"] for path, value in guard.initial.items()},
        "fixture_sha256": {str(p.relative_to(fixture)): digest(p) for p in [manifest, fixture / "Cargo.lock", fixture / "src/lib.rs"]},
        "verification_toolchain": channel, "native_toolchain": args.native_toolchain,
        "runner_sha256": digest(Path(__file__).resolve()),
    })

    commands = LoggedCommands(out, source.parent, receipt["commands"], timeout=args.timeout,
                              guard=guard, status_key="exit_status")

    def run(label, command, environment=env):
        return commands(label, command, environment)

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
    for label, features, flags, success in cases:
        clean = ["cargo", "--config", patch, "clean", "--offline", "--manifest-path", str(manifest), "-p", "verus-infer-native-adoption"]
        require(run(label + "_clean", clean) == 0, 'native replay validation failed')
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
            require(code != 0 and "Partial verification must use `cargo verus focus`" in (out / f"{label}.stderr").read_text(), (label, out))
            receipt["cases"].append({"case": label, "expected_success": False,
                                     "stage": "cargo_cli", "verifier_started": False})
            continue
        require(len(parsed) == 1, (label, out))
        result = parsed[0]["verification-results"]
        inference = parsed[0]["verus-infer"]
        scope = inference["verification_scope"]
        require((code == 0) == success and result.get("success", False) == success, (label, out))
        require(inference["crate_success"] == success, (label, out))
        require(inference["interface_written"] == success, (label, out))
        require(scope["native_collection_completed"], (label, out))
        require(scope["complete_selected_scope_verified"] == success, (label, out))
        require(any(not f["selected"] and "Sum" in f["rust_function"] for f in scope["native_functions"]), 'native replay validation failed')
        require(any(not f["selected"] and "Encode" in f["rust_function"] for f in scope["native_functions"]), 'native replay validation failed')
        if success or label == "false_goal":
            functions = inference["functions"]
            require(len(functions) == 10 and all(f["required_checks_passed"] and f["mode"] == "exact"
                and all(f["proofs"][p]["required"] and f["proofs"][p]["status"] == "proved"
                        for p in ["soundness", "maximality", "exactness"]) for f in functions), (label, out))
        if not success:
            require(scope["checked_exec_bodies"] == 0, (label, out))
        if label in ["excluded_sum", "excluded_codec"]:
            require("cannot use function" in (out / f"{label}.stderr").read_text(), (label, out))
        if label == "excluded_request":
            require("derived trait request requires one checked implementation" in (out / f"{label}.stderr").read_text(), (label, out))
        receipt["cases"].append({"case": label, "expected_success": success, "verification": result,
                                 "inference": inference})
    native_env = dict(env, RUSTUP_TOOLCHAIN=args.native_toolchain, CARGO_TARGET_DIR=str(out / "native-target"),
                      CARGO_ENCODED_RUSTFLAGS="")
    require(run("native", ["cargo", "--config", patch, "test", "--offline", "--locked", "--manifest-path", str(manifest)], native_env) == 0, 'native replay validation failed')


if __name__ == "__main__":
    main()
