"""Pinned native fee helpers and an explicitly authored validation-boundary composition."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import time

from check_fedimint_native import item
from check_fedimint_paymentfee76 import GOALS as ARITHMETIC_GOALS
from infer_experiment import require
from infer_transfer_receipts import metadata
from check_fedimint_full_request import replace_metadata
from infer_eval_support import ArtifactGuard
from infer_transfer_identity import CACHE_VERSION as CACHE, INTERFACE_POLICY as POLICY

PIN = "b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236"
SOURCE_HASHES = {
    "fedimint-core/src/amount.rs": "9b1095eb5ed2f2500887bc7875d28e4cf46177a6d860680a23e08bef7b253dd1",
    "modules/fedimint-lnv2-common/src/gateway_api.rs": "a3eda308673486e831abbf74c73adc6043fe28a9389e74773f804e70c0f551b6",
    "gateway/fedimint-gateway-server/src/config.rs": "3f98b031ecb721db3911c023b91a723815378accb22e59a7395c0921207f848c",
    "gateway/fedimint-gateway-server/src/lib.rs": "2f15bfa050213c3fd041b93d5736d83882ad814ac36472eb7df64688824ca831",
}
METHODS = {
    "Amount::from_msats": ("amount", "pub const fn from_msats("),
    "Amount::from_sats": ("amount", "pub const fn from_sats("),
    "Amount::checked_add": ("amount", "pub fn checked_add("),
    "PaymentFee::checked_add": ("fees", "pub fn checked_add("),
    "PaymentFee::is_within": ("fees", "pub fn is_within("),
    "PaymentFee::absolute_fee": ("fees", "fn absolute_fee("),
    "PaymentFee::fee": ("fees", "pub fn fee("),
    "PaymentFee::add_to": ("fees", "pub fn add_to("),
    "PaymentFee::subtract_from": ("fees", "pub fn subtract_from("),
}
FROZEN_HASHES = {
    "Amount::checked_add": "87524f34513de2ac447c5bbd0f1f62cefff41732b3a896aadfb8bf974d8a1477",
    "PaymentFee::checked_add": "6ad96dedd30054c5632c26b1fe5a5704caa0034d09946833bb34ab6945081aff",
    "PaymentFee::is_within": "075bc37005bd569abcefde7a9493774e8cc70c470970aaa37b8096e3ba8844ed",
}
ITEM_HASHES = dict(FROZEN_HASHES, **{
    "Amount::from_msats": "8e33d1b2ea62a0ea4269cba7a788b17795d8116f9e0061120272d611348989f2",
    "Amount::from_sats": "9d023f1416ce16e4e73dd93cc28029075336e55f8d411e8441adf80fd9e00f52",
    "PaymentFee::absolute_fee": "b1f6426cc31ebf585f2a898d4593338f47b89c9cf64b5dcfe643975efb430d2d",
    "PaymentFee::fee": "7ddbad236c417465b7bb056c4d7583b2f7b270f587f08292ca067a9af6a91784",
    "PaymentFee::add_to": "d17d2a5be5fe51aeb5e6ab93fadb03c7a46127930103b1a5d0337aefc221c188",
    "PaymentFee::subtract_from": "01cc8234518b4f25912ca977ab225e19d06d89f92ba67654713c0f165a9be633",
    "Amount": "ef476dd7b56e7ef92905c5c6fac2de03b7f3f9a32dd9b3176825dddd51f14ce5",
    "PaymentFee": "9265fa026696b79cecc599a87d0920664cbf4a0a1c6708396c0bb6aa260ec644",
    "SEND_FEE_LIMIT": "b66e8b33d21bc4700867010589ce03d71e2ab9499ee578cecf0f42d0d37b6dde",
    "RECEIVE_FEE_LIMIT": "2f121adb17910f3378f5698cd0ed2977565a3e2e77fec0d7d63f18e11f0d8922",
})
NATIVE_TARGETS = set(FROZEN_HASHES)
EXPECTED_INFERRED = set(METHODS) | {"Amount::eq"}
PROVIDER_DERIVED = {"Amount::clone", "Amount::cmp", "Amount::eq", "Amount::partial_cmp", "PaymentFee::clone"}
PROVIDER_AUTHORED = {
    "accepted_fee_bound", "add_bound", "add_unsaturated", "amount_monotone", "amount_sum_intent",
    "base_floor", "checked_sum_intent", "component_limit", "exact_fee", "native_absolute_bound",
    "native_gateway_boundary", "native_limit_equality", "native_limit_intent", "no_overflow",
    "saturation_endpoints", "subtract_bound", "subtract_recovery", "validate_fee_pair", "zero_fee",
}
PROVIDER_SPECS = {
    "Amount::cmp_spec", "Amount::obeys_cmp_spec", "Amount::eq_spec", "Amount::obeys_eq_spec",
    "Amount::obeys_partial_cmp_spec", "Amount::partial_cmp_spec", "Amount::__verus_infer_derived_cmp",
    "Amount::__verus_infer_derived_eq", "Amount::__verus_infer_derived_partial_cmp", "cap", "decision_math",
    "fee_math", "receive_limit_math", "send_limit_math",
}
PROVIDER_CONST_METHODS = {"Amount::from_msats", "Amount::from_sats"}
PROVIDER_PROXIES = {
    "Amount::VERUS_UNERASED_PROXY__from_msats", "Amount::VERUS_UNERASED_PROXY__from_sats",
    "PaymentFee::VERUS_UNERASED_PROXY__RECEIVE_FEE_LIMIT", "PaymentFee::VERUS_UNERASED_PROXY__SEND_FEE_LIMIT",
}
CLIENT_CHECKED = {"all_limits", "imported_amount", "imported_validate_fee_pair", "real_imported_boundary"}
REGIONS = {
    "gateway/fedimint-gateway-server/src/lib.rs": (2627, 2648,
        "0bbbf702c708e9dc5065f0d5e03723e48afd1f06d900ad192f0f85bdb5e54e71"),
    "gateway/fedimint-gateway-server/src/config.rs": (187, 207,
        "155f60d6877d4f6764b22686032d7b009638ec825be548fa0085e3f6b6c1aeb2"),
}

COMPOSITION = r"""
pub open spec fn send_limit_math()->PaymentFee {
    PaymentFee {base:Amount {msats:100_000},parts_per_million:15_000}
}
pub open spec fn receive_limit_math()->PaymentFee {
    PaymentFee {base:Amount {msats:50_000},parts_per_million:5_000}
}
pub fn amount_sum_intent(left:Amount,right:Amount)->(result:Option<Amount>)
    ensures match result {
        None => (left.msats as int)+(right.msats as int)>u64::MAX,
        Some(sum) => sum.msats as int==(left.msats as int)+(right.msats as int),
    },
{ left.checked_add(right) }
pub open spec fn decision_math(lightning:PaymentFee,transaction:PaymentFee,
    send_limit:PaymentFee,receive_limit:PaymentFee)->u8 {
    if (lightning.base.msats as int)+(transaction.base.msats as int)>u64::MAX ||
        (lightning.parts_per_million as int)+(transaction.parts_per_million as int)>u64::MAX {0}
    else if (lightning.base.msats as int)+(transaction.base.msats as int)>send_limit.base.msats ||
        (lightning.parts_per_million as int)+(transaction.parts_per_million as int)>send_limit.parts_per_million {1}
    else if transaction.base.msats>receive_limit.base.msats ||
        transaction.parts_per_million>receive_limit.parts_per_million {2}
    else {3}
}

