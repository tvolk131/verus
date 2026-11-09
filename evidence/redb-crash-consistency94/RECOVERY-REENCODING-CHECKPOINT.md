# Native recovery re-encoding021 — November6,2026

## Plain-language result

The parser and recovery code previously stopped at an in-memory recovered header.
This checkpoint connects that header back to the real serialization code and
then back through parsing and recovery. Repeating the complete cycle gives the
same header. Under the stated starting-state conditions, interrupting the header
write-back at any individual bytes, and retrying it arbitrarily many finite
times, also preserves the recoverable header.

This is a conditional byte/native-body composition result, NOT a proof of the
actual disk-I/O caller, fsync ordering, reachable crash histories, whole database,
or logical KV correctness. All18 final cells remain OPEN. Nothing is adopted;
NEW independent review is required.

## Exact source and checked scope

`reencoding-001.rs` retains the entire sealed020 shared-type provider, changing
only its documentation/imports and appending serialization specifications plus
three full exact native bodies: TransactionId::raw_id, TransactionHeader::to_bytes
and DatabaseHeader::to_bytes. All36 recovered-native bodies/signatures and five
struct records match the pin c82ab865bb362bc4af067d332adc44978ff64813 after the
disclosed annotation/token erasure. This is lexical/type correspondence, not a
native-crate nominal-identity, AST/instruction or original-Git-continuity proof.

The actual fixed112 native wrapper/dispatch013 is used. The scalar-only adapter
and candidate010 arithmetic repairs from012 are NOT imported. Original009
finalize is unchanged. The provider verifies111/0 in both debug modes:43 checked
exec bodies =36 native +6 generated (three Clone, three comparisons) +1 inherited
authored composition; nine inherited proofs. Query totals are not body counts.

## Roundtrip and second recovery

`roundtrip-005.rs` verifies13/0 in both debug modes (three exec, nine proofs).
It checks exact slot/root/header encoding and parsing, canonical-root premises,
normal checksum generation, and raw-byte retention for corrupted slots. A parsed
corrupt slot must include its actual version3 byte and decoded fields/checksum
facts; `parsed_slot` by itself omits raw-version linkage. The actual outer parser
establishes the stronger domain. This is neither a new axiom nor a redb defect.

`recover_encode_finalize` requires only at least320 bytes, calls the actual020
parser/finalizer, native serialization, parser and finalizer again, and proves
the same recovered header with second-finalization clean=true. Acceptance,
projected errors and the first observed clean status remain explicit. The file
length is an arbitrary supplied u64, not yet a proved native physical-length read.

## Interrupted and repeated write-back

`interrupted-005.rs` verifies5/0 in both modes (one exec, four proofs).
`recover_interrupted_writeback` starts with a header in the exact roundtrip
domain, recovery_required=true, and an actual successful-finalization relation
to the target header at the supplied length. Any byte may independently be old
or new. Immutable geometry and both transaction slots are framed; only the
selector/count area may differ. Even arbitrarily torn count words are not
trusted. Actual parsing/finalization returns the exact target header for either
commit-mode policy. `repeated_writeback_images` inducts over any finite retry
sequence targeting that SAME recovered header.

Important scope limit: this theorem preserves the other flags, including
primary_unflushed; it does NOT cover read_header's separate flush-and-clear of
primary_unflushed or the later clearing of recovery_required. Changing those
flags needs its own native caller/barrier proof. The initial native-encoded dirty
header and successful-finalization relation are premises, not initialized/crash
history reachability conclusions. Bytewise mixtures do not imply atomic file
length, sector atomicity, safe cache behavior or checksum collision freedom.

## Genuine consumers and satisfiability

`clients-003.rs` is an ordinary rlib/metadata import,3/0 both modes, two exec
identities. It actually calls native before/after serialization and slice copies
for every symbolic prefix cut0..320, then the actual recovery composition. The
client returns the actual observed clean witness rather than an existential-only
interface. The executable witness covers both mode policies, not native database
initialization. Six unchanged inherited020 fixtures reverify16/0 in both modes.
Five new passing semantic counterparts verify15/0 in both modes: mode selection,
corrupt-slot retention, dirty-count reconstruction, and two-phase error behavior.

