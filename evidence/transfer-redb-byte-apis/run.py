#!/usr/bin/env python3
"""Replay the frozen redb extraction against a dirty, fingerprinted byte extension."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile


ENGINE = Path(__file__).resolve().parents[2]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ENGINE / "source/tools"))
from infer_eval_support import ArtifactGuard, digest, host_info, tool_artifacts
from infer_experiment import (LoggedCommands, positive_timeout, replay_receipt,
                              require, require_exact, require_publication)


BASE_COMMIT = "55bd67ad980e90e92c77efe4453a927f097c3386"
REDB_COMMIT = "c82ab865bb362bc4af067d332adc44978ff64813"
FROZEN_FIXTURES = {
    "codec.rs": "639d1032cc0fb061cf3e4bf7c8b05b14b13e7d856ce047b09e1f740d6565a3cb",
    "false_codec.rs": "0e023d19c7fe79681aaf3b296cbe3dbe66039a331744f54baf0a47efa6880c46",
    "size.rs": "87195e5d9b9a70cfa8cd4ab627b17465bb9ff8bebb1bffe5fef5f3682f5a89a1",
    "address.rs": "0243d41662b8d84e0800edd0364a27dcfb22f5b7730408b42382d8d2d0b510d4",
    "native.rs": "0d8998daa1cecc5f8b2208251ef84d2ab2a8b671c8fa2c2fbf6452c658516cf7",
    "native_mutated.rs": "7cc34e3b10ee7e7429d18192ced3019e4ceef3671d9b89787acfa052b262c564",
}


def focus_fixtures(output):
    source = (ENGINE / "source/rust_verify_test/tests/infer_byte_apis.rs").read_text()
    constants = {name: re.search(r"const " + name + r': &str = r#"(.*?)"#;', source, re.S).group(1)
                 for name in ("PROVIDER", "GOALS")}
    sources = {"byte_provider": constants["PROVIDER"] + constants["GOALS"], "byte_client": """
        use vstd::prelude::*;
        #[verifier::infer] pub fn relay(number:u64)->[u8;8]{byte_provider::encode(number)}
        #[verifier::infer] pub fn decode(bytes:[u8;8])->u64{byte_provider::decode(bytes)}
        fn encoding(number:u64)->(bytes:[u8;8]) ensures bytes@ == vstd::bytes::spec_u64_to_le_bytes(number), {relay(number)}
        fn decoding(bytes:[u8;8])->(number:u64) ensures number == vstd::bytes::spec_u64_from_le_bytes(bytes@), {decode(bytes)}
    """}
    for name, body in (
        ("reversed", "[bytes[7],bytes[6],bytes[5],bytes[4],bytes[3],bytes[2],bytes[1],bytes[0]]"),
        ("shifted", "[bytes[1],bytes[2],bytes[3],bytes[4],bytes[5],bytes[6],bytes[7],bytes[0]]"),
        ("replaced", "[0,bytes[1],bytes[2],bytes[3],bytes[4],bytes[5],bytes[6],bytes[7]]"),
        ("wrong_goal", "bytes"),
    ):
        sources[name] = ("use vstd::prelude::*; #[verifier::infer] pub fn codec(number:u64)->u64 {"
            "let bytes=number.to_le_bytes(); u64::from_le_bytes(" + body + ")}"
            "fn property(number:u64)->(result:u64) ensures result " + ("!=" if name == "wrong_goal" else "==")
            + " number, {codec(number)}")
    for name, body in sources.items():
        (output / (name + ".rs")).write_text("use vstd::prelude::*; verus! {" + body + "}\n")
    (output / "byte_native.rs").write_text("""
