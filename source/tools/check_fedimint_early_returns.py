"""Replay exact deadline inference after readonly-record early-return support."""

import argparse
import json
import os
from pathlib import Path
import re
import tempfile

from infer_eval_support import ArtifactGuard, digest, host_info, tool_artifacts
from infer_experiment import (
    LoggedCommands,
    positive_timeout,
    replay_receipt,
    require,
    require_exact,
    require_publication,
    validate_run,
)


ENGINE = "55bd67ad980e90e92c77efe4453a927f097c3386"
APPLICATION = "b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236"
COMPONENT = "08107c00a21cef35aa9e71613af55aaa72312fe0"
EDITION = "2024"
LIGHTNING = "gateway/fedimint-lightning/src/lib.rs"
CLIENT = "modules/fedimint-gwv2-client/src/lib.rs"
PROJECTION = """pub struct InterceptPaymentRequest {
    pub expiry: u32,
    pub incoming_chan_id: u64,
    pub htlc_id: u64,
}
"""
POLICY = """
pub fn deadline_policy(request: &InterceptPaymentRequest, height: Option<u32>) -> (blocks: u32)
    ensures
        (request.incoming_chan_id != 0 || request.htlc_id != 0) ==> blocks == 0,
        match height {
            None => blocks == 0,
            Some(current) => blocks >= 2 ==> request.expiry as int >= current as int + 2,
        },
        (request.incoming_chan_id == 0 && request.htlc_id == 0) ==> match height {
            None => true,
            Some(current) => request.expiry as int >= current as int + 2 ==> blocks >= 2,
        },
{
    request.lnv2_blocks_to_claim_deadline(height)
}
"""
NATIVE = """
mod original;
mod lowered;
mod regression;

fn main() {
    let ids = [0, 1, 101, u64::MAX];
    let heights = [0, 1, 2, 3, 18, 19, 36, 144, 1000, u32::MAX - 1, u32::MAX];
    let mut checked = 0u64;
    for incoming_chan_id in ids {
        for htlc_id in ids {
            for expiry in heights {
                for height in std::iter::once(None).chain(heights.into_iter().map(Some)) {
                    let original = original::InterceptPaymentRequest { expiry, incoming_chan_id, htlc_id };
                    let lowered = lowered::InterceptPaymentRequest { expiry, incoming_chan_id, htlc_id };
                    let actual = original.lnv2_blocks_to_claim_deadline(height);
                    assert_eq!(actual, lowered.lnv2_blocks_to_claim_deadline(height));
                    assert_eq!(original.incoming_circuit(), lowered.incoming_circuit());
                    if incoming_chan_id != 0 || htlc_id != 0 || height.is_none() {
                        assert_eq!(actual, 0);
                    }
                    if actual >= 2 {
                        assert_eq!((incoming_chan_id, htlc_id), (0, 0));
                        assert!(expiry as u64 >= height.expect("positive deadline has a height") as u64 + 2);
                    }
                    checked += 1;
                }
            }
        }
    }
    let original = original::InterceptPaymentRequest { expiry: 1000, incoming_chan_id: 101, htlc_id: 0 };
    let regression = regression::InterceptPaymentRequest { expiry: 1000, incoming_chan_id: 101, htlc_id: 0 };
    assert_eq!(original.lnv2_blocks_to_claim_deadline(Some(990)), 0);
    assert_eq!(regression.lnv2_blocks_to_claim_deadline(Some(990)), 10);
    println!("{{\\\"differential_inputs\\\":{},\\\"regression_witness\\\":{{\\\"incoming_chan_id\\\":101,\\\"htlc_id\\\":0,\\\"expiry\\\":1000,\\\"height\\\":990,\\\"fixed\\\":0,\\\"regression\\\":10}}}}", checked);
}
"""


def item(text, marker):
    start = text.index(marker)
    opening = text.index("{", start)
    depth = 1
    end = opening + 1
    while depth:
        require(end < len(text), "unclosed selected source item")
        depth += (text[end] == "{") - (text[end] == "}")
        end += 1
    return text[start:end]


def lower(original, *, option_predicate=True, tuple_comparison=True):
    old = "(circuit != NO_INCOMING_CIRCUIT).then_some(circuit)"
    new = "if circuit != NO_INCOMING_CIRCUIT { Some(circuit) } else { None }"
    require(original.count(old) == 1)
    result = original.replace(old, new)
    old = "current_block_height.map_or(0, |height| self.expiry.saturating_sub(height))"
    new = "match current_block_height { Some(height) => self.expiry.saturating_sub(height), None => 0 }"
    require(result.count(old) == 1)
    result = result.replace(old, new)
    if option_predicate:
        old = "self.incoming_circuit().is_some()"
        new = "match self.incoming_circuit() { Some(_) => true, None => false }"
        require(result.count(old) == 1)
        result = result.replace(old, new)
    if tuple_comparison:
        old = "circuit != NO_INCOMING_CIRCUIT"
        require(result.count(old) == 1)
        result = result.replace(old, "circuit.0 != 0 || circuit.1 != 0")
    return result