pub fn checked_sum_intent(left:PaymentFee,right:PaymentFee)->(result:Option<PaymentFee>)
    ensures match result {
        None => (left.base.msats as int)+(right.base.msats as int)>u64::MAX ||
            (left.parts_per_million as int)+(right.parts_per_million as int)>u64::MAX,
        Some(sum) => sum.base.msats as int==(left.base.msats as int)+(right.base.msats as int) &&
            sum.parts_per_million as int==(left.parts_per_million as int)+(right.parts_per_million as int),
    },
{ left.checked_add(right) }

pub fn native_limit_intent(value:&PaymentFee,limit:&PaymentFee)->(result:bool)
    ensures result==(value.base.msats<=limit.base.msats && value.parts_per_million<=limit.parts_per_million),
{ value.is_within(limit) }

pub fn native_limit_equality(value:&PaymentFee)->(result:bool)
    ensures result,
{ value.is_within(value) }

pub fn validate_fee_pair(lightning:PaymentFee,transaction:PaymentFee,
    send_limit:&PaymentFee,receive_limit:&PaymentFee)->(result:(u8,Option<PaymentFee>))
    ensures result.0==decision_math(lightning,transaction,*send_limit,*receive_limit),
        (result.0==3)==(result.1 is Some),
        match result.1 {
            None => true,
            Some(sum) => sum.base.msats as int==(lightning.base.msats as int)+(transaction.base.msats as int) &&
                sum.parts_per_million as int==(lightning.parts_per_million as int)+(transaction.parts_per_million as int) &&
                sum.base.msats<=send_limit.base.msats && sum.parts_per_million<=send_limit.parts_per_million &&
                transaction.base.msats<=receive_limit.base.msats && transaction.parts_per_million<=receive_limit.parts_per_million,
        },
{
    let send_fees=match lightning.checked_add(transaction) {
        None => return (0,None),
        Some(sum) => sum,
    };
    if !send_fees.is_within(send_limit) { return (1,None); }
    if !transaction.is_within(receive_limit) { return (2,None); }
    (3,Some(send_fees))
}

pub fn native_absolute_bound(lower:&PaymentFee,upper:&PaymentFee,amount:u64)->(result:(u64,u64))
    requires lower.base.msats<=upper.base.msats, lower.parts_per_million<=upper.parts_per_million,
    ensures result.0<=result.1,
        result.0 as int==fee_math(lower.base.msats,lower.parts_per_million,amount),
        result.1 as int==fee_math(upper.base.msats,upper.parts_per_million,amount),
{
    let semantic_bound=component_limit(lower,upper,amount);
    (lower.absolute_fee(amount),upper.absolute_fee(amount))
}

