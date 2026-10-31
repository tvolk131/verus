# Original public vector memory phase: concrete blocker

The substantive forward iterator milestone is accepted and immutable in
[hex v2](../verus-infer-hex-v2/README.md). This next phase is **not accepted**:
no public decode_to_vec or unsafe-memory theorem is claimed.

Native phase commit: `ae20db0` (full hash in the closure receipt).
Accepted native iterator commit: `cc0bf36056a190bd7da4cbf226e778580e8bb472`.
Matching compiler/vstd implementation: `1e0cf81d20d0dcec0f137edc9b95a8d9c2d98d60`.

Implemented opt-in selection of the actual public decode_to_vec, both native
error conversions and the unchanged unsafe drain_to_vec. Independent closed
required predicates specify exact success length/contents and the first invalid
original UTF-8 byte/payload/location, with odd byte length rejected first. The
drain requirement specifies all output bytes or its first failing pair, including
native saturated positions. These are requirements awaiting verification, not
assumed contracts. Their definitions are beside the native bodies so private
error fields stay encapsulated. Production code and its pointer operations are
preserved; there is no safe replacement or assumed first-party drain equation.

The final opt-in probe fails at exactly four unsupported real operations:

- Vec::as_mut_ptr (`src/iter.rs:165`)
- core::ptr::write (`src/iter.rs:170`)
- raw-pointer add (`src/iter.rs:171`)
- Vec::set_len (`src/iter.rs:176`)

It publishes no interface. This frontend failure means no phase3 application
body/certificate success is claimed. Error conversion requirements are also
unaccepted within that failed scope. The four diagnostics establish a tooling
boundary, not an upstream defect in hex-conservative.

The [frozen charter](charter.md) requires actual memory justification. Present
vstd Vec::with_capacity exports only an empty logical sequence; it exports no
allocation identity/capacity or spare-storage permission. Existing
vstd::raw_ptr::ptr_mut_write requires an explicit tracked PointsTo permission;
its external wrapper does not justify the original direct core::ptr::write call.
A substantial general bridge is needed for allocation identity/capacity,
pointer provenance and bounds, an initialized prefix, transfer at set_len and
cleanup after early Err while native Vec length remains zero. Exact runtime
size_hint/len must also be checked. Adding typed or unconditional assume-specs
for the four calls would not discharge those obligations and was not done.

Source/config inventory beyond accepted v2: proof selection/required contracts
on the four native functions and error enum, two closed result predicates and a
pure saturation definition, the proof-only proc_macro_hygiene feature for the
original for-loop macro rewriting, native documentation, and **one** additional
check-cfg allowlist name. Lint strictness, feature defaults, lockfiles and native
algorithms remain intact. The parser audit again preserves all **218 original
signatures and executable bodies**. No native assumes, external bodies, source
proof blocks, proof functions, assertions, loop invariants, manual triggers or
blanket exclusions were introduced.

After the failed opt-in probe, the ordinary v2 configuration passed fresh and
restored proofs (**30 results, zero errors**, six new v2 application goals plus
one retained v1 goal, three exact-pair certificates). Native Rust 1.99.1 tests
passed default/all/no-default/alloc-only; MSRV 1.74 default passed. These results
are iterator/native regression evidence and do not certify the public vector
endpoint. V2 already records the 291 inference regressions, 57 const/static,
17 unwind, 90 type-invariant and 2,215 vstd tooling checks. No compiler/vstd
implementation changes were made during this phase.

Replay from the matching engine root, with a new output directory:

```sh
python3 source/tools/check_hex_vec_phase3.py \
  --native-root /Users/tommyvolk/Documents/GitHub/hex-conservative \
  --output /private/tmp/hex-vec-phase3-replay-new
```

The controller checks all four diagnostics and no publication, then runs the
restored iterator/native feature/MSRV replay without mutating source. The native
phase flag is `--cfg verus_hex_vec_phase3` in a direct Verus invocation.
`final-restored` is the final blocker/restoration series. Earlier raw development
probes include corrected annotation/privacy and check-cfg issues; those are not
upstream defects or accepted proofs. All prior accepted source archives and
Aleph/hex evidence remain unchanged. See the closure receipt, source audit,
source/config patch and raw archive/index in this directory.
