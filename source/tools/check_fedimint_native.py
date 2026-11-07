"""Replay the pinned, byte-preserved Fedimint helpers without source desugaring."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import tempfile

from infer_eval_support import ArtifactGuard, digest, host_info, tool_artifacts
from infer_experiment import (LoggedCommands, positive_timeout, replay_receipt, require,
                             require_exact, require_publication, validate_run)


APPLICATION = "b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236"
LIGHTNING = "gateway/fedimint-lightning/src/lib.rs"
CLIENT = "modules/fedimint-gwv2-client/src/lib.rs"
SOURCE_HASHES = {
    LIGHTNING: "6922ada049ba83f999937dce687763604c7aa3714df138e26b84b115f1081ebc",
    CLIENT: "c7475538eac83bc039195b86efe342b0779115c124978bfba1a406f1bbb0ba21",
}
IMPL_HASH = "018b62da6b4e8166972a169ff4bdea762905ea6e466c2fb7e0f4e457281a7b45"
METHOD_HASHES = {
    "pub fn incoming_circuit": "670a0625249c5522466b2d9b3b9d215f2c1e045eea984b3d2a29c566901e0fd8",
    "pub fn lnv2_blocks_to_claim_deadline": "328c1eada09ead864029497765af0ada22c3fdbfe1c2dfd4bf2973343197a2e2",
}
CONSTANT = "pub const NO_INCOMING_CIRCUIT: (u64, u64) = (0, 0);"
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
FOLLOWUP = """
pub fn deadline_boundary(request: &InterceptPaymentRequest) -> (blocks: u32)
    ensures blocks == 0,
{ request.lnv2_blocks_to_claim_deadline(Some(request.expiry)) }

pub fn deadline_monotonicity(request: &InterceptPaymentRequest, earlier: u32, later: u32)
    -> (budgets: (u32, u32))
    ensures earlier <= later ==> budgets.1 <= budgets.0,
{
    (request.lnv2_blocks_to_claim_deadline(Some(earlier)),
     request.lnv2_blocks_to_claim_deadline(Some(later)))
}
"""
GUARD = "        if self.incoming_circuit().is_some() {\n            return 0;\n        }\n\n"
NATIVE = r"""
mod original;
mod regression;

