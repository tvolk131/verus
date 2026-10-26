#!/usr/bin/env python3
"""Evaluate the curated Infer corpus with fresh proofs on cold/warm candidates.

Only Python's standard library is required. Build Verus first. Every run writes
the exact inputs, invocations, stdout JSON, stderr, results.json and summary.md.
Exit nonzero on an unexpected outcome, malformed report or harness timeout.
"""
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_MANIFEST = Path(__file__).with_name("infer_eval") / "cases.json"
OBLIGATIONS = ("soundness", "maximality", "exactness")


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run_compiler(command, env, timeout):
    # The Verus launcher starts a compiler child. On POSIX, terminate the whole
    # owned process group so a host timeout cannot leave that child running or
    # keep its inherited output pipes open.
    with subprocess.Popen(command, cwd=ROOT, env=env, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, text=True,
                          start_new_session=(os.name == "posix")) as process:
        try:
            stdout, stderr = process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired as error:
            try:
                if os.name == "posix":
                    os.killpg(process.pid, signal.SIGKILL)
                else:
                    process.kill()
            except ProcessLookupError:
                pass
            error.stdout, error.stderr = process.communicate()
            raise
        return subprocess.CompletedProcess(command, process.returncode, stdout, stderr)


def validate(report, case, returncode, phase, cold, artifact):
    errors = []
    expected = case["expected_status"]
    success = expected == "verified"
    if report.get("schema_version") != 1:
        errors.append("unsupported or missing report schema")
    if report.get("status") != expected:
        errors.append(f"expected {expected}, observed {report.get('status')}")
    if (returncode == 0) != success or report.get("crate_success") != success:
        errors.append("process/report success disagrees with expected outcome")
    if report.get("interface_written") != success or artifact.exists() != success:
        errors.append("interface publication disagrees with crate outcome")
    functions = report.get("functions", [])
    if success and not functions:
        errors.append("successful inference case produced zero contracts")
    if "expected_loop_functions" in case:
        loop_functions = [f for f in functions if f.get("loop_proofs")]
        if len(loop_functions) != case["expected_loop_functions"]:
            errors.append("missing counted-loop proof annotations")
        for f in loop_functions:
            for proof in f["loop_proofs"]:
                if len(proof["invariants"]) != 2 or len(proof["decreases"]) != 1 or proof["loop_isolation"]:
                    errors.append("incompatible counted-loop proof annotations")
    for f in functions:
        checks = f["proofs"]
        required = OBLIGATIONS if f["mode"] == "exact" else ("soundness",)
        if set(checks) != set(OBLIGATIONS) or any(
            checks[o]["required"] != (o in required) for o in OBLIGATIONS
        ):
            errors.append(f"{f['function']}: incorrect required obligation set")
        passed = all(checks[o]["status"] == "proved" for o in required)
        if f["required_checks_passed"] != passed:
            errors.append(f"{f['function']}: required evidence disagrees with acceptance")
        if success and not passed:
            errors.append(f"{f['function']}: success without required proofs")
        if f["exported"] and (not success or not passed):
            errors.append(f"{f['function']}: exported without successful verification")
        if not f.get("candidate_id") or len(f["candidate_id"]) != 64:
            errors.append("missing candidate identity")
        if phase in ("cold", "warm"):
            expected_cache = "miss" if phase == "cold" else "hit"
            if f["metrics"]["cache"] != expected_cache:
                errors.append(f"{f['function']}: expected cache {expected_cache}")
    if phase == "warm" and cold is not None:
        # IDs bind the actual typed AST, including source spans; cache decoding
        # can restamp formulas. Compare predicates and freshly checked evidence.
        old = {f["function"]: f["contract"] for f in cold["functions"]}
        if old != {f["function"]: f["contract"] for f in functions}:
            errors.append("warm candidate contracts differ from cold contracts")
    if expected == "required_proof_incomplete" and not any(
        c["required"] and c["status"] == "incomplete"
        for f in functions for c in f["proofs"].values()
    ):
        errors.append("expected required resource limit without matching evidence")
    if expected in ("unsupported", "analysis_incomplete") and not report.get("generation_failure"):
        errors.append("generation failure has no diagnostic")
    if expected == "caller_proof_failed" and not report.get("failed_functions"):
        errors.append("caller failure has no failed function")
    return errors


