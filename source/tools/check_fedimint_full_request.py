"""Replay the complete pinned Fedimint request with an explicit opaque hash identity.

Builds the real Bitcoin dependency offline from the application lockfile. Never
edits the application or uses a historical fixture, target or developer checkout.
"""

import argparse
import json
import os
from pathlib import Path
import re
import tempfile
import tarfile

from check_fedimint_native import (APPLICATION, CLIENT, CONSTANT, FOLLOWUP, GUARD,
    IMPL_HASH, LIGHTNING, METHOD_HASHES, POLICY, SOURCE_HASHES, extract, item,
    regress, text_digest)
from infer_eval_support import ArtifactGuard, digest, host_info, tool_artifacts
from infer_experiment import (LoggedCommands, positive_timeout, replay_receipt,
    require, require_exact, require_publication, validate_run)


SERVER = "gateway/fedimint-gateway-server/src/lib.rs"
INPUT_HASHES = dict(SOURCE_HASHES, **{
    SERVER: "2f15bfa050213c3fd041b93d5736d83882ad814ac36472eb7df64688824ca831",
    "Cargo.lock": "2c3e0e34ab540ce550c4d4fd35cd7d12c30c624909edb9909ad96af402879b7b",
})
REQUEST_HASH = "c29581724613471983ca743bcd5f78702684c430d8bba284027514e69da715c6"
CALLERS = {
    "try_handle_lightning_payment_lnv2": (SERVER, "async fn try_handle_lightning_payment_lnv2(",
        "17f641aa03c217bd2c797513c6439bf3e6f3d990fd02d813e9ba1770d59944a4"),
    "get_registered_incoming_contract_and_client_v2": (SERVER,
        "pub async fn get_registered_incoming_contract_and_client_v2(",
        "d74a295d4d0e25d7f0b62901aa39c201d7702c1b60c5d218ac063a831f409be5"),
    "relay_incoming_htlc": (CLIENT, "pub async fn relay_incoming_htlc(",
        "3306c8cfd463fcf4377e649237b5a073eccebea0546cfe50b45647a9d7da3e55"),
    "incoming_relay_plan": (CLIENT, "fn incoming_relay_plan(",
        "c95c04b1d834795965cbab8b38628d3c65df538311c102a95cb4abbb1c517fcb"),
}
FIELDS = ["payment_hash: sha256::Hash", "amount_msat: u64", "incoming_amount_msat: u64",
          "expiry: u32", "incoming_chan_id: u64", "short_channel_id: Option<u64>", "htlc_id: u64"]
OPAQUE = """#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExSha256Hash(sha256::Hash);
"""
IRRELEVANCE = """
pub fn deadline_irrelevance(left: &InterceptPaymentRequest, right: &InterceptPaymentRequest,
    height: Option<u32>) -> (budgets: (u32, u32))
    ensures (left.expiry == right.expiry && left.incoming_chan_id == right.incoming_chan_id
        && left.htlc_id == right.htlc_id) ==> budgets.0 == budgets.1,
{
    (left.lnv2_blocks_to_claim_deadline(height), right.lnv2_blocks_to_claim_deadline(height))
}
"""
DEPENDENCY = """[package]
name = "full-request-dependency"
version = "0.1.0"
edition = "2021"
[dependencies]
bitcoin = { version = "=0.32.8", features = ["serde"] }
[workspace]
"""


def lock_packages(text):
    """Read only identity/checksum fields in Cargo's generated lockfile format.

    Cargo validates/resolves the actual lockfile; this deliberately small reader
    only compares the selected registry packages with the application's pins.
    """
    result = {}
    for block in text.split("[[package]]")[1:]:
        fields = dict(re.findall(r'^(name|version|source|checksum) = ("[^"\n]*")$', block, re.M))
        require("name" in fields and "version" in fields, "malformed lockfile identity")
        fields = {key: json.loads(value) for key, value in fields.items()}
        key = (fields["name"], fields["version"])
        require(key not in result, "ambiguous lockfile identity")
        result[key] = fields
    require(result, "empty lockfile")
    return result


