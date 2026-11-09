# Exact flat scalar pair outputs

The mechanical tuple-return blocker is closed for the frozen arity-two profile.
The unchanged original `data_index_of(bit: u32) -> (usize, usize)` body now has
an inferred unconditional domain and an exact whole-pair equation. The new
`automatic_inferred_helpers.rs` has both original helpers inferred, with no
helper requires/ensures. Its stateful word-view contracts, domain and executable
bodies are identical to `word_goal_automation/automatic_inferred_mask.rs` apart
from adding tuple inference and removing its two authored component equations.
The replay guard checks that exact textual delta and frozen historical hashes.

## Scope and mechanism

Target/profile frozen before implementation in `cc8aa7a3`, on combined parent
`db26cfc7`. Flat pairs contain only the existing bool/machine-integer scalars.
Pairs are exact return/local values, never new input types. Immutable whole-pair
locals, field reads and existing exact control flow/callee composition use the
existing source eligibility, typed term, arithmetic, substitution and formula
budgets. No tuple destructuring in inferred bodies, nesting, tuple enum payload,
heap tuple, arbitrary generic/recursive tuple or approximate pair profile is
added. Pre-existing scalar/enum/scalar-array/shared-slice inputs retain their
existing bounds; no new loop/effect form is enabled.

A small `tuples` module validates native typed constructors and field operators.
Both constructor operands are evaluated and their safety conditions conjoined,
even when a later field read discards one. Mathematical scalar results use the
existing checked summary cast builders to obtain the source component types.
Postconditions use whole native tuple equality, not selected observations.
No new accessor law, assumed helper, executable rewrite or proof axiom is added.

The production original-body soundness/maximality/exactness checker is unchanged.
It independently lowers executable safety and compares an arbitrary typed WHOLE
result with the original body's return. A new backend fault-injection test bypasses
candidate generation and keeps the original pair return fixed: sound but missing
first/second clauses and a weak component range fail exactness; swapped values
fail exactness; a vacuous restrictive domain fails maximality. Complete component
equations pass. The test uses the ordinary typed generic tuple datatype and
production SST-to-AIR/Z3, not an evaluator or candidate-derived observations.

## Builds and proof evidence

Private own compiler/macros/vstd were built with Rust 1.99.1,
`PATH=/home/dev/.cargo/bin:$PATH`, `CARGO_BUILD_JOBS=2`, Cargo commands from
`source/`. Z3 was copied to own ignored `source/z3`, not linked. All compiler/library proof inputs came from this checkout and the copied
standalone solver. No parent bundle was used. No other checkout, application
input or parent runtime artifact was edited.
The own baseline and rebuilt extension vstd each verify 2,226 bodies, zero errors.
Baseline uses default release optimization; extension uses release with VIR opt=1;
unit tests use VIR opt=0 for build time. Checking phases remain enabled.

Before implementation the matching baseline reproduces the helper signature
rejection. `baseline-receipt.json` and the copied baseline bundle are private.
The first extension helper run proves all three checks and its caller: 4 queries,
zero errors. Final replay uses a separate immutable copy of the rebuilt bundle.
Compact tool/source/log hashes and outcomes are in `RESULTS.json`; raw sources,
logs and receipts remain under `/tmp/infer-scalar-tuple-20261109`.

Architecture evidence is deliberately distinct:

* The portable original helper and all eight inferred functions pass ordinary
  full checking with `--no-cheating`; helper domain is `true` for every u32.
* Two separate caller tests select `arch_word_bits() == 32` and `== 64` in their
  authored test preconditions, prove the corresponding usize maximum and run
  the same assertions, including zero and max-u32 helper outputs. Helper
  inference remains unconditional. These are explicit conditional SMT models,
  not native 32-bit execution or assumptions on the helper's bit argument.
* Native standalone original-helper execution checks 65,538 cases on this
  64-bit host, including zero, max-1 and max-u32. Native execution is supporting
  evidence, separate from proof and architecture models.

An initial `global size_of usize == 4` trial was rejected by `--no-cheating`.
Without that flag its 25 verifier queries passed, but the host native layout
assertion failed. Neither trial is accepted evidence. The final tests use no global
layout declaration, no erasure/lifetime/verification bypass, and no assumed
architecture helper. Exploratory logs are retained. A test-setup borrow error,
incorrect generic tuple test instantiation, an invalid approximate-attribute
spelling and a missing transitive native library search path were corrected;
none is reported as a successful proof run.

## Results

The final replay completes 48 runs. All 21 compiler regressions pass: 6 new
pair tests, 12 goal tests and 3 word-goal tests. Details are in `RESULTS.json`.
The accepted stateful fixture proves 20 queries, zero errors. XOR and disabled
word-goal assistance remain negative. Historical baseline still fails its five
set postcondition diagnostics; the forwarding wrapper remains blocked. The
separate canonical/original representation equivalence audit proves 1 query.
These canonical representation definitions remain explicit and are not inferred.

The unrelated `(x + 1, x / y)` case infers `x <= 254 && y != 0`; branch, immutable
pair local, swapped-field callee composition, signed division/remainder and
shared-slice pair cases also check. Negatives include overflow, zero division,
signed MIN/-1, branch under/overflow and short-slice bounds. Both discarded-left
and discarded-right cases retain every operand's safety requirement. Unsupported
arity, nested tuple, tuple input and approximate pair signatures reject.

The independent precision suite passes 25 tests, registry 6, policy/evidence 9,
and contract installation/body preservation 1. Historical/fidelity guards pass
5 + 2 tests. Compiler regression totals,
fresh cold/warm cache evidence, cross-crate/transitive fingerprints and poisoned
cache/metadata outcomes are in the compact receipt. Cache formulas carry pair
component types and all fields, use bounded typed decoding, and never carry
proof results. Weak/malformed/type/key/dependency-corrupted entries may honestly
miss and regenerate; type-correct swapped outputs must fail fresh verification.

## Versions, remaining authored work and trust

Reserve **cache70** (`verus-infer-v70:flat-scalar-pair:4096`) and **policy67**
(`candidate-bound-evidence-v67`). Schema remains **38**: the ordinary typed
native tuple AST already exists, and the certificate representation is unchanged.
Only the policy constant changes in import/export; parent policy66's mandatory
normal checking, full-scope publication and goal-library provenance enforcement
remain intact. Reconcile these reservations with the parent and rebuild the
compiler, libraries, inferred providers and all dependents after integration.
Old policy66 inferred certificates intentionally require rebuilding.

Remaining authored application work: supported-state invariant/padding policy,
logical view/count definitions, semantic input bounds, get/set semantic contracts
and canonical representation vocabulary. The literal historical baseline and
forwarding wrapper are not automatically solved. `get` inference was not added;
its ordinary semantic contract verifies using the two inferred helpers. There
is no full allocator, multiword, free/clear, transaction or concurrency claim.

Existing compiler/SMT/native tuple semantics, vstd and Vec models retain their
usual trust status. Imported `.vir` certificates remain trusted build artifacts
with existing identity/fingerprint checks, not cryptographically authenticated
proof objects. No additional assumed observation or first-party assumption was
introduced. No independent review is claimed. No push or PR was made.

Replay after building a matching bundle:

```sh
python3 -B source/tools/research/scalar_tuple_output/replay.py \
  --verus /private/matching/bundle/verus --output /private/new/output
```
