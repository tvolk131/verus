"""Verify and prepare preserved research inputs without a compiler or network."""

import argparse
import gzip
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import sys
import tarfile


ROOT = Path(__file__).resolve().parents[2]
MANIFEST = "evidence/redb-native-mir-campaign115/portable-inputs/MANIFEST.json"
MAX_OUTPUT_BYTES = 300 * 1024 * 1024
MAX_MEMBERS = 10000
RUNTIME_SOURCE = "kernel170/candidate/source/rust_verify/src/bin/continuation160.rs"
OLD_RUNTIME = 'PathBuf::from("/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/lib")'
NEW_RUNTIME = ('PathBuf::from(std::env::var_os("VERUS_NATIVE_RUNTIME_LIBDIR")'
               '.expect("set VERUS_NATIVE_RUNTIME_LIBDIR to the selected sysroot target library directory"))')


def digest(path):
    result = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            result.update(chunk)
    return result.hexdigest()


def relative_path(value):
    if not isinstance(value, str) or not value or "\\" in value or ":" in value:
        raise ValueError("invalid portable relative path: " + repr(value))
    path = PurePosixPath(value)
    if path.is_absolute() or any(part in {"", ".", ".."} for part in value.split("/")):
        raise ValueError("unsafe portable relative path: " + value)
    return Path(*path.parts)


def input_path(root, name):
    relative = relative_path(name)
    current = root
    for part in relative.parts:
        current = current / part
        if current.is_symlink():
            raise ValueError("symlink input refused: " + str(current))
    if not current.is_file():
        raise ValueError("missing regular input: " + str(current))
    return current


