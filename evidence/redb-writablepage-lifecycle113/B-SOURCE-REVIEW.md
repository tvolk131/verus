# Lane B: original WritablePage lifecycle source review

Decision: proceed with the newly authorized local engineering, but the inspected
engines do not currently establish the original header-to-cache resource edge.
The smallest honest next implementation is a scratch-only, fail-closed capture
of the original temporary/drop origin, without synthesizing any resource effect.
There is no available positive E2/native lifecycle proof to import from the
scalar drop candidate or the affine IR. This is a constructive mechanism finding,
not the historical scope refusal and not a review of lane A's eventual candidate.
No acceptance, native producer, generic Mutex/Arc law, or cell closure is granted.

AUTHORIZATION.md reopens this edge; PLAN113 controls. All18 remain OPEN.
Goal service stays last observed BLOCKED, untouched here.

## Source anchors and existing support

Prefixes: N is the recovered-native/redb/src directory identified in the brief;
D is drop-effects-parent-002; F is D/handoff/source/source; V is
native-leaf-lookup-007/midpoint-runtime-proposal/source/vstd; L is the dirty
checkout. Full paths and whole-file identities are retained in INPUTS.txt and
the identity manifests. Numbered exact excerpts are retained separately.

* N/page_store below means N/tree_store/page_store, never a live integrated
  application. N/page_store/page_manager.rs:1617-1628 contains the unchanged
  chained write, `?`, mem_mut, copy, temporary destruction and Ok.
* F/rust_verify/src/checked_normal_drop.rs:103-123 supports unit, u64,
  mutable references recursively, and concrete unpacked structs whose fields
  are only u64 or &mut u64. This supplies no Arc, slice or Mutex owner support.
  Lines237-301 resolve actual local custom Drop/glue, reject nested field glue,
  and check the normal glue calls the resolved destructor on the original place.
  Lines305-405 reject unsupported calls, branches/cycles and cleanup entry;
  441-473 reject returned values, owner arguments, owner moves/copies and escapes.
  Lines512-600 join original parameter/binder HIR/AstId origins to MIR and demand
  direct lexical owner construction. Lines834,849-879 compare actual lexical
  events and independently captured GHOST/erased-EXEC elaborated operations.
  These checks apply only inside their enforced subset.
* The same file:783-808 requires local unconditional checked Drop, no_unwind,
  mask-none and no nested drops. Lines643-654 generate the actual checked call
  with TwoPhaseBorrowMut, followed at696-697 by a **generated HasResolved
  assumption** for the consumed &mut-u64 fields. That existing experimental
  ownership rule is not a backend proof and remains unadopted. Extending it to
  Arc/Mutex/slice fields would introduce precisely the unproved new rule barred
  here. F/vir/src/resolve_axioms.rs:224-225 suppresses generic owner-to-field
  resolution for Drop datatypes; it cannot discharge the missing fields.
* The frozen review's bool control failed before ownership accounting;
  end-to-end retirement assurance remains open. Its fixtures are not native proof.
* Existing value contracts can express u64 equality/bounds, enum case analysis,
  ordinary Rust borrowing, and a slice copy after obtaining a valid mutable
  slice: V/std_specs/slice.rs:634-639 requires equal lengths and specifies final
  destination bytes. V/std_specs/option.rs:160-165 requires Some for unwrap;
  201-205 specifies take returns the old option and leaves None. These are
  inherited library contracts, not new laws introduced by this lane. They do not
  identify a native allocation, its owner, its lock or an actual cache entry.
  The inspected Option specification has no replace contract. D/staged/source/
  rust_verify/src/fn_call_to_vir.rs:3277-3324 bridges only then_some/map_or,
  not Option::replace. Clearing that scalar issue alone would not prove Drop.
* Arc clone is not entirely unsupported: D/staged/source/rust_verify/src/
  fn_call_to_vir.rs:105-115 models standard Rc/Arc clone as **SMT value identity**.
  V/std_specs/smart_ptrs.rs:41-51 specifies Arc::new/default pointee values.
  None of these establishes strong/weak counts, allocation identity, Arc::get_mut
  exclusivity, nested destruction, or cache return. Do not derive physical
  alias-freedom from equality in that encoding. No adequate get_mut/resource
  contract was found in the inspected interfaces; this is not an exhaustive
  theorem about every possible checked implementation technique.
* L/source/vir/src/native_resource.rs:22-33 requires the default-off feature and
  internal exec checking; :59-72 represents authored abstract token events.
  Its instrumenter and NATIVE_RESOURCE_IR.md limit this to an empty internal
  fixture with hypothetical descriptor authority. No source temporary, Arc,
  lock, borrow, cleanup or Rust Drop producer exists there. It is not E2 and is
  not in either frozen compiler binding. Native/import/export refusals stand.

