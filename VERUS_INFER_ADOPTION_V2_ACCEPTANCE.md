# Native adoption v2 acceptance

Status: complete, October 30, 2026. The independent clean-main replay and parent
audit meet the frozen [v2 charter](VERUS_INFER_ADOPTION_V2.md). This completes a
finite adoption milestone, not arbitrary Rust inference or protocol verification.

## Reproducible result

The independent adopter started at pristine Aleph
`f35c7bba6d628bcafdf0d65dd471cc6f868ec034` in a new app-managed worktree, using
clean fixed Verus `9b24988eff4a250188ff11445115560c54cf4688`. It received the
general guides and explicit semantic requirements, without prior Aleph proof
implementations, evidence, bug hints or parent messages. Its implementation is
`7f1b59332b479493ca1d321f51375567ce375e3b`; evidence is committed as
`0574b4a84b8c20de434ce852c772e333ff7e6d1f`.

The parent independently replayed that completed source with fresh Cargo outputs
and candidate cache. Both the recorded result and fresh replay establish:

- **156 verified results, zero errors** across the actual Cargo roots;
- **35 exact helpers**, all **105 required soundness/maximality/exactness checks**;
- **85 checked executable bodies and one generated proof body**, including
  application coverage functions and actual generated implementations;
- separately, **192 native unit tests and one doctest** (181 consensus, 11 crypto);
- ten deliberate native-source mutations rejected by application verification
  after healthy helper certificates, failed interfaces withheld, no-verify and
  filtered publication rejected, and a restored full positive run.

Types contributes declarations and imports, with zero executable bodies counted.
The compiled inventory lists all 830 native functions, including outside-slice
items; successful Cargo compilation is not counted as proof of those methods.
All 80 recorded source/manifest/lock inputs match current files and both accepted
proof/native receipts. All eight pinned compiler/library artifact hashes and Z3
identity remain unchanged; the fixed compiler and Aleph worktrees remain clean.

## Frozen semantic and domain audit

| Requirement | Checked native connection |
|---|---|
| Original quorum and exact domain | Actual NodeCount expression/derives, mathematical quorum goal and certified-domain equivalence to `n <= usize::MAX/2` |
| Positional collections and Clone | Original NodeMap methods/iterators and real Clone derive; absent slots, bounds, content, complete updates and an adequate checked generic Clone interface |
| Prefix traversal and rejection | Inferred production fill loop; ordinary prefix policy covers zero/empty/full and unchanged whole state on rejection |
| Collector initialization/advance | Field-connected validity, exact initial constructor and checked generic clone transport; all valid rounds below MAX, including MAX-1 to MAX |
| Ingestion/replay/selection | Original getter order and update decisions; full update against the actual captured opaque hash; newest/replay policy, complete selected map and quorum-before-own-parent error order |
| Creator state and raw/checked domains | Nonempty bounded/indexed/sized history, actual growth/range loops, generated mutable-selector frames, earlier-history preservation, all raw rounds below MAX and checked terminal/membership rejection before mutation |
| Creation and coverage | All valid states, arbitrary hash payloads, every requested u16 round, positive frozen-domain counts/in-range creators, and initial arbitrary IDs/zero membership; exact coordinates, selected parent projection, bounds, own parent and quorum |
| Actual output reaches validator | `creation_metadata_goal` obtains actual `create_unit` output, then calls `validate_eligible`, which calls the original public validator |
| Metadata/error policies | Actual projection/iterator/initial/noninitial bodies; full native error ordering and numeric eligibility, with hash values opaque |
| Compared hash pair | Initial bad-hash error retains stored value and conditional inequality with the reported value under the existing PartialEq law; no law flag or deterministic hash equation assumed |
| PreUnit and FullUnit connection | Complete supplied ControlHash forwarding; actual four FullUnit Unit methods and coordinate defaults checked against an authored law whose default is false |

The raw creation contract restricts member count and creator only for positive
rounds; initial creation retains the broader frozen coverage. Its separate
numeric-validator application goal uses the quorum domain, without turning that
restriction into an initial-creation precondition. Older additional parents remain
allowed. Checked APIs preserve the complete declared state on errors. Generic
hash Clone may change values: advancement claims its checked clone relation and
round/occupancy transport, not hash identity.

## Manual inputs, source connection and trust

Authors supplied six views, two validity predicates, coordinate observation and
arithmetic interfaces, sparse iterator observations, and semantic behavior/error/
coverage goals. Ordinary assertions state application requirements. There are no
local supporting proof blocks, executable-loop annotations, manual trigger hints,
mechanical getter contracts or authored mutable-selector identity/frame clauses.
Generated annotations and their normal proof checks supply that scaffolding.
Metadata contracts explicitly connect positional counts with the sparse iterator
observation; these mathematical interface guarantees are inventoried author input
and are proved, rather than assumed or silently attributed to inference.

The generic source inventory was rerun over all 58 supplied files. Its original
two trigger findings came from unchanged documentation mentioning `trigger_tasks`
outside the checked slice. They were false positives, not proof hints. The current
standalone inventory distinguishes identifier hints from string literals, still
finds hints inside native contract attributes, and passes its parser controls.
The regenerated inventory finds no forbidden local proof aids. Parsing remains
an aid to semantic review, not a proof certificate or module-discovery mechanism.

The native refactors are documented and tested: loops moved into actual called
production helpers; primitive counter-field addition replaces the equivalent
generated AddAssign call that inference cannot compose; a local result binding
exposes clone transport; and private creation returns its concrete existing
ConstraintError instead of anyhow erasure. The unchanged production caller logs
Display text and does not inspect/downcast anyhow. These changes add no assumed
first-party implementation and are not presented as protocol bug fixes.

The fourteen explicit opaque dependency declarations retain their authorized
strength: accurate codec/AsRef/lock traits/types and typed read/write/deref
signatures, with no useful output equations. Hasher has no postcondition. Existing
vstd models and checked library lemmas remain the standard boundary. No hash,
encoding, equality, cache-consistency, stored-lock-value or synchronization law is
added. Unused derive exclusions grant no callable contract. Required Clone and
scalar operators/conversions are checked. Signed/UncheckedSigned serialization
and delegation remain unchanged and outside scope.

No confirmed reachable production Aleph defect was established. The independent
bug ledger distinguishes original arithmetic/terminal panic reproductions from
their frozen domains and unresolved upstream reachability. Native tests are not
universal proofs, and negative mutations are not discoveries in original code.

## Evidence and remaining limits

[VERUS_INFER_ADOPTION_V2_EVIDENCE.json](VERUS_INFER_ADOPTION_V2_EVIDENCE.json)
records the independent commits, final artifact/source hashes, parent fresh-run
receipts, inventories, controls, review decisions and local report paths. The
Aleph evidence directory is `verification/native_adoption_v2_replay`; its runner
reproduces Cargo-native proof/native checks and controls without old solutions.

The first independent attempt remains partial at `a014fae`; the separate parent
closure at `e5619e3` remains a parent result. Neither is relabeled as this final
independent success. The automatic replay monitor is disabled. All work is
committed locally; nothing is pushed.

Future work can reduce the generated AddAssign composition refactor and broaden
supported mutation/loop/recursion profiles. Authors still choose abstractions,
invariants, application policies and honest trust boundaries. Protocol agreement,
liveness, async/network behavior, cryptographic correctness, signature delegation
and hash-cache consistency remain outside this completed milestone.
