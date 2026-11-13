# Experimental contract inference

Infer derives contracts for supported Rust implementations and checks bounded
proof annotations for ordinary Verus goals. The caller supplies the intended
property, semantic definitions, abstraction and required input domain. A checked
helper contract alone is not proof of an application property.

See [capabilities](infer_capabilities.md), [adoption](infer_adoption.md), and the
[implementation and testing reference](infer_design.md).

## Start with an example

Build Verus and `vstd` using the repository's [build instructions](https://github.com/verus-lang/verus/blob/main/BUILD.md).
From the repository root:

```sh
export VERUS_Z3_PATH="$PWD/source/z3"
export RUST_MIN_STACK=20971520
source/target-verus/release/verus examples/verus_infer.rs
source/target-verus/release/verus examples/verus_infer.rs --compile -o /tmp/infer-example
/tmp/infer-example
```

`#[verus_infer]` selects eligible functions. Their original bodies remain Rust;
ordinary callers consume the checked contracts. The examples cover arithmetic,
branches, composition, arrays, slices, tagged returns and caller properties.

```rust,ignore
use vstd::prelude::*;

#[verus_infer]
pub fn advance(x: u64) -> u64 {
    if x == u64::MAX { x } else { x + 1 }
}

verus! {
fn caller(x: u64) {
    let result = advance(x);
    assert(result >= x);
}
}
```

Inference is bounded. Unsupported implementation shapes, missing models,
generation limits, required proof failures and incomplete dependencies are
reported rather than replaced with assumed contracts.

## Contract checks

Exact inference requires three separate checks:

- **Soundness:** every admitted input executes safely and satisfies the relation.
- **Safe-domain maximality:** every safe execution is admitted.
- **Output exactness:** the relation admits exactly the implementation's result.

Sound-only inference checks soundness and reports the independent precision
probes. It does not claim maximality or exact output. Caller proofs must use the
actual mode and admitted domain. In particular, check that the application's
required domain implies the generated precondition.

Callees are checked before dependent inferred callers. Failed or incomplete
required checks block dependent use and inferred-interface publication. Candidate
caching saves generation work; it does not replace fresh required proofs.

## Ordinary proof assistance

`--infer-goals` proposes bounded annotations inside eligible ordinary functions.
These are checked proof steps, not new axioms or application specifications.
Supported strategies include scalar arithmetic/packing, finite-set relations,
word/sequence updates, selected loop annotations and mutable-loop observations.

```sh
source/target-verus/release/verus input.rs --infer-goals --no-cheating --output-json
```

Author semantic predicates, views, invariants, transition goals and final exit
clauses as needed. Assistance may remove mechanical steps while leaving that
semantic work intact. A strategy can be skipped conservatively; enabling
assistance must retain ordinary safety, lifetime, trait, well-formedness and mode
checking. Generated executable computations are not permitted as proof glue.

The canonical-u64 fixture keeps its canonicality predicate and semantic loop
invariants. The bitmap fixtures keep their representation/validity definitions.
Neither example establishes arbitrary iterator effects or discovers a project's
high-level meaning automatically.

## Read the result

`--output-json` reports a `verus-infer` object alongside ordinary verification
results. Relevant fields include:

- `status`, `crate_success`, and `interface_written`;
- each inferred function's mode and required `proofs`;
- `verification_scope`, with selected and actually checked bodies;
- `failed_functions`, assistance observations and generation/solver metrics;
- trusted models and imported inference dependencies.

Use the report's own schema version. The report, exported interface and cache
have distinct compatibility roles; a JSON success flag alone is not a proof.
A failed crate does not certify its selected bodies, even if some individual
queries succeeded. Check named helper identities and all required statuses,
including the expected empty helper set for ordinary-only proofs.

Keep these outcomes distinct:

| Outcome | Meaning |
| --- | --- |
| Verified | All required checks in the reported selected scope passed |
| Caller proof failed | A usable helper relation did not establish the caller's goal |
| Unsupported/frontend failure | The intended backend proof was not reached |
| Generation budget exhausted | Candidate generation was incomplete |
| Required proof incomplete | A required solver check did not complete |
| Optional precision incomplete | Only the explicitly optional precision result is missing |

