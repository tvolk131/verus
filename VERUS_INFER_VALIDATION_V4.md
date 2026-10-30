# Native unchecked-unit validation: v4

Status: complete for the finite milestone, October 30, 2026. Verus baseline `f9c12a937`; native accepted
baseline `a2de660d5467ce3611584768285b9227a1e2832b`. V1–v3 remain complete.
Development fixture: `/private/tmp/verus-validation-v4/aleph`. The accepted v3
fixture and fixed engine remain untouched.

## Objective

Verify the original native path from `UncheckedSignedUnit` through
`UncheckedSigned::check` and `Validator::validate_unit` to its returned signed
unit or precise error. Authors supply semantic observations, required behavior
and honest dependency boundaries. General checked inference and goal assistance
should supply routine implementation contracts and proof scaffolding.

Required properties:

1. Successful signature checking wraps the same signable object and signature;
   failure returns the original unchecked object. The abstract verification
   result concerns the actual bytes/signature/index passed to the dependency.
   No deterministic hash, AsRef, keychain or cache law is silently assumed.
2. Successful native validation establishes the actual session and round
   checks, membership equality and checked parent policy, and returns the same
   underlying unit/signature. Existing initial/positive domains and opaque hash
   comparison meaning remain explicit. No required precondition is hidden by
   narrowing to a convenient concrete Hasher, payload, creator or round.
3. Errors follow native priority: signature, session, maximum round, membership,
   then the existing ordered control-hash errors. Error payloads retain the
   original or the actual checked Clone relation as appropriate; generic Clone
   is not assumed to preserve logical identity.
4. Checked wrapper accessors/conversions and concrete FullUnit observations
   connect the complete native call chain. Merely compiling unverified methods
   or assuming their first-party behavior does not satisfy coverage.

The theorem concerns results produced by the actual validation call. `Signed`
has other construction paths, including Decode: its type name alone is not an
authenticity invariant. No claim that all constructible Signed values have
passed verification is permitted without checking every construction path.

## Implementation and trust boundaries

Preserve the native executable algorithms. Any necessary observation binding or
refactor must be documented, behavior-preserving and tested. Do not replace the
production path with an idealized adapter. Keep the accepted v3 semantic goals,
domains, checked implementations and certificate/publication rules passing.

Cryptographic algorithms, SCALE codec implementations, parking_lot synchronization
and hash-cache consistency are outside this finite milestone. New semantic
interfaces must state precisely which results they constrain; they must not
promise determinism, collision resistance, identity-preserving generic cloning,
encoding equations or stored-lock-value relations without justification.
Opaque typed signatures may be supplied where needed, with no useful result law.
First-party wrapper/validator implementations must be checked rather than
assumed. Any additional cryptographic interface law must be explicit in the
theorem and evidenced by checked implementations or an identified external
primitive boundary.

The proposed native interface uses explicit `lawful_index` and
`lawful_verification` predicates, both false by default. A lawful index getter
returns its supplied observation. A lawful keychain reports its supplied member
count and returns the modeled verification decision for the actual byte slice,
signature and signer index, rejecting out-of-range indices. This is a conditional
interface obligation, not a certification of existing unverified keychains.
Native FullUnit index delegation must discharge its own law, and a checked
unrelated concrete provider must demonstrate nonvacuous accepted/rejected paths.
Hashing and AsRef still have no value equation: the proof retains the actual
slice witness rather than recomputing it or presuming repeatability.

Signed/UncheckedSigned serializer exclusions remain unauthorized. Discovering
that an unused derive is unsupported is a recorded blocker, not permission to
exclude it or add an assumed contract. Do not broaden existing v3 exclusions;
previously unverified operations may instead become checked parts of the slice.
Protocol agreement/liveness, network/async execution and arbitrary Rust inference
remain outside scope.

## Acceptance evidence

- A recorded original-body/source/assumption inventory and actual compiled
  closure for both signature checking and outer native validation.
- Positive goals for every native branch and its stated domain, including
  multiple simultaneous failures to exercise priority and successful output
  identity. Explicit dependency nondeterminism must remain represented.
- Deliberate native mutations bypassing or changing each required check,
  changing returned payload, or changing error priority must fail ordinary
  application verification. Restore and replay the full positive source.
- Unrelated generic regressions for engine extensions, cross-crate composition
  and failed/no-verify/filtered publication controls.
- Fresh proof/native outputs, matching toolchains, exact required certificates,
  retained v3 coverage and unchanged accepted v3 inputs/artifacts.

Commit cohesive validated work locally; do not push. Unsupported requirements
remain unfinished until resolved. Keep compiler/model defects, deliberate
mutations and any independently reproduced original application defects distinct.

## Accepted outcome

Verus implementation `5929d1db8`; native implementation
`fa88a4f0956e7a888bbe3d47bc8e14286f2af953`. The original check/validator bodies
verify without executable edits. Direct shared projections can cross opaque
cache fields, and ordinary constructor-shaped error callbacks receive bounded,
normally checked contract assistance. Policy 50 / cache version 56 requires
rebuilding providers and dependents together.

Fresh restored proof: 204 verified results, zero errors, 39 exact helpers and
117 mandatory certificates proved. All 35 v3 helpers and 84 previously checked
items remain covered. Native tests: 192 unit tests and one doctest; inference
regressions: 285; vstd: 2,215 verified. Seven semantic mutations and two
publication controls reject; restored source passes a fresh full replay.
The source inventory compares 179 unchanged original bodies. No exclusion was
added; two Clone exclusions were removed. The old v3 fixture and fixed engine
remain unchanged.

See [the acceptance report](evidence/verus-infer-validation-v4/README.md),
[explicit assumptions](evidence/verus-infer-validation-v4/assumptions.md) and
[machine-readable closure](evidence/verus-infer-validation-v4/closure-summary.json).
This is local acceptance by the implementing agent, with no additional
independent-agent audit claimed. Commits remain local; nothing was pushed.
