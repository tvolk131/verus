#!/usr/bin/env python3
"""Infer the supplied collector's direct add body and check its whole valid domain.

One minimal harness checks invariant preservation; a separate regression oracle
retains the original behavioral postconditions. No vector wrappers are extracted.
All source changes occur in a clone, never in the input AlephBFT checkout.
"""
import argparse
import difflib
import json
import os
from pathlib import Path

from eval_verus_infer_alephbft_counting import library_counting, remove_update_block
from eval_verus_infer_alephbft_parents import (
    CORRECTED, FIXTURES, METHOD, ROOT, VALUE, item, prepare,
    remove_constructor_loop_annotation, validate_native,
)
from infer_experiment import save_results, snapshot_inputs, validate_run
from infer_eval_support import ArtifactGuard, digest, host_info, measured, tool_artifacts

STATE = "verification-infer/collector_state.rs"
PROPERTIES = "verification-infer/parents.rs"


def infer_add(source):
    source = library_counting(source, True)
    end = source.index("    pub fn add(&mut self")
    start = source.rindex("    #[cfg_attr(verus_keep_ghost, verus_spec(", 0, end)
    contract = source[start:end]
    before = item(source, "pub fn add(&mut self")[2]
    source = source[:start] + '    #[cfg_attr(verus_keep_ghost, verus_infer)]\n' + source[end:]
    harnesses = """
    // Minimal developer goal: every valid state can call add and stays valid.
    #[cfg(verus_keep_ghost)]
    #[verus_spec(requires old(self).valid(), ensures final(self).valid())]
    pub fn check_add_invariant(&mut self, creator: usize, payload: T, round: u16) {
        self.add(creator, payload, round);
    }

    // Experiment-only regression oracle, copied from the original add contract.
    #[cfg(verus_keep_ghost)]
""" + contract + """    pub fn check_add_behavior(&mut self, creator: usize, payload: T, round: u16) {
        self.add(creator, payload, round);
    }
"""
    closing = source.index("\n}\n\n#[cfg(test)]")
    source = source[:closing] + harnesses + source[closing:]
    if item(source, "pub fn add(&mut self")[2] != before:
        raise ValueError("inference or harness construction changed the executable add body")
    return source


def add_mutation(source, old, new):
    start, end, method = item(source, "pub fn add(&mut self")
    if method.count(old) != 1: raise ValueError(f"unexpected add mutation target: {old}")
    return source[:start] + method.replace(old, new, 1) + source[end:]


def validate(run, expected, state_count, unsupported=None, state_mode="exact"):
    data = validate_run(run, expected, allow_frontend=bool(unsupported))
    report = data.get("verus-infer", {})
    if unsupported:
        if report.get("status") != "unsupported" or unsupported not in run["stderr"]:
            raise ValueError("missing expected unsupported-source diagnostic")
        return data
    functions = report.get("functions", [])
    states = [f for f in functions if f.get("summary_kind") == "record_state"]
    if len(functions) != 2 + state_count or len(states) != state_count:
        raise ValueError("unexpected inferred helper count")
    for f in functions:
        state = f in states
        if not f["required_checks_passed"] or f["proofs"]["soundness"]["status"] != "proved":
            raise ValueError("missing ordinary candidate soundness proof")
        if f["mode"] != (state_mode if state else "exact"):
            raise ValueError("unexpected precision mode")
        for p in ("maximality", "exactness"):
            evidence = f["proofs"][p]
            required = not state or state_mode == "exact"
            if evidence["status"] != "proved" or evidence["required"] != required:
                raise ValueError("missing state-aware precision proofs or wrong requirement")
        if state and (f["metrics"]["cache"] != "miss" or f["metrics"]["cache_policy"] != "regenerated_state"):
            raise ValueError("record summaries must regenerate and reprove")
    return data


