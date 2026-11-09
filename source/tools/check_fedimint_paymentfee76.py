"""Bounded native PaymentFee arithmetic experiment, separate from shared controllers."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import re
import shutil

from check_fedimint_native import item
from infer_transfer_identity import CACHE_VERSION, ENGINE_IDENTITY, INTERFACE_POLICY, INTERFACE_SCHEMA
from infer_experiment import (LoggedCommands, replay_receipt, require, require_exact,
    require_publication, validate_run, positive_timeout)
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_transfer_receipts import metadata
from check_fedimint_fee_encoding import compiler_artifacts, certificate_import
from check_fedimint_full_request import replace_metadata


PIN = "b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236"
FEES = "modules/fedimint-lnv2-common/src/gateway_api.rs"
AMOUNT = "fedimint-core/src/amount.rs"
SOURCE_HASHES = {
    FEES: "a3eda308673486e831abbf74c73adc6043fe28a9389e74773f804e70c0f551b6",
    AMOUNT: "9b1095eb5ed2f2500887bc7875d28e4cf46177a6d860680a23e08bef7b253dd1",
}
METHODS = ("pub const fn from_msats(", "fn absolute_fee(", "pub fn fee(",
    "pub fn add_to(", "pub fn subtract_from(")

MATH = """
pub open spec fn cap(value:int)->int {
    if value > u64::MAX { u64::MAX as int } else { value }
}
pub open spec fn fee_math(base:u64, ppm:u64, amount:u64)->int {
    cap(cap((amount as int)*(ppm as int))/1_000_000 + (base as int))
}
"""
GOALS = MATH + """
pub fn exact_fee(fee:&PaymentFee, amount:u64)->(result:Amount)
    ensures result.msats as int == fee_math(fee.base.msats,fee.parts_per_million,amount),
{ fee.fee(amount) }

pub fn no_overflow(fee:&PaymentFee, amount:u64)->(result:Amount)
    requires (amount as int)*(fee.parts_per_million as int) <= u64::MAX,
        (amount as int)*(fee.parts_per_million as int)/1_000_000 + (fee.base.msats as int) <= u64::MAX,
    ensures result.msats as int == (amount as int)*(fee.parts_per_million as int)/1_000_000
        + (fee.base.msats as int),
{ fee.fee(amount) }

pub fn base_floor(fee:&PaymentFee, amount:u64)->(result:Amount)
    ensures result.msats >= fee.base.msats,
{ fee.fee(amount) }

pub fn add_bound(fee:&PaymentFee, amount:u64)->(result:Amount)
    ensures result.msats >= amount,
        result.msats as int == cap((amount as int)+fee_math(fee.base.msats,fee.parts_per_million,amount)),
{ fee.add_to(amount) }

pub fn subtract_bound(fee:&PaymentFee, amount:u64)->(result:Amount)
    ensures result.msats <= amount,
        result.msats as int == if (amount as int) < fee_math(fee.base.msats,fee.parts_per_million,amount)
            {0} else {(amount as int)-fee_math(fee.base.msats,fee.parts_per_million,amount)},
{ fee.subtract_from(amount) }

pub fn zero_fee(amount:u64)->(result:(Amount,Amount,Amount))
    ensures result.0.msats==0, result.1.msats==amount, result.2.msats==amount,
{
    let fee=PaymentFee {base:Amount {msats:0},parts_per_million:0};
    (fee.fee(amount),fee.add_to(amount),fee.subtract_from(amount))
}

pub fn saturation_endpoints()->(result:(Amount,Amount,Amount,Amount))
    ensures result.0.msats==u64::MAX/1_000_000,
        result.1.msats==u64::MAX, result.2.msats==u64::MAX, result.3.msats==0,
{
    let product=PaymentFee {base:Amount {msats:0},parts_per_million:u64::MAX};
    let base=PaymentFee {base:Amount {msats:u64::MAX},parts_per_million:1};
    (product.fee(u64::MAX),base.fee(u64::MAX),base.add_to(1),base.subtract_from(1))
}
"""

MONOTONE = MATH + """
#[verifier::nonlinear]
pub fn component_limit(lower:&PaymentFee, upper:&PaymentFee, amount:u64)->(result:(Amount,Amount))
    requires lower.base.msats<=upper.base.msats,
        lower.parts_per_million<=upper.parts_per_million,
    ensures result.0.msats<=result.1.msats,
{ (lower.fee(amount),upper.fee(amount)) }

