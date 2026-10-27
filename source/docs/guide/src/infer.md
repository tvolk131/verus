# Experimental contract inference

Verus Infer generates and checks contracts for a small subset of executable Rust.
Opt a function in with `#[verus_infer]`, imported by `vstd::prelude::*`:

```rust
use vstd::prelude::*;

#[verus_infer]
fn debit(balance: u64, amount: u64) -> u64 {
    balance - amount
}
```

No `requires` or `ensures` is needed on this function. Inference generates a
precondition equivalent to `amount <= balance` and a postcondition stating that
the returned value is the mathematical difference. Callers must prove that
precondition, and can use the postcondition in ordinary Verus proofs.

Inside `verus!`, use `#[verifier::infer]` on an `exec` function instead. The
equivalent attribute spelling for native Rust is `#[verus_verify(infer)]`.
Exact inference is the default. `#[verus_infer(precision = "exact")]` requests
it explicitly. Opt into sound interval summaries with
`#[verus_infer(precision = "sound")]`; inside `verus!`, the equivalent spellings
are `#[verifier::infer]` and `#[verifier::infer(sound)]`. Native attribute
rewriting also supports `#[verus_verify(infer(sound))]`.

## Demand-driven scalar proof assistance

The experimental `--infer-goals` command-line flag asks an additional bounded
proof strategy to handle eligible ordinary caller assertions. For example:

```rust
use vstd::prelude::*;

#[verus_infer]
fn bucket(key: u64) -> u8 { (key & 15) as u8 }

verus! {
fn caller(key: u64) {
    let index = bucket(key);
    assert(index < 16);
}
}
```

Run `verus example.rs --crate-type=lib --infer-goals`. Exact inference still
publishes the equation `result == ((key & 15) as u8)`. The additional strategy
starts from the assertion, substitutes relevant exact summaries and local
definitions, and proposes a conditional theorem for the existing bit-vector
prover. Ordinary verification then uses that checked theorem to establish the
unchanged assertion. It does not eagerly add ranges or alternative equations
to every function's contract.

The first strategy handles assertions whose expanded goals depend on an exact
inferred call and contain bitwise `&`, `|`, or `^`. Free variables must be Booleans
or fixed-width integers. Caller requirements and branch conditions can supply
explicit premises. Type bounds and nonzero divisors are premises too, never
unchecked facts. Each complete theorem is checked and used verbatim, and all
original calls, safety checks and assertions remain. A failed or timed-out
auxiliary proof fails verification, as with a handwritten `by(bit_vector)` proof.
Goal-assisted exports wait for complete verification, including for consumers
with only imported inferred contracts. Previous outputs are invalidated as soon
as their paths are known (before parsing for direct `--export` paths).

This is an opt-in strategy, not yet an ordinary-proof-first retry planner.
Omitted unsupported context can make it less effective than a manual proof.
It handles at most eight requests per caller, with 2,048 expanded nodes and
32 substitution levels per request, 8,192 statement visits and a 4,096-bit
intermediate-width limit. Unsupported/exhausted requests get no generated fact;
the original verifier still must prove them. It does not strengthen sound-only
contracts, infer loop invariants, directly discharge postconditions or implicit
index obligations, or cache successful lemmas. An explicit proved assertion can
provide the fact needed by a subsequent array access.

The source-connected AlephBFT pilot can be reproduced without network access:

```sh
python3 source/tools/eval_verus_infer_alephbft.py \
    --alephbft /path/to/AlephBFT \
    --output /tmp/aleph-infer-pilot --native-tests
```

Build this fork first. The runner makes a local clone at commit
`f35c7bba6d628bcafdf0d65dd471cc6f868ec034`, ignoring the input checkout's uncommitted
changes. It mechanically moves the original `common_vote` function to a shared
production module, checks executable-body identity, and verifies that exact
source file. The canonical remainder schedule and round alternation already
verify without assistance. An equivalent low-bit parity formulation needs two
manual local lemma invocations or the new strategy. This is a representation
bridge, not a new voting-agreement or recovery theorem.

Results include ordinary handwritten-contract/manual-proof, inferred baseline,
inferred/manual-proof and inferred/assisted variants; five semantic negative
controls; the source integration patch; tool fingerprints; proof reports; and
raw diagnostics. Timing observations are not a performance benchmark. Six
production election tests pass with default features. The pinned original
`--no-default-features` build fails in `collection/mod.rs`; the runner compares
the patched and unmodified error signatures and reports this as a baseline
limitation, not as passing tests. Quorum wrappers, generic state mutation and
the larger distributed proofs remain outside this pilot.

## Sound approximations and partial success

```rust
#[verus_infer(precision = "sound")]
fn fee(amount: u8) -> u8 { amount / 100 }
```

This mode proposes a contract equivalent to `requires true; ensures 0 <= result
&& result <= 2`. An ordinary caller can prove the range for every input. It
cannot conclude `fee(255) == 2` from that summary alone: the exact relationship
between the argument and result has deliberately been discarded.

Signed integers from `i8` through `i128` are supported too:

```rust
#[verus_infer(precision = "sound")]
fn scaled(x: i8) -> i8 { x / 10 }
```

This produces a range equivalent to `-12 <= result && result <= 12` for every
input. Signed remainder by 10 produces `-9 <= result && result <= 9`. These
ranges do not establish `scaled(30) == 3`; use an exact-mode contract when a
caller needs that relationship. `examples/verus_infer_partial.rs` demonstrates
both unsigned and signed range properties.

Every published summary still needs a successful ordinary soundness proof
against the original executable. Maximality and output exactness are attempted
separately when justified; counterexamples or timeouts in these optional probes
produce warnings and honest precision labels, while the sound contract remains
usable. A soundness failure or timeout still blocks dependent verification and
inference export. Exact mode continues to require all three proofs and never
silently falls back.

The first abstract domain tracks integer intervals and known/unknown Boolean
values. It handles the supported scalar expressions, refines ranges in reachable
branches and short-circuit expressions, follows immutable copies and Boolean
local guards, joins branch outputs, and composes inferred interfaces. When exact symbolic safety extraction fits
the budget and uses only exact-mode callees, sound mode retains that safety
predicate and widens the output relation to a range. If symbolic expansion
exceeds its budget, or a sound-mode callee prevents exact substitution, a
bounded compositional analysis derives safety conditions from callee
preconditions, result ranges, and the caller's operations. It keeps affine
arithmetic compact, so repeated addition can retain meaningful overflow bounds
without exponentially expanding local substitutions. Arguments start with
their full Rust type ranges; there is no automatic `[0, 1]` or `[-1, 1]` input
restriction.

