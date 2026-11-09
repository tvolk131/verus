# Pinned external Infer transfers

`check_infer_transfers.py` is the common entry point for the source-connected
Fedimint deadline, redb page-number codec, redb page-size, redb address-range,
redb page-relations, and complete Fedimint request replays. It runs the six
controllers sequentially against one selected compiler/library/solver bundle,
then independently checks their proof receipts. It never builds the compiler,
fetches, changes branches, or modifies application sources. The complete-request
runner builds its small real Bitcoin dependency **offline** in fresh evidence;
no existing target is reused or copied.

## Prepare and run

Use a compiler and verified vstd built from the current `verus-infer`-based source
snapshot, following the repository build instructions. Run Cargo from `source/`
so its `.cargo/config.toml` applies. A compiler from a different source snapshot
is not a valid substitute, even if `--version` prints the same base revision.

Keep application clones and raw evidence in persistent scratch, outside the
Verus checkout. Either supply existing clean pinned clones or prepare fresh ones:

```sh
scratch="$(mktemp -d "${TMPDIR:-/tmp}/infer-transfers.XXXXXX")"
git clone https://github.com/fedimint/fedimint.git "$scratch/fedimint"
git -C "$scratch/fedimint" checkout --detach b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236
git clone https://github.com/cberner/redb.git "$scratch/redb"
git -C "$scratch/redb" checkout --detach c82ab865bb362bc4af067d332adc44978ff64813
python3 -B source/tools/prepare_infer_dependencies.py \
  --application "$scratch/fedimint" \
  --destination "$scratch/dependencies" \
  --output "$scratch/dependency-preparation"
python3 -B source/tools/check_infer_transfers.py \
  --fedimint "$scratch/fedimint" \
  --redb "$scratch/redb" \
  --dependency-home "$scratch/dependencies/cargo-home" \
  --output "$scratch/replays"
```

The default tools are `source/target-verus/release/verus` and `source/z3`.
Override them with `--verus` and `--solver`. Select just one transfer with
`--scenario fedimint`, `--scenario redb`, `--scenario redb_page_size`, or
`--scenario redb_address_range`, `--scenario redb_page_relations`, or
`--scenario fedimint-full-request`; only its checkout is then required. All four
redb slices use the same `--redb` clone. The full-request ID is hyphenated in
both the standalone and common receipts.
`--timeout` bounds an individual verifier invocation in the scenario controller;
`--scenario-timeout` bounds the entire controller process, including its nested
proofs and native tests. The defaults are 180 and 1,800 seconds respectively.

Application HEADs must match the pins above, with no tracked or untracked changes.
There is no automatic update to newer upstream sources. Historical reports and
receipts remain separate from fresh proof results. New replays must not require
another developer's worktree or an old temporary receipt to reconstruct their
fixtures.

The additional readonly redb access-prefix replay currently has its own entry
point; it is not one of the six registered common-frontdoor/CI scenarios:

```sh
python3 -B source/tools/check_redb_access_prefix.py \
  --application "$scratch/redb" \
  --verus source/target-verus/release/verus \
  --solver source/z3 \
  --output "$scratch/access-prefix"
```

This extraction preserves the original conversion syntax and helper bodies but
uses a three-field readonly configuration projection, appends a length return
and stops before `storage.read`. It proves range/length consistency over the
checked arithmetic and pointer-size representability domain, not storage or
whole-`get_page` correctness. Portable native/no-cheating results are distinct
from explicitly configured 32/64-bit checks that skip native erasure checking.

## Acceptance and evidence

The redb codec and page-size positive replays use `--infer-goals` without
handwritten packing, byte-inverse, shift-bound, or nonlinear-product proof
blocks. The original codec fixture remains byte-identical at SHA-256
`639d1032cc0fb061cf3e4bf7c8b05b14b13e7d856ce047b09e1f740d6565a3cb`.
The original executable bodies, canonical domains and authored goals remain.
Imported callers and codec injectivity use the same unaided assistance.

Assistance expands only bounded pure published formulas through transparent
record constructors, constant in-bounds literal-array projections and finite
scalar operations. Relational caller premises remain premises of an
unconditional bit-vector theorem check; the ordinary query receives exactly
that checked theorem and still proves the original obligation. Limits are
8 accepted auxiliary requests, 8,192 nodes per expression, depth 64, 8,192
statement visits and 131,072 shared conversion/expansion work units per ordinary
function. Unsupported expressions and exhausted budgets do not install facts.
No application identity, axiom or trusted application summary is introduced.

