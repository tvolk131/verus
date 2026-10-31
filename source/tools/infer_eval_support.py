"""Shared, standard-library-only measurement and artifact-stability helpers."""
import hashlib
import json
import math
import os
import platform
from pathlib import Path
import signal
import subprocess
import sys
import time


def host_info():
    return dict(platform=platform.platform(), machine=platform.machine(),
                python=sys.version, logical_cpus=os.cpu_count(),
                started_utc=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()))


def digest(path):
    value = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


class ArtifactChanged(RuntimeError):
    pass


class ArtifactGuard:
    """Hash immutable tool inputs outside the measured compiler invocation."""
    def __init__(self, paths):
        self.paths = sorted({str(Path(p).resolve()) for p in paths})
        self.initial = self.snapshot()
        missing = [p for p, value in self.initial.items() if value is None]
        if missing:
            raise ArtifactChanged("missing initial artifacts: " + ", ".join(missing))
        self.changes = []

    def snapshot(self):
        result = {}
        for path in self.paths:
            try:
                stat = Path(path).stat()
                result[path] = {"sha256": digest(path), "size": stat.st_size,
                                "mtime_ns": stat.st_mtime_ns, "inode": stat.st_ino}
            except (FileNotFoundError, OSError):
                result[path] = None
        return result

    def check(self):
        observed = self.snapshot()
        changed = [p for p in self.paths if self.initial[p] != observed[p]]
        if changed:
            self.changes.append({"observed": observed, "changed": changed})
            raise ArtifactChanged("tool inputs changed during evaluation: " + ", ".join(changed))


def support_artifacts():
    return [Path(__file__), Path(__file__).with_name("infer_experiment.py")]


def tool_artifacts(verifier, solver, extra=()):
    directory = Path(verifier).parent
    names = ["rust_verify", "vstd.vir", "libvstd.rlib", "libverus_builtin.rlib"]
    paths = [Path(verifier), Path(solver), *support_artifacts(), *extra,
             *(directory / n for n in names)]
    # The exact filenames vary by host. Missing macro libraries are a setup error.
    for stem in ("verus_builtin_macros", "verus_state_machines_macros"):
        found = [p for prefix in ("lib", "") for p in directory.glob(prefix + stem + ".*")
                 if p.suffix in (".so", ".dylib", ".dll")]
        paths.extend(found or [directory / ("lib" + stem + ".missing")])
    return paths


def run_compiler(command, env, timeout, cwd=None):
    with subprocess.Popen(command, cwd=cwd, env=env, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, text=True,
                          start_new_session=(os.name == "posix")) as process:
        try:
            stdout, stderr = process.communicate(timeout=timeout)
        except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
            try:
                if os.name == "posix":
                    os.killpg(process.pid, signal.SIGINT)
                else:
                    process.kill()
            except ProcessLookupError:
                pass
            try:
                error.stdout, error.stderr = process.communicate(timeout=5)
            except (subprocess.TimeoutExpired, KeyboardInterrupt):
                try:
                    if os.name == "posix":
                        os.killpg(process.pid, signal.SIGKILL)
                    else:
                        process.kill()
                except ProcessLookupError:
                    pass
                error.stdout, error.stderr = process.communicate()
            finally:
                if os.name == "posix":
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
            raise
        return subprocess.CompletedProcess(command, process.returncode, stdout, stderr)


def measured(command, env, timeout, cwd, memory=False):
    if memory:
        # A fresh worker gives each sample its own RUSAGE_CHILDREN high-water
        # mark. This is the OS's waited-child maximum RSS, not a summed or
        # simultaneous process-tree peak. Worker startup is outside wall time.
        worker = subprocess.run([sys.executable, str(Path(__file__).resolve()), "--worker"],
                                input=json.dumps(dict(command=command, env=env, timeout=timeout,
                                                      cwd=str(cwd))),
                                text=True, capture_output=True, check=True)
        result = json.loads(worker.stdout)
        if result.get("timeout"):
            error = subprocess.TimeoutExpired(command, timeout, result["stdout"], result["stderr"])
            raise error
        return result
    started = time.perf_counter_ns()
    run = run_compiler(command, env, timeout, cwd)
    return dict(returncode=run.returncode, stdout=run.stdout, stderr=run.stderr,
                wall_micros=(time.perf_counter_ns() - started) // 1000,
                memory=None)


def statistics(values):
    values = sorted(values)
    if not values:
        return {"n": 0}
    n = len(values)
    median = (values[(n - 1) // 2] + values[n // 2]) / 2
    return dict(n=n, min=values[0], median=median,
                p90=values[math.ceil(0.90 * n) - 1], max=values[-1])


def grouped_statistics(rows):
    result = {}
    for case, phase in sorted({(r["case"], r["phase"]) for r in rows}):
        group = [r for r in rows if r["case"] == case and r["phase"] == phase]
        # Expected negative cases are valid timing observations, but a harness
        # error or unexpected proof failure never enters performance statistics.
        passed = [r for r in group if not r["errors"]]
        result[f"{case}/{phase}"] = {
            "observations": len(group), "excluded_failures": len(group) - len(passed),
            "wall_micros": statistics([r["wall_micros"] for r in passed]),
            "waited_child_max_rss_bytes": statistics([
                r["memory"]["bytes"] for r in passed if r.get("memory") and r["memory"].get("bytes") is not None]),
        }
    return result


def _worker():
    args = json.load(sys.stdin)
    started = time.perf_counter_ns()
    try:
        run = run_compiler(**args)
        result = dict(returncode=run.returncode, stdout=run.stdout, stderr=run.stderr)
    except subprocess.TimeoutExpired as error:
        result = dict(timeout=True, stdout=error.stdout or "", stderr=error.stderr or "")
    result["wall_micros"] = (time.perf_counter_ns() - started) // 1000
    try:
        import resource
        raw = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
        if sys.platform == "darwin":
            value = int(raw)
        elif sys.platform.startswith("linux"):
            value = int(raw * 1024)
        else:
            raise RuntimeError("ru_maxrss units unspecified on this platform")
        result["memory"] = dict(bytes=value, method="RUSAGE_CHILDREN.ru_maxrss in a fresh worker",
                                scope="OS waited-child maximum; not simultaneous process-tree peak")
    except (ImportError, RuntimeError) as error:
        result["memory"] = dict(bytes=None, unavailable=str(error))
    print(json.dumps(result))


if __name__ == "__main__":
    if sys.argv[1:] != ["--worker"]:
        raise SystemExit("internal measurement worker")
    _worker()
