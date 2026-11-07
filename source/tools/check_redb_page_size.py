"""Reconstruct and prove the unchanged pinned redb page-size method."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import tempfile

from infer_eval_support import ArtifactGuard, digest, host_info, tool_artifacts
from infer_experiment import (
    LoggedCommands, positive_timeout, replay_receipt, require, require_exact,
    require_publication, snapshot_inputs, validate_run,
)


APPLICATION = "c82ab865bb362bc4af067d332adc44978ff64813"
BASE = "src/tree_store/page_store/base.rs"
SOURCE_HASH = "ce2ae1188b5bad2babe2f7b7df19d82adc2b8baff6b41b957e978437686938c4"
ITEM_HASHES = {
    "PageNumber": "2f35314985c2cba913372168d6fc73f5633b0de1bfc720f73fc5418460b9d867",
    "page_size_bytes": "2a0e31afa33b709b29c216b83b73ad5f69eb5501b8a3330a819996581573b5b8",
}
BOUND = """
proof fn page_size_bound(page_order:u8, page_size:u32)
    requires page_order <= 20,
    ensures
        (1u64 << page_order) as int * page_size as int <= 4503599626321920,
        (1u64 << page_order) as int * page_size as int <= 1048576 * page_size as int,
{
    assert((1u64 << page_order) <= 1048576u64) by(bit_vector)
        requires page_order <= 20;
    assert((1u64 << page_order) as int * page_size as int <= 4503599626321920) by(nonlinear_arith)
        requires (1u64 << page_order) <= 1048576u64, page_size <= 4294967295u32;
    assert((1u64 << page_order) as int * page_size as int <= 1048576 * page_size as int) by(nonlinear_arith)
        requires (1u64 << page_order) <= 1048576u64;
}
"""
CALLERS = """
fn canonical(region:u32,page_index:u32,page_order:u8,page_size:u32)->(result:u64)
    requires page_order <= 20,
    ensures result == (1u64 << page_order) as int * page_size as int,
        result <= 4503599626321920,
        result <= 1048576 * page_size as int,
{
    proof { page_size_bound(page_order,page_size); }
    PageNumber{region,page_index,page_order}.page_size_bytes(page_size)
}
fn full_domain(region:u32,page_index:u32,page_order:u8,page_size:u32)->(result:u64)
    requires page_order < 64,
        (1u64 << page_order) as int * page_size as int <= u64::MAX,
    ensures result == (1u64 << page_order) as int * page_size as int,
{PageNumber{region,page_index,page_order}.page_size_bytes(page_size)}
fn noncanonical_zero()->(result:u64) ensures result==0,
{PageNumber{region:0,page_index:0,page_order:63}.page_size_bytes(0)}
fn noncanonical_one()->(result:u64) ensures result==9223372036854775808,
{
    proof {assert((1u64 << 63) == 9223372036854775808u64) by(bit_vector);}
    PageNumber{region:0,page_index:0,page_order:63}.page_size_bytes(1)
}
"""
NATIVE = """
fn check(page_order:u8,page_size:u32) {
    let page=PageNumber{region:u32::MAX,page_index:u32::MAX,page_order};
    let expected=(1u128 << page_order) * u128::from(page_size);
    assert_eq!(u128::from(page.page_size_bytes(page_size)),expected);
    assert!(expected<=4503599626321920);
    assert!(expected<=1048576 * u128::from(page_size));
}
fn main() {
    let mut count=0u64;
    for page_order in 0u8..=20 {
        for page_size in 0u32..=65535 {check(page_order,page_size);count+=1;}
        for page_size in [0,1,2,4096,65536,0x7fff_ffff,0x8000_0000,u32::MAX-1,u32::MAX] {
            check(page_order,page_size);count+=1;
        }
        let mut state=0x9e37_79b9u32;
        for _iteration in 0..8192 {
            state=state.wrapping_mul(1664525).wrapping_add(1013904223);
            check(page_order,state);count+=1;
        }
    }
    std::panic::set_hook(Box::new(|_information|{}));
    let mut invalid_shifts=0u32;
    for page_order in 64u8..=255 {
        let page=PageNumber{region:0,page_index:0,page_order};
        assert!(std::panic::catch_unwind(||page.page_size_bytes(std::hint::black_box(0))).is_err());
        invalid_shifts+=1;
    }
    let page=PageNumber{region:0,page_index:0,page_order:63};
    assert_eq!(page.page_size_bytes(0),0);
    assert_eq!(page.page_size_bytes(1),1u64<<63);
    assert!(std::panic::catch_unwind(||page.page_size_bytes(std::hint::black_box(2))).is_err());
    println!("{{\\\"canonical_cases\\\":{count},\\\"eager_shift_panics\\\":{invalid_shifts},\\\"overflow_panics\\\":1}}");
}
"""
INPUTS = [
    "source/tools/check_redb_page_size.py", "source/tools/infer_eval_support.py",
    "source/tools/infer_experiment.py", "source/vir/src/infer/records.rs",
    "source/vir/src/infer/records/shape.rs", "source/vir/src/infer/bitwise.rs",
    "source/vir/src/infer/registry.rs", "source/rust_verify/src/import_export.rs",
    "source/vir/src/infer/goals.rs", "source/vir/src/infer/bitvector_hints.rs",
    "source/vir/src/infer/packing_goals.rs",
]


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
    path = application / BASE
    require(digest(path) == SOURCE_HASH, "pinned application source changed")
    text = path.read_text()
    pieces = {
        "PageNumber": item(text, "pub(crate) struct PageNumber"),
        "page_size_bytes": item(text, "    pub(crate) fn page_size_bytes("),
    }
    provenance = {}
    for name, contents in pieces.items():
        target = output / (name + ".original.rs")
        target.write_text(contents + "\n")
        require(digest(target) == ITEM_HASHES[name], "pinned executable extraction changed: " + name)
        provenance[name] = dict(sha256=digest(target), source=BASE,
                                line=text[:text.index(contents)].count("\n") + 1)
    (output / "base.original.rs").write_bytes(path.read_bytes())
    executable = pieces["PageNumber"] + "\nimpl PageNumber {\n" + pieces["page_size_bytes"] + "\n}\n"
    inferred = executable.replace(pieces["page_size_bytes"], "#[verifier::infer]\n" + pieces["page_size_bytes"])
    header = "use vstd::prelude::*;\nverus! {\n"
    unaided_callers = CALLERS.replace("    proof { page_size_bound(page_order,page_size); }\n", "")
    unaided_callers = unaided_callers.replace(
        "    proof {assert((1u64 << 63) == 9223372036854775808u64) by(bit_vector);}\n", "")
    require("proof {" not in unaided_callers and "page_size_bound" not in unaided_callers,
            "unaided page-size caller retains handwritten proof glue")
    historical = header + inferred + BOUND + CALLERS + "}\n"
    positive = header + inferred + unaided_callers + "}\n"
    client_callers = unaided_callers.replace("PageNumber{", "page_size_provider::PageNumber{")
    client_callers = client_callers.replace(
        "page_size_provider::PageNumber{region,page_index,page_order}.page_size_bytes(page_size)",
        "relay(&page_size_provider::PageNumber{region,page_index,page_order},page_size)",
    )
    sources = {
        "page_size": positive,
        "page_size_authored": historical,
        "page_size_provider": positive.replace("pub(crate)", "pub"),
        "page_size_client": header
            + "#[verifier::infer] pub fn relay(page:&page_size_provider::PageNumber,page_size:u32)->u64 {page.page_size_bytes(page_size)}\n"
            + client_callers + "}\n",
        "false_property": historical.replace(
            "ensures result == (1u64 << page_order) as int * page_size as int,",
            "ensures result != (1u64 << page_order) as int * page_size as int,", 1),
        "native": executable + NATIVE,
        "native_mutation": executable.replace("let pages = 1u64", "let pages = 2u64") + NATIVE,
    }
    for name, order, size in (("shift64_zero", 64, 0), ("shift255_zero", 255, 0), ("multiply_overflow", 63, 2)):
        witness = ("proof {assert((1u64 << 63) == 9223372036854775808u64) by(bit_vector);}"
                   if name == "multiply_overflow" else "")
        sources[name] = header + inferred + (
            "fn rejected()->u64 {" + witness + "PageNumber{region:0,page_index:0,page_order:"
            + str(order) + "}.page_size_bytes(" + str(size) + ")}\n}\n")
    sources["shift_all_invalid_zero"] = header + inferred + """