Timeouts, resource exhaustion and parser failures do not count as semantic
negative controls. Assistance explanation coverage is separate from the set of
implemented strategies; absence of an explanation does not establish an attempt.

Full reports belong in temporary output or CI artifacts:

```sh
source/target-verus/release/verus input.rs --output-json --time > /tmp/infer-report.json
```

## Rejections, callees and partial re-checks

One run reports every rejection in the crate, each with its location and reason.
A function that calls a rejected function is reported as blocked at that call;
further rejections inside the same body are found by retrying with the rejected
subexpression replaced by an opaque value (up to 16 per function). Reported
follow-ups that only arise from that replacement are suppressed.

An inferred function may call a function whose specification is *functional*:
its preconditions plus either `returns E` or exactly one `ensures ret == E`.
This covers `vstd` `assume_specification`s for std functions (for example
`u64::is_multiple_of` and `Option::unwrap`) and hand-written contracts of that
shape. A callee that requires `false` and returns `!` (such as the panic behind
`assert_eq!`) is a diverging call. Existing approved models keep precedence.
Callers of a functional callee that returns a structured value use the owned
profile. Relational (non-functional) callee contracts, such as
`Option::unwrap_or_default`'s, are still rejected; there is no automatic
sound-only fallback yet.

`--verify-root`/`--verify-only-module` with `--verify-function` performs a partial
re-check: the selected functions and the inferred functions they rely on
(transitively, through inferred bodies) are checked. A partial run never
certifies or exports inference metadata, so it is refused when exporting.

## Cross-crate inference and Cargo

Public inferred interfaces use Verus's normal `.vir` mechanism. Direct invocations
pair `--export`/`--import name=file.vir` with matching `--extern` libraries and
transitive dependencies. An importer validates candidate-bound proof evidence,
producer/dependency/model compatibility and required modes before consuming it.
Pruned contracts, assumptions and cached candidate templates are not interchangeable.

The maintained three-crate example exercises this composition:

```sh
export PATH="$PWD/source/target-verus/release:$PATH"
cargo verus build --manifest-path examples/verus_infer_workspace/Cargo.toml
cargo verus verify --manifest-path examples/verus_infer_workspace/Cargo.toml
```

For inferred crates, publication waits for full successful checking. Failure
invalidates selected stale outputs, including explicit and Cargo output paths.
Some local experimental capabilities intentionally refuse interface export;
respect that restriction instead of treating a file's existence as certification.

## Safety and trust

The engine preserves Rust's distinction between checked, wrapping, saturating
and signed operations. Index bounds, overflow, shift ranges, division conditions
and supported architecture bounds are part of the checked contract. Narrowing an
input domain changes the task and must be disclosed.

The inherited Verus compiler, Rust frontend, SMT solver and declared `vstd`
models remain trust inputs. For a callee with a functional specification, its
preconditions are taken as its safe domain: safe-domain maximality is relative to
the callee's own contract (the modular reading), which for `vstd` std specs relies
on those specs stating the exact panic condition. Checked foreign-body support is limited to explicitly
recognized bodies and their complete obligations; it is not a generic permission
to trust foreign code. An external type declaration exposes no representation or
cryptographic law by itself.

General unsafe/shared-state inference, arbitrary abstraction discovery and
general mutable-iterator framing are outside the supported scope. The unfinished
owned-UnsafeCell/native state experiment is preserved separately; its original
clients and state/erasure gates are unproved.

## Regression coverage and research history

Canonical test inputs are in `source/rust_verify_test/fixtures/infer/`, with
source hashes and archived origins in `provenance.toml`. Existing Rust tests own
the options, positive outcomes, semantic/safety controls and import/export checks.
The retained REDB/Fedimint cases are bounded application proofs, not whole-project
or crash/durability/cryptographic guarantees.

Historical experiments, full receipts, native/Coq model studies and review
transcripts are preserved at commit
`7dcabedaa93299a3607746b6ec25f5e4778288a4`, tag
`archive/infer-research-2026-11-12`. Their scripts belong to that revision and may
require recorded private inputs. The archive is not a claim that every historical
replay is portable or that every attempted goal succeeded.

REDB crash publication/recovery and the earlier HMAC/native objective remain
unachieved. See the [capability/status table](infer_capabilities.md) for the
maintained scope and [testing reference](infer_design.md) for fresh validation.
