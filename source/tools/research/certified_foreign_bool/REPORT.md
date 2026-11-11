# Certified actual foreign bool publication/import

The complete certification path is implemented in this isolated lane. The
unchanged full-domain bool provider now proves and exports, and an ordinary
separately compiled client without infer-goals proves the result through that
interface. False clients, incomplete checks, stale/missing/conflicting evidence
and failed required auxiliaries refuse publication/import. Fresh independent
parent review and composed-engine reconciliation are still required.

Production engine:3a23cdd5, following the early typed-interface milestone
9dd3f3e9 and implementation3bfb7dfc. Test fixture correction:905dd062. Baseline
is d27f4dd9; accepted LOCAL compiler sources at35bc7b12 match that baseline.
Reserved schema39 / policy71 / cache74 are used. commands.rs and
checked_foreign_packing.rs are unchanged. No parent/sibling live state was read,
no arithmetic routing changes were adopted, and nothing was pushed.

## Implementation

The [interface](INTERFACE.md) describes the required representation and gates.
The publisher fingerprints the actual full Function returned by ordinary Query,
independently checks it against freshly inspected whole typed core MIR and its
full-domain contract, and records fresh successful normal helper queries. Full
scope and zero global proof/compiler/deferred errors are mandatory. Export waits
for the final erased Rust compilation; neither52 nor any other fixed query
count is an eligibility condition.

The foreign vector lives outside public-function pruning and is bound into the
exported API fingerprint. Ordinary and unused re-exports propagate it together
with existing model/dependency fingerprints. The importer now takes TyCtxt and
freshly resolves/reinspects core instance/MIR/compiler/core/target before
returning any imported crate to construct_vir_crate or Query. Missing fields,
incompatible formats, mismatches and conflicts refuse. Both actual legacy/new
consumer directions are exercised. Selected explicit and actual cargo-wrapper
outputs are invalidated before Rust parsing, including ordinary consumers.

## Fresh results

| Trial | Verified/errors |
|---|---:|
| Full symbolic bool provider, exact cold/warm | 2/0 each |
| Ordinary imported bool client, no infer-goals | 1/0 |
| False ordinary client, healthy certified provider | 0/1 |
| Ordinary re-export and transitive client | 1/0 each |
| Private-helper-pruned provider and ordinary client | 3/0; 1/0 |
| Unused ordinary re-export and transitive client | 1/0 each |
| Unchanged full-help Item.new, exact cold/warm | 52/0 each |
| Ordinary imported observers; false protector observer | 5/0; 4/1 |
| Matching source-built standard vstd | 2250/0 |
| Goals-off baseline | 0/1 |
| Failed isolated BV auxiliary, explicit/cargo | 4/1 each; no export |
| Erased Rust type error, explicit/cargo | 2/0 proof, then E0308; no export |

The BV failures have no failed main-body function in the report: the ordinary
auxiliary error alone still blocks publication. Thus a healthy helper/main
query cannot conceal a failed arithmetic proposal. All applications use
no-cheating; the standard vstd driver keeps its inherited admissions policy.

Exact cold/warm inputs, ordinary typed inventories, required foreign certificate
and actual helper queries match. Eq candidate
`f3ceb87a72d76daa663122c11364f61be041b535066f88c9629864b1e9ba8654`
comes from a real miss/hit; soundness, maximality and exactness freshly prove in
both runs. Each Eq AIR has three actual check-valid queries and matching bytes.
The raw full API summary fingerprints differ; both self-bind correctly. This
conservative serialization difference is recorded separately and conflicting
completed artifact fingerprints refuse. No production fingerprint check was
removed to make the comparison pass.

The full provider's exact parser/ordinary inventory has39 bodies:19 Exec,
19 Spec and1 Proof, including the foreign helper and both constructors. The HIR
owner inventory has48 entries, with16 selected exec and1 selected proof body.
All selected/excluded/declaration-only owners and typed body hashes are in
RESULTS.json. The raw query index is28 ordinary root checks,21 isolated BV
checks and3 Eq checks, total52. The bool provider has two ordinary bodies and
one native exec owner. Imported exec/closed bodies and the private foreign
helper are absent; the authored open item_word_wf remains public as intended.
An emitted native MIR inspection retains the original typed From call with
one Copy input and no synthetic helper; this is not a native theorem.

The96 retained feature trials cover complete proofs and metadata controls,
ordinary/unused/transitive/private-pruned imports, real matching/mismatched
compiler-option targets, conflicting evidence in both orders, actual legacy
compatibility, serializer refusals, early Rust parse/type errors, disabled and
filtered checks, and stale selected outputs. Actual --VIA-CARGO wrapper entry
also proves/imports the certified provider and covers extra-filename, explicit
metadata filename and single -o selection. This is not a cargo-spawn claim.

All335 focused compiler regressions pass: core Infer299, new certification3,
LOCAL5, Infer-goals12, NonZero/Eq3 and constructors13.
All149 VIR tests pass. Final counts and immutable receipt identities are in
RESULTS.json, audited under both normal and optimized Python with -B.

## Reproduction and limits

[run.py](run.py) records the offline/locked jobs2 build, private CARGO_HOME,
target-dir and separate build-dir, source/tool/fixture freezes and receipts.
The accepted parent bundle's receipt and all eight hashes are authenticated;
only its solver and the named completed archive/index/getopts inputs are reused
for new proofs. The compiler/macros/vstd are source-built here. Build4's
stripping receipt, the final eight-tool bundle, all raw logs/AIR/MIR and complete
fixtures are private under [PRIVATE_ROOT](PRIVATE_ROOT). Use a fresh review
checkout/root for replay; never rebuild or reuse completed output paths without
archiving them. stable.py archives each complete cold output before identical
working-path reuse. audit.py is a read-only normal/-O-safe acceptance checker.

[Trust and manual work](TRUST_AND_MANUAL.md) records inherited compiler/frontend,
solver, standard-model and artifact trust, plus the unchanged three inline proof
blocks, authored packing proof and seven explicit BV assertions. Observers use
only the baseline public constructor/tag and getter/word contracts; no stronger
external constructor permission/protector postcondition or new premise is
invented. [Exclusions](EXCLUSIONS.md) preserves setup, CLI and analyzer failures
separately from logical counterexamples. Goals-off has no foreign capability or
certificate; any ordinary declaration export follows inherited cap-free artifact
trust and is not used as certified proof evidence.

This enables bounded certified bool test publication only. It creates no new
standard model, FromSpecImpl<bool>/obeys law, external body, axiom, FnDef equality
or executable source replacement. Parent's independently accepted arithmetic146/0 (review73ce0a04, root e099f969)
is routing data,
not this lane's result or a certification threshold. Native correspondence=0;
all44+3/101 remain open. Canonical6/0 and HMAC1/1 UNPROVED are inherited and not
rerun. Global publication=false and goal_complete=false.
