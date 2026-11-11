# Setup and analysis exclusions

All final acceptance trials use the frozen final source-matching bundle and
fresh immutable output directories. Raw logs, MIR, AIR, binaries, caches and
fixtures remain private under PRIVATE_ROOT. Excluded outcomes are retained.

* The command sandbox failed at startup with a loopback permission error.
  Approved checkout-local commands ran outside that broken sandbox; no network,
  host configuration, toolchain, shared cache or sibling/parent state changed.
* An initial setup tried to require every workspace lockfile archive, including
  unused atomic-waker. Removing presence checking was rejected by automatic
  review. The accepted alternative authenticates the completed independent
  review's enumerated inputs, validates present archives against Cargo.lock,
  compares every copied sparse-index/getopts object, and uses offline/locked
  jobs2 with private CARGO_HOME, target-dir and separate build-dir. Missing
  selected dependencies remain hard Cargo failures. No archive was substituted.
* The checkout had no author configuration. Commits used command-local author
  options; no shared/global Git configuration was edited.
* Development build2/3 proof outputs are archived separately and excluded from
  final acceptance; their superseded executable products are not asserted to
  be retained. Final build4, its debug-stripping receipt, all eight tool hashes
  and frozen-final-bundle are retained. Only the final source-matching vstd run
  supplies final vstd evidence.
* prepare-focused-tests finished compilation while vstd was being regenerated,
  so its tool freeze guard failed. It is setup-only and not a passing test
  receipt. The final test runs freeze the completed final bundle.
* The first regression harness looked for tools under target/debug while this
  Cargo places test executables under build/debug. It failed ENOENT before any
  test proof ran. The corrected private profiles contain checksum-identical
  final tools. A subsequent redundant copy of read-only z3 failed; setup now
  skips checksum-identical files or replaces only private mismatched products.
* The new raw test clients initially lacked explicit matching standard-model
  imports. Their fixture helper now imports vstd normally. No contract, proof
  expectation or production import requirement was weakened.
* explicit-no-erasure-check combined mutually exclusive --compile and
  --no-erasure-check, so CLI processing rejected it before compiler callbacks.
  That is not a valid incomplete-check trial. Fresh explicit/cargo *-valid
  trials omit --compile for that option and reach the required checking gate;
  selected stale outputs disappear. The invalid invocation and seeded output
  remain retained, excluded from this valid-CLI guarantee.
* The first legacy consumer put --internal-test-mode after the filename.
  It must be the first argument. Fresh v2 trials exercise both directions of
  actual old/new format refusal with the authenticated accepted legacy engine.
* The first erasure fixture used not(verus_keep_ghost), false in both passes,
  and correctly verified/exported2/0. It never exercised an erased failure.
  Fresh v2 fixtures use not(verus_keep_ghost_body), producing an actual E0308
  after the ghost proof has completed2/0. Neither explicit nor cargo publishes.
* The initial full cold/warm analyzer required equality of the entire trailer.
  Only summaries differed; required foreign evidence, helper query, candidate
  identity, required Eq statuses and all three Eq query bytes matched. The
  corrected read-only analyzer checks those requested identities and records
  both independently self-bound raw API fingerprints. Original epoch data and
  the original analyzer are retained unchanged. No logical oracle was relaxed.
* The native MIR printer qualifies the retained trait as std::convert::From.
  The first inspection matcher expected an unqualified label. Read-only
  reanalysis of the unchanged emitted MIR confirms the exact typed original
  call with one Copy input and no synthetic helper. It supplies no native
  theorem. The first acceptance auditor also overrequired file absence for
  goals-off, which has no foreign capability. Its0/1 proof result and absence
  of a certificate are required; a legacy ordinary declaration export retains
  inherited cap-free artifact trust and is not certified proof evidence.
* Serializer-field mutations are metadata refusals, separately classified from
  healthy false-client/false-BV solver failures and erased compiler failures.
  All are ordinary proof/semantic/compatibility/publication regressions, not
  security, race or adversarial artifact-authentication experiments.

No parent/sibling live checkout, process, build, cache, configuration or
WORK_STATE was read. Historical paths are labels; accepted code was rebound to
this checkout's Git bytes. Only the explicitly named completed bundles/input
artifacts were consulted. No push, PR, native/HMAC proof, global reconciliation
or independent-review acceptance is claimed. Parent's all710 compiler,
per-mode148 Python, canonical26 and observer23 suites are inherited; this lane
runs the focused changed Infer/export/interface/foreign suites and149 VIR tests.
