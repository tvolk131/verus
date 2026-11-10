# Arbitrary finite MODEL heap initialization

This lane implements the missing finite-map operation and a generalized model
operational adequacy proof. It does not prove Rust source entry or complete the
full goal. Only this research directory is delivered; no native facts, hooks,
publication, application changes, shared baseline writes or dependency installs.

`INTERFACE.md` was committed before trials at `da164976`. It freezes the domain,
resource/trust inventory, allRSt0 convention and empty freeability choice. The
first proof/controller trial is preserved at `f9cc7d74`; unsuccessful scripts
are development failures, not semantic negatives. Exact final evidence is in
`results.json` and historical failures in `failed-trials.json`.

The implemented statements are:

- `finite_map_initialize`: for every `h : gmap loc val`, allocates a fresh ghost
  map interpreting exactly `(fun v => (RSt 0,v)) <$> h`, together with one full
  points-to fragment for each actual key/value. The proof uses `map_ind`, the
  existing empty allocator, and fresh insertion. It needs no supplied permission,
  owner, allocation base, zero byte or application premise.
- `initialized_heap_adequacy`: for any given model `e`, `h`, value postcondition
  and initial credit count, initializes that exact model state internally and
  gives the WP proof all the fragments, credits and `time_ctx` in the same heap
  instance. The conclusion is existing `adequate NotStuck`, using existing Iris
  strong adequacy, state interpretation, time initialization, atomic lock pool
  and counter. It preserves their normal safety/race semantics. The WP proof
  obligation remains; this theorem does not say arbitrary programs are safe.
- `initialized_read_adequate`: instantiates that theorem with an actual read of
  any key in any given nonempty heap, returning its exact initial value. This
  exercises heap fragments and operational lifting, rather than merely a pure
  pointer expression. It is solely an instance about the existing model.
- Generic submap and interval laws extract and recombine exactly `h ∖ submap`.
  `larger_interval_partition` splits prefix, middle and suffix of a contiguous
  same-block interval while retaining any disjoint unrelated map. Nonempty
  prefix/suffix give a strict interior subobject; neither is forced empty.
  `byte_interior_payload` handles arbitrary ranged byte lists of any length,
  including 10 and 128, and exposes the existing cloc/FVal physical payload
  shape consumed by v12. It neither imports nor regenerates the old 18+6 proofs.
- `initialized_interior_adequacy` supplies the subobject AND exact frame inside
  the global adequacy initialization. There is no external caller token premise.

All interval/submap premises are pure facts about the supplied raw MODEL map,
not assumptions that an original Rust reference already has `ty_own`. The raw
map is not assumed already to be a valid semantic owner. The byte range premise
states that the selected mathematical integer values are bytes; it is not a
source-validity definition. The general initializer accepts every model value.
No physical reinterpretation or Rust-to-model translation occurs here.

AllRSt0 and global-lock=false describe this model initialization convention.
They are not new conditions on original valid Rust input histories. Empty
freeability authority satisfies `heap_freeable_rel` for every h, as separately
proved. It grants no deallocation right even for an entire initial allocation,
and especially none for a borrowed interior subobject. Existing model payload
read/write rules need no such token. Initial contexts that require freeability,
nonzero read-bank counts, locked cells or suspended logical borrows need separate
initialization from a precise raw-state/partition invariant. Such partition
initialization and the original source context interpretation are not proved.

All fragments refer to the same initialized heap name; extraction/recombination
never reallocates a ghost world. Running the allocator twice creates separate
worlds and cannot convert an existing live heap or recover its loan/provenance.
The physical payload equivalence supplies neither v12's `ty_gho` infrastructure
nor its original unique borrow, lifetime guards, time ticket, prophecy or C/F.

The refusal controls are ordinary proved countertheorems. A fragment with the
wrong initial value, a fragment at an absent location, an altered frame value,
an overlapping alleged separating frame, or two full same-namespace permissions
entails False using the actual map agreement and cell-map exclusivity rules.
These are five checked semantic/resource countertheorems, not failed tactics.
`interval_retains_frame` also proves the exact disjoint frame map is retained.

No actual incoming Rust state, typing derivation, runtime provenance, ancestor
loan/protector, original future observation, or native source step is interpreted.
All 44 minimal native simulations and 101 effects remain OPEN. Native-drop 0/1
and full frozen HMAC 1/1 are inherited diagnostics and are not rerun. The canonical
full-u64/arbitrary-[10] goal and full HMAC prefix+suffix/arbitrary-[128]/all valid
keys including 0 and >128 remain unchanged. No inference assistance, application
proof, runtime replacement writer, source CF import or publication is claimed.
Parent integration and fresh independent review are required.

Validation uses normal `coqc -q` and recursive `coqchk -silent -o` on the immutable
284-object foundation, never a baseline rebuild. Both the first successful run
and final replay use `python3 -B -O`. All 29 named constants (4 definitions and
25 theorems, including auxiliary lemmas/controls; zero Program obligations) get
individual complete `Print Assumptions`. Every one is closed under the global
context. The recursive dependency summary reports only inherited dependent
functional extensionality and Eq_rect_eq, with no assumed positivity, unsafe
(co)fixpoints, type-in-type or rewrite rules.

`check.py` authenticates all 523 foundation source identities, the 284-object map,
tool hashes, ready marker, successful final receipt, loadpaths, shared parsers and
all selected sources against frozen identities in its retained initial snapshot.
Loaded JSON bytes are matched to that snapshot. The copied proof sources retain
their frozen identity through compilation and recursive checking; output proof
objects are guarded through recursive checking. The final controller runs all
prover commands in its private proof directory, including auxiliary lia output.
Six separately frozen provenance controls reject modified source, modified
controller and an omitted frozen input, under both normal and optimized Python,
before any prover command. These are artifact controls, not semantic negatives.
The five semantic countertheorems are separate checked proof terms.

Replay from this checkout with fresh private output directories:

```sh
python3 -B -O source/tools/research/native_heap_initialization/check.py \
  --freeze source/tools/research/native_heap_initialization/final-freeze.json \
  --output /tmp/fresh-native-heap-initialization
python3 -B -O source/tools/research/native_heap_initialization/guard_controls.py \
  --freeze source/tools/research/native_heap_initialization/controls-freeze.json \
  --output /tmp/fresh-native-heap-initialization-controls
```

The freeze intentionally covers consumed proof/trust inputs, not this prose or
post-trial result manifests. Hashes authenticate identities and are not a Rust
semantics, cross-kernel import, source-validity argument or authorization to
publish. Shared `proof_checks.py` assumption parsers remain unchanged.

The final optimized replay passed recursive checking in 70.9 seconds and
reproduced all four object hashes (the three model modules and assumption
inventory) from the first successful trial. The 15 failed development trials
retain source copies, pretrial freezes, diagnostics and receipt/log hashes;
none is counted as a semantic negative or an impossibility result.
