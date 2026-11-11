# S2 frozen scope/interface

Parent99e289bc; W0 registration8925e8d2. Only generic Seq<u64> physical bit-update transport in the two existing word writers is authorized. Source consumers are BuddyAllocator::record_alloc_inner/free_inner and BtreeBitmap::alloc. Complete allocator/search/init/crash consistency is excluded. HMAC/native is deferred and unachieved; F0/R1/F1 stay unsupported. S1 is independent.

s2-reduced.rs is frozen before assisted trials: delete word_bits/word_frame, the two writer entry snapshots and their two proof blocks; retain the charged lemma_full_view plus view/physical-bit assertion in clear_not_full. All other source/contracts/domain/algorithm/glue remain. No local replacement library calls or frame assertions in reduced writers.

Implementation interface: bounded typed visible spec expansion selects an existing unconditional Seq<u64>.update plus physical-bit frame; one immutable bit parameter and mutable receiver only. Generate an erased entry snapshot and ordinary quantified checked pointwise-library call before the original tail. No goal/contract rewrite, new assumption or tree-specific recognition. Full Function projection removes only independently validated scaffolding; normal WF/mode/lifetime/body/erasure/precision/publication gates apply through body_rechecks. Missing or ambiguous catalog/types/patterns/budget skips unchanged.

New ordinary checked laws live in vstd::word. Provisional schema39/policy78/cache81. Generated annotations count separately. 45 active minutes per blocker excluding build waits, then honest checkpoint; no next prerequisite. Parent needs independent implementation review and combined S1/S2 checks; this lane cannot accept/publish or complete the overall goal.