pub fn accepted_fee_bound(lightning:PaymentFee,transaction:PaymentFee,
    send_limit:&PaymentFee,receive_limit:&PaymentFee,amount:u64)->(result:Option<(u64,u64,u64,u64)>)
    ensures (result is Some)==(decision_math(lightning,transaction,*send_limit,*receive_limit)==3),
        match result {None => true,Some(fees) => fees.0<=fees.1 && fees.2<=fees.3},
{
    let decision=validate_fee_pair(lightning,transaction,send_limit,receive_limit);
    match decision.1 {
        None => None,
        Some(sum) => {
            let send=native_absolute_bound(&sum,send_limit,amount);
            let receive=native_absolute_bound(&transaction,receive_limit,amount);
            Some((send.0,send.1,receive.0,receive.1))
        },
    }
}

pub fn native_gateway_boundary(lightning:PaymentFee,transaction:PaymentFee,amount:u64)
    ->(result:Option<(u64,u64,u64,u64)>)
    ensures (result is Some)==(decision_math(lightning,transaction,
        send_limit_math(),receive_limit_math())==3),
        match result {None => true,Some(fees) => fees.0<=fees.1 && fees.2<=fees.3},
{ accepted_fee_bound(lightning,transaction,&PaymentFee::SEND_FEE_LIMIT,&PaymentFee::RECEIVE_FEE_LIMIT,amount) }
"""

def sha(data):
    return hashlib.sha256(data).hexdigest()

def extract(application):
    sources = {name: (application / name).read_bytes() for name in SOURCE_HASHES}
    require({name: sha(data) for name, data in sources.items()} == SOURCE_HASHES, "native_source_guard")
    for name, (start, end, fingerprint) in REGIONS.items():
        require(sha(b"".join(sources[name].splitlines(keepends=True)[start-1:end])) == fingerprint,
            "caller_region_guard")
    texts = dict(amount=sources["fedimint-core/src/amount.rs"].decode(),
        fees=sources["modules/fedimint-lnv2-common/src/gateway_api.rs"].decode())
    selected = {name: item(texts[source], marker) for name, (source, marker) in METHODS.items()}
    selected.update(Amount=item(texts["amount"], "pub struct Amount {"),
        PaymentFee=item(texts["fees"], "pub struct PaymentFee {"))
    for name in ["SEND_FEE_LIMIT", "RECEIVE_FEE_LIMIT"]:
        selected[name] = re.search(r"pub const " + name + r": PaymentFee = PaymentFee \{.*?\};", texts["fees"], re.S).group()
    require({name: sha(selected[name].encode()) for name in FROZEN_HASHES} == FROZEN_HASHES, "native_item_guard")
    return selected, sources

def provider_source(selected, goals=COMPOSITION):
    amount_methods = "\n".join("#[verus_infer]\n" + selected[name] for name in METHODS if name.startswith("Amount::"))
    fee_methods = "\n".join("#[verus_infer]\n" + selected[name] for name in METHODS if name.startswith("PaymentFee::"))
    arithmetic = ARITHMETIC_GOALS.replace(
        "ensures result.0.msats<=result.1.msats,",
        "ensures result.0.msats<=result.1.msats,\n"
        "result.0.msats as int==fee_math(lower.base.msats,lower.parts_per_million,amount),\n"
        "result.1.msats as int==fee_math(upper.base.msats,upper.parts_per_million,amount),", 1)
    return ("use vstd::prelude::*;\n#[verus_verify(infer_derive(PartialEq,PartialOrd,Ord))]\n"
        "#[derive(Clone,Copy,Eq,PartialEq,Ord,PartialOrd)]\n" + selected["Amount"]
        + "\n#[verus_verify]\n#[derive(Copy,Clone)]\n" + selected["PaymentFee"]
        + "\n#[verus_verify]\nimpl Amount {\n" + amount_methods + "\n}\n"
        + "#[verus_verify]\nimpl PaymentFee {\n"
        + "#[verus_spec(ensures Self::SEND_FEE_LIMIT.base.msats==100_000, Self::SEND_FEE_LIMIT.parts_per_million==15_000)]\n"
        + selected["SEND_FEE_LIMIT"] + "\n"
        + "#[verus_spec(ensures Self::RECEIVE_FEE_LIMIT.base.msats==50_000, Self::RECEIVE_FEE_LIMIT.parts_per_million==5_000)]\n"
        + selected["RECEIVE_FEE_LIMIT"] + "\n" + fee_methods + "\n}\n"
        + "verus! {\n" + arithmetic + goals + "\n}\n")

def client_source():
    return r"""use vstd::prelude::*;
