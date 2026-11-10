# Incomplete checked foreign bool conversion implementation

**No production capability or available foreign conversion law was implemented.**
Early interface commit `ae3cd734` freezes the intended typed stages. The lane
implements an actual-MIR diagnostic translator and independent structural
correspondence checker, then stops before registration/proof/publication/import.
This is the task's unsuccessful, fail-closed handoff, not a claim that those
interfaces are impossible to implement. `BLOCKERS.json` names each missing typed
interface and its concrete frontend/query/export/import integration sites.

## Fresh evidence

| Trial | Result | Meaning |
|---|---|---|
| Actual original `u64::from(bool)`, symbolic full bool domain, no requires | **0/1** | True-property library contract gap |
| Local bool cast mirror | **1/0** | Existing cast encoding only; no foreign-body proof |
| Full original Item.new with unchanged manual packing support | **36/1** | Only original protector debug assertion fails |
| Independent packing support without Item.new | **36/0** | Manual support and original tag/get/new/Eq support remain healthy |
| Typed MIR/VIR diagnostics | **20 rejected controls**, four rejected nearby call shapes | Structural diagnostics only, no solver/body certificate |

Original Eq's soundness, full inherited-domain maximality and output exactness
are freshly PROVED in both full trials, with requires true. The full constructor
retains every assertion/debug assertion and `u64::from(protected)`. Its failure
is not a healthy checked-instance negative; that instance does not yet exist.
No full-crate native certification is inferred from a failed crate.

The actual core instance is selected through the semantic diagnostic From trait,
core lang-item crate identity, associated method/impl relationship, caller typing
environment, exact bool/u64 Self/signature, safe Rust ABI and monomorphic Item
args[]. Neither numeric DefId nor impl spelling selects it. Every MIR operation
is matched through rustc enums. The body is exactly two non-dropping locals, one
argument, one non-cleanup block, one Copy(bool)-to-u64 IntToInt assignment to the
return local, then Return. The installed source macro and compiled core artifact
hashes remain distinct; no source-to-stdlib rebuild equivalence is asserted.

The private diagnostic constructs real ordinary VIR with a fresh internal
function/binder/read/span identity and the inherited bool-cast If encoding.
An independent checker reads original MIR and compares this AST's types, modes,
copy, binder, branches and return independently of the proposed ensures. Three
proposal controls (missing ensures, restricted false domain, false ensures)
reject while body correspondence remains healthy. Five body/mode controls and
twelve MIR/local-mapping controls reject. Wrong output type, generic unresolved
From, user From-like trait and indirect function pointer reject separately.
These checks are not ordinary mode/WF/body proofs. The diagnostic helper is never
registered, called or equated with the original FnDef. General function-value/
coercion source mapping is not implemented and receives no new law anywhere.

## Exact body scope and adapters

The original `source.py` generator and pure `inventory.py` reader are reused
after inspection; no old epoch/audit driver is executed. The generator extracts
pinned bodies, overlays authored contracts/observations and inventoried ghost
packing help, and verifies original executable tokens after ghost erasure.
Private BorTag(NonZero<u64>)/Item(u64), all seven BorTag derives, `.map(BorTag)`,
`.get()`, original tag unwrap and full constructor spelling remain unchanged.
The normal-return <=TAG_MASK helper bound is not an application budget or proof
of all out-of-range/error/unwind outcomes. Manual help is unchanged in MANUAL.md.

Support's fifteen checked HIR exec bodies are BorTag::{clone,eq,new,get},
Item::{clone,tag,perm,protected}, Permission::{clone,to_bits,from_bits}, roundtrip,
compare_equal, lookup and boundaries. The ordinary generated BorTag::ctor% and
Item::ctor% bodies are additionally present and checked; they are not omitted
because they are absent from HIR. packed_fields_raw is the sole checked proof
body. Full source additionally checks Item::new and fails its protector assertion.

Excluded native bodies are BorTag Hash::hash, Eq::assert_fields_are_eq,
PartialOrd::partial_cmp, Ord::cmp; Item Hash::hash, PartialEq::eq,
Eq::assert_fields_are_eq; Permission Hash::hash, PartialEq::eq,
Eq::assert_fields_are_eq and Debug::fmt. They receive no new native proof claim.
Permission's prior Structural marker remains an inherited boundary. The exact
raw impl paths, body hashes, all native selected/excluded/declaration-only items,
all ordinary spec/exec/proof bodies and function details are in RESULTS.json.
The bool trials each contain only all_bool as an ordinary executable body.
Diagnostic rustc fixtures are identity/shape inspections, not selected Verus
executable proofs. Their custom FromLike implementation is a rejection fixture,
never a replacement for the actual core conversion.

