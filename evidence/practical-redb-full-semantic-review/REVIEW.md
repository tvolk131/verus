# Independent review: scoped ACCEPT

No blocking finding for the original R3 full semantic guarantees delivered at worker `423a850421951c7c9f4e5e2d6ddb55e024dfeecb`, integrated at `96447dcbdb390fd4308b90b5395a683bce6aec9b`, and reviewed at fixed parent `86fb0f9ce5a6fc5b0b812ca4a99022fe32d3a5ab`. The worker REDB subtree equals the fixed parent subtree. This is new acceptance of both full semantic compositions; the earlier checkpoint review accepted only PARTIAL operational traces.

## Source, domain and actual-call composition

The frozen source comment requires free blocks to occur at their largest available order. Original `corrected.rs` formalizes that as geometry, cross-order zero-block separation and maximal buddy merging. `domain.rs:11` retains exactly that `valid` formula; V2 `canonical` adds only the corresponding abstract shape and those same clauses. Neither predicate includes a transition conclusion, successful allocation, free-cover existence or final-state validity. Token comparisons confirm unchanged width/representation/geometry/flag/valid/geometry_frame and all four free_pages/requested/legal_block/legal_free definitions.

The replay's inherited R2 proof-erasure/token projection authenticates twelve bitmap methods against pinned bitmap source and seven additional original methods/helpers against frozen source bodies. I inspected the actual allocator source and the complete co-located V1 provider, traces and frames. Runtime branches, assertions/debug assertions and updates survive unchanged; V2 adds proofs and wrappers, not replacement algorithms. Frozen source-file and provider hashes were independently checked before replay, beyond the V2 script's own inherited-file checks. All ten inherited V1 hashes match. The frozen original failed `corrected.rs` is preserved, not relabeled as directly proved.

`record_alloc_semantic` (`semantic_v2.rs:592`) calls checked `BuddyAllocator::record_alloc_inner`, whose unchanged V1 contract supplies exact alloc_trace, layout/representation, geometry/storage frames and rejection state_eq. `free_semantic` (`:617`) likewise calls checked `free_inner` and obtains free_trace and frames. `actual_model` (`:245`) derives abstract canonicality and exact page correspondence from original valid alone. After interpretation, `model_actual` (`:264`) derives original valid(final) and page correspondence from proved canonicality plus the original body's checked representation and frames. Its premises do not smuggle in valid(final) or the desired effect.

These are conditional modular guarantees about checked original bodies with original valid (plus original legal_free for free) discharged at the wrappers. They are stronger than abstract-only lemmas. They do **not** change the original methods' operational annotations, establish a public imported caller API, verify the original native crate integration, or prove native memory ownership. The aggregate checks dependencies and clients together; successful VIR exports are not a native crate integration test.

## Mathematical adequacy

`remove_block` (`:132`) uses canonical separation to establish exact page difference and preserved canonicality. `add_block` (`:167`) uses legal in-range disjointness and terminal buddy unavailability to establish exact union and preserved maximal merging. Its extra premises are proved at actual composition sites.

Allocation completeness is substantive proved work: `dyadic_nested` (`:402`) inducts on scale difference; `cover_ancestor` (`:426`) inducts downward over request order. A whole legal block either already has a zero bit, lifts a covering point at order zero, or recursively covers its two complete children. A larger child cover lifts to the parent; two covers confined to the children would give two zero buddies, contradicting canonical merging. Floor geometry supplies complete children even at ragged boundaries. `parent_whole` (`:491`) derives legal, wholly-free parent and a strictly higher available order from that coverage, rather than assuming completeness. `alloc_interpret` then uses the failed-parent contradiction or exact parent removal, the lower-order trace frame and legal disjoint sibling addition. No free-ancestor premise was added to valid/canonical.

For free, `parent_partition` proves parent legality and the exact disjoint child partition. `free_interpret` (`:557`) removes the actually free buddy, derives parent disjointness from that removal and the original legal-free interval, and only then recurses. Ancestor quotient transport gives the returned represented block containing the request. Final canonicality supplies the maximum-order/buddy-unavailable stopping condition. This is maximal merging within the configured maximum and legal geometry.

