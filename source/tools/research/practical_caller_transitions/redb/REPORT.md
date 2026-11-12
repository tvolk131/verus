# R3 first eligibility / baseline / value checkpoint

**Partial ordinary caller proof, full selected target unproved (0/2 full callers).** This is the requested first checkpoint, not milestone completion. No production/common engine/vstd change, replacement caller, push, PR, delegate, parent/sibling live read or new trust bridge. Parent independent review remains required. HMAC/native is **DEFERRED AND UNACHIEVED**.

## Freeze, source and domain

Pre-trial freeze commit **15ccd29f** follows charter 2914c073. FREEZE.json pins REDB c82ab865, exact buddy/bitmap hashes, source items, both accepted providers, all eight combined78 tool hashes and the receipt. Receipt and tools were authenticated before making the complete independently copied `/tmp/caller-redb-2914-zzy6ugq6/bundle`; no ignored root binaries or historical75 runtime. Full checking and `--no-cheating` throughout.

The original `record_alloc_inner` / `free_inner` bodies, two arithmetic helpers and mutable order accessor remain token-for-token after the existing R2 proof projection. The actual BtreeBitmap get/len wrappers are newly checked, not credited to old R2. Accepted old twelve executable bodies and eight manual proof helpers are included unchanged; their annotations are inherited cost. Replay reuses the old token-fidelity and exact native-scope checks against pinned source. Private co-location checks the entire closure, so no new imported interface/callback/representation fact is assumed. Existing imported vstd contracts and normal Verus backend trust are inherited; no native Rust correspondence is claimed.

All actual orders 0..20, arbitrary u32 region lengths (including actual <=2^20 lengths), multiword leaves, R2-valid padded tree heights, zero-length states and incomplete tail buddies remain admitted. `geometry_to_layout` proves every frozen geometric state satisfies the operational layout; `valid_admission` includes every frozen canonical valid state. No single-order/word restriction. The local half-length recurrence is not an assumed stronger domain.

Source `debug_check_consistency` justifies non-overlap and maximal buddy merging. A **zero** bit represents a free block; a **one** bit is not an allocation-ownership predicate. Legal free requires its whole aligned in-range interval to be allocated, justified by page_manager's live allocation tracking intent, not by the weak one-bit debug assertion. Construction/resize validity, allocator search, serialization, crashes and tracking collections remain outside scope.

## What checks, and what does not

Delivered manual caller contracts establish:

- Exact stored logical bit-set transition under `alloc_trace` / `free_trace`, authored recursive models checked against the actual executable branching and R2 bitmap mutations.
- Rejected allocation preserves len/max_order and the entire `free@` sequence of bitmap values; successful allocation is in range. The overstrong frozen opaque-container equality stays failed in corrected.rs; the explicit correction to logical state equality is documented in MANUAL_SCOPE.md.
- Original arithmetic/bounds/debug checks, all bitmap validity/shape/storage frames, and unchanged lower-order bitmap values; free returns the exact order selected by the checked trace and stays within requested/max order.
- Valid-domain admission and repeated-operation controls in the final replay. These controls establish stored-bit effects; intermediate canonical invariant preservation is still unproved.

This is a useful implementation-level operational summary, **not** exact interval allocation/free correctness. Neither caller yet proves preservation of the semantic allocator invariant, successful allocation iff the requested pages are wholly free, page-set removal/addition, weighted page conservation, or the returned block's interval interpretation. The source count_free_pages weighted unset-count loop is not proved by R2's set-bit count theorem. No allocation ledger or false ownership premise is added to make these follow automatically.

## Baselines and retained failures

Private evidence root: `/tmp/caller-redb-2914-zzy6ugq6`. See RESULTS.json for receipt/log hashes, exact identities, timings and literal costs.

- `baselines2` / `baselines3`: accepted combined fixture ordinary17/3, Infer20/0; accepted manual provider25/0 in both modes. These are reruns of accepted component results, not new caller progress. The provider prefix in manual.rs is byte-identical to accepted redb_tree/manual.rs with its empty harness omitted.
- `full`: corrected full-property baseline ordinary31/2 and Infer31/2, both **RESOURCE_FAILURE**. Diagnostics include validity transport, recursive legal-free premise, successful interval effect and rejection equality. Resource outcomes are not clean semantic negatives or evidence of native unsupportability.
- Original `baseline.rs` is immutable. Initial `baselines` launch failed rustup discovery; `baselines2` caller Rust frontend rejected finite Set::new's Option return; `baselines3` rejected an untriggered maximal-buddy quantifier. Corrections were documented/committed before subsequent trials (e79dd371, cefdc0e2). All remain retained, separate from semantic failures.
- `manual1` has proof-only numeric cast frontend failures; `manual2` clean proof failures; `manual3` / `manual4` resource failures. `manual5` checks both callers ordinarily and with Infer. No-new-caller-annotations baseline and authored manual assistance are not conflated.
- `domain` / `checks1`: optional converse layout-to-geometry induction checks ordinarily but exhausts Infer resources, including a local rlimit(20) diagnostic. Retained in reverse_diagnostic.rs, not an implicit prerequisite. Forward admission is checked and sufficient for original-domain coverage.
- `final` is a controller metadata failure after successful ordinary verification/export: inference `interface_written` is false for ordinary `no_inference`, despite a native VIR file being written. Final controller applies the inference publication helper only to Infer exports. `final2` is the earlier successful operational controls matrix.
- The four-level interior client in interior_diagnostic.rs remains resource-limited in both modes (final3/final4, including rlimit20), with original target retained. No broader layer was started. Extra client/control failures cannot establish or invalidate generic caller proof completion by themselves; they remain disclosed.

