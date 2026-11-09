# Frozen redb R1 — cycle 1 implementation handoff

This is a conditional native-block/caller composition, not either whole
`from_bytes` method, whole `finalize`, recovery, storage or checksum verification.
The parent froze this exact boundary before implementation. Independent review
and integration are still required; this directory does not accept the goal.

## Source and scope

Native redb pin: `c82ab865bb362bc4af067d332adc44978ff64813`.
`scope.json` binds seven full native files, nine whole executable method bodies,
three exact contiguous blocks, all record/error payload declarations, relevant
derives and the original offset/size constant declarations. The fixture's native
body/block bytes are unchanged. Visibility, verification annotations and wrapper
signatures are extraction scaffolding. The three wrapper bodies remain exact
blocks inside their original owner impl; they are **not whole native callers**.

`candidate.rs` composes:

1. Original `get_u64` and `TransactionId::new`.
2. Exact transaction-ID/complete-record tail from `header.rs:582`.
3. Exact physical corruption orientation/complete-header assembly tail at 191.
4. Original accessors, total swap and `select_primary_slot`.
5. Exact selection/return block from `finalize` at 279.
6. An explicitly authored `decoded_id_selection_client` joining these boundaries.

The semantic contracts establish exact little-endian IDs at native offset 104,
physical slot/root/version and all unrelated header-field preservation, conditional
preservation of the first 128 corrupt bytes, native branch-sensitive selection,
exact corruption strings and storage-error wrapping, and
`clean == kept_primary && !layout_stale`. Roots, versions, corruption observations,
layout staleness and all geometry fields are arbitrary explicit inputs. They are
not assumed summaries of omitted checksum/root/layout decoders.

Domains remain: leaf lengths at least 8; tails at least 112 with length at least
128 **only if corrupted**; total swap over every `usize`; orientation over every
`usize`; primary accessor index below 2; secondary accessor `(index ^ 1) < 2`;
selector safety `two_phase_commit || primary_corrupted || primary_slot < 2`.
Safe invalid-index early branches and clear lengths 112 through 127 are retained.

## Reusable mechanics and trust

- `infer/slice_array.rs` recognizes canonical core slice-array `TryFrom` and
  blanket `TryInto` followed by `Result::unwrap`. It checks native identity,
  vstd proxy, generic bounds, signature, resolved implementation and concrete
  scalar/array/error types. Its proposals use the already existing vstd
  `std_specs/chunks_exact.rs` slice-array correspondence, generic forwarding and
  `Result::unwrap` specifications. Ordinary independent soundness, safe-domain
  maximality and output-exactness queries check each inferred summary.
- The inference representation supports fixed scalar arrays of lengths 0–64;
  it does not cap input slice lengths. The native 128-byte corrupt-byte copy is
  checked ordinarily against the existing generic vstd conversion specification,
  not summarized by this bounded inferred-array representation.
- `rust_verify/native_const.rs` handles a closed portable `usize` const DAG:
  literal/local-const/zero-argument native helper dependencies and checked
  addition/subtraction/multiplication. Canonical core `size_of` is bound to rustc
  layout queries for bool/fixed-width integers only. Every intermediate fits
  `u32`; target-sized/arbitrary-layout types, casts, unknown/generic calls,
  external-body helpers and nonportable forms decline. The actual source const
  is evaluated by rustc and compared with the guarded DAG. Generated Verus const
  proxies must uniquely match the same-parent native const and the same value.
  This is an explicit use of existing rustc/macro trust, not a new size axiom or
  an application-name gate. Original compiled native expressions are unchanged.
- `infer/xor_goals.rs` discovers unsigned constant-mask shapes through bounded
  contract/spec dependencies. Range and all-ones-complement proposals are
  independently checked by the existing bit-vector backend before their exact
  checked/use pair enters ordinary queries. Premises are retained; no record
  invariant or application summary is assumed.
- The authored `FromSpecImpl<StorageError>` definitions state constructor intent.
  The unchanged native `From` body proves both inherited and explicit contracts.
  No unchecked `From` implementation summary is supplied.

