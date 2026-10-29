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
- A collector capability probe removes the update method's contract and ghost
  proof block, then deliberately adds a local mutable alias. This remains an
  unsupported source shape, not a program defect. The original direct method
  now supports default exact mode through the record experiment below, including
  its `u32::from(round)` dependency.
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
sequence/counting lemmas. Later checkpoints implement bounded record and generic
wrapper composition. Broader opaque mutation and standalone-vector state precision
remain open. Operator-trait inference
is implemented for concrete From/AddAssign in the wrapper trait checkpoint below;
other operator traits remain open.

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
The record checkpoint below adds bounded record composition and ordinary
whole-invariant-domain harnesses. Broader heap mutation, a dedicated domain
interface, collection-loop annotations and trait/operator specifications remain.

## Direct record inference and whole-valid-domain coverage

```sh
python3 source/tools/eval_verus_infer_alephbft_records.py \
  --alephbft /path/to/AlephBFT \
  --output /tmp/infer-aleph-records \
  --native-tests --native-target /tmp/infer-aleph-records-native
```

This checkpoint infers the supplied collector's **original executable `add`
body**, retaining its direct vector operations, `Option<(T, u16)>` match and
`u32::from(round)`. Its counting proof block is removed as before, and no vector
wrappers are extracted. The runner checks body identity against the original
after removing exactly that ghost block. The inferred exact contract relates all
three fields to their old values under the actual branches and includes the
bounds/overflow requirements. The handwritten contract is removed from `add`.

Two ordinary ghost-only caller checks make the result reviewable:

- `check_add_invariant` requires only `old(self).valid()` and ensures
  `final(self).valid()`, for arbitrary creator, payload and round. Its call must
  prove every synthesized safety requirement, establishing coverage of the whole
  supplied valid domain as well as invariant preservation.
- `check_add_behavior` retains the original `add` postconditions as an
  **experiment-only regression oracle**, checking slot behavior, unchanged other
  slots, unchanged round/length and monotone accounting. These clauses are not
  silently assumed by inference or counted as erased from the regression suite.

The supplied invariant, predicate, other methods' state contracts, loop
annotations, transition equality assertion and counting-library equivalence
remain. A minimal developer proof can use the first harness alone; the second
preserves the previous behavioral proof target. This remains the existing manual
working-tree collector core, not clean main's NodeMap/Unit/Hasher collector.
The threshold correction and widened round representation retain their previously
documented runtime origins.

Thirteen formal scenarios cover manual/unassisted/inferred/warm comparisons,
wrong slots/rounds/counters, duplicate counting, changing the round field, missing
accounting, a new overflow restriction, a missing loop invariant and explicit
sound mode. Default exact record inference proves soundness, maximal input safety
and exact final modeled state; the explicit sound comparison records the same
precision proofs as optional. Vector contents are the modeled observations, not
allocation addresses or capacity. Every semantic control must leave all inferred helper proofs
successfully checked. For domain controls, the minimal invariant harness itself
must fail its call precondition; failure of the behavioral oracle alone does not
count as failed input-domain coverage.

The overflow control deliberately inserts `creator + 1` before the bounds guard.
Inference correctly checks its restricted safe contract, while the invariant
harness rejects the lost `creator == usize::MAX` input. A native test catches the
same introduced overflow. Five native runs execute the supplied three tests on
original and inferred bodies and detect wrong-slot, wrong-round and overflow
mutations. No new production bug is claimed.

Outputs include raw diagnostics, `results.json`, `summary.md`, per-case source
snapshots/hashes, `record-inference.patch`, native lockfile and the passing clone.
Tools, model artifacts, supplied source and runner/support files are fingerprinted.
The record summary regenerates/reproves on warm runs; scalar candidates reuse
only proposals and repeat all required proofs. State maximality/exactness remain
independently checked on every run. Timings are not a benchmark.

## Transparent NodeMap composition