## What the actual original chain must establish

1. **Reservation and key.** N/page_store/cached_file.rs:881-905 checks alignment,
   selects the stripe from offset, locks it and removes the read-cache entry,
   checking that entry's length. :907-973 either takes the existing write entry
   or inserts then takes a new one. take_value(:95-100) leaves an existing
   inner Option None; an absent map key and a reserved None entry are different.
   A second take of the reserved key panics. The write-buffer-hit branch does
   not itself check the taken buffer length, so equal length for the later full
   header copy is a real cache invariant, not implied solely by the len argument.
   The overwrite=true flag only selects zero allocation on a cache miss; it does
   not zero every reused buffer. Header correctness follows from the full copy,
   after proving length and resource identity, not from assuming zeroed cache.

2. **Returned ownership.** :974-978 moves offset and data into WritablePage and
   clones the selected buffer Arc. The write guard is a local and is released
   when write returns. WritablePage(:24-28) carries no guard or lifetime tying
   the vacancy to its caller. Ownership of the reservation must survive across
   that unlocked interval. The Arc<Mutex<...>> may intentionally be shared; it
   is the data Arc whose uniqueness mem_mut needs. Conflating these Arcs is an
   incorrect resource rule.

3. **Aliases and mutable bytes.** mem_mut(:35-36) calls get_mut().unwrap(),
   without clone-on-write. read(:659-675,686-709) returns Arc clones, and may
   install clones in the read cache. Removing cache references does not retire
   external Arc/Weak aliases. A read-cache copy in the local `existing` variable
   is dropped on write's return, but returned read handles may survive. Safe
   Rust prevents a live `page.mem()` reference overlapping a mutable borrow of
   that page; this does not eliminate independently owned Arc aliases from
   earlier reads. Native get_mut must succeed only for actual unique allocation
   authority, including the Weak restriction. Its None case is a panic here.
   This review does not claim ordinary header callers actually retain an alias:
   page_manager.rs:868-874 reads the header with read_direct, not cached read.
   That helps the specific caller argument; it is not a global cache history
   invariant or an engine proof of it.

4. **Temporary and generated glue.** page_manager.rs:1623-1626 creates a
   WritablePage temporary through the successful `?`, borrows its bytes and
   copies the actual header encoding. On normal Rust execution the statement's
   temporary is destroyed at the semicolon, before Ok(:1628). The copy's borrow
   ends before destruction. This is a source semantic obligation; this lane did
   not capture actual MIR/glue for that original method. Replacing the chain
   with an explicit drop or a user-named scalar owner changes the evidence.
   Unlike the frozen subset, the original has a Result return, branching,
   fallible calls, a returned owner temporary, Arc field drops and guard glue.
   A larger normal-drop switch cannot silently grant support for these.

5. **Return identity and actual cache.** WritablePage::drop(:41-45) locks the
   same buffer, then calls return_value with the stored offset and a clone of
   the current data. return_value(:91-92) requires an existing None slot. Normal
   return fills it with that data. The temporary mutex guard is released at
   the destructor statement's end. Actual field glue later drops the page's
   buffer/data Arc fields; the cache's inserted clone must remain. This needs
   nested concrete glue/clone/lock provenance, not just a value copy theorem.
   Correct offset, stripe, cache instance, allocation, length, byte authority,
   activation and reservation history must all remain associated.

6. **Exposure to flushing.** lru_cache.rs:85-90 maps the underlying table's
   iterators to every key/value; it does not filter None. cached_file.rs:494-495
   iterates and unwraps each buffer. Thus all entries encountered by a full
   flush must be available, including unrelated pages; returning the header
   alone does not establish that condition. :500-518 later drains them into
   the read cache and clears the stripe. Lowest-priority eviction(:103-117)
   reinserts reserved None entries instead of evicting them, while full flush,
   cancel(:769-775), discard(:529-540), and shrinking(:559-576) demand stronger
   absence-of-outstanding-page conditions. Commit's comment and debug check at
   page_manager.rs:1806-1809 describe the intended discipline, not a verified
   native history. A mutex does not enforce it while the WritablePage lives.

