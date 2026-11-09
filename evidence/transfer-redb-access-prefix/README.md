# redb readonly access range/length prefix

This milestone verifies a faithfully extracted three-field readonly AccessConfig
projection, NOT original whole TransactionalMemory/get_page or storage IO.
The unchanged .into() and .try_into().unwrap() syntax composes with the original
PageNumber helpers using closed canonical primitive numeric models.

Portable no-cheating proof and explicit 32/64-bit supplementary model checks
establish len == (1u64 << order) * size and len == range.end - range.start over
the full checked helper domain plus representability in usize; no order<=20
shortcut. Explicit architecture declarations are configuration-only supplemental
checks without --no-cheating, not 32-bit native execution. The portable proof
keeps --no-cheating and native compilation. Original debug/panic/overflow/shift
and all mandatory inference exactness/publication checks remain in force.

Final evidence: 10 positive, 4 semantic, 21 safety, 4 metadata controls; all 29
seeded inference-aware negative exports removed, 267 repeated required checks.
Native 64-bit sampled oracle checks 7,337 safe calls (2,278 lengths outside the
32-bit domain). Matching vstd 2221/0, VIR139, focused Rust35, Python26 per normal/-O
mode, reviewed relational replay and four existing frontdoors/61 main proofs pass.
No broad suite rerun or whole-application result is claimed.

Runtime/source identity: cache73/policy67/schema38. Parent72/66 and integration74/68
are reserved separately. The NEW replay-checked delta is relative to the reviewed
ten-file reconstruction on parent3b05c191, not clean HEAD; parent import/core-test
fixes are preserved. Existing candidate70 and review evidence remain untouched.

See manifest.json and its immutable scratch REPORT/HANDOFF for exact commands,
hashes, architecture/trust qualifications, diagnostics and replay instructions.
Stop at the following storage.read statement; no speculative state model exists.
