#!/usr/bin/env python3
"""Check Aleph getter bodies and a generic unit-to-collector update adapter.

The Unit/Hasher interface is an explicit reduced adapter with supplied logical
getter contracts. Original creator/round default bodies and coordinate getter
bodies are retained. This is not verification of the full Aleph traits/crates.
"""
import argparse
import difflib
import json
import os
import re
from pathlib import Path

from eval_verus_infer_alephbft import REVISION, git
from eval_verus_infer_alephbft_parents import CORRECTED, FIXTURES, NODE, ROOT, item, prepare
from eval_verus_infer_alephbft_private import private_api, update_only, validate
from eval_verus_infer_alephbft_records import add_mutation
from infer_experiment import save_results
from infer_eval_support import ArtifactGuard, digest, host_info, measured, tool_artifacts

UNITS = "consensus/src/units/mod.rs"
TYPES = "types/src/lib.rs"
API = "verification-infer/trait_node.rs"
STATE = "verification-infer/trait_collector.rs"
PROPERTIES = "verification-infer/trait_properties.rs"

WRAPPER_IMPLS = """
// Explicit verified equivalents of the wrapper's derived conversion/operator.
#[cfg_attr(verus_keep_ghost, verus_infer)]
impl From<usize> for NodeCount {
    fn from(value: usize) -> Self { NodeCount(value) }
}
#[cfg_attr(verus_keep_ghost, verus_infer)]
impl core::ops::AddAssign for NodeCount {
    fn add_assign(&mut self, rhs: Self) { self.0 += rhs.0; }
}
"""


def wrapper_api(source, original_node):
    source = source.replace("NodeCount(self.0.len())", "self.0.len().into()")
    for name in ("size", "get", "insert"):
        if item(source, f"pub fn {name}(")[2] != item(original_node, f"pub fn {name}(")[2]:
            raise ValueError("original NodeMap body changed")
    return source + WRAPPER_IMPLS


def wrapper_collector(source):
    if source.count("direct_parents: usize,") != 1:
        raise ValueError("counter declaration changed")
    source = source.replace("NodeMap, NodeIndex, Hasher, Unit", "NodeMap, NodeIndex, NodeCount, Hasher, Unit")
    source = source.replace("direct_parents: usize,", "direct_parents: NodeCount,")
    source = source.replace("self.direct_parents", "self.direct_parents.0")
    if source.count("self.direct_parents.0 += 1;") != 1:
        raise ValueError("counter increment changed")
    return source.replace("self.direct_parents.0 += 1;", "self.direct_parents += NodeCount(1);")


def operator_control(source, case):
    req = "self.0 as int + rhs.0 as int <= usize::MAX as int"
    body = "self.0 += rhs.0;"
    output = "(self.0 + rhs.0) as usize"
    if case == "restrictive-operator": req += " / 2"
    if case == "weak-operator": req = "true"
    if case == "wrong-operator-model": output = "self.0"
    if case == "replacement-operator": req, body, output = "true", "self.0 = rhs.0;", "rhs.0"
    marker = "#[cfg_attr(verus_keep_ghost, verus_infer)]\nimpl core::ops::AddAssign for NodeCount"
    start, end, _ = item(source, marker)
    return source[:start] + """
#[cfg_attr(verus_keep_ghost, verus_verify)]
impl core::ops::AddAssign for NodeCount {
    #[cfg_attr(verus_keep_ghost, verus_infer)]
    fn add_assign(&mut self, rhs: Self) { BODY }
}
#[cfg(verus_keep_ghost)]
verus! {
impl vstd::std_specs::ops::AddAssignSpecImpl for NodeCount {
    open spec fn obeys_add_assign_spec()->bool { true }
    open spec fn add_assign_req(&self,rhs:Self)->bool { REQ }
    open spec fn add_assign_spec(&self,rhs:Self)->&Self { &NodeCount(OUTPUT) }
}
}
""".replace("BODY", body).replace("REQ", req).replace("OUTPUT", output) + source[end:]


