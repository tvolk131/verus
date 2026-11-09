# Canonical byte encoding95 — not accepted

This separately authorized bounded application experiment used the original
hex-conservative 1.3.0 native-v3 source archive and a coherent, authenticated
operational91 runtime. It is not the historical hex-v3 engine, a current-parent
proof, or the distinct cached rust-hex 0.4.3 lane. No application/compiler source
is integrated: these files preserve a failed experiment and its exact boundary.

## Observed result

The first full-selected-scope verifier invocation failed with three frontend
visibility errors, before any executable or proof body was checked:

- Selecting actual crate-visible `LOWER` and `UPPER` constants exposes opaque
  `Table` constructors outside the native `table` module.
- The crate-visible helper's required selection equation exposes private
  `self.0` outside that module.

This is not a failed nibble/table arithmetic proof and does not qualify for the
plan's single lemma-correction slot. The delegate stopped immediately. There
was one failed invocation, 0.781733667 seconds, one worker, zero checked bodies,
no semantic negative, no export and no accepted encoding capability. Invocations
2–5 were not executed or repurposed. Preparation/discovery failures are retained
separately in the report and archive, not counted as verifier attempts.

The source input, runtime and parent preservation checks passed. The experiment
does not establish an upstream bug or the impossibility of another annotation
design. It gives no reason to add a primitive axiom, weaken opacity, restrict
the intended all-byte/both-case domain, or infer a need for a new Infer feature.

## Retained evidence

- [Unchanged report](REPORT.md), SHA256
  `41366b954bc0bbade3447c41c049f58a52ff8e10515b31f51af64db72e640bb5`.
- `RAW-HANDOFF.tar.gz`, SHA256
  `158c00719e26f3752f373aecc1d66c0492d42a76b75bf69bd29fb88b7675dd01`.
  This file-only archive contains all 93 handoff files: authenticated source
  inputs, proposed source/proof delta, invocation/environment, raw diagnostics,
  result/proof/scope checks, preparation failures and preservation manifests.
  It contains no runtime binary or build cache. All 92 entries in the delegate's
  output manifest were rehashed successfully before retention; every archive
  member was independently compared with its source bytes afterward.
- `RESULTS.json` records exact classification/accounting and archive-member hashes.

## Next useful task — unexecuted

Design a visibility-respecting specification boundary: public contracts should
not expose private representation, and constant correspondence must still be
checked against the actual native initializers. Any proposal must preserve
executable bodies/visibility, arbitrary-table semantics, all 256 bytes, both
cases and the complete retained scope. This handoff authorizes no retry or
alternative runtime; no proposed design is yet established by verification.

The independently reviewed hardening95 milestone remains separate. No new trust,
certification policy, native theorem or final database credit is adopted.
All eighteen final redb obligations remain OPEN, and every prior no-retry rule
remains in force.
