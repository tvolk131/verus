# Frozen external-model build boundary

Own only this new directory and `/tmp/stbor-checked-foundation-20261110/`.
Upstream: official Stacked Borrows commit
`d1471b10fe4cb4f506371b2e3acdd927d0a0c076`; authenticate the completed
inspection snapshot against its committed pins and Git tree, copy tracked bytes
privately, and guard the entire source tree. No modifications of upstream formal
semantics are authorized by this controller.

Plan: private opam 2.3.0; private OCaml 4.09.1 (or a separately frozen compatible
compiler if exact package constraints demand it); Coq 8.9.1; exact Iris
`dev.2019-12-20.0.62798412`; Paco 4.0.0; Equations 1.2.1+8.9. Resolve all transitive
packages from content-pinned repository snapshots and freeze the resulting
archives, package metadata, tool configuration, and complete dependency sources
before formal compilation. Any portability patch needs its own exact diff,
justification and hashes. Never resolve an unpinned dev dependency. Jobs <= 2.

Typed result gate (initial state: NOT READY):

* `setup_failed`: a setup command failed; no semantic result.
* `partial`: explicitly named modules compiled; no complete baseline.
* `checked_external_model`: every `_CoqProject` module compiled normally,
  complete recursive coqchk succeeded without exclusions, declaration/module/
  source/object/dependency/assumption maps are complete, all guards matched,
  and a durable receipt binds commands, exits, logs, tools and products.

Only the last state can produce `baseline-ready.json`, content-bound to that
successful receipt. Metadata is evidence indexing, never a proof premise. No
new axiom/admission/unsafe mode or proof wrapper is allowed. Compiler success
alone is not kernel-check success; neither establishes native validity.

Full goal is unchanged: canonical all u64/arbitrary initialized exclusive [10]
6/0; original independently frozen HMAC writer arbitrary initialized [128],
every valid key length (including 0 and >128), XOR prefix AND unchanged suffix,
1/1 unproved. Same actual checked assistance and normal fidelity/trust/inference/
publication gates plus fresh combined independent acceptance remain required.
Native=0; all 44 minimal + 3 phase / 101 writer effects remain open;
publication=false; goal incomplete. No lawful-entry restriction, source safety,
defined-run, no-unwind, permission, correspondence, C/F, conservative134
refinement, or modern Rust/Miri equivalence is assumed or concluded here.

All failed setup/proof trials are retained separately from semantic controls.
No Rust rebuild/full suite, global environment changes, sibling reads,
subdelegation, push or PR. Early freeze precedes dependency/toolchain builds;
the fully resolved source/tool/dependency freeze must precede proof trials.
