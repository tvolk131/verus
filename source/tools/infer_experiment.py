"""Common execution and evidence checks for source-connected Infer experiments.

Scenario modules retain their source transformations, expected function scope,
and precision/cache policy. These helpers only enforce shared run invariants.
"""
from contextlib import contextmanager
import json
import math
from pathlib import Path
import subprocess
import time

from infer_eval_support import digest, run_compiler


OBLIGATIONS = ("soundness", "maximality", "exactness")


def require(condition, message="replay validation failed"):
    if not condition:
        raise ValueError(message)


def positive_timeout(value):
    timeout = float(value)
    if not math.isfinite(timeout) or timeout <= 0:
        raise ValueError("timeout must be a finite positive number of seconds")
    return timeout


@contextmanager
def replay_receipt(output, receipt):
    """Persist setup, command, validation and restoration failures, including interrupts."""
    receipt["success"] = False
    try:
        yield
    except BaseException as error:
        receipt["failure"] = dict(type=type(error).__name__, message=str(error))
        raise
    else:
        receipt["success"] = True
    finally:
        pending = Path(output) / "receipt.json.tmp"
        pending.write_text(json.dumps(receipt, indent=2) + "\n")
        pending.replace(Path(output) / "receipt.json")


def restore_sources(root, originals, sources, receipt):
    """Attempt every restoration before checking hashes or reporting failures."""
    errors = []
    for path, original in originals.items():
        try:
            if not path.exists() or path.read_bytes() != original:
                path.write_bytes(original)
        except OSError as error:
            errors.append(f"{path}: {error}")
    after = {}
    for path in sources:
        relative = str(path.relative_to(root))
        try:
            after[relative] = digest(path)
        except OSError as error:
            after[relative] = None
            errors.append(f"{path}: {error}")
    receipt["native_sources_after"] = after
    if errors:
        receipt["restoration_errors"] = errors
    require(not errors and receipt["native_sources_before"] == after,
            "source restoration failed")


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


def require_function_coverage(functions, expected):
    """Bind checks to the scenario's explicit helper identities, including zero."""
    names = [function.get("function", "").rsplit("::", 1)[-1] for function in functions]
    require(len(names) == len(set(names)) and set(names) == set(expected),
            "inferred-function coverage mismatch")


def require_failure_kind(stderr, expected):
    kinds = ("postcondition not satisfied", "precondition not satisfied", "assertion failed")
    require(expected in kinds and expected in stderr, "expected semantic diagnostic missing")
    require(not any(kind in stderr for kind in kinds if kind != expected), "mixed failure kinds")


def require_publication(data, expected, artifact):
    inference = data["verus-infer"]
    if (data["verification-results"].get("success", False) != expected
            or inference["crate_success"] != expected
            or inference["interface_written"] != expected
            or Path(artifact).exists() != expected):
        raise ValueError("verification and interface publication disagree")


def validate_goal_run(run, expected, *, expected_functions, exec_bodies,
                      proof_bodies, failure_kind=None, artifact=None):
    """Check an ordinary goal proof/control with an explicit local body inventory.

    Scenario code still binds the semantic property, input domain and source
    projection. Frontend/resource and intentionally poisoned-helper rejections
    use separate validators; they cannot count as ordinary semantic controls.
    """
    require(isinstance(expected, bool), "a goal outcome must be explicit")
    require((expected and failure_kind is None)
            or (not expected and failure_kind is not None),
            "a negative goal needs its intended failure kind")
    executable, proofs = set(exec_bodies), set(proof_bodies)
    require(bool(executable | proofs) and not executable.intersection(proofs),
            "an unambiguous nonempty body inventory is required")
    data = validate_run(run, expected)
    verification = data["verification-results"]
    require(verification.get("is-verifying-entire-crate") is True,
            "goal verification must cover the entire selected crate")
    errors = verification.get("errors")
    require(type(errors) is int and (errors == 0 if expected else errors > 0),
            "goal error count disagrees with the expected outcome")
    inference = data["verus-infer"]
    require(inference.get("crate_success") is expected, "goal crate status disagrees")
    functions = inference.get("functions", [])
    require_function_coverage(functions, expected_functions)
    require_exact(functions)
    scope = inference["verification_scope"]
    require(scope.get("native_collection_completed") is True,
            "goal native body inventory is incomplete")
    bodies = scope["native_functions"]
    require(not any(body.get("selected") is True and body.get("body_status") in (
                "unverified_body_with_contract", "external_specification", "interface_without_checked_body")
                for body in bodies), "a selected local contract has no checked body")
    if expected:
        require(scope.get("complete_selected_scope_verified") is True,
                "goal body checking is incomplete")
        for kind, names in (("exec", executable), ("proof", proofs)):
            checked = [body for body in bodies if body.get("body_status") == f"checked_{kind}_body"]
            require(all(body.get("selected") is True for body in checked),
                    "an unselected body was reported as checked")
            require_function_coverage(checked, names)
            require(scope.get(f"checked_{kind}_bodies") == len(checked),
                    "checked body count disagrees with the named inventory")
    else:
        require_failure_kind(run["stderr"], failure_kind)
        require(not any(message in run["stderr"] for message in (
                    "possible arithmetic underflow/overflow", "invariant not satisfied",
                    "recommendation not met", "error[E")),
                "goal control includes a different failure kind")
        selected = [body for body in bodies
                    if body.get("body_status") == "selected_body_not_certified_by_successful_full_run"]
        require(all(body.get("selected") is True for body in selected),
                "negative goal body selection is incomplete")
        require_function_coverage(selected, executable | proofs)
    if artifact is not None:
        require_publication(data, expected, artifact)
    return data


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
    def __init__(self, output, cwd, commands, *, timeout=None, guard=None,
                 status_key="exit_code"):
        self.output = Path(output)
        self.cwd = Path(cwd)
        self.commands = commands
        self.timeout = timeout
        self.guard = guard
        self.status_key = status_key

    def __call__(self, label, command, environment):
        if self.guard is not None:
            self.guard.check()
        started = time.monotonic()
        try:
            result = run_compiler(command, environment, self.timeout, cwd=self.cwd)
        except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
            self._logs(label, error.stdout or "", error.stderr or "")
            self._record(label, command, environment, started, None,
                         timed_out=isinstance(error, subprocess.TimeoutExpired),
                         interrupted=isinstance(error, KeyboardInterrupt))
            raise
        except OSError as error:
            self._logs(label, "", str(error))
            self._record(label, command, environment, started, None, error=str(error))
            raise
        self._logs(label, result.stdout, result.stderr)
        self._record(label, command, environment, started, result.returncode)
        if self.guard is not None:
            self.guard.check()
        return result.returncode

    def _record(self, label, command, environment, started, exit_code, **details):
        self.commands.append(dict(
            label=label, argv=command, cwd=str(self.cwd), **{self.status_key: exit_code},
            seconds=time.monotonic() - started,
            environment={k: environment[k] for k in (
                "RUSTUP_TOOLCHAIN", "VERUS_Z3_PATH", "VERUS_INFER_CACHE", "CARGO_TARGET_DIR"
            ) if k in environment}, **details,
        ))

    def _logs(self, label, stdout, stderr):
        for suffix, contents in (("stdout", stdout), ("stderr", stderr)):
            (self.output / f"{label}.{suffix}").write_text(contents)