def pinned_dependencies(application_lock, selected_lock):
    available, selected = lock_packages(application_lock), lock_packages(selected_lock)
    checked = []
    for key, package in selected.items():
        if "source" not in package:
            require(key == ("full-request-dependency", "0.1.0"), "unexpected local dependency")
            continue
        require(package.get("source") == "registry+https://github.com/rust-lang/crates.io-index",
                "unexpected dependency source")
        require(package == available.get(key), "dependency is not the application's exact pin: " + str(key))
        require(re.fullmatch(r"[0-9a-f]{64}", package.get("checksum", "")), "missing registry checksum")
        checked.append(package)
    require(any(p["name"] == "bitcoin" and p["version"] == "0.32.8" for p in checked))
    require(any(p["name"] == "bitcoin_hashes" and p["version"] == "0.14.0" for p in checked))
    return checked


def extract_request(source):
    constant, implementation = extract(source)
    definition = item(source, "pub struct InterceptPaymentRequest {")
    require(text_digest(definition) == REQUEST_HASH, "complete request item changed")
    fields = re.findall(r"^    pub ([^,\n]+),$", definition, re.M)
    require(fields == FIELDS, "all seven original fields/types must remain intact")
    derive = "#[derive(Debug, Serialize, Deserialize, Clone)]"
    start = source.index(definition)
    require(source[:start].rstrip().endswith(derive), "original derive changed")
    return constant, definition, implementation, derive


def check_application_pin(head, status, hashes):
    require(head == APPLICATION, "application HEAD differs from pin")
    require(status == "", "application checkout is dirty")
    require(hashes == INPUT_HASHES, "application input changed")


def proof_source(constant, definition, implementation, policy=POLICY + FOLLOWUP + IRRELEVANCE,
                 opaque=OPAQUE):
    annotated = implementation.replace("    pub fn ", "    #[verifier::infer]\n    pub fn ")
    require(annotated.count("#[verifier::infer]") == 2)
    for marker in METHOD_HASHES:
        require(item(annotated, marker) == item(implementation, marker), "method item changed")
    return ("use bitcoin::hashes::sha256;\nuse vstd::prelude::*;\nverus! {\n" + opaque
            + constant + "\n" + definition + "\n" + annotated + policy + "}\n")


def invariant_control(constant, definition, implementation):
    # A legal type invariant needs private fields in a nested module. This
    # negative-only visibility adaptation must reach Infer's invariant gate,
    # rather than fail the frontend's public-field restriction first.
    source = proof_source(constant, definition.replace("    pub ", "    "), implementation,
        "impl InterceptPaymentRequest { #[verifier::type_invariant] pub closed spec fn valid(&self) -> bool { self.expiry < 100 } }\n")
    source = source.replace(OPAQUE + constant, OPAQUE + "mod guarded { use super::*;\n" + constant, 1)
    return source[:-2] + "}\n}\n"


def replace_metadata(blob, edit):
    magic = b"VRINFER1"
    require(blob.endswith(magic), "missing inference trailer")
    decoder = json.JSONDecoder()
    # JSON may contain braces in string values. Only accept a decoded object
    # ending exactly at the trailer with the expected certificate fields.
    for index in range(len(blob) - len(magic)):
        if blob[index:index + 1] != b"{":
            continue
        try:
            text = blob[index:-len(magic)].decode()
            certificate, end = decoder.raw_decode(text)
            if text[end:].strip() or not isinstance(certificate, dict):
                continue
            if not {"schema", "policy", "proofs", "summaries"} <= certificate.keys():
                continue
            edit(certificate)
            return blob[:index] + json.dumps(certificate, separators=(",", ":")).encode() + magic
        except (ValueError, UnicodeError):
            continue
    raise ValueError("could not locate inference metadata")