Historical assisted fixtures and semantic/safety negative controls remain
unchanged and run without this optional assistance. A false auxiliary theorem
must stop verification before its assumption is used; that rejection is not
evidence of an authored postcondition failure. The codec replay therefore also
records a separate unaided decoder-mutant rejection, while the frozen semantic
decoder control still fails its authored postcondition with fresh exactness
checks and a seeded previously verified interface removed. The page-size safety
controls retain eager invalid shifts (including zero sizes) and overflow, while
unaided positive callers include order 63 with sizes zero and one.

The final stdout line identifies a fresh `transfer-suite-*` output directory.
It contains an atomic `receipt.json`, command logs, source and tool fingerprints,
and nested per-scenario evidence. Keep that directory when sharing a result.

A passing combined receipt requires all selected transfers to report `proved`,
source fidelity, explicit trust and limitation inventories, positive proof cases,
semantic negative controls, and seeded stale-interface rejection. The suite
rechecks the selected raw verifier JSON, the three mandatory exact inference
obligations, full-crate checking of each extracted proof fixture, positive export
presence, negative export absence, and authored postcondition failure rather
than a frontend error, panic, timeout, or exhausted resource limit. Native tests
alone are not formal proof evidence. Arithmetic-safety controls are listed
separately as `safety_negative_cases`: they must fail their actual precondition
obligation, not an authored semantic postcondition. They cannot substitute for
the required semantic negative, and mixed failure kinds are rejected.

The `redb_address_range` slice retains the original `PageNumber` fields and both
`page_size_bytes` and `address_range` executable bodies, including `start..end`
and `debug_assert!`. It checks the full inferred shift/product/addition domain,
not a blanket order-at-most-20 approximation. Ordinary generic callers prove
range length and containment under explicit sufficient arithmetic conditions
without solver ghost blocks or optional assistance; constant boundary callers
use `--infer-goals`. End containment does not follow from the debug guard alone:
the original method returns `4090..8186` for size 4096, regional start 4090,
and region size 4096. This is a verified witness, not a rewritten tuple body.

Both `-C debug-assertions=yes` and `no` are verified with cfg witnesses. Only the
former includes the debug guard in its maximal safe domain; the latter accepts
the guard-violation and empty-region zero-length cases. Native debug/release
executions are separately labeled finite supporting oracles. The runner checks
seeded export removal, imported exact summaries, three poisoned metadata cases,
and regenerated candidates despite cache poison. It does not claim record cache
hits. A separately classified `auxiliary_negative_cases` control must fail its
checked bit-vector theorem, not a caller precondition or semantic postcondition;
auxiliary rejection cannot replace either required application control.

### Relational pages and the complete request

`redb_page_relations` preserves both original methods, `PageNumber`, `Range<u64>`
and the active debug guard. Common region/order/configuration, positive u32 page
size and the complete checked intermediate domain imply strict start ordering,
injective starts and ordered endpoint nonoverlap. There is no order-20
restriction. Zero-size ranges can share starts; distinct mixed-order pages can
overlap (`0..8192` and `4096..8192`). This is not an allocator-state invariant.
The controller independently requires eight positives, three genuine authored
semantic negatives, eleven caller-safety negatives, one NIA auxiliary negative,
and four rejected metadata imports, with distinct labels and absent seeded
exports. The polynomial failure must contain
`Verus Infer: polynomial proposal not satisfied`, full verification, positive
errors and all exact helper checks, without ordinary pre/postcondition errors.
Its coarse `caller_proof_failed` status does **not** make it a semantic
counterexample. Existing BV auxiliary classification remains unchanged.

