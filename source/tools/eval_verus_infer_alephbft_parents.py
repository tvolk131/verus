#!/usr/bin/env python3
"""Source-connected quorum/parent experiment; never edits the input checkout.

Clean-main native exploration and boundary counterexamples are distinct from
the formal comparison, which imports the existing manually verified collector.
The corrected production NodeCount method is shared with both builds.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

from eval_verus_infer_alephbft import REVISION, git
from infer_eval_support import ArtifactGuard, digest, host_info, measured, tool_artifacts

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = Path(__file__).with_name("infer_alephbft")
NODE = "crypto/src/node.rs"
COLLECTOR = "consensus/src/creation/collector.rs"
VALUE = "crypto/src/node_count_value.rs"
METHOD = "crypto/src/quorum_method.rs"
CORRECTED = "NodeCount((self.0 / 3) * 2 + ((self.0 % 3) * 2) / 3 + 1)"
MANUAL = """verus_spec(result => ensures
    result.0 as int == (2 * self.0 as int) / 3 + 1,
    result.0 > 0, self.0 > 0 ==> result.0 <= self.0,
    3 * result.0 as int > 2 * self.0 as int,
    3 * (result.0 as int - 1) <= 2 * self.0 as int,
)"""
BRIDGE_MANUAL = """#[verus_spec(result => ensures result as int == (2 * n as int) / 3 + 1)]"""


def item(text, marker):
    start = text.index(marker)
    opening = text.index("{", start)
    depth = 1
    end = opening + 1
    while depth:
        depth += (text[end] == "{") - (text[end] == "}")
        end += 1
    return start, end, text[start:end]


def prepare(repo, checkout):
    subprocess.run(["git", "clone", "--no-hardlinks", "--no-checkout", str(repo), str(checkout)],
                   check=True, capture_output=True, text=True)
    git(checkout, "checkout", "--detach", REVISION)
    original = git(checkout, "show", f"{REVISION}:{NODE}")
    start = original.index("/// Node count.")
    end = original.index("// deriving Mul", start)
    declaration = original[start:end]
    # Preserve all native derives. Ghost mode needs only Copy/Clone; operator
    # derives are not silently assigned specifications by this experiment.
    declaration = declaration.replace("#[derive(", "#[cfg_attr(not(verus_keep_ghost), derive(")
    declaration = declaration.replace(")]\npub struct", "))]\n#[cfg_attr(verus_keep_ghost, derive(Copy, Clone))]\n#[cfg_attr(verus_keep_ghost, verifier::verify)]\npub struct")
    value = ("#[cfg(not(verus_keep_ghost))]\nuse derive_more::{Add, AddAssign, From, Into, Sub, SubAssign, Sum};\n"
             "#[cfg(verus_keep_ghost)]\nuse vstd::prelude::*;\n" + declaration)
    (checkout / VALUE).write_text(value)
    edited = original[:start] + "pub use crate::node_count_value::NodeCount;\n\n" + original[end:]
    a, b, method = item(edited, "pub fn consensus_threshold(")
    doc = edited.rindex("    /// If this is the total node count", 0, a)
    (checkout / NODE).write_text(edited[:doc] + edited[b:])
    lib = checkout / "crypto/src/lib.rs"
    lib.write_text(lib.read_text() + "\nmod node_count_value;\nmod quorum_method;\n")
    shared = ("use crate::NodeCount;\n#[cfg(verus_keep_ghost)]\nuse vstd::prelude::*;\n"
              "#[cfg_attr(verus_keep_ghost, verus_verify)]\nimpl NodeCount {\n"
              "    #[cfg_attr(verus_keep_ghost, verus_infer)]\n    " + method + "\n}\n")
    if "(*self * 2) / 3 + NodeCount(1)" not in method:
        raise ValueError("unexpected pinned threshold implementation")
    (checkout / METHOD).write_text(shared)
    (checkout / "verification-infer").mkdir()
    return shared, dict(revision=REVISION, original_threshold=method,
                        original_node_sha256=hashlib.sha256(original.encode()).hexdigest(),
                        node_count_declaration="same public tuple field; native derives retained",
                        ghost_derives="Copy/Clone only; no assumed operator specifications")