```sh
python3 source/tools/eval_verus_infer_alephbft_wrappers.py \
  --alephbft /path/to/AlephBFT \
  --output /tmp/infer-aleph-wrappers \
  --native-tests --native-target /tmp/infer-aleph-wrappers-native
```

The wrapper checkpoint extracts `NodeMap` and three methods from the pinned
original `crypto/src/node.rs`. `get` and `insert` keep their original signatures,
unrestricted generic payload and executable bodies. `size` explicitly changes
`self.0.len().into()` to `NodeCount(self.0.len())`: this earlier runner retains
that adaptation. The later `--wrapper-ops` variant restores the original body
using an explicit checked From implementation. Native differential tests compare the adapted API with
all three pinned method bodies, including the original conversion. Unrelated
codec/iterator/clone implementations are outside this formal target.

The supplied collector core now stores `NodeMap<(T, u16)>`. Its inferred update
calls `size`, borrowed `get` and mutable `insert`; it consumes shared tuple
patterns and retains ordinary primitive widening and guarded counter arithmetic.
The compiler composes the helpers' checked domains and effects, tracks nested
field frames, and independently checks the complete modeled final state.
All six inferred functions (two scalar helpers, two read-only queries and two
state updates) must pass all three exact obligations in every semantic control.
Record/query templates regenerate and reprove on warm runs; scalar proposals may
hit their cache and still receive fresh proofs.

This is an explicit storage adaptation of the supplied manual core, not the
clean-main generic `UnitsCollector<H>`/`Unit`/`Hasher` implementation. Other methods
retain their contracts, loop annotations and direct access to the underlying
vector. The existing counting equivalence, minimal whole-valid-domain harness and
separate behavioral regression oracle remain. The extracted API lives in the
harness parent module so private storage is transparent to the collector;
automatic opaque views across the original crate boundary are not established.
NodeIndex/NodeCount representations are retained, while unused native derives
are excluded from the proof harness as in the earlier experiments.

Twelve formal scenarios compare unassisted failure, inferred/warm success,
wrong lookup/insertion/length, wrong round/counter, duplicate counting, changed
frames, missing accounting and an added overflow restriction. Domain controls
must fail the minimal invariant harness itself, not just its behavioral oracle.
Seven native runs exercise the original and wrapped supplied tests, lookup/insert
and overflow mutations, the API comparison and a wrong-size mutation of that
comparison. These are universal checks of the supplied properties and deliberate
mutation controls, not claims of newly discovered production bugs.

Outputs include raw diagnostics, `results.json`, source snapshots/hashes, the
pinned original node source, a reviewable storage/inference patch, native lockfile
and an isolated clone restored to its passing variant. Tool/model/support inputs
are fingerprinted; timings are observations rather than a benchmark.

## Private NodeMap API across a compiled crate boundary

```sh
python3 source/tools/eval_verus_infer_alephbft_private.py \
  --alephbft /path/to/AlephBFT \
  --output /tmp/infer-aleph-private
```

This checkpoint compiles the extracted NodeMap API into a separate crate with
private vector storage. A supplied **closed direct sequence view** defines its
mathematical contents. Inferred size/get/insert contracts use that view; the
collector cannot access the private field or unfold the definition. Lookup and
insertion retain their pinned original signatures/bodies. The previously documented
NodeCount constructor adaptation and overflow-safe threshold correction remain.
Unused native derives are excluded from the formal API as in earlier probes.

The consumer verifies only the supplied collector's update and its state specs,
checked counting equivalence, minimal valid-domain harness and separate behavior
oracle. Constructor, round-transition and selection methods are **excluded** from
this formal target; their earlier manually specified checks remain in the other
runners. This does not prove clean-main generic Unit/Hasher code or whole-crate
Aleph correctness. No new runtime algorithm change is introduced by this checkpoint.

Both ghost-only caller harnesses observe the existing checked size query before
the update. This exposes the vector length bound hidden by the closed view, without
adding an input restriction. The view and this observation are supplied guidance;
automatic abstraction and representation-lemma discovery remain future work.

