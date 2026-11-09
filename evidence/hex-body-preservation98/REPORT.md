# Separate preservation audit: hex-conservative 1.3.0

Date: November 7, 2026, America/Chicago. Classification: **AUDIT/METADATA**.

## Bounded result

**PASS for this separately authored preservation oracle only.** Each frozen
positive 03, 05, and 06 independently retains all **32 original executable
scope rows** (31 function labels), against 36 current executable rows (35
labels). There are **96 individual matches**, not a name-set or count-only
inclusion result. Both same-labelled From implementations match individually
in every positive. The **23 AUDIT/METADATA unit tests pass**.

This is sensitive proof-evidence hardening, **not a new native proof, final
acceptance, parent integration, or permission to retain/adopt the repair**.
The strict persisted warm-cache gate remains **UNMET**. **All 18 redb tasks
remain OPEN; trust/admission are UNCHANGED.** The parent alone decides
retention and any separately authorized later product integration.

All writes are confined to this new scratch:
`/private/tmp/verus-hex130-preservation-audit-20261107-4h70icof`.
No parent or frozen-input file is edited. No subdelegation, supplied-harness
execution/import, build, Cargo/Rust invocation, verifier/solver/native
execution, network operation, or Git write occurs. Only newly authored
stdlib Python audit/tests/finalization code runs. No runtime/toolchain
reauthentication or evidence-tree copy is performed.

## Independent inputs and authentication

Frozen implementation root:
`/private/tmp/verus-hex130-privacy-repair-20261107-zm5hKz`.
At initial entry, audit entry/exit, and final handoff, every one of its **143
listed files** is checked against its manifest byte length and SHA256.
The stronger audit receipts also enumerate the complete path set: **144
regular files including the manifest**, zero unlisted files, zero symlinks
or special files. Entry/exit records and external pins must agree.

Pinned SHA256 values (hex, not truncated):

| Input | SHA256 |
| --- | --- |
| Frozen `output-manifest.json` | `3ef7d0e40cf0150e40ee071bfd1629e8f440f5246b42633708ca80206f5c8d62` |
| Independent `REVIEW.md` | `72e536659fb1cf25bb974a088885d29dec14d4eea8d362130e2cb4b244f6b6f8` |
| Independent `independent-evidence.json` | `eb253b6b0d15dd6ca3341b71436a88ef86df36bacc30fa831996b6505bb86ea3` |
| Parent `evidence/verus-infer-hex-v3/native-source.tar.gz` | `e7f4b63eced039d437b54858fc600071b41614aa66b2a9fdc27caa94c9e0de5b` |
| Frozen `historical-v3-baseline.json` | `a8fefc5ea34adebde900cdeb9565c80db14947d9afa6cc8eda695a3a63d0e324` |
| Frozen `invocation-03/stdout` | `a58d90014a77109267c7797a467b93fdf5d3faeb4a449325a40e69fe79420482` |
| Frozen `invocation-05/stdout` | `bb40964dbaf517c3173331e94ccfa8129df964bbabeb1b3a96c6a1bc769845e2` |
| Frozen `invocation-06/stdout` | `55a5c9f25976ae25e5d19d7ac328c34ab3a906a88d32f75cee2fb1f676f7eb12` |

The review documents are under
`/private/tmp/verus-hex130-independent-review-20261107-7xnzpY`.
Both are read in full after pin authentication. Their previously derived
matches are **not consumed as an oracle**: this implementation obtains its
own matches directly from the historical JSON and raw positive stdout JSON,
and constructs its source correspondence directly from archive bytes and
final native-tree bytes. Every implementation-input hash/size is retained
in the authentication receipts; external input hashes/sizes are also listed.

## Identity and source correspondence

The multiset key is **exact `rust_function` + source-relative filename +
normalized four-coordinate source span**. Function labels are diagnostic
only, never an identity key. Original cardinality is asserted to be 32,
current cardinality 36; declared and actual exec/proof counts must agree
(historical proof rows 0, current proof rows 1). The 31/35 label counts are
additional diagnostics, not substitutes for matching. Exact-identity and
same-implementation duplicates fail closed, even if total counts and label
sets survive. Required identity fields and selection flags are checked.
Duplicate JSON object keys are rejected before matching.

Root handling is explicit and strict:

- Historical root: `/Users/tommyvolk/Documents/GitHub/hex-conservative`.
- Current root: `/private/tmp/verus-hex130-privacy-repair-20261107-zm5hKz/native`.
- Only an exact known-root prefix followed by `/` is removed. Other roots,
  sibling-prefix roots, traversal, noncanonical paths, unknown files,
  malformed/empty/reversed spans, and out-of-bounds coordinates are rejected.

There are 53 archived regular files and 54 final native-tree files. The
**52 other archived files are byte-identical**, not merely name-matched;
the sole modified archived file is `src/lib.rs`, and the sole new file is
`src/encoding_spec.rs`. This includes byte-identical `src/error.rs`.

Original lib.rs has 538 lines; final lib.rs has 576: **38 inserted lines**.
Exact byte-lines, including line endings, must form an insertion-only
subsequence. Deletions/replacements fail. A forward greedy embedding gives
each original line's earliest possible final line; a reverse embedding gives
its latest. Both must agree on every line belonging to a matched historical
span. This avoids accepting an arbitrary diff alignment when repeated lines
make a span ambiguous. Insertions inside a multi-line identity span fail;
columns are retained exactly, with byte-based source bounds checked.
Unreferenced repeated/blank-line ambiguity is reported in `line-mapping.json`
but cannot be used to normalize an identity span.