## Exact next sequent and value decision

Let a satisfy the frozen `valid` invariant; let b have the checked geometry frame and the exact checked bit trace for the same request. Prove the **semantic interpretation lemma**, without assuming its conclusion in valid:

`alloc_trace(flags(a), sizes(a), p, o, max) = (flags(b), r)` implies `valid(b)`, `r <==> legal_block(a,p,o) && requested(p,o) subset free_pages(a)`, and on success `free_pages(b) = free_pages(a) - requested(p,o)`.

For free, add `legal_free(a,p,o)` and prove `valid(b)`, exact union with the requested interval, and that the returned-order block represents the merged free interval. Then derive finite cardinality changes/partition and, separately, connect the original weighted count loop. The exact original failed full bodies remain in corrected.rs; no assumption/export supplies these missing facts.

Main remaining cost is semantic interval/buddy/non-overlap reasoning. Ordinary proofs already handle elementary slot borrowing with zero local annotations (consistent with parent's immutable P0 audit); three authored transport lemmas and explicit caller snapshots/blocks handle current nested/recursive frames. Potential reusable frame work is measured in COSTS.json, but no matching cross-lane/common need or annotation reduction is established. Infer generates zero inferred helpers, removes zero authored annotations, and adds only incidental checked assistance. No engine/common work is requested before parent assesses this value checkpoint.

The investigation stops at this bounded checkpoint with these unsupported/unproved cases retained. The earlier practical milestone remains historical accepted input, not resumed HMAC/native work.

## Final checked inventory and literal cost

`final8` passes both positives **46/0** and all three intended single-function semantic/safety negatives **46/1**; every negative removes its seeded Infer VIR export. Full normal checking and no-cheating remain enabled. Positive scope is exactly **22 executable bodies** (19 original methods/helpers plus three scenario clients), **15 proof bodies** (8 inherited R2 + 7 new), and **zero inferred helpers**. Exact identities are in RESULTS.json. No imported caller/API facade is delivered, so no new import-contract correctness is claimed; private provider bodies are all checked in the same crate. Both native positive exports exist; the inference publication flag is asserted only for Infer. No disabled bodies or selective-success accounting.

New authored work: **13 specification definitions** (9 operational/frame/model definitions plus 4 copied full-domain/interpretation definitions), **7 proof helpers** (`state_transport`, `edit_frame`, `frame_trans`, `valid_admission`, `width_positive`, `geometry_to_layout`, `order_half`). The 217-line new manual section includes original executable code as well as contracts: 11 proof blocks, 4 ghost snapshots, 25 assertion statements and 2 reveals. Domain/admission is49 lines; controls90 lines including their negative bodies. COSTS.json records literal per-definition/helper/contract bytes, lines and lexical tokens; the 15,237-byte inherited provider prefix is unchanged and excluded from new-cost counts. Delivered positives need no increased rlimit. Diagnostic-only rlimit20 annotations remain visible in retained failed fixtures.

`get_order_free_mut`, BtreeBitmap get/len and next_higher_order have zero local proof annotations. Buddy parity needs one checked bit-vector assertion. The final interior control required explicit half-length instantiation calls and nested Seq/Set extensionality; final5/final6 resource failures and final7's clean final-equality assertion failure are retained. The one-level interior control now checks: request order3/index5 splits an order4 block, then free merges back to4, restoring exact flags in a len130/max7 initially valid state. The four-level interior client remains unproved, as does general intermediate canonical validity.

Final ordinary solver run: **2,305ms**, **8,669,371 rlimit units**; Infer: **2,964ms**, **19,145,526 units** (single local samples, not a benchmark). Per-function costs are retained. Infer adds pair calls20 in inherited finish_mutation and12 in edit_frame, one inherited word sequence template in U64GroupedBitmap::set, and alternate patterns for storage_frame/geometry_frame. All authored annotations are still present and ordinary verification already succeeds; **zero annotation reduction / no causal automation benefit** is established. The added half-length instantiation helper is an ordinary proof, not an engine feature. Parent P0 evidence is consistent with the zero-annotation accessor result and was not consumed as an assumed allocator theorem.

Replay: `python3 -B source/tools/research/practical_caller_transitions/redb/replay.py --bundle /tmp/caller-redb-2914-zzy6ugq6/bundle --output <fresh-private-path> --phase manual`. `--phase full` reruns the retained full-target baseline; default baseline also checks both accepted providers. Final static checks reauthenticated all delivered source hashes, original allocator fields and lane-only diff; `git diff --check` passes. Proof checkpoint commit: **1223e315**. Parent should independently review this partial artifact and decide whether the missing semantic interpretation warrants another bounded assignment.
