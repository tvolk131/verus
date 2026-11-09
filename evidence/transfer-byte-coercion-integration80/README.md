# Accepted cycle80: borrowed-byte coercions and bounded native links

The local integrated snapshot is accepted within the scopes below after three
independent reviews. Identity is **cache79 / policy73 / schema38**. No commit,
push, manual ref update, PR action or hosted CI dispatch is part of this cycle.
This is not a whole-application, recovery, durability or funds-safety result.

## Mechanical work newly inferred

The byte-append profile now handles inert immutable `Vec<u8, Global>` borrows in
the argument to the existing `extend_from_slice` model, including direct owned
fields and locals. For example, these statements can participate in an inferred
exact contents contract without a hand-authored append postcondition:

```rust
output.extend_from_slice(&self.prefix);
output.extend_from_slice(&local_bytes);
```

The source's current contents are used, call order is retained and the result's
sequence state is updated. This is bounded canonical coercion support, not
general trait dispatch, arbitrary `Deref`, custom allocators, mutable-reference
alias reasoning, effectful expressions or arbitrary element-type cloning.
Unsupported valid programs may still decline conservatively.

One existing standard model identity is newly recognized:
`alloc::vec::impl&%10::deref`. The vstd specification itself is unchanged. The
complete fresh producer and **381** model fingerprints are pinned, not only the
new entry; all 380 common fingerprints changed with the fresh bound artifacts.
Soundness/safety, safe-domain maximality, exactness and ordinary caller checking
remain separate mandatory obligations. No new application axiom is introduced.

The three original generic Fedimint prefix helpers proved in cycle79 remain
source-exact and verified. The new field/local coercion examples and public
import wrappers are explicitly auxiliary; they are not additional extracted
native methods or a claim that the whole prefix transaction implementation is
inferred. Original native method bodies, domains and application files stay
unchanged.

## New redb native link

The original `UnrepairedDatabaseHeader::finalize_transaction_slots` body is
checked over its complete input domain, with an explicitly authored contract:

- A recorded corrupted primary returns the exact native corruption error.
- Otherwise it returns the complete original inner header, preserving every
  field. It does not establish that the primary index is in range.
- It does not implement fallback repair, validate checksums or establish tree
  correctness. A separate source-connected error-normalization auxiliary proves
  only its extracted match-arm behavior, not the full checksum classifier.

The provider has **15 verified obligations / 0 errors**, eleven checked executable
bodies and three checked proof bodies. Only one executable body is the original
native finalization method. All these semantic contracts are authored; this
part of the cycle adds no Infer capability. Original finite native controls run
three tests over 64 gate inputs; they are not universal verification evidence.

## New Fedimint conditional operation link

The existing inferred private key helper and an auxiliary wrapper are composed
with an explicitly authored backend relation. Assuming an operation establishes
`after == before.insert(prefix + key, value)`, the checked footprint lemmas show
that entries outside that namespace are preserved. Incomparable peer prefixes
give a separate conditional namespace-frame result. Backend behavior and prefix
incomparability are premises, not inferred conclusions.

Provider **12/0** and certified imported consumer **2/0** check the preparation /
footprint link. They do not verify the native async insert operation or RocksDB.
The exact native async extraction still fails at opaque types in a trait
definition. Its six finite Rust harness tests cover 127 forwarding, cancellation
and error scenarios, but do not establish async/backend refinement. No inference
progress is claimed for this semantic link.

## Validation and review closure

Fresh parent integrated-runtime checks:

- **508 Python tests normally and 508 under `-O`**, with the receipt-dependent
  redb class enabled rather than skipped.
- All six common transfer scenarios pass. The initial wrong dependency-home
  attempt is retained as a setup failure and gets no proof credit.
- Current full-error redb selector: **47 expected outcomes per mode**; current
  native prefix replay: **29 per mode**, without the optional historical baseline
  comparison. These preserve the previous contracts and complete native scopes.
- Additional native80/runtime79 cross-qualification: seven outcomes per mode.
  Fedimint provider 12/0 cold and warm, imported consumer 2/0, missing backend
  premise rejection and unchanged async frontend blocker; redb gate 15/0 under
  both debug-assertion configurations. These are not full historical-matrix
  reruns. Two earlier parent Python setup mistakes are retained without credit.

Matched implementation-run results are **150 VIR tests, 96 focused Rust tests,
and 2,221 vstd verifications / 0 errors**. Source, 1,088 build inputs, runtime and
toolchain hashes are matched. The parent and reviewers audit these recorded
results; they do not claim to have redundantly rebuilt or rerun those suites.

The independent production review has no findings in the seven-file engine
delta, with 98 freshly checked outcomes. The isolated redb recheck accepts both
P2 repairs: real hash-bound positive libraries and complete native command /
artifact evidence. It freshly checks 28 cases per mode and 43 test methods in
four runs, plus independent receipt and guard-removal probes. Parent Fedimint
tests now catch deletion of each integrated report-shape / fresh-work call.

The final compatibility review accepts its exact ten-path scope: 99 focused
tests per mode, fresh optimized opaque47/prefix29, 800 intended identity/model
rejections per mode, and both isolated Fedimint gate deletions failing for the
intended reasons with zero incidental errors. It audits prior normal replays,
the parent's six-scenario evidence, exact 381-model pins and historical 43-row
native selector scope. The earlier interrupted review remains incomplete
historical evidence and supplies no approval.

Original independent reports and small machine receipts are archived here
unchanged. `validation.json` binds the original sealed packages, raw receipts,
commands/scripts and parent logs. Frozen child documents saying pending review
or describing old reader failures remain historical, not silently relabeled.
Shared checkpoint-ref changes observed by some earlier agents are disclosed in
their reports without attribution. The final compatibility review observes no
ref drift. Hash consistency is not signed or cryptographic execution attestation.

## Trust, exclusions and next frontier

Existing Verus/vstd/SMT specifications, Rust frontend correspondence and explicit
opaque datatype boundaries remain trusted. Allocation success, capacity/OOM,
Drop behavior and general object/reference identity are not established.

No confirmed reachable production bug in redb or Fedimint is established by this
cycle. No database atomicity, torn-write freedom, fsync durability, consensus,
global exactly-once, liveness or funds-safety theorem follows from these slices.

The next application-led cycle should target concrete blockers, not assume the
missing semantics: redb's original decoding/endian/constant/dependency path and
its checksum classifier; Fedimint's async trait/dependency boundary and a precise
author-owned backend operation/atomicity contract. Representation invariants,
valid-slot policy, namespace encoding guarantees and crash/durability semantics
still need explicit definitions and refinement proofs. Mechanical conversions,
byte handling and contract propagation beneath those boundaries remain suitable
Infer work when a source-connected failing example establishes the need.
