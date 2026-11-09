# New native conservative-hex single-byte composition

Result: **IMPLEMENTED_AND_VERIFIED, FROZEN_OPERATIONAL91_ONLY; INDEPENDENT_REVIEW_REQUIRED.**
All edits and results are in this fresh scratch directory. The parent checkout,
frozen repair, review, oracle, runtime, toolchain and old evidence remain read-only.
Work started November 7, 2026 at 20:41:37 America/Chicago
(November 8 at 02:41:37 UTC). Exact final accounting is in `handoff.json`.

## New executable composition

`native/src/iter.rs:356`, `native_single_byte_roundtrip(byte: u8, case: Case)`,
has no precondition and ensures its actual returned decoder result is `Ok(byte)`.
Its executable path is:

1. Actual native `Case::table()` chooses the retained LOWER or UPPER constant.
2. Actual native `Table::byte_to_hex_chars(byte)` produces `[Char; 2]`.
3. Those actual ASCII-valued enum characters are cast to their `u8` ASCII codes.
4. Actual private native `hex_chars_to_byte(encoded[0] as u8, encoded[1] as u8)`
   decodes them, calling the retained native `Char::decode_nibble` twice.

This is new encode/decode composition, not another canonical-table-only goal,
replacement implementation, assumed decoder, or sampled native application run.
The checked contract quantifies over every `u8` and both `Case` variants.
The original private pair decoder remains private: the new goal lives in its
existing module, so no visibility weakening or forwarding export is needed.

The sole new proof helper, `encoding_spec::canonical_digit_decodes`, proves the
inverse of independently specified ASCII digit encoding for nibbles below 16,
separately considering Lower and Upper. Existing exact native encoder contracts
and the same-invocation proved native decoder inference contract supply the
composition. Checked bit-vector arithmetic establishes nibble reconstruction.
No new axiom, assumed specification, external body, admit, trust rule, or
executable replacement is introduced.

This bounded path uses the actual native pair decoder, **not**
`decode_to_array::<1>`, `byte_to_str`, or `byte_to_chars`/Rust `char` conversion.
The two characters are the library's actual ASCII-valued `Char` enum values,
not a fabricated string or spec-only sequence. The optional string/array entry
point extension is not claimed. No BufEncoder, ArrayString, MaybeUninit, unsafe
resource expansion, or new allocation/Drop/I/O obligation is introduced.

## Applications and semantic control

Three conservatively charged direct verifier applications, sequentially, with
one worker and `--num-threads 1`; every application has a 60-second hard limit:

- Attempt 01, initial positive: exit 0, 54 verified, zero errors;
  complete selected scope, 38 executable bodies and two proof bodies.
- Attempt 02, semantic negative: actual native encoding of `0xab` has its actual
  two characters reversed before calling the actual native pair decoder. The
  assertion `decoded == Ok(0xab)` fails at `iter.rs:376:12`, exactly one backend
  assertion error, with no VIR/frontend error. No other assertion/contract error
  is reported. The failed full run receives zero positive body/proof credit.
- Attempt 03, corrected/restored positive: `decoded == Ok(0xba)` verifies for
  both cases; exit 0, 54 verified, zero errors, same full selected scope.
  Every native source-file hash is byte-identical to attempt 01 after restoration.

Only that one expected-result literal changes for the negative. Full source
snapshots, exact commands, allowlisted sanitized environments, cwd, source
hashes, timeout, return codes, full stdout/stderr, exported interfaces, parsed
outputs and scopes are retained under each `attempt-*/`. The positive and
negative sources differ exactly as `semantic-negative.patch` records.
`validation-receipts.json` aggregates all three without resetting the ledger.
Verifier wall time sums to 6.890881 seconds; none times out.

Metadata corrections are explicitly retained: the initial result serializer
looked for the scope at the wrong JSON key and recorded `scope: null` in
attempt 01; its untouched raw output contains the full successful scope, which
the aggregate receipt reads correctly. The first metadata audit failed an
unjustified one-diff-block assertion because shared blank lines split a pure
insertion. Its original traceback and command are retained in
`audit-attempt-01.*`; the corrected audit passed in `audit-attempt-02.stdout`.
The shell wrapper then attempted to assign zsh's read-only `status` variable;
that wrapper error did not rerun verification or invalidate the already-written
successful audit. This error is retained in `operational-notes.json`.

