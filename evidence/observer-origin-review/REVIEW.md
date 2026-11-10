# Independent observer/origin review

**Request changes for two P2 eligibility defects, R1 and R2 in
[FINDINGS.md](FINDINGS.md).** The cited mutable-buffer alias and unrelated-loop
constructor repros are fixed. A reassigned immutable key binding still induces
an invalid generated invariant, and an executable key producer still causes an
Infer frontend abort on an otherwise valid ordinary program. No false-property
acceptance or proof/publication weakening was found. This recommendation concerns
scoped integration, not completion of the native/full-HMAC active goal.

Reviewed committed parent **d35c31fae9d05d7c0e797a60612a8be4716c2dc9**, including
9a44a124 and the prior 168aaca9 review evidence. Work was done without subagents,
production edits, sibling-checkout inspection, push/PR, or access to the parent's
running job, caches, logs, compiler or private Rocq baseline. Only this new evidence
directory is committed. Raw evidence is private at
`/tmp/observer-origin-review-d35c31fa/`.

## Fresh validation

- Own release compiler/macros and matching ordinary full vstd: pass,
  **2,226 verified, zero errors**. No `--no-cheating` was passed to vstd.
- Full VIR library suite: **148 passed**, zero failed/ignored/filtered.
- Focused compiler targets: observer **20/20**, numeric **8/8**, both unfiltered.
- Complete root Infer compiler suite: **618 passed across all 32 root targets**,
  zero failed/ignored/filtered. This includes the focused 28 tests above and full
  `infer` **299/299**, which revalidates F2. Targets run separately with no
  test-name filters; exact per-target counts are in the manifest.
- Canonical replay: **26 expected outcomes**, original controller and independent
  audit pass. Frozen accepted candidate cold/warm **6/0**, executable `{u64}`,
  proof `{}`, inferred helpers `{}`. It retains the full canonical predicate over
  every u64 and arbitrary initialized exclusive `[u8;10]`, with no local mechanical
  proof, theorem call or snapshot. Manual proof remains diagnostic **9/0**.
- Observer replay: **23 expected outcomes**, controller and additional independent
  audit pass. Prefix **2/0 on, 1/1 off**; full XOR-prefix plus unchanged-suffix HMAC
  property **1/1 cold/warm** (postcondition), still unproved. Plain **2/0 both ways**;
  manual canonical without authored length **9/0 both ways** is diagnostic evidence.
- Fresh adversarial probes: **45 frozen sources, 90 off/on invocations**, comprising
  44 original frozen cases plus one separately frozen lifetime-corrected supplement.
  Exact selected exec/proof/helper inventories, decisions, counts and raw log hashes
  are recorded; all app/probe invocations retain `--no-cheating` and normal checks.

Canonical healthy-helper cases explicitly require the `identity` helper and its
soundness/maximality/exactness results before property checks. Semantic negatives
retain healthy helper proofs. Intentional cached helper poison, metadata rejection,
source-shape diagnostics and failed-export cleanup are separately classified. The
additional audit reads the actual exported `.vir` trailer and `.rlib`, checks the
359 trusted-model bindings and named helper proof statuses, and binds the importing
caller's fingerprint to the actual provider bytes. These checks do not authenticate
maliciously forged compiler artifacts.

The observer audit independently requires all 23 labels, their intended failure
classes, the exact selected executable/proof bodies and empty inferred-helper
inventories. Unsupported/mode-invalid frontend outcomes do not count as semantic
negatives. Both failed canonical and observation exports are removed after a
previous successful export.

## Probe results and limits

All original direct-write and four declaration/match-alias F1 repros now verify
**2/0 off and on**, with assistance skipped. Explicit mutable helpers, projections,
returned aliases, escaping and nested target writes also skip and verify. New tuple
move and match-reborrow variants do likewise. The ordinary plain loop retains
assistance and **2/0** verification; readonly target scalar/array copies retain it.
A tuple `ref mut` of a copied array correctly acts on an unrelated value and keeps
assistance. Scalar reads and array copies do not acquire aliases to the original.