def measure(case, phase, source, directory, verifier, env, timeout, cold):
    artifact = directory / "interface.vir"
    (directory / f"{phase}.input.rs").write_bytes(source.read_bytes())
    command = [str(verifier), str(source), "--crate-type=lib", "--edition=2021",
               "--crate-name", case["name"], "--output-json", "--time",
               "--num-threads", "1", "--export", str(artifact)]
    started = time.perf_counter_ns()
    row = {"case": case["name"], "phase": phase, "command": command,
           "source_sha256": digest(source), "features": case["features"],
           "environment": {key: env[key] for key in
                           ("VERUS_Z3_PATH", "RUST_MIN_STACK", "VERUS_INFER_CACHE")}}
    try:
        run = run_compiler(command, env, timeout)
        row["wall_micros"] = (time.perf_counter_ns() - started) // 1000
        (directory / f"{phase}.stdout.json").write_text(run.stdout)
        (directory / f"{phase}.stderr.txt").write_text(run.stderr)
        output = json.loads(run.stdout)
        report = output["verus-infer"]
        row.update(returncode=run.returncode, report=report,
                   times_ms=output.get("times-ms"), build=output.get("verus"))
        row["errors"] = validate(report, case, run.returncode, phase, cold, artifact)
    except (subprocess.TimeoutExpired, OSError, ValueError, KeyError, TypeError) as error:
        # A host timeout or missing JSON is a harness failure, never a solver result.
        row["errors"] = [f"harness failure: {error}"]
        if isinstance(error, subprocess.TimeoutExpired):
            for label, contents in (("stdout.json", error.stdout), ("stderr.txt", error.stderr)):
                if isinstance(contents, bytes):
                    contents = contents.decode(errors="replace")
                (directory / f"{phase}.{label}").write_text(contents or "")
    row.setdefault("wall_micros", (time.perf_counter_ns() - started) // 1000)
    print(f"{case['name']:22} {phase:11} {'PASS' if not row['errors'] else 'FAIL'} "
          f"{row['wall_micros'] / 1000:.1f} ms", flush=True)
    for error in row["errors"]:
        print(f"  {error}", flush=True)
    return row


def summary(rows, cases):
    cold = [r for r in rows if r["phase"] == "cold" and "report" in r]
    functions = [f for r in cold for f in r["report"]["functions"]]
    return {
        "cases": len(cases), "runs": len(rows),
        "passed_runs": sum(not r["errors"] for r in rows),
        "failed_runs": sum(bool(r["errors"]) for r in rows),
        "cold_status_counts": dict(Counter(r["report"]["status"] for r in cold)),
        "cold_function_modes": dict(Counter(f["mode"] for f in functions)),
        "cold_proof_outcomes": {o: dict(Counter(f["proofs"][o]["status"] for f in functions))
                                for o in OBLIGATIONS},
        "features": sorted({feature for case in cases for feature in case["features"]}),
        "phases": {phase: {
            "wall_micros": sum(r["wall_micros"] for r in rows if r["phase"] == phase),
            "generation_micros": sum(r["report"]["metrics"]["generation_micros"]
                                     for r in rows if r["phase"] == phase and "report" in r),
            "candidate_solver_micros": sum(f["metrics"]["solver_micros"]
                for r in rows if r["phase"] == phase and "report" in r
                for f in r["report"]["functions"]),
            "formula_nodes": sum(f["metrics"]["requires_nodes"] + f["metrics"]["ensures_nodes"]
                for r in rows if r["phase"] == phase and "report" in r
                for f in r["report"]["functions"]),
            "cache": dict(Counter(f["metrics"]["cache"]
                for r in rows if r["phase"] == phase and "report" in r
                for f in r["report"]["functions"])),
        } for phase in ("cold", "warm", "incremental")},
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verus", type=Path, default=ROOT / "source/target-verus/release/verus")
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--output-dir", type=Path, help="new directory; default: persistent temporary directory")
    parser.add_argument("--case", action="append", help="select case names (repeatable)")
    parser.add_argument("--timeout", type=float, default=180, help="host seconds per invocation")
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text())
    if manifest.get("schema_version") != 1:
        parser.error("unsupported corpus schema")
    cases = manifest["cases"]
    names = [c["name"] for c in cases]
    if len(set(names)) != len(names):
        parser.error("duplicate corpus case names")
    if args.case:
        if set(args.case) - set(names):
            parser.error("unknown case selection")
        cases = [c for c in cases if c["name"] in args.case]
    if not cases or args.timeout <= 0:
        parser.error("select at least one case and a positive timeout")
    verifier = args.verus.resolve(strict=True)
    directory = args.output_dir or Path(tempfile.mkdtemp(prefix="verus-infer-eval-"))
    if args.output_dir:
        directory.mkdir(parents=True, exist_ok=False)
    directory = directory.resolve()
    print(f"Artifacts: {directory}", flush=True)
    env = dict(os.environ, VERUS_Z3_PATH=os.environ.get("VERUS_Z3_PATH", str(ROOT / "source/z3")),
               RUST_MIN_STACK=os.environ.get("RUST_MIN_STACK", "20971520"))
    rows = []
    for case in cases:
        case_dir = directory / case["name"]
        case_dir.mkdir()
        source = case_dir / "input.rs"
        if depth := case.get("graph_depth"):
            if not 1 <= depth <= 128:
                raise ValueError("graph depth must be between 1 and 128")
            lines = ["use vstd::prelude::*;", "#[verus_infer] pub fn layer_0(x:u32)->u32{x/2}"]
            for i in range(1, depth):
                lines.append(f"#[verus_infer] pub fn layer_{i}(x:u32)->u32{{layer_{i-1}(x)/2}}")
            lines.append(f"verus!{{fn caller(){{let r=layer_{depth-1}(0);assert(r==0);}}}}")
            source.write_text("\n".join(lines) + "\n")
        else:
            source.write_bytes((args.manifest.parent / case["source"]).read_bytes())
        case_env = dict(env, VERUS_INFER_CACHE=str(case_dir / "cache"))
        cold = None
        for phase in ("cold", "warm"):
            row = measure(case, phase, source, case_dir, verifier, case_env, args.timeout, cold)
            rows.append(row)
            if phase == "cold":
                cold = row.get("report")
        if edit := case.get("edit"):
            original = source.read_text()
            if original.count(edit["before"]) != 1:
                raise RuntimeError("incremental edit must identify exactly one source fragment")
            source.write_text(original.replace(edit["before"], edit["after"], 1))
            rows.append(measure(case, "incremental", source, case_dir, verifier, case_env, args.timeout, cold))
    aggregate = summary(rows, cases)
    driver = verifier.with_name("rust_verify" + (".exe" if os.name == "nt" else ""))
    solver = Path(env["VERUS_Z3_PATH"])
    result = {"schema_version": 1, "verifier": str(verifier), "verifier_sha256": digest(verifier),
              "driver_sha256": digest(driver) if driver.is_file() else None,
              "solver_sha256": digest(solver) if solver.is_file() else None,
              "manifest_sha256": digest(args.manifest), "summary": aggregate, "runs": rows}
    (directory / "results.json").write_text(json.dumps(result, indent=2) + "\n")
    lines = ["# Verus Infer evaluation", "", f"{aggregate['passed_runs']}/{len(rows)} expected outcomes passed across {len(cases)} curated cases.", "",
             "This corpus measures selected examples and known limits; it is not an estimate of general Rust coverage.", "",
             "| Case | Phase | Outcome | Wall ms | Expected outcome |", "|---|---|---|---:|---|"]
    for row in rows:
        lines.append(f"| {row['case']} | {row['phase']} | {row.get('report', {}).get('status', 'harness failure')} | {row['wall_micros']/1000:.1f} | {'PASS' if not row['errors'] else 'FAIL'} |")
    lines.extend(["", "Aggregate measurements and proof outcomes:", "", "```json", json.dumps(aggregate, indent=2), "```", ""])
    (directory / "summary.md").write_text("\n".join(lines))
    print(f"{aggregate['passed_runs']}/{len(rows)} runs passed. Results: {directory / 'results.json'}")
    return bool(aggregate["failed_runs"])


if __name__ == "__main__":
    raise SystemExit(main())
