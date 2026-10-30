# Independent clean-main native adoption v2 replay

This is a fresh-context, independent adoption attempt authorized by the user.
Do the work yourself. Do not delegate or spawn other agents. Work only in the
Aleph worktree the app has already bound to this thread. Do not set up a checkout
or worktree yourself. Start by recording pwd, origin, branch, HEAD and git status;
the expected clean base is f35c7bba6d628bcafdf0d65dd471cc6f868ec034. Stop if the app
supplied a different repository or a dirty/unexpected base. Do not inspect or
modify the original Aleph development checkout.

## Inputs and independence

Use the fixed compiler at
/private/tmp/verus-adoption-v2/closure-engine/verus/source/target-verus/release,
from clean Verus commit 9b24988eff4a250188ff11445115560c54cf4688. Its source is
/private/tmp/verus-adoption-v2/closure-engine/verus/source. Matching macros/vstd and Z3
are built there. Do not change compiler/library source or use another build.

Allowed references: this task, the native Aleph source at the clean base, cached
third-party dependency source, and the general Verus guides/examples/tests in
the fixed engine. Start with source/docs/guide/src/infer_adoption.md, infer.md,
exec_attr.md and source/tools/native_adoption/{README.md,src/lib.rs}. The generic authored-source
inventory in source/tools/adoption_inventory may help reconcile source inputs
with the compiled scope, but cannot classify their semantics for you. Other
general compiler tests/docs can help diagnose a profile, but the goal is not to
copy a generic example into an extracted substitute for production code.

Forbidden inputs: other Aleph worktrees/branches/commits, previous Aleph proof
implementations, patches, fixtures, reports, ledgers and experiment threads;
any verification/units_collector, verification/fresh_adoption,
verification/native_adoption_v2 or verification/native_adoption_v2_parent artifacts
in another tree; /private/tmp/verus-adoption-v2 (except the fixed compiler source
explicitly allowed above); /private/tmp/verus-adoption-v1 or previous replay output; the root
Verus adoption milestone/evidence/assurance histories. Do not use git log,
other refs or broad filesystem searches to discover prior solutions. You may
read the clean base and your own subsequent commits and diff. There is no need
to consult an old solution to decide what these native implementations mean.

## Finite required proof scope

Reproduce the following agreed semantic result over the actual sequential
native collector -> Creator -> parent metadata path. This is not protocol-wide
agreement/liveness, asynchronous networking or cryptographic verification.
Different views, names and decompositions are welcome if the properties and
domains below are equivalent. Adding checked APIs is allowed where requested.

Include the native NodeCount arithmetic/conversion/comparison operations used
by consensus_threshold; NodeMap construction, size, get, insert, iter, values
and its required Clone; and generic checked prefix-fill and rejecting prefix-fill
operations over NodeMap. Infer routine supported operations from their real
bodies, including real derive implementations where used. Do not replace derives
with assumed adapters or toy copies.

Include collector initialization (initial-state constructor, initial constructor,
from-previous advancement), lawful unit observation ingestion, actual production
add_unit delegation and prospective parent selection. Include Creator construction,
current round, bounded history growth, mutable collector selection/initialization,
ingestion and PreUnit creation. Raw APIs retain their real domains; add checked
initialization/ingestion/advancement APIs that cover the terminal round with an
explicit error before mutation. Include UnitCoord and PreUnit construction and
accessors; ControlHash construction, parent projection/iteration, member count,
initial/noninitial validation branches and the public validator. Finally check
the actual production FullUnit implementation of Unit against the authored unit
observation law used by the generic client, without claiming Signed delegation.
Required bodies and their real checked call paths must be in the source closure.

## Semantic requirements and domains

* A collection view retains membership positions and absent slots. Its basic
  operations preserve length, positions, inserted values and unrelated entries.
  Prefix filling handles zero/empty/full prefixes; a rejecting prefix API leaves
  the whole state unchanged. Required Clone preserves the chosen logical content
  under an explicit adequate checked generic Clone interface.
* Collector validity: target round is positive; every stored unit round is
  below it; direct-parent count equals exactly the number of occupied slots in
  the preceding round. Establish this from fields rather than assuming it.
  Preserve native initialization, advancement, update, duplicate/replay/fork and
  newest-parent policies. Cover every valid represented state and lawful unit
  observation, not merely states in a bounded trace.
