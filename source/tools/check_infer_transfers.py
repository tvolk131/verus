"""Run pinned redb and Fedimint transfers and independently check their proof receipts."""

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

from infer_eval_support import ArtifactGuard, digest, host_info, tool_artifacts
from infer_experiment import (
    LoggedCommands,
    positive_timeout,
    replay_receipt,
    require,
    require_exact,
    require_publication,
    validate_run,
)


ROOT = Path(__file__).resolve().parents[2]
SCENARIOS = {
    "fedimint": {
        "checkout": "fedimint",
        "commit": "b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236",
        "runner": "source/tools/check_fedimint_native.py",
    },
    "redb": {
        "checkout": "redb",
        "commit": "c82ab865bb362bc4af067d332adc44978ff64813",
        "runner": "source/tools/check_redb_codec.py",
    },
    "redb_page_size": {
        "checkout": "redb",
        "commit": "c82ab865bb362bc4af067d332adc44978ff64813",
        "runner": "source/tools/check_redb_page_size.py",
    },
    "redb_address_range": {
        "checkout": "redb",
        "commit": "c82ab865bb362bc4af067d332adc44978ff64813",
        "runner": "source/tools/check_redb_address_range.py",
    },
}


def contained_file(directory, relative):
    path = Path(relative)
    require(not path.is_absolute() and ".." not in path.parts and path.parts,
            "receipt paths must be nonempty and relative")
    resolved = (directory / path).resolve()
    require(resolved.is_relative_to(directory.resolve()), "receipt path escapes run directory")
    require(resolved.is_file(), "missing receipt input: " + str(resolved))
    return resolved


def command_for(receipt, label):
    require(isinstance(label, str) and re.fullmatch(r"[A-Za-z0-9_-]+", label),
            "invalid proof command label")
    commands = [command for command in receipt["commands"] if command.get("label") == label]
    require(len(commands) == 1, "missing or duplicate proof command: " + label)
    return commands[0]


def check_proof(directory, receipt, label, expected, verifier, failure_kind="postcondition"):
    command = command_for(receipt, label)
    argv = command["argv"]
    require(isinstance(argv, list) and argv and Path(argv[0]).resolve() == verifier.resolve(),
            "proof command did not use the selected verifier")
    require("--no-cheating" in argv and "--output-json" in argv and argv.count("--export") == 1,
            "proof command must verify and exercise interface publication")
    require(not any(argument in argv for argument in ("--no-verify", "--no-lifetime")),
            "proof command disables required checks")
    artifact_index = argv.index("--export") + 1
    require(artifact_index < len(argv), "missing export path")
    artifact = Path(argv[artifact_index])
    if not artifact.is_absolute():
        artifact = Path(command["cwd"]) / artifact
    artifact = artifact.resolve()
    require(artifact.is_relative_to(directory.resolve()), "export escapes run directory")
    stdout = contained_file(directory, label + ".stdout")
    stderr = contained_file(directory, label + ".stderr")
    returncode = command.get("exit_code", command.get("returncode"))
    require(type(returncode) is int, "proof command has no integer exit status")
    data = validate_run(dict(returncode=returncode, stdout=stdout.read_text(),
                             stderr=stderr.read_text()), expected)
    result = data["verification-results"]
    inference = data["verus-infer"]
    require(result.get("success") is expected and inference.get("crate_success") is expected
            and inference.get("interface_written") is expected,
            "proof and publication status fields must be booleans")
    functions = inference["functions"]
    require(functions, "proof receipt has no inferred functions")
    require_exact(functions)
    require_publication(data, expected, artifact)
    require(result.get("is-verifying-entire-crate") is True,
            "partial verification is not transfer evidence")
    if expected:
        require(type(result.get("verified")) is int and result["verified"] > 0
                and result.get("errors") == 0, "positive proof has no verified obligations")
    else:
        require(failure_kind in ("postcondition", "precondition", "auxiliary"), "unknown negative control kind")
        diagnostic = "bitvector assertion not satisfied" if failure_kind == "auxiliary" else failure_kind + " not satisfied"
        others = (["precondition not satisfied", "postcondition not satisfied"] if failure_kind == "auxiliary" else
                  [("precondition" if failure_kind == "postcondition" else "postcondition") + " not satisfied"])
        status = "verification_failed" if failure_kind == "auxiliary" else "caller_proof_failed"
        require(inference.get("status") == status
                and type(result.get("errors")) is int and result["errors"] > 0
                and diagnostic in stderr.read_text() and all(other not in stderr.read_text() for other in others),
                "negative control did not fail its declared " + failure_kind)
    return dict(label=label, expected_success=expected, verified=result["verified"],
                errors=result["errors"], stdout_sha256=digest(stdout),
                stderr_sha256=digest(stderr), interface_written=expected,
                failure_kind=None if expected else failure_kind,
                inference_checks=[dict(function=function["function"],
                                       cache=function.get("metrics", {}).get("cache"))
                                  for function in functions])