7. **Frames and the real map.** lru_cache.rs:76-82 sets the target second-chance
   atomic even in get_mut; a full metadata-unchanged postcondition is false.
   Payloads of other keys can be framed for just take/return only after actual
   table projection/borrow correspondence. The complete write may evict other
   entries/read caches and write them to the backend(:927-958), so a universal
   unrelated-cache unchanged frame for write is false. State touched by those
   effects needs an explicit footprint and actual error-prefix relation.
   fast_hash.rs:13 uses HashMap<u64,_,BuildHasherDefault<FastHasher64>>; it is
   not RandomState. D/staged/source/vstd/std_specs/hash.rs:243-254,384-400,
   656-664 conditions map/iterator effects on builds_valid_hashers<S>. The u64
   key axiom does not supply the custom builder premise. Assuming it is forbidden.
   The no_std BTreeMap substitution at fast_hash.rs:43-50 is a different cfg,
   not permission to replace the original std path to obtain easier contracts.

## Poison, panic, error and interference outcomes

sync.rs:4-12 selects std locks unless redb_no_std. The ordinary path has poison:
lock().unwrap() in write or Drop can panic. Holding a lock serializes access to
its protected cache only while the guard lives; it says nothing about a prior
or future guard's logical history. The no_std spin locks(:87-101,160-162) never
return poison, but their UnsafeCell/atomic safety and progress are separate
unproved semantics. V/std_specs/atomic.rs:5-7 explicitly excludes stored-value
reasoning. Neither cfg supplies an adopted lock law here. Local HeaderGuard
holds an in-process mutex (page_manager.rs:540-543,849-854), but its Rust type
does not prove which cache aliases/operations are covered by that discipline.

* write can return Err before constructing a WritablePage: pressure writeback
  at929-943, or the non-overwrite backend read at969. For this header call the
  latter allocation-miss read is bypassed by true, while pressure writes remain.
  The read entry may already have been removed and byte counters changed.
  flush_lowest_priority(:831-840) reinserts the failing buffer but previous
  successful evictions remain effects. Err is not an unchanged-cache theorem,
  and there is then no successful page temporary whose Drop returns the header.
* Assertions/unwraps, allocation failure, a nonunique Arc, length mismatch or
  serializer failure can prevent normal completion. If unwind cleanup runs,
  the temporary page may return its old bytes; it cannot promise the encoded
  header when the copy never completed. If cleanup Drop also panics, execution
  may abort. No unwind, cleanup or allocation-progress proof is present here.
* return_value first finds the key then performs replace before checking its
  old result. A missing key panics; an occupied key is **already replaced** when
  the is_none assertion fails. A wrong key naming another reserved None entry
  can pass both checks while returning bytes to the wrong page. Assertion success
  therefore does not establish return provenance. Panic under the std guard may
  poison that stripe; a success-only cache law would conceal this difference.
* During a vacancy, cached get unwraps None; full flush unwraps None; cancel
  removes before asserting Some; discard and shrink can panic. Other operations
  are not safe merely because their individual mutex acquisition succeeds.
* flush_write_buffer(:494-496) returns at a failed file.write before draining
  that stripe; earlier stripes may already be drained and earlier backend writes
  occurred. sync_data is called only after flush_write_buffer succeeds(:596-599).
  Availability/byte identity at iteration is not file-write, sync or durability
  correspondence. Those downstream OPEN gates remain distinct.

## Discriminating controls: required classification, none executed here

All rows refer to original statements, preserve the positive's scope/runtime
and require a byte-identical restoration. Predictions are not earned control
credit. A failure before the intended obligation is a frontend/setup refusal,
even if its text mentions ownership. Never report it as a semantic-negative proof.

| Source control | Correct semantic result in a future adequately modeled original-body check | Current inspected support / admissible credit |
|---|---|---|
| At drop:45 return to a different same-stripe key with both keys genuinely reserved None | Must fail offset/reservation association and the changed other-entry frame; runtime assertions can all pass. With an absent/occupied wrong key, panic is a different, less isolated case. | Original Arc/Mutex/Result/temporary/glue path is outside the frozen subset; expect refusal before this semantic check, exact first diagnostic unobserved. Zero semantic credit. |
| Remove only the actual return_value from Drop:45, retaining automatic destruction | Must fail availability and exact header entry at the statement boundary; the slot remains None and later full flush can panic. Consuming an abstract token alone must not satisfy return. | Same original owner/glue refusal. An ordinary toy Drop fixture cannot substitute. |
| Return same-length wrong bytes at Drop:45, choosing an encoding that differs from them | Must fail exact payload identity even if key/vacancy checks pass. E.g. a zero-filled replacement for a chosen nonzero encoding must not pass; its allocation effects must also remain explicit. | Additional unsupported primitives may cause an earlier refusal; no isolated semantic credit unless the positive and mutant actually reach the same payload VC. |
| Retain an owned Arc clone obtained by native cached read:664/675/691 through mem_mut:36; separately retain Weak | get_mut returns None and unwrap panics; a total normal-success contract must fail its uniqueness/no-panic obligation. A contract including panic must preserve that outcome, not invent successful copy. | Arc clone value identity is insufficient; get_mut/count/resource semantics and original path unavailable. Frontend refusal is expected, not proof that aliasing is harmless. |
| Hold `let bytes = page.mem()` across mutable mem_mut and use bytes afterward | Rust borrow checking should refuse overlapping shared/mutable loans. | Frontend borrow refusal is the intended result, not a resource semantic-negative. This differs from the owned Arc case, which safe Rust can express. |
| Allow full flush/cancel while a reservation is outstanding; separately poison the std mutex before Drop | Must reject a blanket normal-success claim or explicitly account for panic/poison/partial mutation. A valid positive needs proven caller history, not assumed success. | No normal-drop cleanup/lock model. A diagnostic refusal earns no panic/history correspondence. |