* Parent selection succeeds exactly when quorum holds and the configured node
  has a preceding-round parent; success returns the actual complete stored map.
  Older additional parents are permitted. Insufficient-quorum error precedes
  missing-own-parent error, including zero-membership behavior.
* Quorum is floor(2*n/3)+1 with the original multiplication domain
  n <= usize::MAX/2. Exact inference must recover the actual original helper
  domain. A separately documented checked arithmetic refactor may broaden it,
  with before/after evidence, but broadening is not needed for this acceptance.
* Creator validity: a nonempty bounded history of correctly indexed, correctly
  sized valid collectors. Check the growth/selection/ingestion/creation policies
  and preservation of that invariant. Raw advancement/ingestion covers all
  rounds below u16::MAX. The additive checked APIs cover u16::MAX and reject it
  before mutation. Do not narrow valid-state or payload domains to get proofs.
* Creation covers every member count in the frozen quorum domain and every
  in-range node, arbitrary payloads and all u16 requested rounds. Initial round
  construction has no parents and permits arbitrary configured node IDs;
  zero membership retains constructor/ingestion/error behavior but has no
  in-range creator. Historical raw vector lookups keep actual bounds obligations.
* Successful creation has requested coordinates, the exact selected-parent
  round projection, eligible parent positions/rounds and native quorum/own-parent
  policy/error precedence. Projection preserves length and occupied creator
  positions and their rounds. On positive-round creation success, explicitly
  establish the member count, own preceding-round parent, bounds for every
  occupied round and the floor(2*n/3)+1 preceding-round quorum. Connect those
  same facts to native metadata numeric eligibility on the actual successful
  creation result, in a universal application goal over the frozen domain.
  Metadata validation covers zero and positive
  rounds with the native branch/error order. Prove complete PreUnit forwarding
  of its supplied ControlHash value, without an assumed hash equation.
* Hash comparison claims must describe exactly the comparisons validation makes
  under the opaque boundary below. In particular, an initial bad-hash error must
  retain the compared stored hash and the reported recomputed value. When the
  existing PartialEq specification obeys its law, those reported values must be
  unequal under that specification. Do not assume the law flag is true for an
  arbitrary hash type. Do not claim constructor/validator hash
  agreement, collision resistance or deterministic encoding/hashing without an
  adequate checked contract. Numeric eligibility must not depend on invented
  hash/encoding facts.

Application contracts and coverage goals must establish these universal
properties with Verus; native bounded tests do not substitute for them. Record
which goals establish each requirement and show the actual native source
connection. Do not count a successful empty crate or inferred arithmetic alone
as completion.

## Permitted author choices and proof scope

Authors choose mathematical views, invariants, semantic generic/library interfaces,
meaningful public behavior/error policies and input-coverage requirements.
Definitions may have mathematical recursion/decreases. Implementations and
connections to fields must verify. Application goals can use ordinary assertions
and contracts. Exact is the default; all inferred helpers in this slice require
soundness, maximality and output exactness, not silent sound-mode fallback.

For acceptance, the production slice must need no local supporting proof blocks,
executable-loop invariants/decreases, manual triggers, mechanical getter contracts
or mutable-selector identity/frame clauses. Ordinary semantic return/input goals
are appropriate; do not rename mechanical scaffolding as a domain property.
Do not insert assumptions, admits, verification bypasses or assumed first-party
behavior. A missing general profile is an adoption blocker to minimize and report,
not permission to manufacture evidence or weaken a requirement.

Executable refactors and additive checked APIs are allowed with explicit
before/after documentation, caller inspection and native tests. In particular a
private checked core may retain a concrete domain error rather than erase it in
anyhow; inspect callers and preserve intended diagnostics/error ordering. Keep
behavioral fixes separate from facts proved about original code. Preserve the
native implementation rather than extracting copies. Commit cohesive verified
chunks in your supplied branch, never push.

Unused generated implementations may be explicitly excluded with external_derive
or the existing equivalent scope mechanism. This is approved scope configuration,
not a callable contract. Inventory every exclusion and ensure no required checked
path uses it. The unused Sum/codec patterns in the generic example are permitted.
ConstraintError's generated Display and std::error::Error implementations may be
left unverified while checking the enum, selector and error order. Do not add
Signed/UncheckedSigned serializer exclusions: that separate proposal remains
unauthorized. Signature serialization and Signed Unit delegation are outside
this goal. Unannotated outside-slice native methods are not silently verified.

