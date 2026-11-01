#!/usr/bin/env python3
"""Run real source mutations in temporary copies; never edit the checkout.

Run from any directory with the repository's Rust toolchain and cached Cargo
dependencies available. --policy-only runs the dependency-free acceptance core.
"""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "source"
POLICY_SOURCE = "vir/src/infer_proof/policy.rs"
POLICY_MUTATIONS = [
    ("missing soundness", "passed[0] && (allow_approx || (passed[1] && passed[2]))", "allow_approx || (passed[1] && passed[2])"),
    ("missing precision", "passed[0] && (allow_approx || (passed[1] && passed[2]))", "passed[0]"),
    ("later success repairs failure", "self.rejected |= outcome != Outcome::Valid;", "self.rejected = outcome != Outcome::Valid;"),
    ("solver timeout ignored", "if timed_out {", "if false {"),
    ("solver failure ignored", "else if unproved {", "else if false {"),
    ("wrong candidate accepted", "if id != &self.id {", "if false {"),
    ("duplicate receipt accepted", "if slot.is_some() {", "if false {"),
    ("poisoned evidence accepted", "!self.poisoned && o == Some(Outcome::Valid)", "o == Some(Outcome::Valid)"),
    ("no query treated as success", "if !self.checked {\n            Outcome::Skipped", "if !self.checked {\n            Outcome::Valid"),
]
VIR_MUTATION_GROUPS = [
    ("Precision", "vir/src/infer_checks.rs", "infer_checks::tests", [
        ("maximality assumes its own precondition", "check.reqs = Arc::new(vec![]);", "check.reqs = original.reqs.clone();"),
        ("exactness examines only the actual output", "let other = SpannedTyped::new(span, typ, ExpX::Var(name.clone()));", "let other = SpannedTyped::new(span, typ, ExpX::Var(dest.clone()));"),
        ("precision body accepted as ordinary input", "validate_query_kind(original, None, span)?;", ""),
        ("missing counted-loop termination measure accepted", "|| invs.iter().any(|inv| !inv.at_entry || !inv.at_exit)\n                || decrease.len() != 1", "|| invs.iter().any(|inv| !inv.at_entry || !inv.at_exit)\n                || false"),
        ("missing native-loop termination measure accepted", "if cond.is_some() || decrease.len() != 1 || invs.len() < 3 || !total {", "if cond.is_some() || false || invs.len() < 3 || !total {"),
    ]),
    ("Loop", "vir/src/infer/loops.rs", "infer::loops::tests", [
        ("executable projection check omitted", "if encode(&projected)? != encode(original.x.body.as_ref().unwrap())? {", "if false {"),
        ("termination assumption admitted", "|| f.attrs.exec_assume_termination", "|| false"),
        ("termination suppression admitted", "|| f.attrs.exec_allows_no_decreases_clause", "|| false"),
        ("non-unit counter update admitted", "constant(&counter_step, 1)", "true"),
        ("wrong counter direction admitted", "counter_sub != countdown ||", "false ||"),
    ]),
    ("Patterns", "vir/src/infer/enums.rs", "infer::coverage_tests", [
        ("match guard ignored", "let test = logical(span, LogicalOp::And, pattern, condition)?;", "let test = logical(span, LogicalOp::And, pattern, boolean(span, true))?;"),
        ("or-pattern binding priority reversed", "summary::conditional(span, left.clone(), a, b)?", "summary::conditional(span, left.clone(), b, a)?"),
    ]),
    ("Views", "vir/src/infer/slices.rs", "infer::coverage_tests", [
        ("subslice length reversed", "view.end,\n            view.start,", "view.start,\n            view.end,"),
        ("subslice index offset subtracted", "BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Allow)),\n            view.start,", "BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)),\n            view.start,"),
    ]),
    ("Bit-vector hints", "vir/src/infer/bitvector_hints.rs", "infer::bitvector_hints::tests", [
        ("missing auxiliary theorem", "ensures: Arc::new(vec![fact.clone()])", "ensures: Arc::new(vec![])"),
        ("disconnected assumed theorem", "StmX::Assume(fact)", "StmX::Assume(fact.new_x(ExpX::Const(Constant::Bool(false))))"),
        ("check/use identity bypassed", "&& Arc::ptr_eq(&ensures[0], used)", "&& true"),
    ]),
]


