#!/usr/bin/env python3
"""Remove Aleph's update-count proof block using checked generic counting facts.

The supplied manual collector's invariant and executable behavior are retained.
A separately verified equivalence connects its original recursive count to the
library definition. The input checkout is read-only; all changes use a clone.
"""
import argparse
import difflib
import json
import os
from pathlib import Path

from eval_verus_infer_alephbft_collections import OPS, PROPERTIES, STATE, extract_operations, validate
from eval_verus_infer_alephbft_parents import CORRECTED, FIXTURES, METHOD, ROOT, VALUE, item, prepare, remove_constructor_loop_annotation, validate_native
from infer_experiment import save_results, snapshot_inputs
from infer_eval_support import ArtifactGuard, digest, host_info, measured, tool_artifacts


def spec_item(source, marker):
    """The supplied spec/proof bodies use a four-space opening brace.

    Unlike executable methods, their ensures clauses can contain earlier `{}`.
    Reject a changed layout rather than silently editing an ensures expression.
    """
    start = source.index(marker)
    opening = source.index("\n    {", start) + 5
    depth, end = 1, opening + 1
    while depth:
        depth += (source[end] == "{") - (source[end] == "}")
        end += 1
    return start, end, opening, source[start:end]


def remove_update_block(source):
    start, end, method = item(source, "pub fn add(&mut self")
    before = method.index("            #[cfg(verus_keep_ghost)]")
    _, after, block = item(method[before:], "proof!")
    if block.count("count_update(") != 1 or block.count("count_bound(") != 1:
        raise ValueError("unexpected update proof block")
    return source[:start] + method[:before] + method[before + after:] + source[end:]


def library_counting(source, remove_hints):
    start, end, opening, original = spec_item(source, "spec fn count_prefix<T>")
    replacement = source[start:opening] + "{ candidates.count_prefix(|slot: Option<(T, u16)>| direct(slot, round), len) }"
    source = source[:start] + replacement + source[end:]
    # Experiment-only evidence: the specification migration preserves meaning
    # for every sequence, predicate round and valid prefix, independent of add.
    reference = original.replace("count_prefix", "reference_count_prefix")
    bridge = """
    proof fn library_count_agrees<T>(candidates: Seq<Option<(T, u16)>>, round: u32, len: nat)
        requires len <= candidates.len(),
        ensures count_prefix(candidates, round, len) == reference_count_prefix(candidates, round, len),
        decreases len,
    { if len > 0 { library_count_agrees(candidates, round, (len - 1) as nat); } }
"""
    pos = source.index("    proof fn count_bound")
    source = source[:pos] + "    " + reference + "\n" + bridge + source[pos:]
    if remove_hints:
        source = remove_update_block(source)
        constructor = """        #[cfg(verus_keep_ghost)]
        proof! {
            count_zero(candidates@, 1, candidates@.len());
        }
"""
        transition = "            count_zero(candidates@, (self.for_round + 1) as u32, candidates@.len());\n"
        if source.count(constructor) != 1 or source.count(transition) != 1:
            raise ValueError("unexpected initialization counting hints")
        source = source.replace(constructor, "", 1).replace(transition, "", 1)
    calls = {
        "count_bound": "candidates.lemma_count_prefix_bound(|slot: Option<(T, u16)>| direct(slot, round), len);",
        "count_update": "candidates.lemma_count_prefix_update(|slot: Option<(T, u16)>| direct(slot, round), index, value, len);",
        "count_zero": "candidates.lemma_count_prefix_zero(|slot: Option<(T, u16)>| direct(slot, round), len);",
    }
    for name, call in calls.items():
        start, end, opening, _ = spec_item(source, "proof fn " + name + "<T>")
        replacement = "" if remove_hints else source[start:opening] + "{ " + call + " }"
        source = source[:start] + replacement + source[end:]
    return source


