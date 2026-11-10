Scoped ACCEPT for combined constructor FnDef and genuine NonZero PartialEq
inference at source base 3aa7f49a04a5e93c2ede882f2b947aac6e39263d.
No actionable production finding in this bounded independent review. The previous
stale zero-arity tests are fixed in the reviewed base. Production files, vstd,
policy, shared toolchains/config/caches and other evidence directories were not
edited. No delegation, install, network operation, push or PR was performed.

Fresh matching evidence:

- Own offline/locked compiler/macros, two Cargo jobs, Rust1.99.1; normal matching
  vstd 2250 verified / 0 errors. No --no-cheating was forwarded to vstd's inherited
  admissions. The acceptance bundle has eight frozen tool hashes, separately
  bound to the successful compiler and vstd receipts.
- Genuine unchanged private-field BorTag with all seven native derives: cold and
  warm compile/export both 6/0. Exactly component::BorTag::eq is inferred, in
  exact mode, requires true, all three required checks PROVED. Warm candidate
  reuse is a cache hit and reruns all three checks. No proof body, proof block,
  snapshot, client theorem call or new assumption in this component.
- Full-width MAX/high-bit diagnostic and optional --infer-goals off both 6/0.
  Off still explicitly requests infer_derive, so it is not an inference ablation.
- Ordinary separately compiled imported client 4/0, exactly client::equal inferred
  with all three checks. Its two exec bodies are equal and goal. Its imported
  original Eq, closed observer and closed eq_spec bodies are actually absent in
  the post-import VIR. Acceptance uses the certified signatures/summaries and
  dependency/model fingerprints, not access to those closed bodies.
- False returned relation 5/1 at compare_equal's postcondition, with all three
  original Eq obligations healthy. No interface published.
- Constant-zero observation and reversed eq model each 1/2: original Eq soundness
  and exactness NOT PROVED, inherited-domain maximality PROVED. These are bad-model
  rejections, not healthy-helper negatives. They stop before ordinary-body
  certification; the other present bodies are not claimed checked in these runs.
- NonZero<u32> and PartialOrd/Ord requests reject at the supported-shape boundary
  (0/0, frontend/profile rejection). No helper or interface is published.
- The unchanged genuine policy68 standalone provider's --import rejects at the
  policy/schema/producer compatibility gate under this policy69 compiler. Since
  producer also differs, this does not isolate policy from producer. A client
  naming its Rust symbols hit E0463 even with its original dependency bundle;
  those attempts remain typed compatibility blockers, not policy/semantic tests.
  The accepted interface-only consumer uses the original .vir/.rlib unmodified.
- New comparison tests 3/3, existing comparison tests 7/7, constructor tests 13/13.
  All application test calls already explicitly request --no-cheating. Their
  directly replayed test binaries are frozen as incoming tools, and all 23 pass.

Original constructor + get + Eq composition passes 10/0. The original runtime
bodies remain NonZero::new(i).map(BorTag) and self.0.get(). Public closed authored
value semantics describe them. Every u64 has the zero/None and nonzero/Some
behavior; get round-trips all inputs, including zero through the None branch.
The component compares two newly created tags and already-created tags including
MAX and the high bit. No low61-bit restriction, replacement callback/runtime body,
proof block, snapshot, inverse-view axiom call or client Eq-to-identity lemma.
This is a component subproof, not the original Item.new/protector/Ord or whole
application. Item::tag was not extended in this review.

Scope accounting is the union of two reports, not the old three-body inventory.
The HIR-native report lists Clone, derived Eq and compare_equal. Original VIR and
ordinary func-details additionally contain the checked generated
component::BorTag::ctor% body. These four exec bodies plus Eq's two extra precision
queries explain 6/0. The exact nine local ordinary function-detail identities
also include five semantic definitions. All are explicitly checked by the
controller; no generated constructor was discarded. Composition has eight exec
bodies (Clone, Eq, ctor%, new, get, all_u64, compare_created, compare_width) and
10 queries. Its seven HIR-native exec bodies plus ctor% are required separately.
The retained excluded native methods are Hash::hash, Eq::assert_fields_are_eq,
PartialOrd::partial_cmp and Ord::cmp; they receive no new verified claim.

Body identities:

