# AlephBFT inference experiments

These experiments distinguish checked proof reductions, unsupported source, and
concrete runtime counterexamples. They never edit the input AlephBFT checkout.
Build this Verus fork first; the release driver, vstd and Z3 must be available.
All commands use local committed Git objects and offline Cargo dependencies.

## Common-vote representation bridge

```sh
python3 source/tools/eval_verus_infer_alephbft.py \
  --alephbft /path/to/AlephBFT \
  --output /tmp/aleph-vote-experiment --native-tests
```

The runner clones pinned clean main at
`f35c7bba6d628bcafdf0d65dd471cc6f868ec034`, ignoring uncommitted changes.
It mechanically shares the original `common_vote` function with production,
checks executable-body identity, and compares manual, inferred, manually assisted
and `--infer-goals` callers. Canonical remainder-based schedule and alternation
properties already verify with inference. The equivalent low-bit parity
formulation requires two local lemma invocations or checked goal assistance.
This does not prove new distributed agreement or recovery properties.

## Quorum and parent-selection checkpoint

```sh
python3 source/tools/eval_verus_infer_alephbft_parents.py \
  --alephbft /path/to/AlephBFT \
  --verified-collector /path/to/AlephBFT/consensus/src/creation/collector/state.rs \
  --output /tmp/aleph-parent-experiment --native-tests
```

`--verified-collector` supplies the **existing manually verified production
collector**, including its contracts, counting definitions/lemmas, frame facts,
loop invariants and proof calls. It defaults to that path in the input working
tree. This file is not part of the pinned clean-main revision. The report records
its hash and source path, and guards it against changes during evaluation.

Native tests run with AlephBFT's pinned production Rust toolchain, after removing
an inherited `RUSTUP_TOOLCHAIN` override. `--native-target /path/to/target` can
reuse a compatible Cargo target directory. Positive native scenarios require
actual executed tests; compilation errors and zero-test runs are rejected.
Only the default-feature build is exercised here. The earlier common-vote runner
separately records the pinned no-default-features compilation failure.

### Sources and runtime changes

The isolated clean-main clone mechanically moves the public `NodeCount(usize)`
declaration and its `consensus_threshold` method into shared modules. Native
derives, native operators and the `NodeCount` API are preserved. Ghost mode
derives only Copy/Clone; it does not assume operator specifications.

The original calculation `(*self * 2) / 3 + NodeCount(1)` is used by the original
native probes. The corrected variant uses:

```rust
NodeCount((self.0 / 3) * 2 + ((self.0 % 3) * 2) / 3 + 1)
```

This is a **real runtime correction**, separately recorded from source splitting
and proof annotations. Inference cannot prove the original multiplication safe
for unrestricted inputs. The collector's round arithmetic is left unchanged in
the clean-main clone so its boundary failure remains observable.

The formal target imports the shared corrected production method and the
supplied manual collector core. A verified scalar adapter connects that core's
existing `super::consensus_threshold` dependency to the NodeCount API. Its
executable adapter contains only `NodeCount(n).consensus_threshold().0`; no
assumed specification or copied collector algorithm is used. This is not a formal
verification of clean main's generic `UnitsCollector<H>` or its whole crate.

### What the comparison establishes

- The manual and inferred variants prove the same five threshold properties:
  exact mathematical threshold, positivity, the committee-size bound for nonempty
  committees, strict two-thirds majority, and minimality.
- An unrestricted caller establishes these properties for every `usize` input.
  The inferred safety predicate is not necessarily printed as `true`; its
  arithmetic clauses are discharged by that caller from type bounds alone.
- The inferred variant replaces **one five-clause production threshold contract**.
  It also replaces one contract on the experiment's scalar adapter; that adapter
  contract is not counted as an eliminated preexisting AlephBFT annotation.
- All existing collector contracts, invariant definitions, induction proofs and
  loop annotations remain in the positive formal comparison. Removing the update
  hint or constructor invariant causes a proof failure. The constructor's
  decreasing measure is retained during invariant removal, so the failure reaches
  verification instead of merely failing the mandatory-termination frontend gate.
- Wrong threshold goals, changed threshold behavior, reintroduced multiplication
  overflow and corrupted collector accounting are semantic negative controls.
  They must leave both inferred helpers successfully checked while the ordinary
  caller/collector proof fails.
