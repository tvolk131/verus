import datetime
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tarfile


ROOT = Path(__file__).resolve().parent
REPOSITORY = ROOT.parents[1]


def command(*arguments):
    return subprocess.check_output(arguments, cwd=REPOSITORY, text=True).strip()


def identity(path):
    path = Path(path).resolve()
    metadata = path.stat()
    return {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "bytes": metadata.st_size, "device": metadata.st_dev, "inode": metadata.st_ino}


def main():
    destination = ROOT / "artifacts-before-proofs.json"
    if destination.exists():
        raise SystemExit("Artifacts already frozen")
    build = REPOSITORY / "source/target-verus/release"
    names = ("verus", "rust_verify", "cargo-verus", "vstd.vir", "libvstd.rlib",
             "libverus_builtin.rlib", "libverus_builtin_macros.dylib",
             "libverus_state_machines_macros.dylib")
    artifacts = {name: identity(build / name) for name in names}
    sysroot = Path(command("rustc", "--print", "sysroot"))
    driver = list((sysroot / "lib").glob("librustc_driver-*.dylib"))
    assert len(driver) == 1
    rustc = Path(command("rustup", "which", "rustc"))
    z3 = REPOSITORY / "source/z3"
    parent_z3 = Path.home() / "Documents/GitHub/verus/source/z3"
    shutil.copyfile(REPOSITORY / "source/vstd/Cargo.lock", ROOT / "vstd-build.Cargo.lock")
    archive = ROOT / "upstream/unsigned-varint-0.8.0.crate"
    archive_checks = {}
    with tarfile.open(archive) as package:
        for relative in ("src/encode.rs", "src/decode.rs", "src/lib.rs", "Cargo.toml", "Cargo.toml.orig", "LICENSE", ".cargo_vcs_info.json"):
            data = package.extractfile("unsigned-varint-0.8.0/" + relative).read()
            archive_checks[relative] = data == (ROOT / "upstream" / relative).read_bytes()
    assert all(archive_checks.values())
    index = Path.home() / ".cargo/registry/index/index.crates.io-1949cf8c6b5b557f/.cache/un/si/unsigned-varint"
    registry = [json.loads(part) for part in index.read_bytes().split(b"\0") if part.startswith(b'{"name"')]
    registry_version = next(entry for entry in registry if entry["vers"] == "0.8.0")
    assert registry_version["cksum"] == identity(archive)["sha256"]
    receipt = {
        "recorded_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "baseline": command("git", "rev-parse", "HEAD"),
        "engine_source_diff": command("git", "diff", "HEAD", "--", "source", "dependencies", "tools", "rust-toolchain.toml"),
        "independent_build_root": str(build.resolve()),
        "parent_release_artifacts_used": False,
        "artifacts": artifacts,
        "rustc_version_verbose": command("rustc", "-Vv"),
        "rustc": identity(rustc),
        "rustc_driver": identity(driver[0]),
        "cargo": identity(command("rustup", "which", "cargo")),
        "z3": identity(z3),
        "z3_version": command(str(z3), "-version"),
        "z3_origin": "Copied only the standalone solver from the parent source/z3; no parent verifier, macros, or vstd artifacts used. Solver supply-chain origin is not independently authenticated.",
        "z3_matches_parent_solver_sha256": identity(z3)["sha256"] == identity(parent_z3)["sha256"],
        "z3_independent_inode": z3.stat().st_ino != parent_z3.stat().st_ino,
        "verus_version": command(str(build / "verus"), "--version"),
        "source_inputs": {relative: identity(REPOSITORY / relative) for relative in (
            "rust-toolchain.toml", "source/Cargo.lock", "source/.cargo/config.toml",
            "source/Makefile.toml", "source/vstd/Cargo.lock")},
        "archive_members_match_source_snapshot": archive_checks,
        "cached_registry_index_version": registry_version,
        "archive_matches_cached_registry_checksum": True,
        "upstream_origin_limit": "Crate archive, locally cached registry checksum and VCS metadata agree. No independent network retrieval or upstream Git checkout authenticated the claimed revision; the selected exact bytes are pinned and bundled.",
        "selection_sha256": identity(ROOT / "selection.json")["sha256"],
        "fixture_hashes": json.loads((ROOT / "pre-proof-fixtures.json").read_text()),
    }
    destination.write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({"receipt": str(destination), "version": receipt["verus_version"],
                      "z3": receipt["z3_version"]}, indent=2))


if __name__ == "__main__":
    main()