The wrapper results cover valid preservation, geometry/storage frames, allocation success **iff** legal and wholly free, exact removal on success, logical identity on rejection, and legal free's exact addition and interpreted merged block. `semantic_roundtrip` checks arbitrary valid legal wholly-free requests through both actual-call wrappers, deriving legal_free after allocation and restoring exact free pages. It does not promise restoration of opaque Vec identity or all representation values after roundtrip.

Rejection state_eq deliberately means equal len/max_order and equal complete free@ sequence. The frozen `*final(self)==*old(self)` formula demanded opaque Vec-container equality unsupported by the existing indexing/writeback interface. The documented correction preserves the logical allocator state without a new Vec identity axiom. Acceptance is of that correction, not the literal opaque equality in the failed file.

`used_pages` is explicitly `[0,len) \ free_pages`. Finite range/filter, interval cardinality, exact effects and checked set cardinality lemmas give the disjoint partition and the ±2^order conservation statements. This is semantic complement bookkeeping, with no assumed ownership ledger. Neither a weighted per-order block-count identity nor the executable `count_free_pages` loop is proved.

## Domain and non-vacuity

Allocation wrapper inputs keep arbitrary u32 page and u8 order; invalid orders/indices reject. Legal orders are all 0..max_order, with max_order anywhere 0..20. All u32 lengths, including zero, remain. Geometry is floor(len/2^order); no power-of-two, fixed word count, exact storage capacity or positive-length restriction is introduced. R2 validity admits multiword/padded trees and extra storage; original bitmap updates and all their required contracts are checked again.

The invariant has concrete witnesses: all-unavailable leaves with consistent all-one summaries/padding satisfy valid for any admitted geometry, and admit any in-range legal free. Zero-length trees can have a single zero-length level with no required data words; valid holds and every allocation rejects. At len130/max7, representing only order7 block0 and order1 block64 as free yields the complete free interval [0,130), including its ragged tail. Actual bitmap witnesses use leaf zero bits at those locations, correct summary bits above them and one padding bits. An interior witness with only order4 block2 free satisfies the inherited split/merge example. These establish meaningful satisfiable premises by inspection of the definitions, not a constructor theorem.

The retained len130/max7 repeated-operation and interior clients remain checked. The general semantic_roundtrip and interpretation theorems quantify over the full domain; examples do not prove those universal results. Original actual region bounds are included; surplus admitted u32 states do not narrow real inputs. Runtime allocation ownership beyond interval disjointness and constructor establishment of valid remain unproved.

## Fresh reproduction and controls

Ran exactly:

```sh
python3 -B -O source/tools/research/practical_caller_transitions/redb/replay_v2.py --output /tmp/practical-redb-full-semantic-review-86fb0f9c/full --full --controls
```

`--full` is necessary. Fresh ordinary and Infer positives each return **96/0**. All six controls return **96/1**, failing respectively only `wrong_free_effect`, `wrong_free_merge`, `wrong_free_domain`, `wrong_alloc_accept`, `wrong_alloc_effect`, `wrong_overlap`. Domain fails the caller precondition; the others fail their intended assertion. The overlap control's premises are satisfiable (e.g. n2/m1, only order0 block0 free); adding free parent0 violates separation. No control is a frontend/resource outcome or an allocator counterexample.

I checked full Rust identities, modes and corresponding SMT entries for every body, not just totals: **25 exec / 47 proof** in both positives, including 19 original methods/helpers, three inherited clients, two semantic wrappers and the general roundtrip; 15 inherited plus 32 new proof bodies. Negatives select the same support plus their intended control, and only that control has unsuccessful SMT entries. Failed crates certify zero selected bodies; healthy supporting SMT results are not selectively promoted to successful crates. Each failed Infer run seeds a positive VIR export before compilation and removes it on failure. Positive ordinary/infer VIR artifacts exist; ordinary's Infer-specific publication marker is false as expected. All runs use `--no-cheating`, full selected-crate checking and normal lifetime/trait/WF/mode/body/erasure checks, without check-disabling flags, extra rlimits or local trust admissions.