For example, a sound `scaled(x: i8)` summary with `requires true` and result
range `[-12, 12]` lets `scaled(x) + 1` accept every `i8` input, since its result
lies in `[-11, 13]`. For `scaled(x) + delta`, a sufficient derived condition is
`-116 <= delta && delta <= 115`, with `x` unrestricted. Callee requirements are
preserved, including when arguments are approximate values. Conditional and
short-circuit guards scope safety conditions to the paths that execute; simple
guards on local values and their copies also refine the available facts.

The verifier must prove safety and the result range under the proposed domain.
These domains can still be conservative when contracts lose correlations. If
analysis exhausts its bounds or cannot express a usable sufficient condition,
it reports incomplete instead of guessing small input values. Ordinary proof
failure still blocks publication. Nonlinear arithmetic can require Verus's
existing solver guidance. Larger admitted domains can also produce wider output
intervals; a sound summary does not preserve caller-specific input/output
relationships unless they are present in the published contract.

Intervals use mathematical integer bounds for arithmetic and join branch
ranges. Signed division accounts for both negative and positive nonzero
divisors, including intervals spanning zero. Safety still excludes zero and
`(MIN, -1)` through the ordinary executable checks. Casts retain a tight interval
when its endpoints remain in one destination modular cycle, including wrapping
intervals such as `128..=130` cast to `i8`, which gives `-128..=-126`.
An interval crossing a modular boundary widens to the full destination range. Signed `MIN` literals are recognized before
their special compiler lowering is analyzed. Enum values, arrays, and early
returns still require exact mode.

Exact inferred callers conservatively require exact-mode callees, including
across crate boundaries and cache hits. Consume a sound-mode interface from an
ordinary verified caller or another sound-mode inferred function. Even when a
sound-mode callee happens to pass its precision probes, this initial composition
policy retains that restriction. Maximality probing is skipped for inferred
callers of sound-mode dependencies: assuming a potentially restrictive callee
precondition would not characterize all safe executions. Boolean top summaries
may also omit an exactness probe when no result relation is retained. Skipped
obligations remain **NOT ESTABLISHED**, never **PROVED**.

Use `precision_rlimit` to limit precision probes independently of the mandatory
soundness proof, in the same units as Verus's `rlimit`:

```rust
#[verus_infer(precision = "sound", precision_rlimit = 0.1)]
fn fee(amount: u8) -> u8 { amount / 100 }
```

Inside `verus!`, combine `#[verifier::infer(sound)]` with
`#[verifier::infer_precision_rlimit(0.1)]`. Without this override, precision
probes inherit the normal function/global solver budget. The override must be
finite and positive (tiny values round up to one solver resource unit); ordinary `#[verifier::rlimit(...)]` still controls soundness.
Other proposed RFC options, safe-only modes, and inference of unsupported data
structures remain unimplemented.

## Running the prototype