Nine cases each build/check a provider and consumer, giving eighteen formal runs.
They cover inferred/warm success, wrong lookup/insertion/size, duplicate accounting,
changed frames, an additional overflow restriction and an explicitly unsupported
lossy view. All four provider functions and both consumer functions must pass
every exact obligation in semantic controls. Domain controls must reject the
minimal invariant harness. The lossy provider remains precisely verified about
its concrete storage, but does not qualify for private-view composition.

Outputs include source snapshots/hashes, immutable tool fingerprints, compiler
commands, reports/diagnostics, the original node source, a reviewable update-scope
patch and summary. Final sources and compiled interfaces use the passing variant.
This runner adds no native behavioral claim; the preceding wrapper runner retains
its seven native comparisons of these same executable API/update bodies.

## Generic trait getters and associated hash payloads

```sh
python3 source/tools/eval_verus_infer_alephbft_traits.py \
  --alephbft /path/to/AlephBFT \
  --output /tmp/infer-aleph-traits
```

This checkpoint extracts the pinned Unit creator/round default bodies and UnitCoord
creator/round bodies unchanged. It builds an explicit **reduced interface adapter**:
`Hasher::Hash: Copy`, `Unit::Hasher`, and the hash/coord/creator/round interface, with
supplied logical observation functions and ordinary contracts. A supplied closed
coordinate-parts definition keeps coordinate fields private. A deterministic ProbeUnit
implementation checks these contracts; it is not Aleph's cached FullUnit implementation.
Unrelated control-hash/session methods, marker/codec bounds and the hashing algorithm
are omitted and recorded in the report. This does not verify the full original traits.

The separate provider also retains the earlier private NodeMap API and its recorded
size adaptation/threshold correction. Its getter declarations and ordinary bodies
must pass full verification before export. The consumer adapts the supplied core to
`Collector<H: Hasher>` storing `H::Hash`. The original direct update body remains;
a new inferred `add_unit<U: Unit<Hasher = H>>` reads creator/hash/round and calls it.
No implementation name or Aleph trait name is recognized by compiler inference.

The supplied invariant/count equivalence and old direct-value behavior oracle remain.
A new unit invariant harness supplies no unit-domain restriction, and a separate unit
behavior oracle expresses the old behavior using supplied observation functions.
Both observe the checked size query to expose its closed-view length bound. The
wrong-slot adapter control must preserve the invariant while failing behavior.
Constructor/transition/selection methods and the original collector's NodeCount
operator interface remain outside this formal target.

Nine cases produce sixteen formal provider/consumer runs: two faulty providers stop
before consumer verification; the remaining cases cover a weak getter contract,
wrong adapter slot, duplicate counting, changed frame, an added unit-input overflow
restriction and inferred/warm success. Semantic consumer controls require all three
inferred functions to retain every exact proof; providers retain their four checked
inferred API functions. Ordinary provider failures must leave no interface, weak
contracts must produce an unsupported diagnostic, and overflow must fail the new
minimal unit harness. Sources, commands, raw diagnostics, hashes, original units/types,
scope adaptations, patch and summary are saved. This runner adds no native behavioral
claim or newly discovered production-bug claim; prior native wrapper controls remain.

## Wrapper conversions and counter operators

```sh
python3 source/tools/eval_verus_infer_alephbft_traits.py \
  --alephbft /path/to/AlephBFT --wrapper-ops \
  --output /tmp/infer-aleph-wrapper-traits
```

This variant retains the pinned `NodeMap::size`, `get`, and `insert` bodies
unchanged, including `self.0.len().into()`. The previous constructor spelling for
size is removed. The supplied generic collector stores `direct_parents: NodeCount`
and uses the production increment expression `self.direct_parents += NodeCount(1)`.
Its logical counter observations explicitly read `.0`.

