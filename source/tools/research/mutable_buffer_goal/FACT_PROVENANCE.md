# Existing iterator facts explain the canonical observer ablation

The unchanged canonical fixture again verifies **6/0** with the observation
decorator completely disabled. This diagnostic uses the delivered immutable
observer-off ablation runtime, fresh cache and normal `--no-cheating` checking.
Its exact checked inventory is executable `{u64}`, proof `{}`, inferred helpers
`{}`. [Receipt, hashes and log anchors](canonical-fact-provenance.json) bind the
historical runtime, original fixture, pretrial freeze and private AIR/SST logs.
This is available-fact provenance, not a new causal ablation or native proof.

The constructor already supplies the important representation facts:

1. `vstd::array::ref_mut_array_unsizing_coercion` preserves the initial array
   view and relates the final slice view to the final array view. This function
   has an inherited `external_body` contract.
2. The inherited `assume_specification` for slice `iter_mut` says its initial
   reference sequence has the slice's length, each initial referent matches its
   input element, and each final referent matches its final output element.
   The actual call assumes this summary in `root.air:15500`; its two element
   quantifiers are present at lines 9160 and 9196.
3. The checked `VerusForLoopWrapper::new` saves the original iterator. The
   wrapper's `seq()` is that snapshot's remaining sequence. The for-loop macro
   adds snapshot equality, wrapper well-formedness and iterator-law invariants;
   these remain available across the isolated loop. The AIR saves the snapshot
   at line 15519 and uses its equality at line 15688.
4. The numeric decorator aligns the authored counter with the wrapper index and
   supplies checked bounded arithmetic/bit-vector work. The authored semantic
   prefix talks directly about `final(it.seq()[j])`. The existing constructor
   correspondence connects those referents to the final buffer.

Thus the observer's additional capacity/observation proposals need not be the
origin of these facts. The controlled ablation independently established their
redundancy for this frozen canonical proof. Presence in an AIR query does not
establish that each individual clause is indispensable; no unsat-core or
contract-removal experiment is claimed. Standard contracts and checking were
retained throughout.

Canonical encoding overwrites the returned prefix from the number. The HMAC
writer also needs the original byte values, and its full theorem additionally
requires unchanged bytes for every unissued suffix position. The prefix observer
helps expose the initial/final maps, but the inherited opaque iterator contracts
do not justify that unused-tail frame. Zip's native indexed implementation also
requires an index-aware ownership projection; inner iterator cursors alone do
not describe the consumed prefix.

A potential common checked foundation is therefore the original-source ownership
and final-element correspondence behind array unsizing and mutable iteration,
with conservation of unissued slots for the second writer. This is a **proposed
proof interface**, not established shared assistance. The current canonical
proof uses inherited summaries; the full HMAC theorem remains unproved.

The [source-entry investigation](../native_source_entry/MISSING_LAYER.md) pinpoints
the first missing judgment: interpret the actual incoming Rust configuration
and typing context into model ownership at the same physical subobject, with
its bytes, provenance, ancestor loans and C/F observations. Extracted loan `0`
describes the local reborrow, not the incoming argument's dynamic loan. Checked
model accessors consume an already established interpretation and cannot create
this source relation. All 44 minimal native simulations and 101 full-writer
effects remain open. No native fact or publication is enabled.

To reproduce the diagnostic, use the immutable runtime identified in the JSON,
copy `candidate-008.rs` to a fresh private directory, set a fresh
`VERUS_INFER_CACHE` and the bundle's `VERUS_Z3_PATH`, then run its `verus` with
`--crate-type=lib --no-cheating --output-json --infer-goals --log vir-sst --log air`
and a private `--log-dir`. The committed 45-case ablation remains the causal
evidence; this one additional invocation only exposes fact provenance.