def gzip_identity(path, expected):
    result = hashlib.sha256()
    length = 0
    with gzip.open(path, "rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            length += len(chunk)
            if length > expected["bytes"] or length > MAX_OUTPUT_BYTES:
                raise ValueError("gzip output exceeds declared bound")
            result.update(chunk)
    if length != expected["bytes"] or result.hexdigest() != expected["sha256"]:
        raise ValueError("decompressed historical input identity mismatch")


def verify(root):
    root = Path(root).resolve()
    manifest_path = input_path(root, MANIFEST)
    manifest = json.loads(manifest_path.read_text())
    if manifest.get("format") != 1:
        raise ValueError("unsupported handoff manifest format")
    identifiers = set()
    for entry in manifest["inputs"]:
        if entry["id"] in identifiers:
            raise ValueError("duplicate input identifier")
        identifiers.add(entry["id"])
        if entry["kind"] not in {"tar", "gzip", "file", "reference"}:
            raise ValueError("unsupported input kind")
        path = input_path(root, entry["path"])
        if path.stat().st_size != entry["bytes"] or digest(path) != entry["sha256"]:
            raise ValueError("input identity mismatch: " + entry["path"])
        if entry["kind"] != "reference":
            relative_path(entry["destination"])
        if entry["kind"] == "gzip":
            gzip_identity(path, entry["uncompressed"])
    return manifest


def archive_members(saved):
    members = saved.getmembers()
    if len(members) > MAX_MEMBERS:
        raise ValueError("archive member bound exceeded")
    seen = set()
    total = 0
    for member in members:
        name = member.name.rstrip("/") if member.isdir() else member.name
        relative = relative_path(name)
        if relative in seen:
            raise ValueError("duplicate archive path: " + name)
        seen.add(relative)
        if not (member.isfile() or member.isdir()):
            raise ValueError("archive links and special files refused: " + name)
        if member.size < 0:
            raise ValueError("negative archive member size")
        total += member.size
        if total > MAX_OUTPUT_BYTES:
            raise ValueError("archive output bound exceeded")
    return members, total


def prepare(root, destination):
    root = Path(root).resolve()
    manifest = verify(root)
    destination = Path(destination).absolute()
    if destination.exists() or destination.is_symlink():
        raise ValueError("destination must not exist")
    planned = []
    total = 0
    for entry in manifest["inputs"]:
        if entry["kind"] == "reference":
            continue
        source = input_path(root, entry["path"])
        if entry["kind"] == "tar":
            with tarfile.open(source, "r:*") as saved:
                members, length = archive_members(saved)
            planned.append((entry, members))
        else:
            length = entry["uncompressed"]["bytes"] if entry["kind"] == "gzip" else entry["bytes"]
            planned.append((entry, None))
        total += length
        if total > MAX_OUTPUT_BYTES:
            raise ValueError("combined preparation output bound exceeded")
    destination.mkdir(parents=True)
    for entry, members in planned:
        source = input_path(root, entry["path"])
        output = destination / relative_path(entry["destination"])
        if entry["kind"] == "tar":
            with tarfile.open(source, "r:*") as saved:
                for member in members:
                    path = output / relative_path(member.name.rstrip("/") if member.isdir() else member.name)
                    if member.isdir():
                        path.mkdir(parents=True, exist_ok=True)
                        continue
                    path.parent.mkdir(parents=True, exist_ok=True)
                    with saved.extractfile(member) as content, path.open("xb") as target:
                        shutil.copyfileobj(content, target)
                    path.chmod(member.mode & 0o777)
        else:
            output.parent.mkdir(parents=True, exist_ok=True)
            opener = gzip.open if entry["kind"] == "gzip" else open
            with opener(source, "rb") as content, output.open("xb") as target:
                shutil.copyfileobj(content, target)
            expected = entry["uncompressed"] if entry["kind"] == "gzip" else entry
            if output.stat().st_size != expected["bytes"] or digest(output) != expected["sha256"]:
                raise ValueError("prepared input identity mismatch")
    runtime_patch = None
    if manifest.get("portable_runtime_patch"):
        source = destination / RUNTIME_SOURCE
        original = source.read_bytes()
        if original.count(OLD_RUNTIME.encode()) != 1:
            raise ValueError("historical runtime path precondition mismatch")
        replacement = original.replace(OLD_RUNTIME.encode(), NEW_RUNTIME.encode())
        source.write_bytes(replacement)
        runtime_patch = dict(path=RUNTIME_SOURCE,
                             before_sha256=hashlib.sha256(original).hexdigest(),
                             after_sha256=hashlib.sha256(replacement).hexdigest(),
                             status="SOURCE_ONLY_UNCOMPILED_UNREVIEWED_PATH_ADAPTER")
    if verify(root) != manifest:
        raise ValueError("input endpoint drift during preparation")
    result = dict(status="PREPARED_NOT_BUILT_NOT_APPROVED", all18="OPEN", native_export="REFUSED",
                  manifest_sha256=digest(root / MANIFEST), prepared_bytes_before_path_adapter=total,
                  runtime_path_adapter=runtime_patch, paths=manifest.get("prepared_paths", {}),
                  compiler_solver_or_application_executed=False)
    with (destination / "PREPARED.json").open("x") as output:
        json.dump(result, output, indent=2, sort_keys=True)
        output.write("\n")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=["verify", "prepare"])
    parser.add_argument("--output", type=Path)
    arguments = parser.parse_args()
    if arguments.operation == "prepare" and arguments.output is None:
        parser.error("prepare requires --output with a new destination")
    if arguments.operation == "verify" and arguments.output is not None:
        parser.error("verify does not create an output directory")
    try:
        if arguments.operation == "verify":
            manifest = verify(ROOT)
            result = dict(status="INPUT_IDENTITIES_MATCH", inputs=len(manifest["inputs"]),
                          all18="OPEN", compiler_solver_or_application_executed=False)
        else:
            result = prepare(ROOT, arguments.output)
    except (OSError, ValueError, KeyError, TypeError, tarfile.TarError, EOFError) as error:
        print("HANDOFF_REFUSED: " + str(error), file=sys.stderr)
        return 1
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