Two explicit trait implementations supply the formal adapter's `From<usize>` and
`AddAssign` API. Their impl-level inference attributes generate ordinary vstd
extension definitions, and their unchanged executable bodies receive all three
exact checks. This verifies the supplied implementations; it does not verify the
original derive_more expansion or automatically annotate arbitrary native derives.
The remaining reduced Unit/Hasher adapter, manual view/invariant/count equivalence,
behavior oracle, size observations, threshold correction and widened-round
supplied collector core remain explicit. Other production collector methods and
cached FullUnit hashing remain outside this target.

Thirteen cases yield twenty-one provider/consumer runs. The provider has six
inferred functions and each reached consumer has three. Three new provider
controls reject a restrictive operator requirement, a weak overflow requirement,
and an incorrect operator model. The restrictive case must pass soundness while
failing maximality/boundary certification. None may export an interface. A fourth
control consistently changes the operator implementation and model to replacement:
its exact candidates pass, while the collector's application goal fails. The
earlier getter, slot, accounting, frame and domain controls remain, followed by
cold/warm positives. Reports distinguish supplied wrapper implementations from
original method bodies. No new native Aleph execution or production bug is claimed.

## Construction and round advancement

```sh
python3 source/tools/eval_verus_infer_alephbft_lifecycle.py \
  --alephbft /path/to/AlephBFT \
  --output /tmp/infer-aleph-lifecycle
```

This checkpoint infers `new` and `next` in the supplied collector core. The direct
Vec variant preserves their executable bodies and removes their handwritten
method contracts, loop annotations and local proof blocks. The former method
contracts become ordinary `check_new` and `check_next` goals. A further lifecycle
harness constructs a collector, adds a candidate, advances, adds another candidate
and proves the invariant still holds. It does not discover that invariant or the
application's count/round meanings.

The private variant uses a separately compiled NodeMap provider. It retains the
pinned `size`, `get`, `insert` and **`with_size`** bodies, including native `.into()`
syntax. A checked `From<NodeCount> for usize` adapter supports the constructor.
NodeMap retains Rust's native `#[derive(Clone)]`, with
`#[verus_verify(clone_via_view)]` before the derive. Its real body must prove the
generated public relational view contract. The supplied collector's `next` now
calls `.clone()`; there is no `copy_contents` adapter. The formal Hasher interface
explicitly requires `Hash: CloneIdentity`, and its checked broadcast law proves
value preservation for the `(Hash, Round)` payload. Copy alone does not establish
Clone equality. No private field is exposed in the consumer.

Five direct cases and six private cases yield **17 provider/consumer runs**. Direct
runs have five exact candidates; private providers have eight and consumers five.
Providers contain one `owned_value` candidate, and consumers two; all require all
three proofs. The native derive separately receives ordinary body verification.
Cold/warm positives bracket semantic controls: wrong initial
round, wrong advance, missed counter reset, premature terminal round and an
unnecessary constructor overflow restriction. Those controls require a named
new/next application goal to fail while all inference checks pass; frontend and
resource failures are not accepted as evidence. Snapshots, commands, artifact
hashes, JSON reports and the explicit adapter patch are retained in the output.

The original source checkout stays untouched. The corrected threshold, supplied
counting equivalence/invariant/behavior oracles, reduced Unit/Hasher interface,
explicit wrapper adapters and widened u32/Option terminal policy remain visible.
This does not prove the full clean-main collector or arbitrary Clone semantics,
and does not claim a new production bug discovery. The existing original
u16 round-overflow issue remains distinct from this adapted terminal policy.

## Borrowed parent-selection queries

```sh
python3 source/tools/eval_verus_infer_alephbft_selection.py \
  --alephbft /path/to/AlephBFT --output /tmp/infer-aleph-selection
```

This adds inferred prospective_parents to the preceding supplied-core lifecycle.
Ordinary goals check readiness iff success, complete returned contents, and the
specific NotEnoughParents/MissingOwnParent error policies. Direct queries borrow
the Vec rather than a slice; private queries borrow NodeMap and use its checked
size/get API. Their branch order and explicit out-of-range error policy are
retained. Ghost-only private callers observe size and conditional get to expose
checked representation/type facts. The original clean-main query lacks that
explicit bounds guard, so this is not a claim about its unrestricted input domain.

