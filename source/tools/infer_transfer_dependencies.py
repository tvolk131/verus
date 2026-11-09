"""Pinned registry graph and archive/source checks shared by setup and acceptance."""

import json
from pathlib import Path
import re
import tarfile

from check_fedimint_full_request import lock_packages

from infer_eval_support import digest
from infer_experiment import require


LOCK = Path(__file__).with_name("infer_full_request.Cargo.lock")
LOCK_SHA256 = "cf1feb5b8762e24c62504f5651b425a8d59528e6aed1e6dba85129cc8f409283"
REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"


def packages(application_lock):
    require(digest(LOCK) == LOCK_SHA256, "reviewed dependency graph changed")
    pinned = lock_packages(LOCK.read_text())
    available = lock_packages(application_lock)
    selected = []
    for entry in pinned.values():
        if "source" not in entry:
            require(entry["name"] == "full-request-dependency", "unexpected local package")
            continue
        identity = {key: entry[key] for key in ("name", "version", "source", "checksum")}
        candidate = available.get((entry["name"], entry["version"]), {})
        require(identity == {key: candidate.get(key) for key in identity},
                "dependency differs from pinned application lock")
        require(identity["source"] == REGISTRY, "unexpected registry")
        selected.append(identity)
    require(len(selected) == 21, "incomplete pinned dependency graph")
    return selected


def validate_metadata(data, selected):
    observed = {(entry["name"], entry["version"]): entry for entry in data["packages"]}
    identities = {(entry["name"], entry["version"]) for entry in selected} | {("full-request-dependency", "0.1.0")}
    require(set(observed) == identities and len(observed) == len(data["packages"]), "Cargo graph identity drift")
    for entry in selected:
        require(observed[(entry["name"], entry["version"])]["source"] == entry["source"],
                "Cargo metadata source substitution")
    identifiers = {entry["id"]: entry["name"] for entry in observed.values()}
    require(len(identifiers) == len(observed), "duplicate Cargo package ID")
    expected = {}
    for block in LOCK.read_text().split("[[package]]")[1:]:
        name = re.search(r'^name = "([^"\n]+)"$', block, re.M).group(1)
        dependencies = re.search(r'^dependencies = (\[[^]]*\])', block, re.M)
        expected[name] = set(re.findall(r'"([^"\n]+)"', dependencies.group(1)) if dependencies else [])
    nodes = data["resolve"]["nodes"]
    actual = {identifiers[node["id"]]: {identifiers[dependency] for dependency in node["dependencies"]} for node in nodes}
    require(len(nodes) == len(actual) and actual == expected, "Cargo dependency edges differ from reviewed lock")
    return observed


def archive_sources(archive, directory, checksum):
    archive, directory = Path(archive), Path(directory)
    require(not archive.is_symlink() and not directory.is_symlink(), "linked registry input")
    require(digest(archive) == checksum, "registry archive checksum mismatch")
    paths = [archive]
    names = set()
    with tarfile.open(archive) as contents:
        for member in contents.getmembers():
            relative = Path(member.name)
            require(relative.parts and relative.parts[0] == directory.name
                    and not relative.is_absolute() and ".." not in relative.parts,
                    "unsafe registry archive path")
            require(member.isdir() or member.isfile(), "registry archive contains a link or special file")
            require(member.name not in names, "duplicate registry archive member")
            names.add(member.name)
            if member.isfile():
                path = directory.parent / relative
                require(not any(parent.is_symlink() for parent in (path, *path.parents)),
                        "linked registry source")
                require(path.is_file() and path.read_bytes() == contents.extractfile(member).read(),
                        "registry source differs from pinned archive")
                paths.append(path)
    expected = {path.resolve() for path in paths[1:]}
    require(all(not path.is_symlink() and (path.is_file() or path.is_dir()) for path in directory.rglob("*")),
            "linked or special registry source")
    actual = {path.resolve() for path in directory.rglob("*") if path.is_file()
              and path.name not in (".cargo-ok", ".cargo-checksum.json")}
    require(actual == expected, "extra or missing registry sources")
    checksum_file = directory / ".cargo-checksum.json"
    if checksum_file.exists():
        checksum_data = json.loads(checksum_file.read_text())
        require(checksum_data.get("package") == checksum, "registry source checksum marker differs")
    return paths