#[verifier::nonlinear]
pub fn amount_monotone(fee:&PaymentFee, lower:u64, upper:u64)->(result:(Amount,Amount))
    requires lower<=upper,
    ensures result.0.msats<=result.1.msats,
{ (fee.fee(lower),fee.fee(upper)) }
"""
GOALS += MONOTONE[len(MATH):] + """
pub fn add_unsaturated(fee:&PaymentFee,amount:u64)->(result:Amount)
    requires (amount as int)+fee_math(fee.base.msats,fee.parts_per_million,amount)<=u64::MAX,
    ensures result.msats as int==(amount as int)+fee_math(fee.base.msats,fee.parts_per_million,amount),
{fee.add_to(amount)}

pub fn subtract_recovery(fee:&PaymentFee,amount:u64)->(result:Amount)
    requires fee_math(fee.base.msats,fee.parts_per_million,amount)<=amount,
    ensures (result.msats as int)+fee_math(fee.base.msats,fee.parts_per_million,amount)==amount,
{fee.subtract_from(amount)}
"""
PROVIDER_TOTAL = 28
PROVIDER_BODIES = 17
GOAL_COUNT = 11


def sha(data):
    return hashlib.sha256(data).hexdigest()


def environment(output, solver):
    result = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
    result.update(GIT_OPTIONAL_LOCKS="0", RUSTUP_TOOLCHAIN="1.99.1", CARGO_BUILD_JOBS="1",
        CARGO_HOME=str(output / "cargo-home"), CARGO_TARGET_DIR=str(output / "target"),
        VERUS_Z3_PATH=str(solver), VERUS_INFER_CACHE=str(output / "candidate-cache"))
    return result


def extract(application):
    sources = {name: (application / name).read_bytes() for name in SOURCE_HASHES}
    require({name: sha(data) for name, data in sources.items()} == SOURCE_HASHES,
        "pinned application source bytes changed")
    amount, fees = sources[AMOUNT].decode(), sources[FEES].decode()
    selected = {"Amount": item(amount, "pub struct Amount {"),
        "PaymentFee": item(fees, "pub struct PaymentFee {")}
    for marker in METHODS:
        selected[marker] = item(amount if "from_msats" in marker else fees, marker)
    return selected


def proof_source(selected, goals="", inferred=True):
    annotation = "#[verifier::infer]\n" if inferred else ""
    methods = "\n".join(annotation + selected[marker] for marker in METHODS[1:])
    return ("use vstd::prelude::*;\nverus! {\n#[derive(Copy, Clone)]\n"
        + selected["Amount"] + "\n#[derive(Copy, Clone)]\n" + selected["PaymentFee"]
        + "\nimpl Amount {\n" + annotation + selected[METHODS[0]] + "\n}\n"
        + "impl PaymentFee {\n" + methods + "\n}\n" + goals + "\n}\n")


def validate_case(run, category, artifact, *, count=5, total=None, bodies=None,
        phase=None, domain=True):
    require(category in ("positive", "semantic", "safety", "frontend", "import", "budget"),
        "unknown case classification")
    positive = category == "positive"
    if category == "budget":
        data=json.loads(run["stdout"])
        require(type(run["returncode"]) is int and run["returncode"] != 0
            and data["verification-results"].get("success") is False
            and not data["verification-results"].get("encountered-vir-error")
            and "resource limit" in run["stderr"]
            and not any(marker in run["stderr"].lower() for marker in ("panicked at","timed out")),
            "budget control failed for a different reason")
    else:
        data = validate_run(run, positive, allow_frontend=category in ("frontend", "import"))
    require_publication(data, positive, artifact)
    inference, results = data["verus-infer"], data["verification-results"]
    require(results.get("success",False) is positive and inference.get("crate_success") is positive
        and inference.get("interface_written") is positive, "non-boolean publication state")
    if count is not None:
        require(results.get("is-verifying-entire-crate") is True,
            "selected or partial verification is not accepted")
        require(len(inference["functions"]) == count, "wrong inferred method count")
        if category != "budget":
            require_exact(inference["functions"])
            require(all(function.get("required_checks_passed") is True
                for function in inference["functions"]), "non-boolean inference acceptance")
        require(all(function.get("exported") is (positive
            and not function.get("function","").endswith("::absolute_fee")) for function in inference["functions"]),
            "inferred publication flags disagree")
        if domain:
            require(all(function["contract"]["requires"] in ("true", "(1000000 != 0)")
                for function in inference["functions"]), "native domain was narrowed")
    if positive:
        scope = inference.get("verification_scope", {})
        require(scope.get("native_collection_completed") is True
            and scope.get("complete_selected_scope_verified") is True,
            "positive native scope incomplete")
        require(type(results.get("verified")) is int and results["verified"] == total
            and type(results.get("errors")) is int and results["errors"] == 0,
            "positive proof count differs")
        require(type(scope.get("checked_exec_bodies")) is int
            and scope["checked_exec_bodies"] == bodies
            and type(scope.get("checked_proof_bodies")) is int and scope["checked_proof_bodies"] == 0,
            "positive native body coverage differs")
        if phase is not None:
            require(phase in ("cold", "warm"), "invalid cache phase")
            require(all(function["metrics"].get("cache_policy") in ("candidate_cache","regenerated_state")
                and function["metrics"]["cache"] == ("hit" if phase == "warm"
                    and function["metrics"]["cache_policy"] == "candidate_cache" else "miss")
                for function in inference["functions"]), "cache phase not observed")
            require(phase != "warm" or count == 0
                or any(function["metrics"]["cache"] == "hit" for function in inference["functions"]),
                "warm phase contained no observed candidate cache hit")
            require(all(type(function["metrics"].get("solver_micros")) is int
                and function["metrics"]["solver_micros"] > 0 for function in inference["functions"]),
                "fresh required-check solver work absent")
    if category in ("semantic", "safety"):
        marker = "postcondition not satisfied" if category == "semantic" else "precondition not satisfied"
        other = "precondition not satisfied" if category == "semantic" else "postcondition not satisfied"
        require(inference.get("status") == "caller_proof_failed"
            and results.get("errors") == 1 and marker in run["stderr"]
            and other not in run["stderr"], "failure was not the expected proof obligation")
    if category == "frontend":
        require(results.get("encountered-error") is True
            and results.get("verified", 0) == 0 and results.get("errors", 0) == 0,
            "frontend negative reached semantic verification")
    if category == "import":
        require(results.get("encountered-vir-error") is True
            and "Verus Infer:" in run["stderr"], "not an inference import rejection")
    if category == "budget":
        require(inference.get("status") == "required_proof_incomplete"
            and any(function.get("required_checks_passed") is False
                and any(proof.get("required") is True and proof.get("status") == "incomplete"
                    and proof.get("reason") == "resource_limit" for proof in function["proofs"].values())
                for function in inference["functions"]), "incomplete proof was not rejected")
    return data


def check_certificate(artifact, functions):
    certificate, fingerprint = metadata(artifact.read_bytes())
    require(certificate["schema"] == INTERFACE_SCHEMA and certificate["policy"] == INTERFACE_POLICY,
        "wrong certificate identity")
    require(len(certificate["proofs"]) == functions
        and isinstance(certificate["summaries"],str)
        and re.fullmatch("[0-9a-f]{64}",certificate["summaries"]) is not None,
        "wrong certificate scope")
    require(all(certificate.get(key) is True for key in ("soundness","maximality","exactness")),
        "certificate aggregate checks missing")
    require(all(entry[1].get(key) is True for entry in certificate["proofs"]
        for key in ("soundness", "maximality", "exactness")), "certificate lost mandatory checks")
    require({entry[0]["path"]["segments"][-1] for entry in certificate["proofs"]}
        == {"from_msats","fee","add_to","subtract_from"}, "unexpected public inferred API")
    return certificate, fingerprint


def check_cache(cache):
    entries = {path.name: json.loads(path.read_text()) for path in cache.glob("*.json")}
    require(bool(entries) and {entry["version"] for entry in entries.values()}
        == {CACHE_VERSION}, "wrong or empty candidate cache identity")
    return {name: digest(cache / name) for name in sorted(entries)}


def run_case(command, env, output, verifier, receipt, name, *, category="positive", count=5,
        total=None, bodies=None, phase=None, imports=(), domain=True, extra=()):
    label = name + ("_" + phase if phase else "")
    artifact = output / (name + ".vir")
    native = output / ("lib" + name + ".rlib")
    snapshot = output / (label + ".snapshot.vir")
    positive = category == "positive"
    seeded = None
    if positive:
        require(not snapshot.exists(), "fresh phase snapshot already exists")
        artifact.unlink(missing_ok=True)
        native.unlink(missing_ok=True)
        require(not artifact.exists() and not native.exists(), "fresh outputs not cleared")
    else:
        artifact.write_bytes((output / "provider_warm.snapshot.vir").read_bytes())
        seeded = digest(artifact)
    argv = [str(verifier), str(output / (name + ".rs")), "--crate-type=lib", "--edition=2021",
        "--crate-name", name, "--no-cheating", "--output-json", "--num-threads", "1",
        "--export", str(artifact), *map(str, imports), *map(str,extra)]
    if name == "provider":
        argv += ["--compile", "-o", str(native)]
    code = command(label, argv, env)
    run = dict(returncode=code, stdout=(output / (label + ".stdout")).read_text(),
        stderr=(output / (label + ".stderr")).read_text())
    data = validate_case(run, category, artifact, count=count, total=total, bodies=bodies,
        phase=phase, domain=domain)
    if positive:
        snapshot.write_bytes(artifact.read_bytes())
        if name == "provider":
            require(native.is_file(), "fresh native provider missing")
            shutil.copyfile(native, output / ("libprovider_" + phase + ".rlib"))
    receipt["cases"].append(dict(name=label, category=category, result=data,
        source_sha256=digest(output / (name + ".rs")),
        seeded_export_sha256=seeded, export_sha256=digest(artifact) if artifact.exists() else None))
    return data


def negative_sources(selected):
    exact=GOALS[:GOALS.index("pub fn no_overflow")]
    denominator=dict(selected)
    denominator[METHODS[1]]=denominator[METHODS[1]].replace(".saturating_div(1_000_000)",
        ".saturating_div(999_999)")
    zero=dict(selected)
    zero[METHODS[1]]=zero[METHODS[1]].replace(".saturating_div(1_000_000)",".saturating_div(0)")
    return {
        "wrong_denominator": (proof_source(denominator,exact),"semantic",5,False),
        "zero_denominator": (proof_source(zero,
            "pub fn bad(fee:&PaymentFee,amount:u64)->Amount {fee.fee(amount)}"),"safety",5,False),
        "missing_infer": (proof_source(selected,exact,inferred=False),"semantic",0,True),
        "unsaturated_policy": (proof_source(selected,"""
