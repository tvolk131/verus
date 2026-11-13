"""Source-connected, all-u64 fee encoding proofs; no new external specifications."""

import argparse
import json
import os
from pathlib import Path
import re
import tempfile

from check_fedimint_full_request import APPLICATION, caller_evidence, replace_metadata
from check_fedimint_native import item, text_digest
from infer_transfer_identity import CACHE_VERSION, INTERFACE_POLICY, INTERFACE_SCHEMA
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import (LoggedCommands, positive_timeout, replay_receipt,
    require, require_exact, require_publication, validate_run)
from infer_transfer_receipts import metadata


CONTRACTS = "modules/fedimint-lnv2-common/src/contracts.rs"
FEES = "modules/fedimint-lnv2-common/src/gateway_api.rs"
AMOUNT = "fedimint-core/src/amount.rs"
RECURRING = "fedimint-recurringdv2/src/main.rs"
RECEIVE = "modules/fedimint-lnv2-client/src/receive_sm.rs"
SOURCE_HASHES = {
    CONTRACTS: "315992d153692b23b128e39dd9ad6b1f9a31dd9c52508250c2bd487f482a125a",
    FEES: "a3eda308673486e831abbf74c73adc6043fe28a9389e74773f804e70c0f551b6",
    AMOUNT: "9b1095eb5ed2f2500887bc7875d28e4cf46177a6d860680a23e08bef7b253dd1",
    RECURRING: "1d6fc58e0ff4e5da26791871e1ffd27b1fbfe16019084c2d41d5ec8b8009c88b",
    RECEIVE: "b84f0ad0d79c2a0ba5dfd3b59a69b12539393010291b7842f8839cf01f6a5840",
}
ITEM_HASHES = {
    "pub fn fee_encoded_expiration(": "02a83cc3787edf3d787752a8695542f918f924d6e87c77ba0d4694b813f690ca",
    "pub fn fee_from_expiration(": "731783fd43cfceef9c8323b6020504b302109256041e962daf396c91d706373c",
}
GOALS = """
pub fn fee_roundtrip(fee: u64) -> (recovered: u64)
    ensures recovered == fee,
{ fee_from_expiration(fee_encoded_expiration(fee)) }

pub fn expiration_roundtrip(expiration: u64) -> (recovered: u64)
    ensures recovered == expiration,
{ fee_encoded_expiration(fee_from_expiration(expiration)) }

pub fn expiration_order(left: u64, right: u64) -> (encoded: (u64, u64))
    ensures (left <= right) == (encoded.0 >= encoded.1),
        (left == right) == (encoded.0 == encoded.1),
{ (fee_encoded_expiration(left), fee_encoded_expiration(right)) }

pub fn future_expiration(fee: u64, now: u64) -> (expiration: u64)
    ensures (expiration > now) == ((fee as int) + (now as int) < u64::MAX),
{ fee_encoded_expiration(fee) }

pub fn encoding_boundaries() -> (encoded: (u64, u64))
    ensures encoded.0 == u64::MAX, encoded.1 == 0,
{ (fee_encoded_expiration(0), fee_encoded_expiration(u64::MAX)) }
"""
ROUNDTRIP = GOALS[:GOALS.index("pub fn expiration_roundtrip")]
EXPECTED_CASES = {
    "provider_cold": "positive", "provider_warm": "positive",
    "client_cold": "positive", "client_warm": "positive", "payment_fee_cold": "positive",
    "unannotated_cold": "before", "mutant_cold": "semantic", "false_future_cold": "semantic",
    "overflow_cold": "safety", "panic_cold": "safety", "malformed_cold": "frontend",
    "budget_cold": "budget",
    **{"client_metadata_" + fault: "import" for fault in
       ("schema", "policy", "exactness", "summaries", "dependencies", "trusted_models", "missing")},
}


def check_pin(head, status, hashes):
    require(head == APPLICATION, "application HEAD differs from pin")
    require(status == "", "application checkout is dirty")
    require(hashes == SOURCE_HASHES, "complete application source input changed")


