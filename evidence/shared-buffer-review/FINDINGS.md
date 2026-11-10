# Fresh scoped review findings

Reviewed c3bf5d36 (production through 676b99bf); production sources unchanged.

## F1 — P2: skip surrounding buffer mutations or observe the iterator borrow

Location: `source/vir/src/infer/buffer_observation_goals.rs:637-641`;
related eligibility: `native_mutable_for.rs:159-320`, proposals: `buffer_observation_goals.rs:332-360,409-422`.

The installer prepends initial/current and final buffer snapshots to the entire
function. The descriptor permits a write to that buffer before or after the
native loop. Consequently, the generated pointwise invariant equates iterator
referents with a different borrow state. A pre-loop write invalidates the initial
view relation; a post-loop write invalidates the eventual view relation.

Both committed minimal repros verify **2/0 with assistance off**, and both fail
**1/1 with assistance on**, solely `invariant not satisfied before loop`. The
unmodified simple loop verifies 2/0 both ways. Inputs remain arbitrary initialized
exclusive `[u8;10]`; these programs have no preconditions or authored invariants.
This is a valid-program rejection, not unsound acceptance or semantic-negative
evidence. The source executes normally and the checked generated proposal fails.

Reproduce from the checkout root using this review's immutable bundle:

```sh
export VERUS_Z3_PATH=/tmp/shared-buffer-review-c3bf5d36/bundle/z3
/tmp/shared-buffer-review-c3bf5d36/bundle/verus evidence/shared-buffer-review/repros/before-write.rs --crate-type=lib --no-cheating --output-json
/tmp/shared-buffer-review-c3bf5d36/bundle/verus evidence/shared-buffer-review/repros/before-write.rs --crate-type=lib --no-cheating --output-json --infer-goals
# Repeat for after-write.rs.
```

Raw independently frozen sources, invocations and reports:
`/tmp/shared-buffer-review-c3bf5d36/probes/{freeze.json,results.json,*-on.stderr,*-off.stdout}`.

Suggested parent fix: conservatively skip when source buffer uses outside the
recognized borrow cannot be proved compatible with the snapshots, or bind both
snapshots to the actual iterator reborrow lifetime. Moving only the initial
snapshot does not address post-loop writes. Keep ordinary checks and source
projection intact; add regression controls for both surrounding mutations.

No production fix was made in this review checkout.

## F2 — P2: update the existing profile-report regression expectation

Location: `source/rust_verify_test/tests/infer_cases/reporting.rs:178-181`;
production declaration: `source/vir/src/infer/report.rs:6-7`.

The COMPLETE current Infer suite exposes the existing test
`reporting_cases::assistance_explanations_distinguish_disabled_and_skipped_profiles`
failing because its exact profile list still contains only
`growth_loop`, `pointwise_transport`, `range_loop`. The current reporter correctly
adds `mutable_buffer_u64_radix128`. This integration did not update the old test.
It is a stale assertion/CI regression, not evidence of false proof acceptance.

Reproduce from `source/`, with this checkout's matching compiler/macros/vstd and
solver, preserving the normal harness:

```sh
PATH=/home/dev/.cargo/bin:$PATH CARGO_BUILD_JOBS=2 VERUS_IN_VARGO=1 \
RUST_MIN_STACK=33554432 VERUS_Z3_PATH=/tmp/shared-buffer-review-c3bf5d36/z3 \
cargo test --release --locked -p rust_verify_test --test infer \
reporting_cases::assistance_explanations_distinguish_disabled_and_skipped_profiles -- --exact
```

The original full-target log is `/tmp/shared-buffer-review-c3bf5d36/infer.log`.
The full 32-target run continued after this failure. Update the expected public
report inventory and retain its determinism/disabled/skipped assertions. Do not
remove the new profile from production reporting or weaken full-check policies.
No test or production fix was made here.
