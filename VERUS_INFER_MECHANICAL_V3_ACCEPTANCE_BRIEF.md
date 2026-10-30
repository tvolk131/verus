# Fresh-context acceptance audit: mechanical adoption v3

The frozen requirements are in VERUS_INFER_MECHANICAL_V3.md. Audit the proposed
adoption using the fixed compiler, general inference/adoption guides, actual
native source and explicitly authored semantic definitions. The parent result
does not complete independent acceptance.

## Fixed inputs

- Engine: clean `fbab46ec4c5a1dd600c80d094ea7fe4fb90e34dd` at
  `/private/tmp/verus-mechanical-v3/engine/verus`; source/build directory `source`,
  executables/libraries `source/target-verus/release`, Z3 `source/z3`.
- Artifact receipt: `/private/tmp/verus-mechanical-v3/PINNED_ENGINE.json`.
- Native source: `/private/tmp/verus-mechanical-v3/aleph`, implementation commit
  `8b7e0f1` on `verus-infer-semantic-adoption-v3`. Existing semantic definitions
  and coverage are authored inputs; supporting implementation proof scaffolding
  is not permitted.
- Original unannotated source for token comparison is under
  `/private/tmp/verus-mechanical-v3/original-syntax`, exported from original Aleph
  `f35c7bba6d628bcafdf0d65dd471cc6f868ec034`.
- Rust proof toolchain 1.99.1, native test toolchain 1.85.1. Use an offline locked
  Cargo build and a matching fixed-vstd patch. Use fresh audit Cargo targets and
  candidate cache, outside the fixed source/engine.

## Required independent decisions

1. Check actual native AddAssign selection/certificates and composition into the
   collector. The original derived implementation and `+= NodeCount(1)` remain
   executable; no authored AddAssign arithmetic model or primitive-field rewrite.
2. Compare the complete ControlHash::new and Creator mutable-selector bodies to
   original tokens. Verify the original loop/opaque-suffix constructor and the
   inline growth/mutable-return path without extracted implementation helpers.
3. Audit actual compiled proof closure, precise helper checks and application
   properties, including retained v2 coverage and domains. Compiler success or
   a selected-scope report alone does not prove the semantic claims.
4. Check that source views/invariants/interfaces express meaning, with no local
   supporting proof blocks, executable-loop annotations, trigger hints, mechanical
   getter contracts or author-written mutable-selector identity/frame clauses.
5. Retain the original quorum domain, initial arbitrary IDs/zero membership,
   positive input coverage, raw/checked round boundaries, complete error ordering,
   opaque hash comparison meaning and actual creation-output eligibility. Do not
   assume hash determinism, generic hash Clone identity, cache consistency,
   encoding equations or new first-party behavior. Signed serializer exclusions
   remain unauthorized and unapplied.
6. Independently run the complete finite proof with fresh outputs/cache and
   ordinary native tests. Inspect semantic mutation/publication evidence and
   distinguish it from original bug discoveries. Source and pinned artifacts must
   remain unchanged. Record concrete objections, missing evidence and honest
   limits rather than weakening acceptance.

Audit read-only. Do not edit application/engine source, relax assumptions or
requirements, push, inspect prior Aleph proof branches/reports, or ask the parent
for solution hints. The current proposed source and its final evidence are the
objects being reviewed. General fixed-engine documentation/source/tests may be
read. Put fresh commands, receipts, findings and final assessment in a separate
task-owned audit directory.