def extract_helpers(source):
    helpers = []
    for marker, expected in ITEM_HASHES.items():
        selected = item(source, marker)
        require(text_digest(selected) == expected, "complete original helper item changed")
        helpers.append(selected)
    return "\n".join(helpers)


def proof_source(helpers, goals=GOALS, inferred=True):
    if inferred:
        helpers = helpers.replace("pub fn ", "#[verifier::infer]\npub fn ")
    return "use vstd::prelude::*;\nverus! {\n" + helpers + "\n" + goals + "}\n"


def mutant(helpers):
    original = "u64::MAX - expiration"
    require(helpers.count(original) == 1, "decoder mutant must target exactly one expression")
    return helpers.replace(original,
        "u64::MAX.saturating_sub(expiration).saturating_sub(1)", 1)


def fee_probe(application):
    amount, fees = ((application / relative).read_text() for relative in (AMOUNT, FEES))
    return ("use vstd::prelude::*;\nverus! {\n" + item(amount, "pub struct Amount {")
        + "\n" + item(fees, "pub struct PaymentFee {")
        + "\nimpl PaymentFee {\n#[verifier::infer]\n" + item(fees, "fn absolute_fee(")
        + "\n}\n}\n")


def check_call_paths(recurring, receive):
    require("fee_encoded_expiration(routing_info.receive_fee.fee(amount).msats)" in recurring,
        "real encoding call path changed")
    require("Amount::from_msats(fee_from_expiration(" in receive,
        "real decoding call path changed")


def compiler_artifacts(compiler):
    libraries = compiler.resolve().parent.parent / "lib"
    paths = sorted(path for path in libraries.rglob("*") if path.is_file()
        and (path.suffix in (".dylib", ".so", ".dll", ".rlib") or ".so." in path.name))
    require(any("rustc_driver" in path.name for path in paths)
        and any(path.name.startswith("libstd-") and path.suffix == ".rlib" for path in paths),
        "Rust compiler runtime or standard libraries missing")
    return [compiler, *paths]


def setup_output(command, label, argv, environment, output):
    code = command(label, list(map(str, argv)), environment)
    require(type(code) is int and code == 0, "setup failed: " + label)
    return (output / (label + ".stdout")).read_text().strip()


def check_provider_phase(data, phase):
    require(phase in ("cold", "warm"), "unknown provider phase")
    functions = data["verus-infer"]["functions"]
    require(len(functions) == 2 and all(function["metrics"]["cache"]
        == ("miss" if phase == "cold" else "hit") for function in functions),
        "unexpected scalar cache behavior")
    require(all(type(function["metrics"].get("solver_micros")) is int
        and function["metrics"]["solver_micros"] > 0 for function in functions),
        "fresh required-check solver work missing")


def check_provider_contracts(cold, warm):
    require([function["contract"] for function in warm["verus-infer"]["functions"]]
        == [function["contract"] for function in cold["verus-infer"]["functions"]],
        "warm exact contract changed")


def certificate_import(certificate, fingerprint):
    identities = {entry[0]["path"]["krate"]["Id"][1] for entry in certificate["proofs"]}
    require(len(identities) == 1 and all(type(identity) is int for identity in identities),
        "ambiguous provider crate identity")
    return [dict(crate_id=next(iter(identities)), fingerprint=fingerprint)]


def check_client_import(data, expected):
    require(data["verus-infer"]["imported_inference_dependencies"] == expected,
        "imported certificate fingerprint differs from checked provider")


def cache_identity(cache, certificate):
    entries = {path.name: json.loads(path.read_text()) for path in sorted(cache.glob("*.json"))}
    versions = {entry["version"] for entry in entries.values()}
    require(versions == {CACHE_VERSION},
        "unexpected or missing candidate cache version")
    version = next(iter(versions))
    policy = re.fullmatch(r"candidate-bound-evidence-v([0-9]+)", certificate["policy"])
    require(policy is not None, "unrecognized interface policy")
    return dict(engine_identity=[int(version.split(":")[0].removeprefix("verus-infer-v")),
        int(policy[1]), certificate["schema"]], cache_version=version,
        cache_entry_sha256={name: digest(cache / name) for name in entries})