Writes hidden in an unrelated array's index, the target's index, an indexed helper
call, or a copied-array temporary before/after the loop all verify off/on while
skipping incompatible observations. Side effects inside constructor blocks also
skip. The original constructor mismatch and nested, conditional, unused-Zip and
shadowed variants verify **3/0 both ways**, with no observer selection. Multiple
mutable constructors/loops conservatively skip; frontend-generated-name lookalikes
in user locals do not confuse the actual binder association.

Stable readonly prefix, transparent key temporary and unrelated before/after-buffer
writes preserve actual useful prefix assistance: **2/0 on versus 1/1 off**. A stable
conditional key expression verifies **2/0 both ways**, with assistance selected.
Combined numeric/prefix code verifies **8/0**, checking exactly `{u64, xor_key}` with
no proof/helper bodies and observations selected for both. Its off **3/3** is a
mixed diagnostic, not a strict semantic negative.

R1 has two Rust-valid sources: local and explicitly same-lifetime parameter key
reassignment. Each verifies **2/0 off** and fails **1/1 on**, solely at the generated
loop-end invariant. R2 checks exactly `{make_key, client}` and verifies **3/0 off**;
on aborts as unsupported before proof. See the minimal sources and commands in
FINDINGS.md. The relevant key-expression reuse is inherited from before both fixes;
no older runtime was built to claim a fresh regression comparison.

An earlier parameter variant without common lifetimes fails normal Rust lifetime
checking in both modes. The primary runner recorded the uncoded rustc diagnostic as
`other` (0/0); the independent audit explicitly reclassifies it as **frontend**.
Both raw records remain intact, and the separately frozen corrected supplement
provides R1's second valid repro. Illegal overlapping aliases reject **E0499** in
both modes. Those four invalid-source frontend runs, and R2's unsupported frontend
run, are excluded from semantic-negative claims. Overflow after an intentional
observer skip remains **1/1 both ways**, preserving normal safety checks. No
resource failure is used as a proof outcome.

After the audit correction, the 90 invocations comprise 77 successful runs, three
prefix-off postcondition failures, three invariant/mixed-diagnostic failures (two
R1 manifestations and the composition-off control), two overflow failures, and
five frontend outcomes. A successful ordinary proof of a trivial writer is not
credited as full transfer or as proof of the native frame theorem.

The long-running outer validation terminal reported **exit 143 when reaped**,
after all 33 test commands (VIR plus 32 Infer targets) had persisted complete
passing summaries and individual return codes zero. The earlier two vstd setup/
build commands also returned zero. The reason for the outer status is unknown;
`session-completion.json` preserves the observation. No successful outer-wrapper
exit is claimed, and no partial test rows or wrapper failure are counted as a
semantic outcome. The collector requires the exact complete target inventory,
zero per-command returns and zero failed/ignored/filtered tests before accepting
the recorded **148 + 618** results. There were no test retries.

## Source and checking review

`entry_observations_compatible` now visits statements and places. Direct mutable
acquisition through declaration initializers and match scrutinees is no longer
missed. Readonly value reads have a separate projection walk that still traverses
index, temporary and WithExpr expressions. Its loop and exact mutable-constructor
exemptions are bounded by the native descriptor's expression budget.

The new shared `native_mutable_for::for_source` follows the actual initializer,
IntoIterator operand, wrapper constructor operand, match scrutinee and arm binding.
The observer compares the full serialized source constructor (including identities)
and checks the invariant wrapper against that actual binding. Numeric assistance
uses the same parser and `plain` scaffold normalization. This closes the unrelated
same-type constructor association tested here; multiple/unsupported constructors
are conservatively skipped.

