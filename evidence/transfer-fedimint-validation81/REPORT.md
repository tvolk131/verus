# Native fee-validation automation — cycle 1 candidate

Status: implemented and locally validated; **independent review and parent integration are pending**.
This is the Fedimint half only, not completion of the two-project goal.
Candidate identity is cache81 / policy75 / schema38. Parent integration owns82/76.

## Scope and source fidelity

The parent froze `evidence/two-project-mechanical-automation/FEDIMINT_SCOPE.md`
before this implementation. Native Fedimint stays read-only at
`b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`.
The exact complete native `Amount { msats: u64 }` and
`PaymentFee { base: Amount, parts_per_million: u64 }` declarations are retained.
The relevant real Rust `Clone, Copy, Eq, PartialEq, Ord, PartialOrd` Amount derives
are retained. Unrelated serialization, encoding, hashing, default and diagnostic
derives, imports and unselected methods are omitted explicitly; these are not
whole-module or whole-application proofs.

| Unchanged original native helper | Complete item SHA-256 | Inferred domain |
| --- | --- | --- |
| `Amount::checked_add`, amount.rs:118 | `87524f34513de2ac447c5bbd0f1f62cefff41732b3a896aadfb8bf974d8a1477` | `true`, all u64² |
| `PaymentFee::checked_add`, gateway_api.rs:252 | `6ad96dedd30054c5632c26b1fe5a5704caa0034d09946833bb34ab6945081aff` | `true`, all u64⁴ |
| `PaymentFee::is_within`, gateway_api.rs:243 | `075bc37005bd569abcefde7a9493774e8cc70c470970aaa37b8096e3ba8844ed` | `true`, all u64⁴; shared roots may alias |

There are **no authored provider contracts on these helpers**. Both native `?`
expressions and their different intermediate/final payload types remain intact.
`None` on either/both overflows is a safe ordinary result, not an invalid input.
The actual `is_within` body is checked, not a componentwise executable surrogate.
The checked native scope explicitly includes the real generated Amount `eq`,
`cmp`, and `partial_cmp` bodies. Their comparison law is not an application axiom.

Native arithmetic helpers remain unchanged, including private `absolute_fee`
and multiplication saturation **before** division by 1,000,000. The original
`from_msats`, `from_sats`, SEND/RECEIVE limit initializers and all nine selected
native method items are source-hashed. Native limits are 100 sats/15,000 ppm
and 50 sats/5,000 ppm, not an invented policy. Verification-only constant intent
contracts are checked against the original initializers and inferred native
`from_sats`; separate mathematical limit values are labeled abstractions.
`from_sats` keeps its original multiplication safety domain; it is not one of
the three total-domain frozen helpers.

## Reusable engine changes

- E1: the existing checked-derive macro selects a `syn::Member` for a singleton
  named primitive integer. Only comparison derives gain named-field support;
  arithmetic/conversion derive restrictions are unchanged. Existing actual-body
  and publication checks still justify the generated comparison proposals.
- E2: a bounded transparent scalar-leaf record profile selects the existing
  complete-value owned interpreter for at least two shared roots, with at most
  eight parameters, depth four and sixteen record leaves. Each parameter keeps
  its own full object/local binding. No single-root projection guard is removed,
  and no disjointness condition is imposed on shared arguments.
- E3: bounded scalar-leaf record-valued Options use that complete-value engine.
  Primitive checked operations reuse the existing identity/type-gated numeric
  models and safety checks, alongside the already approved Option/residual
  correspondences. All soundness, maximality and exactness checks remain required.
- Imports retain complete pure Option-record equations without normalizing their
  tagged conditionals during reconstruction. Pure-formula validation, bounded
  substitution and the exact re-encoded coverage/guard comparison remain active.
  This fixes the otherwise rejected native nested-Option certified import; it
  does not waive the summary coverage check.

No production code recognizes Fedimint, PaymentFee, Amount or their field names.
No standard trust family, application summary axiom, native replacement body,
manual field/overflow/Option proof lemma, proof assumption or domain restriction
is added. `records/shape.rs` and shared readers/controllers/CI/guides are untouched.

## Composition and the accepted correspondence seam

The authored pure `validate_fee_pair` calls the original helpers in this order:
checked addition, overflow rejection, send-limit rejection, receive-limit rejection,
acceptance. Its mathematical contract characterizes all four outcomes over
unrestricted original inputs, including exact-M sums, component bounds and equality.
Local and genuinely imported callers use the inferred native helper interfaces;
the imported client also composes the decisions directly rather than only calling
an ordinary exported top-level theorem.

`native_absolute_bound` calls the existing checked `component_limit` semantic
monotonicity theorem and then the actual private native `absolute_fee` methods.
The retained theorem body has two additional checked **intent** clauses identifying
its result fields with the already authored fee mathematics; no algebraic proof
steps are added. Accepted sum/receive component bounds discharge its hypotheses
for every u64 amount, including saturation. The arithmetic theorem itself was
already supported in fee76 and is not claimed as new automation.

The primary native correspondence region is gateway server lib.rs:2627–2648,
SHA `0bbbf702c708e9dc5065f0d5e03723e48afd1f06d900ad192f0f85bdb5e54e71`.
The secondary region is config.rs:187–207,
SHA `155f60d6877d4f6764b22686032d7b009638ec825be548fa0085e3f6b6c1aeb2`.
The mapping abstracts formatted error payloads into outcome codes and preserves
the fee inputs, native helper calls, decision order and pre-save boundary.
It does **not** verify either whole native caller, surrounding option/configuration
selection, crypto/URL parsing, lookup, error formatting/allocation, assignment,
async save, commit, database, durability or funds safety. Assignment/save at
2650–2652 and commit at2655 remain outside the theorem. Source correspondence is
an explicit audit seam, **not a verified extraction compiler**.

