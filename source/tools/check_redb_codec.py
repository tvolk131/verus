"""Replay the unchanged pinned redb PageNumber codec and independent controls."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from infer_eval_support import ArtifactGuard, digest, host_info, tool_artifacts
from infer_experiment import (
    LoggedCommands, positive_timeout, replay_receipt, require, require_exact,
    require_publication, validate_run,
)


APPLICATION = "c82ab865bb362bc4af067d332adc44978ff64813"
BASE = "src/tree_store/page_store/base.rs"
SOURCE_HASH = "ce2ae1188b5bad2babe2f7b7df19d82adc2b8baff6b41b957e978437686938c4"
FROZEN_HASHES = {
    "PageNumber": "2f35314985c2cba913372168d6fc73f5633b0de1bfc720f73fc5418460b9d867",
    "to_le_bytes": "284704938b6d28b2b3ed3719d9a5ac7ba8ff5b7928410d4b407dd03588554534",
    "from_le_bytes": "10423f60e80091cbe34a029b491e53797ac6b63e388892949136bf4bd8e4f02c",
    "codec.rs": "639d1032cc0fb061cf3e4bf7c8b05b14b13e7d856ce047b09e1f740d6565a3cb",
}
ROUNDTRIP = """
fn canonical_roundtrip(region: u32, page_index: u32, page_order: u8) -> (decoded: PageNumber)
    requires
        region <= 0x000F_FFFF,
        page_order <= 20,
        page_index <= (0x000F_FFFFu32 >> page_order),
    ensures
        decoded.region == region,
        decoded.page_index == page_index,
        decoded.page_order == page_order,
{
    let page = PageNumber { region, page_index, page_order };
    PageNumber::from_le_bytes(page.to_le_bytes())
}
"""
PACKING_PROOF = """
    proof {
        let word: u64 = (0x000F_FFFFu64 & (page_index as u64))
            | ((0x000F_FFFFu64 & (region as u64)) << 20)
            | ((31u64 & (page_order as u64)) << 59);
        assert(
            ((word >> 59) as u8) == page_order
            && (((word >> 20) & 0x000F_FFFFu64) as u32) == region
            && ((word & (0x000F_FFFFu64 >> ((word >> 59) as u8))) as u32) == page_index
            && ((word >> 59) as u8) < 64
            && (word & (0x000F_FFFFu64 >> ((word >> 59) as u8))) <= 4294967295u64
        ) by (bit_vector)
            requires
                region <= 0x000F_FFFF,
                page_order <= 20,
                page_index <= (0x000F_FFFFu32 >> page_order),
                word == (0x000F_FFFFu64 & (page_index as u64))
                    | ((0x000F_FFFFu64 & (region as u64)) << 20)
                    | ((31u64 & (page_order as u64)) << 59);
        vstd::bytes::lemma_auto_spec_u64_to_from_le_bytes();
        assert(encoded@ == vstd::bytes::spec_u64_to_le_bytes(word));
        assert(vstd::bytes::spec_u64_from_le_bytes(encoded@) == word);
    }