def validate(run, expected, inference_count, unsupported=None):
    data = json.loads(run["stdout"])
    result = data["verification-results"]
    if (run["returncode"] == 0) != expected or result.get("success") != expected:
        raise ValueError(f"unexpected verification outcome: {result}")
    if "panicked at" in run["stderr"] or "resource limit" in run["stderr"].lower() or "timed out" in run["stderr"].lower():
        raise ValueError("panic or resource exhaustion is not semantic evidence")
    if unsupported:
        if data.get("verus-infer", {}).get("status") != "unsupported" or unsupported not in run["stderr"]:
            raise ValueError("missing expected source-capability diagnostic")
        return data
    if result.get("encountered-vir-error"):
        raise ValueError("frontend failure is not a semantic negative control")
    functions = data.get("verus-infer", {}).get("functions", [])
    if len(functions) != inference_count:
        raise ValueError("unexpected inferred function count")
    if any(not f["required_checks_passed"] or f["mode"] != "exact"
           or any(f["proofs"][p]["status"] != "proved" or not f["proofs"][p]["required"]
                  for p in ("soundness", "maximality", "exactness")) for f in functions):
        raise ValueError("missing mandatory inference checks")
    return data


def validate_native(run, expected, name):
    text = run["stdout"] + run["stderr"]
    if (run["returncode"] == 0) != expected or "Running unittests" not in text:
        raise ValueError(f"unexpected native outcome or compilation failure: {name}")
    if expected and not re.search(r"test result: ok\. [1-9][0-9]* passed; 0 failed", text):
        raise ValueError("native success must execute tests")
    if not expected and ("test result: FAILED" not in text or "panicked at" not in text):
        raise ValueError("native mutation needs a concrete failing test")
    if name.endswith("counterexample") and ("INFER_PARENT_COUNTEREXAMPLE" not in text or "with overflow" not in text):
        raise ValueError("missing concrete overflow evidence")
    return text


def remove_update_hint(source):
    start = source.index("                count_update(self.candidates@")
    end = source.index(";", start) + 1
    return source[:start] + source[end:]


def remove_constructor_loop_annotation(source):
    end = source.index("        while candidates.len() < node_count")
    start = source.rindex("        #[cfg_attr(verus_keep_ghost, verus_spec(", 0, end)
    # Retain termination evidence so the missing invariant reaches the prover,
    # rather than merely triggering the frontend's mandatory-decreases check.
    return (source[:start] + "        #[cfg_attr(verus_keep_ghost, verus_spec(\n"
            "            decreases node_count - candidates@.len(),\n        ))]\n" + source[end:])


