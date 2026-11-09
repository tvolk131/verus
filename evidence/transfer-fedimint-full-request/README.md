# Complete native Fedimint request: selected helper proof

This replaces the reduced-field diagnostic with a proof over the complete real
seven-field `InterceptPaymentRequest`. Historical reports and their runners are
preserved. The new standalone runner reconstructs the request, both native
helpers and their tuple constant directly from clean Fedimint pin
`b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`, and rebuilds the real Bitcoin
dependency offline. No historical temporary fixture or developer worktree is
needed to reconstruct its inputs.

## Baseline and implementation identity

The starting point is `55bd67ad980e90e92c77efe4453a927f097c3386` plus the full
validated 87-file dirty baseline. The supplied restore patch was applied with
`apply_patch`; all 87 file hashes and nine immutable tool hashes matched before
editing. Baseline manifest SHA256:
`1a52b38c47530e13bf056c0be1c4b0af5635e8236469ec4767f5c248ecee7894`.

HEAD and origin/verus-infer remain at that base. The differently positioned
local verus-infer ref was neither moved nor substituted. No manual branch,
commit, PR, subdelegate, parent-worktree edit, application-clone edit or
historical-evidence rewrite was made. There is no applicable Verus AGENTS.md.
The application instructions and three local codebase/development/style skills
were read; the referenced linked-specs skill is unavailable, and no governed
application source/spec is edited.

The new identity is **cache 71 / policy 65 / schema 38**. Production changes
are limited to the readonly record shape/access gates plus those identity
bumps. No new wire representation, vstd law, axiom or unchecked application
summary is added. See `TRUST.md` for the explicit new opaque correspondence and
its strict unused/readonly limits.

## Reproduced blocker and source fidelity

Frozen matching 69/63 tools reject the complete request's actual
`bitcoin_hashes::sha256::Hash` field. A minimal explicit opaque external-type
declaration reaches Infer but the baseline rejects its record shape. Both
failures are freshly reproduced, including a successful ordinary native
compilation with the actual dependency. No fake Hash type is used.

Bitcoin 0.32.8, bitcoin_hashes 0.14.0 and every one of the 21 selected registry
package identities/checksums match the application's lockfile. Each cached
`.crate` archive is hashed against that pin and every source file compared
against its archive before locked compilation. The minimal graph uses Bitcoin
default features plus serde; it does not establish full application feature
or full-crate equivalence.

All seven fields, including both independent u64 amount fields and
`short_channel_id: Option<u64>`, remain intact. Both full method items and
`pub const NO_INCOMING_CIRCUIT: (u64, u64) = (0, 0);` retain their original bytes.
There is no helper-body desugaring. Positive-fixture adaptations are individually
hashed: omit only the adjacent Debug/Serialize/Deserialize/Clone derive; add
Infer attributes outside method items; wrap the extracted items in `verus!`
with imports; add the explicit opaque type identity declaration. Declaration
visibility and all native field/input types remain unchanged in positives.
The invariant negative alone uses private fields inside a nested module to
reach the invariant gate legally, with the original method bodies preserved.

## Fresh outcomes

- Complete request providers: **11 verified / 0 errors**, cold and warm.
- Imported provider/client composition: **4 / 0**, cold and warm.
- Original authored intercepted-forward/unknown-height policy and two-block
  coverage, expiry boundary, height monotonicity and independence from all
  unused fields prove without any authored input precondition. Both inferred
  helper entry requirements are true over the full native scalar/Option domain.
- Source-preserved forward-guard deletion: **10 / 1**, solely the original
  intercepted-forward authored postcondition. Both helper equations remain
  exact; the seeded export is removed. False policy has the same separate
  semantic classification and 10/1 outcome.
- Real arithmetic-overflow and panic safety controls: **11 / 1** each, with
  both helpers still exact and seeded interfaces removed. These are safety
  failures, not semantic policy counterexamples.
- Malformed goal, missing/wrong real hash proxy, field access, alias, mutable
  access, trait and legal invariant controls reject at frontend/profile gates.
  Six corrupted-import metadata controls reject separately. No frontend,
  auxiliary failure, timeout or resource exhaustion is counted as a semantic
  negative.

