# Repeatable native adoption: v2

Status: active. Verus baseline `bc45dab58`; pristine Aleph baseline
`f35c7bba6d628bcafdf0d65dd471cc6f868ec034`.

## Objective

Make the completed Aleph v1 verification result reproducible from clean main by
a fresh adopter with the current guide and general examples. Authors choose
views, invariants, semantic interfaces, behavior/error policies and input
coverage. Inference and checked goal assistance supply routine implementation
contracts and mechanical proof scaffolding.

The fixed semantic scope is the completed VERUS_INFER_ADOPTION_V1.md charter:
native NodeCount operations used by quorum; required NodeMap operations/Clone;
collector initialization, advancement, ingestion and parent selection; Creator
growth, ingestion and PreUnit creation; coordinate/PreUnit construction/getters;
parent metadata construction/projection/validation; checked production FullUnit
observation connection. The fresh adopter may choose different helper names,
views and decomposition. It must establish equivalent properties and coverage.
Protocol-wide agreement/liveness, async/network behavior and cryptographic
security remain outside this finite milestone.

## Comparable boundaries and input coverage

The earlier blind trial changed two variables: fresh context and stricter
generated-method exclusions than the successful v1 pilot. This milestone makes
the policy explicit before the next trial:

- An author may explicitly exclude generated implementations unused by the
  verified interface closure. Existing external_derive grants no contract and
  makes the excluded operations unavailable to verified callers. Merely being
  generated, inconvenient or unsupported does not authorize an assumed law.
- Every operation used by a proof must have a checked implementation or an
  explicitly inventoried existing library interface. Missing evidence must fail;
  no automatic fallback to an external body or weaker precision is allowed.
- Existing opaque hash/codec/lock boundaries may be reproduced with their v1
  strength. They promise no hash determinism, encoding equation, collision
  resistance, cache consistency or synchronization law. No new useful assumed
  behavior is authorized. Marker/type declarations are not behavioral laws.
- The user-approved ConstraintError Display/Error exclusion is permitted. The
  separate Signed/UncheckedSigned serializer proposal remains unauthorized;
  this milestone does not add those exclusions or claim Signed delegation.
- Quorum retains its original mathematical policy floor(2*n/3)+1 and original
  multiplication safety domain n <= usize::MAX/2. Exact inference must recover
  the actual body domain. A separately documented checked arithmetic refactor
  may broaden it, but broadening is not required to reproduce v1 coverage.
- Raw advancement/ingestion retains its representable-round precondition below
  u16::MAX. Additive checked APIs cover terminal inputs with explicit rejection
  before mutation, as in v1. All stated valid states, in-range creators and
  payloads remain covered; zero membership and initial creation retain their
  frozen special-case coverage. No domain may be narrowed to get a proof through.
- Native executable refactors must be documented and tested. Behavioral fixes
  require separate before/after evidence and must not be attributed to the
  original implementation. Whole-project Rust compilation is not a proof of
  every Rust function or of callers satisfying verified preconditions.

These rules permit scope configuration; they do not turn ignored/excluded code
into trusted summaries. The native source and the checked call path must remain
inspectable.

## Concrete work and validation

1. Audit the fresh trial's generic Sum/Into/codec reproducers against the current
   compiler. Exercise explicit unused-method exclusions and reject verified
   callers of excluded methods. Support concrete conversions actually needed by
   adoption through their original generated/ordinary bodies and inherited
   domains; do not assume wrapper projection semantics by name.
2. Improve the native Cargo recipe and diagnostics. Matching compiler/macros/
   vstd, selected first-party roots and argument forwarding must be reproducible.
   Ordinary erased Rust builds and native tests remain separate checks.
3. Provide clear machine-readable scope reporting for local checked functions,
   ordinary contracts, inference candidates, external declarations/assumptions
   and excluded/ignored native items where available. Reports are presentation,
   not certificate acceptance inputs. A successful empty/no-inference crate
   must not be described as proof of its unannotated implementation.
4. Test generic examples and negative cases: wrong generated conversion bodies,
   missing/excluded requested implementations, wrong application requirements,
   unused unsupported derives, actual use of excluded methods, repeated/cross-
   crate providers, failed/filtered verification and withheld failed exports.
5. Update the general guide using independent examples. Explain author choices,
   existing exclusions, proof closure and dependency effects. Keep Aleph-specific
   solutions and previous experiment artifacts out of the blind trial's inputs.

## Acceptance and stopping rule

The final fresh-context adopter starts in a new app-managed Aleph worktree from
the pinned unannotated main commit, with the finalized compiler held fixed. It
may read general documentation/examples and actual clean native source, but not
previous Aleph proof implementations, patches, fixtures or evidence. It records
baseline/tool identities, complete authored-input/assumption/exclusion/function
inventory, reproduction commands, native checks and proof outcomes.

Completion requires all agreed v1 properties and input coverage, checked native
source connections and exact helper certificates, no local supporting proof
blocks, executable-loop annotations, trigger hints or mechanical getter/selected-
borrow frame contracts. Application goals and semantic definitions/contracts
remain author input. Partial native testing, frontend rejection, empty-crate
success or a smaller arithmetic proof do not complete this goal.

Generic regressions, publication safeguards, appropriate native tests and
semantic negative cases must pass. Resolve relevant blockers generally and
repeat an independent acceptance attempt if necessary, recording every trial
and scope decision. Keep bug discoveries distinct from deliberate mutations and
tool/model gaps; preserve independent original failure/reachability evidence.

Commit cohesive validated chunks without pushing. Completion means reproducible
adoption under these frozen boundaries, not general Rust/serializer support or
discovery of a serious production bug. New scope, trust or coverage changes must
be recorded explicitly; a blocker is not permission to weaken acceptance.

## Implementation checkpoints

The first implementation adds checked owned reverse scalar conversion derives
(`infer_derive(Into)`), canonical primitive identity conversion composition and
read-only compiled native-function scope reporting. The original generated
conversion bodies and inherited domains remain proof obligations. Missing,
excluded, wrong and restrictive requested implementations are rejected; forward
requests do not silently request reverse inference.

The general native Cargo example retains real Sum/codec implementations outside
the selected scope and rejects their use from verified code. It exercises fresh,
repeated, false-goal, excluded-call, excluded-request, no-verify, filtered and
restored runs plus ordinary native execution. The guide now explains the existing
scope mechanism, concrete error interfaces and the compiled-scope report. None
of this declares the final independent Aleph acceptance complete.

The integration checkpoint passes 479 inference compiler regressions, 57 standard
library tests, 82 syntax/attribute tests, 116 backend assurance tests and the
driver metadata test. Matching vstd verifies 2,209 results with no errors. All
nine Cargo cases pass their expected positive/negative outcomes; the positive
example checks six exact helpers (18 certificates) and 20 proof results. Its
native test executes the retained conversions, Sum and codec implementations.
VERUS_INFER_ADOPTION_V2_EVIDENCE.json records source/tool identities, commands,
receipt/log hashes and outcomes. These are integration checks, not Aleph proof
acceptance or a formal verification of the compiler changes.