Independent relational acceptance also pins both inferred preconditions and
equations (including the imported relay), exact per-case verified/error counts,
and the complete named ordinary-body inventory. Every body must be selected,
not automatically derived, and certified only when the full run succeeds;
missing, skipped or external bodies cannot satisfy the scope check. The native
provider rlib and inference interface are guarded together before and after
all imported proofs, and client dependency identities must match the guarded
provider certificate. Both helper certificates require all three exact checks.
The relay's own dependency must name the certificate's address-range helper,
use its provider-reported candidate identity, and retain exact mode.
All five pin/cleanliness/source-enumeration prerequisites must exist and exit
zero. Their command arguments and working directories are checked, before/after
pin and cleanliness output must agree with the pinned clean application, and
source-enumeration output must exactly cover the source ledger without duplicates.
Declared proof negatives are the only permitted nonzero commands. Regression
tests distinguish certificate failures from later fingerprint failures, exercise
integer off-by-one counts and extra guard paths, and check provider-guard ordering
around all imported commands and final publication. These
are evidence-consistency checks, not signatures authenticating arbitrary
user-edited reports or additional application theorems.

`fedimint-full-request` keeps all seven request fields, the literal tuple
constant and both complete original native helper items. Only adjacent derives
are omitted; Infer attributes and the Verus wrapper are outside those items.
The real pinned Bitcoin 0.32.8 / bitcoin_hashes 0.14.0 type is used, not a stub.
An explicit audited opaque external-type identity correspondence transports the
unused immutable `sha256::Hash` field; it asserts no hash behavior, layout,
bytes, constructors, trait laws or invariants. Default Bitcoin features plus
serde are **not** full application feature equivalence.

Independent acceptance rechecks application/source/item/lock identities,
all 21 registry packages and checksums, exact archive/source bytes, dependency
graph edges and artifact guards. Providers must verify 11/0 cold and warm,
with the original unconditional two-helper equations and all six exact checks;
both record summaries honestly miss and regenerate in both runs. Import-only
clients verify 4/0, with zero local inferred functions and nonempty dependencies
matching the guarded provider interface certificate and rlib. The original guard
deletion and false-policy controls must be 10/1 authored `deadline_policy`
postcondition failures, not frontend, safety or auxiliary failures. Overflow
(11/1, `possible arithmetic underflow/overflow`) and panic (11/1, requires-false
precondition) remain separate safety evidence. Nine current frontend/profile and
six exact corrupted-import controls require real nonzero exits, the correct
diagnostic, a seed bound to the successful export, and its subsequent absence.
The ordinary common frontdoor does not request the optional two baseline69
frontend comparisons; if supplied, those comparisons are independently checked
and reported separately, never promoted into current-tool proof outcomes.

These checks do not trust `success`, `source_fidelity` or stale-export flags
alone. Missing, duplicate, tampered, partial, skipped, resource-error or
malformed-boolean evidence fails closed, including under optimized Python.
The inspected funding caller remains **blocked** at its first async persistent
contract lookup. Its source wiring uses `incoming_amount_msat`, not sender-written
`amount_msat`; that inspection proves neither amount/origin authenticity, fees,
stateful fund safety nor whole-gateway behavior.

`success: true` in an individual experiment receipt only says its controller
completed its expected checks. An honestly reported `blocked` application is
retained as such and makes the combined command fail. All requested scenarios
are attempted after ordinary scenario failures; interrupts and changed shared
tool/source inputs abort the suite. Setup and execution failures still leave a
combined receipt. The `application_proofs_proved` field is separate from process
completion so a successful unsupported-syntax experiment cannot become a green
application regression.

The scope is the pinned extracted decision, codec, and arithmetic-query bodies and their authored
properties. It is not whole-gateway fund safety, database correctness, concurrent
execution, crash recovery, cryptographic correctness, or verification of Rust
standard-library implementation bodies. Read each scenario's trust inventory.

## Nightly and manual CI proofs

`.github/workflows/infer-transfers.yml` builds a matching compiler, macro set
and verified vstd, prepares fresh application checkouts, and invokes this common
frontdoor for all registered scenarios. It is separate from the fast harness
unit-test job: a preparation receipt or passing Python controller tests cannot
make an application proof green.

The nightly schedule is 04:23 UTC and explicitly checks out `verus-infer`, not
the default branch. Manual dispatch accepts an explicit Verus ref and defaults
to `verus-infer`. Scheduled workflows must first be published on the repository's
default branch to become active; manual availability also depends on publication.
Adding this workflow locally does not dispatch a hosted run.
The default branch is `main`; this integration neither changes `main` nor claims
that the nightly or a fresh hosted Linux manual dispatch has actually run.

