# Practical caller transitions — active milestone

Authorized by the user's approval of the next caller-level milestone on 2026-11-12. Integration base: d1668d0d; accepted engine: 5ecea5fe, schema39/policy78/cache81. The preceding [practical automation milestone](../practical_redb_fedimint/FINAL_REPORT.md) remains complete. Its original freezes, final validation and unsupported-case denominator are historical inputs, not new results.

Prove useful properties of original REDB allocator callers and Fedimint signature-admission/bookkeeping callers, connecting the previously proved bitmap and quorum components to actual collection/state transitions. Preserve executable algorithms and their actual valid-input domains. Authored semantic definitions, valid-state invariants, semantic arguments and remaining mechanical annotations must be explicit and separately charged.

## Initial candidates and first checkpoint

- REDB: `BuddyAllocator::record_alloc_inner` and `free_inner`, with their genuine bitmap dependencies. Target exact successful allocation/free effects, split/merge consistency, non-overlapping represented blocks and page conservation; rejected allocation must preserve logical state. Establish the real allocator invariant and request domains from source/callers before freezing. Construction, resize, search, serialization, crash consistency and whole-database correctness are not implicitly included.
- Fedimint: `Keychain::is_complete` and the concrete signature-count guard in `ConsensusEngine::validate_signed_session_outcome`, with their genuine collection dependencies. Target the link between concrete distinct signer membership/cardinality and the existing quorum policy, including insufficient-count rejection. Determine what actual verification/bookkeeping bodies are supportable before fixing the theorem. Cryptographic correctness is excluded; this exclusion does not authorize replacing a runtime dependency with an assumed callback summary.

These are preflight selections, not successful proofs or frozen final scopes. Each lane must commit exact source, property, domain, adaptations and trust inventory before its first proof trial. First deliver eligibility and ordinary/Infer baselines on the accepted runtime, including what already proves, unsupported operations and the literal remaining author burden. An unsupported selected caller stays in the denominator; preregister a justified real-code replacement before its trials. A partial property is labeled partial and does not satisfy the full caller target.

## Engineering and completion

1. Freeze and measure both caller cases in isolated worktrees. Reuse accepted application proofs without counting their reruns as new progress.
2. Prove supported caller transitions ordinarily; report semantic work versus mechanical glue. Parent integrates common fixes. Nested collection update/frame transport is a hypothesis, not an already established common need.
3. Before any new engineering layer, record the selected property helped, required manual work removed, expected reusable benefit and stopping condition. Implement shared assistance only after both lanes demonstrate matching required obligations. Do not force a new common layer when the demands differ.
4. Validate original source/domain/property fidelity, relevant semantic/safety/failed-export/import controls, full checked-body/helper identities, and causal automation contribution if any. Obtain a NEW independent review of delivered proofs and implementation. Report useful caller guarantees, exact exclusions, costs and unsupported attempts.

The milestone succeeds when substantive caller-level guarantees in both projects are checked and independently accepted within the existing trust boundary, with the actual automation contribution and remaining work stated accurately. An eligibility checkpoint alone is not completion. If a full candidate is unsupported, report the shortfall instead of asserting whole allocator, consensus or cryptographic correctness.

## Execution limits

- At most 45 active investigation minutes per blocker, excluding build waits, then a recorded scope/value checkpoint. Do not automatically start another prerequisite.
- No new trusted axioms, silent library-contract strengthening, disabled body/lifetime/trait/WF/mode/erasure checks, narrower application inputs or replacement executable algorithms. No mechanical cost hidden as semantic input. No unproved callback, private-representation or native correspondence bridge.
- New Rust semantics, ownership foundations, substantial private-representation machinery, general foreign-MIR certification and Coq/Iris/Miri research are deferred boundaries, not this milestone's prerequisite chain. Small supported ordinary proofs are preferred over expanding engine work.
- Use existing `infer_experiment.py` / `infer_eval_support.py` and focused checks during development; run a matching broader suite at a meaningful integration boundary. Do not create another evidence subsystem or rerun broad matrices for documentation changes.
- Independent isolated REDB/Fedimint workers; primary integration lead; no recursive delegates, shared mutable caches, shell-created worktrees, unrelated discovery, pushes or PRs. Use inherited provider/model settings.
- The earlier [HMAC/native objective](../practical_redb_fedimint/DEFERRED_HMAC_NATIVE.md) remains explicitly **DEFERRED AND UNACHIEVED**. Native correspondences remain zero; historical 44+3/101 obligations and full HMAC suffix are open. This milestone neither resumes nor completes them.