def getter_adapter(units):
    _, _, unit = item(units, "pub trait Unit:")
    _, _, coord = item(units, "impl UnitCoord")
    # These signatures and executable default bodies come from pinned source;
    # only the result binder and ordinary verification contract are added.
    methods = []
    for name, typ in (("creator", "NodeIndex"), ("round", "Round")):
        original = item(unit, f"fn {name}(")[2]
        annotated = original.replace(f"-> {typ} {{",
            f"-> (result: {typ})\n        ensures result == self.{name}_value(),\n    {{", 1)
        if original == annotated: raise ValueError("changed getter signature")
        if original[original.index("{"):] != annotated[annotated.index("{"):]:
            raise ValueError("default getter body changed")
        methods.append(annotated)
    coords = []
    for name, typ in (("creator", "NodeIndex"), ("round", "Round")):
        original = item(coord, f"pub fn {name}(")[2]
        index = 0 if name == "creator" else 1
        annotated = original.replace(f"-> {typ} {{",
            f"-> (result: {typ}) ensures result == self.parts().{index}, {{", 1)
        if original == annotated or original[original.index("{"):] != annotated[annotated.index("{"):]:
            raise ValueError("coordinate getter signature/body changed")
        coords.append(annotated)
    hash_decl = re.search(r"fn hash\(&self\) -> [^;]+;", unit).group(0)
    hash_decl = hash_decl.replace("-> <Self::Hasher as Hasher>::Hash;",
        "-> (result: <Self::Hasher as Hasher>::Hash) ensures result == self.hash_value();")
    return "\nverus! {\n" + """
pub type Round = u16;
// Reduced interface: native marker/codec bounds and unrelated methods omitted.
pub trait Hasher { type Hash: Copy; }
#[derive(Copy,Clone)]
pub struct UnitCoord { round: Round, creator: NodeIndex }
impl UnitCoord {
    pub closed spec fn parts(&self) -> (NodeIndex, Round) { (self.creator, self.round) }
""" + "\n".join(coords) + "\n}\n" + """
pub trait Unit {
    type Hasher: Hasher;
    spec fn creator_value(&self) -> NodeIndex;
    spec fn round_value(&self) -> Round;
    spec fn hash_value(&self) -> <Self::Hasher as Hasher>::Hash;
    fn coord(&self) -> (result: UnitCoord)
        ensures result.parts().0 == self.creator_value(), result.parts().1 == self.round_value();
""" + hash_decl + "\n" + "\n".join(methods) + """
}
// A deterministic concrete adapter, not Aleph's cached FullUnit implementation.
pub struct ProbeHasher;
impl Hasher for ProbeHasher { type Hash = u64; }
pub struct ProbeUnit { coord: UnitCoord, hash: u64 }
impl Unit for ProbeUnit {
    type Hasher = ProbeHasher;
    closed spec fn creator_value(&self) -> NodeIndex { self.coord.creator }
    closed spec fn round_value(&self) -> Round { self.coord.round }
    closed spec fn hash_value(&self) -> u64 { self.hash }
    fn coord(&self) -> UnitCoord { self.coord }
    fn hash(&self) -> u64 { self.hash }
}
}
"""


def generic_collector(original):
    source = update_only(original)
    source = source.replace("use super::{consensus_threshold, NodeMap, NodeIndex};",
        "use super::{consensus_threshold, NodeMap, NodeIndex, Hasher, Unit};")
    source = source.replace("pub struct Collector<T: Copy>", "pub struct Collector<H: Hasher>")
    source = source.replace("candidates: NodeMap<(T, u16)>", "candidates: NodeMap<(H::Hash, u16)>")
    marker = "impl<T: Copy> Collector<T>"
    while marker in source:
        start, end, block = item(source, marker)
        block = block.replace(marker, "impl<H: Hasher> Collector<H>")
        block = block.replace("Seq<Option<(T, u16)>>", "Seq<Option<(H::Hash, u16)>>")
        block = block.replace("payload: T", "payload: H::Hash")
        source = source[:start] + block + source[end:]
    # Keep the previous direct-value update and its independent exact checks.
    # This new inferred adapter removes the need for callers to unpack the unit.
    adapter = """
    #[cfg_attr(verus_keep_ghost, verus_infer)]
    pub fn add_unit<U: Unit<Hasher = H>>(&mut self, unit: &U) {
        let creator = unit.creator().0;
        let payload = unit.hash();
        let round = unit.round();
        self.add(creator, payload, round);
    }

    #[cfg(verus_keep_ghost)]
    #[verus_spec(requires old(self).valid(), ensures final(self).valid())]
    pub fn check_unit_invariant<U: Unit<Hasher = H>>(&mut self, unit: &U) {
        let _ = self.candidates.size();
        self.add_unit(unit);
    }
"""
    start, end, _ = item(source, "pub fn check_add_behavior(")
    annotation = source.rindex("    #[cfg_attr(verus_keep_ghost, verus_spec(", 0, start)
    contract = source[annotation:start]
    for name, value in (("creator", "unit.creator_value().0"), ("payload", "unit.hash_value()"), ("round", "unit.round_value()")):
        contract = re.sub(r"(?<![\w.])" + name + r"\b(?!\s*\()", value, contract)
    oracle = "\n    #[cfg(verus_keep_ghost)]\n" + contract + """    pub fn check_unit_behavior<U: Unit<Hasher = H>>(&mut self, unit: &U) {
        let _ = self.candidates.size();
        self.add_unit(unit);
    }
"""
    source = source[:end] + adapter + oracle + source[end:]
    return source


