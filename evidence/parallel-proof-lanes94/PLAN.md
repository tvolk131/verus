# Parallel proof lanes: Campaign94 and bounded transfer work

The main line remains native redb commit/recovery reasoning for both commit
types and mixed histories. All18 original completion cells remain OPEN.
Three isolated background lanes may make independently reviewable progress:

1. Concrete normal-return destructor/borrow support: ordinary implementation
   toward the native leaf builder, or a precise blocker. No rerun of interrupted
   compiler-data perturbation tasks and no blanket destructor acceptance.
2. Fedimint wallet fee arithmetic: exact checked/wrapping fee computation and
   its consensus-version-dependent result policy, distinct from PaymentFee81.
3. Third-repository trial: cached rust-hex0.4.3, native error-aware slice decoding,
   correct bytes/error position and modified-prefix frames. Full encoder roundtrip
   is a stretch only after the native decoder is established.

Each child gets a tool-managed isolated worktree, a frozen source handoff and
one patch-sized scope. Fresh worktrees omit the parent's uncommitted work, so
the parent overlay must be authenticated and staged before changes. Child
changes are compared with that overlay or the separately identified experimental
seed, NOT blindly with Git HEAD. No parent commit or automatic feature adoption.
Returns must separate inherited source, experimental seed and new child deltas.

Only lane1 may rebuild the compiler, with at most two compiler and two matching
vstd builds. Other lanes use the frozen runtime with zero builds or network.
Each lane has at most48 direct verifier applications, including failed attempts;
checkpoint after the first6 for feasibility. Child subdelegation is disabled.
Ordinary test-only verifier subprocesses are itemized separately. No recursive
review chains, repeated source inventories per minor step or global all-pass claims.

Machine storage was about22GiB free at planning. Preserve at least8GiB free;
lane1 may allocate at most4GiB and other lanes at most1GiB each. Use one compiler
builder with cargo jobs<=2 and verifier threads1. Do not delete existing frozen
packages or caches to make space without parent reassessment.

Disposable builds, caches, raw logs and staging archives go under /private/tmp.
The delegation API chooses managed worktree locations; it exposes no directory
override, so /tmp placement or automatic worktree deletion is NOT promised.
Retain small final source/proof patches, commands, input identities, findings and
coverage limitations in this project before deleting a lane's temporary work.
Record actual worktree/scratch paths at handoff; explicitly remove a worktree
only after integration or deliberate abandonment and preserved deliverables.
An active or unmerged worktree must not be treated as safe to auto-delete.

Every result is independently reviewed before integration if it changes compiler
semantics or proof trust. A proposal, failed proof, frontend block or assumed API
is not a proved native capability. Found native bugs get a clear reproduction,
impact qualification and regression; a clear local fix may be proposed without
silently replacing the pinned original or broadening trust.

The original1331 native paths/Git metadata remain missing. Recovered pinned
contents match every original guard; this is content recovery, not original-path
or metadata continuity. Use the recovered snapshots read-only. Historical/shared
Git objectstore remains NOT_CERTIFIED; managed worktree creation writes expected
Git metadata and is not a claim of global Git nonmutation.
