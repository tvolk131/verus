# Restricted owned-candidate codec107 — reviewed; scratch repair validated

The scratch-only [candidate patch](SOURCE.patch) implements a restricted
loop-free proposal-data codec using actual VIR expression/types and inference
contracts, not a shadow AST. It is exposed only through a private `cfg(test)`
module; non-test builds and existing inference/persistence paths are unchanged.

The wire is untrusted candidate data only. Bounds, current-context keys,
dependency identities and typed reconstruction reject unsupported or malformed
shapes. No proof result, accepted bit or authority is serialized. Authentic
compiler/configuration/target/trust/model and transitive dependency identities
remain the responsibility of a future caller; this prototype does not establish
that caller's completeness or operational compiler provenance.

One offline actual-VIR library test build, jobs=2, reports nine passing tests.
The timing wrapper exits **1** after sandbox-denied `sysctl` metrics; the Cargo
child exit was not separately recorded. Raw compilation/test output is retained,
not relabelled as a successful whole-command receipt. Original and resolved
scratch Cargo locks are separately bound. Actual retained hex130 correspondence
and positive coverage for several admitted operation shapes remain untested.

The [candidate report](CANDIDATE-REPORT.md) states the exact scope and limitations.
`CANDIDATE-HANDOFF.tar.gz` retains all 24 authenticated output-manifest payloads
plus the manifest; it does not copy build targets/toolchains or claim all 194
inputs are embedded. Input identities and tested-source records are included.
Every archived member was compared byte-for-byte and source hashes matched
before/after retention. [RESULTS.json](RESULTS.json) binds those identities.

The [independent review](INDEPENDENT-REVIEW.md) requests changes before integration:
observation definitions did not enforce static ordinary functions, direct
recursion ignored resolved callees, and memoized types could evade the structural
depth bound. Its one actual-VIR build exits **101**: thirteen tests pass and two
reviewer fixtures fail. Those failed fixtures are not accepted counterexamples;
the observation counterexample is separately reproduced by a passing probe.
All 19 review payloads and all 20 archive members authenticate; the supplied
archive SHA256 is retained in [REVIEW-RESULTS.json](REVIEW-RESULTS.json).

The parent repaired all three checks in fresh scratch, without another review
chain or parent codec integration. Corrected eligible fixtures establish five
fail-before regressions, including the previously static-only recursion and
memoization findings. After repair, eighteen focused tests pass. Final source
adds dispatch coverage and passes **174 inference-library tests, including
19 codec tests**. The attempted complete VIR run remains **exit 128**: 174 tests
completed before an unrelated solver-dependent test could not find Z3. The final
174-test run uses the same authenticated executable, without rebuilding; no
full-VIR or solver-proof pass is claimed.

[Repair report](REPAIR108-REPORT.md), [exact proposed candidate](REPAIR108-CANDIDATE.patch),
[candidate-relative delta](REPAIR108-DELTA.patch) and [repair results](REPAIR108-RESULTS.json)
retain the distinction between the unchanged reviewed candidate and the parent
repair. `REPAIR108-HANDOFF.tar.gz` embeds 40 payloads plus its manifest, including
final source, all failed/successful raw runs and source/tool observations.
The repaired source has not received a fresh independent review and remains
scratch-only. Selected-input observations are not complete toolchain attestations.
The simple value-enum positive fixture now succeeds; actual hex130 formula
correspondence remains untested.

No persistence is enabled or accepted. The strict persisted warm-cache gate remains **UNMET**, and any future
load still requires three fresh soundness/maximality/exactness certificates
and unchanged installation/body-preservation gates. No native resource proof,
trust or runtime adoption is established. All eighteen redb cells remain **OPEN**.
