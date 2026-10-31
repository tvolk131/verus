"""Common execution and evidence checks for source-connected Infer experiments.

Scenario modules retain their source transformations, expected function scope,
and precision/cache policy. These helpers only enforce shared run invariants.
"""
import json
from pathlib import Path
import subprocess
import time

from infer_eval_support import digest, run_compiler


OBLIGATIONS = ("soundness", "maximality", "exactness")


def validate_run(run, expected, *, allow_frontend=False):
    data = json.loads(run["stdout"])
    result = data["verification-results"]
    success = result.get("success")
    # Early policy/frontend rejection can happen before verification emits a
    # success field. Only explicitly permitted, marked frontend errors qualify.
    if (allow_frontend and not expected and "success" not in result
            and (result.get("encountered-error") is True
                 or result.get("encountered-vir-error") is True)):
        success = False
    if (run["returncode"] == 0) != expected or success != expected:
        raise ValueError(f"unexpected verification outcome: {result}")
    if any(s in run["stderr"].lower() for s in ("panicked at", "resource limit", "timed out")):
        raise ValueError("panic or resource exhaustion is not semantic evidence")
    if not allow_frontend and result.get("encountered-vir-error"):
        raise ValueError("frontend failure is not a semantic negative control")
    return data


def require_exact(functions):
    for function in functions:
        proofs = function.get("proofs", {})
        if (function.get("mode") != "exact" or not function.get("required_checks_passed")
                or any(proofs.get(p, {}).get("status") != "proved"
                       or proofs.get(p, {}).get("required") is not True for p in OBLIGATIONS)):
            raise ValueError("missing mandatory exact inference checks")


def require_publication(data, expected, artifact):
    inference = data["verus-infer"]
    if (data["verification-results"].get("success", False) != expected
            or inference["crate_success"] != expected
            or inference["interface_written"] != expected
            or Path(artifact).exists() != expected):
        raise ValueError("verification and interface publication disagree")


def save_results(output, result):
    (Path(output) / "results.json").write_text(json.dumps(result, indent=2) + "\n")


def snapshot_inputs(checkout, destination, relatives):
    destination = Path(destination)
    for relative in relatives:
        path = Path(relative)
        if path.is_absolute() or ".." in path.parts:
            raise ValueError("snapshot inputs must be relative to the checkout")
    for relative in relatives:
        path = destination / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes((Path(checkout) / relative).read_bytes())
    return dict(input_snapshot=str(destination),
                source_hashes={p: digest(destination / p) for p in relatives})


class LoggedCommands:
    """Run commands with the hex replay log filenames and receipt format."""
    def __init__(self, output, cwd, commands, *, timeout=None):
        self.output = Path(output)
        self.cwd = Path(cwd)
        self.commands = commands
        self.timeout = timeout

    def __call__(self, label, command, environment):
        started = time.monotonic()
        try:
            result = run_compiler(command, environment, self.timeout, cwd=self.cwd)
        except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
            self._logs(label, error.stdout or "", error.stderr or "")
            self._record(label, command, environment, started, None,
                         timed_out=isinstance(error, subprocess.TimeoutExpired),
                         interrupted=isinstance(error, KeyboardInterrupt))
            raise
        self._logs(label, result.stdout, result.stderr)
        self._record(label, command, environment, started, result.returncode)
        return result.returncode

    def _record(self, label, command, environment, started, exit_code, **details):
        self.commands.append(dict(
            label=label, argv=command, cwd=str(self.cwd), exit_code=exit_code,
            seconds=time.monotonic() - started,
            environment={k: environment[k] for k in (
                "RUSTUP_TOOLCHAIN", "VERUS_Z3_PATH", "VERUS_INFER_CACHE", "CARGO_TARGET_DIR"
            ) if k in environment}, **details,
        ))

    def _logs(self, label, stdout, stderr):
        for suffix, contents in (("stdout", stdout), ("stderr", stderr)):
            (self.output / f"{label}.{suffix}").write_text(contents)