def caller_evidence(application, output):
    fixtures = {}
    for name, (relative, marker, expected) in CALLERS.items():
        selected = item((application / relative).read_text(), marker)
        require(text_digest(selected) == expected, "funding caller source changed")
        path = output / (name + ".source.rs")
        path.write_text(selected)  # exact item bytes, with no added final newline
        fixtures[name] = dict(source=relative, marker=marker, sha256=digest(path))
    caller = (output / "try_handle_lightning_payment_lnv2.source.rs").read_text()
    matcher = (output / "get_registered_incoming_contract_and_client_v2.source.rs").read_text()
    relay = (output / "relay_incoming_htlc.source.rs").read_text()
    require(caller.count("htlc_request.incoming_amount_msat") == 2)
    require("htlc_request.amount_msat" not in caller)
    require("registered_incoming_contract.incoming_amount_msats != amount_msats" in matcher)
    require("blocks_to_claim_deadline < LNV2_CLAIM_DEADLINE_MARGIN" in relay)
    return dict(status="blocked", fixtures=fixtures,
        proposed_property="Any new funding initiated along this caller requires a non-forward payment, "
            "known height, expiry at least two blocks later, and the registered incoming amount equal "
            "to request.incoming_amount_msat. A contract/fee amount relation requires a separate "
            "proved registration invariant.",
        first_blocker="The caller first awaits get_registered_incoming_contract_and_client_v2: "
            "an async persistent lookup keyed by PaymentImage::Hash. Its result and amount equality "
            "depend on database state; the readonly helper model supplies neither that state nor "
            "an async/effect specification.",
        further_boundaries=["Hash value access/payment-image construction is outside unused opaque support",
            "Lightning RPC supplies current height asynchronously",
            "Fresh versus replay funding depends on three asynchronous operation-existence queries",
            "Federation liveness, transaction submission, failures and concurrent operation state",
            "Registered contract, cryptographic commitment and fee provenance"],
        result="Source wiring inspected only; no caller/funding/fee safety theorem or semantic mutant claimed")


