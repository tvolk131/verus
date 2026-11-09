# Before accounting for the proposed stateful bitmap goal

This is pre-freeze evidence, not an implementation result, acceptance verdict,
native resize proof, or new Infer capability.

## Exact baseline

The accepted cache82/policy76/schema38 tree has 2,209 nonignored files. Its
captured snapshot and 1,401 immutable external inputs are bound by
baseline82/SEAL.json, SHA256
d686767f32149fc4a73311b7339ddb61599b5b973a821ee087f89361e44a0e22.
No parent production or native application source has been changed.

## Existing authored state proof

The prior evidence/transfer-redb-bitmap-state77 proof already checks the
unchanged get/set/clear bodies against authored representation and frame
contracts. It retains an authored data_index_of tuple contract and two
explicitly authored, backend-checked broadcast bit-vector lemmas. Only
select_mask is inferred. It does not prove native resize or infer exact
mutable-record state summaries. Those existing results cannot be recounted
as this goal's progress.

## Existing standard-library and assistance support

The baseline vstd already specifies generic native Vec::resize using the
Clone relation and an exact shrink/growth sequence contract. It also models
native RangeInclusive iteration. No new trusted Vec::resize axiom is needed
merely to support the original call.

The baseline production pipeline already has range-loop proof assistance,
including a proposal-only native inclusive-range path, in addition to its
narrow scalar counted-loop inference. The precise compound bitmap-body
eligibility and success must be measured; the existence of those modules is
not evidence that resize already proves.

## Parent arithmetic probes

All four ordinary Verus mathematical proof methods verify with empty bodies:
ceil-word coverage across the full u32 domain, safe partial-word window bounds,
the fact that shrinking never needs extra storage, and div/mod bit decomposition.
This uses no new engine, authored local assertions, extra arithmetic lemma
calls, or --infer-goals. These are pure mathematical characterizations, not
native helper-body correspondence or native resize verification.

Both false or weakened statements reject at their intended postconditions:
removing the last-word bound invalidates the word-window theorem; requiring
strictly excessive ceil coverage is false at zero and exact word multiples.
Each negative reports three verified methods and one failed method. The
original reports four verified methods and zero errors.

Evidence: parent-bounds-before82-v2/receipt.json. Package SEAL SHA256
7cca081c0473a3616d633200992e33a03076b65982fe1341b39955df0f605d7d.
The first attempt's macro parse mistake is preserved separately as an excluded
setup failure and receives no proof credit.

Consequence for scope freeze: do not claim inventing standalone ceil/div/mod
arithmetic as a new capability. Actual work must remove measured low-level
proof plumbing in the unchanged stateful loop and its native composition.

## Confirmed native ceiling-division frontend gap

The exact required_words body fails on the native elements.div_ceil(64) call,
both with an Infer annotation and with an ordinarily authored exact result
contract. The runtime reports that core's u32::div_ceil is unsupported and has
no Verus specification. This is a native API/correspondence gap, distinct from
the already automatic mathematical ceil-word theorem above; it is not an
application counterexample.

The inference-aware failure starts with a genuinely valid freshly generated
old inference export, then removes it. This confirms existing fail-closed
publication behavior; it is not credited as new cleanup capability.

Evidence: required-words-before82/receipt.json. Package SEAL SHA256
1e9245d4dc63e79cdc123a5f12f1ff067741364e1b3d7c493ec4e39765e698fc.

Scope freeze must make an explicit decision about any added native unsigned
ceiling-division standard-library correspondence. It may not be hidden as
an already existing model or described as a proved Rust standard-library
implementation. No new correspondence has been installed or authorized by
this before-evidence file.

## Complete original source ledger

Ten native items, including the original inclusive resize loop, mutable slot
borrow and Vec growth call, are source-line/item/body bound in
native-scope-before82/source-ledger.json. Source ledger SEAL SHA256
dbab4f4427c9c327ad4d01c84b60ba7e9d83ce77049443cb2d1180bf8d7568bd.
This is a source capture, not a frozen implementation scope, extraction-compiler
proof, body verification or new inferred capability.

## Parent native execution oracle

The exact extracted native methods pass, per 64-bit native debug/release mode:

- 169,884 direct resize scenarios and 37,678,212 live-bit observations.
- 148,104 shrink/regrow scenarios and 32,847,672 live-bit observations.
- 911,196 sampled native set/clear compositions with exact stored-word checks.
- 133,120 independent range-mask membership observations.

The independent reference updates individual bits rather than reimplementing
the native word-window algorithm. Full stored-word equality checks cover
padding and spare storage, in addition to live-bit reads and logical length.

Three separately compiled bad-body variants fail native assertions in both
modes: using clear instead of set for a full update, skipping the first word,
and returning prematurely on growth. All original/variant builds succeed;
semantic failures are execution results rather than compiler-setup failures.

Evidence: native-baseline82/receipt.json. Package SEAL SHA256
2ac413af268662692816990bb4fd4042f87ab6ae1a77c920450182154b84ba52.

These are finite tests on selected lengths/patterns. They do not establish a
universal theorem, 32-bit native execution, maximal safe domains, inferred
state exactness, new automation, or independent review. The full u32 theorem
remains a required target, not a conclusion from these observations.

## Still required before implementation

The isolated feasibility researcher must supply the actual unchanged resize
baseline failure, a justified semantic invariant/allowed annotation budget,
native frontend and source-fidelity qualifications, the remaining compound
proof gap, and a concrete reusable automation proposal. The parent must then
freeze scope and every required acceptance/rejection control without changing
the user-approved full-state and regrowth requirements.