- Native generated Eq is exactly `{ self.0 == other.0 }`, SHA256
  1774afdac1844226361985602334fc3b076cb8194e4f4efcc8c0b18fd2eb229b.
- Original combined Eq VIR body SHA256 (expression bytes, without added newline):
  00626e3d5e107ad8dc8f7efe77c282d5813d104f24947e79fc48bfef991cde89.
  The private retained body file with its trailing newline hashes
  c518c7da9277da032411650fbae6e606e39ae74dc437b42e983107c9da8a4fef.
- Generated ctor% original body SHA256:
  f7f32bced5bfe242aba60f1d1d90f8cb4d7b39afa5a5f46e429a0facafceb891.
- Raw original methods are component::impl&%2::clone and
  component::impl&%5::eq; composition new/get are impl&%12. Friendly helper names,
  full source hashes, all raw body hashes and candidate IDs are in RESULTS.json.
  VIR hashes include source spans and are not expected to equal the earlier
  standalone path-bound VIR hash. Native Eq body bytes are identical.

Manual review and trust inventory are in MANUAL.md. The new projection recognizes
core's semantic NonZero<u64> identity and admits only the bounded owner shape.
The generated observation/model is a candidate; original-body obligations remain
mandatory on both cold and warm paths. Registry fingerprints observer definitions,
original source, datatype shape and dependencies. Import/export retains full
certification/publication barriers. No standard-library model or vstd assumption
was changed. Manual review is bounded, not a formal proof of this implementation.

Evidence integrity and excluded attempts:

The early typed checkpoint is commit 53ece81f4d9db4ecf0cf735e8a44b50e2a5aa2a1.
51 delivered completed epochs, 638 retained relative products across those epochs,
and the prescribed canonical/fix receipts authenticate. The review freezes all
2915 own source/dependency files, actual tool inputs and controllers; application
plans bind helper/domain/expected raw and friendly body identities before runs.
Every own command has durable argv/cwd/environment, exit, log/product hashes and
final guards. Accepted controller snapshots and per-phase source/VIR/export copies
are private. Cold and warm exports may serialize differently; each is audited
against its own retained product hash. Mutable build targets are diagnostic;
the frozen eight-tool bundle is the acceptance tool input. Python uses -B -O and
explicit exceptions; no acceptance condition relies on Python assert.

Retained exclusions: initial bwrap RTM_NEWADDR startup (exact require_escalated
mechanism used, no host changes); missing optional locked archive during setup;
private archive placement in only one Cargo registry identity (offline exit101);
retry overwrite of read-only own copied Git pack; initial build guard wrongly
including Cargo's mutable private .global-cache; two HIR/raw-name inventory
controller errors on otherwise6/0 proofs; generic encountered-error versus
frontend-error classification; bad-model early-stop query accounting; symbol-using
old-provider E0463; spec-only observation in an exec if in the first composition
probe; identity parser counting imported Eq functions; final-audit product-path
and unused-script snapshot bookkeeping. These do not supply semantic acceptance.
The initial compiled build's invalidated guard has its own excluded receipt; the
subsequent offline build command has valid final guards. No verifier panic or
resource failure is counted as semantic evidence. No file substitution, metadata
editing, race or security experiment was performed. Only own checkout/history,
immutable Git objects and explicitly named completed artifacts were inspected;
no historical checkout or live parent/sibling build/process was read or polled.

Policy69/cache72/schema38 are the combined boundary; accepted exports/dependents
were regenerated privately. No old-policy success is reused. Raw sources, logs,
binaries, dependency archives, full freezes and receipts stay private under the
work location in work.json. Only compact evidence/controllers are committed here.

Ordering remains UNPROVED. Publication=false; goal_complete=false; native=0;
44 minimal +3 phase /101 effects OPEN. Parent still owns full33Infer,
canonical/observer gates and fresh whole-goal combined acceptance. Canonical
all-u64/arbitrary-initialized-exclusive[u8;10] inherited6/0 does not follow from
these probes. Exact bitcoin_hashes0.14 HMAC arbitrary initialized[u8;128], ALL
lawful keylen including0/>128, BOTH XOR prefix and unchanged suffix remains1/1
UNPROVED. No helper success establishes Rust safety, physical permissions or
original reference C/F.