The parent's `native-gateway-oracle/receipt.json` supplies supporting finite
native evidence: debug/release each cover 83,521 combinations and sampled actual
fee bounds. Its classification remains **zero formal-proof credit**; it neither
substitutes for this Verus proof nor independently reviews this implementation.

## Baseline versus final proof effort

| Work | Accepted cycle80 baseline | This candidate |
| --- | --- | --- |
| Native named Amount comparison | Tuple-only checked-derive frontend decline | Actual named generated comparison bodies checked |
| Two borrowed native fee roots | Non-native tuple control exposed record-profile decline | Independent complete roots; alias/equality accepted |
| Native Amount checked-add | Record Option eligibility decline | Exact `true`-domain inferred contract |
| Native nested PaymentFee checked-add | Canonical typed `?`/record payload decline, even with verified Amount summary | Exact `true`-domain inferred contract and certified import |
| Manual native checked-add contracts | Two authored ordinary helper contracts, 6 verified/0 errors | Both removed; native bodies unchanged |
| Full validation/fee-bound boundary | Not supported under the frozen no-substitute/no-assumed-Ord conditions | Provider 55 verified/0 errors, 31 checked exec bodies; imported client 4/0 |
| Business intent and abstraction | Authored | Still authored explicitly; not claimed automatic |

The sealed feasibility's 11 fresh probes plus the four parent comparison probes
are retained as baseline evidence. This implementation does not recast an already
supported fee arithmetic replay as the new capability.

## Validation and limitations

The dedicated runner is `source/tools/check_fedimint_validation81.py` with explicit
`--application`, `--runtime`, `--sysroot`, and fresh private `--output` paths.
Final raw results are sealed in
`/private/tmp/verus-fedimint-cycle81.lyf209w3/delivery/replay/receipt.json`.

- Fresh matched compiler and vstd: vstd2221/0; VIR150/0.
- Fifteen focused Rust suites:162 tests passed, including the three new native
  tests, native Option, comparisons, publication, owned values, records, numeric
  models and wrapper/trait regressions.
- New integrated acceptance tests:12 pass normally and under `-O`; fee76's9 and
  prefix79's18 reader-unit tests also pass in both modes. Named guards are removed
  individually in the test fixture so each malformed outcome otherwise passes
  the integrated `validate_outcome` path, not a later incidental rejection.
- Full native runner:46 cases —5 positives,18 semantic failures,2 safety failures,
  12 frontend declines,8 certificate-import failures and1 resource-incomplete run.
  All41 negative runs remove the seeded inferred export. All positive required
  checks rerun fresh; warm caches contain three hits and seven regenerated states,
  not reused proof outcomes. Seventy-six immutable runner inputs are unchanged.
- Semantic controls include either/both overflow, exact M, either excessive
  component, equality, receive-only rejection, mixed roots, wrong nested arithmetic,
  skipped ppm overflow, lexicographic policy, decision order, omitted receive check,
  denominator/saturation changes, a reversed **candidate** against the checked
  actual named comparison body, and a genuinely imported false assertion.
- Safety, malformed residual, hidden effects/assumption, missing/generic/multifield/
  opaque/drop/foreign comparison, explicit narrowed-contract, resource and native
  library corruption controls are distinguished rather than called semantic wins.

Existing fee76 and prefix proofs replay with original generated contracts, fresh
checks and genuine imports. Their shared reader identities remain79/73 and reject
81/75 (`wrong certificate identity`, `current cache identity`, and `interface
policy/producer`). These are **not recorded as full legacy-reader passes**. Parent
must reconcile identities at82/76. No shared reader was patched or silently skipped.

Some original Rust test helpers hardcode `source/z3` and omit sysroot arguments.
The sealed private regression harness copies the test cases unchanged, resolves
manifest dependencies to this candidate and adds only explicit sysroot routing
to the common invocation helper; its solver link is private. Original shared
test helpers and source/z3 are untouched. Initial setup failures and corrected
replays are retained, not deleted. Focused changed-file formatting passes;
pre-existing formatting differences in `owned.rs` are deliberately not reformatted.

## Handoff and trust

Delivery contains an exact **dirty-cycle80-relative** apply_patch delta, complete
candidate/source/build inventories, matched runtime, native/toolchain ledgers,
commands, raw controls, prior-seal references and a plain-files reconstruction.
Apply this delta only to the already restored accepted dirty cycle80 baseline;
do not apply the accepted full restoration patch again. Parent's new generic
`infer_fee_validation_composition.rs` is not part of this delta.

Four existing files overlap possible integration work: `infer_derive_ops.rs`,
`owned.rs`, `registry.rs` and `import_export.rs`. The latter two contain only this
candidate's reserved identity changes. The `owned.rs` selector/model/decoder hunks
must be merged with any sibling changes rather than replacing the entire file.

Trust remains ordinary Verus/Rust elaboration and code generation, the checked
derive mechanism, existing numeric/Option/comparison correspondences, solver and
the explicit authored semantic/property/caller audit seam. Immutable native and
baseline runtime/source inputs, own HEAD/index/branch and the prior feasibility
seal are guarded. No native/parent edits, new checkout, subdelegation, network,
shared-cache writes, installs, commits, pushes or manual ref/PR actions occur.
Independent review is pending; no claim that the parent goal is complete follows.
