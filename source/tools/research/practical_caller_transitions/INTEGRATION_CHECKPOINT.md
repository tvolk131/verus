# Caller integration requirements — before application results

This is the parent's source/interface audit, not a proof result. Base2914c073; accepted component sources and original application pins remain those in WORKER_BRIEFS.md. Neither worker's unfinished checkout is an input. Failed initial path lookups were setup mistakes, not proof failures.

## What the existing checked interfaces establish

- `common_set/r2-combined.rs` in the previous milestone checks twelve executable bitmap methods. `BtreeBitmap::set/clear` expose exact leaf-set/count mutation and restoration of summaries, lengths and physical path frames. `get_level_mut` exposes the actual Vec update on return from a borrow. These contracts do not establish allocator consistency across different orders.
- Actual `BtreeBitmap::get/len` at upstream bitmap.rs:29/33 are additional caller dependencies. The old twelve-method inventory includes the **grouped-word** get/len instead. The original tree wrappers delegate through checked get_height/get_level/grouped observations and must be checked in the new dependency closure. They cannot be advertised as previously certified bodies.
- Fedimint's `common_set/f2-reduced.rs` checks the numeric NumPeers methods and finite-universe quorum consequences. Its universe cardinality, signer subsets and fault bounds are explicit theorem premises. `policy_client` is an observation client. Neither proves what `NodeMap::iter().count()` or an actual signatures map denotes.

## REDB obligations that cannot be hidden in the component contracts

Source buddy_allocator.rs:29 says free pages occur at only one, largest order. `debug_check_consistency` checks no base page free at multiple orders (295ff), and maximal buddy merging below max_order (316ff). `count_free_pages` (159ff) weights each order's unset-bit count by its block size. This is different from the old R2 allocated-bit count.

Before accepting caller results, inspect:

1. The invariant's connection to real geometry, logical lengths per order, padding, order bounds and partial final blocks. No one-word/one-order/fresh-state restriction replaces the real valid domain. The source MAX_MAX_PAGE_ORDER definition is in page_manager.rs:51; connect any chosen numeric bound to the actual constant rather than guess it.
2. The meaning of a true bit: it may mean an allocated **or split** block. A true bit alone is not a complete semantic justification that freeing an entire higher-order block is legal. Expose the actual valid-free/ownership precondition and justify it from the intended caller domain.
3. Allocation success removes precisely the requested base-page interval from free capacity, preserves represented-block consistency and non-overlap, and accounts for sibling changes while splitting. Failure preserves logical state even after a recursive search; proof of the helper calls alone is insufficient.
4. Free success adds precisely the legally released interval; merged return order is in range and correctly describes the maximal represented free block. Other represented pages, allocator geometry and bitmap storage frames remain correct.
5. The selected original caller bodies, wrappers, recursion decreases and genuine bitmap observers are all checked. Definitions of expected post-state may not recursively assume the target algorithm's own correctness or replace its executable implementation.

Construction/resize/search, implementation of count_free_pages, recovery/serialization and crash consistency are not silently included. A semantic weighted-capacity theorem is labeled separately from verification of that executable counter.

## Fedimint obligations that cannot be supplied by abstract quorum algebra

Actual Keychain::is_complete first counts a NodeMap iterator, then calls all over the iterator. Actual validate_signed_session_outcome first checks exact signature-map length, constructs a keychain and header, then verifies every entry. The root dependency is package fedimint-aleph-bft0.36.0 (Cargo.toml:127), not a guessed unversioned NodeMap implementation.

Before accepting caller results, inspect:

1. Concrete distinct signer membership/cardinality and its connection to the selected peer universe. Map length or an iterator count is not itself a proof that all entries name known peers. Keep the actual indexed/container representation and initialization domain.
2. Count-guard rejection and accepted-count consequences must belong to the original checked caller, not a rewritten conditional wrapper or an arbitrary count parameter. A partial guard result remains partial.
3. No cryptographic theorem is required here; that exclusion does not create an assumed `verify` callback, a new foreign summary, an opaque collection law or a header/keychain stub. If existing supported contracts cannot close the real dependency chain, record unsupported full validation and preregister a real useful replacement before its trials.
4. Any positive bookkeeping result identifies exactly which original runtime methods are checked and which larger admission guarantees remain open. Imported previous F2 theorems must consume facts actually established by the new code, not restated semantic assumptions.

## Common value and acceptance decision

Existing-model eligibility audit (source inspection, no new proof claimed): `vstd/std_specs/btree.rs:175` supplies BTreeMap iterator cardinality, key/value coverage and duplicate exclusion conditional on `key_obeys_cmp_spec`; :492 supplies executable len's map-view observation through the same key law. For an ordered key this law means `laws_cmp::obeys_cmp`, so merely retaining a runtime Ord derive does not discharge it. `std_specs/iter.rs:145` supplies all's predicate-result relation when the iterator laws hold. Native count at :221 additionally requires `native_count_supported`, a present decrease and its usize bound. `iter_adapters.rs:16` starts the count marker at slice iterators and closes it through specified native adapters; arbitrary next laws do not establish it. These are existing trusted library interfaces, not newly checked native bodies. The checked alternative executable `vstd::iter::count` is a different algorithm and cannot replace the original `.count()` for source-fidelity credit.

Dependency correction: the pinned Fedimint Cargo.lock selects **aleph-bft-crypto0.9.1**, checksum `4e99189e0c434af4bb0f4238925df15091962ae843f509561def48ecf36c163d`. An early parent steer guessed0.9.0; corrected from the actual lock before any parent trial or freeze. Additional dependency source must use the lock identity, not that discarded guess.

No common new automation demand is established by this static audit. Nested collection frames, interval arithmetic and concrete cardinality remain hypotheses. First receive both frozen ordinary/Infer baselines and literal annotation inventories. A new engineering layer needs the actual required proof sites, a causal removal target and an explicit stopping condition; existing automatic arithmetic and redundant broadcasts earn no credit.

Fresh independent review must cover delivered source/domain/property fidelity, checked dependency closure, relevant semantic/safety failures and import/publication behavior, and actual author costs versus generated/library work. Existing component validation is reusable evidence; repeating it is not a caller-level advance. Unsupported original attempts remain counted. HMAC/native stays **DEFERRED AND UNACHIEVED**.
