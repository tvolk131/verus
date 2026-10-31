#!/usr/bin/env python3
"""Replay the native public byte-vector decoder milestone without copying/replacing its algorithms.

Mutations run in the explicitly supplied native checkout and are restored in a
finally block. The caller must have write permission for that checkout. Proofs
use its actual src/lib.rs, default std+alloc cfg and a separate fresh export/cache
per invocation. No Cargo configuration or accepted Aleph fixture is modified.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess

from infer_eval_support import digest as sha, support_artifacts
from infer_experiment import LoggedCommands, require_exact, require_publication, validate_run


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--positive-only", action="store_true")
    args = parser.parse_args()
    root = args.native_root.resolve()
    engine = Path(__file__).resolve().parents[2]
    binary = engine / "source/target-verus/release"
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    paths = [root / "src/lib.rs", root / "src/iter.rs", root / "src/error.rs"]
    originals = {p: p.read_bytes() for p in paths}
    sources = sorted((root / "src").glob("*.rs")) + sorted((root / "tests").rglob("*.rs")) + [
        root / name for name in ["Cargo.toml", "Cargo.lock", "Cargo-recent.lock", "VERUS_INFER.md"]]
    receipt = {"native_sources_before": {str(p.relative_to(root)): sha(p) for p in sources},
               "artifacts": {n: sha(binary / n) for n in ["verus", "rust_verify", "vstd.vir", "libvstd.rlib", "libverus_builtin_macros.dylib"]},
               "runner_sha256": sha(Path(__file__).resolve()),
               "runner_support_sha256": {p.name: sha(p) for p in support_artifacts()},
               "native_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
               "proof_toolchain": subprocess.check_output(["rustc", "+1.99.1", "--version"], text=True).strip(),
               "cases": [], "commands": []}
    env = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.1", VERUS_Z3_PATH=str(engine / "source/z3"))
    for key in ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"]:
        env.pop(key, None)

    run = LoggedCommands(out, engine, receipt["commands"])

    def proof(label, expected=True, flags=(), assistance=True):
        artifact = out / (label + ".vir")
        environment = dict(env, VERUS_INFER_CACHE=str(out / (label + "-cache")))
        command = [str(binary / "verus"), str(root / "src/lib.rs"), "--crate-type", "lib",
                   "--crate-name", "hex_conservative", "--cfg", 'feature="std"', "--cfg", 'feature="alloc"',
                   "--cfg", "verus_hex_vec_phase3", "--infer-goals", "--no-cheating", "--output-json", "--export", str(artifact), *flags]
        if not assistance:
            command.remove("--infer-goals")
        code = run(label, command, environment)
        report = validate_run(dict(returncode=code,
            stdout=(out / (label + ".stdout")).read_text(),
            stderr=(out / (label + ".stderr")).read_text()),
            expected, allow_frontend=bool(flags))
        result = report["verification-results"]
        inference = report["verus-infer"]
        require_publication(report, expected, artifact)
        if expected:
            functions = inference["functions"]
            assert len(functions) == 1, (label, functions)
            assert "hex_chars_to_byte" in functions[0]["function"], (label, functions)
            require_exact(functions)
            assert inference["verification_scope"]["complete_selected_scope_verified"], (label, out)
            checked = {r["function"] for r in inference["verification_scope"]["native_functions"] if r["body_status"] == "checked_exec_body"}
            required = ["Char::decode_nibble", "iter::hex_chars_to_byte",
                        "iter::HexToBytesIter::new", "iter::HexToBytesIter::new_unchecked", "iter::HexToBytesIter::from_pairs",
                        "iter::HexToBytesIter::next", "iter::HexToBytesIter::nth",
                        "iter::HexDigitsIter::new_unchecked", "iter::HexDigitsIter::next", "iter::HexDigitsIter::nth", "iter::HexDigitsIter::size_hint",
                        "iter::pair_decoding_goal", "iter::forward_step_goal", "iter::string_forward_goal", "iter::string_next_goal",
                        "iter::original_error_location_goal", "iter::string_exhaustion_goal", "iter::saturated_error_position_goal",
                        "iter::runtime_length_goal", "iter::forward_drain_domain_goal",
                        "iter::HexToBytesIter::drain_to_vec", "iter::HexToBytesIter::size_hint", "decode_to_vec"]
            assert all("hex_conservative::" + name in checked for name in required), (label, checked)
            assert inference["sequence_pattern_assistance"]
            assert len(inference["array_callback_assistance"]) == 2
            assert len(inference["type_invariant_entry_assistance"]) == 4
        receipt["cases"].append({"label": label, "expected_success": expected, "verification": result, "inference": inference})
        return inference

    try:
        proof("fresh")
        assert len(receipt["cases"][-1]["inference"]["array_callback_assistance"]) == 2
        if not args.positive_only:
            cases = [
                ("wrong_table", paths[0], "t[Char::LowerA as usize] = 10", "t[Char::LowerA as usize] = 11"),
                ("swapped_nibbles", paths[1], "Ok((hih << 4) | loh)", "Ok((loh << 4) | hih)"),
                ("invalid_acceptance", paths[0], "if n == 0xFF {", "if n == 0xFE {"),
                ("low_error_payload", paths[1], ".ok_or((lo, false))?", ".ok_or((hi, false))?"),
                ("wrong_output", paths[1], "Ok((hih << 4) | loh)", "Ok(0)"),
                ("error_priority", paths[1],
                 "let hih = Char::decode_nibble(hi).ok_or((hi, true))?;\n    let loh = Char::decode_nibble(lo).ok_or((lo, false))?;",
                 "let loh = Char::decode_nibble(lo).ok_or((lo, false))?;\n    let hih = Char::decode_nibble(hi).ok_or((hi, true))?;"),
            ]
            cases += [
                ("odd_length", paths[1], "if s.len() % 2 != 0 {", "if s.len() % 2 == 0 {"),
                ("utf8_length", paths[1], "if s.len() % 2 != 0 {", "if (s.len() > 0 && s.as_bytes()[0] > 127) || s.len() % 2 != 0 {"),
                ("chunk_width", paths[1], "Self { iter: digits.chunks_exact(2) }", "Self { iter: digits.chunks_exact(3) }"),
                ("digit_skip", paths[1], 'self.iter.nth(n).map(|digits|', 'self.iter.nth(0).map(|digits|'),
                ("digit_next", paths[1], 'self.iter.next().map(|digits|', 'self.iter.nth(1).map(|digits|'),
                ("decoder_next", paths[1], 'fn next(&mut self) -> Option<Self::Item> { self.nth(0) }', 'fn next(&mut self) -> Option<Self::Item> { self.nth(1) }'),
                ("decoder_skip", paths[1], 'let [hi, lo] = self.iter.nth(n)?;', 'let [hi, lo] = self.iter.nth(0)?;'),
                ("cursor_stride", paths[1], 'self.front_pos = self.front_pos.saturating_add(n).saturating_add(1);', 'self.front_pos = self.front_pos.saturating_add(n).saturating_add(2);'),
                ("byte_position", paths[1], 'let pos = self.front_pos.saturating_add(n).saturating_mul(2);', 'let pos = self.front_pos.saturating_add(n).saturating_mul(3);'),
                ("low_position", paths[1], 'pos: if is_high { pos } else { pos.saturating_add(1) },', 'pos: if is_high { pos } else { pos.saturating_add(2) },'),
                ("cursor_overflow", paths[1], 'self.front_pos = self.front_pos.saturating_add(n).saturating_add(1);', 'self.front_pos = (self.front_pos + n).saturating_add(1);'),
                ("exhaustion_cursor", paths[1], 'let [hi, lo] = self.iter.nth(n)?;', 'self.front_pos = self.front_pos.saturating_add(1);\n        let [hi, lo] = self.iter.nth(n)?;'),
            ]
            cases += [
                ("drain_stride", paths[1], 'ptr = ptr.add(1);', 'ptr = ptr.add(2);'),
                ("drain_uninitialized", paths[1], 'ret.set_len(len);', 'ret.set_len(len.saturating_add(1));'),
                ("drain_wrong_length", paths[1], 'ret.set_len(len);', 'ret.set_len(len.saturating_sub(1));'),
                ("drain_short_capacity", paths[1], 'let mut ret = Vec::with_capacity(len);', 'let capacity = len.saturating_sub(1);\n        let mut ret = Vec::with_capacity(capacity);'),
                ("public_error_conversion", paths[2], 'fn from(e: InvalidCharError) -> Self { Self::InvalidChar(e) }', 'fn from(e: InvalidCharError) -> Self { Self::InvalidChar(InvalidCharError {invalid:e.invalid,pos:0}) }'),
                ("runtime_hint", paths[1], 'fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }', 'fn size_hint(&self) -> (usize, Option<usize>) { (0, Some(0)) }'),
            ]
            for label, path, before, after in cases:
                text = originals[path].decode()
                assert before in text, (label, "missing mutation")
                if label == "drain_stride":
                    start = text.index("pub(crate) fn drain_to_vec")
                    text = text[:start] + text[start:].replace(before, after, 1)
                    path.write_text(text)
                else:
                    path.write_text(text.replace(before, after, 1))
                try:
                    inference = proof(label, False)
                    assert len(inference["functions"]) == 1 and inference["functions"][0]["required_checks_passed"], (label, out)
                    errors = (out / (label + ".stderr")).read_text()
                    assert any(e in errors for e in ["postcondition not satisfied", "invariant", "overflow", "precondition not satisfied"]), (label, out)
                finally:
                    path.write_bytes(originals[path])
            inference = proof("without_assistance", False, assistance=False)
            assert inference["functions"][0]["required_checks_passed"], ("without_assistance", out)
            assert "postcondition not satisfied" in (out / "without_assistance.stderr").read_text()
            proof("no_verify", False, ["--no-verify"])
            proof("filtered", False, ["--verify-root", "--verify-function", "decode_nibble"])
            proof("no_lifetime", False, ["--no-lifetime"])
            assert "requires normal tracked lifetime checks" in (out / "no_lifetime.stderr").read_text()
            for label in ["no_verify", "filtered"]:
                assert "requires full-crate verification" in (out / (label + ".stderr")).read_text(), (label, out)
        proof("restored")
        for label, features in [("default", []), ("all", ["--all-features"]),
                                ("no_default", ["--no-default-features"]),
                                ("alloc", ["--no-default-features", "--features", "alloc"])]:
            environment = dict(env, CARGO_TARGET_DIR=str(out / "native-target"))
            assert run("native_" + label, ["cargo", "+1.99.1", "test", "--offline", "--locked",
                       "--manifest-path", str(root / "Cargo.toml"), "-p", "hex-conservative", *features], environment) == 0, (label, out)
        environment = dict(env, RUSTUP_TOOLCHAIN="1.74.0", CARGO_TARGET_DIR=str(out / "msrv-target"))
        assert run("native_msrv", ["cargo", "+1.74.0", "test", "--offline", "--locked", "--manifest-path", str(root / "Cargo.toml"), "-p", "hex-conservative"], environment) == 0
        receipt["success"] = True
    finally:
        for path, original in originals.items():
            if path.read_bytes() != original:
                path.write_bytes(original)
        receipt["native_sources_after"] = {str(p.relative_to(root)): sha(p) for p in sources}
        assert receipt["native_sources_before"] == receipt["native_sources_after"], "source restoration failed"
        (out / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(out)


if __name__ == "__main__":
    main()
