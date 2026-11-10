# Independent combined mutable-buffer review

Scoped recommendation: **request changes for F1 and F2 (P2)**: the generated-snapshot
eligibility regression and stale compiler regression assertion in [FINDINGS.md](FINDINGS.md). Canonical/prefix milestones
are independently reproduced. No false-property acceptance, new trusted law,
source-body rewrite or termination weakening was found in this review.
This is not whole-goal or native-correspondence acceptance.

Reviewed checkout **c3bf5d36a88acb4ef1120b7becbad14a594944d5**. Production VIR,
rust_verify and vstd bytes match **676b99bf**. No production/application code,
policy/cache/schema, parent/sibling artifacts or global settings were changed.
Only this evidence directory is committed. Raw evidence is private under
`/tmp/shared-buffer-review-c3bf5d36/`.

## Fresh validation

- Own release compiler, macros and matching vstd build: pass; **2,226/0** vstd.
- Full VIR library suite: **148 passed**, zero failed/ignored/filtered.
- COMPLETE root Infer compiler regression suite: **606 passed, 1 failed, 0 ignored, 0 filtered**.
  All 32 discovered `source/rust_verify_test/tests/infer*.rs` targets were run;
  `manifest.json` records each target's actual counts and log hash. The numeric
  eight and observation nine tests are included, not substituted for this suite.
- Python Infer harness tests: **148 normal + 148 optimized**, both pass.
- Canonical replay: **26 expected outcomes**, replay and independent audit pass.
  Cold/warm **6/0**, exec `{u64}`, proof `{}`, inferred helpers `{}`. Healthy
  `identity` helper cases reprove required soundness/maximality/exactness.
  Semantic negatives keep that helper healthy; poisoned-helper, unsupported
  shapes, inherited cast recommendations and metadata rejection stay separate.
- Observation replay: **23 expected outcomes**, independently audited against
  frozen sources/raw logs and explicit named body/helper inventories, including
  zero helpers. Prefix **2/0 on vs 1/1 off**; full HMAC **1/1** postcondition
  failure cold/warm; manual canonical remains **9/0** both ways and plain
  remains **2/0** both ways. Those diagnostics are redundant, not causal transfer.
- Actual macro-erased prefix/full versus original selected upstream loop:
  **2,816 cases each in debug/release**, including all suffix bytes. Finite
  execution evidence, not a formal native frame or refinement theorem.
- Nine frozen additional probes: simple loop **2/0 both ways**; before/after
  buffer writes **2/0 off, 1/1 on** (F1); two numeric encoders **12/0**, with
  both qualified executable bodies checked and no proof/helper bodies; illegal
  overlapping mutable borrows reject **E0499 both ways**, classified as Rust
  frontend failures rather than semantic evidence.

The actual exported provider, rlib, proof trailer, helper identity/statuses,
359 trusted-model bindings and importing caller's exact provider fingerprint
were checked. Policy **68**, cache **71**, schema **38** remain unchanged.
The canonical poisoned helper hits the cache but cannot publish; invalid metadata
rejects import; seeded failed canonical and observation exports disappear.
Disabled-check/filter tests are expected policy rejections, never reinterpreted
as successful verification with disabled checks.

## Source and checking review

Inspected `buffer_goals`, `buffer_observation_goals`, `native_mutable_for`,
`pipeline`, `report`, `assistance`, registry cache, query, infer_report, verifier,
import_export, config and source/evidence controllers. Numeric recognition runs
before observer decoration; the observer reserves every preceding function and
proposal's identities. The common wrapper catalog closes generic bounds and
Spec result mode. Observation snapshots are ghost declarations/initializations,
not executable writes. Their placement has the F1 locality defect.