No new trusted specification family or application axiom is added. The existing
u64/eight-byte little-endian standard model is reused; there is still no u128
checksum LE binding or checksum oracle. Existing accepted opaque specifications
for `std::io::Error` and `std::panic::Location` retain all seven `StorageError`
and six `DatabaseError` variants/payloads. Formatting, opaque internals and Drop
semantics are not proved. Relevant native derives remain compiled; TransactionId
comparison bodies are checked. The native collection names const helper proxies
as selected items without translated bodies; their six exact inference checks
are reported separately, not mislabeled ordinary checked-executable rows.

## Evidence and tests

The private sealed handoff is
`/private/tmp/verus-redb-cycle1.wj99Jj/final-package`. It contains the exact
dirty-cycle80-relative `apply_patch` delta, inventories, non-Git reconstruction,
native/toolchain/baseline/runtime guards, commands, input/output hashes and raw
logs. The full accepted baseline patch is never reapplied during implementation.

Final R1 replays use the supported launcher, explicit sysroot, default erasure,
`--no-cheating --infer-goals --compile`, private caches, and both debug modes.
Each plain full-target run has 47 verified obligations, zero errors, five fresh
exact inferred contracts and 19 checked ordinary executable bodies. There are
81 final matrix cases: five positive runs, 22 typed semantic negatives, and 54
safety negatives. All 1,215 required helper checks run freshly. Every one of the
76 negative runs removes a seeded inferred export. Successful inferred exports
and compiled libraries are hash-bound; ordinary exports are not certificates.

Semantic controls cover endian, orientation, two-phase policy, both corruption
observations, equal-ID swapping, always-primary, clean, messages, wrapping and
corrupt-byte laundering. Safety controls cover leaf lengths 0–7, clear tail 111,
each corrupted length 112–127, unsafe accessors and the unsafe selector branch.
Positive boundaries retain longer slices, short clear tails, corrupt long tails,
invalid-index early branches and total swap.

The new source/receipt guard tests independently exercise named guards, including
joined/separated/cross-wired zero-application-import flags, without an earlier
digest/type failure masking the intended guard. These scope-policy tests are
not compiler proof credit. New Rust integration tests cover the full fixture,
wrong-clean semantics, corrupted-short safety, direct and forwarded conversion,
empty bool arrays, non-byte values, wrong elements and oversized conversion.

Adjacent codec/full-error-selector/fee-encoding/prefix direct compiler replays
are additional regression evidence, not replacement target proofs or legacy
reader acceptance. Existing shared reader identity expectations remain 79/73;
this candidate reserves 80/74/schema38. The parent owns their reconciliation and
its separate generic decoder integration file, which is not duplicated here.

## Before/after and limits

At cycle80 the original leaf declines with `Infer unsupported: generic calls`;
the complete raw candidate also exposes native const/frontend boundaries.
The literal-size diagnostic fragment is labeled a fragment, not a native whole
method. No baseline complete proof existed. The final fixture has zero authored
assertions, proof blocks, local arithmetic/byte/cast/xor lemmas or assumptions;
the author supplies semantic intent/frame contracts, not mechanical proof steps.

All whole decoder/finalize properties, bytes-derived primary-index validity,
checksums, root decoding, geometry/layout computation and application recovery
remain outside this proof. The parent's separate finite native oracle supplies
zero formal proof credit here. No scope reduction or substitute target is used.

## Integration ownership

Own HEAD/index and native HEAD/index/refs remain unchanged. The shared Verus ref
set gains three concurrently observed review/platform refs; no existing ref moves
or disappears. Exact before/after lists are sealed; no manual ref command runs
here. The parent's worktree concurrently changes `infer_derive_ops.rs`,
`import_export.rs`, `infer/owned.rs` and `infer/registry.rs` while integrating the
other track. Those are observed external changes, not writes by this task; do
not claim that all parent files remain unchanged. Own delta is relative to the
original dirty cycle80, not that evolving parent worktree.

Reconcile the two identity lines at parent 82/76, and the small Infer dispatcher,
eligibility and goal-assistance integration hooks if the sibling touches them.
The slice-array implementation and const mechanics are isolated new modules.
No shared reader/controller, CI, guide, goal document or parent-owned generic
integration test is edited by this candidate.