Existing third-party opaque boundaries may be reconstructed at this precise
strength: declare accurate external type/marker/trait bounds for codec Encode,
Decode, Codec and AsRef, with no encoding/decoding/equality equation; using_encoded
may require that its FnOnce closure accepts every byte slice but gives no return
or encoding relationship. Hasher has no functional hash equation. parking_lot
raw/read/write guard types and read/write/Deref/DerefMut may have accurately
typed opaque signatures but no stored-value identity, preservation, cache
consistency or synchronization law. Account for every value they can return.
Use existing vstd models normally; no new useful dependency behavior is approved.
Inventory all boundary declarations and do not mark first-party bodies external
merely because they are difficult. Ask/report a concrete blocker if new trust
would be necessary; do not approve it yourself.

## Workflow, evidence and stopping

Use Cargo-native verification of the actual first-party packages with matching
vstd patched to the fixed source. General runner/guide show package metadata,
selected roots and argument forwarding. Typical verification environment:
RUSTUP_TOOLCHAIN=1.99.1, PATH prefixed with the fixed release binary directory,
VERUS_Z3_PATH=<fixed source>/z3 and fresh dedicated target/candidate-cache paths.
Invoke cargo-verus verus verify --offline --locked with the local vstd patch,
selected first-party packages, --fwd-verus-args-to roots -- --infer-goals
--no-cheating --output-json. Native tests use Rust 1.85.1, a separate target and
ordinary erased builds. A pinned engine is already built; don't spend the trial
re-running its entire regression suite. If a cached dependency is missing,
record the concrete environment blocker rather than silently changing versions.

Keep your journal, requirement matrix, authored-input/assumption/exclusion/function
inventory, commands, source/tool hashes, proof reports and bug ledger under
verification/native_adoption_v2_replay. Inventory all compiled native functions in scope
using verus-infer.verification_scope alongside your semantic classification and
per-helper certificates. Include no-verify/filtered/failed publication checks,
repeated builds, a restored positive run and meaningful source mutations of
quorum, round/update, selection, error order and metadata/creation policies.
Distinguish healthy helper certificates from application-goal rejection. Record
actual native Rust tests separately and keep dependencies/closure inspectable.

Track bug candidates with original-code reproduction, expected behavior, reachable
preconditions and realistic upstream exclusions. Separate a failing proof or
unsupported profile from a bug, native bounded counterexamples from universal
proofs, and reachable defects from mathematically possible misuse of an internal
API. Finding a serious bug is not an acceptance requirement. No prior bug hints
or ledgers are supplied.

Persist through ordinary integration/proof failures using the allowed general
references. If a genuine engine/trust blocker remains, minimize it independently,
record completed versus missing requirements, commit your reviewable result and
report the exact next decision. Do not declare the full milestone complete on
partial results. Your final report must give commit(s), reproducible commands,
native bodies actually checked, exact certificate counts, universal requirements
proved, authored inputs, remaining boundaries/blockers, bug evidence and whether
the full finite acceptance was met.

## Fixed tool identities

Validate the clean commit above and these prebuilt artifacts before work.

```json
{
  "verus": "2b4d5ef2734badc6b322f988dee3b4c8855df8ac890c45c80d320622dee1350c",
  "rust_verify": "ae16f1b672379d649687f330e6b9b0ec8b2a082028412eb70ae0ce5f37c3fe8e",
  "cargo-verus": "bfa83472323bf86334835fe6f354ca6c78af0222202b5dab38825d86529e73f1",
  "vstd.vir": "a74d4205823048539cdc7169f4de7542d0e5b6f067255fe0267a2244d628f579",
  "libvstd.rlib": "3d073ecde724bcab308db47b97cccbf6b2aff59017c8369d473bd032dad72833",
  "libverus_builtin.rlib": "6fb6d26b7ab90f76e4f0318a25264d1035deedd58f2301061b4bf8bb47a2e96e",
  "libverus_builtin_macros.dylib": "05ebb3b9b40e89835a13392499f6651c737c17ac7aef212c9d13140d94ec6e32",
  "libverus_state_machines_macros.dylib": "838eed24609508b768834e01d0c1addc2297a8495d20b9e2475b73fcccabf587"
}
```