use fee_provider::{Amount,PaymentFee,decision_math,native_gateway_boundary,accepted_fee_bound,send_limit_math,receive_limit_math};
verus! {
pub fn real_imported_boundary(lightning:PaymentFee,transaction:PaymentFee,amount:u64)
    ->(result:Option<(u64,u64,u64,u64)>)
    ensures (result is Some)==(decision_math(lightning,transaction,send_limit_math(),receive_limit_math())==3),
        match result {None => true,Some(fees) => fees.0<=fees.1 && fees.2<=fees.3},
{ native_gateway_boundary(lightning,transaction,amount) }
pub fn all_limits(lightning:PaymentFee,transaction:PaymentFee,send:&PaymentFee,receive:&PaymentFee,amount:u64)
    ->(result:Option<(u64,u64,u64,u64)>)
    ensures (result is Some)==(decision_math(lightning,transaction,*send,*receive)==3),
        match result {None => true,Some(fees) => fees.0<=fees.1 && fees.2<=fees.3},
{ accepted_fee_bound(lightning,transaction,send,receive,amount) }
""" + COMPOSITION[COMPOSITION.index("pub fn validate_fee_pair"):COMPOSITION.index("pub fn native_absolute_bound")].replace(
        "validate_fee_pair", "imported_validate_fee_pair") + r"""
pub fn imported_amount(left:Amount,right:Amount)->(result:Option<Amount>)
    ensures match result {None => (left.msats as int)+(right.msats as int)>u64::MAX,
        Some(sum) => sum.msats as int==(left.msats as int)+(right.msats as int)},
{left.checked_add(right)}
}
"""


def guard_publication(outcome):
    data, artifact = outcome["data"], outcome["artifact"]
    positive = outcome["category"] == "positive"
    inference, result = data["verus-infer"], data["verification-results"]
    require(type(outcome["returncode"]) is int and (outcome["returncode"] == 0) is positive
        and result.get("success") is positive and inference.get("crate_success") is positive
        and inference.get("interface_written") is positive, "publication_state_guard")
    require(artifact.is_file() is positive, "publication_artifact_guard")
    require(all(function.get("exported") is (positive and not function["function"].endswith("::absolute_fee"))
        for function in inference.get("functions", [])), "publication_visibility_guard")


def guard_native(outcome):
    selected = outcome["selected"]
    require({name: sha(text.encode()) for name, text in selected.items()} == ITEM_HASHES
        and outcome["source"] == provider_source(selected), "native_body_guard")


def guard_domains(outcome):
    functions = outcome["data"]["verus-infer"]["functions"]
    require(all(function["contract"]["requires"] == "true" for function in functions
        if any(function["function"].endswith("::" + name) for name in NATIVE_TARGETS)),
        "full_domain_guard")


def guard_required(outcome):
    functions = outcome["data"]["verus-infer"].get("functions", [])
    require(len(functions) == outcome["count"] and all(function.get("mode") == "exact"
        and function.get("required_checks_passed") is True
        and all(function.get("proofs", {}).get(name, {}).get("status") == "proved"
            and function["proofs"][name].get("required") is True
            for name in ("soundness", "maximality", "exactness")) for function in functions),
        "required_proofs_guard")


def expected_native_scope(crate):
    require(crate in ("fee_provider", "fee_client"), "native_scope_crate_guard")
    expected = {}
    def add(name, status, selected=True, derived=False, inferred=False, ordinary=False, prefix=True):
        expected[(crate + "::" if prefix else "") + name] = dict(body_status=status, selected=selected,
            automatically_derived=derived, inference_candidate=inferred, ordinary_contract=ordinary)
    if crate == "fee_client":
        for name in CLIENT_CHECKED:
            add(name, "checked_exec_body", ordinary=True)
        return expected
    checked = PROVIDER_DERIVED | PROVIDER_AUTHORED | (set(METHODS) - PROVIDER_CONST_METHODS)
    for name in checked:
        add(name, "checked_exec_body", derived=name in PROVIDER_DERIVED, inferred=name in EXPECTED_INFERRED,
            ordinary=name in PROVIDER_AUTHORED or name in ("Amount::clone", "PaymentFee::clone"))
    for name in PROVIDER_SPECS:
        add(name, "specification_definition_or_declaration")
    for name in PROVIDER_CONST_METHODS:
        add(name, "outside_selected_scope", selected=False, inferred=True)
    for name in PROVIDER_PROXIES:
        add(name, "selected_item_without_translated_body", prefix=False)
    add("<Amount as std::cmp::Eq>::assert_fields_are_eq", "outside_selected_scope",
        selected=False, derived=True, prefix=False)
    return expected


def guard_scope(outcome):
    data = outcome["data"]
    scope, result = data["verus-infer"]["verification_scope"], data["verification-results"]
    require(result.get("is-verifying-entire-crate") is True
        and type(result.get("verified")) is int and result["verified"] == outcome["verified"]
        and type(result.get("errors")) is int and result["errors"] == 0
        and scope.get("native_collection_completed") is True
        and scope.get("complete_selected_scope_verified") is True
        and type(scope.get("checked_exec_bodies")) is int
        and type(scope.get("checked_proof_bodies")) is int,
        "native_scope_guard")
    expected = expected_native_scope(outcome["crate"])
    entries = scope.get("native_functions")
    require(isinstance(entries, list) and len(entries) == len(expected)
        and all(isinstance(entry, dict) and isinstance(entry.get("function"), str) for entry in entries),
        "native_body_set_guard")
    names = [entry["function"] for entry in entries]
    require(set(names) == set(expected), "native_body_set_guard")
    for entry in entries:
        fields = expected[entry["function"]]
        require(entry.get("body_status") == fields["body_status"]
            and all(entry.get(name) is fields[name] for name in
                ("selected", "automatically_derived", "inference_candidate", "ordinary_contract")),
            "native_body_metadata_guard")
    checked_exec = sum(entry["selected"] is True and entry["body_status"] == "checked_exec_body" for entry in entries)
    checked_proof = sum(entry["selected"] is True and entry["body_status"] == "checked_proof_body" for entry in entries)
    require(scope["checked_exec_bodies"] == checked_exec == outcome["bodies"]
        and scope["checked_proof_bodies"] == checked_proof == 0, "native_scope_count_guard")
    if outcome["count"]:
        names = {function["function"].removeprefix(outcome["crate"] + "::")
            for function in data["verus-infer"]["functions"]}
        require(names == EXPECTED_INFERRED, "native_inferred_scope_guard")
        for name in ("Amount::cmp", "Amount::partial_cmp", "Amount::eq"):
            matches = [entry for entry in scope["native_functions"]
                if entry["function"] == outcome["crate"] + "::" + name]
            require(len(matches) == 1 and matches[0].get("automatically_derived") is True
                and matches[0].get("selected") is True
                and matches[0].get("body_status") == "checked_exec_body", "actual_derived_body_guard")


def guard_work(outcome):
    functions = outcome["data"]["verus-infer"]["functions"]
    phase = outcome["phase"]
    require(all(type(function["metrics"].get("solver_micros")) is int
        and function["metrics"]["solver_micros"] > 0 for function in functions), "fresh_work_guard")
    require(all(function["metrics"].get("cache_policy") in ("candidate_cache", "regenerated_state")
        and function["metrics"].get("cache") == ("hit" if phase == "warm"
            and function["metrics"]["cache_policy"] == "candidate_cache" else "miss")
        for function in functions)
        and (phase != "warm" or not functions
            or any(function["metrics"]["cache"] == "hit" for function in functions)), "cache_phase_guard")


def guard_import(outcome):
    dependencies = outcome["data"]["verus-infer"].get("imported_inference_dependencies", [])
    expected = outcome.get("dependency")
    require(bool(dependencies) is bool(expected), "import_presence_guard")
    if expected:
        require(len(dependencies) == 1, "import_count_guard")
        dependency = dependencies[0]
        observed, fingerprint = metadata((expected["folder"] / "provider.vir").read_bytes())
        require(dependency.get("fingerprint") == expected["fingerprint"] == fingerprint
            and observed.get("schema") == 38 and observed.get("policy") == POLICY
            and sha((expected["folder"] / "libprovider.rlib").read_bytes()) == expected["native_sha256"],
            "import_identity_guard")


def guard_classification(outcome):
    category, data, stderr = outcome["category"], outcome["data"], outcome["stderr"]
    result, inference = data["verification-results"], data["verus-infer"]
    require(not any(marker in stderr.lower() for marker in ("panicked at", "out of memory", "timed out")),
        "abnormal_failure_guard")
    if category in ("semantic", "safety"):
        marker = "postcondition not satisfied" if category == "semantic" else "precondition not satisfied"
        require(marker in stderr and result.get("encountered-vir-error") is False
            and result.get("errors", 0) > 0 and inference.get("status") == "caller_proof_failed",
            "proof_classification_guard")
    elif category == "frontend":
        require(result.get("encountered-error") is True and result.get("errors", 0) == 0
            and result.get("verified", 0) == 0, "frontend_classification_guard")
    elif category == "import":
        require(result.get("encountered-vir-error") is True and "Verus Infer:" in stderr,
            "import_classification_guard")
    elif category == "budget":
        require("resource limit" in stderr and inference.get("status") == "required_proof_incomplete",
            "resource_classification_guard")
    else:
        require(category == "positive", "unknown_classification_guard")
        require(result.get("encountered-error") is False and result.get("encountered-vir-error") is False,
            "positive_error_state_guard")
        require(type(inference.get("schema_version")) is int and inference["schema_version"] == 1,
            "positive_report_schema_guard")
    require(outcome.get("diagnostic", "") in stderr, "diagnostic_guard")


def validate_outcome(outcome):
    guard_classification(outcome)
    guard_publication(outcome)
    if outcome["category"] == "positive":
        if outcome["count"]:
            guard_native(outcome)
        guard_required(outcome)
        guard_domains(outcome)
        guard_scope(outcome)
        guard_work(outcome)
        guard_import(outcome)
    elif outcome["category"] in ("semantic", "safety"):
        guard_required(outcome)
    return outcome["data"]


def certificate(artifact):
    value, fingerprint = metadata(artifact.read_bytes())
    require(value.get("schema") == 38 and value.get("policy") == POLICY,
        "certificate_identity_guard")
    require(all(value.get(name) is True for name in ("soundness", "maximality", "exactness"))
        and len(value["proofs"]) == 9
        and all(proofs.get(name) is True for _, proofs in value["proofs"]
            for name in ("soundness", "maximality", "exactness")), "certificate_proofs_guard")
    return value, fingerprint


def environment(output, sysroot, solver):
    for name in ("home", "tmp", "cache", "cargo-home", "target"):
        (output / name).mkdir(parents=True, exist_ok=True)
    toolchains = output / "rustup-home/toolchains"
    toolchains.mkdir(parents=True, exist_ok=True)
    (toolchains / sysroot.name).symlink_to(sysroot, target_is_directory=True)
    return dict(PATH="/Users/tommyvolk/.cargo/bin:/usr/bin:/bin", HOME=str(output / "home"), TMPDIR=str(output / "tmp"),
        XDG_CACHE_HOME=str(output / "cache"), CARGO_HOME=str(output / "cargo-home"),
        CARGO_TARGET_DIR=str(output / "target"), CARGO_NET_OFFLINE="true",
        GIT_OPTIONAL_LOCKS="0", RUSTUP_TOOLCHAIN="1.99.1", RUSTUP_HOME=str(output / "rustup-home"), RUST_MIN_STACK="20971520",
        VERUS_Z3_PATH=str(solver), VERUS_INFER_CACHE=str(output / "candidate-cache"),
        DYLD_LIBRARY_PATH=str(sysroot / "lib"), PYTHONDONTWRITEBYTECODE="1")


def invoke(output, runtime, sysroot, env, label, source, *, crate="fee_provider", category="positive",
        phase="cold", count=10, verified=55, bodies=31, selected=None, dependency=None,
        seed=None, extra=(), diagnostic="", unit=None):
    folder = output / (unit or label)
    folder.mkdir(exist_ok=phase == "warm")
    path, artifact = folder / "input.rs", folder / "provider.vir"
    path.write_text(source)
    if seed is not None:
        artifact.write_bytes(seed)
    else:
        artifact.unlink(missing_ok=True)
    (folder / "libprovider.rlib").unlink(missing_ok=True)
    argv = [str(runtime / "verus"), str(path), "--sysroot", str(sysroot), "--crate-type=lib",
        "--crate-name", crate, "--edition=2021", "--no-cheating", "--output-json", "--num-threads", "1",
        "--export", str(artifact), "--compile", "-o", str(folder / "libprovider.rlib")]
    if dependency:
        argv += ["--extern", "fee_provider=" + str(dependency["folder"] / "libprovider.rlib"),
            "--import", "fee_provider=" + str(dependency["folder"] / "provider.vir"),
            "-L", "dependency=" + str(dependency["folder"])]
    started = time.time()
    process = subprocess.run(argv + list(extra), cwd=folder, env=env, capture_output=True, timeout=180)
    for name in ("stdout", "stderr"):
        (folder / (label + "." + name)).write_bytes(getattr(process, name))
    command = dict(argv=argv + list(extra), cwd=str(folder), environment=env, returncode=process.returncode,
        elapsed_seconds=time.time() - started,
        **{name + "_sha256": sha(getattr(process, name)) for name in ("stdout", "stderr")})
    (folder / (label + ".command.json")).write_text(json.dumps(command, indent=2, sort_keys=True) + "\n")
    outcome = dict(data=json.loads(process.stdout), returncode=process.returncode,
        stderr=process.stderr.decode(), category=category, phase=phase, count=count,
        verified=verified, bodies=bodies, crate=crate, artifact=artifact, selected=selected,
        source=source, dependency=dependency, diagnostic=diagnostic)
    try:
        validate_outcome(outcome)
    except Exception:
        print(label, process.returncode, process.stderr.decode()[-2000:], flush=True)
        raise
    print(label, category, process.returncode, flush=True)
    if artifact.exists():
        (folder / (label + ".snapshot.vir")).write_bytes(artifact.read_bytes())
    return dict(name=label, category=category, folder=folder, artifact=artifact,
        source_sha256=sha(source.encode()), data=outcome["data"], command=command,
        seeded=seed is not None, export_present=artifact.exists())


def controls(selected):
    maximum = "PaymentFee{base:Amount{msats:u64::MAX},parts_per_million:u64::MAX}"
    zero = "PaymentFee{base:Amount{msats:0},parts_per_million:0}"
    statements = {
        "base_overflow": f"let left={maximum}; let right=PaymentFee{{base:Amount{{msats:1}},parts_per_million:0}}; let result=left.checked_add(right); result is Some",
        "ppm_overflow": f"let left={maximum}; let right=PaymentFee{{base:Amount{{msats:0}},parts_per_million:1}}; let result=left.checked_add(right); result is Some",
        "both_overflow": f"let left={maximum}; let result=left.checked_add(left); result is Some",
        "exact_maximum": f"let left={maximum}; let zero={zero}; let result=left.checked_add(zero); result is None",
        "base_limit": "let value=PaymentFee{base:Amount{msats:2},parts_per_million:0}; let limit=PaymentFee{base:Amount{msats:1},parts_per_million:10}; let result=value.is_within(&limit); result",
        "ppm_limit": "let value=PaymentFee{base:Amount{msats:0},parts_per_million:11}; let limit=PaymentFee{base:Amount{msats:1},parts_per_million:10}; let result=value.is_within(&limit); result",
        "equality": f"let value={maximum}; let result=value.is_within(&value); !result",
        "receive_only": f"let value=PaymentFee{{base:Amount{{msats:1}},parts_per_million:0}}; let zero={zero}; let send={maximum}; let result=validate_fee_pair(zero,value,&send,&zero); result.0==3",
    }
    cases = {}
    for name, statement in statements.items():
        prefix, claim = statement.rsplit(";", 1)
        claim = claim.replace("result is Some", "match result {Some(_) => true,None => false}")
        claim = claim.replace("result is None", "match result {None => true,Some(_) => false}")
        goal = f"\npub fn false_{name}()->(result:bool) ensures result, {{{prefix};{claim}}}\n"
        cases[name] = (provider_source(selected, COMPOSITION + goal), "semantic", "postcondition not satisfied")
    mutations = {
        "wrong_nested_add": ("self.msats.checked_add(other.msats)?", "self.msats.checked_sub(other.msats)?"),
        "skipped_ppm_overflow": ("self.parts_per_million.checked_add(rhs.parts_per_million)?", "self.parts_per_million"),
        "lexicographic_limit": ("self.base <= limit.base && self.parts_per_million <= limit.parts_per_million", "self.base < limit.base || (self.base == limit.base && self.parts_per_million <= limit.parts_per_million)"),
        "mixed_borrowed_roots": ("self.base <= limit.base && self.parts_per_million <= limit.parts_per_million", "self.base <= self.base && self.parts_per_million <= self.parts_per_million"),
        "swapped_decisions": ("return (1,None)", "return (2,None)"),
        "removed_receive": ("if !transaction.is_within(receive_limit) { return (2,None); }", ""),
        "wrong_denominator": (".saturating_div(1_000_000)", ".saturating_div(999_999)"),
        "wrong_saturation": (".saturating_mul(self.parts_per_million)\n            .saturating_div(1_000_000)", ".saturating_div(1_000_000)\n            .saturating_mul(self.parts_per_million)"),
    }
    source = provider_source(selected)
    for name, (before, after) in mutations.items():
        require(before in source, "missing mutation site: " + name)
        cases[name] = (source.replace(before, after, 1), "semantic", "postcondition not satisfied")
    cases["unsafe_from_sats"] = (provider_source(selected, COMPOSITION
        + "\npub fn unsafe_amount()->Amount {Amount::from_sats(u64::MAX)}\n"), "safety", "precondition not satisfied")
    cases["eager_pre_residual_panic"] = (source.replace("msats: self.msats.checked_add(other.msats)?",
        "msats: {let optional=self.msats.checked_add(other.msats); let eager=self.msats+other.msats; optional?}", 1),
        "safety", "precondition not satisfied")
    frontends = {
        "missing_derive": source.replace("Eq,PartialEq,Ord,PartialOrd)", "Eq,PartialEq,PartialOrd)", 1),
        "missing_ord_request": source.replace("infer_derive(PartialEq,PartialOrd,Ord)", "infer_derive(PartialEq,PartialOrd)", 1),
        "multifield_comparison": source.replace("pub msats: u64,", "pub msats: u64, pub extra:u64,", 1),
        "generic_comparison": source.replace("pub struct Amount {", "pub struct Amount<T> {", 1),
        "authored_narrow_domain": source.replace("pub fn checked_add(self, other:", "#[verus_spec(requires self.msats==0)]\npub fn checked_add(self, other:", 1),
        "hidden_assumption": source.replace("msats: self.msats.checked_add(other.msats)?", "msats: {proof!{assume(false);} self.msats.checked_add(other.msats)?}", 1),
        "wrong_residual": source.replace("msats: self.msats.checked_add(other.msats)?", "msats: core::result::Result::<u64,u64>::Err(0)?", 1),
        "hidden_effect": source.replace("msats: self.msats.checked_add(other.msats)?", "msats: {let clock=std::time::SystemTime::now(); self.msats.checked_add(other.msats)?}", 1),
    }
    for name, text in frontends.items():
        require(text != source, "missing frontend mutation site: " + name)
        cases[name] = (text, "frontend", "")
    for name, declaration in {
        "drop_shape": "#[verus_verify] pub struct Dropped {pub value:u64}\n#[verus_verify] impl Drop for Dropped {fn drop(&mut self){}}\n#[verus_infer] fn unsupported(value:Dropped)->Option<Dropped>{Some(value)}",
        "opaque_shape": "#[verus_verify(external_body)] pub struct Hidden {pub value:u64}\n#[verus_infer] fn unsupported(value:Hidden)->Option<Hidden>{Some(value)}",
    }.items():
        cases[name] = ("use vstd::prelude::*;\n" + declaration + "\n", "frontend", "")
    cases["foreign_comparison_body"] = (incorrect_comparison_source().replace(
        "#[automatically_derived]\n#[verus_verify]", "#[verus_verify(external)]"), "frontend", "")
    return cases


def incorrect_comparison_source():
    return r"""use vstd::prelude::*;
