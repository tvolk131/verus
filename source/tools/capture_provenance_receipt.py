"""Capture a selected-input command receipt, never a compiler/source attestation.

The caller supplies an exact expected hash inventory for explicit files and
recursive trees. Completeness is relative to that selection, not a dependency
closure. Endpoint observations cannot exclude transient changes, substituted
execution, surviving descendants, or artifacts copied by the command. Only use
trusted commands: this helper sanitizes the environment but is not a sandbox.
"""

import hashlib
import json
import math
import os
from pathlib import Path
import re
import signal
import stat
import subprocess
import time


PLAN_SCHEMA = "selected-input-command-plan-v1"
RECEIPT_SCHEMA = "selected-input-command-receipt-v1"
ENVIRONMENT = {"LANG": "C", "LC_ALL": "C", "TZ": "UTC"}


def exact_fields(value, fields, label):
    if type(value) is not dict or set(value) != set(fields):
        raise ValueError(f"{label}: expected exactly {sorted(fields)}")


def absolute_path(value):
    if type(value) is not str or not value or "\0" in value:
        raise ValueError("path must be a nonempty string without NUL")
    path = Path(value)
    if not path.is_absolute() or str(path) != value or ".." in path.parts:
        raise ValueError("path must be absolute and normalized")
    return path


def relative_path(value):
    if type(value) is not str or not value or "\0" in value:
        raise ValueError("inventory path must be a nonempty string without NUL")
    path = Path(value)
    if path.is_absolute() or str(path) != value or any(part in (".", "..") for part in path.parts):
        raise ValueError("inventory path must be normalized and relative")
    if value == ".":
        raise ValueError("select named files or trees, not the inventory root")
    return path


def digest_value(value):
    if type(value) is not str or re.fullmatch("[0-9a-f]{64}", value) is None:
        raise ValueError("expected a lowercase SHA256 digest")


def validate_plan(plan):
    exact_fields(plan, ("schema", "cwd", "argv", "executable_sha256", "compiler",
                        "inventory", "outputs", "timeout_seconds"), "plan")
    if plan["schema"] != PLAN_SCHEMA:
        raise ValueError("unsupported plan schema")
    absolute_path(plan["cwd"])
    argv = plan["argv"]
    if type(argv) is not list or not argv or any(
            type(argument) is not str or "\0" in argument for argument in argv):
        raise ValueError("argv must be a nonempty list of strings without NUL")
    absolute_path(argv[0])
    digest_value(plan["executable_sha256"])
    compiler = plan["compiler"]
    if compiler is not None:
        exact_fields(compiler, ("path", "sha256"), "compiler")
        absolute_path(compiler["path"])
        digest_value(compiler["sha256"])
    inventory = plan["inventory"]
    exact_fields(inventory, ("root", "files", "trees", "expected_sha256"), "inventory")
    absolute_path(inventory["root"])
    for field in ("files", "trees"):
        names = inventory[field]
        if type(names) is not list:
            raise ValueError(f"inventory {field} must be a list")
        for name in names:
            relative_path(name)
        if len(set(names)) != len(names):
            raise ValueError("duplicate inventory selector")
    if not inventory["files"] and not inventory["trees"]:
        raise ValueError("empty inventory selection")
    expected = inventory["expected_sha256"]
    if type(expected) is not dict or not expected:
        raise ValueError("expected inventory must be a nonempty hash mapping")
    for name, digest in expected.items():
        relative_path(name)
        digest_value(digest)
    outputs = plan["outputs"]
    if type(outputs) is not list or not outputs:
        raise ValueError("outputs must be a nonempty list")
    for name in outputs:
        absolute_path(name)
    if len(set(outputs)) != len(outputs):
        raise ValueError("duplicate output path")
    timeout = plan["timeout_seconds"]
    try:
        valid_timeout = type(timeout) in (int, float) and math.isfinite(timeout) and timeout > 0
    except OverflowError:
        valid_timeout = False
    if not valid_timeout:
        raise ValueError("timeout_seconds must be finite and positive")


def file_identity(path):
    path = Path(path)
    if path.resolve(strict=True) != path:
        raise ValueError(f"noncanonical or symlink path: {path}")
    before = path.lstat()
    if not stat.S_ISREG(before.st_mode):
        raise ValueError(f"not a regular file: {path}")
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
        opened = os.fstat(stream.fileno())
    after = path.lstat()
    signature = lambda status: (status.st_dev, status.st_ino, status.st_size,
                                status.st_mtime_ns, status.st_ctime_ns)
    if signature(before) != signature(opened) or signature(opened) != signature(after):
        raise ValueError(f"file changed while hashing: {path}")
    return {"sha256": digest.hexdigest(), "size_bytes": after.st_size,
            "mtime_ns": after.st_mtime_ns}