Five direct and five private consumer cases, plus one private provider, yield
11 expected formal outcomes. Every consumer has six exact inferred functions,
including three structured values; the provider retains eight. Wrong error,
overpermissive own-parent selection and removed-index-guard controls must fail
the named selection goal while every candidate retains all three proofs. Warm
positives follow the controls. Existing invariant, count, round and trait adapters
remain. Native Clone replaces the former Copy adapter. Cached FullUnit is covered
by the separate implementation checkpoint below.

## Native Unit and Hasher bodies

```sh
python3 source/tools/eval_verus_infer_alephbft_units.py \
  --alephbft /path/to/AlephBFT --output /tmp/infer-aleph-units \
  --native-tests --native-target /tmp/aleph-native-target
```

The runner extracts pinned source bodies for Hasher64::hash, UnitCoord methods,
PreUnit construction/getters, ControlHash getters, FullUnit construction/clone,
all four FullUnit Unit methods, and Unit's creator/round defaults. Native record
Clone derives are retained and proved against supplied public view relations.
The actual Hasher64 body verifies against vstd's existing DefaultHasher model
and a u64 native-endian byte model. No collision-resistance property is asserted.

The formal implementation has two explicit library boundaries/adaptations:

- A separately compiled **trusted codec model** maps the immutable view to opaque
  deterministic SCALE bytes, skipping the cache. FullUnit's encode bridge uses
  that model; its using_encoded default body is checked. The SCALE algorithm and
  generated Encode implementation are not formally verified. Model source and
  compiled artifact hashes are recorded and guarded throughout the run.
- Parking_lot is replaced by vstd's verified RwLock. A fixed ghost predicate permits
  None or Some of the canonical hash. Copied reads release their handle, and writes
  require that predicate. Native `*read()` becomes `read_copy()`, write assignment
  becomes `replace()`, and clone's try_read-and_then becomes `try_read_or(None)`.
  This makes one finite atomic read attempt, returning the allowed fallback on
  failure and releasing successful handles. Ghost constructor arguments and
  explicit type-invariant proofs are added.
  This establishes functional correctness of the adapted body; it does not prove
  parking_lot's RAII/Drop implementation or prove equivalence of its lock protocol.

Native marker/codec bounds remain reduced. Hash requires checked CloneIdentity;
Data requires checked CloneView and has no Copy bound. The model uses the data's
deep view, which must preserve every encoded distinction; CloneView alone does
not prove that model-completeness condition. Concrete callers cover the actual
mock types `[u8;8]` / `u32` and non-Copy `Vec<u8>` data. Application invariants,
count equivalence and the supplied collector's widened-round policy remain inputs.

Seven formal provider/consumer outcomes include cold/warm positives and semantic
rejections for a permissive cache invariant, hashing the wrong input, and a wrong
session getter. All eight provider candidates and five consumer candidates must
retain all three inference proofs even in the negative cases. Failed providers
must not export an interface. Ordinary goals prove repeated-hash equality,
clone/hash consistency and concrete FullUnit use by the supplied collector.

With `--native-tests`, five real Aleph tests exercise codec/hash behavior,
cold/warm caches, non-Copy payloads, cloning, and try_read while a writer holds the lock. Removing
the cloned payload must fail a runtime test; restoring it must pass. These are
runtime evidence for the native library paths, separate from the adapted proof.
The source checkout is never edited. This checkpoint does not verify the whole
Aleph crate, arbitrary Data/Clone implementations, the clean-main collector, or
distributed protocol properties.

## Original native collector

```sh
python3 source/tools/eval_verus_infer_alephbft_native_collector.py \
  --alephbft /path/to/AlephBFT --output /tmp/infer-native-collector \
  --native-tests --native-target /tmp/aleph-native-target
```