def check_completion(clean, cases, guard, imported_guard):
    require(clean == "", "application checkout became dirty")
    guard.check()
    imported_guard.check()
    require(len(cases) == len(EXPECTED_CASES)
        and {case["name"]: case["category"] for case in cases} == EXPECTED_CASES,
        "missing, duplicate or misclassified replay cases")
    failures = [case for case in cases if case["category"] != "positive"]
    require(len(failures) == 14 and all(isinstance(case.get("seeded_export_sha256"), str)
        and re.fullmatch("[0-9a-f]{64}", case["seeded_export_sha256"])
        and case["export_sha256"] is None for case in failures),
        "stale interface survived failure or was not seeded")


def validate_case(run, category, artifact, local_count, marker=None):
    require(category in ("positive", "before", "semantic", "safety", "frontend", "import", "budget"),
        "unknown replay category")
    positive = category == "positive"
    if category == "budget":
        data = json.loads(run["stdout"])
        require(run["returncode"] != 0 and data["verification-results"].get("success") is False
            and not data["verification-results"].get("encountered-vir-error")
            and not any(text in run["stderr"].lower() for text in ("panicked at", "timed out")),
            "budget control failed for an unrelated reason")
    else:
        data = validate_run(run, positive, allow_frontend=category in ("frontend", "import"))
    require_publication(data, positive, artifact)
    inference, results = data["verus-infer"], data["verification-results"]
    if positive:
        scope = inference.get("verification_scope", {})
        require(scope.get("native_collection_completed") is True
            and scope.get("complete_selected_scope_verified") is True,
            "positive replay did not complete its selected native scope")
        expected = {2: (11, 7), 0: (5, 5), 1: (3, 1)}.get(local_count)
        require(expected is not None and type(results.get("verified")) is int
            and results["verified"] == expected[0]
            and type(results.get("errors")) is int and results["errors"] == 0
            and type(scope.get("checked_exec_bodies")) is int
            and scope["checked_exec_bodies"] == expected[1]
            and type(scope.get("checked_proof_bodies")) is int
            and scope["checked_proof_bodies"] == 0,
            "positive replay proof or native-body counts changed")
        if local_count == 1:
            helper = "fee_payment_fee::PaymentFee::absolute_fee"
            native = scope.get("native_functions", [])
            require(len(inference["functions"]) == 1
                and inference["functions"][0].get("function") == helper
                and isinstance(native, list) and len(native) == 1
                and isinstance(native[0], dict) and native[0].get("function") == helper
                and native[0].get("body_status") == "checked_exec_body"
                and native[0].get("selected") is True
                and native[0].get("automatically_derived") is False
                and native[0].get("inference_candidate") is True
                and native[0].get("ordinary_contract") is False,
                "private positive helper identity or native body scope changed")
    if local_count is not None:
        require(results.get("is-verifying-entire-crate") is True,
            "selected-function verification is not a complete replay")
        require(len(inference["functions"]) == local_count, "wrong local inference scope")
        require_exact(inference["functions"])
        domains = {"true", "(1000000 != 0)"} if local_count == 1 else {"true"}
        require(all(function["contract"]["requires"] in domains
            and function["exported"] == (positive and local_count != 1)
            for function in inference["functions"]),
            "domain narrowed or publication barrier disagrees")
    if category in ("semantic", "before"):
        require(inference["status"] == "caller_proof_failed" and results.get("errors") == 1)
        require("postcondition not satisfied" in run["stderr"]
            and "precondition not satisfied" not in run["stderr"], "not a semantic failure")
    if category == "safety":
        require(results.get("errors") == 1, "safety control must fail exactly one obligation")
    if category == "frontend":
        require(results.get("encountered-error") is True
            and type(results.get("verified", 0)) is int and results.get("verified", 0) == 0
            and type(results.get("errors", 0)) is int and results.get("errors", 0) == 0
            and "postcondition not satisfied" not in run["stderr"],
            "frontend control reached semantic verification")
    if category == "import":
        require(results.get("encountered-vir-error") is True and "Verus Infer:" in run["stderr"],
            "not an inference import rejection")
    if category == "budget":
        require("resource limit" in run["stderr"], "budget failure diagnostic missing")
        require(inference["status"] == "required_proof_incomplete"
            and len(inference["functions"]) == 2
            and all(function.get("exported") is False for function in inference["functions"])
            and any(function.get("required_checks_passed") is False
                and any(proof.get("required") is True and proof.get("status") == "incomplete"
                    and proof.get("reason") == "resource_limit" for proof in function["proofs"].values())
                for function in inference["functions"]), "budget control did not block inference")
    if marker:
        require(marker in run["stderr"], "missing expected diagnostic: " + marker)
    return data