Independent whole-Function projection and scaffold validation still constrain
exact additions, original executable/contract/invariant/decreases/resolution nodes,
initializer modes, unique generated binders and fresh expression/place/read IDs.
Observer allocation reserves prior numeric-pass identities. Both decorators request
full function well-formedness and mode rechecks, refresh resolution maps, and then
undergo normal trait, lifetime and body verification. Projection/freshness mutation
controls are included in the fresh full VIR run. The successful composed proof is
additional integration evidence; it does not establish every possible composition.

All generated observations remain checked proposals. R1 is therefore rejected by
ordinary invariant checking; R2 fails the annotation purity boundary. The missing
key eligibility tests cause valid-program rejection, not unchecked proof facts.
No new assumptions, vstd facts, native rules or model contracts were introduced.
Policy **68**, candidate cache **71**, metadata schema **38** are unchanged.
Publication still requires complete normal verification and all required helper
proofs; the fresh poison/import/export controls exercise that barrier.

## Reproduction and evidence binding

Build from `source/` with `/home/dev/.cargo/bin` first in PATH,
`CARGO_BUILD_JOBS=2`, and `VERUS_Z3_PATH=$PWD/z3` (own Z3 **4.16.0**):

```sh
cargo build --release --locked -p rust_verify -p verus -p cargo-verus -p vstd_build
cargo generate-lockfile --offline --manifest-path vstd/Cargo.toml
# Prepend this checkout's target-verus/release to PATH:
target-verus/release/cargo-verus verus build --release --locked --offline --manifest-path vstd/Cargo.toml -- --num-threads 2
```

The committed controllers use this checkout, the fixed private raw directory and
private caches. `validate.py --build-vstd` records the lock/vstd build;
`validate.py` runs full VIR and all 32 root Infer targets with `--test-threads=2`.
`freeze_probes.py` freezes independent source copies before trials. `probe.py` and
`supplement.py` run those two pretrial inventories. `replay.sh` makes an immutable
non-hardlinked, non-symlinked tool bundle and runs the probes, both replays and audits.
For a new replay, use a new private raw path consistently; existing immutable bundle
outputs deliberately refuse overwrite. The supplement was frozen and run separately
before the independent audit, after identifying the first probe's lifetime error.

`source-freeze.json` binds **2,048 tracked source/dependency/toolchain files** before
building. They are rehashed after validation; no source drift is accepted. The
ordinary vstd build's generated lock is copied as `vstd-build.Cargo.lock`. Runtime
receipts bind Rust **1.99.1**, rustc driver, cargo, solver and copied compiler/macros/
vstd; provider producer includes the exact reviewed d35c31fa commit. `manifest.json`
binds commands, environments, exact selected inventories, source/log/receipt/artifact
hashes, independent audits and compact evidence files. Raw logs remain in the private
directory and are not substituted with historical counts.

## Goal boundary

Only scoped integration can be accepted after the eligibility findings are handled.
The frozen canonical domain remains every u64/arbitrary initialized exclusive
`[u8;10]`. The independent frozen bitcoin_hashes **0.14.0** domain remains arbitrary
initialized `[u8;128]` and **all valid immutable key lengths**, including empty and
longer than 128. No zero-buffer, layout or numeric-input restriction was added.
The full property still includes XOR-prefix **and unchanged suffix**, and still
fails **1/1**. Prefix success cannot close that requirement.

Trust includes Verus/rustc/macros/SMT and inherited iterator/prophetic-reference and
external standard models. Ordinary vstd retains its inherited **83 admissions**;
no new application admission or native fact is installed. There are still **zero
actual native correspondences**, **101 open native effect obligations**, and native
publication is **false**. The committed native checker inventory corroborates those
open statuses, and source inspection confirms the unconditional unimplemented-proof
publication barrier. No native extractor, conditional **115/0** model proof or private
Rocq baseline was rebuilt here; none is presented as a native ownership/refinement
proof. Finite native execution was not rerun or credited in this review.