All four original `UnitsCollector` executable bodies are extracted and checked for
byte preservation. Their contracts are inferred in default exact mode. The round
remains u16 and no new bounds guards or terminal policy are added. Supplied ordinary
goals check lifecycle, invariant/count preservation, exact slot updates, frames,
and both selection errors. Checked scalar comparison adapters and a proved private
view-length companion let the original APIs compose without manual size/get
observations in these callers. The existing corrected threshold and Unit/Hasher
codec/lock boundaries remain visible inputs.

Eight formal outcomes cover the provider, cold/warm consumers, and five semantic
negative controls. Wrong counter/reset/error behavior and removed round/node
requirements fail named caller goals while all four inferred candidates retain
their three proof receipts. The optional native run executes the 15 original
collector tests. The input checkout is never edited. The known u16 round overflow
is exposed as a safety requirement, not repaired or newly discovered here.

## Bounded parent collection queries

```sh
python3 source/tools/eval_verus_infer_alephbft_native_collector.py \
  --alephbft /path/to/AlephBFT --output /tmp/infer-parent-queries \
  --parent-queries --native-tests --native-target /tmp/aleph-native-target
```

This separate mode uses ordinary checked `vstd::iter` adapters for sparse
traversal, counting in a round range, first-match future-round search, and
index-preserving parent-round projection. `ParentQueries` supplies exact equations
for its scalar-parameterized queries; ordinary Verus proves their implementations
and callback contracts. A generic record validator infers a complete Result policy
for quorum and future-round checks with all three precision receipts.

Nine formal outcomes include cold/warm composition, checked callback/projection
rejections, and precise-but-wrong threshold/error policies or missing round bounds.
Failed providers must not export interfaces. Cold and warm compiled consumers each
run the same 1,530-case corpus. Eleven native ControlHash tests include a matching
indexed-reference corpus over the original native pipelines. Completion markers
ensure the compiled consumer actually executes its runtime entrypoint.

This mode **adapts** native enumerate/filter_map/filter/count chains to checked
finite implementations and reduces stage errors to u16 payloads. It does not
infer the untouched native iterator chains, the full ControlHash validator/error
enum, or construction-loop invariants. The supplied model equations, callback
contracts and ordinary collection proofs are inputs. Those remaining frontend and
inference extensions are recorded in VERUS_INFER.md.

## Typed noninitial validation

```sh
python3 source/tools/eval_verus_infer_alephbft_validation.py \
  --alephbft /path/to/AlephBFT --output /tmp/infer-typed-validation \
  --native-tests --native-target /tmp/aleph-native-target
```

This experiment uses Aleph's original six-variant `Error<H>` declaration and
infers the original own-parent lookup, member-count wrapper and noninitial
validation body, including its runtime assertion and three `?` calls. The two
count/search pipelines use explicit checked collection queries. Fifteen inferred
methods receive soundness, safe-domain maximality and output exactness proofs.
Supplied ordinary goals check acceptance and the complete first-error policy:
missing own parent, wrong own round, insufficient previous-round quorum, then
future parent. Error payloads retain the actual creator index or round.

Ten formal outcomes include cold/warm runs, a compiled runtime build, wrong error
payload/tag/priority, ignored propagation, a false query callback and removed
round/index requirements. Each negative control must reach a semantic proof
failure, retain the fifteen exact inference receipts and remove its interface.
Both the adapted target and untouched native noninitial methods run the same
7,704 cases across masks, creators, round patterns and u16::MAX. A separate caught
zero-round panic checks that the original assertion is still executable. Native
tests run in an isolated pinned checkout, alongside the ten original ControlHash
tests; the input checkout is not edited.

The error declaration itself covers all six variants, but this experiment proves
the four noninitial errors. The optional modes below cover native pipelines and
construction; initial-round checks and encoding-completeness proofs remain separate. The
query adapter eagerly computes round - 1, so its isolated future-parent leaf has
a narrower zero-round domain than the native empty-collection leaf. The complete
noninitial path's original positive-round assertion makes this difference
irrelevant to its verified scope. Codec, lock, application policy and collection
view boundaries remain the explicit supplied inputs described above.


## Native noninitial collection pipelines