## Implementation boundary and one concrete next code action

No whole original-body positive can presently be justified by the inspected
support. Existing slice/Option/value checks are useful obligations once a valid
native reference has been established; proving them on a supplied buffer would
not advance this missing edge. Checking abstract affine events likewise does
not manufacture the source producer. The custom hasher premise and missing
Option::replace specification are additional dependencies, not opportunities
to introduce a shadow cache or new primitive law.

**Next code action:** in a fresh scratch source candidate based on
F/rust_verify/src/checked_normal_drop.rs, extend the existing authoritative
`capture`/`rewrite_block` origin representation to **record and explicitly
decline this original method-chain temporary**. Retain its actual MIR local,
full place, HIR/AstId origin when present, resolved concrete Drop/glue identity,
and normal versus cleanup successor; do not turn a missing user binder into a
guessed name/span join. At the Arc/Mutex/nested-field or unsupported-origin
boundary return a specific error before any generated call, HasResolved effect
or exported result. Preserve ordinary admission and the existing strict subset.
The useful source delta is this exact producer-capture/refusal mechanism, not
adding Arc to supported_type or asserting the desired pending-buffer state.

This permitted E2-front-end slice introduces no law. It remains an **unbuilt
source candidate**; validation needs separately permitted executable evidence.
No automatic review chain is proposed.

## Authentication, counts and handoff limitations

First real clock: 2026-11-08T14:30:36Z; own absolute cutoff14:40:36Z, earlier
than the parent15:17:06Z. Fresh root is the directory containing this report.
No parent/native/frozen source edits; SOURCE-DELTA.json records the empty
implementation delta. All authored scratch edits used apply_patch. No runtime,
toolchain or cache copied; no subdelegation, network, Git writes, compiler,
verifier, solver, tests, builds, --compile or native application execution.
Verifier/proof applications: **0**. Semantic controls executed: **0**.

Parent entry reconciliation at14:34:31Z: **1483 records, 1481 exact matches,
2 expected GOAL/STATUS after-hashes, 0 unexpected differences**. Full per-path
results are in parent-baseline-comparison.json. The two updates are exactly
DOCUMENT-RECONCILIATION.json; none is silently treated as an input corruption.
Frozen-binding-comparison.json authenticates **39/39 selected runtime/provider/
source records**, including both distinct vstd.vir/libvstd.rlib sets in place.
None was executed/imported; no set was mixed. Selected midpoint vstd source
hashes match recovery021's declared inherited source map. This is integrity,
not a reconstruction or proof of compiler-source/core correspondence.

Entry identities follow initial reads; no atomic/first-read snapshot is claimed.
Wrong-prefix failures survive in entry-identities.json and FAILURES.json.
Preexecution14:34:31Z is a utility phase. Initial logs survive in the transcript,
some truncated; later logs/excerpts survive locally.

Tools, resolved Python/llvm-otool and Python modules are hashed. OS shared-cache/
harness closure and initial Git shim selection were not fully traced. Bootstrap
is not proof-grade preexecution authentication. No verifier closure was exercised.

Disk samples all exceed16GiB; they are not continuous enforcement. SEAL.json
records final size/time/disk/file identities. Digests establish integrity only.

Native-path preservation remains FAILED; native Git continuity UNAVAILABLE;
compiler/source correspondence UNESTABLISHED. The inherited conditional,
unadopted TCB stays unchanged. Content recovery is not historical-path
restoration. S0/Infer/full-scope/import/export and all excluded pipelines remain
unchanged. Handoff only; not lane A acceptance and not final native acceptance.
