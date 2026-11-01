#!/usr/bin/env python3
"""Replay the explicitly unsupported original native vector-memory phase.

The phase must fail closed on four unmodeled real memory operations. A separate
restored iterator/native replay must succeed. This is a blocker receipt, never
acceptance of public vector decoding. No source is mutated by this controller.
"""
import argparse
import json
import os
from pathlib import Path
import sys

from infer_eval_support import ArtifactGuard, digest as sha, tool_artifacts
from infer_experiment import LoggedCommands, positive_timeout, replay_receipt, require, validate_run


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=positive_timeout, default=300.0,
                        help="positive per-command timeout in seconds (default: 300)")
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    receipt = {"commands": [], "public_vector_verified": False}
    with replay_receipt(out, receipt):
        replay(args, out, receipt)
    print(out)


def replay(args, out, receipt):
    native = args.native_root.resolve()
    engine = Path(__file__).resolve().parents[2]
    binary = engine / "source/target-verus/release"
    guard = ArtifactGuard(tool_artifacts(binary / "verus", engine / "source/z3",
                                         extra=[Path(__file__).resolve(),
                                                engine / "source/tools/check_hex_iterator.py"]))
    receipt.update(runner_sha256=sha(Path(__file__).resolve()),
                   artifacts={Path(path).name: value["sha256"] for path, value in guard.initial.items()})
    sources = sorted((native / "src").glob("*.rs")) + [native / p for p in ["Cargo.toml", "Cargo.lock", "Cargo-recent.lock", "VERUS_INFER.md"]]
    before = {str(p.relative_to(native)): sha(p) for p in sources}
    receipt["sources_before"] = before
    env = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.1", VERUS_Z3_PATH=str(engine / "source/z3"), VERUS_INFER_CACHE=str(out / "phase-cache"))
    for key in ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"]:
        env.pop(key, None)
    artifact = out / "blocked.vir"
    command = [str(binary / "verus"), str(native / "src/lib.rs"), "--crate-type", "lib", "--crate-name", "hex_conservative",
               "--cfg", 'feature="std"', "--cfg", 'feature="alloc"', "--cfg", "verus_hex_vec_phase3",
               "--infer-goals", "--no-cheating", "--output-json", "--export", str(artifact)]
    run = LoggedCommands(out, engine, receipt["commands"], timeout=args.timeout, guard=guard)
    try:
        code = run("blocked", command, env)
        receipt.update(failed_probe_command=command, failed_probe_exit=code)
        errors = (out / "blocked.stderr").read_text()
        report = validate_run(dict(returncode=code,
            stdout=(out / "blocked.stdout").read_text(), stderr=errors), False, allow_frontend=True)
        require(report["verification-results"].get("success") is False, "phase 3 must fail closed")
        require(not artifact.exists() and not report["verus-infer"]["interface_written"],
                "blocked phase 3 published an interface")
        lines = [line for line in errors.splitlines() if line.startswith("error:")]
        missing = ["::as_mut_ptr` is not supported", "`core::ptr::write` is not supported", "::add` is not supported", "::set_len` is not supported"]
        receipt["unsupported_operation_diagnostics"] = lines
        require(len(lines) == 5 and all(any(item in line for line in lines) for item in missing), lines)
        restored_command = [sys.executable, str(engine / "source/tools/check_hex_iterator.py"), "--native-root", str(native), "--output", str(out / "restored-iterator"), "--positive-only", "--timeout", str(args.timeout)]
        restored = run("restored", restored_command, env)
        receipt.update(restored_command=restored_command, restored_exit=restored)
        require(restored == 0, "restored iterator/native replay failed")
        restored_receipt = json.loads((out / "restored-iterator/receipt.json").read_text())
        require(restored_receipt.get("success") is True, "restored replay has no successful receipt")
    finally:
        after = {}
        for path in sources:
            after[str(path.relative_to(native))] = sha(path) if path.is_file() else None
        receipt["sources_after"] = after
        require(before == after, "native sources changed during phase 3 replay")
    receipt.update(status="public endpoint unsupported; restored iterator/native checks pass",
                   interface_written=False)


if __name__ == "__main__":
    main()