def regress(lowered):
    guard = "        if match self.incoming_circuit() { Some(_) => true, None => false } {\n            return 0;\n        }\n\n"
    require(lowered.count(guard) == 1)
    return lowered.replace(guard, "")


def proof_source(constant, implementation):
    implementation = implementation.replace("    pub fn ", "    #[verifier::infer]\n    pub fn ")
    return "use vstd::prelude::*;\nverus! {\n" + constant + "\n" + PROJECTION + implementation + POLICY + "}\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--application", type=Path, required=True)
    parser.add_argument("--verus", type=Path, required=True)
    parser.add_argument("--solver", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--original-receipt", type=Path, required=True)
    parser.add_argument("--timeout", type=positive_timeout, default=120)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix="run-", dir=args.output.resolve()))
    receipt = {"commands": [], "cases": [], "host": host_info()}
    with replay_receipt(output, receipt):
        replay(args, output, receipt)
    print(output)


def replay(args, output, receipt):
    engine = Path(__file__).resolve().parents[2]
    application = args.application.resolve()
    verifier = args.verus.resolve()
    baseline_files = [args.baseline / "README.md", args.baseline / "manifest.json", args.original_receipt,
                      args.original_receipt.parent / "record_infer-proof.rs",
                      args.original_receipt.parent / "lowered.rs"]
    original_manifest = json.loads((args.baseline / "manifest.json").read_text())
    require(APPLICATION in json.dumps(original_manifest) and ENGINE in json.dumps(original_manifest))
    receipt["frozen_baseline"] = {str(path.resolve()): digest(path) for path in baseline_files}
    environment = dict(os.environ, VERUS_Z3_PATH=str(args.solver.resolve()),
                       RUSTUP_TOOLCHAIN="1.99.1", RUST_MIN_STACK="20971520")
    command = LoggedCommands(output, output, receipt["commands"], timeout=args.timeout)

    def git(label, checkout, *arguments):
        require(command(label, ["git", "-C", str(checkout), *arguments], environment) == 0)
        return (output / (label + ".stdout")).read_text().strip()

    require(git("engine_head", engine, "rev-parse", "HEAD") == ENGINE)
    tracked_diff = git("engine_tracked_diff", engine, "diff", "HEAD", "--")
    require("source/vir/src/infer/records.rs" in tracked_diff)
    receipt["engine_source"] = dict(base=ENGINE, root=str(engine), dirty=True,
                                  tracked_diff_sha256=digest(output / "engine_tracked_diff.stdout"),
                                  tracked_diff=str(output / "engine_tracked_diff.stdout"))
    engine_files = git("engine_files", engine, "ls-files", "source", "rust-toolchain.toml").splitlines()
    receipt["engine_source"]["tracked_source_sha256"] = {
        relative: digest(engine / relative) for relative in engine_files if (engine / relative).is_file()
    }
    require(git("application_head", application, "rev-parse", "HEAD") == APPLICATION)
    require(git("application_status_before", application, "status", "--porcelain") == "")
    historical_parent = git("historical_parent", application, "rev-parse", COMPONENT + "^")
    historical = git("historical_source", application, "show",
                     historical_parent + ":gateway/fedimint-gateway-server/src/lib.rs")
    require("htlc_request.expiry.saturating_sub(info.block_height)" in historical)
    source = (application / LIGHTNING).read_text()
    package = (application / "Cargo.toml").read_text().split("[workspace.package]", 1)[1].split("\n[", 1)[0]
    require(re.search(r'edition\s*=\s*"([^"]+)"', package).group(1) == EDITION)
    constant = re.search(r"pub const NO_INCOMING_CIRCUIT[^;]+;", source).group()
    require(constant == "pub const NO_INCOMING_CIRCUIT: (u64, u64) = (0, 0);")
    original = item(source, "impl InterceptPaymentRequest {")
    margin = re.search(r"const LNV2_CLAIM_DEADLINE_MARGIN: u32 = (\d+);",
                       (application / CLIENT).read_text()).group(1)
    require(margin == "2", "authored threshold must track the pinned production guard")
    require(set(re.findall(r"self\.(\w+)", original)) ==
            {"expiry", "incoming_chan_id", "htlc_id", "incoming_circuit"})
    source_files = [LIGHTNING, CLIENT, "Cargo.toml", "gateway/fedimint-lightning/Cargo.toml",
                    "gateway/fedimint-gateway-server/src/lib.rs",
                    "gateway/fedimint-lightning/src/tests.rs", "AGENTS.md", "SECURITY.md"]
    source_files += [".agents/skills/" + skill + "/SKILL.md" for skill in
                     ("fedimint-codebase", "fedimint-development", "fedimint-coding-style")]
    snapshot = output / "source-snapshot"
    for relative in source_files:
        destination = snapshot / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes((application / relative).read_bytes())
    receipt.update(engine=ENGINE, application=APPLICATION, edition=EDITION, historical_component=COMPONENT,
                   historical_parent=historical_parent, production_margin=int(margin),
                   source_sha256={relative: digest(application / relative) for relative in source_files},
                   classification="source-connected-extracted-body-with-explicit-desugaring",
                   adaptations=["Project request to the three read fields; drop four unread fields and serde derives",
                                "Keep original methods verbatim for frontend/native controls",
                                "Replace bool::then_some with if/else; Option::map_or and Option::is_some with matches",
                                "Expand inequality with the pinned (0, 0) tuple constant to component comparisons",
                                "Regression deletes only the incoming-circuit early-return guard"],
                   authored="Three safety/coverage postconditions on the ordinary deadline_policy wrapper; both complete methods inferred; no proof blocks or supplied views",
                   trust="Ordinary Verus backend and bundled vstd; no added assumed specifications; production callsite/DB/LND not verified")
    variants = {"original": original, "stdlib_lowered": lower(original, option_predicate=False, tuple_comparison=False),
                "option_lowered": lower(original, tuple_comparison=False),
                "record_infer": lower(original),
                "lowered": lower(original), "regression": regress(lower(original)),
                "malformed_goal": lower(original), "false_goal": lower(original)}
    for name, implementation in variants.items():
        (output / (name + ".rs")).write_text(constant + "\n" + PROJECTION + implementation + "\n")
        checked = proof_source(constant, implementation)
        if name == "malformed_goal":
            checked = checked.replace("blocks == 0,", "blocks == true,", 1)
        if name == "false_goal":
            checked = checked.replace("blocks == 0,", "blocks == 1,", 1)
        (output / (name + "-proof.rs")).write_text(checked)
    (output / "native.rs").write_text(NATIVE)
    require((output / "record_infer-proof.rs").read_bytes() ==
            (args.original_receipt.parent / "record_infer-proof.rs").read_bytes(),
            "the fully inferred application candidate must equal the frozen candidate byte for byte")
    require((output / "lowered.rs").read_bytes() ==
            (args.original_receipt.parent / "lowered.rs").read_bytes())
    receipt["unchanged_desugared_candidate_sha256"] = digest(output / "record_infer-proof.rs")
    require(command("native_compiler_path", ["rustup", "which", "--toolchain", "1.99.1", "rustc"], environment) == 0)
    native_compiler = Path((output / "native_compiler_path.stdout").read_text().strip())
    guard = ArtifactGuard(tool_artifacts(verifier, args.solver, extra=[Path(__file__).resolve(),
                          *baseline_files,
                          *(engine / relative for relative in engine_files if (engine / relative).is_file()),
                          engine / "source/vir/src/infer/records/tests.rs",
                          *(application / relative for relative in source_files),
                          *(output / (name + "-proof.rs") for name in variants),
                          *(output / (name + ".rs") for name in variants), output / "native.rs",
                          native_compiler, *native_compiler.parent.parent.glob("lib/librustc_driver*.dylib")]))
    receipt["artifacts_before"] = guard.initial
    command.guard = guard
    require(command("verifier_version", [str(verifier), "--version"], environment) == 0)
    require("55bd67a" in (output / "verifier_version.stdout").read_text())
    require("dirty" in (output / "verifier_version.stdout").read_text())
    require(command("solver_version", [str(args.solver), "-version"], environment) == 0)
    require(command("native_compiler_version", [str(native_compiler), "-vV"], environment) == 0)
    require(command("native_compile", [str(native_compiler), "--edition=" + EDITION, str(output / "native.rs"),
                                       "-o", str(output / "native")], environment) == 0)
    require(command("native_run", [str(output / "native")], environment) == 0)
    receipt["native"] = json.loads((output / "native_run.stdout").read_text())
    cold_contracts = None
    for name, phase, expected in [("original", "frontend", False), ("stdlib_lowered", "unsupported", False),
                                  ("option_lowered", "unsupported", False), ("record_infer", "cold", True),
                                  ("lowered", "cold", True),
                                  ("lowered", "warm", True), ("regression", "cold", False),
                                  ("false_goal", "cold", False), ("malformed_goal", "frontend", False)]:
        artifact = output / (name + ".vir")
        cache = output / (name + "-cache")
        cache.mkdir(exist_ok=True)
        environment["VERUS_INFER_CACHE"] = str(cache)
        label = name + "_" + phase
        if name in ("regression", "false_goal", "malformed_goal"):
            artifact.write_bytes((output / "lowered.vir").read_bytes())
        previous_interface = dict(sha256=digest(artifact), mtime_ns=artifact.stat().st_mtime_ns) if artifact.exists() else None
        arguments = [str(verifier), str(output / (name + "-proof.rs")), "--crate-type=lib",
                     "--edition=" + EDITION, "--crate-name", "deadline_" + name, "--no-cheating",
                     "--output-json", "--time", "--num-threads", "1", "--export", str(artifact)]
        status = command(label, arguments, environment)
        run = {"returncode": status, "stdout": (output / (label + ".stdout")).read_text(),
               "stderr": (output / (label + ".stderr")).read_text()}
        blocked = name in ("original", "stdlib_lowered", "option_lowered", "malformed_goal")
        data = validate_run(run, expected, allow_frontend=blocked)
        require_publication(data, expected, artifact)
        observed_interface = dict(sha256=digest(artifact), mtime_ns=artifact.stat().st_mtime_ns) if artifact.exists() else None
        if expected and previous_interface:
            require(observed_interface["mtime_ns"] != previous_interface["mtime_ns"], "interface was not freshly written")
        inference = data["verus-infer"]
        if blocked:
            result = data["verification-results"]
            require(result.get("encountered-vir-error") is True or result.get("encountered-error") is True)
            if name == "original":
                require("then_some" in run["stderr"] and "map_or" in run["stderr"])
                require("is not supported" in run["stderr"])
                classification = "unsupported/frontend; not bug detection"
            elif name == "stdlib_lowered":
                require("Verus Infer unsupported" in run["stderr"] and "is_some" in run["stderr"])
                classification = "unsupported/infer Option predicate; not bug detection"
            elif name == "option_lowered":
                require("Verus Infer unsupported" in run["stderr"] and "NO_INCOMING_CIRCUIT" in run["stderr"])
                classification = "unsupported/infer tuple constant; not bug detection"
            else:
                require("SpecEq<bool>" in run["stderr"] and "blocks == true" in run["stderr"])
                classification = "malformed authored goal rejected; not semantic detection"
        else:
            functions = inference["functions"]
            require(len(functions) == 2, "both complete readonly methods must infer exactly")
            require_exact(functions)
            for function in functions:
                metrics = function["metrics"]
                expected_cache = "miss" if metrics.get("cache_policy") == "regenerated_state" else (
                    "hit" if phase == "warm" else "miss")
                require(metrics["cache"] == expected_cache, "unexpected candidate-cache behavior")
            contracts = {function["function"]: function["contract"] for function in functions}
            if phase == "warm":
                require(contracts == cold_contracts, "warm predicates changed")
            if name in ("lowered", "record_infer"):
                if name == "lowered":
                    cold_contracts = contracts
                require(inference["status"] == "verified")
                require(all(function["exported"] for function in functions))
                scope = inference["verification_scope"]
                require(scope["complete_selected_scope_verified"] is True and scope["checked_exec_bodies"] == 3)
                require(scope["checked_proof_bodies"] == 0)
                require(all("saturating_sub" in model["function"] for model in inference["trusted_models"]),
                        "only the existing canonical saturating-subtraction dependency is expected")
                methods = [function for function in scope["native_functions"] if function["body_status"] == "checked_exec_body"]
                require(sum(function["inference_candidate"] for function in methods) == 2)
                require(sum(function["ordinary_contract"] for function in methods) == 1)
                classification = "proved extracted/refactored body, not whole gateway"
            else:
                require(inference["status"] == "caller_proof_failed")
                require("postcondition not satisfied" in run["stderr"])
                require("precondition not satisfied" not in run["stderr"])
                require(data["verification-results"].get("errors") == 1)
                if name == "regression":
                    require("request.incoming_chan_id != 0 || request.htlc_id != 0" in run["stderr"],
                            "the faithful mutant must fail the intercepted-forward policy clause")
                require(any("deadline_policy" in function for function in inference["failed_functions"]))
                require(all(not function["exported"] for function in functions))
                classification = "semantic authored-policy rejection; both inferred contracts exactly proved"
        receipt["cases"].append(dict(name=name, phase=phase, classification=classification,
                                     previous_interface=previous_interface, observed_interface=observed_interface,
                                     interface_exists=artifact.exists(), inference=inference,
                                     verification=data["verification-results"]))
    require(git("application_status_after", application, "status", "--porcelain") == "")
    guard.check()
    receipt["artifacts_after"] = guard.snapshot()


if __name__ == "__main__":
    main()
