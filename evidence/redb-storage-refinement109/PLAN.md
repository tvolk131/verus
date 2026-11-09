# Approved native storage-refinement round109

## Authorization and deadline

The user's "Absolutely!" approves the immediately preceding proposal: one
60-minute, 4-GiB scratch-only redb storage-refinement implementation/review round,
parallel subagents, no compiler/runtime builds, no adoption of new trusted
assumptions and no weakened gates. It does not approve diagnostic107-D1, reset
earlier exhausted allowances or authorize any excluded task.

Round start: November 7, 2026 21:35:26 America/Chicago (November 8 03:35:26 UTC).
Hard round deadline: November 8 04:35:26 UTC. Stop new optional work by 04:23:26
UTC and reserve the final twelve minutes for retention/accounting. Every child
must obey the earlier of its own limit and the round deadline. No automatic
extensions. Preserve incomplete results honestly rather than overrunning.

## Target and scope

Working first final-cell target: two-phase acknowledged durability. This is not
established as cheapest and does not reduce the eighteen-cell objective.
This round's local milestone is fixed-geometry, single-process Immediate
commit -> pending write buffer -> backend-visible/durable images -> public
acknowledgement, with every selected Result error branch accounted for.
Growth/shrink, all operation histories, allocator/lifecycle and native resource
bindings remain final-cell obligations, not silently removed requirements.

The native sources are recovered content at recorded pin
c82ab865bb362bc4af067d332adc44978ff64813, under
/private/tmp/verus-crash-consistency94-20261105/native-leaf-lookup-007/recovered-native/redb.
Original native-path preservation FAILED; Git continuity UNAVAILABLE. Frozen
operational compiler/source correspondence remains UNESTABLISHED. Hashes do not
repair these qualifications. Parent inspected initial source anchors to scope
this plan before capturing the input manifest; no atomic first-read claim.

## Parallel implementation lanes

1. **Buffered storage:** implement a checked, source-anchored refinement candidate
   distinguishing pending application bytes, backend-visible bytes and durable
   bytes. Produce write/sync transitions from an explicit checked flush/control
   program rather than requiring the desired trace in theorem premises. Cover
   same-offset coalescing, arbitrary stripe iteration, successful-prefix write
   errors, retained failing stripe, and drain-before-sync ordering. Backend
   write/sync semantics and native buffer admission remain explicitly OPEN if
   not checked; do not adopt them or hide them as an accepted native producer.
2. **Public acknowledgement:** implement a checked, source-anchored two-phase
   commit/public-return control-flow candidate. Include both header writes,
   their actual flushes, fallible header holds, prepare and post-publication
   continuations. Establish that checked public Ok follows the necessary successful
   barriers; exhibit post-sync Err without rollback and stale-secondary early
   crash prefixes. Do not assume the old secondary is the newly prepared slot.
   Link every modeled action to exact native anchors and list missing bindings.
3. **Independent controls:** implement a separate finite outcome/source oracle
   with tests for the native call ordering, coalesced writes, write/sync failures,
   stale secondary and post-sync error. Challenge the prospective proof scopes;
   provide semantic mutants with corrected counterparts. Finite Python tests
   and source-token guards are not formal native execution correspondence.

Each proof lane gets one worker, at most sixteen existing-verifier applications,
25 minutes total, 22-minute optional-work cutoff, at most 768 MiB scratch and
at most 90 seconds per application. Charge failed setup/launch attempts honestly.
The controls lane gets 16 minutes, 64 MiB scratch and no verifier/solver launches.
Max two verifier workers globally. No Rust/Cargo/library/engine/vstd/application
builds: checking authored proof libraries with the existing verifier is allowed;
executing native applications is not. No copying toolchains or caches.

After candidate handoff, commission one fresh independent review of the actual
frozen candidates and controls, not just this plan. Reserve 15 minutes, 512 MiB,
one worker and six existing-verifier applications for that review. Small findings
are fixed by the parent with regressions; no automatic review chain. Parent and
retained evidence share at most 1 GiB; total round scratch stays below 4 GiB and
global free space strictly above 16 GiB. Resource observations must state their
sampling limits, not pretend to be hard quotas.

## Proof and trust boundaries

Use actual recovered source and existing checked vocabulary. If native std/cache/
resource bindings prevent verification of original bodies, retain the strongest
checked reference/extracted fragment with that exact limitation. Never call an
authored interpreter the original Rust implementation, claim a checked producer
from equal byte sequences, or put an unestablished native correspondence into a
requires clause and claim native closure. No new assume/admit/axiom/external_body
or trusted adapter. No primitive laws, runtime adoption, admission/export change,
source integration or final-cell promotion is authorized here.

Model both error returns and the separate unestablished panic/abort/resource
paths explicitly; do not prove them unreachable by omission. Existing abstract
storage semantics are not native backend proofs. Symbolic positive coverage,
localized backend-semantic negatives, corrected counterparts and byte-identical
restoration are required for each claimed theorem milestone. Preserve every
failed attempt, raw stdout/stderr, actual child status, argv/environment, checked
scope, source/tool hashes and artifact publication outcomes.

All parent/native/frozen files are read-only to children. Use fresh /private/tmp
scratch and apply_patch for authored edits; no Git writes, network or subdelegation.
Reauthenticate used inputs at entry/preexecution/exit, including direct tools and
any transitive tool shim actually used. Existing runtime use does not repair its
compiler/source provenance. Outputs are candidates until parent retention/review.

Never resume or reroute growth049, zero-growth050, mutation030, refused-padding95
or refused drop-pipeline work. In particular, the missing WritablePage/lifecycle
admission edge may be recorded but is not permission to implement that pipeline.
Keep S0, full-scope, Infer and import/export barriers unchanged. All eighteen
redb cells remain OPEN unless their original complete acceptance gates are met;
this bounded local milestone is not sufficient for such a promotion.
