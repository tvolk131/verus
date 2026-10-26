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
values. It handles the same supported scalar expressions, joins branch ranges,
and composes inferred interfaces. When exact symbolic safety extraction fits
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
`(MIN, -1)` through the ordinary executable checks. Casts retain an input range
that fits the destination; when wrapping is possible, analysis conservatively
uses the full destination range. Signed `MIN` literals are recognized before
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

## Supported code and safety policy

Exact mode supports `bool`, fixed-width unsigned integers (`u8`, `u16`, `u32`,
`u64`, `u128`), and fixed-width signed integers (`i8`, `i16`, `i32`, `i64`,
`i128`), immutable parameters and locals, scalar return values, `+`, `-`, `*`,
`/`, `%`, signed negation, comparisons, Boolean operations, signed/unsigned integer
casts, and `if` expressions. Short-circuit conditions guard the safety checks of
the expressions that are actually evaluated. Casts retain Rust's truncation
semantics; inferred arithmetic formulas otherwise use mathematical integers.
As in ordinary Verus proofs, callers may need a `by(bit_vector)` assertion to
derive a concrete value from a narrowing cast outside its destination range.

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
Sound interval mode supports unsigned/signed scalar integers and Booleans,
including signed intermediate expressions.

Exact mode additionally supports core `Option<T>` and `Result<T, E>` as inputs,
locals, and return values. Each payload must be a supported scalar or `()`;
nested enums and user-defined enums are not supported. Constructors and simple
exhaustive `match` expressions may use by-value bindings, scalar literal
patterns, unit patterns, and wildcards. Arms retain Rust's first-match priority,
and only the selected arm's operations contribute safety conditions. Match
guards, or-patterns, reference patterns, and mutable bindings are unsupported.
Enum expressions currently require exact mode; sound interval inference remains
scalar-only.

The fixed-width unsigned and signed methods `checked_add`, `checked_sub`, `checked_mul`,
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
methods such as `checked_add_unsigned`, checked negation, and wrapping/saturating
methods remain outside the current allowlist.

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
syntax or proof assumptions. Early returns currently require exact mode;
`let-else` remains unsupported.

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
Result error types must be identical. Success payload types may change between
the operand and the enclosing function. Arbitrary `From` conversions, custom
`Try`, try blocks, nested enums, and sound-mode enum inference remain unsupported.

Propagation uses existing vstd `branch` and `from_residual` specifications.
Its internal `ControlFlow` and `Infallible` values are recognized only inside the
complete lowering; their presence does not permit general generic calls or proof
statements. Result's identity conversion relies on Rust's existing blanket
`From<T> for T` semantics and vstd's trusted identity axiom. Its generic residual
model has no `no_unwind` clause; inference preserves that limitation. In
particular, a wrapper that can propagate an `Err` cannot establish `no_unwind`
from this model. No new axioms or stronger standard specifications were added for `?`.

Exact mode also supports immutable, flat fixed-size arrays `[T; N]` with
`0 <= N <= 64`, where `T` is `bool` or a supported fixed-width unsigned/signed integer.
Arrays may be passed by value or through `&[T; N]`, copied or borrowed into
immutable locals, selected by branches, or constructed with element-list
literals such as `[x, x + 1]` and repetition literals such as `[x; 4]`. Functions return
the previously supported scalar or enum values; array return values and array
payloads inside enums are not supported.

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

`usize` inputs, locals, integer literals, comparisons, and casts from `usize` to
supported fixed-width unsigned/signed types are available for index calculations.
General `usize` arithmetic, `usize` return values, and casts to `usize` remain
unsupported. Array indexing uses Verus's existing array model; its normal bounds
assertions independently check the inferred safe domain. Shared references use
Verus's existing immutable value representation. Length, array-to-slice coercion,
and repetition use a closed allowlist of existing core/vstd models, specialized
only to supported scalar elements and fixed lengths. The two pure vstd array
helpers additionally declare `no_unwind`, and inference requires this guarantee
on all approved array models. There are no new array axioms.

This array subset excludes nested arrays, mutable references, reference-valued
elements, dynamic slice parameters or locals, array patterns, and mutation.
Signatures and local bindings support one shared reference to an array; compiler
reborrows are handled transparently. The temporary slice conversion used by
`.len()` is permitted only at fixed array types, without enabling general slice
interfaces or other generic calls. Sound interval mode does not yet support
arrays, shared-array references, or `usize` signatures.

`examples/verus_infer_arrays.rs` composes array lookup, addition, and checked
arithmetic, proves a pricing property, and runs success, overflow, and invalid-tier
cases. It also constructs a repeated fee schedule and passes it through a borrowed
interface. It can be verified, compiled, and run with the workflow above.

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

Unsupported features include loops, recursion, converting `Result` errors with `?`, calls to functions
without inferred contracts other than the approved standard models,
general platform-sized arithmetic, integer bitwise operations,
references beyond shared scalar arrays, mutable state, collections beyond the supported arrays, generics,
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

Current inference metadata uses schema 6; compositional safety inference uses candidate
cache version 11. The internal candidate-identity field introduced with schema 6
also changes serialized VIR:
rebuild `vstd`, other `.vir` libraries, and their dependents with the updated
compiler. Ordinary exports still omit inference proof trailers.

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
subset has no loops, effects, or source-level assumptions. Calls use inferred
summaries or the explicitly approved standard models, including the array helpers
and fixed-array length lowering. Those methods
rely on the existing trusted vstd specifications, just as ordinary Verus
verification does. The driver names these dependencies in its diagnostics and
preserves their fingerprints through caches and crate exports.
There are no new array axioms, assumed candidate contracts, or additional
externalized bodies. The two existing external-body array helpers now declare
`no_unwind`; those declarations are additions to the trusted specifications.
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
