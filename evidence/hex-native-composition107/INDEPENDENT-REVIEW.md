# Independent composition107 findings and bounded direct rechecks

## Bounded verdict

**NO NEW THEOREM-SPECIFIC BLOCKING FINDING FOUND; THREE DIRECT RECHECKS CONFIRM
THE STATED SINGLE-BYTE COMPOSITION, FROZEN OPERATIONAL91 ONLY.** This is an
independent engineering/proof-evidence review using the same authenticated
frozen backend, not a new proof authority, compiler qualification, mathematical
certificate, or final native acceptance. No trust/runtime adoption or parent
semantic integration is authorized. The strict persisted owned warm-cache gate
remains **UNMET**, compiler/current-parent-source correspondence remains
**UNESTABLISHED**, and all **18 redb cells remain OPEN**.

The review began November 7, 2026, 20:51:50 America/Chicago (November 8,
02:51:50 UTC). The hard launch cutoff is 03:00:50 UTC and total deadline is
03:03:50 UTC. Exact completion, resource observations, and hashes are in the
sealed handoff. No fourth verifier attempt is needed or launched.

## Findings and objections

1. **Existing acceptance blocker, not a new theorem defect: persisted owned
   warm-cache gate remains unmet.** All four reviewed successful raw runs
   (candidate 01/03 and fresh review 01/03) report the sole inherited helper as
   `cache: miss`, `cache_policy: regenerated_state`. Fresh review cache storage
   is empty. These are real rechecks, not persisted warm-candidate reuse. They
   neither close nor reinterpret the predecessor review's cache objection.
2. **Qualification boundary, not repaired here:** in-place artifact identity
   authenticates operational91 and its recorded models/loader, not correspondence
   of that compiler to current parent sources or historical-v3 engine equivalence.
   No build, current-source qualification, runtime adoption, or new trust claim
   is made. The ordinary backend and imported vstd/library models remain trusted.
3. **Scope caveat:** 54 successful verifier obligations are not 54 executable
   bodies, nor whole native-crate coverage. The successful scope explicitly says
   `selected local Verus items; not all Rust code`: 38 executable bodies plus
   two proof bodies, with 155 rows outside selection, 42 specification rows,
   and three selected constant proxy rows without translated body in this scope
   inventory. The broad `is-verifying-entire-crate: true` JSON flag does not
   override that coverage limitation. Constant proxy rows alone are not body
   receipts. The actual LOWER, UPPER and decoder TABLE names also occur in raw
   backend function details, but those details alone are not individual proof
   certificates. The claim here rests on retained source and successful complete
   selected invocations, not an invented one-obligation-per-body accounting.

There is no additional objection to retaining this narrow new theorem as
frozen-runtime-only evidence, subject to these unchanged boundaries. None of
these observations authorizes final acceptance or removes predecessor findings.

## The actual theorem and helper

I reviewed `native/src/iter.rs:356`, not merely the earlier canonical encoding
goal. `native_single_byte_roundtrip(byte: u8, case: Case)` has no precondition
and ensures its actual return value is `Ok(byte)`. Both enum variants and every
legal byte are symbolic inputs. Its executable expression is exactly
`case.table().byte_to_hex_chars(byte)`, followed by the actual enum-to-u8 casts
and actual private `hex_chars_to_byte(encoded[0] as u8, encoded[1] as u8)` call.
That retained decoder calls `Char::decode_nibble` twice, uses its original
`ok_or`/early-return operations, and reconstructs with `(hih << 4) | loh`.

`native/src/encoding_spec.rs:50` adds the sole new proof helper,
`canonical_digit_decodes`. Its domain is `nibble: u8` with `nibble < 16`, not a
restriction on the new theorem. The call sites discharge it for `byte / 16`
and `byte % 16`; retained `encoding_index_facts` proves both nibble bounds and
the shift/mask equivalences. The new bit-vector assertion proves
`((byte / 16) << 4) | (byte % 16) == byte` across the entire byte domain.

The helper relates two independent arithmetic specifications: decimal ASCII
48+n and alphabetic ASCII 87+n or 55+n versus decoding's independent range
tests. Neither specification calls the new theorem or native encoder/decoder.
The helper's two Case branches are ordinary checked proof bodies, not axioms.
Since 0 <= nibble < 16, specified ASCII values lie in 48..57, 65..70 or 97..102;
the integer-to-u8 cast is exact. Actual Char discriminants are ASCII-valued
`repr(u8)` enum variants; the actual output casts are not fabricated strings,
transmutes, unsafe conversions, or lossy unchecked truncations.