## Controls, failures and conservative accounting

34/40 applications:14 complete positive scopes, nine intended backend
rejection-only scopes, eleven ZERO-credit failed drafts/resource/frontend scopes.
Exactly24 NEW exec/proof identities (11 exec,13 proof), including test counterparts
and all draft names; no further identity allowance remains. Source versions are
retained separately. Six fixture identities and all020 base declarations are
inherited, not silently recounted as new native bodies.

Five paired one-claim negatives019–023 reject at their exact intended assertions.
Three native-body alterations031–033 reject inside the affected checked member
(mode-bit encoding, primary-byte encoding and checksum-corruption detection).
034 withdraws the repeated-writeback transition premise and rejects at the
corresponding induction assertion. Negative scopes are not complete verified
scopes. Verifier errors:1 counts one failed function, not a diagnostic count.

030's corrupted-raw-byte mutation exhausted the prover resource limit: ZERO
mutation/control credit, no retry. Its original class remains uncovered by a
clean body mutation. 031 initially stopped because the reader expected a
postcondition/signature diagnostic, while the mutation correctly failed an
unchanged intermediate selector assertion1585. The initial ZERO assessment is
preserved; a separately labeled late source-pair/location/all-error-header audit
establishes rejection-only credit without another verifier launch. See parent
reassessments030/031 and both retained assessments. No database/verifier semantic
defect is inferred from that reader mismatch.

Other failed drafts002–004,007,009,011–012,014–016 receive ZERO complete-scope or
control credit, regardless of individually successful queries. Failures include
resource exhaustion, missing byte/array/XOR/trigger/induction facts, an invalid
import from a read-only private-copy patch failure, and incomplete client/control
obligations. Corrected versions keep native algorithms and final claims intact.
An auxiliary progress-listing command observed033 before its receipt existed;
it is not an additional verifier launch or failure. All34 applications retain
input, argv, environment, bound imports, output, timing and scope receipts.
`import-retention-ledger.json` locates exact historical imported metadata/binaries,
including superseded versions, by hash in retained per-attempt outputs.

## Trust, preservation and review status

Zero new trust, compiler/vstd/Cargo/harness builds, target-native executions,
network operations, native/parent/compiler edits, Git writes or adoption.
Application compile/export operations are separately recorded. Existing std
array extensionality is used; it is not a new application axiom. Inherited TCB
includes two reviewed016 std bool contracts, six LE boundaries, fixed112 hash,
codec/StructuralEq and existing compiler/std assumptions—not checked Rust-core
bodies. Compiler source binding remains NONEXHAUSTIVE1084.

Initial/final2347 parent source paths,1401 immutable content guards,1331 recovered
native contents, nine runtime files,3340 sysroot files, frozen020/012/016 packages,
bound reviews and owned HEAD/index/branch/dirty state match. Shared-ref additions
are separately inventoried without attribution. Original1331 native paths remain
MISSING; original-path preservation FAILED, native Git continuity UNAVAILABLE,
historical/shared objectstore NOT_CERTIFIED. Content checks do not restore those.
Free space remains above8GiB. Completed020 review278 payloads and constructor019C
handoff648 payloads authenticate as LATE handoffs, not retroactive initial bindings.
020's review supports its bounded scope, not this unreviewed021 extension.

## Reproduction and next obligation

`python3 -B reproduce.py` authenticates the sealed package with ZERO verifier
launches. Optional replay uses a NEW nonexistent /private/tmp directory, exact
bound sysroot/runtime, fullscope --no-cheating and no source changes. The frozen
21-application recipe pauses after12 for an actual reviewer reassessment; it was
NOT run by this author and is not precredited. See REPRODUCTION.md.

The next real bridge is native read_header/write_header/cache/flush behavior,
including the actual WritablePage destructor, original cache ownership, barriers,
flag-clearing/error paths and physical file length. Treating those native methods
as a trusted recovery/commit oracle would not close the goal and is not authorized
here. All six properties for one-phase, two-phase and mixed histories remain OPEN.