fn main() {
    let ids = [0, 1, 101, u64::MAX];
    let boundaries = [0, 1, 2, 3, 18, 19, 36, 144, 1000, u32::MAX - 1, u32::MAX];
    let mut checked = 0u64;
    for incoming_chan_id in ids {
        for htlc_id in ids {
            for expiry in boundaries {
                for height in std::iter::once(None).chain(boundaries.into_iter().map(Some)) {
                    let request = original::InterceptPaymentRequest { expiry, incoming_chan_id, htlc_id };
                    let expected = if incoming_chan_id != 0 || htlc_id != 0 { 0 }
                        else { height.map_or(0, |current| (expiry as u64).checked_sub(current as u64).unwrap_or(0) as u32) };
                    assert_eq!(request.lnv2_blocks_to_claim_deadline(height), expected);
                    assert_eq!(request.incoming_circuit(), if incoming_chan_id == 0 && htlc_id == 0 { None } else { Some((incoming_chan_id, htlc_id)) });
                    checked += 1;
                }
            }
        }
    }
    let original = original::InterceptPaymentRequest { expiry: 1000, incoming_chan_id: 101, htlc_id: 0 };
    let regression = regression::InterceptPaymentRequest { expiry: 1000, incoming_chan_id: 101, htlc_id: 0 };
    assert_eq!(original.lnv2_blocks_to_claim_deadline(Some(990)), 0);
    assert_eq!(regression.lnv2_blocks_to_claim_deadline(Some(990)), 10);
    let hold = original::InterceptPaymentRequest { expiry: u32::MAX, incoming_chan_id: 0, htlc_id: 0 };
    assert_eq!(hold.lnv2_blocks_to_claim_deadline(Some(u32::MAX - 2)), 2);
    assert_eq!(hold.lnv2_blocks_to_claim_deadline(Some(u32::MAX)), 0);
    println!("{{\"native_oracle_inputs\":{},\"nonvacuity_witness\":2,\"regression_fixed\":0,\"regression_mutant\":10}}", checked);
}
"""


def text_digest(text):
    return hashlib.sha256(text.encode()).hexdigest()


def item(text, marker):
    start = text.index(marker)
    opening = text.index("{", start)
    depth, end = 1, opening + 1
    while depth:
        require(end < len(text), "unclosed selected source item")
        depth += (text[end] == "{") - (text[end] == "}")
        end += 1
    return text[start:end]


def extract(source):
    constant = re.search(r"pub const NO_INCOMING_CIRCUIT[^;]+;", source).group()
    implementation = item(source, "impl InterceptPaymentRequest {")
    require(constant == CONSTANT and text_digest(implementation) == IMPL_HASH,
            "pinned native constant or implementation changed")
    for marker, expected in METHOD_HASHES.items():
        require(text_digest(item(implementation, marker)) == expected, "pinned native method changed")
    return constant, implementation


def regress(implementation):
    require(implementation.count(GUARD) == 1, "mutation needs exactly the original forward guard")
    return implementation.replace(GUARD, "", 1)


def proof_source(constant, implementation, policy=POLICY):
    annotated = implementation.replace("    pub fn ", "    #[verifier::infer]\n    pub fn ")
    require(annotated.count("#[verifier::infer]") == 2)
    for marker in METHOD_HASHES:
        require(item(annotated, marker) == item(implementation, marker), "inference attribute changed a native method")
    return "use vstd::prelude::*;\nverus! {\n" + constant + "\n" + PROJECTION + annotated + policy + "}\n"


def replay(args, output, receipt):
    application, verifier, solver = args.application.resolve(), args.verus.resolve(), args.solver.resolve()
    engine = Path(__file__).resolve().parents[2]
    environment = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.1", VERUS_Z3_PATH=str(solver), RUST_MIN_STACK="20971520")
    command = LoggedCommands(output, output, receipt["commands"], timeout=args.timeout)

    def invoke(label, argv):
        require(command(label, argv, environment) == 0, "setup/native command failed: " + label)
        return (output / (label + ".stdout")).read_text().strip()

    require(invoke("application_head", ["git", "-C", str(application), "rev-parse", "HEAD"]) == APPLICATION)
    require(invoke("application_clean", ["git", "-C", str(application), "status", "--porcelain"]) == "")
    require({relative: digest(application / relative) for relative in SOURCE_HASHES} == SOURCE_HASHES)
    constant, implementation = extract((application / LIGHTNING).read_text())
    require("const LNV2_CLAIM_DEADLINE_MARGIN: u32 = 2;" in (application / CLIENT).read_text())
    receipt["source_sha256"] = SOURCE_HASHES
    receipt["method_sha256"] = METHOD_HASHES
    receipt["constant_sha256"] = text_digest(constant)
    receipt["implementation_sha256"] = text_digest(implementation)
    receipt["adaptations"] = ["Project three read scalar fields, omit serde and unrelated fields", "Insert infer attributes outside byte-preserved method items", "Wrap extracted items in verus!; no source desugarings"]
    receipt["engine_commit"] = invoke("engine_head", ["git", "-C", str(engine), "rev-parse", "HEAD"])
    source_files = invoke("engine_files", ["git", "-C", str(engine), "ls-files", "--cached", "--others", "--exclude-standard", "source", "rust-toolchain.toml"]).splitlines()
    receipt["engine_source_sha256"] = {relative: digest(engine / relative) for relative in source_files if (engine / relative).is_file()}
    invoke("engine_patch", ["git", "-C", str(engine), "diff", "HEAD", "--", "source"])
    receipt["engine_patch_sha256"] = digest(output / "engine_patch.stdout")
    for relative in SOURCE_HASHES:
        destination = output / "source-snapshot" / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes((application / relative).read_bytes())
    variants = {"original": implementation, "regression": regress(implementation)}
    for name, body in variants.items():
        (output / (name + ".rs")).write_text(constant + "\n" + PROJECTION + body + "\n")
    proofs = {"native": proof_source(constant, implementation),
              "followup": proof_source(constant, implementation, POLICY + FOLLOWUP),
              "regression": proof_source(constant, variants["regression"]),
              "false_goal": proof_source(constant, implementation, POLICY.replace("blocks == 0,", "blocks == 1,", 1)),
              "malformed_goal": proof_source(constant, implementation, POLICY.replace("blocks == 0,", "blocks == true,", 1))}
    for name, source in proofs.items():
        (output / (name + "-proof.rs")).write_text(source)
    (output / "oracle.rs").write_text(NATIVE)
    generated_sources = sorted(output.glob("*.rs"))
    receipt["generated_source_sha256"] = {path.name: digest(path) for path in generated_sources}
    compiler = Path(invoke("native_compiler", ["rustup", "which", "--toolchain", "1.99.1", "rustc"]))
    guard = ArtifactGuard(tool_artifacts(verifier, solver, extra=[compiler, Path(__file__),
        *generated_sources, *(application / relative for relative in SOURCE_HASHES), *(engine / relative for relative in receipt["engine_source_sha256"])]))
    command.guard = guard
    receipt["artifacts_before"] = guard.initial
    invoke("native_build", [str(compiler), str(output / "oracle.rs"), "--edition=2024", "-C", "overflow-checks=yes", "-o", str(output / "oracle")])
    receipt["native_oracle_binary_sha256"] = digest(output / "oracle")
    receipt["native_oracle"] = json.loads(invoke("native_oracle", [str(output / "oracle")]))
    cold_contracts = None
    for name, phase, expected in [("native", "cold", True), ("native", "warm", True),
                                  ("followup", "cold", True), ("regression", "cold", False),
                                  ("false_goal", "cold", False), ("malformed_goal", "frontend", False)]:
        label = name + "_" + phase
        artifact = output / (name + ".vir")
        if not expected:
            artifact.write_bytes((output / "native.vir").read_bytes())
        previous = dict(sha256=digest(artifact), mtime_ns=artifact.stat().st_mtime_ns) if artifact.exists() else None
        environment["VERUS_INFER_CACHE"] = str(output / (name + "-cache"))
        status = command(label, [str(verifier), str(output / (name + "-proof.rs")), "--crate-type=lib", "--edition=2024", "--crate-name", "deadline_" + name,
            "--no-cheating", "--output-json", "--time", "--num-threads", "1", "--export", str(artifact)], environment)
        run = dict(returncode=status, stdout=(output / (label + ".stdout")).read_text(), stderr=(output / (label + ".stderr")).read_text())
        malformed = name == "malformed_goal"
        data = validate_run(run, expected, allow_frontend=malformed)
        require_publication(data, expected, artifact)
        inference = data["verus-infer"]
        if malformed:
            require("blocks == true" in run["stderr"] and "SpecEq<bool>" in run["stderr"])
            classification = "malformed goal; frontend control only"
        else:
            functions = inference["functions"]
            require(len(functions) == 2)
            require({function["function"].split("::")[-1] for function in functions} ==
                    {"incoming_circuit", "lnv2_blocks_to_claim_deadline"})
            require_exact(functions)
            require(all(function["contract"]["requires"] == "true" for function in functions),
                    "native helper requirements must cover the full scalar domain")
            for function in functions:
                metrics = function["metrics"]
                predicted = "miss" if metrics.get("cache_policy") == "regenerated_state" or phase == "cold" else "hit"
                require(metrics["cache"] == predicted, "dishonest cold/warm cache accounting")
            contracts = {function["function"]: function["contract"] for function in functions}
            if phase == "warm":
                require(contracts == cold_contracts and previous["mtime_ns"] != artifact.stat().st_mtime_ns)
            elif name == "native":
                cold_contracts = contracts
            if expected:
                require(all(function["exported"] for function in functions))
                classification = "byte-preserved native helpers exactly inferred; ordinary authored policy proved"
            else:
                require("postcondition not satisfied" in run["stderr"] and "precondition not satisfied" not in run["stderr"])
                require(data["verification-results"].get("errors") == 1)
                require(any("deadline_policy" in function for function in inference["failed_functions"]))
                if name == "regression":
                    require("request.incoming_chan_id != 0 || request.htlc_id != 0" in run["stderr"],
                            "guard deletion must fail the original intercepted-forward policy clause")
                require(all(not function["exported"] for function in functions))
                classification = "semantic authored-policy rejection; inference remains exact"
        receipt["cases"].append(dict(name=name, phase=phase, label=label, classification=classification,
            source_sha256=digest(output / (name + "-proof.rs")), previous_interface=previous,
            observed_interface=dict(sha256=digest(artifact), mtime_ns=artifact.stat().st_mtime_ns) if artifact.exists() else None,
            interface_exists=artifact.exists(), inference=inference, verification=data["verification-results"]))
    require(invoke("application_clean_after", ["git", "-C", str(application), "status", "--porcelain"]) == "")
    guard.check()
    receipt["artifacts_after"] = guard.snapshot()
    receipt["transfer"].update(status="proved", source_fidelity=True,
        positive_cases=["native_cold", "native_warm", "followup_cold"],
        semantic_negative_cases=["regression_cold", "false_goal_cold"], stale_export_rejected=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("application", "verus", "solver", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--timeout", type=positive_timeout, default=120)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix="run-", dir=args.output.resolve()))
    receipt = dict(commands=[], cases=[], host=host_info(), transfer=dict(scenario="fedimint", application_commit=APPLICATION,
        status="blocked", source_fidelity=False, positive_cases=[], semantic_negative_cases=[], stale_export_rejected=False,
        limitations=["Three-field projection, not full gateway/async/database/crypto verification", "Native oracle tests finite inputs; formal policy covers the full scalar domain", "Separate scalar/newtype projection profile is not extended"],
        trust=["Ordinary Verus backend and existing vstd specifications", "Canonical Rust API-to-checked-bridge correspondence is trusted frontend code; no new assumed specification", "Authored deadline policy is not inferred"]))
    try:
        with replay_receipt(output, receipt):
            replay(args, output, receipt)
    finally:
        print(output)


if __name__ == "__main__":
    main()
