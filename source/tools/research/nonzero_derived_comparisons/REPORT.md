# Checked genuine NonZero derived equality

Implemented a bounded checked PartialEq extension. Actual built-in derived
`BorTag::eq` agrees with the existing standard NonZero view over **every genuine
NonZero<u64>**, without a tag budget or extra precondition. This is a standalone
source component, not a native/kernel correspondence or application acceptance.
PartialOrd and Ord remain **UNPROVED**.

Early typed milestone: `b074d91ddde3f5af1abc3c0cc4138f9f6b28baca`, based on
`bc894bebb619260b6723105875294837c45e845f`. INTERFACE.md preserves that earlier
inference-profile barrier; it is not the final implementation status.

## Source, interface, and implementation

The pinned Rust source is commit 58cf8f9c35b355199727627d07ae34a4e2846d67.
The extracted genuine declaration, including all seven derives and its PRIVATE
NonZero<u64> field, has SHA-256
`0107890f5ffee6b94a4179c0351a4ff121e0e61b9886067e94370a65391e9abd`.
The macro retains this item, adding only erased specification definitions and
opt-in annotations. No field visibility widening, runtime replacement impl,
constructor spelling change, new runtime behavior, or restricted input tags.
Trials borrow two already-created tags; no constructor lane is needed.

`infer_derive(PartialEq)` now accepts a nongeneric single NonZero<u64> field.
It emits the existing derived-eq request, a closed standard-view observation,
and a closed PartialEqSpecImpl candidate. The VIR recognizer requires the real
core::num::nonzero::NonZero<u64>, not just a source spelling. Generic wrappers,
other payloads, and NonZero ordering requests are outside this macro profile.

The candidate-only newtype projection uses u64 observations; lifting reinstalls
calls to the typed observation on the original parameters. Genuine dynamic
NonZero equality calls can propose equality of those observations. Neither the
projection nor the marker establishes a theorem. The unchanged original native
body must establish soundness, safe-domain maximality against the inherited
trait domain, and output exactness. Ordinary WF, modes, traits, lifetimes,
termination, body checks and full-crate checking remain enabled.

Closed observations omit their bodies on export. The import path recognizes
their symbols/signatures and projects their calls when validating the already
certified summary. Existing proof trailers, dependency/model fingerprints,
producer checks and ordinary consumer checking remain in force. No import
certificate, schema, policy or cache version is changed.

Production changes: infer_derive_ops.rs; comparison-specific logic in
wrapper_compare.rs and wrapper_binary.rs; narrow projection hooks in newtypes.rs,
registry.rs and infer.rs. The latter two changes only supply the comparison
observation context. infer_compare.rs, rust_verify/external.rs,
automatic_derive.rs, rust_to_vir_expr.rs, function-value/constructor translation,
all vstd files, and global version constants are unchanged. No constructor-owned
file was edited or constructor commit merged.

## Checked body and obligation inventory

Only **PartialEq::eq** is selected among the comparison derives. Its genuine
Rust 1.99.1 compiler-expanded body is `{ self.0 == other.0 }`, SHA-256
`1774afdac1844226361985602334fc3b076cb8194e4f4efcc8c0b18fd2eb229b`.
The original pre-inference VIR body, including its exact epoch spans, has SHA-256
`f95e83397ef7e15e2e6aa4d474e83ddb333c65354760640ca767490b56cb6827`.
Both full expansion and VIR excerpt are private in
`/tmp/nonzero-derive-body-ny_0y7oi`; the receipt and all products are hash-bound.
The expansion also confirms actual PartialOrd delegates to wrapper Ord, whose
body delegates to NonZero Ord. These ordering bodies are not selected here.

Final macro epoch `/tmp/nonzero-owned-macro-4okwhwsz` checks **5/0**, cold and
warm. Exactly one inferred helper: `component::BorTag::eq`. Its domain is `true`;
its result is equality of the two standard-view observations. Soundness,
maximality and exactness are all required and proved, including on cache hits.
The complete checked native body set is exactly:

- original `<BorTag as std::clone::Clone>::clone`;
- original `<BorTag as std::cmp::PartialEq>::eq`;
- authored ordinary client `compare_equal`, body `{ a == b }`.

There are zero local proof bodies, proof blocks, mechanical lemma calls,
snapshots or new axiom calls in this final macro component. Spec vocabulary is
`tag_value`, the generated view observation, the request marker, and the two
PartialEqSpecImpl definitions. Five is the verifier query total, not five native
methods. Clone uses existing derived-Clone checking. Copy has no translated
executable method. Hash, Eq::assert_fields_are_eq, PartialOrd::partial_cmp and
Ord::cmp remain present but outside selected verification scope. Inherited `ne`
is supported by the existing trait contract, not claimed as a separately checked
local derived body. RESULTS.json retains the complete native inventory.

The imported ordinary client checks **4/0**, with one exact local helper
`client::equal`, and two checked local exec bodies (`equal`, `goal`). Its runtime
body calls the actual imported `<BorTag as PartialEq>::eq`. The original provider
has checked native bodies and certified export metadata; no ignored derived
body is counted as checked.

## Tests and controls

