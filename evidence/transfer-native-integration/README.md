# Integrated native Infer application transfers

## Scope and result

This checkpoint combines the redb codec and unchanged-body Fedimint deadline
milestones, then adds one bounded redb page-size follow-up. The engine starts
from `verus-infer` commit `55bd67ad980e90e92c77efe4453a927f097c3386`, with the
earlier byte and readonly-return changes preserved. The final uncommitted
source snapshot, compiler/library/solver hashes, actual validation commands,
and fresh combined receipt are identified in `manifest.json`. A base commit
alone does not identify this dirty-source build. No commits or PRs are made.

| Slice | Application pin | Authored property and scope |
| --- | --- | --- |
| Fedimint deadline | `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236` | Both extracted native helper bodies and the tuple constant are byte-preserved. Exact inference supports the deadline-safety/coverage policy and a boundary/height-monotonicity follow-up over the full scalar input domain. |
| redb PageNumber codec | `c82ab865bb362bc4af067d332adc44978ff64813` | Unchanged encoding/decoding bodies establish roundtrip and injectivity on the original canonical domain; certified interfaces compose through imported callers. |
| redb page size | Same redb pin | The unchanged method establishes its equation and `result <= 4_503_599_626_321_920` for every `u32` page size when `page_order <= 20`, without extra size/region/index restrictions. |

The Fedimint request remains a three-field extraction, not the gateway's full
native type or state machine. The codec domain remains `region <= 0xfffff`,
`order <= 20`, and `index <= (0xfffff >> order)`. The successful codec caller
uses explicit **checked ghost packing facts and an existing byte-inverse
lemma**. Its frozen unaided caller still fails bitwise composition obligations.
The page-size bound likewise uses explicit checked bit-vector/nonlinear
arithmetic ghost proofs. These are not unchanged unassisted caller proofs, nor
new input restrictions, trusted application summaries, or assumed lemmas.

## Reusable implementation and integration fixes

- Canonical primitive `u64` little-endian APIs and fixed-width integer
  `TryFrom(...).unwrap()` compose without discarding representability, operand
  safety, or panic obligations.
- Readonly record early returns retain path-dependent requirements. Native
  `bool::then_some`/`Option::map_or` bridges preserve eager ordinary arguments
  and defaults, while callback requirements apply only on `Some`. Independent
  callback precision remains mandatory.
- Straight-line by-value record-to-array codecs support scalar accumulators
  and literal shifts. Nested blocks retain updates to outer bindings, discard
  block-local bindings, and cannot hide updates from readonly operand checks.
- The new array receiver route is restricted to user-defined datatype paths;
  core `Option`/`Result` array functions retain their existing scalar route.
- Straight-line borrowed record queries returning finite integers support
  variable shifts through existing typed finite-bit models. The shift-width
  obligation stays eager even when a later multiplication is by zero; product
  overflow remains separately checked. This does not admit mutable receivers,
  arbitrary control flow, or effectful operands.
- Shared registry restoration retains checked callback decoration as well as
  loop decoration. Both native Option and integer-unwrap maximality gates
  survive composition. Final versions are **cache 64 / policy 58 / schema 38**.

The independent codec review identified the nested-block local-update defect.
Fresh soundness/exactness rejected its wrong candidate and removed the export:
it was a completeness defect, not an accepted false contract. Parent validation
also caught dropped callback decoration during merging and the overly broad
Result-to-array receiver route. Both failed closed and are fixed with regression
coverage. Development failures remain in the scratch logs rather than being
silently replaced by passing results.

## Shared replay and negative controls

From the engine root, using clean locally available pinned clones:

```sh
RUSTUP_TOOLCHAIN=1.99.1 python3 -B source/tools/check_infer_transfers.py \
  --fedimint /path/to/pinned/fedimint \
  --redb /path/to/pinned/redb \
  --output /private/tmp/infer-native-replays
```