Add `--native-pipelines` to the typed-validation command above. This mode retains
all four original noninitial method bodies, including the native
`parents().filter(...).count()` and `parents().any(...)` callbacks. Both collection
query implementation adapters are removed. The collection/parent iterator
factories retain their native operations with named locals and supplied ghost
contracts/proofs; their meanings are not inferred. The runner checks pinned source
factory bodies before installing that interface.

The same fifteen inferred methods receive all three exact-mode proofs. Cold/warm
runs, semantic mutations, and the 7,704-case native and proof-target runtime corpora
check the same typed first-error policy. An additional empty/nonempty zero-round
future-search probe preserves Rust's lazy behavior; the complete noninitial
validator still rejects round zero through its original runtime assertion.

This adds an explicit trusted Rust-library model for native enumerate/filter_map
and implementation-specific count behavior. Native Map/Enumerate count delegates
to the inner count and does not establish mapper execution or exhaustion;
Filter/FilterMap count evaluates callbacks and establishes completion on return.
Only supported native iterator families receive these laws. It is not a proof of
precompiled Rust core. The older checked `vstd::iter` consumer experiment remains
available and retains its previous trust boundary.

Factory/view meanings, codec/lock boundaries, corrected quorum arithmetic,
application policy and caller goals remain supplied. The construction mode below
also covers original ControlHash::new loop inference; full-validation mode below covers initial rounds.


## Native ControlHash construction and typed validation

```sh
python3 source/tools/eval_verus_infer_alephbft_validation.py \
  --alephbft /path/to/AlephBFT --output /tmp/infer-native-construction \
  --native-construction --native-tests --native-target /tmp/aleph-native-target
```

This mode implies `--native-pipelines` and keeps the original `ControlHash::new`
body, including its native sparse iterator loop and original update calls. It
infers an exact unrestricted-domain fold/hash summary. The original
`create_control_hash` body has an ordinary supplied hash-observation contract;
the NodeMap codec bridge uses the existing explicit encoded-view model. No new
codec or hash algorithm is assumed beyond that existing boundary.

Sixteen inferred methods receive all three required proofs. Supplied caller goals
check the complete constructed view and connect it to the original noninitial
validator's exact errors and acceptance policy. Twelve formal outcomes include
cold/warm verification, a runtime build, the earlier semantic controls, a wrong
projected round and a wrong hash input. The changed implementations can retain
valid exact inferred contracts while failing the intended ordinary caller goals;
failed providers publish no interface.

The proof-only codec bridge has no executable implementation. The formal runtime
corpus therefore retains supplied states. The separate untouched native corpus
uses Aleph's actual constructor, SCALE implementation and hasher for 378 maps plus
an empty map, checks every projected round and the combined hash, and runs the
same 7,704 typed-validation cases. Both preserve the zero-round search probes.

Loop invariants and the fold are synthesized. Collection and iterator meanings,
factory proofs and index bounds, the hash query, codec boundary, caller policy and
goals remain supplied. The first native-loop profile handles total one-accumulator
folds; partial domains, early exits and arbitrary iterator loops remain open.
Use the full-validation mode below for initial-round hash behavior. Proving the
codec implementation itself remains separate work.

## Complete public validation and demanded constructor facts

Add `--full-validation` to the command above. This implies both native modes and
adds inference of the unchanged `validate_initial_round` and public `validate`
bodies. Eighteen methods receive soundness, maximality and exactness proofs.
Complete value-only `Result` equations compose without repeatedly rebuilding
separate tag/payload observations. The dispatcher fits the unchanged 4,096-node
formula budget. Its safety domain and output are inferred; separate ordinary
caller goals supply the intended complete typed-error policy and conditional
creator-index safety bound. Application intent is not inferred.

Constructor callers now state bounds using the **input map length**, and prove
length preservation and per-slot round projection without handwritten fold
induction. `--infer-goals` selects checked fold length, read, sparse projection and
congruence lemmas only for requested observations. The explicit iterator meaning
uses the reusable `sparse_entries` vocabulary; its native factory proof remains
supplied and checked. A no-parent input constructs a value that passes initial
validation, with no creator-index restriction for round zero.

