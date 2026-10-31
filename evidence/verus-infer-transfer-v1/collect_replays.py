import json
from pathlib import Path
import shutil


ROOT = Path(__file__).resolve().parent


def main():
    for label, temporary in (
        ("replay-failed", "/private/tmp/verus-infer-transfer-v1-replay"),
        ("replay-corrected", "/private/tmp/verus-infer-transfer-v1-replay-corrected"),
    ):
        source = Path(temporary)
        destination = ROOT / label
        destination.mkdir(exist_ok=False)
        shutil.copytree(source / "raw", destination / "raw")
        shutil.copyfile(source / "replay.json", destination / "replay.json")
    observations = {
        "origin": "Reconstructed from the interactive execution transcript; these are setup observations, not compiler/proof receipts. Timings were not independently captured.",
        "events": [
            {"argv": ["python3", "evidence/verus-infer-transfer-v1/prepare.py"],
             "exit_status": 1, "elapsed_seconds": None,
             "diagnostic": "FileNotFoundError for cached unsigned-varint-0.8.0/.cargo-checksum.json. Removed that nonexistent optional snapshot item before the first selection record and before any proofs; cached registry index and archive were later checked instead."},
            {"argv": ["cp", "/Users/tommyvolk/Documents/GitHub/verus/source/z3", "source/z3"],
             "exit_status": None, "elapsed_seconds": None,
             "diagnostic": "Exit status not separately captured from the following build command. Subsequent artifact receipt confirms a standalone copy, independent inode and identical SHA-256; no parent release artifacts were copied."},
            {"argv": ["python3", "evidence/verus-infer-transfer-v1/replay.py", "--output", "/private/tmp/verus-infer-transfer-v1-replay"],
             "exit_status": 1, "elapsed_seconds": None,
             "diagnostic": "Initial replay double-relocated /private/tmp in new paths, causing six native command failures and file-not-found frontend failures, not semantic application failures. Two native test launch exceptions have no automatic receipt because run.py initially caught timeouts only. Complete available logs and summary retained in replay-failed/. Path ordering and launch-error logging corrected; all 16 subsequent replay command checks passed."},
        ],
    }
    (ROOT / "setup-events.json").write_text(json.dumps(observations, indent=2) + "\n")
    print("Retained both replay transcripts and setup observations")


if __name__ == "__main__":
    main()