def selected_inventory(specification):
    root = Path(specification["root"])
    if root.resolve(strict=True) != root or not root.is_dir():
        raise ValueError("inventory root must be a canonical directory")
    selected = {root / name for name in specification["files"]}

    def scan_error(error):
        raise error

    for name in specification["trees"]:
        tree = root / name
        if tree.resolve(strict=True) != tree or not tree.is_dir():
            raise ValueError(f"inventory tree missing or symlinked: {name}")
        for directory, subdirectories, filenames in os.walk(tree, onerror=scan_error, followlinks=False):
            for child in subdirectories + filenames:
                entry = Path(directory) / child
                if entry.is_symlink():
                    raise ValueError(f"symlink in inventory: {entry}")
            selected.update(Path(directory) / filename for filename in filenames)
    return {str(path.relative_to(root)): file_identity(path) for path in sorted(selected)}


def changed_names(before, after):
    return sorted(name for name in before.keys() | after.keys()
                  if before.get(name) != after.get(name))


def content_hashes(inventory):
    return {name: identity["sha256"] for name, identity in inventory.items()}


def invocation_kind(argv):
    """Classify argv shape, not tool behavior, build success, or correspondence."""
    if "--is-vstd" in argv[1:]:
        return "vstd-only-command-shape"
    if Path(argv[0]).name in ("cargo", "cargo.exe") and argv[1:2] == ["build"]:
        packages = [argument.split("=", 1)[1] for argument in argv if argument.startswith("--package=")]
        packages.extend(argv[index + 1] for index, argument in enumerate(argv[:-1])
                        if argument in ("-p", "--package"))
        if "rust_verify" in packages:
            return "engine-build-command-shape"
    return "other-command"


