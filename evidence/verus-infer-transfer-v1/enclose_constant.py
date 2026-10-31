import datetime
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def main():
    changes = []
    for name in ("ordinary", "inferred"):
        original = ROOT / "fixtures" / (name + ".rs")
        corrected = ROOT / "fixtures" / ("constant-enclosed-" + name + ".rs")
        assert not corrected.exists()
        before = original.read_text()
        constant = "const U64_LEN: usize = 10;"
        enclosure = "verus! {\n" + constant + "\n}"
        assert before.count(constant) == 1
        after = before.replace(constant, enclosure)
        assert after.replace(enclosure, constant) == before
        corrected.write_text(after)
        changes.append({"before": str(original.relative_to(ROOT)),
                        "after": str(corrected.relative_to(ROOT)),
                        "before_sha256": hashlib.sha256(before.encode()).hexdigest(),
                        "after_sha256": hashlib.sha256(after.encode()).hexdigest()})
    receipt = {
        "recorded_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "reason": "Initial trials had both an unsupported native macro-generated for loop and an accidentally unannotated constant. Enclose only the unchanged constant in verus! so the independent original-loop blocker is isolated. Retain every initial fixture and log. Selection, original macro, u64 body, property and full input domain remain unchanged.",
        "first_proofs_already_attempted": True,
        "selection_sha256": hashlib.sha256((ROOT / "selection.json").read_bytes()).hexdigest(),
        "changes": changes,
    }
    (ROOT / "constant-enclosure-adapter.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