`prepare_infer_transfers.py` takes the exact application pins from `SCENARIOS`
in `check_infer_transfers.py`; they are not duplicated in YAML. It shallow-fetches
only those commits, verifies detached HEAD and clean status, requires disjoint
application/evidence paths, refuses either destination if it already exists or
is a symlink, and retains command/failure receipts once fresh evidence storage
has been established. Rejected repeat invocations leave old receipts and logs
untouched. Git context variables and global/system Git configuration are not
inherited; the actual repository root is checked immediately after initialization
and again after checkout. It does not build
or prove the applications, and its receipt explicitly leaves
`application_proofs_proved` false. Example preparation:

```sh
python3 -B source/tools/prepare_infer_transfers.py \
  --destination /private/tmp/fresh-infer-applications \
  --output /private/tmp/infer-preparation-evidence
```

`prepare_infer_dependencies.py` is a separate explicit network/setup boundary.
It verifies the real Fedimint repository root, clean pin and application lock,
then uses the reviewed minimal lock (whose 21 identities/checksums are checked
against that application lock) for `cargo fetch --locked` in a fresh isolated
Cargo home. Cargo may query registry index metadata; only the exact pinned
archives/sources are accepted. An offline locked metadata pass independently
checks graph edges, checksums and extracted bytes before proof. No compiler,
application build or dependency substitution occurs in preparation. Git context
and Cargo/Rust compiler-wrapper overrides are sanitized; no ambient environment
or credentials are dumped. Destinations must be fresh, nonsymlinked and disjoint
from application/evidence paths; repeated calls cannot overwrite receipts.
Proof commands stay offline, and their native dependency target is fresh inside
each full-request replay. For a local, already-populated real cache, optional
`--cache-home /path/to/cargo-home` copies only the exact 21 pinned archives and
their sparse index entries into the fresh home and makes setup itself offline.
It never copies targets. The hosted workflow uses explicit fetching, not this
developer-cache option or any developer temporary path.

The workflow has read-only repository permissions, does not persist checkout
credentials, and does not trigger on pull requests. It bounds setup commands,
individual verifier calls, scenario controllers and the overall job. All ten
steps have deadlines: active-work step budgets total 80 minutes, upload has
five minutes, and the 90-minute job leaves another five minutes of margin.
Solver setup and compiler/vstd commands also use GNU `timeout` on the Linux
runner. These limits preserve time for upload after a failed or timed-out step;
external cancellation or runner loss can still prevent upload. Artifacts
retain the exact Verus revision, compiler/vstd/solver logs, preparation receipts
and raw application proof receipts for 14 days, including on failures. The
existing semantic/safety/auxiliary distinctions, source guards, mandatory exact
checks and publication barriers remain the acceptance criteria.
The new dependency-fetch step has five minutes (a four-minute process bound,
15-second cleanup and a 180-second per-command bound). The six-scenario proof
step is reduced from 35 to 30 minutes: six 270-second controller bounds consume
27 minutes, leaving three minutes for shared guards, cleanup and receipt checks.
Individual verifier calls retain their 180-second bound. A timeout is a failed
run, not evidence of an application bug or a reason to accept incomplete proof.

## Additional bounded mechanical pilots

Four standalone runners extend the application-led experiments beyond the six
common scenarios. They are not yet additional hosted CI scenarios:

```sh
python3 -B source/tools/check_redb_bitmap_range.py \
  --application /path/to/pinned-redb --verus "$VERUS" \
  --solver "$VERUS_Z3_PATH" --output /tmp/fresh-redb-bitmap
python3 -B source/tools/check_fedimint_fee_encoding.py \
  --application /path/to/pinned-fedimint --verus "$VERUS" \
  --solver "$VERUS_Z3_PATH" --output /tmp/fresh-fedimint-fee
python3 -B source/tools/check_redb_bitmap_state.py \
  --application /path/to/pinned-redb --verus "$VERUS" \
  --solver "$VERUS_Z3_PATH" --compiler "$(rustup which rustc)" \
  --output /tmp/fresh-redb-bitmap-state
python3 -B source/tools/check_fedimint_paymentfee76.py \
  --application /path/to/pinned-fedimint --verus "$VERUS" \
  --baseline-verus "$BASELINE_VERUS" --solver "$VERUS_Z3_PATH" \
  --output /tmp/fresh-fedimint-paymentfee
```

