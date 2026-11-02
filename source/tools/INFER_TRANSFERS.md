# Pinned external Infer transfers

`check_infer_transfers.py` is the common entry point for the source-connected
Fedimint deadline, redb page-number codec, redb page-size, and redb address-range
replays. It runs the scenario controllers sequentially against one selected compiler/library/solver bundle,
then independently checks their proof receipts. It never builds, fetches,
changes branches, or modifies application sources.

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
python3 -B source/tools/check_infer_transfers.py \
  --fedimint "$scratch/fedimint" \
  --redb "$scratch/redb" \
  --output "$scratch/replays"
```

The default tools are `source/target-verus/release/verus` and `source/z3`.
Override them with `--verus` and `--solver`. Select just one transfer with
`--scenario fedimint`, `--scenario redb`, `--scenario redb_page_size`, or
`--scenario redb_address_range`; only its checkout is then required. All three
redb slices use the same `--redb` clone.
`--timeout` bounds an individual verifier invocation in the scenario controller;
`--scenario-timeout` bounds the entire controller process, including its nested
proofs and native tests. The defaults are 180 and 1,800 seconds respectively.

Application HEADs must match the pins above, with no tracked or untracked changes.
There is no automatic update to newer upstream sources. Historical reports and
receipts remain separate from fresh proof results. New replays must not require
another developer's worktree or an old temporary receipt to reconstruct their
fixtures.

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

## Harness tests

The independent front-door tests need only Python and do not clone applications,
build Verus, or invoke a solver:

```sh
python3 -B -m unittest discover -s source/tools -p 'test_infer_transfers.py'
python3 -B -O -m unittest discover -s source/tools -p 'test_infer_transfers.py'
```

They cover false controller success, missing exact checks, malformed or duplicate
case records, source pins, disabled verification, frontend/timeout failures posed
as semantic negatives, escaping receipt paths, stale exports, and continued
execution of the other scenario after an ordinary failure. The existing
`infer-tooling` CI test discovery includes these tests; the scenario controllers
also have their own fixture and evidence validation tests.