The retained Table encoder remains valid for its general Table domain: its
unconditional selection equations use the actual private view, while canonical
ASCII conclusions are implications for matching canonical tables. There is no
new requires clause on that native method, nor an assumption that every possible
Table equals LOWER or UPPER. `Case::table` returns the actual retained constants.

The inferred decoder contract is generated and proved in each same invocation:
soundness, maximality and exactness are required and all `proved`; the helper is
available to same-crate callers and `exported: false`. Its apparent requires
condition reduces to `0 <= 4 && 4 < 8` when both digits are valid, otherwise
true. It introduces no hidden ASCII/canonical-only entry restriction. The full
contract and dependency list are byte-for-byte equal as JSON values to the
authenticated predecessor's. No unchecked helper summary or circular assumed
inverse is introduced. The ordinary frozen inference/backend remains the trust
boundary; generated report fields are not promoted into a new proof authority.

The decoder remains plain private `fn` in its original module. Table's tuple
field and the closed View definition retain their prior privacy. New goal and
control functions are private, gated by `verus_keep_ghost`, and do not add a
production API. This proves the verifier-side native composition, not deployment
or execution of an application. It does **not** cover `decode_to_array::<1>`,
`byte_to_str`, Rust-char conversion, unsafe buffers, allocation, Drop or I/O.

## Independent preservation and trust audits

`independent-audit.json` redoes the identity/span multiset comparison using the
authenticated independent oracle's byte-line embedding checks, not candidate
generated matches or function-name sets. Separate insertion maps cover lib.rs
and iter.rs. Each candidate positive and each fresh positive matches all 32
historical fully qualified implementation/file/normalized-span identities,
each with multiplicity one. The distinct From overloads remain independently
present at original error.rs spans `150:5..150:41` and `157:5..157:45`:

- `<error::DecodeVariableLengthBytesError as core::convert::From<error::InvalidCharError>>::from`
- `<error::DecodeVariableLengthBytesError as core::convert::From<error::OddLengthStringError>>::from`

The six executable additions relative to that historical32 scope are Case clone,
Case::table, canonical_byte_encoding_goal, Table::byte_to_hex_chars, the new
roundtrip and the new swap control. The two current checked proof bodies are
encoding_index_facts and canonical_digit_decodes. Relative to the predecessor,
only two executable bodies and one proof body are new. Counts are explicitly
separate from the 54 obligation total.

`independent-source-trust-audit.json` independently regenerates both supplied
patches exactly. Relative to the predecessor, only encoding_spec.rs and iter.rs
change, by insertions; lib.rs and decoding_spec.rs are byte-identical. Against
the 53-file historical archive the tree has exactly the inherited extra
encoding_spec.rs, with only lib.rs and iter.rs differing among original files.
LOWER/UPPER initializer text and lib derives match the archive exactly. No new
assume, admit, external_body, assumed specification, axiom, unsafe construct,
trusted annotation or executable replacement occurs in the new insertions.

All 382 imported model entries/fingerprints, inference dependencies, arithmetic
policy and concrete assistance inventories match the frozen predecessor in
all four successful runs. Assistance-explanation diagnostics differ by source
locations/hygiene and the three new functions; ignoring source-location fields,
there are exactly six extra skipped-profile decisions (two existing profiles
per new function), no removed decisions and no new applied assistance. The
preexisting diagnostic profile names are not excluded-project work or new trust.

## Historical attempts and documentary failures

All three historical raw attempts were parsed and checked independently against
their aggregate receipts, invocation records and source snapshots:

- Historical 01: exit 0, 54 verified, 0 errors; 38 executable + 2 proof bodies.
  Its original result serializer has `scope: null`; the untouched raw stdout
  has full scope at `verus-infer.verification_scope`. The aggregate uses that
  raw location correctly. Correcting metadata is **not** another verifier replay.
- Historical 02: exit 1, 53 successful obligations and exactly one backend
  assertion failure at iter.rs:376:12 on `decoded == Ok(0xabu8)`. No VIR error,
  no interface export and zero positive body/proof credit for the failed full
  application. The abort summary is not a second assertion failure.
- Historical 03: exit 0, 54 verified, 0 errors; restored corrected `0xba`
  expectation, full scope; every recorded source hash and all eight source
  snapshot files match historical 01 byte-for-byte.

