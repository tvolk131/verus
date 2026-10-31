import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time


ROOT = Path(__file__).resolve().parent


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("label")
    parser.add_argument("--cwd", default=str(ROOT.parents[1]))
    parser.add_argument("--timeout", type=int, default=1800)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    arguments = parser.parse_args()
    command = arguments.command
    if command and command[0] == "--":
        command = command[1:]
    output = ROOT / "raw"
    output.mkdir(exist_ok=True)
    receipt = output / (arguments.label + ".json")
    if receipt.exists():
        raise SystemExit("Refusing to overwrite frozen run: " + str(receipt))
    started = time.monotonic()
    record = {
        "label": arguments.label,
        "argv": command,
        "cwd": str(Path(arguments.cwd).resolve()),
        "started_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "environment": {key: os.environ[key] for key in (
            "PATH", "RUSTUP_TOOLCHAIN", "RUSTC_BOOTSTRAP", "RUST_MIN_STACK",
            "VERUS_Z3_PATH", "VERUS_INFER_CACHE", "CARGO_TARGET_DIR",
            "CARGO_HOME", "RUSTUP_HOME", "CARGO_NET_OFFLINE"
        ) if key in os.environ},
        "timeout_seconds": arguments.timeout,
        "selection_sha256": digest(ROOT / "selection.json"),
    }
    stdout_path = output / (arguments.label + ".stdout")
    stderr_path = output / (arguments.label + ".stderr")
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        try:
            process = subprocess.run(command, cwd=arguments.cwd, stdout=stdout,
                                     stderr=stderr, timeout=arguments.timeout)
            record["exit_status"] = process.returncode
        except subprocess.TimeoutExpired:
            record["exit_status"] = None
            record["timed_out"] = True
        except OSError as error:
            record["exit_status"] = None
            record["launch_error"] = str(error)
            stderr.write((str(error) + "\n").encode())
    record["elapsed_seconds"] = time.monotonic() - started
    record["stdout_sha256"] = digest(stdout_path)
    record["stderr_sha256"] = digest(stderr_path)
    receipt.write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps(record, indent=2))
    raise SystemExit(record["exit_status"] if record["exit_status"] is not None
                     else 127 if "launch_error" in record else 124)


if __name__ == "__main__":
    main()
