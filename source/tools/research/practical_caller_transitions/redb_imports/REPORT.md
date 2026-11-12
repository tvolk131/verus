# Checked separate REDB caller — parent checkpoint

The imported caller proves that two consecutive allocations of the same interval cannot both succeed, for every original valid allocator and every u32 page/u8 order. This is a conditional guarantee about original checked allocator calls through the semantic wrapper. It requires no supplied interval-freeness, nonempty-interval or configured-order assumption. The original allocator has already received scoped independent semantic acceptance; this additional public API adaptation still needs its own review.

The provider is the exact six-part full V2 aggregate with eleven reversible declaration changes: BuddyAllocator becomes public with private fields; record_alloc_semantic and width_positive become public; width/requested are public open definitions; valid/geometry_frame/legal_block/free_pages/state_eq/used_pages are public closed observations. No function body, contract formula, application domain or original proof annotation changes. The replay reverses those changes and checks byte equality against the original aggregate before verification. The interface exposes authored interval arithmetic and opaque allocator observations, not private fields or new ownership/representation axioms.

The separate client contains two runtime calls to the exported wrapper. Its argument is an arbitrary original valid allocator. If both results were true, exact removal after the first call and whole-interval availability before the second contradict membership of the interval's first page. It uses one ghost snapshot, one proof block, two explicit existing checked lemma calls (width positivity and vstd distributivity), one existing range-property broadcast and three assertions. All remain charged. There is no Infer annotation reduction, inferred helper or new common engineering layer.

Fresh guarded optimized-Python replay completed all ten expected outcomes:

- Ordinary and Infer providers each compile and verify96/0, with exactly25 executable/47 proof bodies and zero inferred helpers.
- A separately compiled client verifies1/0 with goals off/on against each provider: four successful imported caller runs.
- Against each provider, the false simultaneous-success control and missing-validity precondition control reject1/1, failing only their respective named function. Each failed Infer run removes its seeded stale export. Imported provider artifacts remain bound and unchanged.

All applications retain --no-cheating, complete local-body coverage, normal lifetime/trait/WF/mode/body/erasure checks and the accepted78 runtime. Successful provider runs precede import eligibility; failed ordinary exports are never used as providers. Summed compiler wall time is46.433s across ten launches, a single reproduction sample, not a benchmark or speedup claim. No build, production/compiler/vstd change or broad regression rerun was needed.

The two earlier trials remain failed proof-development evidence. Both full providers verified96/0. The first client lacked the explicit nonempty-interval arithmetic step; the second isolated nonlinear query omitted the equation for its local x. Their failed receipts, generated source snapshots and logs remain at /tmp/practical-r3-imports-first/ and /tmp/practical-r3-imports-second/. The third preregistered correction uses an existing checked distributivity lemma. Neither development failure is a semantic negative or silently repaired historical success. Chronological freezes and source commits are5a7d23c4, f79d83db and1bcb8751.

RESULTS.json binds all ten outcomes, exact source/body inventories, compiler commands/timings, eight positive VIR/rlib artifacts and both failed receipts. Completed receipt: /tmp/practical-r3-imports-third/receipt.json, SHA256 f7a6f460d6d7feef5be98a852617cedc6a4406857da568e06ead33aece7d6e65.

Replay from this checkout:

```sh
python3 -B -O source/tools/research/practical_caller_transitions/redb_imports/replay.py \
  --freeze source/tools/research/practical_caller_transitions/redb_imports/FREEZE_3.json \
  --output /tmp/fresh-redb-import-output
```

The provider includes the fully checked free composition, but free_semantic remains private and no separate free API client is claimed here. This checkpoint does not prove native crate integration, constructors establishing valid, runtime count_free_pages, ownership-ledger conservation, crash consistency, whole-database correctness or the pending Fedimint initializer. Trust remains the accepted Verus/Rust frontend, solver and existing vstd/collection/arithmetic models. HMAC/native remains **DEFERRED AND UNACHIEVED**; the full active caller goal remains incomplete.