"""
INJECTIVITY = """
fn canonical_injectivity(region: u32, page_index: u32, page_order: u8,
    other_region: u32, other_index: u32, other_order: u8)
    requires
        region <= 0x000F_FFFF, page_order <= 20,
        page_index <= (0x000F_FFFFu32 >> page_order),
        other_region <= 0x000F_FFFF, other_order <= 20,
        other_index <= (0x000F_FFFFu32 >> other_order),
{
    let first = PageNumber { region, page_index, page_order };
    let second = PageNumber { region: other_region, page_index: other_index, page_order: other_order };
    let first_bytes = first.to_le_bytes();
    let second_bytes = second.to_le_bytes();
    if first_bytes == second_bytes {
        let decoded = PageNumber::from_le_bytes(first_bytes);
        assert(decoded.region == region && decoded.page_index == page_index && decoded.page_order == page_order);
        assert(decoded.region == other_region && decoded.page_index == other_index && decoded.page_order == other_order);
        assert(region == other_region && page_index == other_index && page_order == other_order);
    }
}
"""
FOCUS = """
use vstd::prelude::*;
verus! {
#[verifier::infer] pub fn narrow(number: u64) -> u32 { u32::try_from(number).unwrap() }
#[verifier::infer] pub fn signed(number: i64) -> u8 { u8::try_from(number).unwrap() }
fn caller(number: u64) -> (result: u32)
    requires number <= u32::MAX,
    ensures result == number,
{ narrow(number) }
fn signed_caller(number: i64) -> (result: u8)
    requires 0 <= number <= u8::MAX,
    ensures result == number,
{ signed(number) }
}
"""
NATIVE = """
fn check(region: u32, page_index: u32, page_order: u8) {
    let original = PageNumber { region, page_index, page_order };
    let bytes = original.to_le_bytes();
    let word = u64::from(page_index) | (u64::from(region) << 20) | (u64::from(page_order) << 59);
    let expected: [u8; 8] = core::array::from_fn(|position| (word >> (position * 8)) as u8);
    assert_eq!(bytes, expected, "encoding oracle");
    let decoded = PageNumber::from_le_bytes(bytes);
    assert_eq!(decoded.region, region, "region roundtrip");
    assert_eq!(decoded.page_index, page_index, "index roundtrip");
    assert_eq!(decoded.page_order, page_order, "order roundtrip");
}
fn main() {
    check(1, 0, 0);
    let mut count = 1u64;
    for order in 0..=20u8 {
        let maximum = 0x000F_FFFFu32 >> order;
        for index in 0..=maximum {
            let region = index.wrapping_mul(2654435761) & 0x000F_FFFF;
            check(region, index, order);
            count += 1;
        }
        for region in [0, 1, 0x7_FFFF, 0xF_FFFE, 0xF_FFFF] {
            for index in [0, maximum / 2, maximum] {
                check(region, index, order);
                count += 1;
            }
        }
    }
    println!("native codec cases: {count}");
}
"""


def item(text, marker):
    start = text.index(marker)
    opening = text.index("{", start)
    depth = 1
    end = opening + 1
    while depth:
        depth += (text[end] == "{") - (text[end] == "}")
        end += 1
    return text[start:end]


def extract(application, output):
    require(digest(application / BASE) == SOURCE_HASH, "pinned application source changed")
    text = (application / BASE).read_text()
    pieces = {"PageNumber": item(text, "pub(crate) struct PageNumber")}
    pieces.update({name: item(text, "    pub(crate) fn " + name + "(")
                   for name in ("to_le_bytes", "from_le_bytes")})
    for name, contents in pieces.items():
        path = output / (name + ".original.rs")
        path.write_text(contents + "\n")
        require(digest(path) == FROZEN_HASHES[name], "frozen executable extraction changed: " + name)
    header = "use vstd::prelude::*;\nuse core::ops::Range;\nverus! {\n"
    codec = header + pieces["PageNumber"] + "\nimpl PageNumber {\n"
    codec += "\n".join("#[verifier::infer]\n" + pieces[name]
                        for name in ("to_le_bytes", "from_le_bytes")) + "\n}\n"
    (output / "codec.rs").write_text(codec + ROUNDTRIP + "}\n")
    require(digest(output / "codec.rs") == FROZEN_HASHES["codec.rs"], "frozen domain or caller changed")
    authored = ROUNDTRIP.replace("    PageNumber::from_le_bytes(page.to_le_bytes())",
                                "    let encoded = page.to_le_bytes();\n" + PACKING_PROOF + "    PageNumber::from_le_bytes(encoded)")
    public_codec = codec.replace("pub(crate)", "pub")
    client = (ROUNDTRIP + INJECTIVITY).replace("PageNumber", "codec_provider::PageNumber")
    client = client.replace("codec_provider::PageNumber::from_le_bytes(page.to_le_bytes())", "relay(page.to_le_bytes())")
    sources = {
        "codec_authored": codec + authored + "}\n",
        "codec_complete": codec + ROUNDTRIP + INJECTIVITY + "}\n",
        "codec_provider": public_codec + ROUNDTRIP + INJECTIVITY + "}\n",
        "codec_client": "use vstd::prelude::*; verus! {\n"
            + "#[verifier::infer] pub fn relay(bytes:[u8;8])->codec_provider::PageNumber {codec_provider::PageNumber::from_le_bytes(bytes)}\n"
            + client + "}\n",
        "decoder_mutation": codec.replace("((temp >> 20) & 0x000F_FFFF)", "((temp >> 21) & 0x000F_FFFF)") + authored + "}\n",
        "decoder_mutation_unaided": codec.replace("((temp >> 20) & 0x000F_FFFF)", "((temp >> 21) & 0x000F_FFFF)") + ROUNDTRIP + "}\n",
        "false_goal": codec + authored.replace("decoded.region == region,", "decoded.region != region,") + "}\n",
        "malformed_goal": codec + authored.replace("decoded.region == region,", "decoded.region == true,") + "}\n",
        "conversion_provider": FOCUS,
        "panic_goal": FOCUS[:-2] + "fn panic_goal(number:u64)->u32 {narrow(number)}\n}\n",
        "conversion_client": """use vstd::prelude::*; verus! {
            #[verifier::infer] pub fn relay(number:u64)->u32 { conversion_provider::narrow(number) }
            fn caller(number:u64)->(result:u32) requires number <= u32::MAX, ensures result == number,
            { relay(number) }
        }""",
    }
    require(sources["decoder_mutation"] != codec + authored + "}\n", "mutation did not match")
    native = pieces["PageNumber"] + "\nimpl PageNumber {\n"
    native += "\n".join(pieces[name] for name in ("to_le_bytes", "from_le_bytes")) + "\n}\n"
    sources["native"] = native + NATIVE
    sources["native_mutation"] = native.replace("((temp >> 20) & 0x000F_FFFF)", "((temp >> 21) & 0x000F_FFFF)") + NATIVE
    for name, source in sources.items():
        (output / (name + ".rs")).write_text(source)
    return {name: dict(sha256=FROZEN_HASHES[name], source=BASE,
                      line=text[:text.index(contents)].count("\n") + 1)
            for name, contents in pieces.items()}


def classify(data, stderr, exit_code):
    result = data["verification-results"]
    if any(fragment in stderr.lower() for fragment in ("panicked at", "resource limit", "timed out")):
        return "resource_or_internal_failure"
    if result.get("encountered-vir-error") or "success" not in result or "error[E" in stderr:
        return "frontend_rejection"
    if exit_code == 0 and result.get("success"):
        return "proved"
    functions = data.get("verus-infer", {}).get("functions", [])
    exact = bool(functions) and all(function.get("required_checks_passed") for function in functions)
    if exact and "postcondition not satisfied" in stderr and "precondition not satisfied" not in stderr:
        return "semantic_goal_failure"
    if exact and "precondition not satisfied" in stderr:
        return "caller_obligation_failure"
    return "proof_blocked"


def export_path(output, name, phase, seeded):
    return output / ((name + "_" + phase if seeded else name) + ".vir")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--application", type=Path, required=True)
    parser.add_argument("--verus", type=Path, required=True)
    parser.add_argument("--solver", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=positive_timeout, default=180)
    args = parser.parse_args()
    args.application = args.application.resolve()
    args.verus = args.verus.resolve()
    args.solver = args.solver.resolve()
    args.output.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix="redb-codec-", dir=args.output.resolve()))
    engine = Path(__file__).resolve().parents[2]
    transfer = dict(scenario="redb", application_commit=APPLICATION, status="blocked",
                    source_fidelity=False, positive_cases=[], semantic_negative_cases=[],
                    stale_export_rejected=False, limitations=["extracted codec only; not whole-database correctness"],
                    trust=["ordinary Verus backend and existing vstd contracts",
                           "Rust core primitive implementation bodies are not formally verified"])
    receipt = dict(transfer=transfer, cases=[], commands=[], output=str(output), host=host_info(),
                   verification_accepted=False)
    try:
        with replay_receipt(output, receipt):
            environment = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.1", VERUS_Z3_PATH=str(args.solver))
            run = LoggedCommands(output, engine, receipt["commands"], timeout=args.timeout)
            require(run("application_pin", ["git", "-C", str(args.application), "rev-parse", "HEAD"], environment) == 0)
            require((output / "application_pin.stdout").read_text().strip() == APPLICATION, "application pin mismatch")
            receipt["extraction"] = extract(args.application, output)
            transfer["source_fidelity"] = True
            receipt["domain"] = dict(region_max="0xfffff", order_max=20, index_max="0xfffff >> order", quantified="all such values")
            receipt["source_fidelity"] = dict(application_executable_body_edits=0,
                frozen_unaided_wrapper_preserved=True, authored_wrapper_additions="none in accepted codec, injectivity, or imported caller; historical assisted fixture and semantic controls retained unchanged",
                public_provider_changes="visibility only; method bodies unchanged",
                manually_assumed_application_summaries=0, input_domain_restrictions_added=0)
            source_paths = subprocess.check_output(["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"], cwd=engine).decode().split("\0")
            receipt["engine_sources"] = {relative: digest(engine / relative) for relative in sorted(set(source_paths))
                                         if relative.startswith("source/") and (engine / relative).is_file()}
            receipt["generated_sources"] = {path.name: digest(path) for path in output.glob("*.rs")}
            require(run("sysroot", ["rustc", "--print", "sysroot"], environment) == 0)
            sysroot = Path((output / "sysroot.stdout").read_text().strip())
            guard = ArtifactGuard(tool_artifacts(args.verus, args.solver, extra=[
                Path(__file__), args.application / BASE, *output.glob("*.rs"),
                *(engine / relative for relative in receipt["engine_sources"]),
                sysroot / "bin/rustc", *sysroot.glob("lib/librustc_driver*.dylib"),
                *sysroot.glob("lib/rustlib/*/lib/libcore-*.rlib"),
                *sysroot.glob("lib/rustlib/*/lib/libstd-*.rlib"),
            ]))
            run.guard = guard
            receipt["artifacts"] = guard.initial
            for label, command in (
                ("verus_version", [str(args.verus), "--version"]),
                ("rustc_version", ["rustc", "-vV"]),
                ("solver_version", [str(args.solver), "-version"]),
            ):
                require(run(label, command, environment) == 0)

            def verify(name, phase, count, expected=None, dependency=None, imported=None, stale_seed=None):
                label = name + "_" + phase
                seeded = expected is False or name in ("decoder_mutation", "false_goal", "malformed_goal", "panic_goal")
                artifact = export_path(output, name, phase, seeded)
                if seeded:
                    artifact.write_bytes(stale_seed if stale_seed is not None else b"seeded stale export")
                environment["VERUS_INFER_CACHE"] = str(output / (name + "-cache"))
                command = [str(args.verus), str(output / (name + ".rs")), "--crate-type", "lib", "--crate-name", name,
                           "--no-cheating", "--output-json", "--compile", "--export", str(artifact),
                           "-o", str(output / ("lib" + name + ".rlib"))]
                if name not in ("decoder_mutation", "false_goal", "malformed_goal"):
                    command.append("--infer-goals")
                if dependency:
                    command += ["--extern", dependency + "=" + str(output / ("lib" + dependency + ".rlib")),
                                "--import", dependency + "=" + str(imported or output / (dependency + ".vir"))]
                code = run(label, command, environment)
                data = json.loads((output / (label + ".stdout")).read_text())
                stderr = (output / (label + ".stderr")).read_text()
                classification = classify(data, stderr, code)
                row = dict(label=label, classification=classification, report=data,
                           report_sha256=digest(output / (label + ".stdout")),
                           stale_export_seeded=seeded, artifact_exists=artifact.exists(),
                           stale_export_kind="prior_verified_interface" if stale_seed is not None else "sentinel" if seeded else None,
                           first_diagnostic=next((line for line in stderr.splitlines() if line.startswith("error")), None))
                receipt["cases"].append(row)
                if expected is not None:
                    validate_run(dict(stdout=json.dumps(data), stderr=stderr, returncode=code), expected,
                                 allow_frontend=phase.startswith("metadata") or name == "malformed_goal")
                    require_publication(data, expected, artifact)
                if classification in ("proved", "semantic_goal_failure", "caller_obligation_failure"):
                    functions = data["verus-infer"]["functions"]
                    require(len(functions) == count, "unexpected inferred function count: " + label)
                    require_exact(functions)
                    if classification == "proved":
                        transfer["positive_cases"].append(label)
                    elif classification == "semantic_goal_failure":
                        transfer["semantic_negative_cases"].append(label)
                if classification != "proved":
                    require(not artifact.exists(), "failed verification retained stale export")
                return row

            for phase in ("cold", "warm"):
                verify("conversion_provider", phase, 2, True)
                verify("conversion_client", phase, 1, True, "conversion_provider")
            provider = (output / "conversion_provider.vir").read_bytes()
            start = provider.rindex(b'{"schema":')
            metadata = json.loads(provider[start:-8])
            receipt["conversion_provider_metadata"] = metadata
            previous_client = (output / "conversion_client.vir").read_bytes()
            require(any("try_from" in str(model) for model in metadata["trusted_models"]))
            require(any("unwrap" in str(model) for model in metadata["trusted_models"]))
            for mutation, expected in (
                ("policy", "incompatible inference schema, safety policy, or producer"),
                ("proof", "incomplete inference proof guarantees"),
                ("summary", "inferred contract fingerprint mismatch"),
                ("model", "stale or missing trusted-model fingerprint"),
            ):
                changed = json.loads(json.dumps(metadata))
                if mutation == "policy":
                    changed["policy"] = "candidate-bound-evidence-v55"
                elif mutation == "proof":
                    changed["soundness"] = False
                elif mutation == "summary":
                    changed["summaries"] = "corrupted"
                else:
                    changed["trusted_models"][0][1] = "stale"
                imported = output / (mutation + ".vir")
                imported.write_bytes(provider[:start] + json.dumps(changed).encode() + b"VRINFER1")
                row = verify("conversion_client", "metadata_" + mutation, 0, False,
                             "conversion_provider", imported, stale_seed=previous_client)
                stderr = (output / (row["label"] + ".stderr")).read_text()
                require(expected in stderr, "unintended import rejection")
                row["rejected_for"] = expected
            panic = verify("panic_goal", "negative", 2, False)
            require(panic["classification"] == "caller_obligation_failure", "unwrap panic requirement was lost")
            for phase in ("cold", "warm"):
                verify("codec", phase, 2, True)
            for phase in ("cold", "warm"):
                verify("codec_authored", phase, 2)
            codec_passed = all(row["classification"] == "proved" for row in receipt["cases"]
                               if row["label"].startswith("codec_") and not row["label"].startswith("codec_authored_"))
            if codec_passed:
                for phase in ("cold", "warm"):
                    verify("codec_complete", phase, 2, True)
                    verify("codec_provider", phase, 2, True)
                    verify("codec_client", phase, 1, True, "codec_provider")
                for name in ("decoder_mutation", "false_goal"):
                    negative = verify(name, "negative", 2, False,
                                      stale_seed=(output / "codec.vir").read_bytes())
                    require(negative["classification"] == "semantic_goal_failure", "frontend rejection is not a semantic counterexample")
                unaided_negative = verify("decoder_mutation_unaided", "negative", 2, False,
                                          stale_seed=(output / "codec.vir").read_bytes())
                require(unaided_negative["classification"] == "proof_blocked",
                        "the unaided mutant must fail its checked auxiliary theorem")
                require_exact(unaided_negative["report"]["verus-infer"]["functions"])
                require(len(unaided_negative["report"]["verus-infer"]["functions"]) == 2)
                require("bitvector assertion not satisfied" in
                        (output / "decoder_mutation_unaided_negative.stderr").read_text())
                receipt["unaided_mutant_rejection"] = "checked auxiliary theorem failure, not counted as authored semantic evidence"
                malformed = verify("malformed_goal", "negative", 0, False,
                                   stale_seed=(output / "codec_authored.vir").read_bytes())
                require(malformed["classification"] == "frontend_rejection")
                transfer["status"] = "proved"
                receipt["verification_accepted"] = True
            else:
                transfer["limitations"].append("formal codec property and faithful mutant are not assessed: see codec diagnostics")
                receipt["current_blocker"] = next(row["first_diagnostic"] for row in receipt["cases"]
                                                   if row["label"].startswith("codec_authored_") and row["classification"] != "proved")
                transfer["blocker"] = receipt["current_blocker"]
            transfer["stale_export_rejected"] = all(not row["artifact_exists"] for row in receipt["cases"] if row["stale_export_seeded"])
            for name in ("native", "native_mutation"):
                require(run(name + "_compile", ["rustc", "--edition=2024", "-O", "-C", "debug-assertions=yes",
                            str(output / (name + ".rs")), "-o", str(output / name)], environment) == 0)
                code = run(name + "_execute", [str(output / name)], environment)
                require((code == 0) == (name == "native"), "unexpected native codec result")
                if name == "native_mutation":
                    require("region roundtrip" in (output / (name + "_execute.stderr")).read_text())
                else:
                    require((output / (name + "_execute.stdout")).read_text().strip() == "native codec cases: 2097467", "native coverage count changed")
            receipt["native"] = dict(cases=2097467, region_cross_product_exhaustive=False,
                                      faithful_mutant_detected=True, formal_proof=False)
            require(digest(args.application / BASE) == SOURCE_HASH, "application changed during replay")
            guard.check()
            receipt["artifacts_unchanged"] = True
            receipt["generated_artifacts"] = {path.name: digest(path) for path in output.iterdir()
                                               if path.is_file() and (path.suffix in (".vir", ".rlib") or path.name in ("native", "native_mutation"))}
    finally:
        print(output)


if __name__ == "__main__":
    main()
