# Owned-candidate codec108: parent repair of independent review findings

Date: November 7, 2026, America/Chicago; execution receipts use November 8 UTC.

## Disposition

The independent codec107 review requested changes before even private test-only
integration. The parent fixed its three findings in fresh scratch, without
another delegated review chain. The corrected prototype is retained as a
candidate, not integrated into parent source. This is no production-readiness,
persisted-cache, native-proof or compiler/source-correspondence approval.

The original candidate and original independent review remain byte-identical.
In particular, the review's 13 passes and two failed reviewer fixtures remain
historical results; neither failed fixture is relabelled as a successful probe.

## Repairs and fail-before evidence

- **R1, observation definition eligibility:** require `FunctionKind::Static`,
  `ItemKind::Function`, specification-mode inner and outer returns, and agreement
  of their types. The existing pure-mode/body/generic/proxy/parameter/name gates
  remain. Regressions reject a trait-method declaration, const/static items and
  non-specification or mismatched returns; a valid static call still round-trips.
- **R2, direct self-recursion:** use `DynamicResolved.resolved`, not the nominal
  trait-method name. Unknown dispatch, proof-function calls and unresolved
  external defaults are conservatively refused in this restricted source profile.
  Static and resolved non-self calls remain possible. A corrected simple
  value-enum-returning From fixture passes both the owned signature gate and
  nonrecursive Context construction, then demonstrates the unwanted resolved
  self-call acceptance on the original codec. The repaired codec rejects it.
  This is a new parent reproduction, not the earlier failed reviewer probe.
- **R3, structural type depth:** retain each interned type's structural height
  and charge that height when a memoized child is reused under new ancestors.
  The bound is 12 nodes from root to leaf. Existing count/cycle checks remain.
  Both direct repeated Option wrapping and sequential valid observation
  signatures demonstrate the old acceptance beyond the bound and the repair's
  refusal. Depth 12 remains accepted; depth 13 is rejected.

The first parent run executes these regressions against the unchanged codec
semantics: **13 pass, five fail**, Cargo exit **101**. All five failures are at
the intended refusal assertions after their acceptance/control prerequisites.
The simple value-enum positive fixture succeeds in this run, unlike the invalid
nested-Envelope enum fixture attempted by the reviewer. Its replacement does
not expand the supported datatype profile. The nine original author tests pass.

## Validation and accounting

Three offline Cargo library-test invocations, jobs 2, used a fresh copied
air/vir/vir_macros workspace and the original candidate's resolved lock. No
compiler, companion-vstd or native application build occurred. The workspace
is deliberately smaller than the parent workspace; this does not claim an
identical full-workspace feature graph or parent build.

1. **Red:** original codec plus corrected regression fixtures: 13 passed,
   five expected regression failures; exit 101.
2. **Green:** repaired codec: **18 passed, zero failed**; Cargo exit 0.
3. **Broader attempt:** final formatted source plus an additional dispatch
   regression: 211 selected tests, **174 completed passes**, including all
   **19 codec tests**. The next, unrelated solver-dependent `infer_checks` test
   could not start Z3 under the selected environment. Cargo/test exit **128**;
   there is no complete full-VIR pass or solver-proof credit. No Z3 process ran.
4. **Final solver-free inference recheck:** invoke the exact authenticated unit
   test executable from invocation 3 with filter `infer::`, without rebuilding:
   **174 passed, zero failed, 37 filtered out**, direct test exit 0. This includes
   all 19 final codec tests and the existing inference regression suite.

The additional dispatch regression covers direct static self-calls, accepted
non-self static calls, resolved actual-vs-nominal identity, and refusal of
unknown/proof/external-default dispatch. These are synthetic actual-VIR fixtures,
not a claim that a pinned frontend emitted every combination used by a test.

Each invocation records argv, explicit environment, source identities, selected
tool identities and the actual direct child exit. Actual toolchain cargo/rustc/
rustdoc paths are used directly, with RUSTC/RUSTDOC fixed; no rustup shim is in
the selected PATH. The final unit executable is hashed before and after its
direct run. These selected-input observations do not authenticate all transitive
linker, dynamic-library or compiler-build inputs and are not attestations.

Original 194 input identities, 194 tested identities, 24 candidate payloads and
19 independent-review payloads authenticate at entry and each run's preflight.
The review archive matches its supplied SHA256 and every embedded member.
Copied baseline source authenticates across all 190 records. All per-run source
records and selected tools match after execution. Final exit authentication is
retained separately. Endpoint hashes do not claim continuous/atomic observation.

No Git writes, network, parent source changes, persistence/inference wiring,
trusted laws/producers, certificate/admission/export changes or excluded-task
retries occurred. The receipt helper integrated in the previous round is not
modified here. Scratch is approximately 323 MiB; sampled endpoint free space
stays above 16 GiB. The four-minute per-command timeout was never reached;
resource observations are endpoints, not continuous hard quotas.

## Remaining gates

- The repaired source has parent regression validation, **not a fresh independent
  review of the repair**, and remains scratch-only. No integration is authorized
  merely by retaining it.
- Actual retained hex130 formula correspondence and persisted owned-candidate
  reuse/invalidation/failure controls remain untested. The warm-cache gate stays
  **UNMET**. Positive enum fixture coverage is now present, not universal exactness.
- General transitive call-graph recursion remains outside this restricted check.
  Caller identity/configuration/dependency completeness remains an obligation.
- Every future load still requires three fresh soundness, safe-domain maximality
  and exactness certificates, with existing installation/body-preservation gates.
- Compiler/source, source-path/Git continuity, native resource/storage correspondence
  and final native acceptance remain unestablished. All **18 redb cells OPEN**.
- The proposed 60-minute/4-GiB native storage-refinement implementation/review
  round and the separate compiler/Arc diagnostic remain **unapproved**. Neither
  is executed or authorized by this codec repair.

`repair-only.patch` is relative to the unchanged codec107 candidate. The full
`candidate.patch` is relative to the current parent owned.rs baseline and adds
only a private cfg(test) module and its two files. Raw red/green/broader/final
logs, selected source/tool observations and exact final source are included in
the repair handoff. Large build targets and toolchains are deliberately omitted.
