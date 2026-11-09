"""Reconstruct dirty77 plus the bounded delta IN MEMORY, without a checkout."""

import argparse
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import subprocess


ROOT = Path(__file__).resolve().parents[2]
CHECKPOINT = "3b05c1912b9d7314ad00c832c4065f4362562e02"
BASE_PATCH = "545e330ee87d539aa727154a6121f251f557a18867be62517a9867592a372e12"
PREFIX = "evidence/transfer-fedimint-atomic-replay/"
TOOLS = {"source/tools/check_fedimint_atomic_replay.py", "source/tools/test_fedimint_atomic_replay.py"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(content):
    return hashlib.sha256(content).hexdigest()


def git(*arguments, content=None):
    environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
    environment["GIT_OPTIONAL_LOCKS"] = "0"
    return subprocess.check_output(["git", "-c", "core.fsmonitor=false", *arguments],
        input=content, cwd=ROOT, env=environment)


def checkpoint_tree():
    require(git("rev-parse", "HEAD").decode().strip() == CHECKPOINT, "wrong checkpoint")
    entries = []
    for entry in git("ls-tree", "-rz", "HEAD").split(b"\0"):
        if not entry:
            continue
        metadata, name = entry.split(b"\t", 1)
        mode, kind, object_id = metadata.split()
        require(kind == b"blob", "non-blob checkpoint entry is outside reconstruction scope")
        entries.append((name.decode(), mode, object_id))
    objects = io.BytesIO(git("cat-file", "--batch", content=b"".join(object_id + b"\n"
        for name, mode, object_id in entries)))
    tree = {}
    for name, mode, object_id in entries:
        header = objects.readline().split()
        require(header[:2] == [object_id, b"blob"], "unexpected Git object")
        tree[name] = objects.read(int(header[2]))
        require(objects.read(1) == b"\n", "bad Git batch framing")
    require(objects.read() == b"", "trailing Git objects")
    for name, mode, object_id in entries:
        if mode == b"120000":
            target = str(PurePosixPath(name).parent / tree[name].decode())
            require(target in tree, "unsupported symlink target")
            tree[name] = tree[target]
    return tree


def apply_virtual_patch(tree, patch, additions_only=False):
    lines = patch.decode().splitlines(keepends=True)
    require(lines[0] == "*** Begin Patch\n" and lines[-1] == "*** End Patch\n", "invalid patch wrapper")
    position = 1
    touched = []
    while position < len(lines) - 1:
        header = lines[position].rstrip("\n")
        require(header.startswith("*** Add File: ") or header.startswith("*** Update File: "),
                "unsupported patch directive")
        addition = header.startswith("*** Add File: ")
        name = header.split(": ", 1)[1]
        require(not Path(name).is_absolute() and ".." not in PurePosixPath(name).parts,
                "unsafe patch path")
        require(name not in touched, "patch touches a file twice")
        touched.append(name)
        position += 1
        block = []
        while not lines[position].startswith("*** "):
            block.append(lines[position])
            position += 1
        if addition:
            require(name not in tree and all(line.startswith("+") for line in block), "invalid addition")
            tree[name] = "".join(line[1:] for line in block).encode()
            continue
        require(not additions_only and name in tree, "unexpected baseline edit in agent delta")
        original = tree[name].decode().splitlines(keepends=True)
        rebuilt, cursor, offset = [], 0, 0
        while offset < len(block):
            require(block[offset].startswith("@@"), "missing hunk marker")
            offset += 1
            hunk = []
            while offset < len(block) and not block[offset].startswith("@@"):
                require(block[offset][0] in " +-", "invalid hunk line")
                hunk.append(block[offset])
                offset += 1
            old = [line[1:] for line in hunk if line[0] in " -"]
            new = [line[1:] for line in hunk if line[0] in " +"]
            location = next((candidate for candidate in range(cursor, len(original) - len(old) + 1)
                if original[candidate:candidate + len(old)] == old), None)
            require(location is not None, "hunk does not match: " + name)
            rebuilt.extend(original[cursor:location])
            rebuilt.extend(new)
            cursor = location + len(old)
        rebuilt.extend(original[cursor:])
        tree[name] = "".join(rebuilt).encode()
    return touched


def write_json(path, value):
    path.write_text(json.dumps(value, sort_keys=True, indent=2) + "\n")


def deliver(parent, destination):
    require(not destination.exists() and destination.is_relative_to(Path("/private/tmp"))
            and not destination.is_relative_to(parent), "delivery must be new private scratch")
    baseline = json.loads((parent / "handoff/file-hashes.json").read_text())
    require(len(baseline) == 1976, "baseline file scope changed")
    require(all((ROOT / name).is_file() and sha256((ROOT / name).read_bytes()) == expected
        for name, expected in baseline.items()), "dirty77 baseline changed")
    names = set(git("ls-files", "-z").decode().split("\0"))
    names.update(git("ls-files", "--others", "--exclude-standard", "-z").decode().split("\0"))
    names.discard("")
    additions = sorted(names - set(baseline))
    require(additions and all(name in TOOLS or name.startswith(PREFIX) for name in additions),
            "changes outside bounded unique scope")
    require(TOOLS <= set(additions), "missing implementation/test file")
    require(set(names) == set(baseline) | set(additions), "unexpected missing files")
    inventory = {}
    patch = ["*** Begin Patch\n"]
    for name in additions:
        content = (ROOT / name).read_bytes()
        text = content.decode()
        require(not text or text.endswith("\n"), "non-newline text cannot be reproduced: " + name)
        inventory[name] = dict(sha256=sha256(content), bytes=len(content), baseline_sha256=None)
        patch.append("*** Add File: " + name + "\n")
        patch.extend("+" + line for line in text.splitlines(keepends=True))
    patch.append("*** End Patch\n")
    encoded = "".join(patch).encode()
    frozen = (parent / "handoff/combined.apply_patch").read_bytes()
    require(sha256(frozen) == BASE_PATCH, "wrong frozen patch")
    tree = checkpoint_tree()
    apply_virtual_patch(tree, frozen)
    require({name: sha256(content) for name, content in tree.items()} == baseline,
            "clean checkpoint plus frozen patch does not reconstruct dirty77")
    before = dict(tree)
    touched = apply_virtual_patch(tree, encoded, additions_only=True)
    require(set(touched) == set(additions), "agent delta scope mismatch")
    require(all(tree[name] == content for name, content in before.items()), "delta modified baseline")
    expected = dict(baseline, **{name: item["sha256"] for name, item in inventory.items()})
    require({name: sha256(content) for name, content in tree.items()} == expected,
            "baseline plus delta differs from implementation bytes")
    destination.mkdir(parents=True, exist_ok=False)
    (destination / "dirty77-relative.apply_patch").write_bytes(encoded)
    (destination / "dirty77-relative.apply_patch.sha256").write_text(sha256(encoded) + "\n")
    reconstruction = dict(success=True, method="Git object bytes plus two in-memory apply_patch replays",
        created_checkout=False, baseline_files=len(baseline), added_files=len(additions),
        reconstructed_files=len(tree), all_file_hashes_match=True, baseline_unchanged=True,
        checkpoint=CHECKPOINT, baseline_patch_sha256=BASE_PATCH, delta_sha256=sha256(encoded))
    write_json(destination / "reconstruction.json", reconstruction)
    manifest = dict(classification="CHECKED CONDITIONAL MODEL-ONLY; independent review pending",
        checkpoint=CHECKPOINT, patch_base="EXACT dirty77 baseline; do NOT apply combined patch again",
        frozen_patch_sha256=BASE_PATCH, delta_sha256=sha256(encoded), changed_files=len(additions),
        files=inventory, reconstruction_sha256=sha256((destination / "reconstruction.json").read_bytes()))
    write_json(destination / "inventory.json", manifest)
    print(json.dumps(reconstruction, sort_keys=True))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--parent", type=Path, default=Path("/private/tmp/verus-cycle77-review-fixes.XCkOUU"))
    parser.add_argument("--delivery", type=Path, required=True)
    arguments = parser.parse_args()
    deliver(arguments.parent.resolve(), arguments.delivery.resolve())