def provider_certificate(blob):
    certificate, fingerprint = metadata(blob)
    require(type(certificate["schema"]) is int and certificate["schema"] == INTERFACE_SCHEMA
        and certificate["policy"] == INTERFACE_POLICY
        and all(certificate.get(key) is True for key in ("soundness", "maximality", "exactness"))
        and len(certificate["proofs"]) == 2 and certificate["trusted_models"] == []
        and certificate["dependencies"] == [], "unexpected provider certificate scope or trust")
    require({entry[0]["path"]["segments"][-1] for entry in certificate["proofs"]}
        == {"fee_encoded_expiration", "fee_from_expiration"}
        and all(entry[1].get(key) is True for entry in certificate["proofs"]
            for key in ("soundness", "maximality", "exactness")),
        "helper certificate scope or checks changed")
    return certificate, fingerprint


def run_case(command, environment, output, verifier, receipt, *, name, phase="cold",
        category="positive", local_count=2, imports=(), extra=(), marker=None, imported_guard=None):
    label = name + "_" + phase
    source = output / (("provider" if name == "budget" else name) + ".rs")
    artifact = output / (("provider" if name == "provider" else label) + ".vir")
    phase_artifact = output / (label + ".vir")
    positive = category == "positive"
    if positive:
        require(not phase_artifact.exists(), "positive export path already exists")
        artifact.unlink(missing_ok=True)
        require(not artifact.exists(), "working export path was not cleared")
    else:
        provider = output / "provider_warm.vir"
        artifact.write_bytes(provider.read_bytes())
        require(artifact.is_file() and digest(artifact) == digest(provider),
            "failure export was not seeded")
    seeded = digest(artifact) if artifact.exists() else None
    argv = [str(verifier), str(source), "--crate-type=lib", "--edition=2021", "--crate-name",
        "fee_provider" if name == "provider" else "fee_" + name, "--no-cheating", "--output-json",
        "--num-threads", "1", "--export", str(artifact), *imports, *extra]
    if name == "provider":
        native_artifact = output / "libfee_provider.rlib"
        native_artifact.unlink(missing_ok=True)
        argv += ["--compile", "-o", str(native_artifact)]
    if imports:
        require(imported_guard is not None, "imported provider guard missing")
        imported_guard.check()
    code = command(label, argv, environment)
    if imports:
        imported_guard.check()
    run = dict(returncode=code, stdout=(output / (label + ".stdout")).read_text(),
        stderr=(output / (label + ".stderr")).read_text())
    data = validate_case(run, category, artifact, local_count, marker)
    if name == "provider":
        check_provider_phase(data, phase)
        require(native_artifact.is_file(), "provider native artifact missing")
        phase_artifact.write_bytes(artifact.read_bytes())
        (output / ("libfee_provider_" + phase + ".rlib")).write_bytes(native_artifact.read_bytes())
    if name == "client" and positive:
        check_client_import(data, receipt["provider_import"])
    receipt["cases"].append(dict(name=label, category=category, result=data,
        seeded_export_sha256=seeded, export_sha256=digest(artifact) if artifact.exists() else None))
    return data


