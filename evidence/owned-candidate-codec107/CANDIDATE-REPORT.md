# Restricted owned-candidate codec107 — sealed implementation candidate

Date: November 7, 2026, America/Chicago (November 8 UTC).
Parent: `/Users/tommyvolk/.t3/worktrees/verus/t3-60978068`, read only.
Scratch: `/private/tmp/verus-owned-codec107-20261107-kcJbs7`.
Disposition: **candidate only; independent review required before any integration**.

## Delivered implementation

`candidate.patch` is baseline-relative and changes exactly two VIR source paths:

1. `source/vir/src/infer/owned.rs`: adds a private `#[cfg(test)] mod candidate_data`.
2. `source/vir/src/infer/owned/candidate_data.rs`: actual restricted typed codec
   and nine unit tests, 1,334 formatted lines including tests.

The full candidate source is retained in `source/vir`. It is **not a shadow
AST**: the encoder consumes actual `crate::ast::Expr` and inference `Contract`;
the decoder constructs actual `SpannedTyped`, `ExprX`, `Typ`, `FieldOpr`, typed
parameters/results, and `Contract` using current source definitions. It reuses
the existing scalar `summary::unary` and `summary::binary` typed builders where
appropriate. Whole-value equality uses exact current type equality and retains
all nested owned fields. Current signature eligibility uses the existing owned
`validate_signature`; nothing installs the result.

**Compilation boundary:** this typed conversion compiled and ran inside the
actual VIR library's `cfg(test)` harness. It is absent from non-test builds.
There is no production adoption, feature enablement, inference persistence,
load/store call site, export, or certificate change. The existing owned bypass
in `registry.rs` is byte-for-byte unchanged.

## Wire and rejection boundary

The schema is `owned-proposal-loop-free-v1`. The wire contains only schema,
current-context key, exact sorted dependency keys, a flat typed data-node table,
and pre/post root indices. It has no serialized compiler types, identifiers,
spans, executable bodies, admitted contracts, proof outcomes, or authority.

Supported data: current parameter/result slots, bool/integer constants,
restricted typed scalar unary/binary/logical operations, conditionals,
complete tuple/Option/Result/eligible record/value-enum construction, variant
tests, field projections, and explicitly supplied monomorphic pure leaf
observation calls. Constructor/projection names and types are resolved against
the current signature and current datatype definitions, never supplied VIR.
Observation names and signatures are selected by slots into current definitions;
their definitions are included in identity. Calls must be static, fully final,
without injected implementations/type arguments, attached bodies, post-args,
or assume-external permission. Leaf observation bodies may not call anything.

The flat table requires strictly backward edges, forbidding cycles/forward
references. Decoder bounds are 262,144 input bytes, 2,048 table nodes, depth 32,
4,096 expanded nodes per expression, at most 8 child edges per node, 128 current
types, and structural type depth below 12. Shared-node amplification counts
expanded size rather than just table size. Context also bounds observations,
dependencies, datatype count, and identity component lengths. Integers have
canonical decimal spelling, at most 256 magnitude bits, and exact known-range
checks. Signed/unsigned machine-word literals are deliberately rejected rather
than guessing architecture bounds.

Decoding rejects malformed/truncated/oversized/noncanonical input, unknown or
ignored fields/tags, stale schema/key/dependencies, duplicate dependency keys,
out-of-range slots/roots, orphan nodes, wrong arity/variant/field/datatype/type,
result references anywhere under the precondition, binders/triggers, unapproved
calls/operators, structural type recursion, source loops/direct self-calls,
and ineligible current source. Canonical JSON only is intentional: insignificant
whitespace or key reordering is rejected, not treated as a separate valid wire.
Parsing retains serde_json's default nesting limit; no unbounded-depth option
is enabled.

The key hashes the schema, entire current typed Function Debug representation
(body/signature/attributes/spans included), actual datatype definitions, actual
approved observation definitions, sorted dependency keys, and explicit policy
components for compiler identity, complete configuration, target/architecture,
trust, and model/dependency policy. No output path is stripped. Tests demonstrate
export-path/configuration invalidation. This is intentionally conservative,
not a claim of cross-run canonical semantic identity. **A future trusted caller
must supply complete authentic compiler/configuration/target/trust/model policy
values and transitive dependency identities**; this prototype cannot verify
caller completeness. It is not an operational compiler/source receipt.

## Exact build and results

Exactly one offline Rust library/test-harness build was launched, with Cargo
jobs 2, incremental compilation disabled, and no debug information. The full
executed command and sanitized child environment are in `run-focused.sh`;
substantive preparation/validation commands are in `COMMANDS.md`.