#[verus_verify]
#[derive(Copy,Clone,Eq,PartialEq,PartialOrd)]
pub struct Amount {pub msats:u64}
verus! {
impl Amount {pub open spec fn __verus_infer_derived_cmp(&self)->bool{true}}
impl vstd::std_specs::cmp::OrdSpecImpl for Amount {
    open spec fn obeys_cmp_spec()->bool{true}
    open spec fn cmp_spec(&self,rhs:&Self)->core::cmp::Ordering {
        vstd::std_specs::cmp::OrdSpec::cmp_spec(&rhs.msats,&self.msats)
    }
}
}
#[automatically_derived]
#[verus_verify]
impl Ord for Amount {
    fn cmp(&self,rhs:&Self)->core::cmp::Ordering {self.msats.cmp(&rhs.msats)}
}
"""


def replay(args):
    output, runtime, sysroot = (path.resolve() for path in (args.output, args.runtime, args.sysroot))
    output.mkdir(parents=True, exist_ok=False)
    selected, native = extract(args.application.resolve())
    env = environment(output, sysroot, runtime / "z3")
    inputs = [path for path in runtime.iterdir() if path.is_file()]
    inputs += [args.application.resolve() / name for name in SOURCE_HASHES]
    inputs += [sysroot / "bin/rustc", *[path for path in (sysroot / "lib").rglob("*")
        if path.is_file() and path.suffix in (".rlib", ".dylib", ".so")]]
    inputs += [Path(__file__), Path(__file__).with_name("check_fedimint_paymentfee76.py")]
    guard = ArtifactGuard(inputs)
    receipt = dict(status="INCOMPLETE", schema=38, policy=POLICY, cache=CACHE, cases=[],
        native_pin=PIN, native_sources={name: sha(value) for name, value in native.items()},
        items={name: sha(value.encode()) for name, value in selected.items()},
        extraction_compiler_verified=False, whole_native_callers_verified=False,
        source_regions={name: dict(start=start, end=end, sha256=fingerprint)
            for name, (start, end, fingerprint) in REGIONS.items()},
        artifacts_before=guard.initial)
    def save():
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2, sort_keys=True, default=str) + "\n")
    save()
    try:
        source = provider_source(selected)
        for phase in ("cold", "warm"):
            row = invoke(output, runtime, sysroot, env, "provider_" + phase, source,
                selected=selected, phase=phase, unit="provider")
            receipt["cases"].append(row)
        provider = output / "provider"
        cert, fingerprint = certificate(provider / "provider.vir")
        original = (provider / "provider.vir").read_bytes()
        receipt["certificate"] = cert
        dependency = dict(folder=provider, fingerprint=fingerprint,
            native_sha256=sha((provider / "libprovider.rlib").read_bytes()))
        receipt["dependency"] = dependency
        for phase in ("cold", "warm"):
            receipt["cases"].append(invoke(output, runtime, sysroot, env, "client_" + phase,
                client_source(), crate="fee_client", count=0, verified=4, bodies=4,
                phase=phase, unit="client", dependency=dependency))
        for name, (text, category, diagnostic) in controls(selected).items():
            receipt["cases"].append(invoke(output, runtime, sysroot, env, name, text,
                category=category, seed=original, diagnostic=diagnostic))
        receipt["cases"].append(invoke(output, runtime, sysroot, env, "incorrect_named_cmp",
            incorrect_comparison_source(), count=0, category="semantic", seed=original))
        false_import = client_source().removesuffix("}\n") + "\npub fn false_imported(value:&PaymentFee)->(result:bool) ensures !result, {value.is_within(value)}\n}\n"
        receipt["cases"].append(invoke(output, runtime, sysroot, env, "false_imported", false_import,
            count=0, crate="fee_client", category="semantic", dependency=dependency, seed=original))
        receipt["cases"].append(invoke(output, runtime, sysroot, env, "budget", source,
            category="budget", seed=original, extra=("--rlimit", "0.000001")))
        edits = {
            "schema": lambda value: value.update(schema=37),
            "policy": lambda value: value.update(policy="candidate-bound-evidence-v73"),
            "missing_proof": lambda value: value["proofs"].pop(),
            "failed_proof": lambda value: value["proofs"][0][1].update(exactness=False),
            "summary": lambda value: value.update(summaries="0" * 64),
            "model": lambda value: value["trusted_models"][0].__setitem__(1, "0" * 64),
            "dependency": lambda value: value.update(dependencies=[dict(crate_id=0, fingerprint="0" * 64)]),
        }
        for name, edit in list(edits.items()) + [("missing_metadata", None)]:
            forged = replace_metadata(original, edit) if edit else original[:-8]
            (provider / "provider.vir").write_bytes(forged)
            receipt["cases"].append(invoke(output, runtime, sysroot, env, "import_" + name,
                client_source(), crate="fee_client", count=0, category="import", seed=original,
                dependency=dependency))
            (provider / "provider.vir").write_bytes(original)
        native_blob = (provider / "libprovider.rlib").read_bytes()
        (provider / "libprovider.rlib").write_bytes(b"invalid native artifact")
        receipt["cases"].append(invoke(output, runtime, sysroot, env, "import_invalid_native",
            client_source(), crate="fee_client", count=0, category="frontend", seed=original,
            dependency=dependency, diagnostic="invalid metadata"))
        (provider / "libprovider.rlib").write_bytes(native_blob)
        receipt["cases"].append(invoke(output, runtime, sysroot, env, "final_client", client_source(),
            crate="fee_client", count=0, verified=4, bodies=4, dependency=dependency))
        cache_entries = sorted((output / "candidate-cache").glob("*.json"))
        require(cache_entries and all(json.loads(path.read_text()).get("version") == CACHE
            for path in cache_entries), "candidate_cache_identity_guard")
        receipt["candidate_cache"] = {path.name: sha(path.read_bytes()) for path in cache_entries}
        require((provider / "provider.vir").read_bytes() == original
            and (provider / "libprovider.rlib").read_bytes() == native_blob,
            "provider_artifacts_restored_guard")
        guard.check()
        receipt["artifacts_after"] = guard.snapshot()
        receipt["inputs_unchanged"] = True
        receipt["status"] = "PASS"
        save()
    except BaseException as error:
        receipt["error"] = str(error)
        save()
        raise


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("application", "runtime", "sysroot", "output"):
        parser.add_argument("--" + name, required=True, type=Path)
    replay(parser.parse_args())


if __name__ == "__main__":
    main()
