import datetime
import hashlib
import json
from pathlib import Path
import shutil
import subprocess


ROOT = Path(__file__).resolve().parent
REPOSITORY = ROOT.parents[1]
BASELINE = "45f93e51f4d0c0f1b71ef645932d54a7862ec983"
UPSTREAM = Path.home() / ".cargo/registry/src/index.crates.io-1949cf8c6b5b557f/unsigned-varint-0.8.0"


def git(*arguments):
    return subprocess.check_output(["git", *arguments], cwd=REPOSITORY, text=True).strip()


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    selection = ROOT / "selection.json"
    if selection.exists():
        raise SystemExit("Selection already frozen")
    assert git("rev-parse", "HEAD") == BASELINE
    assert git("diff", BASELINE, "--", "source", "dependencies", "tools", "rust-toolchain.toml") == ""
    snapshot = ROOT / "upstream"
    for relative in ("src/encode.rs", "src/decode.rs", "src/lib.rs", "Cargo.toml", "Cargo.toml.orig", "LICENSE", ".cargo_vcs_info.json"):
        destination = snapshot / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(UPSTREAM / relative, destination)
    archive = Path.home() / ".cargo/registry/cache/index.crates.io-1949cf8c6b5b557f/unsigned-varint-0.8.0.crate"
    shutil.copyfile(archive, snapshot / archive.name)
    source = (snapshot / "src/encode.rs").read_text()
    macro = source[source.index("macro_rules! encode {"):source.index("/// Encode the given `u8`")].rstrip()
    function = source[source.index("pub fn u64("):source.index("/// Encode the given `u128`")].rstrip()
    constant = "const U64_LEN: usize = 10;"
    assert constant in source
    selected = macro + "\n\n" + function + "\n\n" + constant + "\n"
    (ROOT / "original-selected.rs").write_text(selected)
    record = {
        "schema": 1,
        "frozen_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "before_first_proof": True,
        "engine_baseline": BASELINE,
        "head": git("rev-parse", "HEAD"),
        "head_tree": git("rev-parse", "HEAD^{tree}"),
        "baseline_tree": git("rev-parse", BASELINE + "^{tree}"),
        "branch": git("branch", "--show-current"),
        "initial_clean_checkout_observed": True,
        "engine_diff_at_selection": git("diff", BASELINE, "--", "source", "dependencies", "tools", "rust-toolchain.toml"),
        "provenance_issue": "None in provided source checkout. Parent release artifacts have uncommitted-work provenance and will not be used. Build an independent engine in this worktree. Shared installed rustc and copied standalone Z3 must be fingerprinted separately.",
        "candidate_assessment": "Only unsigned-varint bodies inspected before selection; cache filename inventory also listed base64 and httparse, but neither was tried. Pure no_std encoder, no allocation or I/O, compact macro plus u64 wrapper. Finite 10-byte array and mutable iterator, break, shifts, inclusive subslice and returned borrow may exceed documented inference/frontend support. Selection is not contingent on a proof passing.",
        "upstream": {
            "crate": "unsigned-varint", "version": "0.8.0",
            "repository": "https://github.com/paritytech/unsigned-varint",
            "revision_from_cargo_vcs_info": "10d55e5dbdd7f61b6495bd12a49d06df82e08612",
            "license": "MIT", "source_cache": str(UPSTREAM),
            "selected_items": ["src/encode.rs: encode! macro", "src/encode.rs: pub fn u64", "src/encode.rs: U64_LEN"],
            "archive_sha256": digest(snapshot / archive.name),
            "snapshot_sha256": {str(path.relative_to(snapshot)): digest(path) for path in sorted(snapshot.rglob("*")) if path.is_file()},
            "original_selected_sha256": digest(ROOT / "original-selected.rs"),
        },
        "required_domain": "Every Rust u64 number (0..=18446744073709551615), every initialized mutable [u8; 10] with arbitrary previous byte contents, under Rust's exclusive-borrow rules. No numerical bound, ASCII restriction, zero-buffer restriction, or other precondition may be added. Target: aarch64-apple-darwin, 64-bit usize.",
        "required_property": "The returned prefix is the canonical unsigned base-128 varint for number: length 1..=10; byte j has payload floor(number / 128^j) mod 128; bit 7 is set exactly on nonfinal bytes; number < 128^length and, for length > 1, number >= 128^(length-1). Required for the entire domain, without panics or failed debug assertions. Preservation of untouched buffer suffix will also be tested natively, but is not part of the frozen proof goal.",
        "allowed_adaptations": "Verus import/macro enclosure, explicit returned lifetime if required, Verus named-result syntax and authored postconditions/spec radix definition. Retain original encode! and selected u64 executable body byte-for-byte. No mutable-iterator replacement, indexing rewrite, body externalization, trusted specifications, engine/macro/vstd edits, extra preconditions or candidate shopping.",
        "protocol": "Run native tests and independent baseline build; ordinary vs --infer-goals in separate cold then warm processes, preserving raw stdout JSON/stderr/status/timing. Also try #[verus_infer] on the identical macro-using body to expose exact-inference support, without claiming its result as application policy. A checked positive proof requires semantic mutation rejection. If frontend rejects the unchanged body, stop rather than adapt away the evidenced blocker. Warm means retained candidate cache and OS state, never cached proofs; an early frontend error may produce no candidates or proofs.",
        "ai_assistance": "OpenAI Codex:gpt-6.1-sol, authorized replacement for rate-limited Claude Opus 5.5; no other agents.",
    }
    selection.write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps({"selection": str(selection), "sha256": digest(selection)}, indent=2))


if __name__ == "__main__":
    main()