The accepted78 receipt and all eight tool hashes authenticated at both `/tmp/practical-combined78-5ecea5fe-core/bundle` and the frozen V2 private bundle. I made and rehashed another private copy at `/tmp/practical-redb-full-semantic-review-86fb0f9c/bundle`. The unmodified replay uses its explicitly frozen immutable completed private runtime `/tmp/redb-semantic-v2-3fa15230/bundle`, also authenticated; no runtime override or repinned source was introduced. All caches/output are fresh private paths. Parent receipt `c50b04f62d31619dbe94b7590f3e38edc6ab7f5dd52a84551d5f3bdb16463cfa` was authenticated as supporting evidence only.

## Costs, historical failures and trust limits

Independent static counts and all per-definition/helper byte/line measurements match COSTS_V2: **17 definitions** (four copied originals), **32 new ordinary manual helpers**, **125 assertions**, **15 polynomial assertions**, 10 explicit triggers, four wrapper proof blocks, two snapshots; semantic source646 lines/30,302 bytes. Nine helpers concern representation/readback/transport and23 concern semantics. Controls48 lines and roundtrip25 lines, their contracts/snapshots/assertions, inherited annotations and the charged replay remain additional work. No authored annotation is removed. Infer generates zero helpers; its incidental174 checked set-pair calls and inherited single word template are rechecking assistance, not demonstrated new benefit or common automation. Exact per-function assistance is recorded in RESULTS.

This review launched eight compiler runs, summed compiler wall113.154s; positives took8.107s and14.096s. These are single reproduction samples, not benchmarks. No rebuild or broader matrix was needed. All18 historical V2 epoch receipts and their explicitly named logs authenticate, including initial resource exhaustion, trigger/frontend failures, helper failures, partition/composition resource failure, rejected opacity syntax, coverage failures and later staged successes. They retain their original classifications and source/log artifacts; no historical failed diagnostic is repaired or promoted. A host bwrap setup failure and a scratch cost-extraction mistake are separately recorded as review setup diagnostics, not proof failures.

Trust remains the accepted engine78, Rust/Verus frontend and solver, existing vstd arithmetic/finite-set and Vec/index/mutable-writeback/word contracts, and the fully rechecked inherited bitmap proofs. No new trusted axiom, callback summary, clone/ownership law, private-representation admission or native correspondence bridge was found. Construction/resize/search/serialization, weighted counting, crash consistency and whole allocator/database correctness remain unproved. Future Fedimint initializer work is outside this review. The full caller goal remains active; HMAC/native remains **DEFERRED AND UNACHIEVED**, native correspondences zero and historical44+3/101/full HMAC suffix unchanged.

## Artifact binding

INPUT_HASHES.json contains fixed commit/tree/blob objects, reviewed file SHA256s and all runtime tool hashes. RESULTS.json contains exact checked/failed inventories, costs, historical epoch bindings, fresh log hashes and exports. Fresh raw logs, aggregate source and receipt remain at the private output above. No implementation files changed; evidence-only scope milestone is `f41e4625`.

- Replay SHA256: `02b39a4f05d93e2fde2f51bbb5384947079a3480bb9b00d574fd80a277f2a10f`.
- Fresh aggregate SHA256: `7380d8cea929172721838357f66da496aa10e234c8c77c6892fc48a26d908124`.
- Fresh receipt SHA256: `735154ceed22ecdd1467cd20cd4693fb2a8d6a66f0ce625cb41ad9a2d4c67535`.
- Ordinary VIR SHA256: `3290b327eace943d6b779da9b3dc495369e6cc2bb01b2bca4277473b454560d6`.
- Infer VIR SHA256: `47309cd33ec38cefe7ce306a786eaaab8a74b73e347361a68dccfb1f9ecee96d`.
- Accepted runtime receipt SHA256: `3e04cec7545fd1b44781ba6e8c2447e2109339a9f5257ef7b605d81896a16452`.
