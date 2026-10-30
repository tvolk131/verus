# Hex-conservative forward pair decoding, milestone 1

Frozen scope: native baseline `129db6ac54d25b08e0b927c225aefe4a8df05a57`,
engine baseline `3e7fe46401bcc35f9fe781daab4939f368aa616e`.
This is a new adoption experiment, independent of the accepted Aleph v3/v4
fixtures, archives and fixed engines. Those artifacts must remain unchanged.

The first endpoint is the actual internal `hex_chars_to_byte(hi, lo)` in
`src/iter.rs`, including the actual `Char::decode_nibble` and its function-local
static initializer in `src/lib.rs`. The domain is every pair of Rust `u8` values,
including all non-ASCII bytes, with no precondition. Production algorithms,
initializer assignments, discriminants, signatures and iterator bodies remain
intact. An annotation is not proof of the annotated behavior.

## Independent required semantics

Define `digit(b)` by ASCII ranges, independently of the implementation table:
`b - 48` for 48..57; `b - 65 + 10` for 65..70; `b - 97 + 10` for
97..102; invalid otherwise. This accepts upper and lower case identically.
`decode_nibble(b)` must return precisely this optional digit for every byte.

The pair result must be `Err((hi, true))` if `digit(hi)` is invalid, even when
`lo` is invalid too. Otherwise it must be `Err((lo, false))` if `digit(lo)` is
invalid. Otherwise it must be `Ok(16 * digit(hi) + digit(lo))`.
This is an exact forward decoding requirement, not a round-trip requirement.
The pair endpoint consumes exactly two input bytes and produces one byte on
success. It has no variable-length or string argument.

## Acceptance gates

1. Baseline native tests before annotation; record SHA, toolchains, lockfile and
   feature sets without replacing native configuration. Replay native tests on
   the proof toolchain 1.99.1, and baseline default tests on MSRV 1.74.0.
2. Verify the real initializer and both native bodies. Authors may supply the
   range semantics above and honest domains; implementation summaries and
   routine proof assistance should come from general engine support. No assumed
   first-party/table equations, replacement decoder or blanket exclusions.
3. Each inferred helper must pass ordinary body soundness, safe-domain maximality
   and output exactness. Independently check the range/error semantic goals.
   Unsupported constructs remain explicit blockers, not accepted assumptions.
4. Mutations of a table assignment, nibble order, invalid acceptance and error
   priority must fail the semantic goal. A changed implementation may still have
   healthy inference certificates; these do not establish the required behavior.
5. Failed, no-verify and filtered invocations must not publish accepted interfaces.
   New general support requires unrelated regressions, including cross-crate use
   if a new inferred interface profile is introduced. Fresh restored proof and
   matching-toolchain native tests close acceptance.
6. Inventory every source/configuration change and preserve commands, logs,
   hashes, proof scope and limitations. Commit cohesive validated work locally;
   no pushes, PRs, messages or additional agents.

## Inspected extension boundary, outside this milestone's acceptance

`HexToBytesIter::new` checks **UTF-8 byte length**, returning odd length before
examining characters. `new_unchecked` uses `str::as_bytes` and `chunks_exact(2)`.
Invalid non-ASCII input reports an individual UTF-8 byte, not a Unicode scalar;
there is no ASCII-input restriction. The forward iterator computes the pair
position using saturating addition/multiplication and adds one, saturating, for
the low-byte error. Arbitrary `nth` can skip earlier invalid pairs.

`decode_to_vec` is alloc-gated (`std` implies `alloc`). It checks even byte
length, allocates capacity, drains in order using unsafe `ptr::write`/`ptr::add`,
and calls unsafe `set_len` only after complete success. `drain_to_slice` asserts
iterator/slice length equality and also uses unsafe pointer writes. Earlier
slice writes can remain visible on error. Verifying a substitute safe drain
would not verify these native routines.

`decode_to_array<const N>` checks `hex.len() == N * 2` before decoding and
returns `InvalidLength { invalid: hex.len(), expected: 2 * N }` otherwise.
Arithmetic safety requires `N <= usize::MAX / 2`; larger N has Rust's configured
overflow behavior and is not silently admitted to a mathematical length theorem.
Fixed-length rejection precedes invalid-byte rejection. Both public APIs convert
native error variants through actual `From` implementations in `src/error.rs`.

A future public-endpoint milestone must verify iterator state, byte locations,
length checks, error conversions and unsafe memory operations, with successful
output length `input_bytes.len()/2` and every element following the equation
above. This pair milestone does not claim those results or full-crate coverage.
