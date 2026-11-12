# Independent frozen owned-cell design challenge

Baseline HEAD: `0dd6130fc221f403078797b1f878fba4ada7ea7e` (authenticated with git before edits). Start: 2026-11-12 14:42:19 UTC. Budget: at most 45 active minutes, conservatively measured as wall time; no builds excluded.

Own only `evidence/native-owned-cell-design-challenge/`. Review the five frozen `source/tools/research/redb_crash_protocol/native_state_link/design/` documents, authenticated baseline inputs, and existing compiler/erasure/import infrastructure needed to challenge the proposal. This is a new independent design challenge, not review or acceptance of a live implementation. Do not inspect sibling or live parent outputs, delegate, edit implementation, run broad builds, push, or create a PR.

Question: can the actual installed core `UnsafeCell<u64>` new/into_inner/get_mut bodies support a closed owned/exclusive interpretation with full ordinary and Infer checks, arbitrary lawfully initialized input, correct reborrow/writeback, and a mandatory final-native-input-state proof? Challenge complete-unit admission, spec and private-field authority, native !Sync, erasure, foreign/unchecked/pruned/import/cache bypasses, and unknown destructor/unwind effects. A shared Mutex obstruction alone does not refute the explicitly unshared slice.

Deliverables: actionable design findings or scoped feasibility; small bounded native counterexamples/diagnostics only where useful; exact evidence and limitations; ordered evidence-only local commits. No future implementation is accepted by this review. Full CP1/CP2 and source_coupling remain false. HMAC/native remains DEFERRED AND UNACHIEVED.