The default runs all three slices against one selected verifier/vstd/solver
bundle. `--scenario fedimint`, `--scenario redb`, and
`--scenario redb_page_size` select individual slices. Both redb slices reuse
the same checkout. There is no network, cloning, source mutation, or compiler
build during replay. See `source/tools/INFER_TRANSFERS.md` for setup and flags.

The shared entry point reopens selected raw proof reports, checks all three
fresh required exact-inference obligations, full fixture verification,
positive interface presence, and rejected negative exports. It guards source
and tool identities, checks application pins/cleanliness before and after,
retains failure receipts, and attempts subsequent slices after ordinary
scenario failure. Controller completion is distinct from application proof:
an honestly blocked slice cannot make the suite pass.

Faithful Fedimint guard deletion and redb decoder mutation retain successful
exact helper inference but fail the authored policy. False goals reject too.
Page-size safety controls separately reject order 64/255, every order at least
64 even with size zero, and order 63 with size two. Safety controls must fail
their **precondition** obligation; the false-property control must fail its
**postcondition**. The suite rejects swapped/mixed diagnoses, frontend errors,
panics, timeouts, and resource exhaustion as substitutes. Seeded stale exports
are removed while positive exports remain available.

Cold and warm proofs rerun required checks. Readonly/owned record candidates
regenerate and honestly report cache misses; these are not proof-cache hits.
Native oracles and native mutants provide finite differential evidence, not
the arbitrary-input formal proof or an exhaustive application-state search.

## Validation, review, and trust

The final shared replay reports `success: true` and
`application_proofs_proved: true`: all **three slices pass**. Its 29 selected
proof commands reestablish **132 mandatory exact-inference obligations**,
including the expected negative caller failures. The raw result includes
Fedimint **8/0** cold/warm and **10/0** follow-up, codec **68/0** authored
roundtrip and **71/0** injectivity/provider, imported codec callers **8/0**,
and page-size **12/0** direct/imported runs. These counts are verifier
obligations per fixture, not counts of whole application functions verified.

Final combined validation passes **572 Rust tests**: 125 VIR unit tests and
447 integration tests across 16 binaries. It also passes **138 normal Python
tests** and **30 optimized Python test executions**, plus a fresh vstd build
with **2,221 verified / 0 errors**. Rustfmt, Verusfmt, and `git diff --check`
pass. All 1,306 guarded source files and the full 1,314-entry source/tool
artifact ledger remain unchanged during the combined proof replay.

`manifest.json` records the final build, fresh vstd verification, VIR units,
focused and adjacent Rust integration suites, normal/optimized Python harness
tests, formatting, and the actual combined front-door replay. Historical
reports stay unchanged and describe their isolated snapshots; their test
counts are not added to the final combined test total.

Independent reviews cover readonly-return soundness, primitive byte trust,
codec/conversion inference, native Option/frontend correspondence, and the
combined local-update/profile/callback/receipt integration. The integration
review reports no substantive additional finding, but explicitly excludes the
later page-size feature and final safety-negative extension. Those receive
parent source review and fresh combined tests/replays, not an invented
independent review. Review report paths, hashes, and exact limits are recorded
in the manifest.

The trusted boundary remains the ordinary Verus/Rust/SMT pipeline and imported
library specifications. The two native byte specifications explicitly trust
primitive behavior and `no_unwind`; Rust standard-library implementation bodies
are not verified. The two native Option bridge bodies are verified, but the
frontend correspondence from libcore APIs to those definitions remains trusted
code, not a universal equivalence proof for arbitrary effects/drop behavior.
No new trusted application summary or application axiom is introduced.

This is **not** whole-Fedimint fund safety, payment lifecycle correctness,
whole-redb/database correctness, concurrency, crash recovery, or cryptographic
verification. It is not a full-workspace green claim: the previously disclosed
18 unchanged VIR doctest failures were not repaired or included in the passing
library/integration commands. The next higher-value research step is either
reusable checked packing assistance to reduce caller ghost boilerplate or a
separately scoped stateful gateway invariant, not a larger claim from these
helper proofs.