def compile_tests(command, cwd, env):
    result = subprocess.run(command, cwd=cwd, env=env, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(f"Mutation-test compilation failed: {command}\n{result.stdout}{result.stderr}")
    return result.stdout


def run(command, cwd, env, *, should_pass):
    result = subprocess.run(command, cwd=cwd, env=env, text=True, capture_output=True)
    output = result.stdout + result.stderr
    started = re.findall(r"^running ([0-9]+) tests?$", output, re.MULTILINE)
    completed = re.findall(
        r"^test result: (ok|FAILED)\. ([0-9]+) passed; ([0-9]+) failed; "
        r"([0-9]+) ignored; ([0-9]+) measured; [0-9]+ filtered out;.*$",
        output, re.MULTILINE,
    )
    valid = len(started) == 1 and len(completed) == 1
    if valid:
        status, passed, failed, ignored, measured = completed[0]
        passed, failed, ignored, measured = map(int, (passed, failed, ignored, measured))
        valid = (passed + failed > 0 and measured == 0
                 and int(started[0]) == passed + failed + ignored)
        if should_pass:
            valid &= result.returncode == 0 and status == "ok" and failed == 0
        else:
            valid &= result.returncode > 0 and status == "FAILED" and failed > 0
    if not valid:
        raise RuntimeError(f"Unexpected mutation-test result: {command}\n{output}")
    return output


def change(source, before, after):
    if not before or source.count(before) != 1:
        raise RuntimeError(f"Mutation site missing or ambiguous: {before!r}")
    if before == after:
        raise RuntimeError(f"Mutation does not change source: {before!r}")
    return source.replace(before, after, 1)


def variants(source, mutations):
    return [("baseline", source)] + [
        (name, change(source, before, after)) for name, before, after in mutations
    ]


def preflight():
    groups = [("Acceptance", POLICY_SOURCE, POLICY_MUTATIONS)] + [
        (group, relative, mutations) for group, relative, _, mutations in VIR_MUTATION_GROUPS
    ]
    for group, relative, mutations in groups:
        try:
            variants((SOURCE / relative).read_text(), mutations)
        except RuntimeError as error:
            raise RuntimeError(f"{group} ({relative}): {error}") from error


def policy_tests(directory, env):
    source = (SOURCE / POLICY_SOURCE).read_text()
    path = directory / "policy.rs"
    executable = directory / "policy-tests"
    for name, contents in variants(source, POLICY_MUTATIONS):
        path.write_text(contents)
        compile_tests(
            ["rustc", "--edition=2024", "--test", str(path), "-o", str(executable)],
            ROOT, env,
        )
        run([str(executable)], ROOT, env, should_pass=name == "baseline")
        print(f"Acceptance {'passed' if name == 'baseline' else 'detected'}: {name}", flush=True)


def vir_test_binary(workspace, env):
    output = compile_tests(
        ["cargo", "test", "-p", "vir", "--lib", "--no-run", "--message-format=json"],
        workspace, env,
    )
    artifacts = [json.loads(line) for line in output.splitlines()]
    executables = [
        artifact["executable"] for artifact in artifacts
        if artifact.get("reason") == "compiler-artifact"
        and artifact["target"]["name"] == "vir" and artifact.get("executable")
    ]
    if len(executables) != 1:
        raise RuntimeError(f"Expected one VIR test executable: {output}")
    return executables[0]


def precision_tests(directory, env):
    # Only VIR and its two local dependencies are needed. Reuse the real
    # workspace dependency/lint settings, with a smaller temporary member list.
    workspace = directory / "source"
    workspace.mkdir()
    for name in ("vir", "air", "vir_macros"):
        shutil.copytree(SOURCE / name, workspace / name)
    manifest = (SOURCE / "Cargo.toml").read_text()
    manifest, count = re.subn(
        r"(?ms)^members = \[.*?^\]",
        'members = ["vir", "air", "vir_macros"]', manifest, count=1,
    )
    if count != 1:
        raise RuntimeError("Could not construct temporary VIR workspace")
    (workspace / "Cargo.toml").write_text(manifest)
    shutil.copy2(SOURCE / "Cargo.lock", workspace / "Cargo.lock")
    shutil.copy2(ROOT / "rust-toolchain.toml", directory / "rust-toolchain.toml")
    shutil.copytree(SOURCE / ".cargo", workspace / ".cargo")
    env = dict(env, CARGO_NET_OFFLINE="true",
               VERUS_Z3_PATH=env.get("VERUS_Z3_PATH", str(SOURCE / "z3")),
               CARGO_TARGET_DIR=str(directory / "target"),
               CARGO_BUILD_TARGET_DIR=str(directory / "target"),
               CARGO_BUILD_BUILD_DIR=str(directory / "target"))
    for group, relative, test_filter, mutations in VIR_MUTATION_GROUPS:
        path = workspace / relative
        source = path.read_text()
        for name, contents in variants(source, mutations):
            path.write_text(contents)
            try:
                executable = vir_test_binary(workspace, env)
                result = run([executable, test_filter], workspace, env,
                             should_pass=name == "baseline")
            except RuntimeError as error:
                raise RuntimeError(f"{group}: {name}: {error}") from error
            failures = re.findall(r"^test (\S+) \.\.\. FAILED$", result, re.MULTILINE)
            print(f"{group} {'passed' if name == 'baseline' else 'detected'}: {name}"
                  + (f" ({', '.join(failures)})" if failures else ""), flush=True)
        path.write_text(source)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--policy-only", action="store_true")
    args = parser.parse_args()
    preflight()
    with tempfile.TemporaryDirectory(prefix="verus-infer-mutations-") as temporary:
        directory = Path(temporary)
        policy_tests(directory, os.environ.copy())
        if not args.policy_only:
            precision_tests(directory, os.environ.copy())


if __name__ == "__main__":
    main()