Final matching compiler/macros were built offline, two jobs, private Cargo home,
Git dependency objects, target/temp/cache directories. Matching normal vstd
checks **2226/0**; --no-cheating was NOT forwarded into vstd. Every application
trial has --no-cheating and ordinary complete checking. Immutable retained final
eight-file tools: `/tmp/nonzero-derive-frozen-tools-8hnei2sw`.

- Cold/warm checked export: 5/0 twice, candidate miss then hit, all three
  obligations freshly proved. Assistance-off: 5/0. Inference on explicit derives
  is still active when optional --infer-goals is off; this is not an inference
  ablation. Ordinary certified import: 4/0.
- Arbitrary borrowed tags and explicit MAX/high-bit cases: 5/0. The high bit is
  written as exact decimal 9223372036854775808, with no domain restriction.
- False returned equality relation: 4/1, exactly compare_equal fails, with the
  identical healthy inferred eq obligations and positive companion. Failed-crate
  native statuses correctly say not certified by a successful full run; they are
  not promoted to checked-body certificates.
- Constant-zero observation and reversed proposed eq model: each 1/2, exactly
  the original derived eq fails soundness and exactness while maximality proves.
  These are invalid-candidate/body-contract failures, **not** healthy-helper
  semantic negatives. No executable replacement body was authored or mutated.
- NonZero<u32> and NonZero ordering requests fail macro/profile checking, with
  zero inferred helpers. They are unsupported-shape failures, not semantic
  negatives and not certified methods.
- Direct standard NonZero::cmp's true view-order postcondition fails 0/1. The
  unchanged standard NonZero module has PartialEqSpecImpl/PartialOrdSpecImpl but
  no OrdSpecImpl (its source cites Destruct). No stronger standard law is assumed.
- Repository tests: new infer_nonzero_derive suite **3/3**, existing
  infer_compare_derive suite **7/7**. Final receipt:
  `/tmp/nonzero-derive-tests-ycbn7tvv/receipt.json`.

All 51 completed named epochs, including mistakes and failed controllers, are
retained in epochs.json and RESULTS.json. Earlier annotation WF errors, the
scalar-policy gate, dynamic-call eligibility failure, safely rejected closed-spec
import, a bit-shift-form high-bit proof gap, and a test-only spec-in-exec mode
error are separate from accepted proofs and semantic negatives. An identity
controller initially expected friendly rather than raw impl names; its failed
receipt is retained. Four initial launcher runs lacked command-local rustup PATH;
those are setup failures. One offline build lacked copied Git dependency objects;
one successful compiler invocation then had a product-inventory path error.
Both are retained, not relabeled successful proof epochs. Normal bwrap startup
failed RTM_NEWADDR; tools used the explicit approved require_escalated mechanism.
No resource/compiler-crash result is counted as semantic evidence.

A setup diagnostic used `ps -eo ...` filtered to the worker's exact setup command;
it emitted only this worker's two records, but enumerated the host process table.
No sibling checkout, build contents or source was inspected. This is disclosed
because the task requested isolation; no stronger claim of zero process-table
access is made. All later progress checks used only owned output paths.

## Trust, limitations, and versions

TCB: pinned Rust/compiler lifetime checks, Verus frontend/macros, logical
encodings, solver, and existing matching vstd specifications. In particular,
standard NonZero's opaque view/get and comparison models and core trait contracts
are inherited trust, not body-checked standard-library proofs in this lane.
The accepted equality component uses no new assumption or client equality law.

The early ghost identity trial checks 2/0, using the existing inverse-view axiom
twice. The typed diagnostic equality trial includes that unused checked ghost
lemma and checks 6/0. It is not a premise or implicit global theorem for the final
5/0 macro proof. No Rust Eq-to-identity law is assumed. No vstd specification,
axiom, model, assume_specification, external-body stub or axiom was added.

The cache fingerprint now includes comparison observations as candidate inputs.
Global schema38 / policy68 / cache71 constants remain unchanged, as instructed;
all caches here are private. Parent owns combined eligibility-policy/version
reconciliation and rebuilding interfaces/dependents. A fresh build at the final
commit has a different producer Git label from this dirty milestone build and
must regenerate certified exports. Stored tool/source hashes, not build labels,
bind these epochs. Intermediate mutable build targets were subsequently reused;
only retained frozen bundles and final source identities are acceptance tools.

The constructor lane was not composed. This standalone baseline has 2226 vstd
queries and the exact three-body macro inventory above. The parent's combined
constructor build adds checked native constructor bodies (and reported 2250 vstd
queries). Combined acceptance must require every such body; it must not drop them
to fit this standalone inventory. Independent combined acceptance remains open.

Scope stays native=0; 44 minimal + 3 phase / 101 writer effects OPEN;
publication=false; goal_complete=false. This does not prove native permissions,
source safety, original reference C/F, full Item.new, canonical 6/0 application,
or the exact bitcoin_hashes 0.14 HMAC pad-XOR target. Full HMAC remains 1/1
UNPROVED for arbitrary initialized [u8;128], every lawful keylen including 0 and
>128, BOTH XOR prefix and unchanged suffix. No component result closes those goals.

See REPLAY.md for audit and reproduction. Raw sources, expansions, logs, solver
output, binaries and full receipts stay private in named /tmp artifacts; only
compact evidence and controllers are in this directory.