def require_domain_rejection(data, stderr):
    if "precondition not satisfied" not in stderr or not any(
        f.endswith("::check_add_invariant") for f in data["verus-infer"]["failed_functions"]
    ):
        raise ValueError("expected failure of whole-invariant-domain call coverage")


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
    extra = [Path(__file__), supplied, FIXTURES / "parents_properties.rs"]
    extra += [Path(__file__).with_name(p) for p in ("eval_verus_infer_alephbft_counting.py",
        "eval_verus_infer_alephbft_parents.py", "eval_verus_infer_alephbft.py", "infer_eval_support.py")]
    guard = ArtifactGuard(tool_artifacts(verus, z3, extra=extra))
    result = dict(host=host_info(), tool_artifacts=guard.initial, source={}, runs=[], native=[])
    checkout = output / "checkout"

    def save():
        save_results(output, result)

    def snapshot(name, native=False):
        relatives = [VALUE, METHOD, STATE, PROPERTIES]
        if native:
            relatives += ["verification-infer/native.rs", "verification-infer/native/Cargo.toml"]
        return snapshot_inputs(checkout, output / ("native-inputs" if native else "inputs") / name, relatives)

    try:
        shared, result["source"] = prepare(repo, checkout)
        (checkout / METHOD).write_text(shared.replace("(*self * 2) / 3 + NodeCount(1)", CORRECTED))
        original = supplied.read_text()
        inferred = infer_add(original)
        if item(remove_update_block(original), "pub fn add(&mut self")[2] != item(inferred, "pub fn add(&mut self")[2]:
            raise ValueError("direct add body differs from the supplied executable source")
        properties = (FIXTURES / "parents_properties.rs").read_text().replace("// BRIDGE_ATTRIBUTE", "#[verus_infer]")
        (checkout / PROPERTIES).write_text(properties)
        result["source"]["collector"] = dict(path=str(supplied), sha256=digest(supplied),
            origin="supplied manually verified working-tree core, not clean-main generic collector",
            executable_add_identical=True, extracted_vector_wrappers=0,
            removed_from_add="original handwritten contract and counting proof block",
            generated="guarded safety conditions and a final relation for every field",
            developer_goal="valid on entry implies safe call and valid on exit, for arbitrary creator/payload/round",
            regression_oracle="original add contract retained separately on check_add_behavior",
            retained="supplied invariant, state contracts on other methods, loop annotations and transition equality assertion",
            specification_equivalence="checked generic count equivalence retained from counting experiment")
        env = dict(os.environ, VERUS_Z3_PATH=str(z3), RUST_MIN_STACK="20971520", VERUS_INFER_CACHE=str(output / "candidate-cache"))
        cases = ("manual", "unassisted", "inferred", "inferred-warm", "wrong-slot", "wrong-round", "wrong-counter",
                 "duplicate-count", "changed-round-field", "missing-accounting", "extra-overflow-domain", "missing-loop-invariant", "explicit-sound")
        for name in cases:
            state = original if name == "manual" else inferred
            expected = name in ("manual", "inferred", "inferred-warm", "explicit-sound")
            enabled = name != "unassisted"
            unsupported = None
            if name == "wrong-slot": state = add_mutation(state, "self.candidates[creator] =", "self.candidates[0] =")
            if name == "wrong-round": state = add_mutation(state, "self.candidates[creator] = Some((payload, round))", "self.candidates[creator] = Some((payload, 0))")
            if name == "wrong-counter": state = add_mutation(state, "self.direct_parents += 1", "self.direct_parents += 2")
            if name == "duplicate-count": state = add_mutation(state, "previous_round < round", "previous_round <= round")
            if name == "changed-round-field": state = add_mutation(state, "self.direct_parents += 1;", "self.direct_parents += 1; self.for_round += 1;")
            if name == "missing-accounting": state = state.replace("#[verus_spec(requires old(self).valid(), ensures final(self).valid())]", "#[verus_spec(ensures final(self).valid())]", 1)
            if name == "extra-overflow-domain": state = add_mutation(state, "        if creator >=", "        let extra = creator + 1;\n        if creator >=")
            if name == "missing-loop-invariant": state = remove_constructor_loop_annotation(state)
            if name == "explicit-sound":
                state = state.replace('verus_infer)', 'verus_infer(precision = "sound"))', 1)
            (checkout / STATE).write_text(state)
            command = [str(verus), str(checkout / PROPERTIES), "--crate-type=lib", "--crate-name=aleph_records", "--no-cheating", "--output-json"]
            if enabled: command.append("--infer-goals")
            inputs = snapshot(name)
            guard.check(); run = measured(command, env, 60, ROOT); guard.check()
            (output / f"{name}.stdout.json").write_text(run["stdout"])
            (output / f"{name}.stderr.txt").write_text(run["stderr"])
            data = validate(run, expected, 0 if name == "manual" else 1, unsupported, "sound" if name == "explicit-sound" else "exact")
            if not unsupported and name in ("extra-overflow-domain", "missing-accounting"):
                require_domain_rejection(data, run["stderr"])
            if name == "inferred-warm" and any(f["metrics"]["cache"] != "hit" for f in data["verus-infer"]["functions"] if f["summary_kind"] != "record_state"):
                raise ValueError("warm scalar candidates were not reused and rechecked")
            result["runs"].append(dict(case=name, expected_success=expected,
                classification="unsupported_source" if unsupported else "verified" if expected else "unassisted_proof_failure" if name == "unassisted" else "semantic_rejection",
                command=command, returncode=run["returncode"], wall_micros=run["wall_micros"],
                verification=data["verification-results"], report=data.get("verus-infer"), **inputs))
            save(); print(f"{name}: expected outcome", flush=True)
        if args.native_tests:
            native_dir = checkout / "verification-infer/native"; native_dir.mkdir()
            manifest = native_dir / "Cargo.toml"
            manifest.write_text('[package]\nname="aleph-record-shared-core"\nversion="0.0.0"\nedition="2021"\n[workspace]\n[lib]\npath="../native.rs"\n[dependencies]\nderive_more = { version="1.0", features=["full"] }\n')
            _, _, bridge = item(properties, "pub fn consensus_threshold(")
            (checkout / "verification-infer/native.rs").write_text(
                '#[path="../crypto/src/node_count_value.rs"]\nmod node_count_value;\npub use node_count_value::NodeCount;\n#[path="../crypto/src/quorum_method.rs"]\nmod quorum_method;\n'
                + bridge + '\n#[path="collector_state.rs"]\nmod collector;\n')
            native_env = dict(os.environ); native_env.pop("RUSTUP_TOOLCHAIN", None)
            if args.native_target: native_env["CARGO_TARGET_DIR"] = str(args.native_target.resolve())
            native_cases = (("original", original), ("inferred", inferred),
                ("wrong-slot", add_mutation(inferred, "self.candidates[creator] =", "self.candidates[0] =")),
                ("wrong-round", add_mutation(inferred, "self.candidates[creator] = Some((payload, round))", "self.candidates[creator] = Some((payload, 0))")),
                ("extra-overflow-domain", add_mutation(inferred, "        if creator >=", "        let extra = creator + 1;\n        if creator >=")))
            for name, state in native_cases:
                (checkout / STATE).write_text(state)
                command = ["cargo", "test", "--offline", "--manifest-path", str(manifest), "--lib", "collector::tests", "--", "--nocapture"]
                inputs = snapshot(name, True)
                guard.check(); run = measured(command, native_env, 180, checkout); guard.check()
                (output / f"native-{name}.txt").write_text(run["stdout"] + run["stderr"])
                expected = name in ("original", "inferred")
                validate_native(run, expected, name)
                result["native"].append(dict(case=name, expected_success=expected, command=command, returncode=run["returncode"], wall_micros=run["wall_micros"], **inputs))
                save(); print(f"native {name}: expected outcome", flush=True)
            if (native_dir / "Cargo.lock").exists(): (output / "native-Cargo.lock").write_text((native_dir / "Cargo.lock").read_text())
        (checkout / STATE).write_text(inferred)
        (output / "record-inference.patch").write_text("".join(difflib.unified_diff(original.splitlines(keepends=True), inferred.splitlines(keepends=True), fromfile="a/collector_state.rs", tofile="b/collector_state.rs")))
        guard.check(); result["success"] = True
        (output / "summary.md").write_text(
            "# AlephBFT direct record inference\n\n"
            "The supplied manual collector's original executable add body is inferred directly, with no vector wrappers. "
            "Its handwritten contract is replaced by a checked exact state summary. "
            "A minimal ordinary harness proves safe-call coverage and invariant preservation for every valid state and arbitrary arguments. "
            "The original behavioral clauses remain separately as an experiment-only regression oracle. "
            "The counting-library equivalence, other state contracts, loop annotations and transition equality assertion remain. "
            "This is the supplied manual core, not clean main's NodeMap/Unit/Hasher collector.\n\n"
            + "\n".join(f"- {r['case']}: {r['classification']}" for r in result["runs"])
            + "\n\n" + "\n".join(f"- native {r['case']}: expected outcome" for r in result["native"])
            + "\n\nSemantic controls require successfully checked inferred helpers. "
            "The extra-overflow control adds a real unsafe operation; its exact contract identifies a smaller safe domain than requested. "
            "No new production bug is claimed. State precision is independently checked against the original body; timings are not a benchmark.\n")
    except Exception as error:
        result["success"] = False; result["error"] = str(error)
        raise
    finally: save()


if __name__ == "__main__": main()