fn main() {
    let mut count = 0;
    for position in 0..8 {
        for byte in 0..=255u64 {
            for background in [0, u64::MAX, 0x0123456789abcdef] {
                let number = (background & !(255 << (position * 8))) | (byte << (position * 8));
                let expected: [u8;8] = core::array::from_fn(|index| (number >> (8*index)) as u8);
                let bytes = byte_provider::encode(number);
                assert_eq!(bytes, expected);
                assert_eq!(byte_provider::decode(expected), number);
                assert_eq!(byte_provider::word_roundtrip(number), number);
                assert_eq!(byte_provider::byte_roundtrip(expected), expected);
                count += 1;
            }
        }
    }
    assert_eq!(byte_provider::constant(), [239,205,171,137,103,69,35,1]);
    assert_eq!(byte_provider::increment(u64::MAX-1), [255;8]);
    println!("native byte boundary cases: {count}");
}
""")
    return sources


def focus_replay(output, run, env, receipt, binary):
    rows = []
    for name, count in (("byte_provider", 6), ("byte_client", 2), ("reversed", 1),
                        ("shifted", 1), ("replaced", 1), ("wrong_goal", 1)):
        positive = name.startswith("byte_")
        for phase in (("cold", "warm") if positive else ("negative",)):
            label = name + "_" + phase
            artifact = output / (name + ".vir")
            env["VERUS_INFER_CACHE"] = str(output / (name + "-cache"))
            command = [str(binary / "verus"), str(output / (name + ".rs")), "--crate-type", "lib",
                "--crate-name", name, "--no-cheating", "--output-json", "--compile",
                "--export", str(artifact), "-o", str(output / ("lib" + name + ".rlib"))]
            if name == "byte_client":
                command += ["--extern", "byte_provider=" + str(output / "libbyte_provider.rlib"),
                            "--import", "byte_provider=" + str(output / "byte_provider.vir")]
            code = run(label, command, env)
            data = json.loads((output / (label + ".stdout")).read_text())
            stderr = (output / (label + ".stderr")).read_text()
            require((code == 0) == positive, label + ": " + stderr[-2000:])
            require_publication(data, positive, artifact)
            functions = data["verus-infer"]["functions"]
            require(len(functions) == count, "wrong focused candidate count")
            require_exact(functions)
            if positive:
                require(all(function["metrics"]["cache"] == ("miss" if phase == "cold" else "hit")
                            for function in functions), "incorrect cache classification")
            else:
                require("postcondition not satisfied" in stderr, "unintended semantic failure")
            rows.append(dict(label=label, report=data))
    provider = (output / "byte_provider.vir").read_bytes()
    start = provider.rindex(b'{"schema":')
    metadata = json.loads(provider[start:-8])
    require(any("to_le_bytes" in str(model) for model in metadata["trusted_models"]))
    require(any("from_le_bytes" in str(model) for model in metadata["trusted_models"]))
    for mutation, expected in (("policy", "incompatible inference schema, safety policy, or producer"),
                               ("proof", "incomplete inference proof guarantees"),
                               ("summary", "inferred contract fingerprint mismatch"),
                               ("model", "stale or missing trusted-model fingerprint")):
        changed = json.loads(json.dumps(metadata))
        if mutation == "model":
            changed["trusted_models"][0][1] = "stale"
        else:
            key, value = {"policy": ("policy", "candidate-bound-evidence-v54"),
                          "proof": ("soundness", False), "summary": ("summaries", "corrupted")}[mutation]
            changed[key] = value
        bad = output / (mutation + ".vir")
        bad.write_bytes(provider[:start] + json.dumps(changed).encode() + b"VRINFER1")
        label = "metadata_" + mutation
        artifact = output / (label + ".vir")
        code = run(label, [str(binary / "verus"), str(output / "byte_client.rs"), "--crate-type", "lib",
            "--crate-name", label, "--no-cheating", "--output-json", "--export", str(artifact),
            "--extern", "byte_provider=" + str(output / "libbyte_provider.rlib"),
            "--import", "byte_provider=" + str(bad)], env)
        require(code != 0 and not artifact.exists())
        require(expected in (output / (label + ".stderr")).read_text(), "unintended import failure")
        rows.append(dict(label=label, rejected_for=expected))
    require(run("byte_native_compile", ["rustc", "--edition=2024", str(output / "byte_native.rs"),
        "--extern", "byte_provider=" + str(output / "libbyte_provider.rlib"),
        "-L", "dependency=" + str(binary), "-L", "dependency=" + str(output),
        "-o", str(output / "byte_native")], env) == 0)
    require(run("byte_native_execute", [str(output / "byte_native")], env) == 0)
    receipt["focus"] = rows
    receipt["byte_provider_metadata"] = metadata


def source_snapshot(output):
    paths = subprocess.check_output(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"], cwd=ENGINE, timeout=30
    ).decode().split("\0")
    paths = sorted(path for path in set(paths) if path and (
        path.startswith(("source/", "dependencies/", "tools/")) or path == "rust-toolchain.toml"))
    hashes = {path: digest(ENGINE / path) for path in paths if (ENGINE / path).is_file()}
    tree = hashlib.sha256(json.dumps(hashes, sort_keys=True).encode()).hexdigest()
    (output / "engine-source.json").write_text(json.dumps(dict(sha256=tree, files=hashes), indent=2) + "\n")
    diff = subprocess.check_output(["git", "diff", "--binary", "HEAD"], cwd=ENGINE, timeout=30)
    tracked = set(subprocess.check_output(["git", "ls-files", "-z"], cwd=ENGINE, timeout=30).decode().split("\0"))
    for path in paths:
        if path not in tracked:
            addition = subprocess.run(["git", "diff", "--no-index", "--binary", "--", "/dev/null", path],
                                      cwd=ENGINE, capture_output=True, check=False, timeout=30)
            require(addition.returncode == 1, "cannot record new source file")
            diff += addition.stdout
    (output / "engine.patch").write_bytes(diff)
    return dict(base=BASE_COMMIT, source_tree_sha256=tree,
                patch_sha256=digest(output / "engine.patch"), files=hashes)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path, required=True,
                        help="read-only frozen transfer-redb evidence directory")
    parser.add_argument("--redb-source", type=Path, required=True,
                        help="read-only pinned application clone; a separate clone is made")
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument("--timeout", type=positive_timeout, default=180)
    args = parser.parse_args()
    require(args.output_root.is_dir(), "output root must exist")
    output = Path(tempfile.mkdtemp(prefix="run-", dir=args.output_root.resolve()))
    receipt = dict(commands=[], cases=[], host=host_info(), output=str(output),
                   verification_accepted=False, redb_commit=REDB_COMMIT)
    with replay_receipt(output, receipt):
        env = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.1", VERUS_Z3_PATH=str(ENGINE / "source/z3"))
        run = LoggedCommands(output, ENGINE, receipt["commands"], timeout=args.timeout)
        baseline = args.baseline.resolve()
        frozen_manifest = json.loads((baseline / "manifest.json").read_text())
        for name in ("run.py", "native_harness.rs", "README.md"):
            require(digest(baseline / name) == frozen_manifest["durable_files"]["evidence/transfer-redb/" + name],
                    "frozen baseline changed")
        original_receipt = Path(frozen_manifest["raw_evidence"]["authoritative_replay"]) / "receipt.json"
        if original_receipt.exists():
            require(digest(original_receipt) == frozen_manifest["raw_evidence"]["receipt_sha256"],
                    "frozen original receipt changed")
        spec = importlib.util.spec_from_file_location("frozen_redb", baseline / "run.py")
        frozen = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(frozen)
        require(frozen.ENGINE_COMMIT == BASE_COMMIT and frozen.REDB_COMMIT == REDB_COMMIT)
        receipt["baseline"] = dict(path=str(baseline), files={path: digest(baseline / path)
            for path in ("README.md", "manifest.json", "run.py", "native_harness.rs")})
        for label, root, expected in (("engine_head", ENGINE, BASE_COMMIT),
                                      ("source_head", args.redb_source, REDB_COMMIT)):
            require(run(label, ["git", "-C", str(root), "rev-parse", "HEAD"], env) == 0)
            require((output / (label + ".stdout")).read_text().strip() == expected, "pin mismatch")
        require(run("clone", ["git", "clone", "--local", "--no-hardlinks", str(args.redb_source),
                              str(output / "redb")], env) == 0)
        root = output / "redb"
        require(run("redb_clean", ["git", "-C", str(root), "diff", "--exit-code", "HEAD"], env) == 0)
        receipt["engine"] = source_snapshot(output)
        receipt["source_hashes"] = {path: digest(root / path) for path in
            (frozen.BASE, frozen.LAYOUT, "src/tree_store/page_store/page_manager.rs", "AGENTS.md")}
        receipt["extraction"] = frozen.extract(root, output)
        require(receipt["source_hashes"] == frozen_manifest["application"]["source_hashes"])
        require(receipt["extraction"] == frozen_manifest["application"]["extraction"])
        require(all(digest(output / name) == expected for name, expected in FROZEN_FIXTURES.items()),
                "frozen executable bodies, domains or native controls changed")
        receipt["frozen_source_comparison"] = dict(extraction_matches=True, generated_matches=True,
            fixtures=FROZEN_FIXTURES, original_receipt_sha256=digest(original_receipt)
            if original_receipt.exists() else None)
        focus_fixtures(output)
        receipt["generated_sources"] = {path.name: digest(path) for path in output.glob("*.rs")}
        receipt["domain"] = dict(region_max="0x000fffff", order_max=20,
            index_max="0x000fffff >> order", quantified="all such values",
            reserved_bits="native-only separate audit; canonical encoding clears them")
        require(run("sysroot", ["rustc", "--print", "sysroot"], env) == 0)
        sysroot = Path((output / "sysroot.stdout").read_text().strip())
        binary = ENGINE / "source/target-verus/release"
        guard = ArtifactGuard(tool_artifacts(binary / "verus", ENGINE / "source/z3", extra=[
            Path(__file__), baseline / "run.py", baseline / "native_harness.rs",
            baseline / "README.md", baseline / "manifest.json",
            *([original_receipt] if original_receipt.exists() else []),
            sysroot / "bin/rustc", *sysroot.glob("lib/librustc_driver*.dylib"),
            *sysroot.glob("lib/rustlib/*/lib/libcore-*.rlib"),
            *sysroot.glob("lib/rustlib/*/lib/libstd-*.rlib"),
            *(root / path for path in receipt["source_hashes"]),
            *(args.redb_source / path for path in receipt["source_hashes"]),
            *(ENGINE / path for path in receipt["engine"]["files"]), *output.glob("*.rs")]))
        receipt["artifacts"] = guard.initial
        run.guard = guard
        for label, argv in (("compiler_version", [str(binary / "verus"), "--version"]),
                            ("rust_version", ["rustc", "-vV"]),
                            ("solver_version", [str(ENGINE / "source/z3"), "-version"])):
            require(run(label, argv, env) == 0)
        focus_replay(output, run, env, receipt, binary)
        for label, source in (("native", "native.rs"), ("native_mutated", "native_mutated.rs")):
            require(run(label + "_compile", ["rustc", "--edition=2024", "-O", "-C",
                "debug-assertions=yes", str(output / source), "-o", str(output / label)], env) == 0)
            code = run(label + "_execute", [str(output / label)], env)
            require((code == 0) == (label == "native"), "unexpected native outcome")
            if label == "native_mutated":
                require("region roundtrip" in (output / (label + "_execute.stderr")).read_text())
        for case in ("codec", "false_codec", "size", "address", "address_release"):
            cache = output / (case + "-cache")
            for phase in ("cold", "warm"):
                label = case + "_" + phase
                artifact = output / (label + ".vir")
                env["VERUS_INFER_CACHE"] = str(cache)
                source = "address" if case == "address_release" else case
                command = [str(binary / "verus"), str(output / (source + ".rs")),
                    "--crate-type", "lib", "--crate-name", "redb_" + case, "--infer-goals",
                    "--no-cheating", "--output-json", "--export", str(artifact)]
                if case == "address_release":
                    command += ["-C", "debug-assertions=no"]
                try:
                    code = run(label, command, env)
                except subprocess.TimeoutExpired:
                    receipt["cases"].append(dict(label=label, classification="timeout_incomplete"))
                    continue
                report = json.loads((output / (label + ".stdout")).read_text())
                stderr = (output / (label + ".stderr")).read_text()
                classification = frozen.classify(code, report, stderr)
                if "verus-infer" in report:
                    require_publication(report, classification == "verified", artifact)
                    if classification == "verified":
                        require_exact(report["verus-infer"]["functions"])
                else:
                    require(not artifact.exists(), "frontend failure published an interface")
                receipt["cases"].append(dict(label=label, classification=classification,
                    primitive_byte_frontend_rejection="u64::to_le_bytes` is not supported" in stderr
                        or "u64::from_le_bytes` is not supported" in stderr,
                    report=report, artifact_exists=artifact.exists(),
                    cache_files=len(list(cache.glob("*.json"))) if cache.exists() else 0))
        guard.check()
        receipt["source_hashes_after"] = {path: digest(root / path) for path in receipt["source_hashes"]}
        require(receipt["source_hashes_after"] == receipt["source_hashes"], "application changed")
        receipt["verification_accepted"] = all(case["classification"] == "verified"
            for case in receipt["cases"] if case["label"].startswith("codec_"))
        receipt["classification"] = "bounded_byte_extension_replay"
    print(output)


if __name__ == "__main__":
    main()
