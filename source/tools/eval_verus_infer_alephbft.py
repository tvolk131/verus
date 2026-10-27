#!/usr/bin/env python3
"""Reproduce the pinned, source-connected AlephBFT demand-driven proof pilot.

The input checkout is read-only. A local clone ignores its uncommitted changes.
Only the original common_vote function is moved to a shared production module;
its executable body is checked byte-for-byte. No network access is required.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

from infer_eval_support import ArtifactGuard, digest, host_info, measured, tool_artifacts

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = Path(__file__).with_name("infer_alephbft")
REVISION = "f35c7bba6d628bcafdf0d65dd471cc6f868ec034"
ELECTION = "consensus/src/extension/election.rs"
SHARED = "consensus/src/extension/common_vote.rs"
LEMMA = "assert(round % 2u16 == (round & 1u16)) by(bit_vector);"
MANUAL_CONTRACT = "verus_spec(result => ensures result == (relative_round != 3 && (relative_round <= 4 || relative_round % 2 == 1)))"


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args], text=True)


def prepare(repo, checkout):
    subprocess.run(["git", "clone", "--no-hardlinks", "--no-checkout", str(repo), str(checkout)],
                   check=True, capture_output=True, text=True)
    git(checkout, "checkout", "--detach", REVISION)
    original = git(checkout, "show", f"{REVISION}:{ELECTION}")
    types = git(checkout, "show", f"{REVISION}:types/src/lib.rs")
    if "pub type Round = u16;" not in types.splitlines():
        raise ValueError("production Round alias changed")
    start = original.index("fn common_vote(")
    end = original.index("\nenum CandidateOutcome", start)
    function = original[start:end].rstrip()
    shared = ("use crate::Round;\n\n#[cfg(verus_keep_ghost)]\nuse vstd::prelude::*;\n\n"
              "#[cfg_attr(verus_keep_ghost, verus_infer)]\n"
              + function.replace("fn common_vote(", "pub(super) fn common_vote(", 1) + "\n")
    if shared[shared.index("fn common_vote("):].rstrip() != function:
        raise ValueError("mechanical split changed the executable function")
    (checkout / SHARED).write_text(shared)
    (checkout / ELECTION).write_text(original[:start]
                                   + '#[path = "common_vote.rs"]\nmod common_vote;\nuse common_vote::common_vote;\n'
                                   + original[end:])
    (checkout / "verification-infer").mkdir()
    return shared, dict(revision=REVISION, original_path=ELECTION, shared_path=SHARED,
                        original_function_sha256=hashlib.sha256(function.encode()).hexdigest(),
                        shared_executable_identical=True, round_alias="u16")


def validate(run, expected, inferred):
    data = json.loads(run["stdout"])
    result = data["verification-results"]
    if (run["returncode"] == 0) != expected or result.get("success") != expected:
        raise ValueError(f"unexpected verification outcome: {result}")
    if result.get("encountered-vir-error"):
        raise ValueError("frontend error is not an expected semantic negative control")
    if "resource limit" in run["stderr"].lower() or "timed out" in run["stderr"].lower():
        raise ValueError("resource exhaustion is not an expected semantic failure")
    inference = data.get("verus-infer", {})
    if inferred:
        functions = inference["functions"]
        if len(functions) != 1 or not functions[0]["required_checks_passed"]:
            raise ValueError("missing verified exact contract")
        f = functions[0]
        if f["mode"] != "exact" or f["contract"]["requires"] != "true":
            raise ValueError("pilot changed precision or restricted the original input domain")
        if any(f["proofs"][p]["status"] != "proved" or not f["proofs"][p]["required"]
               for p in ("soundness", "maximality", "exactness")):
            raise ValueError("missing required inference proof")
    return data


def native_errors(text):
    messages = re.findall(r"^error(?:\[E\d+\])?:[^\n]*", text, re.MULTILINE)
    return dict(messages=[m for m in messages if not m.startswith("error: could not compile")],
                sites=re.findall(r"(error(?:\[E\d+\])?:[^\n]+)\n\s*--> ([^\n]+)", text))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--alephbft", type=Path, required=True, help="local AlephBFT repository containing the pinned commit")
    parser.add_argument("--verus", type=Path, default=ROOT / "source/target-verus/release/verus")
    parser.add_argument("--z3", type=Path, default=ROOT / "source/z3")
    parser.add_argument("--output", type=Path, required=True, help="new or empty output directory")
    parser.add_argument("--native-tests", action="store_true", help="also test the real production election module in both feature configurations")
    args = parser.parse_args()
    output = args.output.resolve()
    if output.exists() and any(output.iterdir()):
        parser.error("output directory must be empty")
    output.mkdir(parents=True, exist_ok=True)
    checkout = output / "checkout"
    verus, z3 = args.verus.resolve(), args.z3.resolve()
    guard = ArtifactGuard(tool_artifacts(verus, z3, extra=[Path(__file__), FIXTURES / "properties.rs"]))
    result = dict(host=host_info(), source={}, tool_artifacts=guard.initial, runs=[], native=[])

    def save():
        (output / "results.json").write_text(json.dumps(result, indent=2) + "\n")

    try:
        shared, result["source"] = prepare(args.alephbft.resolve(), checkout)
        goals = (FIXTURES / "properties.rs").read_text()
        env = dict(os.environ, VERUS_Z3_PATH=str(z3), RUST_MIN_STACK="20971520")
        cases = [
            ("manual-contract-and-lemma", True, False, True, False, None),
            ("inferred-baseline", False, True, False, False, None),
            ("inferred-manual-lemma", True, True, True, False, None),
            ("inferred-assisted", True, True, False, True, None),
            ("wrong-goal", False, True, False, True, "goal"),
            ("changed-production-body", False, True, False, True, "source"),
            ("manual-changed-production-body", False, False, True, False, "source"),
            ("missing-branch-assumption", False, True, False, True, "branch"),
            ("unsafe-round-increment", False, True, False, True, "overflow"),
        ]
        base_contract = None
        for name, expected, inferred, manual, assisted, mutation in cases:
            source = shared if inferred else shared.replace("verus_infer", MANUAL_CONTRACT)
            properties = goals.replace("// MANUAL_LEMMA", LEMMA if manual else "")
            if mutation == "goal":
                properties = properties.replace("assert(vote == (round != 3", "assert(vote != (round != 3", 1)
            elif mutation == "source":
                source = source.replace("relative_round == 3", "relative_round == 2", 1)
            elif mutation == "branch":
                properties = properties.replace("if round >= 5 {", "if true {", 1)
            elif mutation == "overflow":
                properties = properties.replace("requires round >= 5, round < u16::MAX,", "requires round >= 5,", 1)
            (checkout / SHARED).write_text(source)
            path = checkout / "verification-infer" / f"{name}.rs"
            path.write_text(properties)
            snapshot = output / "inputs" / name
            (snapshot / Path(SHARED).parent).mkdir(parents=True)
            (snapshot / path.relative_to(checkout).parent).mkdir(parents=True)
            (snapshot / SHARED).write_text(source)
            (snapshot / path.relative_to(checkout)).write_text(properties)
            command = [str(verus), str(path), "--crate-type=lib", "--crate-name=aleph_infer_pilot",
                       "--no-cheating", "--output-json"]
            if assisted:
                command.append("--infer-goals")
            guard.check()
            run = measured(command, dict(env, VERUS_INFER_CACHE_DIR=str(output / "cache" / name)), 60, ROOT)
            guard.check()
            (output / f"{name}.stdout.json").write_text(run["stdout"])
            (output / f"{name}.stderr.txt").write_text(run["stderr"])
            data = validate(run, expected, inferred)
            if inferred:
                contract = data["verus-infer"]["functions"][0]["contract"]
                if base_contract is None:
                    base_contract = contract
                elif mutation == "source":
                    if contract == base_contract:
                        raise ValueError("production mutation did not change the freshly verified contract")
                elif contract != base_contract:
                    raise ValueError("proof assistance changed the base inferred contract")
            row = dict(case=name, expected_success=expected, assisted=assisted,
                       manual_intermediate_lemmas=properties.count(LEMMA),
                       handwritten_callee_contracts=0 if inferred else 1,
                       command=command, returncode=run["returncode"], wall_micros=run["wall_micros"],
                       verification=data["verification-results"], report=data.get("verus-infer"),
                       verus=data.get("verus"), input_snapshot=str(snapshot),
                       shared_source_sha256=digest(snapshot / SHARED),
                       properties_sha256=digest(path))
            result["runs"].append(row)
            save()
            print(f"{name}: {'verified' if expected else 'rejected as expected'}", flush=True)
        (checkout / SHARED).write_text(shared)
        # Save the small source integration diff, including the new shared file.
        diff = git(checkout, "diff", "--", ELECTION)
        new = subprocess.run(["git", "diff", "--no-index", "/dev/null", SHARED], cwd=checkout,
                             capture_output=True, text=True)
        (output / "production.patch").write_text(diff + new.stdout)
        if args.native_tests:
            native_env = dict(os.environ)
            native_env.pop("RUSTUP_TOOLCHAIN", None)
            result["native_toolchain"] = subprocess.check_output(["cargo", "--version"], cwd=checkout,
                                                                 env=native_env, text=True).strip()
            for default in (True, False):
                command = ["cargo", "test", "--offline", "-p", "aleph-bft", "extension::election", "--lib"]
                if not default:
                    command.append("--no-default-features")
                run = measured(command, native_env, 180, checkout)
                name = "default" if default else "no-default-features"
                (output / f"native-{name}.txt").write_text(run["stdout"] + run["stderr"])
                if run["returncode"] == 0 and "6 passed; 0 failed" in run["stdout"]:
                    result["native"].append(dict(features=name, command=command, returncode=0,
                                                 status="6_tests_passed", wall_micros=run["wall_micros"]))
                    print(f"native {name}: 6 election tests passed", flush=True)
                else:
                    # An untouched baseline build failure is a reported limit,
                    # never a successful test or a reason to alter the protocol.
                    patched = (checkout / ELECTION).read_text()
                    try:
                        (checkout / ELECTION).write_text(git(checkout, "show", f"{REVISION}:{ELECTION}"))
                        baseline = measured(command, native_env, 180, checkout)
                    finally:
                        (checkout / ELECTION).write_text(patched)
                    (output / f"native-{name}-unmodified.txt").write_text(baseline["stdout"] + baseline["stderr"])
                    errors = native_errors(run["stderr"])
                    if not errors["messages"] or baseline["returncode"] == 0 or errors != native_errors(baseline["stderr"]):
                        raise ValueError(f"new native build/test failure: {name}")
                    result["native"].append(dict(features=name, command=command, returncode=run["returncode"],
                                                 status="unmodified_baseline_build_failure",
                                                 error_signatures=errors, wall_micros=run["wall_micros"]))
                    print(f"native {name}: unmodified baseline has the same {len(errors['messages'])} compilation errors", flush=True)
        guard.check()
        result["success"] = True
        (output / "summary.md").write_text(
            "# AlephBFT demand-driven pilot\n\n"
            f"Pinned source: `{REVISION}`. The shared executable body is unchanged.\n\n"
            "The canonical remainder schedule and alternation already verify with inference.\n"
            "The equivalent low-bit parity formulation needs two handwritten local lemma invocations\n"
            "or the experimental `--infer-goals` strategy. It does not establish a new consensus theorem.\n\n"
            + "\n".join(f"- {r['case']}: expected {'success' if r['expected_success'] else 'rejection'}; "
                         f"observed exit {r['returncode']}; {r['wall_micros']/1e6:.3f}s"
                         for r in result["runs"])
            + "\n\n" + "\n".join(f"- Native {r['features']}: {r['status']}" for r in result["native"])
            + "\n\nTimings are single local observations, not a performance benchmark.\n")
    except Exception as error:
        result["success"] = False
        result["error"] = str(error)
        print(str(error), file=sys.stderr)
    finally:
        save()
    return 0 if result.get("success") else 1


if __name__ == "__main__":
    sys.exit(main())
