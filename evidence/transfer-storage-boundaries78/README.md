# Reviewed native storage-boundary transfers, cycle78

November 2, 2026. Both bounded application results are accepted after independent
review and the redb receipt-binding repairs. Engine identity remains
**cache77 / policy71 / schema38**. This cycle changes no production Rust, vstd,
native application source, shared guide, controller or CI workflow. Historical
implementation reports retain their original pending-review wording; this
acceptance record supplies the subsequent review and integration status.

## redb: original selection with the complete error declaration

The four unchanged extracted bodies `select_primary_slot`, `primary_slot`,
`secondary_slot` and `swap_primary_slot` now check with the complete native
`StorageError`, all payloads and native error attributes. Two explicitly authored
opaque datatype declarations cover `std::io::Error` and
`std::panic::Location<'a>`; no behavior specification or application axiom was
added. Extraction visibility/module/unused-derive adaptations remain explicit.

Under the authored representation requirement `primary_slot < 2`, the checked
contracts establish two-phase primary preference/failure, both-corrupt failure,
fallback from a corrupt primary, selection of a strictly newer noncorrupt
secondary, preservation for equal/stale/corrupt secondary cases, the exact
kept-original-primary flag, and framing of every other logical record field.
Corruption observations are those at entry, not a proved checksum/tree predicate.
Only transaction-ID comparison plumbing is inferred; the selection contracts,
clients and checked XOR lemma are authored.

Each provider reports **23 verified / 0 errors**, exactly 43 scope rows with
15 checked executable bodies and one proof body. Four of those executable bodies
are the native methods; the others are checked comparison derives and authored
clients. The certified consumer reports **5 / 0**. Generated Debug, opaque
payload behavior and destructors receive no proof credit. Native callers have
not been shown to establish the representation invariant. Recovery, tree
validity, allocation/reuse, torn writes and durability remain unproved.

The independent review found a receipt-only VIR/rlib argument cross-wire defect.
The first repair's recheck additionally exposed joined-option imports and a
cardinality-test gap. The final reader binds `--import` to the selected VIR and
`--extern` to its rlib, requires exactly those two imports, rejects joined option
spellings, and checks unrecorded imports. Targeted tests detect removal of the
individual joined-option and total-cardinality guards. Both bounded corrections
are independently accepted; no false selector theorem was observed.

## Fedimint: source-connected pure prefix construction

Three byte-identical native bodies are checked: both `get_full_key` methods and
`PrefixDatabaseTransaction::get_full_range`. Original record fields/order/types,
`Vec<u8>`, slices, clone/extend calls and Range construction are retained, with
declared scaffold/visibility/contract adaptations.

Explicitly authored contracts and checked lemmas establish exact concatenation
`full_key@ == prefix@ + key@`, prefix membership, same-prefix suffix cancellation
and injectivity, conditional separation for prefix-INCOMPARABLE namespaces, and
exact transformed range endpoints. A checked comparable-prefix overlap witness
and a failing unconditional-disjointness goal expose the necessary condition.
Actual BigSize-encoded module prefixes are not proved incomparable.

The provider reports **10 verified / 0 errors**: three native and two authored
executable bodies, five proof bodies and three specification definitions, with
13 selected identities. Imported consumers report **3 / 0** each. These are
ordinary checked contracts and imports, not inferred summaries. Infer still
requests three helpers, generates zero, and rejects native
`Vec::extend_from_slice`: **ZERO INFERENCE PROGRESS for this transfer**.

This does not prove transaction-wide confinement, encoding correctness, backend
key/range behavior, token lifetime, async dispatch, commit/drop/hooks, consensus,
fund safety, atomicity or durability. The earlier conditional transaction/replay
model still has zero established native correspondence; this pure bridge does
not silently discharge its assumptions.

## Validation and trust

- Parent full Python discovery: **435 tests normally and 435 under `-O`**.
- Dedicated parent replays: **47 redb and 13 Fedimint expected outcomes per mode**,
  plus all six existing common transfer scenarios on the repaired tree.
- Final independent redb recheck: **63 tests and 47 compiler cases per mode**;
  genuine receipts accept, cross-wires/joined variants reject, and isolated
  check removals cause the intended assertion failures without incidental errors.
- Independent Fedimint review: **41 tests and 13 replay outcomes per mode**, with
  all 15 acceptance-guard removals detected and non-vacuity/false-goal probes.
- The source/build/runtime/toolchain/native hash ledgers match. Native pins are
  redb `c82ab865bb362bc4af067d332adc44978ff64813` and Fedimint
  `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`. Native clones remain unchanged.

Counts describe checked queries and intended test outcomes, not whole-application
theorems. Full vstd, engine and broad Rust suites were NOT freshly rebuilt or
reverified this cycle; the unchanged sealed runtime/vstd is reused. Existing
Verus/Rust/SMT and standard-library specifications remain trusted, including
Vec clone/extend correspondence; Rust std implementations are not verified here.
Ordinary failed-proof exports may remain despite `interface_written=false`;
none is imported as successful proof evidence. Inference-aware failed-export
removal is accounted for separately. Receipt consistency is not cryptographic
attestation. Shared orchestration ref activity is disclosed in the reviews;
this work issues no commit, push, branch/ref change or PR action.

## Replay, evidence and next mechanical work

The portable entrypoints are `source/tools/check_redb_slot_selection_opaque.py`
and `source/tools/check_fedimint_prefix_boundary.py`. Each takes `--application`,
`--verus`, `--solver` and a fresh `--output`; neither requires an old scratch
snapshot. Their implementation reports document exact commands and trust scopes.
`validation.json` binds fresh parent receipts/logs. `manifest.json` binds the
archived reviews and original immutable paths. Full-patch reconstruction and
complete source/build/runtime ledgers are in the final external handoff.

Four additional unreviewed SYNTHETIC diagnostic probes identify a concrete next
Infer target: byte-vector clone already infers, including through a borrowed
generic record; native append does not. Range-valued composition has a separate
generic-call/profile blocker. These probes are not native application proofs.
The next bounded extension should infer primitive byte-vector append from the
existing library semantics, then replay unchanged native prefix helpers, without
inventing namespace, backend, transaction or durability facts. Those higher-level
abstraction boundaries remain author-owned.
