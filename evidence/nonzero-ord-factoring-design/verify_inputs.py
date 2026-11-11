"""Read-only review integrity audit; no compiler, proof, test or native checking.

Only own source/Git and explicitly named sealed files are read. Do not import
or execute the historical run/audit/finish/authentication controllers.
"""
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
BASE = "778c5e6a2a048e4a776be65ea354f73da4de10ba"
DATA = ROOT / "source/tools/research/nonzero_ordering_goal"
PRIVATE = {
    "/tmp/nonzero-ordering-baseline-qoptrgkf",
    "/tmp/nonzero-ordering-mir-x1sbsnfo",
}
EXTERNAL = {
    "/tmp/native-miri-policy-pin-20261110/sources/src/tools/miri/src/borrow_tracker/mod.rs",
    "/tmp/verus-infer-buffer-cycle-20261109/nonzero-ordering-parent-auth/receipt.json",
    "/tmp/verus-infer-buffer-cycle-20261109/auth-nonzero-ordering.py",
}


def need(ok, message):
    if not ok:
        raise RuntimeError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def sha(path):
    return digest(path.read_bytes())


def relative(name):
    path = Path(name)
    need(not path.is_absolute() and ".." not in path.parts, "nonlocal path")
    return path


def audit():
    inputs = json.loads((HERE / "INPUTS.json").read_text())
    need(inputs["base"] == BASE, "wrong review base")
    for name, expected in inputs["own_files"].items():
        path = relative(name)
        data = (ROOT / path).read_bytes()
        need(digest(data) == expected, "own bytes changed: " + name)
        original = subprocess.check_output(["git", "show", BASE + ":" + name], cwd=ROOT)
        need(data == original, "not prepared-base source: " + name)
    need(set(inputs["external_files"]) == EXTERNAL, "unexpected external input")
    for name, expected in inputs["external_files"].items():
        need(sha(Path(name)) == expected, "named external bytes changed: " + name)
    for name in inputs["inherited_git_rebindings"]:
        path = relative(name)
        original = subprocess.check_output(["git", "show", "5d79f711:" + name], cwd=ROOT)
        need((ROOT / path).read_bytes() == original, "inherited bytes differ: " + name)

    seal = json.loads((DATA / "SEAL.json").read_text())
    need(set(seal["roots"]) == PRIVATE, "unexpected sealed root")
    count = 0
    for root, files in seal["roots"].items():
        for name, expected in files.items():
            need(sha(Path(root) / relative(name)) == expected, "sealed bytes changed: " + name)
            count += 1
    for name, expected in seal["controller_inputs"].items():
        need(sha(ROOT / relative(name)) == expected, "controller changed: " + name)
    need(count == 168 and len(seal["controller_inputs"]) == 4, "seal cardinality")

    baseline = json.loads((DATA / "BASELINE.json").read_text())
    mir = json.loads((DATA / "MIR.json").read_text())
    need(len(baseline["cases"]) == 11, "baseline cardinality")
    need(len(mir["inventory"]) == 7, "MIR cardinality")
    need(mir["foreign_certificates_produced"] == 0 and not mir["ordering_accepted"], "scope drift")
    raw = (Path(mir["private_root"]) / "inspect/stdout").read_text()
    sections = raw.split("BEGIN_INSTANCE ")[1:]
    need(len(sections) == 7, "whole body sections")
    by_instance = {section.splitlines()[0]: section for section in sections}
    for row in mir["inventory"]:
        section = by_instance[row["instance"]]
        need(digest(section.encode()) == row["whole_mir_sha256"], "MIR section hash")
        need(len(section.encode()) == row["whole_mir_bytes"], "MIR section bytes")
        need(not row["unavailable"], "unavailable MIR")
    need(mir["inventory"] == inputs["MIR_diagnostic_inventory"], "recorded inventory differs")
    declaration = "#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]\npub struct BorTag(NonZero<u64>);"
    pinned = Path(next(name for name in EXTERNAL if name.endswith("borrow_tracker/mod.rs")))
    need(pinned.read_text().count(declaration) == 1, "original declaration")
    need(digest(declaration.encode()) == baseline["declaration_sha256"], "declaration hash")
    for name, expected in inputs["review_files"].items():
        need(sha(HERE / relative(name)) == expected, "review bytes changed: " + name)
    return {
        "success": True,
        "scope": "read-only source/dataset/review byte integrity; no semantic or native checks",
        "base": BASE,
        "sealed_artifacts": count,
        "controller_inputs": 4,
        "baseline_cases": 11,
        "MIR_instances": 7,
        "new_proofs": 0,
        "new_foreign_certificates": 0,
        "ordering_accepted": False,
        "native_correspondences": 0,
        "publication": False,
        "goal_complete": False,
        "inputs_sha256": sha(HERE / "INPUTS.json"),
    }


if __name__ == "__main__":
    print(json.dumps(audit(), sort_keys=True))
