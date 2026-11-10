# Fix-review findings

## R1 — P2: F1 remains open for declaration and match-pattern aliases

Location: `source/vir/src/infer/buffer_observation_goals.rs:667-688`
(`entry_observations_compatible`). Reviewed production engine ea897a9b, runtime
freshly built at c9d072ad with identical production bytes.

The guard recognizes only expression-level assignments, borrows and mutable-ref
`ReadPlace` moves. VIR also represents alias acquisition directly as a **place**
in `StmtX::Decl.init` and `ExprX::Match` scrutinees. Visiting these places does not
call the expression matcher on a `ReadPlace`; later writes are rooted in the new
alias local, so `may_reach` returns false. Pattern `ref mut` has the same gap.

Four frozen ordinary programs independently verify **2/0 with assistance off**,
but select `client` for observation assistance and fail **1/1 with assistance on**,
solely `invariant not satisfied before loop`:

- `move-let-after.rs`: after the native loop, `let alias = buf; alias[0] = 42;`.
- `pattern-ref-before.rs`: before the loop, `let ref mut alias = *buf; alias[0] = 42;`.
- `pattern-ref-after.rs`: the same mutable pattern and write after the loop.
- `match-move-after.rs`: after the loop, `match buf { alias => { alias[0] = 42; } }`.

The sources have no preconditions, authored invariants, helper bodies or reduced
input domain. Both modes use normal complete checks and `--no-cheating`. Off runs
check exactly the named executable `client`; on runs collect the same whole body
but fail its generated entry invariant. These are valid-program rejections, not
false-property acceptance or semantic-negative evidence. They are not intentional
unsupported-case skips: the observer actually applies its incompatible snapshots.

The originals before/after direct-write repros now both verify **2/0 off and on**,
and the tested explicit helper/reborrow/escape cases correctly skip assistance.
That closes those manifestations, but does not close F1's alias requirement.

Suggested correction: cover mutable acquisition/movement in declaration and match
patterns (including `ref mut` from target-rooted places), or conservatively skip
those target uses. Preserve normal proof/lifetime/mode checks and unrelated-array
prefix assistance. Do not infer a native frame fact from this eligibility test.
No production correction was made in this review.

Reproduce from the checkout root after the fresh build described in `REVIEW.md`:

```sh
export VERUS_Z3_PATH=/tmp/shared-buffer-fix-review-c9d072ad/bundle/z3
/tmp/shared-buffer-fix-review-c9d072ad/bundle/verus evidence/shared-buffer-fix-review/repros/move-let-after.rs --crate-type=lib --no-cheating --output-json
/tmp/shared-buffer-fix-review-c9d072ad/bundle/verus evidence/shared-buffer-fix-review/repros/move-let-after.rs --crate-type=lib --no-cheating --output-json --infer-goals
# Repeat with the three other named frozen repros.
```

`manifest.json` binds the pretrial source freeze, tool bundle, raw outputs,
selection inventories and independent audit. Raw outputs are private in
`/tmp/shared-buffer-fix-review-c9d072ad/probes/`.

## F2 disposition

The test now expects the exact four public profiles. No disabled, skipped,
determinism or publication assertions were removed. Fresh full-target results
are recorded in `REVIEW.md` and `manifest.json`.
