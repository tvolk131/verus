# Fresh independent stable-key recheck

Recommend scoped acceptance of **a93b774adf5c035aa0186a6efa4f60103552bd0b**;
R1/R2 are closed and no new blocker was reproduced. This does not accept the
whole native/full-HMAC goal. [FINDINGS.md](FINDINGS.md) gives the disposition;
[PROBES.md](PROBES.md) gives every frozen off/on outcome.

Reviewed worktree baseline **e84eed6c79d9c104cefada569de0c99221dc2b51**. Relative
to **d35c31fae9d05d7c0e797a60612a8be4716c2dc9**, all `source/` files outside
research are unchanged except the descriptor and three regression tests. Those
production/test bytes match a93b774a exactly. Own compiler/macros and ordinary
full vstd were built here with Rust **1.99.1** and `CARGO_BUILD_JOBS=2`. The
read-only parent bundle supplied only the copied, pinned Z3 **4.16.0** solver
and a source-hash reference; its compiler/macros/vstd were not run or reused.

No subagents, push, PR, implementation edits, parent/sibling edits, parent suite
polling or private Coq/Rocq builds were used. Only this new evidence directory is
committed. Raw evidence is `/tmp/stable-key-review-e84eed6c/`.

## Validation

- Own release compiler build: pass. Matching ordinary full vstd: **2,226/0**;
  no `--no-cheating` was passed into vstd.
- Complete VIR library suite: **148 passed**, zero failed/ignored/filtered.
- Complete focused compiler targets: observer **23/23**, numeric **8/8**,
  zero failed/ignored/filtered.
- Full root `infer` target for F2: **299/299**, zero failed/ignored/filtered.
  The accepted reporting fix remains closed under this fresh unfiltered target.
- Canonical replay and independent controller audit: **26 expected outcomes**.
  Frozen candidate cold/warm **6/0**, exactly executable `{u64}`, no local proof
  or inferred-helper bodies. Full canonical predicate and original domain retained.
- Observer replay and independent audit: **23 expected outcomes**. Prefix **2/0
  cold/warm, 1/1 off**; full XOR-prefix plus untouched-suffix HMAC **1/1
  cold/warm**, still an authored postcondition failure. Plain **2/0 both ways**.
  Mode-invalid/ghost-write frontends are explicitly excluded from semantic negatives.
- **71 frozen sources / 142 off/on invocations**: 45 byte-identical earlier
  cases, 26 new cases. Every application/probe invocation has `--no-cheating`
  and normal body, lifetime, trait and mode checks. No narrowing verification flags.

This review does **not** run the other 29 root Infer targets or claim a complete
32-target/621-test result. The parent's separate full suite was not inspected.
The old d35c31fa 618/0 and Python counts are historical, not new validation here.

## Adversarial results and evidence limits

All three actual R1/R2 sources pass ordinary verification with assistance off/on,
with exactly the expected executable bodies and empty proof/inferred-helper
inventories. Reassigned keys no longer acquire generated observations, and Exec
key producers no longer reach annotation-purity synthesis. Earlier declaration,
match-alias and unrelated-constructor repros remain fixed. Qualified actual native
calls retain selection; qualified key-producer lookalikes skip cleanly.

New probes cover immutable locals, same-name immutable/mutable shadows, a shadow
introduced after iterator construction, blocks capturing local keys, shared-reference,
tuple and struct projections, conditional/match keys, mutable parameters before and
during iteration, reassignment through mutable aliases and through `&mut &slice`,
and side effects in blocks/helpers. All valid trivial writers verify off/on.
The readonly bare-parameter prefix retains **2/0 on, 1/1 off**. Its unrelated-buffer
writes also preserve useful assistance. Numeric/prefix composition retains **8/0**
with exactly `{u64, xor_key}` and empty proof/helper bodies; off **3/3** is a mixed
diagnostic, not a strict semantic negative.

No valid off-positive/on-negative regression was observed. Of 142 invocations,
119 succeed, ten fail postconditions, two fail safety checks, two fail assertions,
one composition-off run has invariant/mixed diagnostics, and eight are frontend
outcomes. The eight frontend outcomes are four paired invalid inputs: the old
non-shared-lifetime mutable parameter, overlapping mutable borrows, attempted
mutation of an immutable parameter, and an additional lifetime-invalid reborrow.
These are excluded from semantic-negative claims. A source can select assistance
and still fail ordinary Rust checking; no checked-acceptance claim is made for it.
No resource failure, panic or timeout was counted as proof evidence.