## Missing implementation boundary

1. Pre-register a checked actual instance before Query's function-stub collection;
   map only the exact typed direct call while retaining original erasure/lifetime
   and trait identity. A synthetic static AST alone cannot supply this mapping.
2. Feed it through normal WF/mode/trait/lifetime and ordinary solver body checks.
   Structural correspondence and syntactic proposal validation cannot substitute
   for these checks. No inference checks are claimed for the foreign body.
3. Force post-proof publication even outside infer-goals and after private-helper
   pruning. Existing ordinary exports can precede proof completion. A new foreign
   dependency cannot silently use that path. Carry complete body/contract/MIR/
   semantic/compiler/core/target and transitive dependency fingerprints.
4. Freshly resolve/revalidate those dependencies before imported closed facts are
   available. Existing import_crates has no TyCtxt and its trusted-model mechanism
   fingerprints known models, not this actual foreign MIR instance. Preserve its
   existing strict producer and closed-summary checks. Parent owns version changes.

No healthy-instance false-result, cold/warm availability or certified import/
publication test is claimed: those require the missing checked instance. No
FromSpecImpl<bool>, obeys_from_spec=true, assume_specification, external-body stub,
vstd model, axiom, source-runtime replacement or new no-unwind premise was added.
The frontend translation's TCB is not a full Rust/borrow/provenance theorem.

## Receipts and operational limits

Sixteen completed owned epochs are in attempts.jsonl. Full argv/environment,
controller/source snapshots, exits, raw MIR/VIR/logs, private exports, binaries
and full receipt/product manifests stay private at those exact paths. RESULTS
authenticates every retained receipt/product and all selected evidence guards.
The accepted combined bundle receipt is SHA256
`2e3fcc58f6fca9364061329104710937ec2aac6aab95a5c48d068c1ca6ee9bb7`;
success, schema38/policy69/cache72, vstd2250/0 and **all eight** tool hashes are
checked before epochs and in audit. Parent tag-v2 receipt is separately
authenticated as `ba75225dcf12c21a8f7c17fcfdef493d1083aa3ed2c865d74740b0a66de476aa`.
Compiler-version epoch confirms the pinned Rust1.99.1 commit. No modified engine,
compiler/macros or vstd proof build is claimed. Only own VIR and diagnostic tools
were built, offline/locked with two Cargo jobs and selectively copied locked
archives/index entries plus an immutable getopts Git database.

Two initial cache-preparation failures occurred before the owned receipt wrapper:
the overly broad copy first lacked chacha20-0.10.1, then the VIR dependency-closure
copy lacked rand_core-0.10.1. They are setup failures, not proof evidence, and have
no pre-epoch frozen receipt. Later successful builds show that the latter archive
is optional for this actual build; it is not asserted to block VIR or the engine.
A one-off artifact locator query also failed before the wrapper. These evidence
limitations are disclosed, not reconstructed as successful frozen epochs.
The first receipted offline attempt lacked a workspace index entry and failed.
Compiler API/import errors are separately retained. One library build exited0
but its controller changed while running, so final guards rejected it; it is
explicitly excluded, never accepted as evidence. A subsequent library build used
the own checkout's ignored build-dir because of Cargo's separate build-dir
setting; its external intermediate products were not fully inventoried. The
final fresh library build explicitly confines both target-dir and build-dir to
its private root and retains external product hashes. Earlier builds are not the
final accepted diagnostic-library artifact. No sibling output was inspected.

Sandbox startup failed with bwrap RTM_NEWADDR; the exact require_escalated tool
mechanism was used. No host/network changes, installs, shared cache/toolchain/
config writes, subdelegation, push/PR, process queries, historical checkout reads
or sibling waiting occurred. Parent's broad unchanged suite was not rerun.

Audit: `python3 -B -O source/tools/research/checked_bool_from_mir/audit.py`.
Explicit Python gates remain active under -O. Final artifact guards are produced
by final_guard.py. No full receipts/raw logs/copied upstream sources or binaries
are committed.

Publication=false (baseline support has a private verified export only), native=0,
goal_complete=false. The original full-bool From property and full Item.new remain
UNPROVED. Canonical full6/0 remains parent evidence; full original HMAC over
arbitrary initialized[u8;128], every lawful key length including0/>128 and both
XOR prefix/unchanged suffix remains1/1 UNPROVED. Both full frozen properties
still need actual checked automation and independent whole-goal acceptance;
44minimal+3phase/101writer effects remain OPEN. No whole-goal or reference-C/F
claim is introduced.