Each decorator independently projects its whole incoming Function, preserving
original executable, contract, invariant, decreases and resolution nodes. Observer
schema/freshness tests reject consistently corrupted receipt/body identities,
existing parameter/local/quantifier names, prior-pass names, duplicate binders,
node collisions, executable writes and mode corruption. Composed numeric/observer
code passes full function WF and mode checking with refreshed resolution maps;
ordinary safety, auxiliary assertions/lemmas and original application obligations
remain the acceptance boundary. Selection reports and empty inferred lists alone
are not treated as proof.

The cache stores typed candidate formulas, not proved results. Publication waits
for the ordinary complete verification run and all required helper proofs.
Imports check schema/policy/producer, API fingerprint, proof inventory, dependent
fingerprints and artifact-bound standard models. These consistency checks are
**not cryptographic validation of malicious .vir/.rlib artifacts**.

## Commands and tool identity

Rust **1.99.1**, Z3 **4.16.0**, own Verus **0.2026.11.10.c3bf5d3**, release.
Build from `source/` with `/home/dev/.cargo/bin` first, `CARGO_BUILD_JOBS=2`:

```sh
cargo build --release --locked -p rust_verify -p verus -p cargo-verus -p vstd_build
cargo generate-lockfile --offline --manifest-path vstd/Cargo.toml
# PATH also starts with this checkout's target-verus/release; explicit solver:
target-verus/release/cargo-verus verus build --release --locked --offline --manifest-path vstd/Cargo.toml -- --num-threads 2
cargo test --release --locked -p vir --lib
# For EVERY one of the 32 root Infer targets (manifest records exact argv):
cargo test --release --locked -p rust_verify_test --test TARGET -- --test-threads=2
```

Compiler tests use `VERUS_IN_VARGO=1`, `RUST_MIN_STACK=33554432`, explicit
`VERUS_Z3_PATH` and private cache/log directories. The common Infer test helper
hard-codes this checkout's `source/z3`; that ignored file is a private solver
copy with the recorded hash. The initially absent solver/vstd lock/bundle were
setup conditions, not proof outcomes. No failed proof run was relabeled.

Replay/audit command arrays, all tool/source/log/receipt hashes and per-case
outcomes are bound in `manifest.json`. The immutable bundle contains independent
copies (no symlinks/hardlinks) of the freshly built compiler/macros/vstd and the
recorded solver. A 1,093-file pretrial tracked-source inventory is hashed and
rechecked after validation. The supplemental audit can be repeated with:

```sh
python3 -B -O evidence/shared-buffer-review/audit_replays.py --raw /tmp/shared-buffer-review-c3bf5d36
```

## Trust and exclusions

Both domains remain unchanged: **every u64/arbitrary initialized exclusive
[u8;10]**, and **arbitrary initialized [u8;128]/every valid immutable key length**,
including empty and keys longer than 128. No zero-buffer or layout restriction.
Authored semantic definitions, processed-prefix invariants and exit obligations
remain authored inputs; generated mechanics remain checked proposals.

Trust includes Verus/rustc/macros/SMT and inherited vstd/external iterator and
prophetic-reference specifications. The normal vstd build retains its inherited
83 admissions. All canonical/observation replays and added probes use
`--no-cheating` and normal lifetime/trait checks; regression targets retain their
normal harness settings, including intentional policy-negative flags. No new
vstd assumption or native rule/model/trust change was installed.

Native research publication controllers were inspected and still fail closed,
including forged `proved` labels. Actual native ownership entry, pointer effects,
helper/drop effects and source refinement remain unproved; all 101 native effect
obligations remain open. Native extraction/conditional carrier proof research was
not rebuilt here and is not cited as fresh proof of those laws. Full frozen HMAC
XOR-prefix **plus suffix** remains **1/1** and the whole goal remains incomplete.

The optional capacity ablation was not performed: full regression coverage took
priority. Sole ownership of the sequence-capacity proposal is structural evidence;
this review makes no claim that capacity is indispensable, nor that parser reuse
or prefix success establishes full shared transfer. Historical 143/17/148 results
are not substituted for any current validation above.
