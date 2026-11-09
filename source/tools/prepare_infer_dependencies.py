"""Fetch only the full-request's reviewed real registry graph, before offline proof."""

import argparse
import json
import os
from pathlib import Path
import shutil

from check_fedimint_full_request import DEPENDENCY, INPUT_HASHES
from check_infer_transfers import check_application
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, positive_timeout, replay_receipt, require
from infer_transfer_dependencies import LOCK, LOCK_SHA256, archive_sources, packages, validate_metadata
from prepare_infer_transfers import check_repository_root, git_environment, nonsymlinked


def cargo_environment(home):
    environment = {key: value for key, value in git_environment().items()
                   if not key.startswith(("CARGO_", "RUSTC", "RUSTDOC", "RUSTFLAGS", "RUSTUP_TOOLCHAIN"))}
    environment.update(CARGO_HOME=str(home), CARGO_NET_OFFLINE="true", CARGO_BUILD_JOBS="2",
                       RUSTUP_TOOLCHAIN="1.99.1")
    return environment


def index_path(name):
    if len(name) < 3:
        return str(len(name)) + "/" + name
    if len(name) == 3:
        return "3/" + name[0] + "/" + name
    return name[:2] + "/" + name[2:4] + "/" + name


def seed_cache(home, cached_home, selected):
    cached_home = Path(cached_home).resolve()
    candidates = [directory for directory in (cached_home / "registry/cache").glob("*")
                  if all((directory / (entry["name"] + "-" + entry["version"] + ".crate")).is_file()
                         for entry in selected)]
    require(len(candidates) == 1, "cached registry archives are missing or ambiguous")
    registry = candidates[0].name
    for entry in selected:
        name = entry["name"] + "-" + entry["version"] + ".crate"
        source = cached_home / "registry/cache" / registry / name
        require(not source.is_symlink() and digest(source) == entry["checksum"], "cached archive checksum mismatch")
        destination = home / "registry/cache" / registry / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
        relative = Path("registry/index") / registry / ".cache" / index_path(entry["name"])
        source = cached_home / relative
        require(source.is_file() and not source.is_symlink(), "missing cached sparse index entry")
        destination = home / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
    relative = Path("registry/index") / registry / "config.json"
    config = json.loads((cached_home / relative).read_text())
    require(config.get("dl") == "https://static.crates.io/crates", "unexpected cached registry endpoint")
    (home / relative).write_text(json.dumps(config) + "\n")


def prepare(application, destination, output, timeout, cache_home=None):
    application, destination, output = map(lambda path: Path(path).absolute(), (application, destination, output))
    for path in (destination, output):
        nonsymlinked(path)
        require(not path.exists(), "dependency and evidence destinations must be fresh")
    for left, right in ((destination, output), (application, destination), (application, output)):
        require(not left.resolve().is_relative_to(right.resolve())
                and not right.resolve().is_relative_to(left.resolve()), "setup destinations must be disjoint")
    output.mkdir(parents=True, mode=0o700)
    receipt = dict(schema_version=1, kind="full_request_dependency_preparation", commands=[],
                   application_proofs_proved=False, network_fetch=cache_home is None)
    try:
        with replay_receipt(output, receipt):
            destination.mkdir(parents=True, mode=0o700)
            home, project = destination / "cargo-home", destination / "project"
            home.mkdir(mode=0o700)
            (project / "src").mkdir(parents=True)
            environment = cargo_environment(home)
            run = LoggedCommands(output, project, receipt["commands"], timeout=timeout)
            check_repository_root(run, "fedimint", application, environment, "before")
            check_application(run, "fedimint-full-request", application, environment, "before")
            require(digest(application / "Cargo.lock") == INPUT_HASHES["Cargo.lock"], "application lock drift")
            selected = packages((application / "Cargo.lock").read_text())
            application_guard = ArtifactGuard([application / relative for relative in INPUT_HASHES])
            receipt["application_before"] = application_guard.initial
            (project / "Cargo.toml").write_text(DEPENDENCY)
            (project / "Cargo.lock").write_bytes(LOCK.read_bytes())
            (project / "src/lib.rs").write_text("pub use bitcoin::hashes::sha256;\n")
            inputs = ArtifactGuard([LOCK, *(project / relative for relative in ("Cargo.toml", "Cargo.lock", "src/lib.rs"))])
            run.guard = inputs
            if cache_home is not None:
                seed_cache(home, cache_home, selected)
            fetch_environment = dict(environment, CARGO_NET_OFFLINE="true" if cache_home else "false")
            fetch = ["cargo", "fetch", "--locked", "--manifest-path", str(project / "Cargo.toml")]
            if cache_home is not None:
                fetch.append("--offline")
            require(run("dependency_fetch", fetch, fetch_environment) == 0, "pinned dependency fetch failed")
            require(run("dependency_metadata", ["cargo", "metadata", "--offline", "--locked", "--format-version", "1",
                        "--manifest-path", str(project / "Cargo.toml")], environment) == 0,
                    "offline pinned metadata validation failed")
            metadata = json.loads((output / "dependency_metadata.stdout").read_text())
            observed = validate_metadata(metadata, selected)
            guarded = []
            for entry in selected:
                package = observed[(entry["name"], entry["version"])]
                require(package["source"] == entry["source"], "dependency source substitution")
                source = Path(package["manifest_path"]).parent
                require(source.resolve().is_relative_to(home.resolve()), "registry source escapes prepared home")
                archive = home / "registry/cache" / source.parent.name / (source.name + ".crate")
                guarded.extend(archive_sources(archive, source, entry["checksum"]))
            archives = list((home / "registry/cache").glob("*/*.crate"))
            require(len(archives) == 21 and not (destination / "target").exists(), "unexpected fetch/build inputs")
            application_guard.check()
            inputs.check()
            check_repository_root(run, "fedimint", application, environment, "after")
            check_application(run, "fedimint-full-request", application, environment, "after")
            receipt.update(cargo_home=str(home), packages=selected, lock_sha256=LOCK_SHA256,
                           sources=ArtifactGuard(guarded).initial, application_after=application_guard.snapshot())
    finally:
        print(output)
    return home


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("application", "destination", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--timeout", type=positive_timeout, default=180)
    parser.add_argument("--cache-home", type=Path, help="optional read-only real Cargo cache; setup also stays offline")
    arguments = parser.parse_args()
    prepare(arguments.application, arguments.destination, arguments.output, arguments.timeout, arguments.cache_home)


if __name__ == "__main__":
    main()