Use the same clean application pins as the common suite, a matching
cache77/policy71/schema38 runtime, and fresh private outputs. The new fee-arithmetic
runner also requires the sealed cycle75 baseline verifier to reproduce its
original blocker; it is not the verifier used for the successful new proofs.
Use a Rust toolchain matching the sealed runtime. The bitmap output
must not already exist. The bitmap runner proves unchanged `bits_in_range`
membership in debug/release domains and an authored by-value set/clear observer
with preservation outside the interval. It does not prove `resize`, bitmap
representation, allocation or storage. Computed-shift proof proposals are
checked; they do not assume the original shift's safety.

The fee runner preserves both original scalar functions, proving full-u64
round trips, order reversal, endpoints and the exact future-expiration
condition. Existing Infer annotations suffice: no fee-specific engine model
or native application edit is added. The separate fee-arithmetic runner now
proves the unchanged native calculation using the actual nested Amount/PaymentFee
representations and the new unsigned `saturating_div` model. It establishes the
exact saturation-before-division equation, conditional unsaturated equations,
monotonicity, amount bounds and endpoints. The native/std correspondence is
trusted; application arithmetic goals are authored and checked. Async funding
still needs authored state, height, operation and authenticity contracts.
Neither fee runner proves fund safety.
The fee replay clears working outputs and archives fresh phase-specific exports, binds clients to
the warm certificate, and checks cache identity from the emitted entries.
Its guard includes the compiler's shared libraries and standard-library rlibs,
separately from the sealed Verus/solver package. Harness tests exercise both
individual rejection gates and their replay wiring without invoking a compiler.
See `evidence/transfer-redb-bitmap-range75/README.md` and
`evidence/transfer-fedimint-fee-encoding/README.md` for source fidelity, trust
and the distinction between generated arithmetic and authored intent.

The bitmap-state runner verifies the original extracted `get`, `set` and `clear`
bodies against an authored logical bit view and representation condition. It
preserves every other live bit, logical length and vector length, and describes
the exact stored-word update. Set's return value refers to the full selected
stored word, including padding. The tuple helper contract and two checked
bit-vector lemmas are authored; only scalar mask plumbing is inferred. No new
mutable Infer profile, constructor, resize, allocator or storage proof is claimed.
See `evidence/transfer-redb-bitmap-state77/README.md` and
`evidence/transfer-fedimint-paymentfee76/REPORT.md` for the original candidate
evidence; the bitmap candidate's recorded75/69 identity is historical.

Live cache, certificate and numeric receipt expectations share
`infer_transfer_identity.py`. Its tests bind all three identities to the Rust
engine declarations, so an engine update cannot leave independent reader
constants silently stale. This does not make historical receipts acceptable
under a different engine identity or replace artifact/source guards. These
standalone pilots have not been added to the six-scenario hosted workflow.
The encoding runner's historical unsupported-division probe is now an exact
private-helper positive: its nineteen cases have five positives and fourteen
seeded failures. The fixed native denominator's `1000000 != 0` domain is true
for all inputs; arbitrary narrowed domains and a false public-export claim are
rejected. The old cycle75 receipt and its fifteen-failure count remain historical.

## Native byte-prefix inference

`check_fedimint_prefix_infer79.py` is a dedicated source-connected replay for
the same pinned Fedimint database helpers, separate from the six-scenario CI
suite and the earlier ordinary-contract prefix bridge:

```sh
python3 -B source/tools/check_fedimint_prefix_infer79.py \
  --application /path/to/pinned-fedimint --verus "$VERUS" \
  --solver "$VERUS_Z3_PATH" --sysroot "$SYSROOT" \
  --baseline-verus "$CYCLE78_VERUS" --output /tmp/fresh-prefix-inference
```

Use the matched cache78/policy72/schema38 runtime and Rust toolchain, with a
fresh output directory. The optional baseline verifier adds the thirtieth case:
it declines the same native selection at `extend_from_slice`. Without that
option, the replay has 29 cases. The three native methods retain their original
private visibility, generic domains, field declarations and item bytes; only
Infer attributes are added. Exact contracts give `requires true` and complete
prefix-plus-suffix sequence equations, including both range endpoints. Two
explicitly auxiliary public inferred wrappers enable imported composition
without widening native visibility.