def check_receipt(path, scenario, verifier):
    receipt = json.loads(path.read_text())
    require(receipt.get("success") is True, "scenario controller did not complete successfully")
    transfer = receipt["transfer"]
    require(transfer.get("scenario") == scenario
            and transfer.get("application_commit") == SCENARIOS[scenario]["commit"],
            "wrong scenario or application pin in receipt")
    require(transfer.get("source_fidelity") is True, "missing original-source fidelity check")
    for field in ("limitations", "trust"):
        require(isinstance(transfer.get(field), list) and transfer[field]
                and all(isinstance(value, str) and value.strip() for value in transfer[field]),
                "missing explicit " + field + " inventory")
    status = transfer.get("status")
    require(status in ("proved", "blocked"), "unknown transfer status")
    summary = dict(scenario=scenario, application_commit=transfer["application_commit"],
                   status=status, source_fidelity=True, limitations=transfer["limitations"],
                   trust=transfer["trust"], receipt=str(path), receipt_sha256=digest(path))
    if status == "blocked":
        require(isinstance(transfer.get("blocker"), str) and transfer["blocker"].strip(),
                "blocked transfer must identify the actual blocker")
        summary["blocker"] = transfer["blocker"]
        return summary
    positives = transfer.get("positive_cases")
    negatives = transfer.get("semantic_negative_cases")
    safety = transfer.get("safety_negative_cases", [])
    auxiliary = transfer.get("auxiliary_negative_cases", [])
    require(isinstance(positives, list) and positives
            and isinstance(negatives, list) and negatives,
            "proved transfer needs positive and semantic negative controls")
    require(isinstance(safety, list), "safety negative controls must be a list")
    require(isinstance(auxiliary, list), "auxiliary negative controls must be a list")
    labels = positives + negatives + safety + auxiliary
    require(all(isinstance(label, str) for label in labels) and len(set(labels)) == len(labels),
            "proof cases must have distinct string labels")
    require(transfer.get("stale_export_rejected") is True,
            "missing seeded stale-interface rejection control")
    summary["proofs"] = [check_proof(path.parent, receipt, label, expected, verifier, failure_kind)
                         for expected, cases, failure_kind in (
                             (True, positives, "postcondition"),
                             (False, negatives, "postcondition"),
                             (False, safety, "precondition"),
                             (False, auxiliary, "auxiliary"))
                         for label in cases]
    summary["stale_export_rejected"] = True
    return summary


def receipt_from_stdout(stdout, output):
    lines = stdout.strip().splitlines()
    require(lines, "scenario runner did not report an output directory")
    directory = Path(lines[-1]).resolve()
    require(directory != output.resolve() and directory.is_relative_to(output.resolve()),
            "scenario runner reported a directory outside this fresh run")
    return contained_file(directory, "receipt.json")


