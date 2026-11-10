# Frozen external-model baseline interface / pretrial gate

Base checkout: 6c74c9b1. Ownership: only this new directory and
`/tmp/tree-borrows-checked-foundation-20261110/`. No subdelegation, Rust builds,
engine/vstd/global policy changes, shared environment changes, publication or push.

Authenticate/copy the completed Simuliris snapshot against inspection pins and
all tracked git blobs at e3abfcdc301f92250d4227fc03c00463bf57b982. The full 185-file
SHA256 inventory is `upstream-source.json`. Actual build file is `_RocqProject`,
not `_CoqProject`; it requires 9.2 warning options. Select Rocq 9.2.0, Equations
1.3.2+9.2, Iris dev.2026-09-10.3.b3495ef9 (not `dev`). Freeze complete transitive
package versions, repository revisions, package metadata and source archive or
git digests before compiling any proof. Bootstrap OCaml 4.14.2 and opam 2.3.0
privately; maximum two jobs. No global installs/repository updates. Repository
snapshots: opam-repository 78c202b54f936f55f02e3237df71f8b4d2405807,
opam-coq-archive 06733fd8dcbd7ca2e61bc38c72d6062a52f20d57,
iris/opam cf9219cd2ae11d400e34233b14e245285546dbca.
Opam binary SHA256 324e78e3f33efeba279aacf9f9610cfec7b2df7d7e0e1640f75f09de85f96cc9.

Scope roots (under theories): tree_borrows/{bor_semantics,defs,steps_wf,
steps_progress,adequacy}.v, tree_borrows/step_laws/steps_retag.v and
simulation/{slsls,lifting}.v. Compute the actual recursive closure with Rocq's
dependency generator and normal upstream make rules; do not hand-select only
successful modules. Compile full closure normally and recursively kernel check
all roots and imported modules, with no unsafe exclusions, admitted obligations,
axiom stubs or silently changed semantic rules. Inventory compiled declarations
including definitions, constructors, eliminators, projections and dependencies;
record individual assumptions for semantic/progress/retag/adequacy/public
simulation exports and complete closure axiom/unsafe-flag summary.

## Baseline result gate

Initial state is NOT READY and NOT CHECKED. Only publish the private
`baseline-ready.json` after a durable successful normal build receipt AND full
recursive kernel check, all inventories/assumptions, runner/loadpaths/tool
versions, exact source/dependency/object hashes, and final guards matching the
pretrial snapshot. A compiler/object/metadata success alone never passes this
gate. Keep failed setup/proof trials separately from semantic controls (none
planned); keep durable pid/pgid/startticks/log and terminal status for own jobs.
Timeout is not terminal. Any inability to complete the full closure is reported
as partial/blocked, with no ready artifact. Any compatibility patch needs full
isolated diff and pins, preserving equations and theorem statements.

## Exact interpretation and limits

This is a checked published EXTERNAL MODEL baseline only. `source_safe` and
`pool_safe` are hypotheses to inventory, never proofs that the source is safe.
Fresh child creation/read and Reserved-to-Active transitions are not Rust
borrowck two-phase activation. Actual pinned rustc operations, arbitrary lawful
incoming references/ancestors, phase/cleanup behavior, source observation, and
original/pending Verus C/F require separate unproved correspondence. No source
safety, defined-execution, no_unwind, permission, narrower domain or successful
operation premise may be smuggled into entry validity.

Canonical every-u64/arbitrary initialized [10] remains 6/0; original independently
frozen HMAC every valid key length including 0 and >128/arbitrary initialized
[128], XOR prefix AND unchanged suffix, remains 1/1 unproved. Native=0;
all 44+3 minimal/phase and 101 writer effects open. Same actual checked assistance,
normal fidelity/trust/inference/publication gates and fresh combined independent
acceptance remain required. Publication=false; overall goal incomplete.
