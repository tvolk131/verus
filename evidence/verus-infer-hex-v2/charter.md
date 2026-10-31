# Native string-backed forward iterator milestone

Frozen baseline: native `5ccfbad247d5a332408929173e4072fbf06d8bd1`,
engine `b42a50e3e96e32b9c76b25dd51caaacd5a120c37`.
Hex v1 and Aleph v3/v4 evidence, archives and pinned fixtures remain immutable.

The selected production chain is the real string length-checking constructor,
`str::as_bytes`, `HexDigitsIter::new_unchecked`, `slice::chunks_exact(2)`,
`HexDigitsIter::next/nth`, `HexToBytesIter::from_pairs` and its actual generic
`Iterator::next/nth`. No replacement iterator or assumed first-party behavior
can close acceptance. Additional ordinary contracts express required behavior;
general checked support supplies routine assistance where needed.

For arbitrary Rust `str` s, let B be its original UTF-8 bytes. Construction must
reject odd byte length with exactly that length, before any character decoding.
Otherwise the initial pair cursor is zero and remaining pairs are precisely
`[B[2*i], B[2*i+1]]` for `0 <= i < B.len()/2`.

For an arbitrary forward history, remaining pairs form the corresponding suffix
of the original pairs. `nth(n)` with n below the remaining length consumes n+1
pairs, returns only the selected pair's result using the accepted v1 arithmetic
and error-order theorem, and reports its original-input byte offset (plus one
for a low-byte failure). Skipped invalid pairs produce no error. On out-of-range
n, the result is None, all remaining pairs are consumed, and the native
front_pos stays unchanged because its update follows the early `?` exit.
Subsequent next/nth calls remain exhausted. next is the actual nth(0) delegation.

The implementation's saturating operations must be checked. The generic
decoder's transition uses the exact saturating formula even at arbitrary usize
states. In the string-backed reachable domain, a successful selected pair is
inside B, so its original byte positions and the updated cursor cannot overflow.
Out-of-range usize::MAX requests must be covered without computing n+1 in usize.
No ASCII, nonempty-input or successful-decoding restriction is imposed.

Acceptance requires connected application theorems for construction and arbitrary
forward steps/history, actual native body coverage, retained v1 theorems, unrelated
general-support regressions, source/assumption inventories, semantic mutations
(length, selection/skipping, cursor updates, exhaustion and error locations),
fail-closed publication controls, restored fresh proof/native feature and MSRV
replays, local commits and preserved prior artifacts. Tool/vstd counts are
reported separately from application theorems.

After this milestone, advance the actual public decode_to_vec. Its real
with_capacity/as_mut_ptr/ptr::write/ptr::add/set_len path must prove bounds,
allocation identity, initialized-prefix ownership and safe early-error cleanup.
A safe replacement, assumed drain contract or byte-sequence-only model does not
verify that native memory path. If a required pointer/initialization model is
missing, retain the unchanged path and record a concrete tested blocker as the
next phase. Fixed-array decoding and mixed backward iteration are later scope.
