# Proposed replacement goal: practical Infer automation for REDB and Fedimint

This is a user-requested prioritization reset. It replaces the frozen HMAC/native-bridge dependency chain as the active critical path. That earlier objective remains unachieved and deferred; do not mark it complete or weaken its property to claim success. Preserve existing accepted capabilities and historical evidence. This brief does not authorize resuming a paused thread until the user resumes it.

## Goal text

Increase Infer's ability to discharge recurring mechanical proof obligations in real REDB and Fedimint code, so authors can concentrate on semantic definitions, intended behavior, state invariants and genuinely semantic proofs.

Starting from the accepted branch, select and preregister one substantive component from each project before proof trials or engine changes. Each must have an important caller-visible correctness property, a credible connection to a larger project invariant, and its original executable algorithm and actual valid-input domain preserved. Prefer allocator/page-state correctness in REDB and amount/accounting, validation or state-transition correctness in Fedimint; inspect the real sources before fixing the exact cases. A tiny helper or synthetic packet example alone is not sufficient. Existing helper proofs can be reused, but rerunning them is not new progress.

Deliver source-connected proofs for both components, with authored semantics explicit and substantially less application-local mechanical proof glue. Demonstrate at least one reusable Infer improvement that causally reduces manual work at independent real call sites in both projects. Preserve all required ordinary body, source-fidelity, lifetime, trait, well-formedness, mode, inference-precision, erasure and publication checks within the existing documented trust boundary.

For each attempted case, report the property and scope, baseline, semantic inputs, mechanical proof burden before/after, exact automation contribution, remaining manual proofs and assumptions, verification cost, and negative controls. Validate the shared improvement through ordinary callers/imports where relevant, cold/warm behavior and independent review. Finish with a concise practical capability/limitation map and reproducible validation. Application proof burden removed and cross-project reuse are the primary outcomes; theorem, query, test and receipt totals are supporting evidence only.

## Scope and stopping rules

- Do not make Miri verification, native borrowing soundness, Coq/Iris foundations, the frozen HMAC suffix theorem, or general foreign-MIR certification prerequisites of this milestone.
- No new trusted axioms, silently strengthened library contracts, disabled checks, narrowed application inputs or replacement executable algorithms. Do not relabel mechanical invariants/lemmas as semantic inputs to hide manual work.
- Investigate a blocker for at most 45 minutes of active investigation, excluding build waits, before a recorded scope checkpoint. A checkpoint is not permission to automatically start another prerequisite.
- If a blocker requires new Rust semantics, an ownership foundation, substantial private-representation machinery or an expanding prerequisite chain, record the unsupported boundary and defer that case. New foundational research needs a separate explicit user decision.
- An unsupported case is not a successful proof. Keep it in the coverage denominator. A replacement case must be selected and registered before its trials, with the reason for replacement recorded. Do not remain trapped by one frozen target, and do not hide unsuccessful attempts.
- Every implementation/delegation must name the selected real call sites helped, the mechanical obligation addressed, the expected reusable benefit and its stopping condition. Prefer a common fix over project-name-specific recognizers.
- Keep source/body/domain/property definitions stable within each experiment. Correct a bad specification openly rather than presenting a specification change as automation progress.

## Execution and durable state

1. On user resume, inspect the current branch and paused/in-flight work. Preserve local changes and evidence; do not delete unreviewed work. Stop or defer obsolete foundational workers safely using the task-management tools. Finish only the bounded checks necessary to understand an in-flight change; do not automatically extend it into another stage. No pushes or PRs without authorization.
2. Replace the old active priorities in the goal, plan, concise handoff/current-status documents and worker briefs. Archive the original unmet HMAC/native requirements as deferred. Do not leave a stale `next dependency` directing execution back into Miri research.
3. First checkpoint: a brief inventory of accepted reusable features versus research-only/provisional work, the selected REDB/Fedimint components and their important properties, and baselines of what already verifies and what manual mechanical work remains. Provide a short ranked backlog tied to those cases.
4. Use isolated worktrees for independent REDB and Fedimint baselines/experiments. The primary agent integrates common fixes. Workers must return unsupported boundaries instead of recursively widening their tasks. Do not spawn agents merely to fill capacity.
5. Reuse existing proof/replay utilities. Run focused checks during development and a matching broader suite at a meaningful integration boundary. Do not add another custom evidence subsystem or rerun full matrices for documentation-only changes. Preserve appropriate semantic, safety, stale-export and inference-negative controls.
6. Conduct a value checkpoint before each new engineering layer: which selected application property gets closer, what manual work disappears, and is this still the most direct route? If there is no concrete answer, return to the practical backlog.

## Completion criteria

- One substantive caller-visible property proved in each project, preserving the specified source and domain.
- A reusable automation improvement used causally in both projects, with explicit removal of nontrivial mechanical proof work and no hidden replacement glue.
- Necessary soundness/inference/import/publication checks and independent review pass; costs and trust boundaries are recorded accurately.
- A concise report distinguishes completed application results from unsupported or deferred properties. No claim of whole-database, whole-protocol, cryptographic, crash-consistency or native-Rust correctness beyond what the actual specifications and proofs establish.

If these criteria are not met within the bounded experiment, report the shortfall at the checkpoint. Do not invent a deeper research program simply to keep the goal running. The north star remains broadly useful, trustworthy automation for important REDB/Fedimint properties.
