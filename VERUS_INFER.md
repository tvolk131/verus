# Verus Infer

Verus Infer derives checked implementation contracts and proposes proof steps
for supported Rust code. Authors supply intended behavior, required input
domains, abstractions, invariants and dependency assumptions. Ordinary Verus
verification checks the original executable and the generated proposals.

This branch is an experimental Verus fork. Supported profiles are bounded;
general Rust coverage and arbitrary invariant discovery are not implemented.

## Start here

- [Current direction and next milestones](VERUS_INFER_DIRECTION.md).
- [Adoption workflow](source/docs/guide/src/infer_adoption.md): choose a property,
  infer supporting behavior, supply abstraction boundaries, and check the goal.
- [Feature reference](source/docs/guide/src/infer.md): supported syntax, profiles,
  restrictions, reports, cache behavior and cross-crate interfaces.
- [Capability map](source/docs/guide/src/infer_capabilities.md): locate a profile
  and identify its remaining authored semantic inputs.
- [Assurance and trust](VERUS_INFER_ASSURANCE.md): acceptance rules, trusted
  boundaries, regression controls and separately scoped formalization work.
- [Pinned external transfers](source/tools/INFER_TRANSFERS.md): prepare and replay
  the Fedimint and redb application slices against a matching engine bundle.

Use matching compiler, macros, vstd and dependency interfaces. Current version
identities are recorded at the top of the feature reference; historical
checkpoint versions are not compatible alternatives.

## What the automation establishes

Exact inference requires three independently checked obligations: soundness,
safe-domain maximality and output exactness within the supported observations.
Sound mode requires the sound guarantees it reports without claiming both
precision properties. Cached candidates always receive fresh required proofs.

Proof assistance proposes ordinary contracts, invariants, lemma applications,
triggers or solver queries for authored goals. A proposed annotation is not a
proof result. Inference-aware and `--infer-goals` interface publication waits
for complete required verification. Legacy ordinary Verus exports are outside
that gate and can be written before verification; a physical file alone is not
a successful proof receipt.

An inferred description of an implementation does not establish its intended
policy. Check that the required inputs satisfy its preconditions and that its
results satisfy an independently authored application property. Not every
function in that proof closure needs an exact inferred contract.

## Application evidence

Selected native Aleph and hex proof closures are documented in
[mechanical adoption v3](VERUS_INFER_MECHANICAL_V3_ACCEPTANCE.md),
[native validation v4](evidence/verus-infer-validation-v4/README.md) and
[hex vector decoding](evidence/verus-infer-hex-v3/README.md).
The later [integrated transfer checkpoint](evidence/infer-proof-composition/README.md)
records bounded Fedimint deadline and redb codec, page-size and address-range
properties, including reduction of caller proof glue.

Fresh research records the [unsigned-varint canonical blocker and partial length proof](evidence/verus-infer-varint-3b05c191/README.md)
and an [authored stateful redb bitmap invariant](source/tools/research/redb_single_word_set/REPORT.md).
The latter uses explicit reusable proof-library assistance; neither experiment
is a claim of automatic end-to-end inference for its full application.

The checked [word-view extension](source/tools/research/word_goal_automation/REPORT.md)
removes local bitmap proof glue when equivalent generic representation symbols
are supplied. The [full-request Fedimint transfer](evidence/transfer-fedimint-full-request/README.md)
retains all seven fields, including the real SHA-256 type through an explicit
opaque declaration. Both passed combined parent replays and
[independent review](evidence/combined-db26-review/README.md); the review's sole
diagnostic assertion finding is corrected. Exact scalar-pair inference then
removed both original bitmap helper contracts and passed a
[separate review](evidence/pair-0c00-review/README.md), with its harness findings
corrected. The [five-milestone acceptance record](evidence/infer-cycle-20261109/README.md)
collects final parent validation, remaining authored work and blockers.

Each record retains its own source pin, domain, authored inputs, assumptions,
validation and exclusions. These are not whole-protocol, whole-gateway or
whole-database proofs. Use fresh replay receipts to establish current results;
historical counts do not establish a fresh build's verification status.

## RFC and implementation history

The original RFC and accumulated implementation history are preserved in
[the immutable pre-consolidation version](https://github.com/tvolk131/verus/blob/3b05c1912b9d7314ad00c832c4065f4362562e02/VERUS_INFER.md).
It contains hypothetical APIs and earlier restrictions; use the current feature
reference for supported behavior. For an offline copy:

```sh
git show 3b05c1912b9d7314ad00c832c4065f4362562e02:VERUS_INFER.md
```

Frozen milestone documents and evidence directories retain their original
claims and pins. This entry point replaces the historical feature checklist
so ordinary adoption does not require reading the entire development journal.