pub fn bad(fee:&PaymentFee,amount:u64)->(result:Amount)
    ensures result.msats as int==(amount as int)*(fee.parts_per_million as int)/1_000_000
        +(fee.base.msats as int),
{fee.fee(amount)}
"""),"semantic",5,True),
        "strict_add": (proof_source(selected,"""
pub fn bad(fee:&PaymentFee,amount:u64)->(result:Amount) ensures result.msats>amount,
{fee.add_to(amount)}
"""),"semantic",5,True),
        "unsigned_subtract": (proof_source(selected,MATH+"""
pub fn bad(fee:&PaymentFee,amount:u64)->(result:Amount)
    ensures result.msats as int==(amount as int)-fee_math(fee.base.msats,fee.parts_per_million,amount),
{fee.subtract_from(amount)}
"""),"semantic",5,True),
        "signed_type": ("use vstd::prelude::*;\nverus! {\n#[verifier::infer] "
            "pub fn signed(value:i64,denominator:i64)->i64 {value.saturating_div(denominator)}\n}",
            "frontend",None,False),
        "wrong_type": ("use vstd::prelude::*;\nverus! {\n#[verifier::infer] "
            "pub fn wrong(value:u64)->u64 {value.saturating_div(1_000_000u32)}\n}",
            "frontend",None,False),
        "wrong_identity": ("""use vstd::prelude::*;