Historical spans are forwarded into final-source coordinates, then compared
exactly to current spans. Existing lib.rs identities map as follows in all
three positives:

- `<Char as core::clone::Clone>::clone`: `238:16:238:21` -> `245:16:245:21`.
- `Char::decode_nibble`: `293:5:293:53` -> `300:5:300:53`.
- `decode_to_vec`: `135:1:135:83` -> `137:1:137:83`.

The four additional exec implementations are explicitly inventoried, not
credited toward the original 32: `<Case as core::clone::Clone>::clone`,
`Case::table`, `encoding_spec::canonical_byte_encoding_goal`, and
`table::Table::byte_to_hex_chars`. Each positive's recorded full 54-file
source-hash map is independently checked against final bytes, its lib.rs and
encoding_spec.rs snapshots match, and its result binds its exact stdout hash.
This checks recorded evidence only; no command recorded in those files runs.

Key final source SHA256 values:

- `src/lib.rs`: `41d8e48a89103fd0fb018e81a5c2af13e75c8d0eb96eba677e33f40742869715`.
- `src/encoding_spec.rs`: `c9369b01b2fdcc238dc3120670ac8c13ce7b245193fe0158a17a1d6d5a980c45`.
- `src/error.rs`: `81f4849fff2da8f6c5e11260f048f949917ac94e6d8a1f9b6959a236994d6f44`.

## Individually preserved overloads

Shared diagnostic label:
`hex_conservative::error::DecodeVariableLengthBytesError::from`.

Exact Rust identities:

1. `<error::DecodeVariableLengthBytesError as core::convert::From<error::InvalidCharError>>::from`.
2. `<error::DecodeVariableLengthBytesError as core::convert::From<error::OddLengthStringError>>::from`.

| Positive | InvalidCharError implementation | OddLengthStringError implementation |
| --- | --- | --- |
| 03 | `src/error.rs:150:5:150:41`, multiplicity 1 -> 1, MATCH | `src/error.rs:157:5:157:45`, multiplicity 1 -> 1, MATCH |
| 05 | `src/error.rs:150:5:150:41`, multiplicity 1 -> 1, MATCH | `src/error.rs:157:5:157:45`, multiplicity 1 -> 1, MATCH |
| 06 | `src/error.rs:150:5:150:41`, multiplicity 1 -> 1, MATCH | `src/error.rs:157:5:157:45`, multiplicity 1 -> 1, MATCH |

`matches.json` persists all 32 matches per positive, original/current row
indices, exact implementation strings, both spans, raw source fields, and
individual multiplicities. It also repeats the two overload records in an
explicit per-positive `both_overloads` list.

## Runnable validation and negative classifications

Run from this scratch directory. Interpreter used:
`/Library/Developer/CommandLineTools/usr/bin/python3`.

```sh
python3 -B audit.py
python3 -B -m unittest -v test_audit
python3 -B validate.py
```

All three commands exit **0**. `validate.py` repeats the first two as bounded
subprocesses of the same interpreter, persists actual argv/cwd/UTC/exit codes
and elapsed times in `commands.json`, and writes structured test results.
`audit-run.log` and `unit-tests.log` are raw combined stdout/stderr. The
canonical unit run reports **Ran 23 tests; OK**. The audit and tests each
complete well below one second; there is no proof runtime invocation.

Every synthetic test is **AUDIT/METADATA**, never a native semantic negative.
Tests reject: removal of either overload while duplicating its sibling with
unchanged total count and labels; changed Rust implementation under the same
function label; duplicate historical/current identities and implementation
multiplicity; swapped overload spans; altered span/file/current root/historical
root; path traversal; missing required fields; wrong declared/actual scope
counts; extra rows; lib.rs mapping failure; any other archived-file change;
unexpected source files; ambiguous insertion embeddings; insertion inside an
identity span; out-of-bounds spans; and duplicate JSON keys. Positive tests
cover all original rows and both overloads, reordered multiset rows, a
synthetic insertion mapping, and real archived-to-final lib.rs mappings.

## Handoff artifacts, bounds, and limitations

Runnable source: `audit.py`, `test_audit.py`, `validate.py`, `finalize.py`.
Structured evidence: `matches.json`, `line-mapping.json`,
`source-comparison.json`, `results.json`, `test-results.json`, `commands.json`.
Authentication: `authentication-entry.json` (initial entry),
`audit-authentication-entry.json`, `audit-authentication-exit.json`, and
`authentication-handoff-final.json`. `handoff.json` records final timing,
resource bounds, report hash/size, and finalization command/exit status.
`output-manifest.json` hashes/sizes every other scratch artifact; it excludes
only itself to avoid a self-reference. No copied source trees or runtimes.

The complete task starts at `2026-11-07T23:56:46Z`; the finalization receipt
enforces **<=720 seconds** through handoff, **>16 GiB free**, scratch **<32 MiB**,
and report **<=12 KiB**. The receipt gives actual observations; validation
observed about 52 GiB free and less than 1 MiB of scratch artifacts.

**Limits:** metadata identities and insertion-only source correspondence do
not establish executable-body equivalence, semantic preservation of inserted
code, proof replay, fresh runtime qualification, runtime acceptance, whole
native-crate coverage, or any new product/trust/admission acceptance. The
audit relies on the authenticated historical/current reporting metadata and
the supplied selected-scope classifications; it does not independently parse
Rust, verify compiler reporting, or turn source hashes into proof credit.
This audit deliberately does not repair the frozen harness in place. The
strict persisted warm-cache gate remains **UNMET**, **all 18 redb OPEN**, and
**trust/admission unchanged**. Stop at handoff; parent decides retention and
any later integration.