def check_application(run, name, application, environment, phase):
    for suffix, arguments in (("head", ["rev-parse", "HEAD"]),
                              ("status", ["status", "--porcelain", "--untracked-files=all"])):
        label = name + "_" + phase + "_" + suffix
        require(run(label, ["git", "-C", str(application), *arguments], environment) == 0,
                "cannot inspect pinned " + name + " checkout")
        text = (run.output / (label + ".stdout")).read_text().strip()
        require(text == SCENARIOS[name]["commit"] if suffix == "head" else not text,
                "application checkout has the wrong pin or local changes: " + name)


def run_suite(args):
    args.output.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix="transfer-suite-", dir=args.output.resolve()))
    receipt = dict(schema_version=1, host=host_info(), commands=[], scenarios=[],
                   application_proofs_proved=False)
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", VERUS_Z3_PATH=str(args.solver.resolve()))
    scenarios = args.scenario or list(SCENARIOS)
    try:
        with replay_receipt(output, receipt):
            require(len(set(scenarios)) == len(scenarios), "duplicate requested scenario")
            run = LoggedCommands(output, ROOT, receipt["commands"], timeout=args.scenario_timeout)
            require(run("engine_files", ["git", "ls-files", "--cached", "--others",
                                         "--exclude-standard", "--", "source", "rust-toolchain.toml"],
                        environment) == 0)
            source_paths = sorted(set((output / "engine_files.stdout").read_text().splitlines()))
            require(source_paths, "missing engine source ledger")
            inputs = [ROOT / relative for relative in source_paths]
            inputs.extend(ROOT / SCENARIOS[scenario]["runner"] for scenario in scenarios)
            guard = ArtifactGuard(tool_artifacts(args.verus, args.solver, extra=inputs))
            receipt["artifacts"] = guard.initial
            receipt["source_sha256"] = {relative: digest(ROOT / relative) for relative in source_paths}
            run.guard = guard
            for scenario in scenarios:
                try:
                    checkout = SCENARIOS[scenario]["checkout"]
                    application = getattr(args, checkout)
                    require(application is not None, "missing --" + checkout + " checkout")
                    application = application.resolve()
                    check_application(run, scenario, application, environment, "before")
                    destination = output / scenario
                    destination.mkdir()
                    argv = [sys.executable, "-B", str(ROOT / SCENARIOS[scenario]["runner"]),
                            "--application", str(application), "--verus", str(args.verus.resolve()),
                            "--solver", str(args.solver.resolve()), "--output", str(destination),
                            "--timeout", str(args.timeout)]
                    code = run(scenario, argv, environment)
                    require(code == 0, "scenario runner failed: " + scenario)
                    path = receipt_from_stdout((output / (scenario + ".stdout")).read_text(), destination)
                    summary = check_receipt(path, scenario, args.verus)
                    check_application(run, scenario, application, environment, "after")
                    receipt["scenarios"].append(summary)
                except (OSError, ValueError, KeyError, TypeError, subprocess.TimeoutExpired) as error:
                    receipt["scenarios"].append(dict(scenario=scenario, status="failed",
                                                     failure=dict(type=type(error).__name__, message=str(error))))
            guard.check()
            receipt["application_proofs_proved"] = all(
                scenario["status"] == "proved" for scenario in receipt["scenarios"])
            require(receipt["application_proofs_proved"],
                    "not all requested application proofs passed; inspect per-scenario evidence")
    finally:
        print(output)
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fedimint", type=Path)
    parser.add_argument("--redb", type=Path)
    parser.add_argument("--scenario", action="append", choices=list(SCENARIOS))
    parser.add_argument("--verus", type=Path, default=ROOT / "source/target-verus/release/verus")
    parser.add_argument("--solver", type=Path, default=ROOT / "source/z3")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=positive_timeout, default=180)
    parser.add_argument("--scenario-timeout", type=positive_timeout, default=1800)
    run_suite(parser.parse_args())


if __name__ == "__main__":
    main()