def require_assistance(data, expected):
    selections = data["verus-infer"].get("goal_assistance", [])
    add = [s for s in selections if s["function"].endswith("::Collector::add")]
    if bool(add) != expected:
        raise ValueError("unexpected counting strategy selection on add")
    if expected and (len(add) != 1 or add[0]["strategy"] != "counting_broadcast"
                     or len(add[0]["lemmas"]) != 4):
        raise ValueError("missing closed counting theorem selection")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--alephbft", type=Path, required=True)
    parser.add_argument("--verified-collector", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--verus", type=Path, default=ROOT / "source/target-verus/release/verus")
    parser.add_argument("--z3", type=Path, default=ROOT / "source/z3")
    parser.add_argument("--native-tests", action="store_true")
    parser.add_argument("--native-target", type=Path)
    args = parser.parse_args()
    output = args.output.resolve()
    if output.exists() and any(output.iterdir()): parser.error("output directory must be new or empty")
    repo = args.alephbft.resolve()
    supplied = (args.verified_collector or repo / "consensus/src/creation/collector/state.rs").resolve()
    if not supplied.is_file(): parser.error("provide the existing manually verified collector")
    output.mkdir(parents=True, exist_ok=True)
    verus, z3 = args.verus.resolve(), args.z3.resolve()
    extra = [Path(__file__), supplied, FIXTURES / "collection_ops.rs", FIXTURES / "parents_properties.rs"]
    extra += [Path(__file__).with_name(p) for p in (
        "eval_verus_infer_alephbft_collections.py", "eval_verus_infer_alephbft_parents.py",
        "eval_verus_infer_alephbft.py", "infer_eval_support.py")]
    guard = ArtifactGuard(tool_artifacts(verus, z3, extra=extra))
    result = dict(host=host_info(), tool_artifacts=guard.initial, source={}, runs=[], native=[])
    checkout = output / "checkout"

    def save():
        save_results(output, result)

    def snapshot(name, native=False):
        relatives = [VALUE, METHOD, STATE, OPS, PROPERTIES]
        if native:
            relatives += ["verification-infer/native.rs", "verification-infer/native/Cargo.toml"]
        return snapshot_inputs(checkout, output / ("native-inputs" if native else "inputs") / name, relatives)

    try:
        shared, result["source"] = prepare(repo, checkout)
        (checkout / METHOD).write_text(shared.replace("(*self * 2) / 3 + NodeCount(1)", CORRECTED))
        original = supplied.read_text()
        extracted = extract_operations(original)
        manual = library_counting(extracted, False)
        assisted = library_counting(extracted, True)
        ops = (FIXTURES / "collection_ops.rs").read_text()
        properties = (FIXTURES / "parents_properties.rs").read_text().replace("// BRIDGE_ATTRIBUTE", "#[verus_infer]")
        properties += '\n#[path="collection_ops.rs"]\npub mod collection_ops;\n'
        (checkout / PROPERTIES).write_text(properties); (checkout / OPS).write_text(ops)
        result["source"]["collector"] = dict(path=str(supplied), sha256=digest(supplied),
            origin="existing manually verified working-tree core, not clean-main generic collector",
            invariant="original logical clauses; count delegates to generic library recurrence",
            specification_equivalence="library_count_agrees checked against original recursive reference",
            removed="add proof block; four counting calls across add/new/next; count_bound/count_update/count_zero proof definitions",
            retained="state contracts, predicate, loop annotations and next's sequence equality assertion",
            shared_proof_reuse="all three original recursive counting proof definitions are replaced by library proofs",
            experiment_only="recursive reference count and checked equivalence proof")
        env = dict(os.environ, VERUS_Z3_PATH=str(z3), RUST_MIN_STACK="20971520",
                   VERUS_INFER_CACHE=str(output / "candidate-cache"))
        cases = ("original-manual", "library-manual", "library-unassisted", "assisted", "assisted-warm",
                 "wrong-slot", "wrong-round", "wrong-counter", "missing-accounting", "missing-loop-invariant", "changed-count-definition")
        for name in cases:
            state = extracted if name == "original-manual" else manual if name == "library-manual" else assisted
            enabled = name not in cases[:3]
            expected = name in ("original-manual", "library-manual", "assisted", "assisted-warm")
            if name == "wrong-slot": state = state.replace("&mut self.candidates, creator,", "&mut self.candidates, 0,", 1)
            if name == "wrong-round": state = state.replace("&mut self.candidates, creator, Some((payload, round))", "&mut self.candidates, creator, Some((payload, 0))", 1)
            if name == "wrong-counter": state = state.replace("self.direct_parents += 1", "self.direct_parents += 2", 1)
            if name == "missing-accounting":
                old = "&&& self.direct_parents == count_prefix(\n                self.candidates@, self.for_round, self.candidates@.len())"
                if state.count(old) != 1: raise ValueError("unexpected accounting invariant")
                state = state.replace(old, "&&& true", 1)
            if name == "missing-loop-invariant": state = remove_constructor_loop_annotation(state)
            if name == "changed-count-definition":
                state = state.replace("candidates.count_prefix(|slot: Option<(T, u16)>| direct(slot, round), len)",
                                      "candidates.count_prefix(|slot: Option<(T, u16)>| !direct(slot, round), len)", 1)
            (checkout / STATE).write_text(state)
            command = [str(verus), str(checkout / PROPERTIES), "--crate-type=lib", "--crate-name=aleph_counting", "--no-cheating", "--output-json"]
            if enabled: command.append("--infer-goals")
            inputs = snapshot(name)
            guard.check(); run = measured(command, env, 60, ROOT); guard.check()
            (output / f"{name}.stdout.json").write_text(run["stdout"])
            (output / f"{name}.stderr.txt").write_text(run["stderr"])
            data = validate(run, expected, 3)
            require_assistance(data, enabled and name != "missing-accounting")
            if name == "assisted-warm" and any(f["metrics"]["cache"] != "hit"
                for f in data["verus-infer"]["functions"] if "collection_ops::" not in f["function"]):
                raise ValueError("scalar warm candidates were not reused and rechecked")
            result["runs"].append(dict(case=name, expected_success=expected,
                classification="verified" if expected else "unassisted_proof_failure" if name == "library-unassisted" else "semantic_rejection",
                assisted=enabled, add_counting_lemma_calls=2 if name in cases[:2] else 0,
                command=command, returncode=run["returncode"], wall_micros=run["wall_micros"],
                verification=data["verification-results"], report=data["verus-infer"], **inputs))
            save(); print(f"{name}: expected outcome", flush=True)
        if args.native_tests:
            native_dir = checkout / "verification-infer/native"; native_dir.mkdir()
            manifest = native_dir / "Cargo.toml"
            manifest.write_text('[package]\nname="aleph-counting-shared-core"\nversion="0.0.0"\nedition="2021"\n'
                                '[workspace]\n[lib]\npath="../native.rs"\n[dependencies]\n'
                                'derive_more = { version="1.0", features=["full"] }\n')
            _, _, bridge = item(properties, "pub fn consensus_threshold(")
            (checkout / "verification-infer/native.rs").write_text(
                '#[path="../crypto/src/node_count_value.rs"]\nmod node_count_value;\npub use node_count_value::NodeCount;\n'
                '#[path="../crypto/src/quorum_method.rs"]\nmod quorum_method;\n' + bridge
                + '\n#[path="collector_state.rs"]\nmod collector;\n#[path="collection_ops.rs"]\nmod collection_ops;\n')
            native_env = dict(os.environ); native_env.pop("RUSTUP_TOOLCHAIN", None)
            if args.native_target: native_env["CARGO_TARGET_DIR"] = str(args.native_target.resolve())
            for name, state in (("original", original), ("assisted", assisted),
                ("wrong-slot", assisted.replace("&mut self.candidates, creator,", "&mut self.candidates, 0,", 1)),
                ("wrong-round", assisted.replace("&mut self.candidates, creator, Some((payload, round))", "&mut self.candidates, creator, Some((payload, 0))", 1))):
                (checkout / STATE).write_text(state)
                command = ["cargo", "test", "--offline", "--manifest-path", str(manifest), "--lib", "collector::tests", "--", "--nocapture"]
                inputs = snapshot(name, True)
                guard.check(); run = measured(command, native_env, 180, checkout); guard.check()
                (output / f"native-{name}.txt").write_text(run["stdout"] + run["stderr"])
                expected = name in ("original", "assisted")
                validate_native(run, expected, name)
                result["native"].append(dict(case=name, expected_success=expected, command=command,
                    returncode=run["returncode"], wall_micros=run["wall_micros"], **inputs))
                save(); print(f"native {name}: expected outcome", flush=True)
            if (native_dir / "Cargo.lock").exists(): (output / "native-Cargo.lock").write_text((native_dir / "Cargo.lock").read_text())
        (checkout / STATE).write_text(assisted)
        (output / "counting-assistance.patch").write_text("".join(difflib.unified_diff(
            extracted.splitlines(keepends=True), assisted.splitlines(keepends=True),
            fromfile="a/collector_state.rs", tofile="b/collector_state.rs")))
        guard.check(); result["success"] = True
        (output / "summary.md").write_text(
            "# AlephBFT counting assistance\n\n"
            "The supplied manual collector's invariant is retained. Its count delegates to the generic library recurrence; "
            "a checked equivalence proof compares that definition with the original recursive count. "
            "The executable add method has no counting proof block or counting lemma calls. "
            "All three application counting proof definitions and four calls across add/new/next are removed. "
            "Loop annotations, state contracts and next's sequence equality assertion remain manual. "
            "The reference count and equivalence proof are experiment-only evidence.\n\n"
            + "\n".join(f"- {r['case']}: {r['classification']}" for r in result["runs"])
            + "\n\n" + "\n".join(f"- native {r['case']}: expected outcome" for r in result["native"])
            + "\n\nThe new automation selects four checked library lemmas using scoped broadcast activation. "
            "It adds no axioms or trusted external bodies and does not infer the invariant or the whole collector. "
            "State precision is still not established. Timing observations are not a benchmark.\n")
    except Exception as error:
        result["success"] = False; result["error"] = str(error)
        raise
    finally:
        save()


if __name__ == "__main__":
    main()