The metadata audit's first failed `assert len(blocks) == 1` traceback is
retained and explained by shared blank lines splitting pure insertions. The
corrected second metadata audit returned 0 and wrote its successful result.
Its zsh wrapper subsequently failed on assignment to reserved read-only
`status`; the retained note preserves `zsh:18: read-only variable: status`.
These are documentary/audit/wrapper events, not new verifier applications and
not proof credits. I did not execute or repair frozen candidate controllers.

## Three fresh direct rechecks

All used the authenticated in-place operational91 executable, exact recorded
sysroot/loader/Z3 and model artifacts, a sanitized allowlisted environment,
`--infer-goals --no-cheating --output-json --num-threads 1`, and one sequential
worker. No runtime, toolchain, model or cache was copied. Only eight small native
Rust source files were copied into this review scratch; source snapshots and
new proof outputs are review evidence, not compiler/runtime builds.

| Fresh attempt | Backend result | Wall seconds | Export |
| --- | --- | ---: | --- |
| 01 full positive | exit 0; 54 verified; 0 errors | 2.402781666 | yes, private helper not exported |
| 02 swapped negative, wrong expected 0xab | exit 1; 53 verified; exactly 1 assertion error | 2.746213791 | no |
| 03 byte-identical restored, correct expected 0xba | exit 0; 54 verified; 0 errors | 3.216451583 | yes, private helper not exported |

Fresh verifier wall time totals **8.365447040 seconds**. Every launch is charged
before execution; three of four available review attempts were used, no reset
and no fourth launch. Each had a 60-second cap, implemented by waiting at most
59.5 seconds before process-group kill; none timed out. No concurrent review
worker or subdelegate was started; only the freed hex slot was used, without
claiming exclusive use of all system slots.

The negative changes only `Ok(0xbau8)` to `Ok(0xabu8)` in the swapped control.
Its raw stderr contains exactly the intended backend assertion at iter.rs:376:12
plus the one-error abort summary, not a frontend error, panic, timeout or
status-only oracle. `interface_written` is false, no .vir export exists, and
checked-body/proof credit is zero. Fresh 03 restores all eight source files
byte-identically to fresh 01 and to the final candidate's corresponding sources.
Both successful rechecks have complete selected 38-exec/2-proof scope.

Every attempt directory retains exact argv, sanitized environment, cwd, source
hashes/snapshots, timeout, status, wall time, complete stdout/stderr, scope and
inference fields. The source/trust audits check all six attempts, including
their raw outputs and negative no-export behavior, without conflating the
historical three applications with the fresh review's separate four-attempt cap.

## Authentication, resource limits and handoff

Entry and exit authenticate all 132 pinned candidate payloads; report, output
manifest and handoff match the three user-supplied hashes. The retained parent
archive matches SHA256
`4fc0fa56c8f1795a419f74759644f29e183e992a9d197d3ce622943646d2f210`,
and all 134 regular members are byte-identical to their candidate files. The
candidate contains 135 regular files: those 134 members plus auxiliary
`handoff.sha256`; that auxiliary file is also hashed in the review inventory.

There are 202 authenticated immutable input files, including all predecessor
manifest payloads, the independent review and identity oracle, used parent
source bindings, native archive, nine runtime artifacts, recorded rustc, all
30 recorded loader libraries and the pinned runtime manifest. These are the
recorded inventory bindings, not authentication of every possible OS/system
library or an entire sysroot tree. Entry/exit inventories must match exactly.
Archives are inspected in memory, never extracted into or modified in place.

Scratch remains below 128 MiB; global free disk checks remain above 16 GiB.
The final ledger reports actual file allocation, directory allocation, elapsed
time and observed free-disk minima, rather than claiming continuous OS-level
resource isolation. Verifier core dumps are disabled and each output file has
an 8 MiB file-size limit. The recorded 60-second worker cap is independent of
the total review deadline. No further research chain is started.

One early read-only inspection looked for runtime-manifest.json inside runtime/
and failed; the recorded root-level manifest was then read and authenticated.
That lookup did not start a verifier or change any input. All review audit
programs completed successfully; no verifier application was hidden as metadata.

No parent/candidate/frozen edit, Git write, subdelegation, compiler/vstd/runtime/
Cargo build, native application run, network/download, old-evidence deletion,
excluded growth049/zero-growth050/mutation030/refused-padding95/drop-pipeline
work, native final acceptance, trust/runtime adoption or parent integration
occurred. Only this fresh /private/tmp scratch was written. The parent retains
decision authority over this sealed review and any separate later work.
