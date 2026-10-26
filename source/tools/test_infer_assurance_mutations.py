#!/usr/bin/env python3
"""Run real source mutations in temporary copies; never edit the checkout.

Run from any directory with the repository's Rust toolchain and cached Cargo
dependencies available. --policy-only runs the dependency-free acceptance core.
"""
import argparse
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "source"


def run(command, cwd, env, *, should_pass):
    result = subprocess.run(command, cwd=cwd, env=env, text=True, capture_output=True)
    output = result.stdout + result.stderr
    # A compile failure or a filter that runs no tests is not a detected mutant.
    ran = re.search(r"running [1-9][0-9]* tests", output)
    expected = result.returncode == 0 if should_pass else result.returncode != 0
    if not ran or not expected:
        raise RuntimeError(f"Unexpected mutation-test result: {command}\n{output}")


def change(source, before, after):
    if source.count(before) != 1:
        raise RuntimeError(f"Mutation site missing or ambiguous: {before!r}")
    return source.replace(before, after, 1)


def policy_tests(directory, env):
    source = (SOURCE / "vir/src/infer_proof/policy.rs").read_text()
    path = directory / "policy.rs"
    executable = directory / "policy-tests"
    mutations = [
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
    for name, contents in [("baseline", source)] + [
        (name, change(source, before, after)) for name, before, after in mutations
    ]:
        path.write_text(contents)
        compiled = subprocess.run(
            ["rustc", "--edition=2024", "--test", str(path), "-o", str(executable)],
            cwd=ROOT, env=env, text=True, capture_output=True,
        )
        if compiled.returncode:
            raise RuntimeError(compiled.stdout + compiled.stderr)
        run([str(executable)], ROOT, env, should_pass=name == "baseline")
        print(f"Acceptance {'passed' if name == 'baseline' else 'detected'}: {name}", flush=True)


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
    env = dict(env, CARGO_NET_OFFLINE="true", VERUS_Z3_PATH=str(SOURCE / "z3"),
               CARGO_TARGET_DIR=str(directory / "target"),
               CARGO_BUILD_TARGET_DIR=str(directory / "target"),
               CARGO_BUILD_BUILD_DIR=str(directory / "target"))
    path = workspace / "vir/src/infer_checks.rs"
    source = path.read_text()
    mutations = [
        ("maximality assumes its own precondition", "check.reqs = Arc::new(vec![]);", "check.reqs = original.reqs.clone();"),
        ("exactness examines only the actual output", "let other = SpannedTyped::new(span, typ, ExpX::Var(name.clone()));", "let other = SpannedTyped::new(span, typ, ExpX::Var(dest.clone()));"),
        ("precision body accepted as ordinary input", "validate_query_kind(original, None, span)?;", "// Incorrectly omit the query-kind check."),
        ("missing loop termination measure accepted", "|| decrease.len() != 1", "|| false"),
    ]
    for name, contents in [("baseline", source)] + [
        (name, change(source, before, after)) for name, before, after in mutations
    ]:
        path.write_text(contents)
        run(["cargo", "test", "-p", "vir", "infer_checks::tests"], workspace, env,
            should_pass=name == "baseline")
        print(f"Precision {'passed' if name == 'baseline' else 'detected'}: {name}", flush=True)

    # Reset the precision checker before independently mutating the loop-only
    # installation/eligibility boundary in the same temporary workspace.
    path.write_text(source)
    path = workspace / "vir/src/infer/loops.rs"
    source = path.read_text()
    mutations = [
        ("executable projection check omitted", "if encode(&projected)? != encode(original.x.body.as_ref().unwrap())? {", "if false {"),
        ("termination assumption admitted", "|| f.attrs.exec_assume_termination", "|| false"),
        ("termination suppression admitted", "|| f.attrs.exec_allows_no_decreases_clause", "|| false"),
        ("non-unit counter update admitted", "constant(&counter_step, 1)", "true"),
        ("wrong counter direction admitted", "counter_sub != countdown ||", "false ||"),
    ]
    for name, contents in [("baseline", source)] + [
        (name, change(source, before, after)) for name, before, after in mutations
    ]:
        path.write_text(contents)
        run(["cargo", "test", "-p", "vir", "infer::loops::tests"], workspace, env,
            should_pass=name == "baseline")
        print(f"Loop {'passed' if name == 'baseline' else 'detected'}: {name}", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--policy-only", action="store_true")
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="verus-infer-mutations-") as temporary:
        directory = Path(temporary)
        policy_tests(directory, os.environ.copy())
        if not args.policy_only:
            precision_tests(directory, os.environ.copy())


if __name__ == "__main__":
    main()
