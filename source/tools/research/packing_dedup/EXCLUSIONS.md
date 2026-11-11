# Retained exclusions

* `test-rust_verify-unit-missing-release-setup` is retained in PRIVATE_ROOT and
  included in RESULTS.json receipts. It ran all six compiler library units:
  five passed (all three new packing units and both API fingerprint units),
  while the existing tuple-mapping unit failed opening its hardcoded
  `source/target-verus/release/vstd.vir` path. This is a missing test fixture,
  not a failed proof or accepted semantic result. The controller now aliases
  that own release directory to the already frozen matching bundle and reruns
  the same executable, unmodified, under `test-rust_verify-unit` (6/0).
  No production/test assertion or compiler byte changed for the correction.
* The original preliminary AIR audit is scoped to explicitly completed evidence.
  It removes diagnostic comments and occurrence-only qid/skolemid labels for
  inventory comparison; it is not an SST equality oracle or trusted fact source.
  Fresh independently built disabled-selection trials provide the real ablation.
* Completed historical combined71-local (adapter-WF setup failure) is excluded
  as instructed; it was not inspected or reused. The successful v2 controller
  and other supplied old audit executables are not executed unchanged.
* Historical .vir/provider bundles are never positive inputs for a changed
  compiler. Only the completed eight-tool bundle is authenticated for baseline
  inventory; the solver and enumerated offline Cargo inputs are copied from
  permitted completed evidence. No supplied directory is modified.
* Parent/sibling live checkouts, targets, caches, configs, processes, replay/state
  and running all36 evidence are excluded. No claims about their outcomes.
* Debug concurrent wall times are retained, including slower candidate timings;
  there is no speedup claim, release-performance claim, cherry-picked timing
  sample, warm-cache substitution for fresh Eq checks, or fixed-total success gate.
* Native/whole-goal and inherited canonical/HMAC obligations are outside this
  optimization experiment. See ASSUMPTIONS.md; no such obligation is closed.

Audit development retained in private `whole-typed-differences.private` and
`cw-typed.private`: a strict complete-form comparison first exposed the generated
Eq candidate ID difference between independent bundles, and a first strict C/W
audit refused the existing one-token Eq ensure diagnostic relocation. No engine
trial was discarded or rerun for these differences. The final audit records both
IDs/raw log hashes and permits exactly that generated attribute rebinding between
bundles; across C/W it requires the complete typed log to differ in exactly one
known diagnostic token (5:19–5:31 →4:29–4:38, expansion18), confined to the Eq
ensure. All remaining semantic/typed tokens must match. Auxiliary AIR contains
one additional Function-Recommends diagnostic query after failure; it is recorded
separately from genuine verification totals rather than silently counted as a
successful proof. These are inventory qualifications, not proof/import bypasses.