The original probe controller finished all 142 commands and then exited **1** on
a stale hypothesis that the old `{ key }` prefix temporary would remain selected
and prove. Its actual outcome is **1/1 off and on**, only the authored prefix
postcondition, no observer on. This is exactly the intended unsupported-shape
boundary, not a valid-program ordinary-acceptance regression. `adjudicate.py`
preserves the original results/hypotheses and requires the exact two objections,
all original input guards, all command counts, log hashes and every other frozen
expectation. No application proof was rerun or edited. `finish_replay.sh` continues
with the untouched bundle after that preserved controller failure; no successful
exit of the original wrapper is claimed.

## Source, logic and trust review

The 18-line eligibility guard runs before proposal construction. It accepts only
`ExprX::Var` or `ReadPlace(Local)` with the exact `VarIdent` of an immutable Exec
parameter and an exactly matching shared-byte-slice type. `VarIdent` includes its
Rustc disambiguator (`ast.rs` and `rust_to_vir_base.rs`), while `user_mut` is derived
from the actual parameter binding annotation (`rust_to_vir_expr.rs` and
`rust_to_vir_func.rs`). Same-spelling local shadows cannot satisfy that identity.
An immutable shared `[u8]` binding cannot be changed through a legal mutable alias;
its byte element type has no interior mutability. No key-length bound is added.
The restriction deliberately makes no evaluated-value capture, alias-stability
analysis or assumed iterator/key equation.

The descriptor's actual init/IntoIterator/wrapper/match source chain remains
unchanged. `entry_observations_compatible` still excludes incompatible mutable
uses in statements and places, including index and temporary effects. Both
decorators retain independent whole-Function projection, exact scaffold checking,
fresh expression/place/read/binder identities and cross-pass reservation. The VIR
suite includes the projection, receipt corruption and collision mutation controls.
`query.rs` still uses full function WF rechecks for `body_rechecks`, full mode
rechecks, refreshed resolution maps and final merged-crate checking. Normal Rust
lifetime/trait checks and ordinary SMT body obligations remain enabled.

No executable, contract, semantic invariant, predicate, vstd model or trust fact
was added or changed by the fix. Generated formulas remain checked proposals.
Policy **68**, candidate-cache **71** and metadata schema **38** are unchanged.
Canonical healthy-helper controls bind soundness/maximality/exactness before the
property gate, and cache/import/export poison controls retain their required
rejections. The additional audit reads the actual provider `.vir` trailer and
`.rlib`, all 359 trusted-model bindings and `identity` proof statuses, and checks
the consumer fingerprint against that provider. These are honest compiler-artifact
checks, not authentication against adversarially forged binaries/artifacts.

## Reproduction and binding

From the checkout root, with `/home/dev/.cargo/bin` first in PATH:

```sh
python3 -B -O evidence/stable-key-review/validate.py --build
python3 -B -O evidence/stable-key-review/validate.py --build-vstd
python3 -B -O evidence/stable-key-review/validate.py
bash evidence/stable-key-review/replay.sh
# The preserved stale-shape hypothesis ends the original wrapper after probes.
python3 -B -O evidence/stable-key-review/adjudicate.py
bash evidence/stable-key-review/finish_replay.sh
python3 -B -O evidence/stable-key-review/collect.py
```

For a new replay use a fresh private raw path consistently; immutable bundle
creation deliberately refuses overwrite. `freeze_probes.py` generated the
committed and independent private source copies before any invocation.
`source-freeze.json` binds 2,099 tracked source/dependency/toolchain inputs before
building, then rechecks them after validation. The ordinary vstd-generated lock
is copied as `vstd-build.Cargo.lock`. ArtifactGuard protects the copied runtime,
macros, solver, support tools, controllers and frozen probe inputs during execution.
The manifest binds full hashes, exact argv/cwd/environment, per-command statuses,
per-target counts, complete selected body inventories, raw outputs, independent
audits and compact evidence. No tracked production/toolchain source drift is accepted.

## Goal boundary

Canonical remains every u64 and arbitrary initialized exclusive `[u8;10]`, with
no application-local mechanical proof, helper call or snapshot in the frozen
accepted candidate. Its authored semantic definitions, prefix invariants and exit
clauses remain. Full HMAC remains the original bitcoin_hashes 0.14.0 XOR-prefix
plus untouched-suffix property, arbitrary initialized `[u8;128]` and **all valid
immutable key lengths**, including 0 and greater than 128. Prefix success does
not prove the suffix or full transfer.

The accepted ablation **50debf660be6a9aa73d6a7e6143bac1c01f4ae1a** remains separate:
observer redundant for canonical, necessary as a group for prefix, capacity
redundant for both. Necessary shared proof work is not confirmed. Model 18+6
results are separate previously checked theorems, not rebuilt here. There remain
**zero actual native correspondences, all 101 effect obligations open and native
publication false**; the inspected publication barrier still unconditionally
rejects a claimed native proof. The private semantic-controller work and baseline
were not tested. Ordinary vstd's inherited 83 admissions remain within the stated
trust boundary; no new app/probe admission is used. **The full goal is incomplete.**
