#!/usr/bin/env python3
"""Replay the explicitly unsupported original native vector-memory phase.

The phase must fail closed on four unmodeled real memory operations. A separate
restored iterator/native replay must succeed. This is a blocker receipt, never
acceptance of public vector decoding. No source is mutated by this controller.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    native = args.native_root.resolve()
    engine = Path(__file__).resolve().parents[2]
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    sources = sorted((native / "src").glob("*.rs")) + [native / p for p in ["Cargo.toml", "Cargo.lock", "Cargo-recent.lock", "VERUS_INFER.md"]]
    before = {str(p.relative_to(native)): sha(p) for p in sources}
    binary = engine / "source/target-verus/release"
    env = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.1", VERUS_Z3_PATH=str(engine / "source/z3"), VERUS_INFER_CACHE=str(out / "phase-cache"))
    for key in ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"]:
        env.pop(key, None)
    artifact = out / "blocked.vir"
    command = [str(binary / "verus"), str(native / "src/lib.rs"), "--crate-type", "lib", "--crate-name", "hex_conservative",
               "--cfg", 'feature="std"', "--cfg", 'feature="alloc"', "--cfg", "verus_hex_vec_phase3",
               "--infer-goals", "--no-cheating", "--output-json", "--export", str(artifact)]
    with (out / "blocked.stdout").open("w") as stdout, (out / "blocked.stderr").open("w") as stderr:
        result = subprocess.run(command, cwd=engine, env=env, stdout=stdout, stderr=stderr)
    report = json.loads((out / "blocked.stdout").read_text())
    errors = (out / "blocked.stderr").read_text()
    assert result.returncode != 0 and not report["verification-results"]["success"]
    assert not artifact.exists() and not report["verus-infer"]["interface_written"]
    lines = [line for line in errors.splitlines() if line.startswith("error:")]
    missing = ["::as_mut_ptr` is not supported", "`core::ptr::write` is not supported", "::add` is not supported", "::set_len` is not supported"]
    assert len(lines) == 5 and all(any(item in line for line in lines) for item in missing), lines
    replay = ["python3", str(engine / "source/tools/check_hex_iterator.py"), "--native-root", str(native), "--output", str(out / "restored-iterator"), "--positive-only"]
    with (out / "restored.stdout").open("w") as stdout, (out / "restored.stderr").open("w") as stderr:
        restored = subprocess.run(replay, cwd=engine, env=env, stdout=stdout, stderr=stderr)
    assert restored.returncode == 0
    after = {str(p.relative_to(native)): sha(p) for p in sources}
    assert before == after
    receipt = dict(status="public endpoint unsupported; restored iterator/native checks pass", public_vector_verified=False,
                   failed_probe_command=command, failed_probe_exit=result.returncode, unsupported_operation_diagnostics=lines,
                   interface_written=False, restored_command=replay, restored_exit=restored.returncode,
                   sources_before=before, sources_after=after, runner_sha256=sha(Path(__file__).resolve()),
                   artifacts={p:sha(binary / p) for p in ["verus", "rust_verify", "vstd.vir", "libvstd.rlib", "libverus_builtin_macros.dylib"]})
    (out / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(out)


if __name__ == "__main__":
    main()