def request_collector_inference(source):
    end = source.index("    pub fn add(&mut self")
    start = source.rindex("    #[cfg_attr(verus_keep_ghost, verus_spec(", 0, end)
    source = source[:start] + "    #[cfg_attr(verus_keep_ghost, verus_infer)]\n" + source[end:]
    # Remove the method's explicit ghost proof block as well as its contract.
    # The source-capability probe must not ask inference to accept proof code.
    method_start, method_end, method = item(source, "pub fn add(&mut self")
    proof_start = method.index("            #[cfg(verus_keep_ghost)]")
    _, proof_end, _ = item(method, "proof!")
    method = method[:proof_start] + method[proof_end:]
    return source[:method_start] + method + source[method_end:]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--alephbft", type=Path, required=True)
    parser.add_argument("--verified-collector", type=Path, help="existing shared production collector state.rs; defaults to the input working tree")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--verus", type=Path, default=ROOT / "source/target-verus/release/verus")
    parser.add_argument("--z3", type=Path, default=ROOT / "source/z3")
    parser.add_argument("--native-tests", action="store_true")
    parser.add_argument("--native-target", type=Path, help="optional reusable Cargo target directory")
    args = parser.parse_args()
    output = args.output.resolve()
    if output.exists() and any(output.iterdir()):
        parser.error("output directory must be new or empty")
    repo = args.alephbft.resolve()
    collector_path = (args.verified_collector or repo / "consensus/src/creation/collector/state.rs").resolve()
    if not collector_path.is_file():
        parser.error("provide the existing manually verified collector with --verified-collector")
    output.mkdir(parents=True, exist_ok=True)
    verus, z3 = args.verus.resolve(), args.z3.resolve()
    extra = [Path(__file__), Path(__file__).with_name("eval_verus_infer_alephbft.py"),
             Path(__file__).with_name("infer_eval_support.py"),
             FIXTURES / "parents_properties.rs", FIXTURES / "collector_native.rs", collector_path]
    guard = ArtifactGuard(tool_artifacts(verus, z3, extra=extra))
    result = dict(host=host_info(), tool_artifacts=guard.initial, runs=[], native=[], source={})
    checkout = output / "checkout"

    def save():
        (output / "results.json").write_text(json.dumps(result, indent=2) + "\n")

    def native(name, filters, expect_success, manifest=None):
        env = dict(os.environ); env.pop("RUSTUP_TOOLCHAIN", None)
        if args.native_target:
            env["CARGO_TARGET_DIR"] = str(args.native_target.resolve())
        package = ["--manifest-path", str(manifest)] if manifest else ["-p", "aleph-bft"]
        command = ["cargo", "test", "--offline", *package, "--lib", filters, "--", "--nocapture"]
        relatives = [NODE, COLLECTOR, VALUE, METHOD, "crypto/src/lib.rs"]
        if manifest:
            relatives += ["verification-infer/native.rs", "verification-infer/collector_state.rs", "verification-infer/native/Cargo.toml"]
        snapshot = output / "native-inputs" / name
        for relative in relatives:
            dest = snapshot / relative; dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_text((checkout / relative).read_text())
        guard.check()
        run = measured(command, env, 180, checkout)
        guard.check()
        text = run["stdout"] + run["stderr"]
        (output / f"native-{name}.txt").write_text(text)
        validate_native(run, expect_success, name)
        result["native"].append(dict(case=name, expected_success=expect_success, command=command,
                                     returncode=run["returncode"], wall_micros=run["wall_micros"],
                                     classification="observed_counterexample" if name.endswith("counterexample") else "test_passed" if expect_success else "negative_control_rejected",
                                     input_snapshot=str(snapshot), source_hashes={p: digest(snapshot / p) for p in relatives},
                                     exploration=re.findall(r"INFER_PARENT_EXPLORE[^\n]+", text),
                                     counterexamples=re.findall(r"INFER_PARENT_COUNTEREXAMPLE[^\n]+", text)))
        save(); print(f"native {name}: expected outcome", flush=True)

    try:
        shared, result["source"] = prepare(repo, checkout)
        collector = collector_path.read_text()
        result["source"]["manual_collector"] = dict(path=str(collector_path), sha256=digest(collector_path),
            origin="input working tree, not pinned clean main", executable_body_changes_by_runner=False,
            retained="existing contracts, counting definitions/lemmas, loop invariants and proof calls")
        original_collector = git(checkout, "show", f"{REVISION}:{COLLECTOR}")
        result["source"]["original_collector_sha256"] = hashlib.sha256(original_collector.encode()).hexdigest()
        probe = (FIXTURES / "collector_native.rs").read_text()
        (checkout / COLLECTOR).write_text(original_collector + "\n" + probe)
        corrected = shared.replace("(*self * 2) / 3 + NodeCount(1)", CORRECTED)
        goals = (FIXTURES / "parents_properties.rs").read_text()
        (checkout / "verification-infer/collector_state.rs").write_text(collector)
        result["source"]["runtime_fix"] = dict(original="(*self * 2) / 3 + NodeCount(1)", corrected=CORRECTED,
            reason="avoid n*2 overflow; preserve the NodeCount API", round_advance="unchanged; concrete overflow retained as a finding")
        if args.native_tests:
            native("original-sequences", "infer_parent_probe::bounded_actual_api_sequences", True)
            native("original-threshold-counterexample", "infer_parent_probe::original_threshold_has_a_concrete_overflow", True)
            native("original-round-counterexample", "infer_parent_probe::original_round_advance_has_a_concrete_overflow", True)
            (checkout / METHOD).write_text(corrected)
            native("corrected-boundaries", "infer_parent_probe::corrected_threshold_matches_boundary_oracle", True)
            native("corrected-production-caller", "infer_parent_probe::bounded_actual_api_sequences", True)
            native("corrected-existing-collector-tests", "creation::collector::tests", True)
            mutations = {
                "duplicate-count": ("Some((_, r)) if *r < round", "Some((_, r)) if *r <= round"),
                "stale-count": ("if round == self.for_round - 1", "if round <= self.for_round - 1"),
                "future-slot": ("if round >= self.for_round", "if round > self.for_round"),
                "missing-own-parent": ("_ => Err(ConstraintError::MissingOwnParent)", "_ => Ok(&self.candidates)"),
            }
            for name, (old, new) in mutations.items():
                if old not in original_collector: raise ValueError(f"mutation target missing: {name}")
                (checkout / COLLECTOR).write_text(original_collector.replace(old, new, 1) + "\n" + probe)
                native(name, "infer_parent_probe::bounded_actual_api_sequences", False)
            (checkout / COLLECTOR).write_text(original_collector + "\n" + probe)
            native_dir = checkout / "verification-infer/native"; native_dir.mkdir()
            native_manifest = native_dir / "Cargo.toml"
            native_manifest.write_text('[package]\nname="aleph-parent-shared-core"\nversion="0.0.0"\nedition="2021"\n'
                                       '[workspace]\n[lib]\npath="../native.rs"\n[dependencies]\n'
                                       'derive_more = { version="1.0", features=["full"] }\n')
            _, _, bridge = item(goals, "pub fn consensus_threshold(")
            (checkout / "verification-infer/native.rs").write_text(
                '#[path="../crypto/src/node_count_value.rs"]\nmod node_count_value;\npub use node_count_value::NodeCount;\n'
                '#[path="../crypto/src/quorum_method.rs"]\nmod quorum_method;\n'
                + bridge + '\n#[path="collector_state.rs"]\nmod collector;\n')
            native("manual-collector-core", "collector::tests", True, native_manifest)
            lock = native_dir / "Cargo.lock"
            if lock.exists():
                result["source"]["manual_core_native_lock_sha256"] = digest(lock)
                (output / "manual-core-Cargo.lock").write_text(lock.read_text())
        env = dict(os.environ, VERUS_Z3_PATH=str(z3), RUST_MIN_STACK="20971520",
                   VERUS_INFER_CACHE=str(output / "candidate-cache"))
        cases = ["manual", "inferred", "inferred-warm", "wrong-goal", "changed-threshold", "overflowing-threshold",
                 "missing-update-hint", "missing-loop-invariant", "wrong-collector-update", "collector-inference-unsupported"]
        for name in cases:
            source, state, properties = corrected, collector, goals
            expected = name in ("manual", "inferred", "inferred-warm")
            unsupported = None
            if name == "manual":
                source = source.replace("verus_infer", MANUAL)
                properties = properties.replace("// BRIDGE_ATTRIBUTE", BRIDGE_MANUAL)
            else:
                properties = properties.replace("// BRIDGE_ATTRIBUTE", "#[verus_infer]")
            if name == "wrong-goal": properties = properties.replace("/ 3 + 1,", "/ 3 + 2,", 1)
            if name == "changed-threshold": source = source.replace("/ 3 + 1)", "/ 3 + 2)", 1)
            if name == "overflowing-threshold": source = source.replace(CORRECTED, "NodeCount((self.0 * 2) / 3 + 1)")
            if name == "missing-update-hint": state = remove_update_hint(state)
            if name == "missing-loop-invariant": state = remove_constructor_loop_annotation(state)
            if name == "wrong-collector-update": state = state.replace("self.direct_parents += 1", "self.direct_parents += 2", 1)
            if name == "collector-inference-unsupported":
                state = request_collector_inference(state)
                unsupported = "calls require an acyclic same-crate inferred callee"
            (checkout / METHOD).write_text(source)
            (checkout / "verification-infer/collector_state.rs").write_text(state)
            path = checkout / "verification-infer/parents.rs"; path.write_text(properties)
            snapshot = output / "inputs" / name
            for relative in (VALUE, METHOD, "verification-infer/collector_state.rs", "verification-infer/parents.rs"):
                dest = snapshot / relative; dest.parent.mkdir(parents=True, exist_ok=True)
                dest.write_text((checkout / relative).read_text())
            command = [str(verus), str(path), "--crate-type=lib", "--crate-name=aleph_parents", "--no-cheating", "--output-json"]
            guard.check(); run = measured(command, env, 60, ROOT); guard.check()
            (output / f"{name}.stdout.json").write_text(run["stdout"])
            (output / f"{name}.stderr.txt").write_text(run["stderr"])
            data = validate(run, expected, 0 if name == "manual" else 2, unsupported)
            if name == "inferred-warm" and any(f["metrics"]["cache"] != "hit" for f in data["verus-infer"]["functions"]):
                raise ValueError("warm comparison did not reuse candidates")
            result["runs"].append(dict(case=name, expected_success=expected, classification="unsupported_source" if unsupported else "verified" if expected else "semantic_rejection",
                command=command, returncode=run["returncode"], wall_micros=run["wall_micros"], verification=data["verification-results"],
                report=data.get("verus-infer"), input_snapshot=str(snapshot),
                handwritten_production_threshold_contracts=1 if name == "manual" else 0,
                handwritten_adapter_contracts=1 if name == "manual" else 0,
                collector_proofs_retained=name not in ("missing-update-hint", "missing-loop-invariant", "collector-inference-unsupported")))
            save(); print(f"{name}: expected outcome", flush=True)
        # Probe the actual pinned Mul implementation independently. Verus trait
        # preconditions cannot be replaced by an implementation's inferred P.
        original_node = git(checkout, "show", f"{REVISION}:{NODE}")
        _, _, mul = item(original_node, "impl Mul<usize> for NodeCount")
        mul = mul.replace("    fn mul(", "    #[verus_infer]\n    fn mul(", 1)
        operator = ("use vstd::prelude::*;\nuse std::ops::Mul;\n"
                    '#[path="../crypto/src/node_count_value.rs"]\nmod node_count_value;\n'
                    "pub use node_count_value::NodeCount;\n#[verus_verify]\n" + mul)
        path = checkout / "verification-infer/operator.rs"; path.write_text(operator)
        snapshot = output / "inputs/operator-inference-unsupported"
        for relative in (VALUE, "verification-infer/operator.rs"):
            dest = snapshot / relative; dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_text((checkout / relative).read_text())
        command = [str(verus), str(path), "--crate-type=lib", "--no-cheating", "--output-json"]
        guard.check(); run = measured(command, env, 60, ROOT); guard.check()
        (output / "operator-unsupported.stdout.json").write_text(run["stdout"])
        (output / "operator-unsupported.stderr.txt").write_text(run["stderr"])
        data = validate(run, False, 0, "preconditions are inherited from the trait")
        result["runs"].append(dict(case="operator-inference-unsupported", classification="unsupported_source",
            expected_success=False, command=command, returncode=run["returncode"], verification=data["verification-results"],
            report=data.get("verus-infer"), input_snapshot=str(snapshot)))
        save()
        # Preserve the successful inference variant as the final reviewable patch.
        (checkout / METHOD).write_text(corrected)
        (checkout / "verification-infer/collector_state.rs").write_text(collector)
        (checkout / "verification-infer/parents.rs").write_text(goals.replace("// BRIDGE_ATTRIBUTE", "#[verus_infer]"))
        patch = git(checkout, "diff", "--", NODE, "crypto/src/lib.rs", COLLECTOR)
        for relative in (VALUE, METHOD):
            patch += subprocess.run(["git", "diff", "--no-index", "/dev/null", relative], cwd=checkout, capture_output=True, text=True).stdout
        (output / "production-and-probes.patch").write_text(patch)
        guard.check(); result["success"] = True
        (output / "summary.md").write_text(
            "# AlephBFT quorum and parent experiment\n\n"
            "Clean-main native probes use pinned production types, operators, collector and callers. "
            "The full-domain threshold fix is a real runtime change, kept separate from inference.\n\n"
            "The formal comparison imports the existing manually verified production collector from the input working tree, "
            "not the clean-main generic collector. Its state contracts, counting proofs and loop annotations remain. "
            "A verified scalar adapter connects that collector to the corrected production NodeCount API.\n\n"
            "Inference replaces one five-clause production threshold contract and one experiment adapter contract. "
            "An unrestricted caller proves the corrected threshold accepts every usize input. "
            "Removing the collector update hint or constructor loop invariant is expected to fail. "
            "General collector inference remains unsupported. No operator trait semantics are assumed.\n\n"
            + "\n".join(f"- {r['case']}: {r['classification']} (exit {r['returncode']})" for r in result["runs"])
            + "\n\n" + "\n".join(f"- native {r['case']}: {r['classification']} (exit {r['returncode']})" for r in result["native"])
            + "\n\nTiming observations are not a performance benchmark. Bounded exploration is not a universal proof.\n")
    except Exception as error:
        result["success"] = False; result["error"] = str(error)
        raise
    finally:
        save()


if __name__ == "__main__":
    main()
