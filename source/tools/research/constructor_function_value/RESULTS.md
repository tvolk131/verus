# Constructor function-item frontend capability

Base: `bc3317f327f99b2f8b18d1268c8e1a8e0c844445`.
Early frozen interface: `935a8eab`; implementation: `c15b1e53`; checked
Rust item identity / typed-field correction: `0e61102d`.

The original executable body `NonZero::new(i).map(BorTag)` now verifies for every
`u64`, with no precondition: the result is Some iff the input is nonzero, and its
actual `NonZero<u64>` payload view equals the input. The fixture contains no
application closure adapter, proof block, assumed callback law, or infer annotation.
The full native `new` body bytes match the pinned compiler source; their SHA256 is
`78b385b088022a0e3b0abc5968018a26f0590d2b221a99a8f2e40ccfb1aaeead`.
The fixture omits unrelated native derives/methods; this is a typed original-body
subproof, not verification of the Miri crate or derived comparisons.

| Check | Actual outcome |
| --- | --- |
| Authenticated inherited baseline, same literal fixture | Frontend unsupported, exit 1 |
| Own matching compiler/macros, two jobs, offline private cache | Build success |
| Own matching vstd, ordinary verification | 2250 verified / 0 errors |
| Original new, inference off, `--no-cheating` | 2 verified / 0 errors |
| Original new, inference on, `--no-cheating` | 2 verified / 0 errors |
| Fresh native-pinned acceptance, Python `-B -O`, off/on | 2/0 each, all guards passed |
| Focused integration suite | 13 passed / 0 failed |
| Direct exact-command case receipts | 5 positive; 2 proof rejections; 3 frontend unsupported; 3 Rust rejections |
| Actual NonZero::get roundtrip direct case | 4 verified / 0 errors |

The two checks counted for the original fixture are `bortag::BorTag::new` and the
ordinary generated `bortag::BorTag::ctor%` body. Both on/off inference reports have
`generated_functions=0`, `functions=[]`, and every assistance/helper list empty.
The enabled run still lists inherited trusted-model fingerprints; these are not
new helpers or an inference gain. See `evidence.json` for exact identities,
commands, exit codes, log/product hashes, direct-case classifications and receipts.
Raw logs, manifests, binaries and source snapshots remain private beneath
`/tmp/constructor-function-value-t3-3c5e9110/`.

## Translation and bounded support

`rust_to_vir_ctor.rs` registers an ordinary exec function for each eligible local
Rust tuple constructor. The function has the actual constructor DefId/path and
inherited ADT type/const parameters and bounds. Ordered field parameters feed an
ordinary VIR `Ctor`; its exact result equality is proved from that checked body.
Body reads use the ordinary copy/move and resolution machinery. All generated
nodes have fresh span IDs, scoped binder identities and no captures. Internal
nodes have no HIR counterpart; their raw spans are marked internal while retaining
diagnostic source data. Original application HIR/MIR remains unchanged.

`rust_to_vir_expr.rs` accepts the path only when its Rust type is the very same
registered constructor FnDef. Normal FnDef machinery then provides its Fn-family
interface and checked requires/ensures. No closure type is substituted, no
function types are equated, and no new axioms, external-body stubs, assumes, vstd
contracts, inference policies, cache/schema versions or global versions are added.
`FunctionOrConstInfo::Constructor` routes the complete generated function through
ordinary initial/full WF, mode, resolution and proof checks; Rust checks the
unchanged executable application, including move, lifetime, visibility and trait
constraints.

Verified coverage includes local tuple structs, tuple enum variants, `.map(Self)`,
type and const instantiation, bounded parameters, zero-field tuple constructors,
move-only fields, lifetime-bearing shared-reference fields, public constructor
access and checked higher-order application. False variant, first/second field
order, false postcondition and missing higher-order requires tests are genuine
proof rejections. Invalid moves, escaping lifetimes and private constructors are
Rust rejections. Mixed/ghost-mode fields, type-invariant-bearing constructors and
function-pointer coercions are clear unsupported frontend rejections, not semantic
negatives. Opaque/external/imported constructor items are outside the new path.
Mutable-reference/HRTB/opaque-return callback shapes have no new coverage claim;
existing frontend restrictions and checks still apply. Registration is early and
covers eligible local constructors even if not used as values.

## Inherited trust and remaining work

`vstd/std_specs/option.rs:209` assumes Option::map's callback precondition on Some,
preserves Some/None, and ensures the callback postcondition for the mapped payload.
`vstd/std_specs/nonzero.rs:69` assumes the real NonZero::new branch/payload contract;
`:97` assumes NonZero::get returns its view. NonZero is an external opaque type
with inherited view/zero axioms at lines 43–67. The focused roundtrip case checks
actual NonZero::get via the unchanged getter expression and recovers the input for
every u64. No new standard-library assumption was needed. Matching vstd was built
normally, without forwarding `--no-cheating` into its inherited axioms. Every
application and regression verifier trial uses `--no-cheating` and normal checks.

Compiler: Rust 1.99.1, commit
`58cf8f9c35b355199727627d07ae34a4e2846d67`. The canonical receipt's prescribed SHA
and success, all eight immutable baseline bundle hashes, all nine rebound engine
source hashes and all ten native source pins were authenticated. Independent
review and parent integration/combined acceptance remain pending. No sibling
comparison-lane file was needed.

Full canonical/HMAC/native statuses are unchanged: canonical 6/0 is inherited,
not newly re-established here; HMAC remains 1/1 UNPROVED; Native=0 and 44 minimal
+ 3phase / 101 writer effects remain OPEN. No helper/kernel/source-hash success
establishes runtime permission, original C/F, native safety or full completion.
`publication=false`, `goal_complete=false`.

## Failed trials and setup deviation

Receipts preserve the initial missing-rustup launcher failure; a missing private
Git dependency cache; two typed compiler failures; the first focused suite's
incorrect expected lifetime wording / unreachable second assertion count; and a
direct-receipt classifier's incorrect expected higher-order and implicit-pointer
diagnostic strings. Completed successful direct receipts were authenticated and
reused when finishing the classifier, with no tool/source replacement.
These were corrected separately. No frontend error or panic is classified as a
semantic rejection, and no resource/proof failure was hidden as success.

The normal sandbox failed before launch with `bwrap ... RTM_NEWADDR`; execution
used the tool's exact escalation mechanism. A broad initial instruction-discovery
`find .. -name AGENTS.md -print` was interrupted before output (exit 130), then
replaced by exact ancestor/local paths. It was an instruction-discovery scope
mistake; no sibling source contents or build/process state were read. An oversized
private cache copy was explicitly stopped and replaced by copying locked cached
archives/metadata and the pinned Git dependency. No install, shared-cache/config
edit, network/host modification, subdelegation, push or PR occurred.
