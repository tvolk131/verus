#!/usr/bin/env python3
"""Check the supplied collector update across a genuine private NodeMap crate.

The supplied direct sequence view is explicit. Only the update and its supplied
state specification are formal targets here; construction/transition/query
methods and the clean-main Unit/Hasher collector are outside this checkpoint.
"""
import argparse
import difflib
import json
import os
from pathlib import Path

from eval_verus_infer_alephbft import REVISION, git
from eval_verus_infer_alephbft_parents import CORRECTED, FIXTURES, NODE, ROOT, item, prepare
from eval_verus_infer_alephbft_wrappers import node_map_source, wrap_collector, api_mutation
from eval_verus_infer_alephbft_records import add_mutation, require_domain_rejection
from infer_eval_support import ArtifactGuard, digest, host_info, measured, tool_artifacts

API = "verification-infer/private_node.rs"
STATE = "verification-infer/private_collector.rs"
PROPERTIES = "verification-infer/private_properties.rs"


def private_api(original, threshold):
    source = node_map_source(original)
    view = """
#[cfg(verus_keep_ghost)]
verus! {
impl<T> View for NodeMap<T> {
    type V = Seq<Option<T>>;
    closed spec fn view(&self) -> Self::V { self.0@ }
}
}
"""
    # Same scalar representation and threshold body; unused native derives are
    # excluded, as in the earlier formal adapters. No guessed operator model.
    return ("use vstd::prelude::*;\n#[derive(Copy,Clone)]\n"
            "#[cfg_attr(verus_keep_ghost,verifier::verify)]\npub struct NodeCount(pub usize);\n"
            + threshold.replace("use crate::NodeCount;", "") + source + view)


def update_only(original):
    source = wrap_collector(original)
    source = source[:source.index("\n#[cfg(test)]")]
    for name in ("new", "next", "prospective_parents"):
        start, end, _ = item(source, f"pub fn {name}(")
        annotation = source.rindex("    #[cfg_attr(verus_keep_ghost, verus_spec(", 0, start)
        source = source[:annotation] + source[end:]
    source = source.replace("self.candidates.0@", "self.candidates@")
    # A closed view does not automatically expose the Vec length bound. Observe
    # the existing checked size query in both ghost-only callers, without adding
    # a domain assumption or changing add's executable body.
    source = source.replace("        self.add(creator, payload, round);",
                            "        let _ = self.candidates.size();\n        self.add(creator, payload, round);")
    if "self.candidates.0" in source or "NodeMap(candidates)" in source:
        raise ValueError("private client still accesses the representation")
    return source


