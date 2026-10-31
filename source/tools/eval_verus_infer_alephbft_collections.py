#!/usr/bin/env python3
"""Check inferred Vec summaries against Aleph's supplied counting invariant.

The input checkout is read-only. This mechanically extracts three operations
from the existing manually verified collector in an isolated pinned clone.
It does not synthesize the collector invariant or its induction lemmas.
"""
import argparse
import difflib
import json
import os
from pathlib import Path

from eval_verus_infer_alephbft_parents import (
    CORRECTED, FIXTURES, METHOD, ROOT, VALUE, item, prepare,
    remove_update_hint, validate_native,
)
from infer_experiment import save_results, snapshot_inputs, validate_run
from infer_eval_support import ArtifactGuard, digest, host_info, measured, tool_artifacts

OPS = "verification-infer/collection_ops.rs"
STATE = "verification-infer/collector_state.rs"
PROPERTIES = "verification-infer/parents.rs"


def extract_operations(source):
    start, end, method = item(source, "pub fn add(&mut self")
    changes = (
        ("self.candidates.len()", "crate::collection_ops::length(&self.candidates)"),
        ("match self.candidates[creator]", "match crate::collection_ops::read(&self.candidates, creator)"),
        ("self.candidates[creator] = Some((payload, round));",
         "crate::collection_ops::replace(&mut self.candidates, creator, Some((payload, round)));"),
    )
    for old, new in changes:
        if method.count(old) != 1:
            raise ValueError(f"unexpected supplied collector operation: {old}")
        method = method.replace(old, new)
    return source[:start] + method + source[end:]


def manual_operations(source):
    source = source.replace("verus_infer)]\npub fn length", "verus_spec(result => ensures result == v@.len()))]\npub fn length")
    source = source.replace("verus_infer)]\npub fn read", "verus_spec(result => requires i < v@.len(), ensures result == v@[i as int]))]\npub fn read")
    return source.replace('verus_infer(precision = "sound")',
                          "verus_spec(requires i < old(v)@.len(), ensures final(v)@ == old(v)@.update(i as int, value))")