Build Verus as described in [BUILD.md](https://github.com/verus-lang/verus/blob/main/BUILD.md),
including `vstd`, then run from the repository root:

When invoking a debug compiler outside `source/`, also set
`RUST_MIN_STACK=20971520`, matching `source/.cargo/config.toml`. Cargo commands
launched inside `source/` receive that setting from the project configuration.

```sh
export VERUS_Z3_PATH="$PWD/source/z3"
source/target-verus/release/verus examples/verus_infer.rs
```

To compile and run the verified example:

```sh
source/target-verus/release/verus examples/verus_infer.rs --compile -o /tmp/verus-infer-example
/tmp/verus-infer-example
```

If building manually with Cargo, the equivalent development setup from `source/` is:

```sh
cargo build --release -p rust_verify -p verus -p cargo-verus
PATH="$PWD/target-verus/release:$PATH" target-verus/release/cargo-verus verus build --release --manifest-path vstd/Cargo.toml
```

The driver prints each candidate formula and separate results for:

- **Soundness:** every admitted input executes safely and satisfies the result relation.
- **Safe-domain maximality:** every safe execution starts from an admitted input.
- **Output exactness:** the result relation admits exactly the executable's returned value.

Inferred functions are checked in callee-first order. A failed or incomplete
callee stops verification before any dependent inferred function is checked.
After all required checks succeed for every inferred function, the driver makes
those contracts available to ordinary caller verification. Exact mode requires
all three checks; sound mode requires soundness and records the precision probes
independently. A required check failure or timeout prevents dependent verification
and publication. Individual proof results remain visible in diagnostics.

The example automatically infers two more layers over `compute` and `debit`,
then proves a new application property using the top-level summary. It also uses
`fee` to prove that inputs below 100 incur no fee. These proofs use callee
contracts without inlining the executable implementations.

## Structured contract reports

`--output-json` includes a `verus-infer` object with schema version 1. Save it
with the existing timing breakdown using:

```sh
source/target-verus/release/verus examples/verus_infer_partial.rs \
  --output-json --time > /tmp/infer-report.json
```

Stdout contains the JSON; source and proof diagnostics remain on stderr. Check
the process exit status as well as the report. Normal compiler and verification
failures still produce a report when control returns to the driver; an invalid
command line, process crash, or external interruption may produce none.

Each generated function records its name, source location, parameter/result
types, requested `mode`, actual `candidate_id`, separate `contract.requires` and
`contract.ensures` strings, and three `proofs` entries. These predicates come
from the actual installed candidate. They use mathematical integer arithmetic,
explicit machine casts, and `rust_div`/`rust_rem` for signed truncating arithmetic.
`result` names the return value. Bounded simplification happens before installation:
redundant typed bounds and Boolean conditions can disappear, affine terms can
cancel, and simple local-view bounds can reduce to a direct index bound. The
simplified formulas themselves receive all required fresh proofs. For example,
`(x + 1) - x` on `u8` has `requires x <= 254; ensures result == 1`; cancellation
retains the prior addition's safety condition. There is no separate display-only
predicate substituted for the checked formula.

| Proof status | Meaning |
| --- | --- |
| `proved` | Candidate-bound evidence records successful verification of every required subquery for that obligation. |
| `not_proved` | The query did not establish the obligation. This does not by itself prove the candidate false. |
| `incomplete` | At least one subquery exhausted its solver resource limit. |
| `not_established` | No successful check is recorded; `reason` explains a skip, a check not run, or unavailable matching evidence. |

Every proof entry also has `required` and `reason` fields. Sound mode requires
soundness; exact mode requires all three obligations. A sound-mode dependency
explains a skipped maximality probe with `sound_mode_dependency`. An unconstrained
Boolean result can skip exactness with `unconstrained_result`. Neither skip
becomes a proved precision claim. Optional failed or incomplete probes can
coexist with successful sound-mode verification.

`required_checks_passed` uses the same candidate-identity and poisoned-evidence
checks as acceptance. `available_to_same_crate_callers` records whether all
inferred functions passed the required-check barrier. `exported` additionally
requires successful interface writing and public visibility. A later ordinary
caller failure can leave valid inference proofs visible while preventing
interface export. The crate-level `interface_written` reports the actual export
action; an invocation without an export path leaves it false.

Crate-level `status` distinguishes `verified`, `no_inference`, `unsupported`,
`analysis_incomplete`, `generation_failed`, `required_proof_failed`,
`required_proof_incomplete`, `caller_proof_failed`, and `verification_failed`.
`generation_failure` retains the synthesis diagnostic. Other frontend, import,
and verifier errors use `verification_failed` with details on stderr. An early
frontend failure can precede discovery of annotations; zero requested or
generated functions in such a failed invocation does not mean it had no
inference annotations. Partial generation reports only completed candidates,
without granting them proof status or publication.

Per-function dependencies record inferred callee identities and modes. The
crate report also lists validated imported inference fingerprints and trusted
standard-model fingerprints, including transitive assumptions. These preserve
the existing Verus trust policy. This JSON is a read-only report; it cannot be
imported as a certificate or used to bypass verification. Candidate IDs identify
actual typed formulas and provenance, including source spans; they are not stable
semantic hashes for comparing cold and warm runs.

Counted-loop functions also report `loop_proofs`: the installed invariant
predicates, decreasing measure, loop-isolation setting, and arithmetic prover.
These are proposed proof annotations and settings. Successful ordinary soundness establishes invariant
and termination obligations under the reported precondition; the separate
precision queries retain their own checks. The report does not mint an additional
termination receipt or treat an unchecked annotation as a theorem.

`metrics` records generation time, candidate-cache hits/misses, candidate SMT
time, and expanded predicate AST sizes. Generation and solver results remain
separate: a cache hit still requires fresh proofs. Optional precision queries
and diagnostic reruns contribute to SMT time. The whole-generation timer also
includes dependency ordering and failed work. See the
[evaluation corpus](../../../tools/infer_eval/README.md) for measurement scope.

## Supported code and safety policy

Exact mode supports `bool`, `usize`, `isize`, fixed-width unsigned integers (`u8`, `u16`, `u32`,
`u64`, `u128`), and fixed-width signed integers (`i8`, `i16`, `i32`, `i64`,
`i128`), immutable parameters and locals, scalar return values, `+`, `-`, `*`,
`/`, `%`, signed negation, comparisons, Boolean operations, signed/unsigned integer
casts, bitwise `&`, `|`, `^`, `!`, shifts `<<`/`>>`, and `if` expressions.
Short-circuit conditions guard the safety checks of
the expressions that are actually evaluated. Casts retain Rust's truncation
semantics; inferred arithmetic formulas otherwise use mathematical integers.
As in ordinary Verus proofs, callers may need a `by(bit_vector)` assertion to
derive a concrete value from a narrowing cast outside its destination range.

`usize`/`isize` arithmetic and casts use Verus's existing symbolic pointer-width model.
The default verifies both 32-bit and 64-bit architectures. Existing
`global size_of usize == 4;` or `== 8;` declarations select one width. Native
compilation also checks that the declaration matches the actual Rust target.
The explicit-width regression probes test verification with `--no-erasure-check`;
portable examples retain erasure checks and run on the host. For example:

```rust
#[verus_infer]
fn advance(index: usize, count: usize) -> usize { index + count }
```

Its contract requires the mathematical sum to lie in `0..=usize::MAX` and ensures
that the result equals that sum. `usize::MAX` remains symbolic in formulas,
candidate caches, and exported interfaces. Casts to `usize` truncate at the modeled
width: `4294967297u64 as usize` is `1` on 32-bit and `4294967297` on 64-bit.
Both platform types work as flat enum payloads, array/slice elements and scalar
results. Signed division excludes zero divisors and `isize::MIN` with `-1`, with
Rust's truncation semantics. The seven checked methods listed below also support
both types through existing vstd models. Counted loops remain unsigned, including
`usize`; signed counted loops require further work. See
`examples/verus_infer_usize.rs` and `examples/verus_infer_platform.rs`.

For ordinary arithmetic, the safety policy excludes overflow, underflow, and
division/remainder by zero, consistently with Verus's existing executable
arithmetic checks. This policy applies even when a release Rust executable would
wrap on overflow. Inference adds no runtime checks, changes no executable body,
and imposes no verification overhead on the compiled program. Unverified Rust
callers are responsible for respecting inferred preconditions.

Signed arithmetic excludes results below `MIN` or above `MAX`; negation excludes
`MIN`. Signed `/` rounds toward zero, and `%` retains the dividend's sign. Both
operations exclude a zero divisor and the pair `(MIN, -1)`, including remainder
even though its mathematical result would be zero. For example:

```rust
#[verus_infer]
fn reverse(delta: i64) -> i64 { -delta }

#[verus_infer]
fn quotient(total: i64, count: i64) -> i64 { total / count }
```

`reverse` requires `delta != i64::MIN`. `quotient` requires `count != 0` and
`total != i64::MIN || count != -1`; it returns `-2` for `(-7, 3)`. Inferred
formulas display Rust division/remainder as `rust_div`/`rust_rem`. They use
Verus's existing inline vstd definitions, whose declaration and artifact
fingerprints are recorded even though the source contains an operator rather
than an explicit call. Import `vstd::prelude::*` when using signed `/` or `%`;
missing models produce an unsupported diagnostic. Signed casts preserve Rust's
truncation, sign extension, and reinterpretation behavior. No new standard
specifications or axioms were added.

See `examples/verus_infer_signed.rs` for a runnable example covering negative
division, overflow-safe adjustments, conservation, and signed casts. Signed
values are also supported in flat enum payloads and fixed arrays in exact mode.
Sound interval mode supports fixed-width and platform-sized unsigned/signed
scalar integers and Booleans, including signed intermediate expressions.
Platform-sized analysis computes independent 32-bit and 64-bit ranges and guards
differing results with `usize::BITS == 32`. For example, `x: usize` divided by
`10` yields `0..=429496729` on 32-bit and `0..=1844674407370955161` on 64-bit.
The matching safety predicates and ranges are preserved through composition,
caches, and imported interfaces. Mandatory soundness proves these claims;
optional precision checks can still report that exact output is not established.

Exact mode additionally supports core `Option<T>` and `Result<T, E>` as inputs,
locals, and return values. Each payload must be a supported scalar or `()`;
nested enums and user-defined enums are not supported. Constructors and simple
exhaustive `match` expressions may use by-value bindings, scalar literal
patterns, unit patterns, and wildcards. Arms retain Rust's first-match priority,
and only the selected arm's operations contribute safety conditions. Match
guards and or-patterns are supported. Guards execute only after their pattern
matches and earlier arms fail; their arithmetic and inferred calls retain safety
requirements. Or-pattern alternatives must bind the same immutable by-value
names and types. Verus currently rejects combining an or-pattern with a guard in
one arm. Reference patterns and mutable bindings remain unsupported.
Enum expressions currently require exact mode; sound interval inference remains
scalar-only.

The fixed-width and platform-sized unsigned and signed methods `checked_add`, `checked_sub`, `checked_mul`,
`checked_div`, `checked_div_euclid`, `checked_rem`, and `checked_rem_euclid` are
also supported in exact mode. Inference recognizes their core identities and
existing vstd specifications, rather than accepting arbitrary methods with the
same name. An unsuccessful operation returns `None`; it does not impose an
overflow or zero-divisor precondition. Evaluating the arguments must still be
safe. The generated postcondition describes both the enum variant and its
payload exactly.

For signed values, `checked_div`/`checked_rem` use Rust's rounding/sign rules;
`checked_div_euclid`/`checked_rem_euclid` use Euclidean division with a nonnegative
remainder. All four return `None` for a zero divisor or `(MIN, -1)`. Mixed-sign
methods are also supported: unsigned receivers accept `checked_add_signed` and
`checked_sub_signed`; signed receivers accept `checked_add_unsigned` and
`checked_sub_unsigned`, using the corresponding signed/unsigned width. Both
signed and unsigned types support `checked_neg`: signed `MIN` returns `None`,
and unsigned negation returns `Some(0)` only for zero. These checked methods
remain exact-mode enum operations.

For example:

```rust
#[verus_infer]
fn checked_debit(balance: u64, amount: u64) -> Option<u64> {
    balance.checked_sub(amount)
}

#[verus_infer]
fn validated_debit(balance: u64, amount: u64) -> Result<u64, ()> {
    match checked_debit(balance, amount) {
        Some(remaining) => Ok(remaining),
        None => Err(()),
    }
}
```

Both functions admit every pair of `u64` inputs. A successful result contains
`balance - amount`; the error variant characterizes insufficient funds. The
standalone `examples/verus_infer_enums.rs` composes checked debit, a fee, and
validation, then proves conservation for successful transfers and runs concrete
success, insufficient-funds, overflow, and zero cases. Run it with the same
`--compile` workflow shown above.

Exact mode supports explicit early `return` values in blocks, branches, match
arms, and expressions using the supported types. For example:

```rust
#[verus_infer]
fn charge(balance: u64, amount: u64, fee: u64) -> Result<u64, u8> {
    let total = match amount.checked_add(fee) {
        None => return Err(1),
        Some(total) => total,
    };
    if total > balance { return Err(2); }
    Ok(balance - total)
}
```

This function safely accepts every input and characterizes both errors and the
successful result. Returns stop evaluation: operations after an exit contribute
no safety conditions, while operations evaluated before it still do. This also
applies to left-to-right operand/array-element evaluation and short-circuit
branches. Guarded return values and the continuing path are synthesized as pure
conditional formulas; the original executable and its return checks stay intact.
All three independent proof obligations are still mandatory. Eligibility checks
still inspect the whole source, so an early return cannot hide unsupported
syntax or proof assumptions. Early returns and richer patterns currently require exact mode.
`let-else` supports the same flat patterns, with a returning else branch:

```rust
#[verus_infer]
fn bump(input: Option<u8>) -> Option<u8> {
    let Some(value) = input else { return None; };
    Some(value + 1)
}
```

The initializer is evaluated once. Failed matching executes only the else
branch; successful matching introduces bindings for the continuing block. The
example requires `value < 255` only for `Some(value)`. Safety operations in the
else branch also remain enforced. Unsupported code is rejected even on a path
that always returns or a pattern that never matches.

Exact mode also supports `?` on the supported flat `Option` and `Result` types:

```rust
#[verus_infer]
fn charge_checked(balance: u64, amount: u64, fee: u64) -> Option<u64> {
    let total = amount.checked_add(fee)?;
    Some(balance.checked_sub(total)?)
}

#[verus_infer]
fn add_fee(input: Result<u8, u16>, fee: u8) -> Result<u8, u16> {
    Ok(input? + fee)
}
```

`charge_checked` safely accepts every input, returning `None` on overflow or
insufficient funds. `add_fee` propagates the original `Err` payload, and requires
that addition fit in `u8` only when the input is `Ok`. The operand is evaluated
once, before deciding whether to exit; operations evaluated before `?` retain
their safety conditions. Nested propagation, branches, early returns, and
acyclic inferred calls compose normally, including through caches and crates.
Result errors may retain their type or use the existing lossless primitive
integer `From` conversions. For example, `Result<u8, u8>` can propagate into
`Result<u8, u16>` without restricting error values. The closed set is:

- Unsigned widening `u8/u16/u32/u64` to larger fixed unsigned types.
- Signed widening `i8/i16/i32/i64` to larger fixed signed types.
- Unsigned to strictly wider fixed signed types.
- `u8/u16` to `usize`, `u8` to `isize`, and `i8/i16` to `isize`.

These are 35 concrete conversions backed by the existing vstd models, valid
under both pointer widths. Success payload types may change between the operand
and enclosing function. Arbitrary user-defined `From`, narrowing conversions,
custom `Try`, try blocks, nested enums, and sound-mode enum inference remain
unsupported.

Propagation uses existing vstd `branch` and `from_residual` specifications.
Its internal `ControlFlow` and `Infallible` values are recognized only inside the
complete lowering; their presence does not permit general generic calls or proof
statements. Result's identity conversion relies on Rust's existing blanket
`From<T> for T` semantics and vstd's trusted identity axiom for unchanged error
types, or existing primitive `FromSpecImpl` specifications for approved widening. Its generic residual
model has no `no_unwind` clause; inference preserves that limitation. In
particular, a wrapper that can propagate an `Err` cannot establish `no_unwind`
from this model. No new axioms or stronger standard specifications were added for `?`.

Exact mode also supports immutable, flat fixed-size arrays `[T; N]` with
`0 <= N <= 64`, where `T` is `bool`, `usize`, `isize`, or a supported fixed-width unsigned/signed integer.
Arrays may be passed by value or through `&[T; N]`, copied or borrowed into
immutable locals, selected by branches, or constructed with element-list
literals such as `[x, x + 1]` and repetition literals such as `[x; 4]`. Owned
fixed scalar arrays may also be returned and composed across inferred calls,
caches, and crate interfaces. Array payloads inside enums remain unsupported.
Array results are described as whole values. Exactness quantifies over an
arbitrary valid array and compares all elements using Verus's existing
extensional equality, including lengths zero and 64. The existing vstd
`axiom_array_ext_equal` and its containing artifact are recorded as a trusted
model dependency; no new equality axiom is introduced.

```rust
#[verus_infer]
fn fee_at(fees: &[u64; 4], tier: usize) -> u64 {
    fees[tier]
}

#[verus_infer]
fn checked_fee(fees: &[u64; 4], tier: usize) -> Option<u64> {
    if tier < fees.len() { Some(fee_at(fees, tier)) } else { None }
}
```

The first function infers `tier < 4` (also printing the redundant lower bound
`0 <= tier`) and an exact result equal to `fees[tier]`. The guarded function
admits every `usize` input and describes both `Some(fees[tier])` and `None`
exactly. Branch and short-circuit guards restrict bounds checks to reads that
execute. Every element of an array literal is evaluated, so an overflowing
element still requires a safety precondition even when another element is read.
An unguarded read from an empty array has no safe input and receives an
unsatisfiable precondition; guarded functions can still use empty arrays safely.
Repetition evaluates its operand once, including when the length is zero. Thus
`[x + 1; 0].len()` still requires that `x + 1` not overflow. `.len()` preserves
the safety conditions of its receiver even though the result is a known constant.

`usize` inputs, locals, arithmetic, comparisons, results, and casts to/from
supported integer types are available for index calculations. Expressions such as
`values[index + 1]` retain both arithmetic overflow and indexing bounds checks.
Array indexing uses Verus's existing array model; its normal bounds
assertions independently check the inferred safe domain. Shared references use
Verus's existing immutable value representation. Length, array-to-slice coercion,
and repetition use a closed allowlist of existing core/vstd models, specialized
only to supported scalar elements and fixed lengths. The two pure vstd array
helpers additionally declare `no_unwind`, and inference requires this guarantee
on approved executable array models. There are no new array axioms.

This array subset excludes nested arrays, mutable references, reference-valued
elements, array patterns, and mutation. Shared dynamic slices are supported as
described below.
Signatures and local bindings support one shared reference to an array; compiler
reborrows are handled transparently. Array-to-slice conversion also supports
forwarding these arrays to inferred slice helpers. Sound interval mode does not yet support
arrays or shared-array references.

`examples/verus_infer_arrays.rs` composes array lookup, addition, and checked
arithmetic, proves a pricing property, and runs success, overflow, and invalid-tier
cases. It also constructs a repeated fee schedule and passes it through a borrowed
interface. It can be verified, compiled, and run with the workflow above.

Exact mode additionally supports read-only dynamic slices `&[T]`, with the same
scalar element types. Their runtime lengths stay symbolic, so the fixed-array
limit of 64 elements does not apply to slice inputs. For example:

```rust
#[verus_infer]
fn at(values: &[u8], index: usize) -> u8 { values[index] }

#[verus_infer]
fn get(values: &[u8], index: usize) -> Option<u8> {
    if index < values.len() { Some(at(values, index)) } else { None }
}
```

`at` requires `0 <= index < values.len()` and returns exactly `values[index]`.
`get` admits every index and describes `Some(values[index])` or `None` exactly.
The ordinary executable supplies the independent bounds checks. Guarded reads,
short-circuit conditions, length comparisons between slices, immutable aliases,
shared reborrows, and selection between slice parameters compose normally.
Scalar and flat `Option`/`Result` returns, early returns, checked arithmetic,
and `?` retain their existing support and restrictions. Arithmetic on read
elements adds its usual overflow/division requirements.

Inferred callers can pass shared slices or coerce supported fixed arrays into
slices. Array coercion retains the exact array term, specializing subsequent
lengths and reads. Receiver evaluation remains eager: `get(&[x + 1; 0], 0)`
still requires that `x + 1` not overflow, even though `get` returns `None`.
Ordinary callers can pass arrays larger than 64 elements to slice interfaces;
inferred bodies that construct arrays retain the fixed-array policy above.

Slice summaries use existing Verus length/index operations and existing vstd
specifications. The canonical `spec_slice_len` and `spec_slice_index`
declarations and their containing artifacts are recorded as implicit model
dependencies when used. The executable `.len()` model is recognized by the
same closed identity/signature checks as before. No slice axioms or stronger
library specifications were added. Cached candidates are retyped and reproved;
cross-crate interfaces and JSON reports preserve all three exact-mode guarantees.

Local immutable subslices support `&s[start..end]`, `&s[start..]`, `&s[..end]`,
and `&s[..]` for shared scalar slice inputs and aliases. Inferred conditions
include `start <= end <= s.len()` and any subsequent index bound relative to
the view. Empty views are valid; reading one still requires a valid index.
Nested views and passing views to inferred slice callees are supported. View
lengths reduce to `end - start`; reads reduce to the original base plus the
accumulated offset. Range operands retain Rust's evaluation order and all
preceding safety checks. Views are local synthesis data and cannot occur in
installed contracts or cached formulas. Each contract refers to original inputs.

Subslice calls recognize the complete concrete core `Index` lowering for these
four ranges. Ordinary Verus checks its existing standard preconditions. In
maximality queries, only validated canonical shared-range calls may use their
existing exact bounds requirements to select successful executions; generic
types and actual arguments are instantiated. This extends the prior inferred
callee path without bypassing ordinary checks. Their vstd artifact is
fingerprinted, including range, indexing, and sequence specifications.

This subset excludes mutable slices, returned slice references, direct slicing
of fixed arrays, inclusive ranges, stored range objects, reference-valued
elements, general generic slice methods, slice traversal loops, and sound-mode
slice inference. Conditional selection involving a newly constructed local view
is also unsupported. Conditional coercions
mixing arrays of different fixed lengths, or mixing an array term and a dynamic
slice term, report an unsupported conditional; selection between dynamic slice
inputs is supported even when their runtime lengths differ.

`examples/verus_infer_slices.rs` proves lookup and pricing properties, composes
checked arithmetic and `?`, and runs empty, invalid-index, overflow, signed-edge,
and 300-element cases. The three-crate Cargo workspace also composes a slice
lookup and charge through exported interfaces. The broader
`examples/verus_infer_coverage.rs` combines richer patterns, widened errors,
array outputs, and subslices, with a three-crate Cargo example as well.

## Counted numeric loops

Exact mode supports unsigned accumulator loops over two or three immutable
same-width inputs (`usize`, `u8`, `u16`, `u32`, `u64`, or `u128`):

```rust
#[verus_infer]
fn subtract_steps(x: u64, n: u64, step: u64) -> u64 {
    let mut result = x;
    let mut remaining = n;
    while remaining > 0 {
        result -= step;
        remaining -= 1;
    }
    result
}
```

This infers mathematical `requires n * step <= x` and
`ensures result == x - n * step`. Addition infers
`requires x + n * step <= MAX` and `ensures result == x + n * step`.
The product is unbounded mathematical arithmetic. Zero steps/counts admit the
full input range; there is no small-count cap or incomplete unrolling.

The counter may instead start at zero and use `while i < n` with `i += 1`.
Either counter direction supports accumulator addition or subtraction. The step
may be an unsigned literal or immutable same-width input (including `x` or `n`);
`usize::MAX` is supported symbolically. Unit steps retain simpler `x + n` /
`x - n` contracts. All parameters must be used. Local names and parameter order
may differ, and the two declarations may be reversed. Updates may use `+=`/`-=`
or `v = v + operand` / `v = v - operand`. The accumulator update must precede the
unit counter update; the tail expression returns the accumulator.

The whole typed body is checked. Other guards/counter steps, extra statements,
dynamic step expressions, signed loops, nested/multiple loops, breaks/continues,
handwritten loop annotations, and general mutation remain unsupported. Loop
inference requires exact mode. `assume_termination` and
`exec_allows_no_decreases_clause` settings are rejected. Loop-free inferred
wrappers can compose the exact summaries.

The prefix invariants are `counter <= n` and
`result == x ± completed * step`. For ascending counters, `completed = counter`
and the measure is `n - counter`; for countdown counters, `completed = n - counter`
and the measure is `counter`. Loop isolation is disabled so ordinary soundness
can use its precondition without including that predicate in the invariants.
Verus checks invariant entry/preservation, arithmetic safety, termination, and
the result relation. Parameter products and symbolic platform constants select
Verus's existing nonlinear prover. Literal-step loops do not require it.
Neither selection adds an assumption or trusted helper specification.

Maximality removes the candidate precondition and checks the prefix invariants
and termination independently. At a successful exit, type bounds establish the
safe domain. Exactness retains the arbitrary-output query. All three required
outcomes gate dependent callers and export.

A production projection check removes exactly the generated invariants, measure,
and isolation setting and compares every remaining serialized typed-body field
with source. Annotation terms must belong to the restricted pure summary
language. Candidate identities bind source, installed annotations, actual
contract, and nonlinear-prover setting. Cache hits recheck eligibility and
regenerate proof settings; caches contain no bodies, annotations, or outcomes.
See `VERUS_INFER_LOOPS.md` for the design and fault-injection strategy.

`examples/verus_infer_loops.rs` demonstrates all four parameter-step families,
constant/zero steps, composition, and runtime boundary cases. The Cargo workspace
composes stepped addition and countdown subtraction across three crates.

## Bitwise operations and explicit arithmetic policies

Exact mode supports `&`, `|`, `^`, and integer `!` on every supported integer
type, including `usize`/`isize`. Operands retain their finite signedness and width
through arithmetic expressions, enum payloads, cache decoding, and imported calls.
Signed right shifts extend the sign bit. Left shifts discard high bits as Rust
does. Normal `<<` and `>>` require `0 <= amount < bit_width`; a zero left operand
does not remove that requirement. The amount may use any supported integer type.
No new bitwise axioms are added; ordinary Verus checks the original operations.

Both signed and unsigned integers support `wrapping_add`, `wrapping_sub`,
`wrapping_mul`, `saturating_add`, `saturating_sub`, and `saturating_mul` in exact
mode. Wrapping returns the representable modular result; saturation clamps the
mathematical result to the receiver's `MIN..=MAX`. Their total semantics admit
all inputs, while preserving safety checks of argument expressions. They do not
change the overflow policy of ordinary `+`, `-`, or `*`.

Sound scalar mode uses bounded numeric transfers: `x & 15` has range `0..=15`,
and `(x & 15) >> 1` has range `0..=7`, across signed/unsigned supported widths.
Branch guards can tighten shift operands. Wrapping casts and wrapping/saturating
methods retain useful ranges when the input bounds allow it; otherwise they widen.
Verus checks auxiliary conditional range lemmas with its existing bit-vector
prover and uses that identical theorem in the ordinary proof. Operand ranges and
finite type bounds are premises: the ordinary proof must establish them before
using the result. These hints add no axiom and change no executable expression.
Auxiliary failures/timeouts belong to the function's obligation and cannot be
hidden by another successful subquery. Bounded automation may still fail to prove
a valid candidate; it never converts missing evidence into a verified contract.
Compositional safety retains shift-amount checks and guarded arithmetic. Lost
relations or insufficient range information can still produce an incomplete result.
Optional precision failures retain honest labels and never replace soundness.

The new checked negation, unsigned `checked_sub_signed`, and signed saturating
specifications are explicit trusted vstd additions, audited in the repository's
`VERUS_INFER_ASSURANCE.md`. Other supported numeric methods reuse existing models.
Models require canonical core identities, matching receiver/argument/return types,
a vstd proxy, empty preconditions, and `no_unwind`; their declarations and entire
artifacts are fingerprinted. Rebuild vstd when using the added models.

This batch does not add rotate/count methods, wrapping/saturating mixed-sign
methods, explicit wrapping shifts, or general user-defined integer models.
`examples/verus_infer_integer_ops.rs` verifies caller properties and runs wrapping,
saturation, bitwise, shift, and checked-result boundary examples.

## Application kernels and evaluation

`examples/verus_infer_applications.rs` contains fifteen pure Rust kernels for
big-endian parsing, protocol fields, slice windows, checked purchases, price
pairs, capped fees, and signed wire codes. It includes ordinary caller proofs
and a native boundary runner. The corpus adds wrong caller properties, invalid
windows, an unsupported iterator checksum, and an intentionally incomplete
bit-vector proof. These examples measure selected functionality within the
supported subset.

From the repository root, after completing the build:

```sh
python3 source/tools/eval_verus_infer.py --case applications \
  --repetitions 3 --memory --output-dir /tmp/infer-app-eval
python3 source/tools/eval_verus_infer_cargo.py \
  --repetitions 3 --memory --output-dir /tmp/infer-cargo-eval
```

The first runner always rechecks proofs, including warm candidate hits. A
no-edit Cargo run may skip verification through Cargo's normal incremental
build cache; the Cargo report distinguishes that from fresh proof work. Its
contract edit must reject a stale caller and pass after its property is updated.
Both runners detect changed/missing compiler or model artifacts and abort rather
than mixing builds. Memory is the OS waited-child maximum RSS, measured in a
fresh worker, not a simultaneous process-tree peak. See
`source/tools/infer_eval/README.md` for the measurement definitions and limits.

## Analysis budgets and other restrictions

The synthesizer merges branch results as conditional expressions instead of
enumerating paths. It bounds both source traversal and expanded formula size at
4,096 nodes, and bounds dependency traversal depth at 128 functions. Solver checks use Verus's `--rlimit` and per-function
`#[verifier::rlimit(...)]` / `#[verus_verify(rlimit(...))]` settings. A resource
limit is reported as incomplete, separately from a counterexample or unsupported
syntax.

Static, nongeneric calls to other inferred functions in the same crate or to
certified inferred dependency interfaces are supported, including forward
references, shared helpers, conditional calls, and
short-circuit guards. Dependencies must form an acyclic graph. Callee summaries
are substituted without analyzing their executable bodies again.

Unsupported features include general loops (the counted pattern below is supported),
recursion, converting `Result` errors with `?`, calls to functions
without inferred contracts other than the approved standard models,
integer operations outside the allowlist above,
references beyond shared scalar arrays/slices, mutable state, collections beyond the supported arrays/slices, generics,
and ghost/proof statements.
An ordinary verified caller may compose inferred functions at any supported depth.
Handwritten contracts cannot be combined with inference yet; they receive an
explicit diagnostic and are never silently overwritten.

A versioned registry tracks candidates and their transitive dependencies. Set
`VERUS_INFER_CACHE` to a directory to persist candidate formulas across runs:

```sh
VERUS_INFER_CACHE=/tmp/verus-infer-cache source/target-verus/release/verus examples/verus_infer.rs
```

Cache keys include the typed source, signature, function attributes, inference
schema/safety policy, verifier arguments, transitive callee keys, approved
standard-model declarations, and fingerprints of their containing artifacts.
Changes to supporting specifications and axioms also invalidate candidates.
Entries with unchanged keys remain
reusable; source-location changes can conservatively cause additional misses.
Malformed, oversized, stale, or incompatible entries are treated as misses.
Cache I/O is best effort and atomic. Cached preconditions and general postconditions use a restricted language;
types, parameter/result identities, and diagnostic spans are rebuilt from current
source. Requested precision is part of candidate identity, and cache validation
cannot turn a sound-mode interface into an exact-mode one.
Source eligibility is checked on every run. **Proof results are never cached:**
every reused candidate must pass the checks required by its requested mode before
dependent verification. Optional probes are rerun too; their outcomes are not
recovered from candidate cache data.
Fresh and cached formulas use one installer that preserves the executable body
and signature. Its candidate identity covers the original typed function, actual
contract formulas, configuration, and transitive dependencies. Query evidence is
bound to that identity and the obligation kind; duplicate or mismatched records
invalidate acceptance rather than overwriting an earlier outcome.
Old content-addressed entries remain on disk; the directory may be deleted to
clear the cache. Without the environment variable, storage stays in memory.

Source eligibility and executable arithmetic policies are shared between cache
reuse and fresh synthesis. Cache decoding and imported-summary substitution use
the same typed summary builders, which reject unsupported operators, inconsistent
operand types, and invalid enum projections. Summary traversal is bounded even
when constant conditions simplify a large input formula to a small result.

## Cross-crate inference and Cargo

Public inferred functions use Verus's existing `.vir` export/import mechanism.
A dependency can infer its public interface, another crate can infer a wrapper
around that interface, and an application can prove an explicit property using
the resulting summary. Private functions and executable bodies are pruned using
the existing library-export rules; importers need only the inferred contracts.
Exact contracts still describe implementation behavior, including behavior
computed by private helpers.

The three-crate example in `examples/verus_infer_workspace` demonstrates this:

```sh
# From the repository root, after building Verus and cargo-verus:
export PATH="$PWD/source/target-verus/release:$PATH"
export VERUS_Z3_PATH="$PWD/source/z3"
cargo verus build --manifest-path examples/verus_infer_workspace/Cargo.toml
cargo verus verify --manifest-path examples/verus_infer_workspace/Cargo.toml
```

`base` exports an inferred increment implemented through a private helper;
`middle` infers a function that calls that dependency twice; `app` infers one
more wrapper and proves that its result is the original input plus two.
The same workspace also exports sound-only fee ranges, composes them through
sound-mode wrappers, and proves their caller ranges for the full input types.
It also exports checked `debit` and exact fee interfaces from `base`, infers a
`Result`-returning validation function in `middle`, and proves successful-transfer
conservation in `app` using its inferred wrapper.
An array-based fee lookup additionally composes through all three crates; the
application proves the exact total under the inferred index and overflow bounds.
No dependency implementation is inlined into a consumer proof.

For direct compiler invocations, use the normal `--export library.vir` and
`--import library=library.vir` options alongside matching Rust libraries supplied
with `--extern`. Supply the transitive inferred `.vir` dependencies as well;
`cargo verus` resolves these automatically.

For a crate with inferred definitions, export is deferred until full-crate
verification, including each function's required inference checks, has succeeded. Publication
uses an atomic file replacement. A re-verification invalidates an existing
certified inference export before checking dependencies or local proofs, so a
failed verification cannot leave that old artifact available at the output path.
`--no-verify` and partial verification filters are rejected for local inferred
definitions.

Inference exports carry a versioned proof-metadata trailer recording soundness,
maximality, exactness, the safety policy, producer version/toolchain/revision,
contract fingerprints, per-function proof outcomes, and imported inferred-dependency
fingerprints. Approved standard-model dependencies additionally record fingerprints
of both their declarations and containing vstd artifacts; imports reject stale
or missing models, including changes to supporting spec functions and axioms. The current
schema supports mixed exact/sound libraries: a sound-only
function does not downgrade an unrelated exact interface. Exact-mode exports must
have all three guarantees, while sound-mode exports must have proved soundness;
missing precision guarantees are preserved rather than invented. Import
rejects missing, incompatible, incomplete, conflicting, or stale metadata with
a rebuild diagnostic. Changes to imported summaries also invalidate dependent
local candidate-cache entries. Cargo's existing `.vir` dependency tracking
triggers rebuilds of consumers. Fingerprints conservatively include diagnostic
locations and all imported inferred interfaces, so some semantically unchanged
rebuilds may also invalidate dependents.

The current inference policy uses metadata schema 14 and candidate-cache version 19.
Loop candidate identity version 3 also binds the selected nonlinear-prover setting.
Cache identities include the merged architecture of the crate and its imports;
exported contract fingerprints also bind that architecture. A library proved
for a fixed width constrains its consumers to that width; incompatible fixed
widths are rejected by Verus's existing architecture merge. A portable library
can be consumed at either width. All cached candidates still receive fresh proofs.
Rebuild inferred libraries and their dependents with the updated compiler.
The internal candidate-identity field introduced with schema 6 changed serialized
VIR; libraries built before that change, including `vstd`, also need rebuilding.
Ordinary exports still omit inference proof trailers.

Ordinary libraries keep their existing export representation. Imported proof
metadata uses Verus's normal trusted-library model: it records completed proofs;
it is not a portable proof object or a cryptographic signature. Dependency bodies
are not reverified on import. Local inferred wrappers receive fresh proofs using
the imported guarantees. The candidate cache remains untrusted and stores no
proof results.

Annotation is opt-in; functions without the attribute retain normal Verus behavior.

## Implementation and trust

Candidate synthesis operates on typed VIR after mode checking and erasure-mode
collection, before simplification and SST lowering. Keeping generated specs out
of source-based erasure analysis preserves executable parameter/local modes.
The executable body remains intact. The provisional `requires` and `ensures`
fields serve as the in-memory contract store and become exported dependency
interfaces only after successful verification.

The existing `dual_spec` macro was evaluated as a starting point. It copies
executable logic to a spec function, but still needs explicit contracts and does
not synthesize a safety domain. VIR provides explicit arithmetic policies and
typed branches, so the prototype implements a separate synthesis pass and uses
the existing verifier for checking and modular consumption.

The inference passes are untrusted: an unsound candidate must fail checking.
A deliberately imprecise candidate may fail optional precision probes while
retaining its proved sound guarantees.
Soundness uses the ordinary verification query. Precision checks operate on the
original executable's verifier-generated SST:

1. **Maximality:** remove the candidate precondition from the query assumptions,
   replace built-in safety assertions with safety assumptions, and prove the
   candidate precondition at every normal return. This checks `Safe => P` using
   the verifier's own safety conditions. At a call, the already-verified callee
   precondition selects successful executions and its exact postcondition
   describes the result. The ordinary soundness query still
   proves all safety assertions under `P`.
2. **Exactness:** introduce an arbitrary output `y` and prove
   `Q(inputs, y) <==> y == actual_return` using the original executable body under
   `P`. The arbitrary output is constrained to valid Rust values, including the
   enum's payload types. This rejects overly weak output relations even when ordinary contract
   verification accepts them.

These checks rely on the existing Verus/rustc arithmetic and array models and Z3, plus the
small precision-check transformation as part of the verifier. The supported
subset excludes general effects and source-level assumptions. Counted loops
allow only the recognized two-local mutation pattern and checked proof
decoration described above. Calls use inferred
summaries or the explicitly approved standard models, including the array helpers
and fixed-array length lowering. Those methods rely on trusted vstd specifications, just as ordinary Verus
verification does. The integer-operation batch additionally introduces 36 trusted
primitive-method declarations, explicitly audited in `VERUS_INFER_ASSURANCE.md`;
these specifications are assumptions about Rust library behavior. The driver names these dependencies in its diagnostics and
preserves their fingerprints through caches and crate exports.
No inferred executable body is externalized or assumed, and no array axioms are
added. The two existing external-body array helpers declare `no_unwind`; those
and the numeric specifications above are explicit trusted-specification additions.
The precision transformations and proof-publication machinery are also part of
the added trust boundary and have not themselves been formally verified.
The driver validates inferred functions in separate buckets before starting any
dependent inferred bucket or ordinary caller bucket, including when ordinary
callers are verified in parallel. Exact-mode caller precision therefore depends
on exact-mode callee guarantees; sound-mode callers never assume unproved
maximality from their dependencies.
The ordinary and precision bodies must match their query kinds. Only an original
ordinary query can record soundness, and only a matching precision query can
record its precision guarantee. Diagnostic/profile reruns do not supply evidence.
The acceptance component requires every required subquery to succeed; missing,
failed, skipped, and timed-out results cannot be repaired by later success.
Ordinary parallel workers cannot replace previously recorded inference outcomes.

Inference describes implementation behavior. Application intent, such as value
conservation, still belongs in explicit caller and system specifications.

The repository-root `VERUS_INFER_ASSURANCE.md` tracks current safeguards,
near-term consolidation, and longer-term formal verification targets.

## Regression tests

From `source/`, after building the compiler and builtin libraries:

```sh
cargo test --release -p vir infer
cargo test --release -p rust_verify_test --test infer
```

The tests exhaustively compare `u8` arithmetic, branching, and short-circuiting
summaries against concrete Rust execution. Backend fault-injection tests submit
incorrect candidates directly to the production precision checker and Z3:
overpermissive and restrictive domains, incorrect outputs, and sound but
imprecise output relations. Integration tests cover caller composition, numeric
boundaries, unsupported constructs, bounded formulas, and solver-limit
publication failures. Cross-crate tests cover a three-library chain, stale
dependency fingerprints, missing/incompatible/incomplete proof metadata, private
helper pruning, and failed/partial-verification export suppression. The Cargo
regression `cargo_inferred_contracts_across_three_crates` builds and verifies the
example workspace through `cargo verus`. Partial-inference tests additionally
check exhaustive interval containment for safe `u8` executions, useful range
proofs, bounded safety synthesis after exact formula-budget exhaustion,
independent precision timeouts, compositional domains, fresh cache proofs, and
per-function imported precision metadata.
Exhaustive testing supplements the solver proofs.
Structured-report regressions additionally check fresh proofs on cache hits,
honest optional timeouts and skips, failed caller export suppression, imported
candidate/model identities, and rejection of a corrupted cached precondition
with removal of the previous export.
Slice regressions compare generated domains and outputs with concrete Rust
reads/checked arithmetic over small vectors and every `u8`/`i8` element value,
including empty and 300-element inputs. They also cover caller bounds and
overflow rejection, symbolic lengths, aliases and guards, typed cache corruption,
restrictive domains failing maximality, three-crate composition, stale model
artifacts, and failed-proof export removal.
Enum regressions additionally compare checked `u8` arithmetic against concrete
Rust for every input pair, check supported widths and match-arm safety, compose
enum interfaces through cached cross-crate builds, reject stale model
fingerprints, and verify that poisoned cached variants or payloads fail checking.
Hardening tests cover truncated, oversized, and malformed artifacts, duplicate or
missing proof metadata, and a cached restrictive precondition that passes
soundness but fails maximality. That exact-mode failure blocks both inferred and
ordinary dependents and invalidates the previous export, including with parallel
verification enabled.
Array regressions exhaustively compare bounds and overflow predicates for
two-element `u8` arrays against concrete Rust, check empty and 64-element arrays,
cover conditional and short-circuit reads and eager literal evaluation, exercise
array summaries and literal formulas through cached cross-crate builds, and
ensure a cached wrong index fails independent verification.
Read-only array tests additionally cover shared-reference forwarding, cached
cross-crate composition, repetitions of length 0 and 64, zero-length operand
overflow, model lookalikes, and changes to trusted-model artifacts. The latter
must both invalidate candidate keys and block old inference certificates.

Assurance tests also cover body preservation, query separation, candidate/evidence
identity, exact/sound acceptance combinations, receipt ordering, duplicate and
mismatched records, and worker isolation. From the repository root, run:

```sh
python3 source/tools/test_infer_assurance_mutations.py
```

The source mutation runner uses temporary copies, cached Cargo dependencies, the
pinned Rust toolchain, and the production precision checker with Z3. Its
`--policy-only` option tests the dependency-free acceptance component without
Cargo. Each baseline must pass; selected weakened implementations must run tests
and fail them. Mutation coverage is testing evidence, not a formal proof.

Early-return regressions exhaustively compare five control-flow/operand-order
cases over every pair of `u8` inputs with Rust checked operations. Native tests
cover nested conditions, both short-circuit operators, enum matches, unit
payloads, guarded borrowed-array reads, unreachable operations, and retained
pre-exit overflow checks. A cached three-crate transfer chain verifies
conservation; a poisoned precondition must fail maximality at early exits and
suppress old exports.
