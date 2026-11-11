# S2: checked word-sequence frames (proposal)

Parent99e289bc; early scope/fixture freeze ec86679d; implementation20fa79a9; safety control933b974a; final eligibility hardening580299b1. Parent integrates and accepts. Independent implementation review and combined S1/S2 checks are required. HMAC/native is **deferred and unachieved**; F0/R1/F1 remain unsupported in the denominator. No new common-use or whole-goal completion claim.

## Property and author work

The SAME registered R2 BtreeBitmap set/clear/update_to_root and twelve-method closure, on every valid nonempty multiheight/multiword tree, arbitrary spare words/padding, any valid bit including repeats: exact allocated leaf set/count, restored ancestor summaries, length/storage/padding/nonpath frame. Concrete consumers are BuddyAllocator::record_alloc_inner -> set, free_inner -> clear, BtreeBitmap::alloc -> set. Complete allocator callers/search/init/crash consistency and native refinement are not proved.

The existing R2 fidelity checker checks the reduced fixture and every component variant against pinned source and original contracts/specs. s2-reduced.rs deletes ONLY word_bits/word_frame, two writer snapshots and their proof blocks/calls, and makes W0's charged clear_not_full adaptation: existing lemma_full_view PLUS an explicit view/physical-bit assertion. No local replacement library calls/definitions/frame assertions, changed runtime, new precondition or smaller domain in the two writers. All other helpers, rootward-loop facts/fuel/triggers/snapshots remain authored.

| Literal source inventory | Original manual | S2 author | S2 generated |
|---|---:|---:|---:|
| Local mechanical proof helpers | 8 | 6 | 0 |
| Assertions (including forall) | 37 | 29 | 2 forall blocks |
| Entry/local ghost snapshots | 9 | 7 | 2 |
| Exec-local proof blocks | 9 | 7 | 2 |
| App-local BV directives | 3 | 0 | 0 |
| Static local helper calls | 15 | 11 | 0 |
| Other ordinary library proof calls | 0 | 1 lemma_full_view | 2 pointwise calls |

Categories overlap. Total logical snapshots remain9 and exec proof blocks9; generated work has not disappeared. New ordinary vstd library: two open specs and three checked proof bodies,7 assertions/3 BV directives,6 backend queries in the separate strict helper check. The added pointwise wrapper is explicit factoring, not a newly assumed theorem. The one range broadcast is retained, already default/redundant, and earns zero removal credit. S1's separately accepted extra set assertion removal is excluded here; this checkout does not integrate S1.

Remaining authored calls: layer_edit2, leaf_edit2, clear_not_full2, finish_mutation2, propagation1, ancestor_next2 (including recursion), lemma_full_view1. Six loop-entry predicates, three exit predicates, two fuel directives, two explicit triggers, and original decreases remain charged. Semantic membership/hierarchy/allocation definitions remain authored; clear_not_full's adaptation remains mechanical.

## Mechanism and checks

Only typed visible local spec expansion and the existing generic Seq<u64>.update/shift/mask definition drive selection. Exactly one update/frame pair, one immutable integer bit and mutable receiver, constant set/clear value. No application/field/project names. Exact old/current/final roots and field paths must agree; authored mutable aliases, ambiguous/multiple buffers, conditional mutation, loops/returns/closures, effectful/nonboolean exit tails, absent catalog/types/visibility or exhausted node/recursion budgets skip unchanged. Runtime assert/panic guards and signature-validated Vec indexed mutation are recognized without changing them. Capture-prone expansion skips instead of renaming source variables.

The generated scaffold is W0's receiver snapshot, then the EXISTING unexpanded physical-bit quantified goal with a checked vstd::word::checked_seq_bit_update_at call. It preserves ordinary trigger inference and adds no padding assertion. Prefix precedes original statements; exit proof follows original mutation through frontend trailing-block wrappers, before the unchanged tail. Original statements and leaf tails retain Arc identity.

Independent validation reselects from the original contracts/catalog and reconstructs the exact snapshot/init/mode/type/root/field/Seq/index/value/quantifier/call/scope/placement. Deleting ONLY validated new declarations/ghost blocks recovers the full serialized Function, including contracts/decreases/metadata/impl paths, with extra identity checks for raw diagnostic spans and original statement/tail nodes. Global fresh Expr/Place/read/statement IDs and textual binder names reserve all current/imported function ASTs AFTER preceding passes. Independent collision auditing includes previous functions/decorators. Ledgers are allocated only after complete eligibility. Consistent receipt/body corruption, duplicate calls, metadata/contracts/executable/decreases/placement mutations, aliases, capture, wrong catalogs/types and translated Place reads have focused tests.