The original empty-map hash call also remains unchanged. General view-query
specialization composes its supplied checked hash observation with the inferred
temporary map. Generic hash comparisons use an explicit checked `PartialEqIdentity`
bound in the supplied Hasher interface. No mathematical meaning is inferred from
Rust's `Eq` marker.

Seventeen expected formal outcomes include the initial error count, hash comparison,
hash payload order, round dispatcher and missing public caller bound, in addition
to construction and noninitial controls. The proof-target runtime exercises the
public dispatcher on the existing
7,704 noninitial cases. The untouched native runtime additionally runs 254 initial
cases, including bad hashes, parent-error priority, empty maps and out-of-range
creators. Its real constructor, SCALE codec and hasher run as before. The proof-only
codec bridge remains non-executable, so initial hash execution belongs to that
separate native corpus.

Views, checked factory proofs, intended application policies, the corrected quorum
arithmetic and existing codec/lock/native-library boundaries remain supplied. This
does not verify SCALE bytes, hash security, the whole crate, or protocol correctness.

## Original parent-validator orchestration

Add `--parent-validation` to the validation runner command. It implies the full
validation and native modes. Twenty-nine methods now infer exact summaries,
including the original `Validator::validate_unit_parents`, `Signed::as_signable`,
and nine UnitCoord/PreUnit/FullUnit constructors or accessors. Their executable
bodies are extracted from the pinned source without method-contract adapters.

The parent method composes native `map_err`, a captured PreUnit clone and `?`.
Its separate supplied policy proves wrong-member-count priority, the copied
complete PreUnit view on both error paths, the exact wrapped ControlHash error,
and return of the complete original Signed object on success. Its conditional
creator bound is required only when the member count matches and the round is
nonzero. Parent-specific controls check the member comparison, nested error,
false copied-context caller goal, ignored error and missing caller bound. Failed
providers publish no interface. These controls test detection; they are not
discovered Aleph bugs. The guarded run records 22 expected formal outcomes,
including cold/warm verification and the runtime build.

The input is an already checked Signed value. The fixture preserves the original
wrapper fields and accessor body, checks field-layout drift, and reduces unused
Signable/Index/codec marker bounds. A checked Keychain count equation is supplied.
This does not verify signing, signature authentication, or the outer
`Validator::validate_unit` session/max-round checks. `--native-tests` additionally
runs the six untouched validator tests with the actual signature implementation.
Existing codec, verified-lock, corrected-quorum and native iterator boundaries
remain in force.

## Factory proofs from authored views

Add `--factory-proofs` to the validation runner command. It implies parent/full
validation and both native pipeline/construction modes. NodeMap::iter and
ControlHash::parents are extracted from the pinned source with their complete
original signatures and bodies. Each receives one
`#[verus_verify(iterator_via_view(...))]` annotation. The new fixture retains the
authored entries/parents observations and removes the manual callback contracts,
factory proof blocks and named-local adaptations from the earlier fixture.

`--infer-goals` regenerates one total immutable callback equation for each factory.
Ordinary Verus checks those closures and the generated calls to verified generic
factory combinators. The sparse-entry origin lemma supplies index provenance from
the authored view. No new native iterator assumption is introduced. The factories
are ordinary checked interfaces; the same 29 consuming/supporting methods retain
their three exact-mode proofs. Factory callback proposals are reported separately.

Additional controls change the emitted index and mapped round. Each must fail its
named ordinary factory proof and leave no exported interface. Existing validation,
payload, priority, constructor and caller-domain controls remain enabled. Native
tests and runtime corpora continue to execute the original native code.

This first profile handles shared factories returning one native map or an
enumerated slice filter_map with direct total immutable callbacks. Authored output
views, application policies, checked Clone/equality laws, codec/lock/library
boundaries and the already checked Signed input remain explicit. Arbitrary iterator
factory inference and maximal/exact iterator-object contracts are not claimed.