## Implementation identity and preservation

`implementation-identity-audit.json` reuses the authenticated independent
identity/span oracle, extending its insertion mapping to `iter.rs` as well as
the inherited `lib.rs`. For **each** successful application it matches a
multiset of 32 original fully qualified Rust implementation identities plus
source-relative file and uniquely mapped source span. Both same-labelled
`DecodeVariableLengthBytesError::from` implementations match separately, with
their original spans. Function-name sets are not the preservation oracle.

Relative to the frozen repair, only `src/iter.rs` and `src/encoding_spec.rs`
change, by insertions. All existing frozen source bytes remain in order;
`lib.rs` and `decoding_spec.rs` are byte-identical to the frozen repair.
There are two new executable goal/control bodies and one new proof body.
All original32 executable bodies/signatures, privacy, derives and constant
initializers are retained. The original LOWER and UPPER initializer strings
and lib derives are separately compared against the authenticated archive;
all other archived files remain byte-identical except inherited lib insertions
and the new iter insertions. Detailed line embeddings and hashes are retained.

`final-source.patch` is the exact NEW delta against the frozen successful repair.
`complete-versus-archive.patch` includes the inherited repair plus this new delta
against the authoritative native source archive. The complete final native
tree is `native/`; do not confuse the incremental patch with a standalone patch
against an unmodified upstream checkout.

## Authentication, trust and retained boundaries

Before execution, before/after every application, and at exit, the controller
authenticates the exact in-place frozen runtime manifest and all nine runtime
artifacts, recorded rustc, 30 loader libraries, eight qualification artifacts,
successful historical build receipts/logs, source/freeze identities, native
archive and native source identities. No runtime/toolchain/cache is copied.
Used frozen native source files match the frozen invocation source manifest.
The nine inspected source bindings are rehashed against their frozen identities.
The repair's complete output manifest and pinned independent-review identities
also authenticate in the final metadata audit. Entry and exit input inventories
are compared for equality, rather than silently accepting changed inputs.

Runtime manifest SHA256:
`1fa38f105d5b9fda9bda2ad3b0ee4518f083f917bc77dcbce6585ff152e476a3`.
Native archive SHA256:
`e7f4b63eced039d437b54858fc600071b41614aa66b2a9fdc27caa94c9e0de5b`.
Runtime: `/private/tmp/verus-operational91-runtime.q91/runtime`.
Original matched sysroot and loader are retained, with explicit sanitized
environment and no inherited secrets. These are authenticated bindings to the
frozen operational91 runtime/standard models, **not** a new compiler build,
fresh full-current-parent/compiler-source correspondence, historical-v3 engine
equivalence, or general native-crate acceptance. Ordinary Verus backend and
matching imported vstd/library specifications remain part of the trust base.
Exact model fingerprints, inferred contracts/certificates, assistance inventory,
and differences from the frozen repair are in `trust-and-inference-inventory.json`.

**The strict persisted owned warm-cache gate remains UNMET.** The helper reports
`cache: miss`, `cache_policy: regenerated_state`; this proof does not close,
waive, replace, or reinterpret that separate gate. No cache hit is claimed.
No current-parent, whole-native-crate, other-target/version, runtime adoption,
final native acceptance, parent integration, or trust adoption is authorized.
All 18 redb cells remain OPEN; S0, ownership/Drop/panic/glue/allocator/I/O/history
and ordinary Infer/full-scope/import/export/source-continuity barriers remain.

No builds, native application runs, network/downloads, Git writes, parent edits,
old-evidence deletions, subdelegation, or excluded growth049/zero-growth050/
mutation030/refused-padding95/drop-pipeline work occurred. Scratch remains below
128 MiB and checked global free disk above 16 GiB. Only three of 16 applications
were spent. The final source and artifacts require **independent review before
any adoption**; this implementation agent does not perform that independent
review or mark it complete. Stop at this sealed handoff.