def capture(plan, output):
    """Run one trusted command; retain failures, never reuse a receipt directory.

    Malformed plans raise before creating evidence. Runtime/preflight failures
    produce success=false receipts. Required artifacts must not exist beforehand
    and must have modification timestamps no earlier than command launch.
    """
    validate_plan(plan)
    plan = json.loads(json.dumps(plan))
    output = absolute_path(str(output))
    if output.parent.resolve(strict=True) != output.parent:
        raise ValueError("receipt parent must be canonical")
    output.mkdir(exist_ok=False)
    receipt = {
        "schema": RECEIPT_SCHEMA, "plan": plan, "environment": dict(ENVIRONMENT),
        "inventory_scope": "complete only for declared files and recursive trees at endpoint snapshots",
        "invocation_kind": invocation_kind(plan["argv"]),
        "classification_basis": "argv shape only; not executable behavior or build attestation",
        "compiler_directly_named_as_executable": plan["compiler"] is not None
            and plan["compiler"]["path"] == plan["argv"][0],
        "establishes": {"binary_source_correspondence": False, "runtime_inventory_completeness": False,
                        "native_proof": False, "historical_custody": False},
        "inputs_before": None, "inputs_after": None, "input_drift": None,
        "executable_before": None, "executable_after": None,
        "compiler_before": None, "compiler_after": None,
        "outputs": {}, "command": {"started": False, "exit_code": None, "timed_out": False,
                                   "interrupted": False,
                                   "launch_time_ns": None},
        "errors": [], "success": False,
    }
    started = time.monotonic()
    launched_ns = None
    process = None
    try:
        cwd = Path(plan["cwd"])
        if cwd.resolve(strict=True) != cwd or not cwd.is_dir():
            raise ValueError("cwd must be a canonical directory")
        before = selected_inventory(plan["inventory"])
        receipt["inputs_before"] = before
        mismatch = changed_names(plan["inventory"]["expected_sha256"], content_hashes(before))
        if mismatch:
            raise ValueError(f"initial selected-input mismatch: {mismatch}")
        executable = file_identity(Path(plan["argv"][0]))
        receipt["executable_before"] = executable
        if executable["sha256"] != plan["executable_sha256"]:
            raise ValueError("executable identity mismatch")
        if plan["compiler"] is not None:
            compiler = file_identity(Path(plan["compiler"]["path"]))
            receipt["compiler_before"] = compiler
            if compiler["sha256"] != plan["compiler"]["sha256"]:
                raise ValueError("compiler identity mismatch")
        protected = {Path(plan["inventory"]["root"]) / name for name in before}
        protected.add(Path(plan["argv"][0]))
        if plan["compiler"] is not None:
            protected.add(Path(plan["compiler"]["path"]))
        for name in plan["outputs"]:
            artifact = Path(name)
            if artifact in protected or artifact.is_relative_to(output):
                raise ValueError(f"output overlaps an input or receipt directory: {artifact}")
            if artifact.parent.resolve(strict=True) != artifact.parent:
                raise ValueError(f"output parent is not canonical: {artifact}")
            if os.path.lexists(artifact):
                raise ValueError(f"seeded output refused: {artifact}")
        log = output / "command.log"
        with log.open("xb") as stream:
            launched_ns = time.time_ns()
            receipt["command"]["launch_time_ns"] = launched_ns
            process = subprocess.Popen(plan["argv"], cwd=cwd, env=dict(ENVIRONMENT),
                                       stdout=stream, stderr=subprocess.STDOUT,
                                       start_new_session=os.name == "posix")
            receipt["command"]["started"] = True
            try:
                receipt["command"]["exit_code"] = process.wait(timeout=plan["timeout_seconds"])
            except subprocess.TimeoutExpired:
                receipt["command"]["timed_out"] = True
                if os.name == "posix":
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                else:
                    process.kill()
                receipt["command"]["exit_code"] = process.wait()
        if receipt["command"]["timed_out"] or receipt["command"]["exit_code"] != 0:
            raise ValueError("command failed or timed out")
    except (KeyboardInterrupt, SystemExit) as error:
        receipt["command"]["interrupted"] = True
        receipt["errors"].append(f"command interrupted: {type(error).__name__}: {error}")
        if process is not None:
            try:
                if os.name == "posix":
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                elif process.poll() is None:
                    process.kill()
                receipt["command"]["exit_code"] = process.wait()
            except Exception as cleanup_error:
                receipt["errors"].append(
                    f"interruption cleanup failed: {type(cleanup_error).__name__}: {cleanup_error}")
        raise
    except Exception as error:
        receipt["errors"].append(f"{type(error).__name__}: {error}")
    finally:
        try:
            after = selected_inventory(plan["inventory"])
            receipt["inputs_after"] = after
            if receipt["inputs_before"] is not None:
                receipt["input_drift"] = changed_names(content_hashes(receipt["inputs_before"]),
                                                       content_hashes(after))
                if receipt["input_drift"]:
                    receipt["errors"].append("selected inputs drifted")
        except Exception as error:
            receipt["errors"].append(f"input rescan failed: {type(error).__name__}: {error}")
        for role, path in (("executable", plan["argv"][0]),
                           ("compiler", None if plan["compiler"] is None else plan["compiler"]["path"])):
            if path is not None:
                try:
                    identity = file_identity(Path(path))
                    receipt[f"{role}_after"] = identity
                    if receipt[f"{role}_before"] != identity:
                        receipt["errors"].append(f"{role} identity drift or unavailable initial identity")
                except Exception as error:
                    receipt["errors"].append(f"{role} rescan failed: {type(error).__name__}: {error}")
        if receipt["command"]["started"]:
            for name in plan["outputs"]:
                try:
                    identity = file_identity(Path(name))
                    receipt["outputs"][name] = identity
                    if identity["mtime_ns"] < launched_ns:
                        receipt["errors"].append(f"stale output timestamp: {name}")
                except Exception as error:
                    receipt["errors"].append(f"output capture failed: {type(error).__name__}: {error}")
        if launched_ns is not None:
            try:
                receipt["command"]["log"] = file_identity(output / "command.log")
            except Exception as error:
                receipt["errors"].append(f"log capture failed: {type(error).__name__}: {error}")
        receipt["command"]["seconds"] = time.monotonic() - started
        receipt["success"] = (receipt["command"]["started"]
                              and receipt["command"]["exit_code"] == 0
                              and not receipt["command"]["timed_out"]
                              and not receipt["command"]["interrupted"]
                              and not receipt["errors"])
        with (output / "receipt.json").open("x") as stream:
            json.dump(receipt, stream, indent=2, sort_keys=True, allow_nan=False)
            stream.write("\n")
    return receipt
