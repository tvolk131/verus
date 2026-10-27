#!/usr/bin/env python3
"""Controlled three-crate Cargo incremental evaluation (offline, isolated copies)."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

from infer_eval_support import (ArtifactChanged, ArtifactGuard, digest, grouped_statistics, host_info,
                                measured, tool_artifacts)

ROOT = Path(__file__).resolve().parents[2]
SOURCES = {
    "base": '''use vstd::prelude::*;
#[verus_infer] pub fn increment(x:u16)->u16 { x + 1 }
#[verus_infer(precision="sound")] pub fn kind(x:u8)->u8 { x & 15 }
#[verus_infer] pub fn independent(x:u16)->u16 { x / 10 }
''',
    "middle": '''use vstd::prelude::*;
#[verus_infer] pub fn twice(x:u16)->u16 {
    infer_eval_base::increment(infer_eval_base::increment(x))
}
#[verus_infer(precision="sound")] pub fn kind(x:u8)->u8 { infer_eval_base::kind(x) }
''',
    "app": '''use vstd::prelude::*;
#[verus_infer] pub fn top(x:u16)->u16 { infer_eval_middle::twice(x) }
verus! {
pub fn property(x:u16)->(r:u16)
    requires x <= u16::MAX - 2,
    ensures r as int == x as int + 2,
{ top(x) }
pub fn mask_property(x:u8) { let r=infer_eval_middle::kind(x); assert(r<=15); }
}
''',
}


def fixture(directory):
    directory.mkdir()
    (directory / "Cargo.toml").write_text('[workspace]\nmembers=["base","middle","app"]\nresolver="2"\n')
    for name, source in SOURCES.items():
        part = directory / name
        (part / "src").mkdir(parents=True)
        (part / "src/lib.rs").write_text(source)
        dependency = {"middle": "base", "app": "middle"}.get(name)
        manifest = f'''[package]
name="infer-eval-{name}"
version="0.1.0"
edition="2021"
[dependencies]
vstd={{path={json.dumps(str(ROOT / "source/vstd"))}}}
'''
        if dependency:
            manifest += f'infer-eval-{dependency}={{path="../{dependency}"}}\n'
        manifest += '[package.metadata.verus]\nverify=true\ntrusted_crates=["vstd"]\n'
        (part / "Cargo.toml").write_text(manifest)


def reports_in(text):
    decoder = json.JSONDecoder()
    reports = []
    offset = 0
    while True:
        offset = text.find("{", offset)
        if offset < 0:
            return reports
        try:
            obj, end = decoder.raw_decode(text, offset)
            offset = end
            if isinstance(obj, dict) and "verus-infer" in obj:
                reports.append(obj["verus-infer"])
        except ValueError:
            offset += 1


def interfaces(target):
    return {str(p.relative_to(target)): {"sha256": digest(p), "mtime_ns": p.stat().st_mtime_ns}
            for p in sorted(target.rglob("*.vir")) if "infer_eval_" in p.name}


def contracts(reports):
    return {f["function"]: f["contract"] for report in reports for f in report["functions"]}


def validate(row, before, expected_reports):
    errors = []
    stale = row["phase"] == "contract_edit_stale_caller"
    if (row["returncode"] == 0) == stale:
        errors.append("unexpected Cargo exit status")
    reports = row["reports"]
    if any(r.get("schema_version") != 1 or r.get("status") != "no_inference" or not r.get("crate_success") for r in row["ancillary_reports"]):
        errors.append("unexpected ancillary crate report")
    if len(reports) != expected_reports:
        errors.append(f"expected {expected_reports} fresh crate reports, observed {len(reports)}")
    for report in reports:
        failed_app = stale and report.get("status") == "caller_proof_failed"
        if report.get("schema_version") != 1 or not report.get("functions"):
            errors.append("missing inference schema or contracts")
        if report.get("crate_success") == failed_app or report.get("interface_written") == failed_app:
            errors.append("incorrect crate success/publication")
        if report.get("status") != ("caller_proof_failed" if failed_app else "verified"):
            errors.append("unexpected crate proof outcome")
        for f in report.get("functions", []):
            for proof in f["proofs"].values():
                if proof["required"] and proof["status"] != "proved":
                    errors.append("missing required fresh candidate proof")
    if stale and not any(r.get("status") == "caller_proof_failed" for r in reports):
        errors.append("stale caller property was not rejected")
    actual = row["interfaces"]
    if row["phase"] == "warm" and actual != before:
        errors.append("no-edit Cargo run rewrote interfaces")
    expected_crates = {"base", "middle"} if stale else {"base", "middle", "app"}
    for name in ("base", "middle", "app"):
        exists = any("infer_eval_" + name in p for p in actual)
        if exists != (name in expected_crates):
            errors.append(f"incorrect interface presence for {name}")
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verus", type=Path, default=ROOT / "source/target-verus/release/verus")
    parser.add_argument("--output-dir", type=Path)
    parser.add_argument("--repetitions", type=int, default=1)
    parser.add_argument("--timeout", type=float, default=300)
    parser.add_argument("--memory", action="store_true")
    args = parser.parse_args()
    if args.repetitions < 1 or args.timeout <= 0:
        parser.error("positive repetitions and timeout required")
    verifier = args.verus.resolve(strict=True)
    cargo = verifier.with_name("cargo-verus")
    directory = args.output_dir or Path(tempfile.mkdtemp(prefix="infer-cargo-eval-"))
    if args.output_dir:
        directory.mkdir(parents=True, exist_ok=False)
    directory = directory.resolve()
    print(f"Artifacts: {directory}", flush=True)
    env = dict(os.environ, CARGO_NET_OFFLINE="true",
               VERUS_Z3_PATH=os.environ.get("VERUS_Z3_PATH", str(ROOT / "source/z3")),
               RUST_MIN_STACK=os.environ.get("RUST_MIN_STACK", "20971520"))
    host = host_info()
    rows, aborted, guard = [], None, None
    phases = ["cold", "warm", "body_edit", "contract_edit_stale_caller", "contract_edit_fixed_caller"]
    try:
        # Cargo compiles vstd from source. Bind those sources and manifests too.
        sources = [p for name in ("vstd", "builtin", "builtin_macros", "state_machines_macros")
                   for p in (ROOT / "source" / name).rglob("*")
                   if p.is_file() and (p.suffix in (".rs", ".toml") or p.name == "Cargo.lock")]
        guard = ArtifactGuard(tool_artifacts(verifier, env["VERUS_Z3_PATH"], [cargo, Path(__file__), *sources]))
        for repetition in range(1, args.repetitions + 1):
            repeat = directory / f"repeat-{repetition}"
            repeat.mkdir()
            workspace, target = repeat / "workspace", repeat / "target"
            fixture(workspace)
            invocation = repeat / "invocation"
            invocation.mkdir()
            current_env = dict(env, VERUS_INFER_CACHE=str(repeat / "candidate-cache"),
                               CARGO_TARGET_DIR=str(target), CARGO_BUILD_TARGET_DIR=str(target),
                               CARGO_BUILD_BUILD_DIR=str(target))
            before = {}
            cold_contracts = None
            for phase in phases:
                base = workspace / "base/src/lib.rs"
                app = workspace / "app/src/lib.rs"
                if phase == "body_edit":
                    base.write_text(base.read_text().replace("{ x + 1 }", "{ let offset = 1; x + offset }", 1))
                if phase == "contract_edit_stale_caller":
                    base.write_text(base.read_text().replace("let offset = 1", "let offset = 2", 1))
                if phase == "contract_edit_fixed_caller":
                    app.write_text(app.read_text().replace("- 2", "- 4").replace("+ 2", "+ 4"))
                snapshot = repeat / (phase + ".sources")
                shutil.copytree(workspace, snapshot)
                command = [str(cargo), "verify", "--manifest-path", str(workspace / "Cargo.toml"),
                           "--", "--output-json", "--num-threads", "1"]
                row = dict(case="cargo_three_crates", phase=phase, repetition=repetition,
                           command=command, errors=[], environment={k: v for k, v in current_env.items()
                           if k.startswith(("CARGO_", "VERUS_", "RUST_MIN_STACK")) and k in
                           ("CARGO_NET_OFFLINE", "CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR", "CARGO_BUILD_BUILD_DIR",
                            "VERUS_Z3_PATH", "VERUS_INFER_CACHE", "RUST_MIN_STACK")})
                try:
                    guard.check()
                    run = measured(command, current_env, args.timeout, invocation, args.memory)
                    row.update({k: run[k] for k in ("returncode", "wall_micros", "memory")})
                    for kind in ("stdout", "stderr"):
                        (repeat / f"{phase}.{kind}.txt").write_text(run[kind])
                    all_reports = reports_in(run["stdout"]) + reports_in(run["stderr"])
                    row["reports"] = [r for r in all_reports if r.get("status") != "no_inference"]
                    row["ancillary_reports"] = [r for r in all_reports if r.get("status") == "no_inference"]
                    row["interfaces"] = interfaces(target)
                    row["errors"] = validate(row, before, 0 if phase == "warm" else 1 if phase == "contract_edit_fixed_caller" else 3)
                    if phase == "cold":
                        cold_contracts = contracts(row["reports"])
                    if phase == "body_edit" and contracts(row["reports"]) != cold_contracts:
                        row["errors"].append("body-only edit changed displayed contracts")
                    if phase == "body_edit":
                        independent = [f for r in row["reports"] for f in r["functions"]
                                       if f["function"].endswith("::independent")]
                        if len(independent) != 1:
                            row["errors"].append("body edit lost the independent candidate")
                        else:
                            row["independent_candidate_cache"] = independent[0]["metrics"]["cache"]
                    if phase == "contract_edit_stale_caller":
                        current = contracts(row["reports"])
                        row["changed_contracts"] = sorted(k for k in current if current[k] != cold_contracts.get(k))
                        if not any(k.endswith("::top") for k in row["changed_contracts"]):
                            row["errors"].append("semantic edit did not update the transitive candidate")
                    before = row["interfaces"]
                except (OSError, ValueError, KeyError, TypeError, subprocess.TimeoutExpired, subprocess.CalledProcessError) as error:
                    row["errors"] = [f"harness failure: {error}"]
                    if isinstance(error, subprocess.TimeoutExpired):
                        for kind in ("stdout", "stderr"):
                            (repeat / f"{phase}.{kind}.txt").write_text(getattr(error, kind) or "")
                try:
                    guard.check()
                except ArtifactChanged as error:
                    row["errors"].append(str(error))
                    aborted = str(error)
                rows.append(row)
                print(f"repeat {repetition} {phase}: {'FAIL' if row['errors'] else 'PASS'}", flush=True)
                for error in row["errors"]:
                    print("  " + error, flush=True)
                if aborted:
                    raise ArtifactChanged(aborted)
            guard.check()
    except (ArtifactChanged, OSError, ValueError) as error:
        aborted = str(error)
    result = dict(schema_version=2, host=host, aborted=aborted, runs=rows,
                  planned_runs=args.repetitions * len(phases),
                  artifact_baseline=guard.initial if guard else None,
                  artifact_changes=guard.changes if guard else [],
                  statistics=grouped_statistics(rows))
    (directory / "results.json").write_text(json.dumps(result, indent=2) + "\n")
    lines = ["# Cargo Infer incremental evaluation", "", f"Aborted: {aborted}", "",
             "Each repetition has isolated Cargo and candidate caches. Warm Cargo runs can skip verification entirely.", "",
             "| Repetition | Phase | Result | Wall ms |", "|---:|---|---|---:|"]
    lines.extend(f"| {r['repetition']} | {r['phase']} | {'FAIL' if r['errors'] else 'PASS'} | {r.get('wall_micros', 0)/1000:.1f} |" for r in rows)
    lines.extend(["", "```json", json.dumps(result["statistics"], indent=2), "```", ""])
    (directory / "summary.md").write_text("\n".join(lines))
    print(f"Results: {directory / 'results.json'}", flush=True)
    return bool(aborted or any(r["errors"] for r in rows) or len(rows) != result["planned_runs"])


if __name__ == "__main__":
    raise SystemExit(main())