def validate(run, expected, collection_count, unsupported=None):
    data = validate_run(run, expected, allow_frontend=bool(unsupported))
    report = data.get("verus-infer", {})
    if unsupported:
        if report.get("status") != "unsupported" or unsupported not in run["stderr"]:
            raise ValueError("missing expected source-capability diagnostic")
        return data
    functions = report.get("functions", [])
    collections = [f for f in functions if "collection_ops::" in f["function"]]
    if len(collections) != collection_count or len(functions) != 2 + collection_count:
        raise ValueError("unexpected inferred helper count")
    for f in functions:
        if not f["required_checks_passed"] or f["proofs"]["soundness"]["status"] != "proved":
            raise ValueError("missing mandatory helper proof")
        state = f["function"].endswith("collection_ops::replace")
        if f["mode"] != ("sound" if state else "exact"):
            raise ValueError("unexpected helper precision mode")
        for p in ("maximality", "exactness"):
            evidence = f["proofs"][p]
            if state:
                if (evidence["status"] != "not_established" or evidence["required"]
                        or evidence["reason"] != "state_precision_not_supported"):
                    raise ValueError("state precision must not be claimed")
            elif evidence["status"] != "proved" or not evidence["required"]:
                raise ValueError("missing exact helper precision proof")
        if f in collections and (f["metrics"]["cache"] != "miss"
                                  or f["metrics"]["cache_policy"] != "regenerated_template"):
            raise ValueError("collection candidates must regenerate and be rechecked")
    return data


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
    if output.exists() and any(output.iterdir()):
        parser.error("output directory must be new or empty")
    repo = args.alephbft.resolve()
    supplied = (args.verified_collector or repo / "consensus/src/creation/collector/state.rs").resolve()
    if not supplied.is_file():
        parser.error("provide the existing manually verified collector")
    output.mkdir(parents=True, exist_ok=True)
    verus, z3 = args.verus.resolve(), args.z3.resolve()
    extra = [Path(__file__), Path(__file__).with_name("eval_verus_infer_alephbft_parents.py"),
             Path(__file__).with_name("eval_verus_infer_alephbft.py"),
             Path(__file__).with_name("infer_eval_support.py"), supplied,
             FIXTURES / "collection_ops.rs", FIXTURES / "parents_properties.rs"]
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
        operations = (FIXTURES / "collection_ops.rs").read_text()
        properties = (FIXTURES / "parents_properties.rs").read_text().replace("// BRIDGE_ATTRIBUTE", "#[verus_infer]")
        properties += '\n#[path="collection_ops.rs"]\npub mod collection_ops;\n'
        (checkout / PROPERTIES).write_text(properties)
        result["source"]["collector"] = dict(path=str(supplied), sha256=digest(supplied),
            origin="existing manually verified input working tree; not pinned clean-main collector",
            transformation="extract length/read/replace in add into three shared generic helpers",
            retained="all collector contracts, counting definitions and induction lemmas, loop invariants, and proof calls",
            existing_collector_proof_annotations_removed=0,
            experiment_wrapper_contracts_replaced=3)
        env = dict(os.environ, VERUS_Z3_PATH=str(z3), RUST_MIN_STACK="20971520",
                   VERUS_INFER_CACHE=str(output / "candidate-cache"))
        cases = ("baseline", "manual-wrappers", "inferred", "inferred-warm", "inferred-goals",
                 "wrong-slot", "wrong-round", "wrong-counter", "missing-update-hint",
                 "extra-helper-effect", "state-exact-unsupported")
        for name in cases:
            state, ops = extracted, operations
            expected = name in cases[:5]
            unsupported = None
            if name == "baseline": state = original; ops = ""
            if name == "manual-wrappers": ops = manual_operations(operations)
            if name == "wrong-slot": state = state.replace("&mut self.candidates, creator,", "&mut self.candidates, 0,", 1)
            if name == "wrong-round": state = state.replace("&mut self.candidates, creator, Some((payload, round))", "&mut self.candidates, creator, Some((payload, 0))", 1)
            if name == "wrong-counter": state = state.replace("self.direct_parents += 1", "self.direct_parents += 2", 1)
            if name == "missing-update-hint": state = remove_update_hint(state)
            if name == "extra-helper-effect":
                ops = ops.replace("v[i] = value;", "v[i] = value; v[0] = value;")
                unsupported = "collection"
            if name == "state-exact-unsupported":
                ops = ops.replace('verus_infer(precision = "sound")', "verus_infer")
                unsupported = "sound"
            (checkout / STATE).write_text(state); (checkout / OPS).write_text(ops)
            command = [str(verus), str(checkout / PROPERTIES), "--crate-type=lib", "--crate-name=aleph_collections",
                       "--no-cheating", "--output-json"]
            if name == "inferred-goals": command.append("--infer-goals")
            inputs = snapshot(name)
            guard.check(); run = measured(command, env, 60, ROOT); guard.check()
            (output / f"{name}.stdout.json").write_text(run["stdout"])
            (output / f"{name}.stderr.txt").write_text(run["stderr"])
            data = validate(run, expected, 0 if name in ("baseline", "manual-wrappers") else 3, unsupported)
            if name == "inferred-warm" and any(
                    f["metrics"]["cache"] != "hit" for f in data["verus-infer"]["functions"]
                    if "collection_ops::" not in f["function"]):
                raise ValueError("warm scalar helpers must reuse candidates and repeat their proofs")
            result["runs"].append(dict(case=name, expected_success=expected,
                classification="unsupported_source" if unsupported else "verified" if expected else "semantic_rejection",
                command=command, returncode=run["returncode"], wall_micros=run["wall_micros"],
                verification=data["verification-results"], report=data.get("verus-infer"), **inputs))
            save(); print(f"{name}: expected outcome", flush=True)
        # Native tests compile exactly the same extracted executable functions.
        (checkout / OPS).write_text(operations)
        (checkout / STATE).write_text(extracted)
        if args.native_tests:
            native_dir = checkout / "verification-infer/native"; native_dir.mkdir()
            manifest = native_dir / "Cargo.toml"
            manifest.write_text('[package]\nname="aleph-collection-shared-core"\nversion="0.0.0"\nedition="2021"\n'
                                '[workspace]\n[lib]\npath="../native.rs"\n[dependencies]\n'
                                'derive_more = { version="1.0", features=["full"] }\n')
            _, _, bridge = item(properties, "pub fn consensus_threshold(")
            (checkout / "verification-infer/native.rs").write_text(
                '#[path="../crypto/src/node_count_value.rs"]\nmod node_count_value;\npub use node_count_value::NodeCount;\n'
                '#[path="../crypto/src/quorum_method.rs"]\nmod quorum_method;\n'
                + bridge + '\n#[path="collector_state.rs"]\nmod collector;\n'
                '#[path="collection_ops.rs"]\nmod collection_ops;\n')
            native_env = dict(os.environ); native_env.pop("RUSTUP_TOOLCHAIN", None)
            if args.native_target: native_env["CARGO_TARGET_DIR"] = str(args.native_target.resolve())
            for name, state in (("baseline", original), ("extracted", extracted),
                                ("wrong-slot", extracted.replace("&mut self.candidates, creator,", "&mut self.candidates, 0,", 1)),
                                ("wrong-round", extracted.replace("&mut self.candidates, creator, Some((payload, round))", "&mut self.candidates, creator, Some((payload, 0))", 1))):
                (checkout / STATE).write_text(state)
                command = ["cargo", "test", "--offline", "--manifest-path", str(manifest), "--lib", "collector::tests", "--", "--nocapture"]
                inputs = snapshot(name, native=True)
                guard.check(); run = measured(command, native_env, 180, checkout); guard.check()
                (output / f"native-{name}.txt").write_text(run["stdout"] + run["stderr"])
                expected = name in ("baseline", "extracted")
                validate_native(run, expected, name)
                result["native"].append(dict(case=name, expected_success=expected,
                    classification="test_passed" if expected else "negative_control_rejected",
                    command=command, returncode=run["returncode"], wall_micros=run["wall_micros"], **inputs))
                save(); print(f"native {name}: expected outcome", flush=True)
            lock = native_dir / "Cargo.lock"
            if lock.exists(): (output / "native-Cargo.lock").write_text(lock.read_text())
        (checkout / STATE).write_text(extracted)
        patch = "".join(difflib.unified_diff(original.splitlines(keepends=True), extracted.splitlines(keepends=True),
                                           fromfile="a/collector_state.rs", tofile="b/collector_state.rs"))
        patch += "".join(difflib.unified_diff([], operations.splitlines(keepends=True),
                                            fromfile="/dev/null", tofile="b/collection_ops.rs"))
        (output / "collection-extraction.patch").write_text(patch)
        guard.check(); result["success"] = True
        (output / "summary.md").write_text(
            "# AlephBFT collection-summary experiment\n\n"
            "The supplied collector is the existing manually verified working-tree core, not clean main. "
            "Its add method calls extracted generic Vec length/read/replace helpers in both native and Verus builds.\n\n"
            "Inference replaces three experiment wrapper contracts. It removes zero existing collector proof annotations. "
            "The collector invariant, recursive counting lemmas, proof calls and loop annotations remain explicit and checked. "
            "Length/read get all three exact-mode proofs. Replace gets an ordinary soundness proof; "
            "state maximality and exactness are explicitly not established. No new axioms or external bodies are added.\n\n"
            + "\n".join(f"- {r['case']}: {r['classification']}" for r in result["runs"])
            + "\n\n" + "\n".join(f"- native {r['case']}: {r['classification']}" for r in result["native"])
            + "\n\nSemantic mutations fail while all inferred helper obligations pass. "
            "Unsupported shapes are classified separately. Timing observations are not a benchmark.\n")
    except Exception as error:
        result["success"] = False; result["error"] = str(error)
        raise
    finally:
        save()


if __name__ == "__main__":
    main()