def check_domain(data, stderr):
    if "precondition not satisfied" not in stderr or not any(
        f.endswith("::check_unit_invariant") for f in data["verus-infer"]["failed_functions"]
    ): raise ValueError("new unit adapter lost valid-domain coverage without a harness rejection")


def unsupported_getter(run):
    data = json.loads(run["stdout"])
    if run["returncode"] == 0 or data["verification-results"].get("success") is not False \
        or data.get("verus-infer", {}).get("status") != "unsupported" \
        or "trait inference requires a certified precondition-free getter" not in run["stderr"] \
        or any(s in run["stderr"].lower() for s in ("panicked at", "resource limit", "timed out")):
        raise ValueError("weak contract was not explicitly rejected")
    return data


def validate_stage(run, expected, count, stage):
    data = validate(run, expected, count)
    kinds = sorted(f["summary_kind"] for f in data["verus-infer"]["functions"])
    wanted = (["record_state", "record_state", "record_view", "record_view", "value", "value"] if count == 6 else
              ["record_state", "record_view", "record_view", "value"]) if stage == "provider" else ["record_state", "record_state", "value"]
    if kinds != wanted: raise ValueError("missing getter/update inference kinds")
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--alephbft", type=Path, required=True)
    parser.add_argument("--verified-collector", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--wrapper-ops", action="store_true", help="retain the original size conversion and use a NodeCount counter")
    parser.add_argument("--verus", type=Path, default=ROOT / "source/target-verus/release/verus")
    parser.add_argument("--z3", type=Path, default=ROOT / "source/z3")
    args = parser.parse_args(); repo = args.alephbft.resolve(); output = args.output.resolve()
    supplied = (args.verified_collector or repo / "consensus/src/creation/collector/state.rs").resolve()
    if output.exists() and any(output.iterdir()): parser.error("output must be new or empty")
    if not supplied.is_file(): parser.error("provide the supplied collector core")
    output.mkdir(parents=True, exist_ok=True); verus = args.verus.resolve(); z3 = args.z3.resolve()
    extra = [Path(__file__), supplied, FIXTURES / "parents_properties.rs"]
    extra += [Path(__file__).with_name(n) for n in (
        "eval_verus_infer_alephbft.py", "eval_verus_infer_alephbft_parents.py",
        "eval_verus_infer_alephbft_counting.py", "eval_verus_infer_alephbft_records.py",
        "eval_verus_infer_alephbft_wrappers.py", "eval_verus_infer_alephbft_private.py", "infer_eval_support.py")]
    guard = ArtifactGuard(tool_artifacts(verus, z3, extra=extra))
    result = dict(host=host_info(), tool_artifacts=guard.initial, runs=[])
    checkout = output / "checkout"
    def save():
        save_results(output, result)
    try:
        shared, origin = prepare(repo, checkout)
        node = git(checkout, "show", f"{REVISION}:{NODE}")
        units = git(checkout, "show", f"{REVISION}:{UNITS}")
        types = git(checkout, "show", f"{REVISION}:{TYPES}")
        if not re.search(r"type Hash:.*Copy", types): raise ValueError("pinned Hash no longer has its Copy bound")
        original = supplied.read_text()
        api = private_api(node, shared.replace("(*self * 2) / 3 + NodeCount(1)", CORRECTED)) + getter_adapter(units)
        state = generic_collector(original)
        if args.wrapper_ops:
            api = wrapper_api(api, node)
            state = wrapper_collector(state)
        properties = (FIXTURES / "parents_properties.rs").read_text()
        start = properties.index('#[path = "../crypto/src/node_count_value.rs"]')
        end = properties.index("// The existing shared collector")
        properties = properties[:start] + "pub use aleph_trait_api::{NodeCount,NodeIndex,NodeMap,Hasher,Unit};\n\n" + properties[end:]
        properties = properties.replace("// BRIDGE_ATTRIBUTE", "#[verus_infer]").replace('"collector_state.rs"', '"trait_collector.rs"')
        (checkout / PROPERTIES).write_text(properties)
        (output / "original-units.rs").write_text(units); (output / "original-types.rs").write_text(types)
        result["source"] = dict(**origin, collector=str(supplied), collector_sha256=digest(supplied),
            scope="reduced Unit/Hasher adapter and supplied update core; not full original traits, cached FullUnit or clean-main generic collector",
            preserved="pinned Unit creator/round default bodies, coordinate getter bodies, NodeMap lookup/insert bodies",
            supplied="three observation functions, exact getter contracts, coordinate contracts, direct view, invariant/count equivalence and size observations",
            omitted="Unit control_hash/session_id and native marker/codec bounds; Hasher hashing algorithm; collector lifecycle",
            new_adapter="Collector<H> stores H::Hash and an inferred add_unit reads Unit getters then calls checked add",
            existing_adaptations="NodeCount size constructor; threshold correction; supplied-core widened round")
        if args.wrapper_ops:
            result["source"].update(
                wrapper_scope="original NodeMap size/get/insert bodies; supplied generic collector with a NodeCount counter and production increment expression",
                wrapper_models="impl attributes generate checked ordinary extension definitions; explicit trait implementations replace the native derives in this formal adapter",
                existing_adaptations="threshold correction; supplied-core widened round; reduced Unit/Hasher interface and supplied collector core")
        env = dict(os.environ, VERUS_Z3_PATH=str(z3), RUST_MIN_STACK="20971520", VERUS_INFER_CACHE=str(output / "candidate-cache"))
        directory = checkout / "verification-infer"; lib = directory / "libaleph_trait_api.rlib"; interface = directory / "aleph_trait_api.vir"
        cases = ("wrong-creator-body", "wrong-hash-implementation", "weak-round-contract", "wrong-adapter-slot", "duplicate-count",
                 "changed-frame", "extra-unit-overflow-domain", "inferred", "inferred-warm")
        if args.wrapper_ops: cases = ("restrictive-operator", "weak-operator", "wrong-operator-model", "replacement-operator") + cases
        for case in cases:
            current_api = api; current_state = state
            if case.endswith("-operator") or case == "wrong-operator-model": current_api = operator_control(api, case)
            if case == "wrong-creator-body": current_api = api.replace("self.coord().creator()", "NodeIndex(0)")
            if case == "wrong-hash-implementation": current_api = api.replace("fn hash(&self) -> u64 { self.hash }", "fn hash(&self) -> u64 { 0 }")
            if case == "weak-round-contract": current_api = api.replace("ensures result == self.round_value()", "ensures result <= self.round_value()")
            if case == "wrong-adapter-slot": current_state = current_state.replace("let creator = unit.creator().0;", "let creator = 0usize;")
            if case == "duplicate-count": current_state = add_mutation(state, "*previous_round < round", "*previous_round <= round")
            if case == "changed-frame":
                increment = "self.direct_parents += NodeCount(1);" if args.wrapper_ops else "self.direct_parents += 1;"
                current_state = add_mutation(state, increment, increment + " self.for_round += 1;")
            if case == "extra-unit-overflow-domain": current_state = current_state.replace("        let creator = unit.creator().0;", "        let creator = unit.creator().0;\n        let extra = creator + 1;")
            (checkout / API).write_text(current_api); (checkout / STATE).write_text(current_state)
            snapshot = output / "inputs" / case; snapshot.mkdir(parents=True)
            for p in (API, STATE, PROPERTIES): (snapshot / Path(p).name).write_text((checkout / p).read_text())
            provider = [str(verus), str(checkout / API), "--crate-type=lib", "--crate-name=aleph_trait_api", "--export", str(interface), "--compile", "-o", str(lib), "--infer-goals", "--no-cheating", "--output-json"]
            consumer = [str(verus), str(checkout / PROPERTIES), "--crate-type=lib", "--crate-name=aleph_traits", "--extern", f"aleph_trait_api={lib}", "--import", f"aleph_trait_api={interface}", "--infer-goals", "--no-cheating", "--output-json"]
            provider_ok = case not in ("wrong-creator-body", "wrong-hash-implementation", "restrictive-operator", "weak-operator", "wrong-operator-model")
            stages = [("provider", provider, provider_ok, 6 if args.wrapper_ops else 4)]
            if provider_ok: stages.append(("consumer", consumer, case in ("inferred", "inferred-warm"), 3))
            for stage, command, expected, count in stages:
                guard.check(); run = measured(command, env, 60, ROOT); guard.check()
                (output / f"{case}-{stage}.stdout.json").write_text(run["stdout"])
                (output / f"{case}-{stage}.stderr.txt").write_text(run["stderr"])
                unsupported = case == "weak-round-contract" and stage == "consumer"
                if unsupported:
                    data = unsupported_getter(run)
                elif case in ("restrictive-operator", "weak-operator", "wrong-operator-model"):
                    data = json.loads(run["stdout"])
                    expected_message = "postcondition not satisfied" if case == "wrong-operator-model" else "inherited trait precondition must equal"
                    if run["returncode"] == 0 or data["verification-results"].get("encountered-vir-error") or data["verification-results"].get("success") is not False or expected_message not in run["stderr"] or any(s in run["stderr"].lower() for s in ("panicked at", "resource limit", "timed out")):
                        raise ValueError("operator boundary was not rejected by verification")
                    funcs = data.get("verus-infer", {}).get("functions", [])
                    op = [f for f in funcs if f["function"].endswith("::add_assign")]
                    if len(funcs) != 6 or len(op) != 1 or op[0]["required_checks_passed"]:
                        raise ValueError("failed operator received exact certification")
                    if case == "restrictive-operator" and op[0]["proofs"]["soundness"]["status"] != "proved":
                        raise ValueError("restrictive control must distinguish soundness from maximality")
                else: data = validate_stage(run, expected, count, stage)
                if stage == "provider" and not expected:
                    if interface.exists() or not any(s in run["stderr"] for s in ("postcondition not satisfied", "inherited trait precondition must equal")):
                        raise ValueError("failed ordinary getter implementation published an interface")
                if stage == "consumer" and case == "extra-unit-overflow-domain": check_domain(data, run["stderr"])
                if stage == "consumer" and case == "wrong-adapter-slot":
                    failures = data["verus-infer"]["failed_functions"]
                    if not any(f.endswith("::check_unit_behavior") for f in failures) or any(f.endswith("::check_unit_invariant") for f in failures):
                        raise ValueError("wrong slot must fail intent while preserving the minimal invariant")
                result["runs"].append(dict(case=case, stage=stage, expected_success=expected, command=command,
                    verification=data["verification-results"], report=data.get("verus-infer"),
                    source_hashes={p.name:digest(p) for p in snapshot.iterdir()}, input_snapshot=str(snapshot), wall_micros=run["wall_micros"]))
                save(); print(f"{case}/{stage}: expected outcome", flush=True)
        (output / "trait-collector.patch").write_text("".join(difflib.unified_diff(
            original.splitlines(keepends=True), state.splitlines(keepends=True), fromfile="a/collector_state.rs", tofile="b/trait_collector.rs")))
        guard.check(); result["success"] = True
        (output / "summary.md").write_text("# Aleph trait getter checkpoint\n\n" + json.dumps(result["source"], indent=2) + "\n\n" + "\n".join(f"- {r['case']}/{r['stage']}: expected outcome" for r in result["runs"]) + "\n")
    except Exception as error:
        result["success"] = False; result["error"] = str(error); raise
    finally: save()


if __name__ == "__main__": main()