def replay(args, output, receipt):
    application, verifier, solver = args.application.resolve(), args.verus.resolve(), args.solver.resolve()
    engine = Path(__file__).resolve().parents[2]
    environment = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.1", CARGO_BUILD_JOBS="2",
        RUST_MIN_STACK="20971520", VERUS_Z3_PATH=str(solver))
    if getattr(args, "dependency_home", None) is not None:
        from prepare_infer_dependencies import cargo_environment
        environment = dict(cargo_environment(args.dependency_home.resolve()),
            RUST_MIN_STACK="20971520", VERUS_Z3_PATH=str(solver))
    command = LoggedCommands(output, engine / "source", receipt["commands"], timeout=args.timeout)

    def invoke(label, argv):
        require(command(label, list(map(str, argv)), environment) == 0, "setup failed: " + label)
        return (output / (label + ".stdout")).read_text().strip()

    check_application_pin(invoke("application_head", ["git", "-C", application, "rev-parse", "HEAD"]),
        invoke("application_clean", ["git", "-C", application, "status", "--porcelain"]),
        {p: digest(application / p) for p in INPUT_HASHES})
    for relative in INPUT_HASHES:
        path = output / "application-inputs" / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes((application / relative).read_bytes())
    constant, definition, implementation, derive = extract_request((application / LIGHTNING).read_text())
    require("const LNV2_CLAIM_DEADLINE_MARGIN: u32 = 2;" in (application / CLIENT).read_text())
    receipt["source_sha256"] = INPUT_HASHES
    receipt["items"] = dict(request=REQUEST_HASH, implementation=IMPL_HASH, methods=METHOD_HASHES,
        constant=text_digest(constant), omitted_derive=text_digest(derive), opaque_correspondence=text_digest(OPAQUE))
    receipt["adaptations"] = [
        dict(change="Omit the adjacent Debug/Serialize/Deserialize/Clone derive only", original=derive,
             before_sha256=text_digest(derive), after_sha256=text_digest("")),
        dict(change="Add one Infer attribute before each complete original method item",
             before_sha256=text_digest(implementation), after_sha256=text_digest(
                 implementation.replace("    pub fn ", "    #[verifier::infer]\n    pub fn "))),
        dict(change="Wrap exact request, constant and annotated impl in verus! and import real sha256 type",
             request_sha256=REQUEST_HASH),
        dict(change="Add explicit opaque external-type identity correspondence; no SHA/layout/trait laws",
             sha256=text_digest(OPAQUE)),
    ]
    receipt["funding_caller"] = caller_evidence(application, output)

    dependency = output / "dependency"
    (dependency / "src").mkdir(parents=True)
    (dependency / "Cargo.toml").write_text(DEPENDENCY)
    (dependency / "src/lib.rs").write_text("pub use bitcoin::hashes::sha256;\n")
    (dependency / "Cargo.lock").write_bytes((application / "Cargo.lock").read_bytes())
    target = output / "dependency-target"
    environment.update(CARGO_TARGET_DIR=str(target), CARGO_BUILD_BUILD_DIR=str(target))
    metadata = json.loads(invoke("dependency_resolve", ["cargo", "metadata", "--offline",
        "--format-version", "1", "--manifest-path", dependency / "Cargo.toml"]))
    selected = pinned_dependencies((application / "Cargo.lock").read_text(), (dependency / "Cargo.lock").read_text())
    locked_hash = digest(dependency / "Cargo.lock")
    dependency_sources = []
    for package in metadata["packages"]:
        if package["source"] is not None:
            directory = Path(package["manifest_path"]).parent
            matching = next(p for p in selected if p["name"] == package["name"] and p["version"] == package["version"])
            archive = directory.parents[2] / "cache" / directory.parent.name / (directory.name + ".crate")
            require(digest(archive) == matching["checksum"], "registry archive differs from pinned checksum")
            dependency_sources.append(archive)
            with tarfile.open(archive) as contents:
                for member in contents.getmembers():
                    require(not member.issym() and not member.islnk(), "registry archive contains a link")
                    if not member.isfile():
                        continue
                    relative = Path(member.name)
                    require(relative.parts[0] == directory.name and ".." not in relative.parts
                        and not relative.is_absolute(), "invalid registry archive path")
                    path = directory.parent / relative
                    require(path.read_bytes() == contents.extractfile(member).read(),
                            "registry source file differs from pinned archive: " + str(path))
                    dependency_sources.append(path)
    dependency_guard = ArtifactGuard([*dependency_sources, dependency / "Cargo.toml",
        dependency / "Cargo.lock", dependency / "src/lib.rs"])
    receipt["dependency_sources_before"] = dependency_guard.initial
    command.guard = dependency_guard
    invoke("dependency_locked", ["cargo", "build", "--offline", "--locked", "--release", "--manifest-path", dependency / "Cargo.toml"])
    require(digest(dependency / "Cargo.lock") == locked_hash, "locked build changed dependency lock")
    receipt["dependency_sources_after"] = dependency_guard.snapshot()
    deps = target / "release/deps"
    bitcoin = list(deps.glob("libbitcoin-*.rlib"))
    require(len(bitcoin) == 1, "ambiguous Bitcoin artifact")
    arguments = ["--extern", "bitcoin=" + str(bitcoin[0]), "-L", "dependency=" + str(deps)]
    receipt["dependency"] = dict(packages=selected, lock_sha256=locked_hash,
        bitcoin_rlib_sha256=digest(bitcoin[0]), features="bitcoin default features plus serde; not full application feature equivalence",
        artifacts={str(p.relative_to(target)): digest(p) for p in target.rglob("*") if p.is_file() and
            (p.suffix in (".rlib", ".rmeta", ".dylib", ".json") or ".fingerprint" in p.parts)})

    policies = POLICY + FOLLOWUP + IRRELEVANCE
    sources = {
        "provider": proof_source(constant, definition, implementation),
        "regression": proof_source(constant, definition, regress(implementation)),
        "false_policy": proof_source(constant, definition, implementation, policies.replace("blocks == 0,", "blocks == 1,", 1)),
        "malformed": proof_source(constant, definition, implementation, policies.replace("blocks == 0,", "blocks == true,", 1)),
        "unsupported": proof_source(constant, definition, implementation, opaque=""),
        "wrong_proxy": proof_source(constant, definition, implementation, opaque=OPAQUE.replace("sha256::Hash", "bitcoin::hashes::sha512::Hash")),
        "opaque_access": proof_source(constant, definition, implementation.replace(
            "        let circuit =", "        &self.payment_hash;\n        let circuit =")),
        "opaque_alias": proof_source(constant, definition, implementation.replace(
            "        let circuit =", "        let alias = self;\n        let _hash = &alias.payment_hash;\n        let circuit =")),
        "opaque_mutable": proof_source(constant, definition, implementation, policies +
            "#[verifier::infer] pub fn opaque_mutable(request: &mut InterceptPaymentRequest) {}\n"),
        "opaque_by_value": proof_source(constant, definition, implementation, policies +
            "#[verifier::infer] pub fn opaque_by_value(request: InterceptPaymentRequest) -> [u8; 8] { request.amount_msat.to_le_bytes() }\n"),
        "opaque_trait": proof_source(constant, definition, implementation.replace(
            "        let circuit =", "        let _equal = self.payment_hash == self.payment_hash;\n        let circuit =")),
        "opaque_invariant": invariant_control(constant, definition, implementation),
        "overflow": proof_source(constant, definition, implementation, policies +
            "pub fn overflow(r: &InterceptPaymentRequest, x: u32) -> u32 { r.incoming_circuit(); x + 1 }\n"),
        "panic": proof_source(constant, definition, implementation, policies +
            'pub fn original_panic(r: &InterceptPaymentRequest) { r.incoming_circuit(); panic!("safety control"); }\n'),
    }
    sources["client"] = "use vstd::prelude::*;\nuse full_provider::InterceptPaymentRequest;\nverus! {\n" + policies + "}\n"
    for name, source in sources.items():
        (output / (name + ".rs")).write_text(source)
    native = output / "native.rs"
    native.write_text("use bitcoin::hashes::sha256;\n" + constant + "\n" + definition + "\n" + implementation + "\n")
    compiler = Path(invoke("rustc", ["rustup", "which", "--toolchain", "1.99.1", "rustc"]))
    receipt["engine_commit"] = invoke("engine_head", ["git", "-C", engine, "rev-parse", "HEAD"])
    files = invoke("engine_files", ["git", "-C", engine, "ls-files", "--cached", "--others", "--exclude-standard",
        "source", "dependencies", "rust-toolchain.toml"]).splitlines()
    engine_sources = [engine / p for p in files if (engine / p).is_file()]
    receipt["engine_source_sha256"] = {str(p.relative_to(engine)): digest(p) for p in engine_sources}
    inputs = [*output.glob("*.rs"), *dependency.rglob("*"), *deps.glob("*.rlib"), *deps.glob("*.rmeta"),
        *deps.glob("*.dylib"), *dependency_sources, *engine_sources, compiler,
        *(application / p for p in INPUT_HASHES)]
    guard = ArtifactGuard(tool_artifacts(verifier, solver, extra=[p for p in inputs if p.is_file()]))
    baseline_guard = ArtifactGuard(tool_artifacts(args.baseline_verus.resolve(), solver)) if args.baseline_verus else None
    if baseline_guard:
        receipt["baseline_artifacts_before"] = baseline_guard.initial
    command.guard = guard
    receipt["artifacts_before"] = guard.initial
    invoke("native_build", [compiler, native, "--edition=2021", "--crate-type=lib", "-C", "overflow-checks=yes",
        "-o", output / "native.rlib", *arguments])
    receipt["native_rlib_sha256"] = digest(output / "native.rlib")

    def check(name, phase="cold", category="positive", *, tool=verifier, imports=(), marker=None):
        label = name + "_" + phase
        artifact = output / (label + ".vir")
        positive = category == "positive"
        if not positive:
            artifact.write_bytes((output / "provider_cold.vir").read_bytes())
        previous = digest(artifact) if artifact.exists() else None
        environment["VERUS_INFER_CACHE"] = str(output / (name + "-cache"))
        source = output / (("provider" if name == "baseline_opaque" else name) + ".rs")
        argv = [str(tool), str(source), "--crate-type=lib", "--edition=2021", "--crate-name",
            "full_provider" if name == "provider" else "full_" + name, "--no-cheating", "--output-json",
            "--num-threads", "1", "--export", str(artifact), *arguments, *imports]
        if name == "provider":
            argv += ["--compile", "-o", str(output / "libfull_provider.rlib")]
        code = command(label, argv, environment)
        run = dict(returncode=code, stdout=(output / (label + ".stdout")).read_text(),
            stderr=(output / (label + ".stderr")).read_text())
        frontend = category == "frontend"
        data = validate_run(run, positive, allow_frontend=frontend)
        require_publication(data, positive, artifact)
        functions = data["verus-infer"]["functions"]
        if not frontend:
            require(data["verification-results"].get("is-verifying-entire-crate") is True,
                    "selected-function verification is not a full replay")
        if not frontend and name != "client":
            require(len(functions) == 2, "both complete native helpers must be checked")
            require_exact(functions)
            require(all(f["contract"]["requires"] == "true" for f in functions), "input coverage narrowed")
            require(all(f["exported"] == positive for f in functions), "helper publication barrier disagrees")
            require(all(f["metrics"]["cache"] == "miss" and f["metrics"].get("cache_policy") == "regenerated_state"
                for f in functions), "record summaries must honestly regenerate on warm runs")
        if category == "semantic":
            require(data["verus-infer"].get("status") == "caller_proof_failed")
            require(data["verification-results"].get("errors") == 1)
            require("postcondition not satisfied" in run["stderr"] and "precondition not satisfied" not in run["stderr"])
            require(any("deadline_policy" in f for f in data["verus-infer"]["failed_functions"]))
        elif category == "safety":
            require(data["verification-results"].get("errors") == 1)
            expected_safety = "possible arithmetic underflow/overflow" if name == "overflow" else "precondition not satisfied"
            require(expected_safety in run["stderr"], "safety control must fail an actual obligation")
        elif frontend:
            require(data["verification-results"].get("encountered-error") is True or
                    data["verification-results"].get("encountered-vir-error") is True)
        if name == "client" and positive:
            require(not functions and data["verus-infer"]["imported_inference_dependencies"],
                    "client must use the imported provider's checked equations")
        if marker:
            require(marker in run["stderr"], "missing expected diagnostic: " + marker)
        row = dict(name=name, phase=phase, category=category, seeded_export_sha256=previous,
            export_sha256=digest(artifact) if artifact.exists() else None, source_sha256=digest(source),
            verification=data["verification-results"], inference=data["verus-infer"])
        receipt["cases"].append(row)
        return data

    cold = check("provider")
    warm = check("provider", "warm")
    require([f["contract"] for f in cold["verus-infer"]["functions"]] ==
            [f["contract"] for f in warm["verus-infer"]["functions"]], "warm equations changed")
    imported_guard = ArtifactGuard([output / "libfull_provider.rlib", output / "provider_warm.vir"])
    receipt["provider_artifacts_before"] = imported_guard.initial
    imported = ["--extern", "full_provider=" + str(output / "libfull_provider.rlib"),
        "--import", "full_provider=" + str(output / "provider_warm.vir"), "-L", "dependency=" + str(output)]
    check("client", imports=imported)
    check("client", "warm", imports=imported)
    if args.baseline_verus:
        check("unsupported", "baseline", "frontend", tool=args.baseline_verus.resolve(),
            marker="bitcoin_hashes::sha256::Hash` is not supported")
        check("baseline_opaque", "baseline", "frontend", tool=args.baseline_verus.resolve(),
            marker="Verus Infer unsupported")
    check("regression", category="semantic", marker="request.incoming_chan_id != 0 || request.htlc_id != 0")
    check("false_policy", category="semantic")
    check("overflow", category="safety")
    check("panic", category="safety")
    for name, marker in [("malformed", "blocks == true"), ("unsupported", "is not supported"),
                         ("wrong_proxy", "is not supported"), ("opaque_access", "must remain unused"),
                         ("opaque_alias", "Verus Infer unsupported"), ("opaque_mutable", "Verus Infer unsupported"),
                         ("opaque_by_value", "exact borrowed readonly record"),
                         ("opaque_trait", "trait inference requires"), ("opaque_invariant", "Verus Infer unsupported")]:
        check(name, category="frontend", marker=marker)
    # Mutate only independent copies, never the successful provider artifact.
    original = (output / "provider_warm.vir").read_bytes()
    for fault in ("policy", "proof", "summaries", "dependencies", "trusted_models", "missing"):
        if fault == "missing":
            corrupted = original[:-8]
        else:
            def edit(certificate):
                if fault == "policy": certificate["policy"] = "wrong-policy"
                elif fault == "proof": certificate["exactness"] = False
                elif fault == "summaries": certificate["summaries"] = "0" * 64
                elif fault == "trusted_models": certificate["trusted_models"][0][1] = "0" * 64
                else: certificate["dependencies"] = [dict(crate_id=0, fingerprint="0" * 64)]
            corrupted = replace_metadata(original, edit)
        path = output / ("bad-" + fault + ".vir")
        path.write_bytes(corrupted)
        bad_imports = list(imported)
        bad_imports[bad_imports.index("--import") + 1] = "full_provider=" + str(path)
        check("client", "metadata_" + fault, "frontend", imports=bad_imports, marker="Verus Infer:")
    require(invoke("application_clean_after", ["git", "-C", application, "status", "--porcelain"]) == "")
    guard.check()
    imported_guard.check()
    if baseline_guard:
        baseline_guard.check()
        receipt["baseline_artifacts_after"] = baseline_guard.snapshot()
    receipt["provider_artifacts_after"] = imported_guard.snapshot()
    receipt["artifacts_after"] = guard.snapshot()
    receipt["transfer"].update(status="proved", source_fidelity=True, field_count=7,
        positive_cases=[r["name"] + "_" + r["phase"] for r in receipt["cases"] if r["category"] == "positive"],
        semantic_negative_cases=[r["name"] + "_" + r["phase"] for r in receipt["cases"] if r["category"] == "semantic"],
        safety_negative_cases=[r["name"] + "_" + r["phase"] for r in receipt["cases"] if r["category"] == "safety"],
        frontend_negative_cases=[r["name"] + "_" + r["phase"] for r in receipt["cases"] if r["category"] == "frontend"],
        stale_export_rejected=all(r["seeded_export_sha256"] and r["export_sha256"] is None
            for r in receipt["cases"] if r["category"] != "positive"))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("application", "verus", "solver", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--baseline-verus", type=Path)
    parser.add_argument("--dependency-home", type=Path,
                        help="fresh explicit registry preparation; proof remains offline")
    parser.add_argument("--timeout", type=positive_timeout, default=300)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix="full-request-", dir=args.output.resolve()))
    receipt = dict(commands=[], cases=[], host=host_info(), transfer=dict(
        scenario="fedimint-full-request", application_commit=APPLICATION, status="blocked",
        source_fidelity=False, positive_cases=[], semantic_negative_cases=[], stale_export_rejected=False,
        limitations=["Selected helper bodies, not full application/dependency feature equivalence",
            "All native scalar/Option inputs; hash ranges over the real external type without SHA laws",
            "Funding caller remains blocked at async state/external effects; no funding or fee theorem"],
        trust=["Explicit audited external-type correspondence for bitcoin_hashes::sha256::Hash",
            "Readonly unused opaque transport only; no layout, values, invariants, traits or SHA behavior asserted",
            "Rust typing/erasure, ordinary Verus backend, existing vstd and native Option correspondences",
            "Authored policy is independent intent, never inferred from the guard"]))
    try:
        with replay_receipt(output, receipt):
            replay(args, output, receipt)
    finally:
        print(output)


if __name__ == "__main__":
    main()