def replay(args, output, receipt):
    application, verifier, solver = (value.resolve() for value in
        (args.application, args.verus, args.solver))
    engine = Path(__file__).resolve().parents[2]
    environment = dict(os.environ, GIT_OPTIONAL_LOCKS="0", RUSTUP_TOOLCHAIN="1.99.1",
        VERUS_Z3_PATH=str(solver), VERUS_INFER_CACHE=str(output / "candidate-cache"))
    command = LoggedCommands(output, engine / "source", receipt["commands"], timeout=args.timeout)

    def invoke(label, argv):
        return setup_output(command, label, argv, environment, output)

    check_pin(invoke("application_head", ["git", "-C", application, "rev-parse", "HEAD"]),
        invoke("application_clean", ["git", "-C", application, "status", "--porcelain"]),
        {relative: digest(application / relative) for relative in SOURCE_HASHES})
    for relative in SOURCE_HASHES:
        destination = output / "application-inputs" / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes((application / relative).read_bytes())
    receipt["source_sha256"] = SOURCE_HASHES
    receipt["funding_caller"] = caller_evidence(application, output)
    check_call_paths((application / RECURRING).read_text(), (application / RECEIVE).read_text())
    helpers = extract_helpers((application / CONTRACTS).read_text())
    receipt["item_sha256"] = ITEM_HASHES
    receipt["adaptations"] = ["extract both original scalar function items without preceding doc comments",
        "add Infer attributes outside the items; import vstd and wrap in verus!",
        "append independently authored mathematical intent; no native body or type edits"]
    sources = {
        "provider": proof_source(helpers),
        "unannotated": proof_source(helpers, ROUNDTRIP, inferred=False),
        "mutant": proof_source(mutant(helpers), ROUNDTRIP),
        "false_future": proof_source(helpers,
            "pub fn false_future() -> (expiration: u64) ensures expiration > 0,\n"
            "{ fee_encoded_expiration(u64::MAX) }\n"),
        "overflow": proof_source(helpers, "pub fn overflow(fee: u64) -> u64 "
            "{ fee_from_expiration(fee_encoded_expiration(fee)) + 1 }\n"),
        "panic": proof_source(helpers, 'pub fn panic_control() { panic!("safety control"); }\n'),
        "malformed": proof_source(helpers, ROUNDTRIP.replace("recovered == fee", "recovered == true")),
        "payment_fee": fee_probe(application),
        "client": "use vstd::prelude::*;\nuse fee_provider::{fee_encoded_expiration, fee_from_expiration};\n"
            + "verus! {\n" + GOALS + "}\n",
    }
    for name, source in sources.items():
        (output / (name + ".rs")).write_text(source)
    native = output / "native.rs"
    native.write_text(helpers + "\nfn main() {\n"
        "let values = [0, 1, 2, 1_000, u32::MAX as u64, u64::MAX / 2, u64::MAX - 1, u64::MAX];\n"
        "for fee in values { assert_eq!(fee_from_expiration(fee_encoded_expiration(fee)), fee); }\n}\n")
    compiler = Path(invoke("rustc", ["rustup", "which", "--toolchain", "1.99.1", "rustc"]))
    files = invoke("engine_files", ["git", "-C", engine, "ls-files", "--cached", "--others",
        "--exclude-standard", "source", "dependencies", "rust-toolchain.toml"]).splitlines()
    engine_sources = [engine / relative for relative in files if (engine / relative).is_file()]
    receipt["engine_source_sha256"] = {str(path.relative_to(engine)): digest(path) for path in engine_sources}
    app_files = invoke("application_files", ["git", "-C", application, "ls-files", "--cached",
        "--others", "--exclude-standard"]).splitlines()
    compiler_paths = compiler_artifacts(compiler)
    inputs = [*engine_sources, *(application / relative for relative in app_files),
        *output.glob("*.rs"), *list((engine / "source/tools/fixtures/transfers/transfer-fedimint-fee-encoding").glob("*")),
        *compiler_paths]
    guard = ArtifactGuard(tool_artifacts(verifier, solver, extra=[path for path in inputs if path.is_file()]))
    receipt["artifacts_before"] = guard.initial
    receipt["rust_toolchain_artifacts"] = {str(path.resolve()): guard.initial[str(path.resolve())]
        for path in compiler_paths}
    command.guard = guard
    invoke("native_build", [compiler, native, "--edition=2021", "-C", "overflow-checks=yes",
        "-o", output / "native"])
    invoke("native_samples", [output / "native"])

    def check(name, phase="cold", **options):
        return run_case(command, environment, output, verifier, receipt, name=name, phase=phase,
            imported_guard=imported_guard, **options)

    imported_guard = None
    cold = check("provider")
    provider = check("provider", "warm")
    receipt["provider_candidate_ids"] = [function["candidate_id"]
        for function in provider["verus-infer"]["functions"]]
    check_provider_contracts(cold, provider)
    receipt["cold_provider_candidate_ids"] = [function["candidate_id"]
        for function in cold["verus-infer"]["functions"]]
    certificate, fingerprint = provider_certificate((output / "provider_warm.vir").read_bytes())
    receipt.update(cache_identity(output / "candidate-cache", certificate))
    receipt["provider_import"] = certificate_import(certificate, fingerprint)
    imported_guard = ArtifactGuard([output / "provider_warm.vir", output / "libfee_provider_warm.rlib"])
    imports = ["--extern", "fee_provider=" + str(output / "libfee_provider_warm.rlib"),
        "--import", "fee_provider=" + str(output / "provider_warm.vir")]
    for phase in ("cold", "warm"):
        check("client", phase, local_count=0, imports=imports)
    check("unannotated", category="before", local_count=0, imports=imports, marker="recovered == fee")
    check("mutant", category="semantic", marker="recovered == fee")
    check("false_future", category="semantic", marker="expiration > 0")
    check("overflow", category="safety", marker="possible arithmetic underflow/overflow")
    check("panic", category="safety", marker="precondition not satisfied")
    check("malformed", category="frontend", local_count=None, marker="error[E0277]")
    check("payment_fee", local_count=1)
    check("budget", category="budget", local_count=None, extra=["--rlimit", "0.000001"])
    original = (output / "provider_warm.vir").read_bytes()
    for fault in ("schema", "policy", "exactness", "summaries", "dependencies", "trusted_models", "missing"):
        if fault == "missing":
            corrupted = original[:-8]
        else:
            def edit(certificate):
                replacements = dict(schema=0, policy="unapproved-policy", exactness=False,
                    summaries="0" * 64, dependencies=[dict(crate_id=0, fingerprint="0" * 64)],
                    trusted_models=[[certificate["proofs"][0][0], "0" * 64]])
                certificate[fault] = replacements[fault]
            corrupted = replace_metadata(original, edit)
        path = output / ("bad-" + fault + ".vir")
        path.write_bytes(corrupted)
        bad_imports = list(imports)
        bad_imports[bad_imports.index("--import") + 1] = "fee_provider=" + str(path)
        diagnostics = dict(schema="incompatible inference schema, safety policy, or producer",
            policy="incompatible inference schema, safety policy, or producer",
            exactness="incomplete inference proof guarantees", summaries="inferred contract fingerprint mismatch",
            dependencies="stale or missing inferred dependency fingerprint",
            trusted_models="stale or missing trusted-model fingerprint",
            missing="missing or invalid inference proof metadata")
        check("client", "metadata_" + fault, category="import", local_count=None,
            imports=bad_imports, marker="Verus Infer: " + diagnostics[fault])
    check_completion(invoke("application_clean_after", ["git", "-C", application, "status", "--porcelain"]),
        receipt["cases"], guard, imported_guard)
    receipt.update(artifacts_after=guard.snapshot(), provider_artifacts=imported_guard.snapshot(),
        status="proved", stale_exports_removed=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("application", "verus", "solver", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--timeout", type=positive_timeout, default=180)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix="fee-encoding-", dir=args.output.resolve()))
    receipt = dict(commands=[], cases=[], status="blocked", application_commit=APPLICATION,
        domain="all u64 inputs, no authored entry precondition",
        trust=["existing Rust/frontend/Verus/backend/Z3 arithmetic correspondence only"],
        limitations=["scalar fee transport only, not fee computation or fund safety",
            "no persistent-state, clock, operation, hash, identity or crypto specification"])
    try:
        with replay_receipt(output, receipt):
            replay(args, output, receipt)
    finally:
        print(output)


if __name__ == "__main__":
    main()