- Requesting inference on the generic mutable collector removes the update
  method's contract and ghost proof block, and is separately reported as
  unsupported. With the bounded collection models now available, its first
  observed blocker is the `u32::from(round)` dependency. Generic mutable record
  receivers and whole-method frame/count inference also remain unsupported.
  This is not classified as a program defect.
- Requesting inference on the actual pinned `Mul<usize>` implementation is also
  unsupported. Verus inherits operator preconditions from its trait/extension
  specifications; an implementation cannot introduce an inferred `requires`.
  No workaround changes the trait's meaning or bypasses that restriction.

### Runtime exploration and counterexamples

The deterministic native `Unit` adapter supplies creator, round and hash, exactly
the fields read by `add_unit`. It does not simulate authentication or a complete
DAG. The explorer executes the actual production collector with committee sizes
1 through 4, unit rounds 0 through 3, two payload/hash variants per coordinate,
and update/advance sequences of length at most four. It checks slot updates,
first-maximum tie handling, unchanged other slots, exact direct-parent accounting,
and successful selection iff a quorum and own direct parent are present.

The checkpoint observed 1,152 distinct states and 10,388 transitions in each of
the original and corrected runs. These are bounded tests, not a universal proof.
Four native source mutations must fail: duplicate counting, stale-parent
counting, accepting future slots, and accepting a missing own parent. The
15 existing production collector tests also pass after the threshold correction.
An isolated native Cargo target additionally imports the same shared threshold
method, the identical scalar adapter body, and the supplied manual collector;
all three of that collector core's existing tests pass.

Two existing boundary issues have concrete API-level reproducers:

- `NodeCount(usize::MAX / 2 + 1).consensus_threshold()` overflows at multiplication
  with overflow checking enabled. Corrected threshold tests include this value,
  `usize::MAX - 1` and `usize::MAX` against a `u128` mathematical oracle.
- Starting from a one-member collector, 65,534 actual round-advance calls reach
  round 65,535; the next call overflows. This records the API boundary and does
  not establish reachability through every surrounding service configuration.

The probe tests catch these expected panics to preserve concrete evidence.
Successful expected-counterexample tests do not mean the original functions are
safe at those inputs. No additional accounting defect was observed in the bounded
unmodified sequences. These findings reproduce known correction targets in the
existing manual verification work; they are not new distributed-protocol bugs.

### Reports and reproduction

Outputs include `results.json`, `summary.md`, raw verifier/native diagnostics,
per-case formal/native input snapshots, the patched clone, and a reviewable
`production-and-probes.patch`. The patch includes the native test instrumentation.
Compiler, solver, libraries, runner/support modules, fixtures and the supplied
collector are fingerprinted; rebuilding them during evaluation invalidates the
run. Cache entries contain candidates only, and the warm case repeats all three
required proofs. Timings are single local observations, not a benchmark.

To replay a mutated formal case, restore **all** files from its `inputs/<case>`
snapshot to the clone before running its recorded command. The clone is left at
the successful inferred variant. Native cases mutate the real collector before
each Cargo run; their mutations and commands are defined in the runner and raw
diagnostics retain the failing trace.

## Bounded collection views and state effects

The next checkpoint is implemented by
`source/tools/eval_verus_infer_alephbft_collections.py`. From the repository root:

```sh
python3 source/tools/eval_verus_infer_alephbft_collections.py \
  --alephbft /path/to/AlephBFT \
  --output /tmp/infer-aleph-collections \
  --native-tests --native-target /tmp/infer-aleph-collections-native
```

The input checkout remains read-only. In an isolated pinned clone, the runner
imports the existing manually verified working-tree `collector/state.rs`, uses
the corrected shared `NodeCount` threshold and its checked adapter, and extracts
only three operations from `Collector::add`: vector length, copy read and slot
replacement. Both native and ghost builds use the functions in
`collection_ops.rs`. This is the supplied manual collector core, not the generic
clean-main collector. The threshold runtime correction and widened collector
round representation retain their separate origins from the preceding experiment.

Length and read receive soundness, maximality and exactness proofs. Replacement
uses explicit sound mode and gets the complete sequence update relation, including
unchanged length/other slots. Its maximality and state exactness are explicitly
not established. The vector/sequence specifications already exist in vstd; this
experiment adds no assumed contracts or axioms. Collection candidates regenerate
and reprove on warm runs while the two scalar candidates are reused and rechecked.