The scratch workspace was reduced to the original `air`, `vir`, and
`vir_macros` crates. Their source trees were copied, not dependency caches,
toolchains, runtime trees, or historical targets. Only the candidate module and
its test-only declaration change their source. The scratch workspace manifest
change is a harness accommodation, **not part of candidate.patch**. Offline
Cargo pruned/updated the scratch lockfile; both original and actual resolved
lockfile hashes are retained. Dependency selection used preexisting local
Cargo cache with networking explicitly disabled. There was no download.

Raw stdout records:

```
running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 192 filtered out; finished in 0.02s
```

Raw stderr records completed compilation in 30.54 seconds and execution of
`target/debug/deps/vir-b95bba01b14f0825`. The outer `/usr/bin/time -l` prints
31.08 real / 40.18 user / 5.74 sys, then
`time: sysctl kern.clockrate: Operation not permitted` and returns **1**.
`logs/build.exit-code` preserves that failure. The Cargo child's exit code was
not separately captured, so **do not report the whole command as exit 0**.
All nine test outcomes and successful compilation are independently visible in
the raw logs. No retry build or second harness invocation was performed to
replace this result. Maximum RSS is unavailable; no successful memory-profile
measurement is claimed.

The patch dry-run in a second fresh scratch baseline returned 0. All 194 input
files were reauthenticated after the build, including current parent VIR/AIR/
macro source, workspace manifest/lockfile, toolchain specification, and required
evidence reports. `INPUT_MANIFEST.json` contains actual per-file sizes/SHA256,
not hashes copied from historical prose. Parent identity is the tested current
snapshot, **not** a claim that current AST/def definitions match a frozen
operational91 engine. `TESTED_MANIFEST.json` binds actual compiled source,
scratch manifests, build script, installed rustc/cargo executables, and test
binary. No compiler or verifier executable was built.

## Coverage, limitations, and review requests

Passing positive tests cover real typed complete record / nested Result / full
two-field error tuple round-trip, exact reconstructed typed-expression Debug
equality on that fixture, deterministic re-encoding, current static leaf
observation calls, and observation-definition invalidation. Passing negative
tests cover truncation at every byte position of the fixture, malformed and
oversized data, unknown tags/authority fields, metadata/dependency mismatches,
types/slots/arity/variant/field faults, result-in-precondition, recursion/forward
edges/orphans/depth/amplification/node budgets, unsupported operators/calls/
triggers, noncanonical/out-of-range integers, recursive/deep/unsupported types,
ineligible source, and key changes for policy/source/datatype/dependency data.

This is not exhaustive language qualification or fuzzing. In particular:

- No actual retained hex130 candidate was captured, decoded, or compared; no
  source-to-runtime or frontend-lowering correspondence is established.
- The successful nested error fixture is inside a two-field return record so
  it satisfies the existing owned signature gate. It is not the actual hex
  helper. Its whole typed error AST is preserved, not just the success byte.
- No successful projection/variant-test/Some/None/conditional/arithmetic/bitwise
  round-trip is independently exercised. The observation test's name mentions
  projections but its positive fixture exercises observation calls; negative
  projection/variant tests do run. These branches compile and reuse current
  typed rules but need additional positive tests under a separately authorized
  build budget.
- Generic function signatures, dynamically resolved/generic observations,
  ReadPlace, references/heap/sequences, binders, quantified formulas, library
  model calls beyond explicit leaf observations, and architecture-bound nodes
  are deliberately unsupported. This does not yet claim the full observed hex
  formula fragment is representable.
- Source recursion detection is direct self-call refusal plus call-free
  observation bodies and acyclic wire/type structures, not a general imported
  executable call-graph termination analysis.
- The restricted types and AST shape are validated, but a well-typed wrong
  proposal can still decode. That is intentional untrusted candidate behavior,
  not evidence that any contract is true.
- Structural equality tests on typed fixtures are not a proof of semantic
  correspondence for every accepted expression. Positive operator coverage,
  property testing, complete key-policy binding, and independent review remain.
- Dependency source caches were not copied or exhaustively content-hashed;
  actual resolved Cargo.lock and installed toolchain executables are bound.

## Sealed disposition

No parent edits, Git writes, subdelegation, network, dependency/toolchain/cache
copies, rust_verify/vstd/runtime/compiler-binary builds, verifier/solver/native
target-app executions, persistence/load/store integration, certificate/admission/
export change, primitive axiom, producer, or trust/S0 adoption occurred. The only
executed Rust target is the explicitly authorized focused library test harness.
No historical files were deleted. Excluded growth049, zero-growth050,
mutation030, refused-padding95, and drop-pipeline work was not pursued.

Any future actual candidate load must still receive **three fresh certificates:
soundness, safe-domain maximality, and output exactness**, against unchanged
current source and through unchanged installation/body-preservation gates.

**Strict persisted warm-cache gate: UNMET. No cache acceptance. No new native
proof credit. All 18 redb cells: OPEN. Independent review before integration.**