report.body_rechecks requests ordinary full WF/mode checking and refreshed resolution/binding maps. Existing lifetime/trait/body/erasure/precision/publication gates remain. Proposals are ordinary proof attempts; generated reports are not proof receipts. No infer helper is synthesized or cached. Schema39/policy78/cache81 are provisional S2 reservations; parent reconciles S1 independently.

## Validation and reproduction

**Final hardened implementation validated; proposal ready for NEW independent S2 review and parent combined acceptance.** Matching compiler580299b1532d7de3ae0b12cd15a098d346167153 (debug, Rust1.99.1), fresh normal vstd2256/0; full VIR165/0 (8 S2 tests), refreshed compiler-library8/0, new focused compiler7/0, existing word regressions3/0. All checks remain enabled. The focused suite also checks renamed application/field names, wrong runtime mutation, all three original false properties, missing valid-bit safety precondition, and unrelated programs.

Final immutable existing-utility receipt: /tmp/practical-s2-final-validation3/receipt.json, SHA256 **7b697e8ef25acf0ec075c268ec29e0e6f09bfbc8c68f2bccb84cca3f2d5fed57**, success=true,13 expected outcomes:

| Outcome | Verified/errors | Scope/result |
|---|---:|---|
| Infer OFF | 18/2 | ONLY U64GroupedBitmap::set/clear postconditions |
| Infer cold + --compile; repeat | 20/0 each |12 exec+6 proof;2 generated snapshots/forall blocks/library calls; helpers=[] |
| False leaf/repeat-count/frame |20/1 each|ONLY the named false caller postcondition; no failed export |
| Compiled/exported API provider |22/0|14 exec+6 proof; unchanged two-method testing facade added separately |
| Ordinary/inferred imports |3/0 each|3 exec/0 proof, no authored proof or generated helper |
| Existing successful export overwritten by false caller |20/1|Stale interface removed |
| Old policy, wrong schema, incomplete required precision |frontend controls|All three imports rejected, no output published |

The complete R2 property is proved in the positive runs. Negative runs are not certified full-crate results: the receipt records exact failed identities, healthy remaining queries, no mixed safety/frontend/resource failure, and no interface publication. Empty inferred helper inventory is explicit. No cache-hit or speedup claim.

Development full R2 trial8 passes20/0 with12 exec+6 proof bodies, exactly2 snapshots/2 quantified blocks/2 pointwise calls. Strict standalone W0 helper freshly proves/compiles with --no-cheating:3 proof bodies/6 queries. Standard normal full vstd build passes2256/0, using inherited admissions normally (no no-cheating forwarded). Full VIR library suite passes164/0, including7 S2 tests. The later hardened final results above supersede this development run.

Private build/development logs: /tmp/practical-s2-at99-build. Own authenticated offline Cargo inputs only, CARGO_BUILD_JOBS=2, pinned Rust1.99.1 and solver SHA e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765. Accepted75's8 tools+receipt were privately copied and hashed before use. No network/shared mutable Cargo/other checkout or worker/cache/process reads, no agents, push/PR or layer extension.

Build from implementation commit with own CARGO_HOME/CARGO_TARGET_DIR and explicitly pin build.build-dir as well (for example cargo --config 'build.build-dir="/tmp/OWN/target"') and all standard vargo flags (`--cfg proc_macro_span --cfg verus_keep_ghost --cfg span_locations --cfg procmacro2_semver_exempt`), offline packages rust_verify, verus, verus_builtin, verus_builtin_macros, verus_state_machines_macros, vstd_build. Copy matching tools into a private runtime; run vstd_build there normally from source/. Set RUSTUP_HOME=/home/dev/.rustup, RUSTUP_TOOLCHAIN=1.99.1, VARGO_TOOLCHAIN=1.99.1-x86_64-unknown-linux-gnu, VERUS_Z3_PATH to pinned solver. Replay:

`python3 -B -O source/tools/research/practical_redb_fedimint/word_frame/s2-replay.py /tmp/OWN-MATCHING-RUNTIME /tmp/practical-s2-UNIQUE`

The controller uses existing infer_experiment/infer_eval_support and the original R2 fidelity checker. It guards source/runtime hashes, full body identities, empty inferred helper set, exact failed-function identities, normal compilation/export/import and failed-publication/metadata gates. Repeated run is not a cache-hit or speedup claim.