def validate(run, expected, count, unsupported=False):
    data = json.loads(run["stdout"])
    result = data["verification-results"]
    if (run["returncode"] == 0) != expected or result.get("success") != expected:
        raise ValueError(f"unexpected verification outcome: {result}")
    if any(s in run["stderr"].lower() for s in ("panicked at", "resource limit", "timed out")):
        raise ValueError("panic/resource failure is not proof evidence")
    if unsupported:
        if "visible transparent record fields" not in run["stderr"]:
            raise ValueError("missing explicit unsupported-view diagnostic")
        return data
    if result.get("encountered-vir-error"):
        raise ValueError("frontend failure is not a semantic rejection")
    functions = data["verus-infer"]["functions"]
    if len(functions) != count:
        raise ValueError("unexpected inferred-function count")
    for f in functions:
        if f["mode"] != "exact" or not f["required_checks_passed"]:
            raise ValueError("incomplete exact certification")
        for p in ("soundness", "maximality", "exactness"):
            if not f["proofs"][p]["required"] or f["proofs"][p]["status"] != "proved":
                raise ValueError("missing exact obligation")
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--alephbft", type=Path, required=True)
    parser.add_argument("--verified-collector", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--verus", type=Path, default=ROOT / "source/target-verus/release/verus")
    parser.add_argument("--z3", type=Path, default=ROOT / "source/z3")
    args = parser.parse_args()
    repo = args.alephbft.resolve(); output = args.output.resolve()
    supplied = (args.verified_collector or repo / "consensus/src/creation/collector/state.rs").resolve()
    if output.exists() and any(output.iterdir()): parser.error("output must be new or empty")
    if not supplied.is_file(): parser.error("provide the supplied collector core")
    output.mkdir(parents=True, exist_ok=True)
    verus = args.verus.resolve(); z3 = args.z3.resolve()
    extra = [Path(__file__), supplied, FIXTURES / "parents_properties.rs"]
    extra += [Path(__file__).with_name(n) for n in (
        "eval_verus_infer_alephbft.py", "eval_verus_infer_alephbft_parents.py",
        "eval_verus_infer_alephbft_counting.py", "eval_verus_infer_alephbft_records.py",
        "eval_verus_infer_alephbft_wrappers.py", "infer_eval_support.py")]
    guard = ArtifactGuard(tool_artifacts(verus, z3, extra=extra))
    result = dict(host=host_info(), tool_artifacts=guard.initial, runs=[])
    checkout = output / "checkout"
    def save(): (output / "results.json").write_text(json.dumps(result, indent=2) + "\n")
    try:
        shared, origin = prepare(repo, checkout)
        threshold = shared.replace("(*self * 2) / 3 + NodeCount(1)", CORRECTED)
        original_node = git(checkout, "show", f"{REVISION}:{NODE}")
        original = supplied.read_text(); api = private_api(original_node, threshold)
        state = update_only(original)
        properties = (FIXTURES / "parents_properties.rs").read_text()
        start = properties.index('#[path = "../crypto/src/node_count_value.rs"]')
        end = properties.index("// The existing shared collector")
        properties = properties[:start] + "pub use aleph_node_api::{NodeCount,NodeIndex,NodeMap};\n\n" + properties[end:]
        properties = properties.replace("// BRIDGE_ATTRIBUTE", "#[verus_infer]")
        properties = properties.replace('"collector_state.rs"', '"private_collector.rs"')
        (checkout / PROPERTIES).write_text(properties)
        for name in ("get", "insert"):
            if item(api, f"pub fn {name}(")[2] != item(original_node, f"pub fn {name}(")[2]:
                raise ValueError("original API body changed")
        result["source"] = dict(**origin, collector=str(supplied), collector_sha256=digest(supplied),
            formal_scope="supplied collector update and invariant, not clean-main Unit/Hasher or other lifecycle methods",
            api="separate compiled crate with private storage and a supplied closed direct sequence view",
            size_adaptation="self.0.len().into() -> NodeCount(self.0.len())",
            handholding="existing invariant/count equivalence/behavior oracle; size observation in ghost-only callers",
            runtime_changes="previously documented threshold correction and supplied-core round widening only")
        env = dict(os.environ, VERUS_Z3_PATH=str(z3), RUST_MIN_STACK="20971520",
                   VERUS_INFER_CACHE=str(output / "candidate-cache"))
        directory = checkout / "verification-infer"
        lib = directory / "libaleph_node_api.rlib"; interface = directory / "aleph_node_api.vir"
        # Leave both source and compiled interfaces in the passing configuration.
        cases = ("lossy-view", "wrong-lookup", "wrong-insert", "wrong-size", "duplicate-count",
                 "changed-frame", "extra-overflow-domain", "inferred", "inferred-warm")
        for case in cases:
            current_api = api; current_state = state
            if case == "wrong-lookup": current_api = api_mutation(api, "get", "self.0[node_id.0]", "self.0[0]")
            if case == "wrong-insert": current_api = api_mutation(api, "insert", "self.0[node_id.0]", "self.0[0]")
            if case == "wrong-size": current_api = api_mutation(api, "size", "NodeCount(self.0.len())", "NodeCount(self.0.len()+1)")
            if case == "duplicate-count": current_state = add_mutation(state, "*previous_round < round", "*previous_round <= round")
            if case == "changed-frame": current_state = add_mutation(state, "self.direct_parents += 1;", "self.direct_parents += 1; self.for_round += 1;")
            if case == "extra-overflow-domain": current_state = add_mutation(state, "        if creator >=", "        let extra = creator + 1;\n        if creator >=")
            if case == "lossy-view": current_api = api.replace("{ self.0@ }", "{ self.0@.take(0) }")
            (checkout / API).write_text(current_api); (checkout / STATE).write_text(current_state)
            snapshot = output / "inputs" / case; snapshot.mkdir(parents=True)
            for path in (API, STATE, PROPERTIES): (snapshot / Path(path).name).write_text((checkout / path).read_text())
            provider = [str(verus), str(checkout / API), "--crate-type=lib", "--crate-name=aleph_node_api",
                        "--export", str(interface), "--compile", "-o", str(lib), "--no-cheating", "--output-json"]
            consumer = [str(verus), str(checkout / PROPERTIES), "--crate-type=lib", "--crate-name=aleph_private",
                        "--extern", f"aleph_node_api={lib}", "--import", f"aleph_node_api={interface}",
                        "--infer-goals", "--no-cheating", "--output-json"]
            for stage, command, expected, count in (("provider", provider, True, 4),
                                                   ("consumer", consumer, case in ("inferred", "inferred-warm"), 2)):
                guard.check(); run = measured(command, env, 60, ROOT); guard.check()
                (output / f"{case}-{stage}.stdout.json").write_text(run["stdout"])
                (output / f"{case}-{stage}.stderr.txt").write_text(run["stderr"])
                data = validate(run, expected, count, case == "lossy-view" and stage == "consumer")
                if stage == "consumer" and case in ("extra-overflow-domain", "wrong-size"):
                    require_domain_rejection(data, run["stderr"])
                result["runs"].append(dict(case=case, stage=stage, expected_success=expected, command=command,
                    verification=data["verification-results"], report=data.get("verus-infer"),
                    input_snapshot=str(snapshot), source_hashes={p.name:digest(p) for p in snapshot.iterdir()},
                    wall_micros=run["wall_micros"]))
                save(); print(f"{case}/{stage}: expected outcome", flush=True)
        (checkout / API).write_text(api); (checkout / STATE).write_text(state)
        (output / "private-collector.patch").write_text("".join(difflib.unified_diff(
            original.splitlines(keepends=True), state.splitlines(keepends=True),
            fromfile="a/collector_state.rs", tofile="b/private_collector.rs")))
        (output / "original-node.rs").write_text(original_node)
        (output / "summary.md").write_text(
            "# Private NodeMap inference\n\nA separate compiled provider keeps NodeMap storage private. "
            "A supplied closed direct sequence view connects independently checked inferred API contracts "
            "to the supplied collector update. Both ghost-only caller goals observe the checked size query "
            "to expose the hidden vector length bound. No additional input-domain assumption is added. "
            "This is the supplied update core, not clean-main Unit/Hasher or constructor/transition inference.\n\n"
            + "\n".join(f"- {r['case']}/{r['stage']}: expected {'success' if r['expected_success'] else 'rejection'}"
                         for r in result["runs"]) + "\n")
        guard.check(); result["success"] = True
    except Exception as error:
        result["success"] = False; result["error"] = str(error); raise
    finally: save()


if __name__ == "__main__": main()