The dedicated replay declares **25 cases**: 4 positives, 2 semantic negatives,
2 safety negatives and 17 frontend/import controls (including the two baseline
comparisons). Its six non-frontend local-inference runs perform **36 mandatory
helper checks**, all proved. Imported clients consume the provider certificate.
Cold and warm readonly record summaries regenerate and reprove; both honestly
report cache misses, never cache hits. Every negative seeds a successful
provider export and requires its removal. All application proof commands use
no-cheating and full extracted-crate verification.

## Actual funding caller: blocked, not promoted

The selected nearest caller is
`Gateway::try_handle_lightning_payment_lnv2` in the pinned gateway-server source.
It passes `htlc_request.incoming_amount_msat` to both persistent contract
matching and `relay_incoming_htlc`, never the sender-written `amount_msat`.
The matcher compares the persisted registered incoming amount with its supplied
amount. The relay gates only fresh `CreateReceiveAndCompletion` funding on the
two-block deadline and federation liveness; replay/resume plans differ.

The proposed authored local policy is: a newly initiated funding operation
along this caller has the helper's zero-circuit origin classification, known
height, at least two remaining blocks, and a registered incoming amount equal
to `request.incoming_amount_msat`. Authentic backend origin and an amount/fee
relation require separate source/state invariants; neither is inferred from
the old guard or a field name.

The first actual blocker is the caller's awaited persistent contract lookup,
keyed by `PaymentImage::Hash`, before the deadline helper. It depends on async
database state; hash value access also exceeds the unused opaque path. Later
RPC, operation-existence queries, cryptographic commitments, federation
liveness, transaction submission/failure and concurrency are further boundaries.
Exact caller/matcher/relay/plan source items and hashes are preserved here and
re-extracted by the runner. The caller status remains **blocked**. No caller,
funding, fee, amount-authenticity or stateful fund-safety proof/mutant is claimed.

## Matching build and regression validation

Matching release compiler/driver/builtin/vstd-build tools are built from this
checkout with Rust 1.99.1, offline/locked Cargo, source/.cargo settings, two Cargo
jobs and independent target/build directories. Only unchanged macro dylibs and
the solver are independently copied from the immutable baseline with explicit
hashes. The final vstd invocation directly uses RUST_MIN_STACK=20971520 and
verifies **2,221 / 0**; this avoids vstd_build's internal 10 MiB stack override.
Earlier development vstd runs are retained but not added to this fresh count.

Fresh validation passes **135 VIR library tests** and **577 integration tests
across all 28 infer binaries**, with no ignored tests, plus **155 Python tests
normally and 155 under -O**. One initial optimized execution of the unchanged
`test_artifact_mutation_aborts_before_next_command` expected an ArtifactChanged
error but observed its five-second TimeoutExpired instead; the isolated test
and the complete optimized rerun pass. That retained harness timing failure is
not application semantic evidence. The new offline suite contributes 11 Python tests;
the new Rust integration binary contributes 4 cases, including seven separate
fail-closed probes. Cargo VIR testing uses `--lib`, excluding the 18 known
unrelated doctest failures. Formatting and diff checks pass. All four original
transfer slices independently report proved against the frozen matching tools;
the shared controller/guide and separate parent-owned CI/preparation files are
not changed by this delta.

## Replay and handoff

```sh
python3 -B source/tools/check_fedimint_full_request.py \
  --application /path/to/clean/pinned/fedimint \
  --verus /path/to/matching/verus --solver /path/to/z3 \
  --output /private/tmp/full-request-replays
python3 -B -m unittest discover -s source/tools -p 'test_fedimint_full_request.py'
```

Optional `--baseline-verus` adds the frozen-baseline comparisons. Ordinary
replays need the exact registry archives/sources cached for offline Cargo.

Fresh full-request receipt: `/private/tmp/verus-fedimint-full-request-ggivj6y7/final-replays/full-request-702qwo9t/receipt.json`.
Fresh four-slice receipt: `/private/tmp/verus-fedimint-full-request-ggivj6y7/four-slices/transfer-suite-1bpndywv/receipt.json`.
Private build/source/Cargo fingerprint ledgers, exact command/log receipts,
frozen tools, replay-verified baseline-relative patch, changed-file hashes and
separate parent integration/guide proposal are under `/private/tmp/verus-fedimint-full-request-ggivj6y7`. The new scenario
is not registered by this child, and the blocked funding pilot is not a green
default. The parent owns independent review and publication.