## Retained failures, cost and boundary

All failed setup/frontend/proof-development trials remain private, separate from semantic-negative evidence. Default bwrap loopback failure used expressly authorized command-local escalation. Initial offline Cargo lacked its private Git refs directory/ref; rebuilt only from authenticated objects. Early compiler attempts lacked rustup on PATH/full VARGO_TOOLCHAIN identity; first full-vstd attempt lacked standard ghost macro flags and checked no bodies. First full VIR suite lacked VERUS_Z3_PATH and exited128; corrected full suite passed. Compiler-library tests initially lacked their hardcoded test runtime; the unchanged suite subsequently passed against the fresh private bundle temporarily exposed by an owned source/target-verus/release link. This link was removed after tests. No old ignored binaries were read.

R2 trials1/2/3/5/6 were selector misses (18/2, no generated frames); trial4 caught an independent template Spec-read mismatch before verification; trial7 generated both frames but fully expanded the observation and failed ordinary trigger inference. Trial8 returned to the literal W0 snapshot/unexpanded observation template and passed20/0. None is a weakened-domain proof or semantic counterexample. All helper/probe/application trials used --no-cheating; normal full vstd used its standard admission policy.

Early freeze21:45 UTC; first full R2 generated success22:23 UTC on2026-11-11, under45 wall minutes including builds, hence under the active-blocker limit. Subsequent work is the required same-layer validation, not another prerequisite. No RSS measurement/statistical speedup claim. Final replay command-wall seconds: off6.997, cold/compiled31.950, repeat33.477, leaf33.705, repeat-count34.969, frame34.783, provider31.311, ordinary/inferred imports3.796/6.934, stale export29.777; metadata controls0.044/0.045/0.063. Sum247.849 seconds, measured by LoggedCommands, excludes setup/hash checks and is not CPU time; concurrent focused tests ran. Full VIR tests8.63s, compiler-library3.45s, focused S2 tests252.05s, existing word tests18.71s, excluding compilation. Final post-hardening compiler rebuild3m43s; normal vstd verification/compilation completed2256/0 (no separate wall timer recorded).

A late review added the bounded pure-boolean/empty-tail gate and8th S2 VIR test: never insert a proof before a still-effectful executable tail. Matching compiler/vstd/full VIR/library/focused compiler/13 replay controls were refreshed after this production change. The earlier complete successful pre-hardening receipt /tmp/practical-s2-final-validation2/receipt.json is retained (SHA2560f4b08baef025d4a8b87b3f404ad5f9b55d0a678628ac6b387184b484196bca2).

Storage setup outcome: the first final replay exited120 during ENOSPC and left empty cold.stdout/receipt.json.tmp; /tmp/practical-s2-final-validation remains unsuccessful, never counted. The first focused compiler run had2 passes then5 input-directory storage failures before Verus. Logs remain. Only this task's disposable private Cargo incremental/build outputs were removed (about13GB incremental) before fresh reruns; sources, failed logs and final artifacts remain. No unrelated/shared caches were touched.

Actual test-path audit: the three invoked test executables are ordinary files under /tmp/practical-s2-at99-build/target/debug/deps; their resolved paths are identical. common/mod.rs therefore resolves /tmp/practical-s2-at99-build/target-verus/debug, an owned link to runtime-tail; all8 tool hashes match the final replay. The compiler-library hardcoded release path also pointed only to runtime-tail. Private audit /tmp/practical-s2-at99-build/test-paths.json SHA2567c3fe9b9669c3479266b5d816495c967d3c55f9ee5c53ec5a600cf04b4e88631 records executable hashes/paths. Cargo commands ran from the checkout root with --manifest-path, not source/ (whose .cargo/config.toml sets both build dirs). Completed good checks used verified private actual paths; future reproduction must pin both directories or relocate exact authenticated binaries beside their runtime. No unresolved foundational boundary or new prerequisite was opened.

TCB: existing Verus/Rust frontend, solver/backend, imported ordinary vstd specifications and existing admitted standard models. Added word proof bodies are checked; no axiom/assume/external_body/stronger hidden contract, new Rust/ownership/private-representation/foreign-MIR/Coq authority, bypassed check or application semantic premise. Host x86_64 evidence only; no new all-target/native theorem. Stop after S2; parent owns independent review and broader combined acceptance.