Only local `Vec<u8, Global>` appends with inert immutable slice arguments and
the concrete byte-range profiles are added. Existing vstd models justify the
proposals; there is no new application axiom or behavioral library spec.
Cold and warm owned candidates regenerate and perform fresh mandatory checks,
not cache hits. Authored cancellation and namespace-separation lemmas remain
explicit: separation requires prefix incomparability, not a fabricated
fixed-width BigSize encoding. No backend range semantics, transaction-wide
confinement, async, crash durability, consensus or funds theorem follows.

Historical prefix-bridge evidence remains bound to its original runtime; it is
not relabeled as new-engine evidence. See
`evidence/transfer-fedimint-prefix-infer79/README.md` for the implementation
handoff and the cycle79 acceptance record for reconciled compatibility results.

## Bounded two-project composed mechanics

The frozen goal in `evidence/two-project-mechanical-automation/GOAL.md` adds two
dedicated targets, not two more registered common-frontdoor/hosted-CI scenarios.
The common six-scenario suite remains a compatibility regression and does not
stand in for either new target's complete matrix.

Fedimint's dedicated replay uses the matched compiler, macros and vstd directory:

```sh
python3 -B source/tools/check_fedimint_validation81.py \
  --application /path/to/pinned-fedimint --runtime /path/to/matched-runtime \
  --sysroot /path/to/compatible-rust-toolchain --output /tmp/fresh-validation
```

The historical `81` filename identifies the target, not permission to consume
stale certificates: the live reader obtains the cache/policy/schema identity
from `infer_transfer_identity.py`. All 46 cases, exact checked native/body sets,
actual imported consumers, both error flags, fresh mandatory work and failed
export cleanup must pass. The native checked-add/is_within helpers have no
handwritten contracts, and overflow remains a safe `None` result over the full
original domains. Fee decision order and the checked component-bound theorem
remain authored intent. The pure caller adapter is explicitly not the native
async caller.

redb's source fixture and exact whole-item/block/domain ledger are
`evidence/redb-native-composition-cycle1/{candidate.rs,scope.json}`.
`check_redb_native_composition.py` checks source correspondence only; it is NOT
the 81-case proof runner or an independent proof certificate. The delivery's
replay scripts execute the complete debug-on/off matrix with the selected
matching runtime and its nine-file hash ledger. The final delivery records their
commands and reconstruction separately. Both decoded little-endian IDs flow
through the original selection policy and final clean/error result, including
longer safe slices and invalid-index early-return branches. Three exact native
contiguous blocks remain explicitly conditional on authored context inputs;
neither whole decoder nor whole finalize is claimed verified.

Generic Rust regressions additionally exercise byte/record composition and real
certified imports. Existing tooling CI discovery includes the new Python guard
tests, but an ordinary local test pass is not a hosted Linux run or a manual
workflow dispatch. Current acceptance, remaining failures and trust boundaries
are recorded by the frozen goal rather than inferred from this command list.

## Harness tests

The separate resource-boundary engineering replay is documented in
[`VERUS_INFER_STATUS.md`](../../VERUS_INFER_STATUS.md). From the checkout root,
run `python3 -B source/tools/check_native_resource_boundaries.py --output NEW_DIRECTORY`
to build and test both feature profiles offline. This does not run application
proofs, extend the six-scenario suite or certify native ownership. Its Python
acceptance tests are included in normal tooling discovery.

The independent front-door tests need only Python and do not clone applications,
build Verus, or invoke a solver:

```sh
python3 -B -m unittest discover -s source/tools -p 'test_infer_transfers.py'
python3 -B -O -m unittest discover -s source/tools -p 'test_infer_transfers.py'
python3 -B -m unittest discover -s source/tools -p 'test_infer_transfer*.py'
python3 -B -O -m unittest discover -s source/tools -p 'test_infer_transfer*.py'
```

They cover false controller success, missing exact checks, malformed or duplicate
case records, source pins, disabled verification, frontend/timeout failures posed
as semantic negatives, escaping receipt paths, stale exports, and continued
execution of the other scenario after an ordinary failure. The existing
`infer-tooling` CI test discovery includes these tests; the scenario controllers
also have their own fixture and evidence validation tests.