verus! {
struct Impostor {value:u64}
impl Impostor {fn saturating_div(&self,denominator:u64)->u64 {self.value}}
#[verifier::infer] fn wrong(value:&Impostor,denominator:u64)->u64 {value.saturating_div(denominator)}
}
""","frontend",None,False),
    }


def native_source(selected):
    source = proof_source(selected, inferred=False)
    source = source.removeprefix("use vstd::prelude::*;\nverus! {\n").removesuffix("\n}\n")
    return source + r'''
fn main() {
    let values = [0,1,2,999_999,1_000_000,1_000_001,u32::MAX as u64,
        u64::MAX/1_000_000,u64::MAX/2,u64::MAX-1,u64::MAX];
    let mut checked=0;
    for amount in values { for ppm in values { for base in values {
        let fee=PaymentFee {base:Amount {msats:base},parts_per_million:ppm};
        let maximum=u64::MAX as u128;
        let product=((amount as u128)*(ppm as u128)).min(maximum);
        let expected=(product/1_000_000+(base as u128)).min(maximum) as u64;
        assert_eq!(fee.absolute_fee(amount),expected);
        assert_eq!(fee.fee(amount).msats,expected);
        assert_eq!(fee.add_to(amount).msats,((amount as u128)+(expected as u128)).min(maximum) as u64);
        assert_eq!(fee.subtract_from(amount).msats,if expected>amount {0} else {amount-expected});
        checked+=1;
    }}}
    std::panic::set_hook(Box::new(|_|{}));
    for numerator in 0u8..=255 { for denominator in 1u8..=255 {
        assert_eq!(numerator.saturating_div(denominator),numerator/denominator);
    }}
    let zero=std::hint::black_box(0u64);
    assert!(std::panic::catch_unwind(||u64::MAX.saturating_div(zero)).is_err());
    println!("native_fee_vectors={checked}; exhaustive_u8_nonzero_pairs=65280; u64_zero_panics=true");
}
'''


def freeze(application, output):
    output.mkdir(parents=True, exist_ok=False)
    env = environment(output, Path("/private/tmp/verus-redb-next75.sOFIw9yU/tools75/z3"))
    head = subprocess.check_output(["git", "-C", str(application), "rev-parse", "HEAD"],
        env=env, text=True).strip()
    status = subprocess.check_output(["git", "-C", str(application), "status", "--porcelain",
        "--untracked-files=all"], env=env, text=True)
    require(head == PIN and status == "", "native checkout differs from clean pin")
    selected = extract(application)
    (output / "selected.json").write_text(json.dumps(selected, indent=2) + "\n")
    (output / "source-fidelity.json").write_text(json.dumps(dict(pin=PIN,
        source_sha256=SOURCE_HASHES, item_sha256={name: sha(text.encode())
            for name, text in selected.items()}), indent=2) + "\n")
    (output / "provider.rs").write_text(proof_source(selected))


def replay(args):
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    application, verifier, baseline = (path.resolve() for path in
        (args.application, args.verus, args.baseline_verus))
    root = Path(__file__).resolve().parents[2]
    env = environment(output, args.solver.resolve())
    selected = extract(application)
    receipt = dict(commands=[], cases=[], pin=PIN, source_sha256=SOURCE_HASHES,
        item_sha256={name: sha(text.encode()) for name, text in selected.items()},
        adaptations=["extract exact Amount and PaymentFee struct items; omit external derives/serde attribute",
            "retain Copy/Clone derives, native field types and nested Amount representation",
            "extract exact from_msats, absolute_fee, fee, add_to and subtract_from items",
            "add Infer attributes outside items; wrap in verus and import vstd",
            "append independent, explicit mathematical goals; no application assumptions"],
        engine_identity=list(ENGINE_IDENTITY))
    receipt["environment"]={key:env[key] for key in ("GIT_OPTIONAL_LOCKS","RUSTUP_TOOLCHAIN",
        "CARGO_BUILD_JOBS","CARGO_HOME","CARGO_TARGET_DIR","VERUS_Z3_PATH","VERUS_INFER_CACHE")}
    files = subprocess.check_output(["git", "-C", str(root), "ls-files", "--cached", "--others",
        "--exclude-standard"], env=env, text=True).splitlines()
    native_files = subprocess.check_output(["git", "-C", str(application), "ls-files", "--cached",
        "--others", "--exclude-standard"], env=env, text=True).splitlines()
    compiler = Path(subprocess.check_output(["rustup", "which", "--toolchain", "1.99.1", "rustc"],
        env=env, text=True).strip())
    paths = [root / path for path in files if (root / path).is_file()]
    paths += [application / path for path in native_files if (application / path).is_file()]
    paths += tool_artifacts(verifier, args.solver.resolve())
    paths += [path for path in baseline.parent.iterdir() if path.is_file()]
    paths += compiler_artifacts(compiler)
    guard = ArtifactGuard(paths)
    receipt["artifacts"] = guard.initial
    receipt["runtime_hashes"] = {path.name: digest(path) for path in verifier.parent.iterdir()
        if path.is_file()}
    command = LoggedCommands(output, root / "source", receipt["commands"], timeout=args.timeout, guard=guard)
    with replay_receipt(output, receipt):
        head = subprocess.check_output(["git", "-C", str(application), "rev-parse", "HEAD"], env=env, text=True).strip()
        clean = subprocess.check_output(["git", "-C", str(application), "status", "--porcelain",
            "--untracked-files=all"], env=env, text=True)
        require(head == PIN and clean == "", "native pin is not clean")
        (output / "provider.rs").write_text(proof_source(selected, GOALS))
        require(all(item((output / "provider.rs").read_text(),marker)==selected[marker]
            for marker in METHODS), "native method bytes changed during scaffolding")
        baseline_artifact = output / "baseline.vir"
        baseline_command = [str(baseline), str(output / "provider.rs"), "--crate-type=lib",
            "--edition=2021", "--crate-name", "provider", "--no-cheating", "--output-json",
            "--num-threads", "1", "--export", str(baseline_artifact)]
        code = command("baseline", baseline_command, env)
        run = dict(returncode=code, stdout=(output / "baseline.stdout").read_text(),
            stderr=(output / "baseline.stderr").read_text())
        validate_case(run, "frontend", baseline_artifact, count=None)
        require("saturating_div" in run["stderr"] and "not supported" in run["stderr"],
            "baseline failed for a different reason")
        for phase in ("cold", "warm"):
            data = run_case(command, env, output, verifier, receipt, "provider",
                total=PROVIDER_TOTAL, bodies=PROVIDER_BODIES, phase=phase)
            require({function["function"].split("::")[-1] for function in data["verus-infer"]["functions"]}
                =={"from_msats","absolute_fee","fee","add_to","subtract_from"},"native inference method scope differs")
            if phase == "cold":
                cold = data
            else:
                require([function["contract"] for function in cold["verus-infer"]["functions"]]
                    == [function["contract"] for function in data["verus-infer"]["functions"]],
                    "warm contracts differ")
        phases = [entry for entry in receipt["commands"] if entry["label"] in ("provider_cold", "provider_warm")]
        require(len(phases) == 2 and phases[0]["argv"] == phases[1]["argv"],
            "warm compiler arguments changed")
        certificate, fingerprint = check_certificate(output / "provider.vir", 4)
        receipt["provider_certificate"] = certificate
        receipt["provider_fingerprint"] = fingerprint
        receipt["cache_entry_sha256"] = check_cache(output / "candidate-cache")
        imported = ArtifactGuard([output / "provider.vir", output / "libprovider.rlib"])
        imports = ["--import", "provider=" + str(output / "provider.vir"),
            "--extern", "provider=" + str(output / "libprovider.rlib")]
        (output / "client.rs").write_text("use vstd::prelude::*;\nuse provider::{Amount,PaymentFee};\nverus! {\n"
            + GOALS + "\n}\n")
        for phase in ("cold", "warm"):
            imported.check()
            data = run_case(command, env, output, verifier, receipt, "client", count=0,
                total=GOAL_COUNT, bodies=GOAL_COUNT, phase=phase, imports=imports)
            require(data["verus-infer"]["imported_inference_dependencies"]
                == certificate_import(certificate, fingerprint), "client imported a different certificate")
            imported.check()
        receipt["import_artifacts"] = imported.initial
        for name,(source,category,count,domain) in negative_sources(selected).items():
            (output / (name+".rs")).write_text(source)
            run_case(command,env,output,verifier,receipt,name,category=category,count=count,domain=domain,
                imports=imports if name=="missing_infer" else ())
        (output / "budget.rs").write_text(proof_source(selected,GOALS))
        run_case(command,env,output,verifier,receipt,"budget",category="budget",extra=("--rlimit","0.000001"))
        edits = {
            "old_policy":lambda certificate:certificate.update(policy="candidate-bound-evidence-v69"),
            "old_schema":lambda certificate:certificate.update(schema=37),
            "missing_proof":lambda certificate:certificate["proofs"].pop(),
            "failed_exactness":lambda certificate:certificate["proofs"][0][1].update(exactness=False),
            "artifact_binding":lambda certificate:certificate.update(summaries="0"*64),
        }
        def change_native_model(certificate):
            matches=[entry for entry in certificate["trusted_models"]
                if entry[0]["path"]["segments"]==["num","impl&%9","saturating_div"]]
            require(len(matches)==1,"native division trust binding not found")
            matches[0][1]="0"*64
        edits["native_binding"]=change_native_model
        for name,edit in edits.items():
            forged=output/(name+".import.vir")
            forged.write_bytes(replace_metadata((output/"provider.vir").read_bytes(),edit))
            (output/(name+".rs")).write_bytes((output/"client.rs").read_bytes())
            run_case(command,env,output,verifier,receipt,name,category="import",count=None,
                imports=("--import","provider="+str(forged),"--extern",
                    "provider="+str(output/"libprovider.rlib")))
        (output/"native.rs").write_text(native_source(selected))
        for profile,optimization in (("debug","0"),("checked_release","3")):
            binary=output/("native_"+profile)
            require(command("native_build_"+profile,[str(compiler),str(output/"native.rs"),
                "--edition=2021","-C","opt-level="+optimization,"-C","overflow-checks=yes",
                "-o",str(binary)],env)==0,"native compilation failed")
            require(command("native_run_"+profile,[str(binary)],env)==0,"native arithmetic vectors failed")
            require((output/("native_run_"+profile+".stdout")).read_text().strip()
                =="native_fee_vectors=1331; exhaustive_u8_nonzero_pairs=65280; u64_zero_panics=true",
                "native coverage report differs")
        receipt["cache_entry_sha256"] = check_cache(output / "candidate-cache")
        failures=[case for case in receipt["cases"] if case["category"]!="positive"]
        require(len(receipt["cases"])==20 and len(failures)==16
            and all(re.fullmatch("[0-9a-f]{64}",case.get("seeded_export_sha256") or "")
                and case["export_sha256"] is None for case in failures),"failure export was not seeded or survived")
        require(subprocess.check_output(["git","-C",str(application),"status","--porcelain",
            "--untracked-files=all"],env=env,text=True)=="","native checkout became dirty")
        require(subprocess.check_output(["git","-C",str(application),"rev-parse","HEAD"],env=env,text=True).strip()
            ==PIN,"native pin changed")
        require(subprocess.check_output(["git","-C",str(root),"rev-parse","HEAD"],env=env,text=True).strip()
            =="3b05c1912b9d7314ad00c832c4065f4362562e02","engine HEAD changed")
        imported.check()
        guard.check()
    return receipt


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--application", required=True, type=Path)
    parser.add_argument("--freeze", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--verus", type=Path)
    parser.add_argument("--baseline-verus", type=Path)
    parser.add_argument("--solver", type=Path)
    parser.add_argument("--timeout", type=positive_timeout, default=120)
    args = parser.parse_args()
    if args.freeze is not None:
        freeze(args.application.resolve(), args.freeze.resolve())
    else:
        require(all(value is not None for value in (args.output, args.verus, args.baseline_verus, args.solver)),
            "replay requires output, verifier, baseline verifier and solver")
        replay(args)