fn rejected(page_order:u8)->u64 requires page_order>=64,
{PageNumber{region:0,page_index:0,page_order}.page_size_bytes(0)}
}
"""
    for name, source in sources.items():
        (output / (name + ".rs")).write_text(source)
    return provenance


def classify(data, stderr, exit_code):
    result = data["verification-results"]
    if any(fragment in stderr.lower() for fragment in ("panicked at", "resource limit", "timed out")):
        return "resource_or_internal_failure"
    if result.get("encountered-vir-error") or "success" not in result or "error[E" in stderr:
        return "frontend_rejection"
    if exit_code == 0 and result.get("success"):
        return "proved"
    functions = data.get("verus-infer", {}).get("functions", [])
    if not functions or not all(function.get("required_checks_passed") for function in functions):
        return "proof_blocked"
    if "precondition not satisfied" in stderr:
        return "caller_obligation_failure"
    if "postcondition not satisfied" in stderr:
        return "semantic_goal_failure"
    return "proof_blocked"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ("application", "verus", "solver", "output"):
        parser.add_argument("--" + option, type=Path, required=True)
    parser.add_argument("--timeout", type=positive_timeout, default=180)
    args = parser.parse_args()
    args.application = args.application.resolve()
    args.verus = args.verus.resolve()
    args.solver = args.solver.resolve()
    args.output.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix="redb-page-size-", dir=args.output.resolve()))
    engine = Path(__file__).resolve().parents[2]
    transfer = dict(
        scenario="redb_page_size", application_commit=APPLICATION, status="blocked",
        source_fidelity=False, positive_cases=[], semantic_negative_cases=[], safety_negative_cases=[], stale_export_rejected=False,
        limitations=["extracted page_size_bytes only; no address_range, debug_assert, mutable effects, or database theorem",
                     "native oracle is finite support, not the formal all-u32 domain",
                     "import provider changes pub(crate) to pub solely for cross-crate visibility",
                     "warm invocations regenerate readonly-record candidates; they do not claim candidate-cache hits"],
        trust=["ordinary Verus verifier, SMT backend, and existing primitive conversion contracts",
               "generated conditional shift/product theorems checked unconditionally by the existing bit-vector backend",
               "historical semantic and safety controls retain their original checked ghost proofs",
               "no application axioms, trusted specs, assumed order invariant, or suppressed panic obligation"],
    )
    receipt = dict(transfer=transfer, cases=[], commands=[], output=str(output), host=host_info(),
                   verification_accepted=False)
    try:
        with replay_receipt(output, receipt):
            environment = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.1", VERUS_Z3_PATH=str(args.solver))
            run = LoggedCommands(output, engine, receipt["commands"], timeout=args.timeout)
            require(run("application_pin", ["git", "-C", str(args.application), "rev-parse", "HEAD"], environment) == 0)
            require((output / "application_pin.stdout").read_text().strip() == APPLICATION, "application pin mismatch")
            receipt["extraction"] = extract(args.application, output)
            receipt["proof_assistance"] = dict(
                flag="--infer-goals", positive_handwritten_proof_blocks=0,
                original_domain="all u32 page sizes, arbitrary region/index, canonical order <= 20",
                historical_semantic_and_safety_controls_preserved=True,
                noncanonical_valid_cases="order 63 with sizes 0 and 1")
            transfer["source_fidelity"] = True
            receipt.update(snapshot_inputs(engine, output / "inputs", INPUTS))
            guard = ArtifactGuard(tool_artifacts(args.verus, args.solver,
                [engine / relative for relative in INPUTS] + [args.application / BASE]))
            run.guard = guard
            receipt["artifacts"] = guard.initial
            for label, command in (
                ("verus_version", [str(args.verus), "--version"]),
                ("rustc_version", ["rustc", "-vV"]),
                ("solver_version", [str(args.solver), "-version"]),
            ):
                require(run(label, command, environment) == 0)

            def verify(name, phase, expected, classification="proved", dependency=None, stale_seed=None):
                label = name + "_" + phase
                artifact = output / (label + ".vir")
                if stale_seed is not None:
                    artifact.write_bytes(stale_seed)
                environment["VERUS_INFER_CACHE"] = str(output / (name + "-cache"))
                command = [str(args.verus), str(output / (name + ".rs")), "--crate-type", "lib", "--crate-name", name,
                           "--no-cheating", "--output-json", "--compile", "--export", str(artifact),
                           "-o", str(output / ("lib" + name + ".rlib"))]
                if name in ("page_size", "page_size_provider", "page_size_client"):
                    command.append("--infer-goals")
                if dependency:
                    command += ["--extern", dependency + "=" + str(output / ("lib" + dependency + ".rlib")),
                                "--import", dependency + "=" + str(output / (dependency + "_cold.vir"))]
                code = run(label, command, environment)
                data = json.loads((output / (label + ".stdout")).read_text())
                stderr = (output / (label + ".stderr")).read_text()
                observed = classify(data, stderr, code)
                functions = data.get("verus-infer", {}).get("functions", [])
                receipt["cases"].append(dict(label=label, classification=observed, report=data,
                    report_sha256=digest(output / (label + ".stdout")), artifact=str(artifact),
                    artifact_exists=artifact.exists(), stale_export_seeded=stale_seed is not None,
                    stale_seed_sha256=None if stale_seed is None else hashlib.sha256(stale_seed).hexdigest()))
                validate_run(dict(stdout=json.dumps(data), stderr=stderr, returncode=code), expected)
                require_publication(data, expected, artifact)
                require(len(functions) == 1, "expected exactly one inferred function: " + label)
                require_exact(functions)
                require(observed == classification, "unexpected classification: " + label + ": " + observed)
                if expected:
                    transfer["positive_cases"].append(label)
                elif classification == "caller_obligation_failure":
                    transfer["safety_negative_cases"].append(label)
                else:
                    transfer["semantic_negative_cases"].append(label)
                return artifact

            positive_artifacts = [verify("page_size", phase, True) for phase in ("cold", "warm")]
            positive_artifacts.append(verify("page_size_authored", "reference", True))
            positive_artifacts.append(verify("page_size_provider", "cold", True))
            positive_artifacts.extend(verify("page_size_client", phase, True, dependency="page_size_provider")
                                      for phase in ("cold", "warm"))
            stale_seed = positive_artifacts[0].read_bytes()
            for name in ("shift64_zero", "shift255_zero", "shift_all_invalid_zero", "multiply_overflow"):
                verify(name, "negative", False, "caller_obligation_failure", stale_seed=stale_seed)
            verify("false_property", "negative", False, "semantic_goal_failure", stale_seed=stale_seed)
            require(all(path.is_file() and path.stat().st_size > 0 for path in positive_artifacts),
                    "negative controls removed a positive export")
            transfer["stale_export_rejected"] = True
            receipt["positive_artifacts"] = {path.name: digest(path) for path in positive_artifacts}
            for name in ("native", "native_mutation"):
                binary = output / name
                command = ["rustc", "--edition=2021", "-O", "-C", "overflow-checks=yes",
                           str(output / (name + ".rs")), "-o", str(binary)]
                require(run(name + "_compile", command, environment) == 0)
                code = run(name + "_run", [str(binary)], environment)
                require((code == 0) == (name == "native"), "native oracle or mutation control failed")
            receipt["native"] = json.loads((output / "native_run.stdout").read_text())
            receipt["native"].update(formal_proof=False, faithful_mutant_detected=True)
            receipt["required_proofs_proved"] = 3 * len(receipt["cases"])
            receipt["fixture_hashes"] = {path.name: digest(path) for path in sorted(output.glob("*.rs"))}
            guard.check()
            receipt["verification_accepted"] = True
            transfer["status"] = "proved"
    except BaseException as error:
        transfer["status"] = "blocked"
        transfer["blocker"] = str(error) or type(error).__name__
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
        raise
    finally:
        print(output)


if __name__ == "__main__":
    main()