The supplied counting invariant, recursive induction lemmas, contracts, loop
invariants and calls to `count_update`/`count_bound` remain explicit and checked.
Inference replaces **three experiment-introduced wrapper contracts** and removes
**zero existing collector proof annotations**. The original code used builtin
vector specifications directly; extracting these helpers is a capability and
composition experiment, not yet a net reduction in the original proof effort.
`--infer-goals` is also tested, but its scalar strategy does not select the
counting lemmas.

The checkpoint passes 11 formal scenarios: original/manual-wrapper/inferred/
warm/goal-assisted positives, proof rejections for wrong slots, rounds, counters
and a missing counting-update hint, and separately classified unsupported extra
effects/default-exact mutation. All helper obligations pass in the semantic
negative cases. Four native runs execute the supplied three collector tests on
both original and extracted implementations, then catch wrong-slot/wrong-round
mutations. These are selected tests and universal Verus checks of the supplied
contracts, not a new distributed-protocol theorem.

Outputs include raw diagnostics, `results.json`, `summary.md`, per-case formal
and native source snapshots/hashes, a reviewable `collection-extraction.patch`,
and the isolated clone restored to its passing inferred variant. Tools, model
artifacts, input collector, runner/support code and fixtures are fingerprinted;
changes during evaluation abort the run. Timings are not a benchmark.

The counting-assistance checkpoint below implements bounded selection of checked
sequence/counting lemmas. General mutable records, generic helper composition and
state-aware precision checking remain separate designs. Operator-trait inference
also needs a design through Verus's inherited/extension specifications.

## Supplied counting-invariant assistance

```sh
python3 source/tools/eval_verus_infer_alephbft_counting.py \
  --alephbft /path/to/AlephBFT \
  --output /tmp/infer-aleph-counting \
  --native-tests --native-target /tmp/infer-aleph-counting-native
```

This follows the collection experiment on the same supplied manually verified
collector. It keeps the state contracts, predicate and loop annotations, while
the count definition delegates to the reusable `Seq::count_prefix` recurrence.
An experiment-only copy of the original recursive count and a checked
`library_count_agrees` proof establish that the migration preserves meaning for
every sequence, round and valid prefix. The original definition remains available
as independent reference evidence, not an assumption.

The assisted variant removes:

- The counting proof block in `Collector::add` and its two lemma calls.
- The constructor and round-transition calls to `count_zero`.
- All three application-specific recursive counting proof definitions:
  `count_bound`, `count_update` and `count_zero`.

The existing sequence-equality assertion in `next`, both loops' annotations,
state contracts and invariant clauses remain supplied. The shared generic
library supplies four checked lemmas: count bounds, slot-update accounting, zero
counts and the implication from a full count to all slots matching. The full-count
fact lets a known nonmatching slot establish room for a counter increment.

`--infer-goals` follows supplied count contracts/assertions through bounded spec
definitions, then enables that fixed theorem catalog through scoped broadcasts.
Verus handles conditional theorem instantiation and checks every original
postcondition and safety assertion. This adds no axiom, external-body assumption,
generated executable operation or inferred protocol invariant. The three vector
helpers still receive their original inference checks; the state summary still
does not claim maximality/exactness.

Eleven formal scenarios compare the original manual proof, generic-library manual
proof, unassisted failure, assisted/warm success, wrong slot/round/counter,
missing accounting invariant, missing constructor invariant and a changed count
definition. The equivalence check must reject a changed counting predicate.
All five helper contracts must remain successfully checked in semantic controls.
The missing-loop-invariant case retains termination evidence, so it reaches a
proof failure instead of an unsupported-source diagnostic. Four native runs
execute the supplied three tests against original and assisted bodies and detect
wrong-slot/wrong-round mutations.

Outputs include `results.json`, `summary.md`, raw diagnostics, per-case formal and
native snapshots/hashes, `counting-assistance.patch` and the passing assisted
clone. Inputs, tools, model artifacts and runner/support files are fingerprinted.
Warm scalar candidates are reused and rechecked; collection templates regenerate
and recheck. Discovery and solver work are bounded; timings are not a benchmark.

This is a proof-effort reduction on the supplied manual core, not a proof of the
clean-main generic collector. Reusable library lemmas replace application
induction; arbitrary induction and invariant discovery remain unsupported.
Further work includes general record mutation, expected-domain audits, collection
loop annotations and trait/operator specifications.
