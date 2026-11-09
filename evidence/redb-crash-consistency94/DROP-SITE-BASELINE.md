# Small destruction-site capability baseline

The authorized side quest now has a concrete semantic target, separate from a
diagnostic graph. These fixtures are not native redb and close no campaign cell.

`drop_prototype_cases/normal_writeback.rs` implements real Rust Drop that writes7
through an actual borrowed u64. The existing body checker verifies the Drop body
and its effect contract, but parent attempt085 rejects the caller's postcondition:
the current verifier does not propagate that effect at the implicit scope exit.
This is the intended missing-capability baseline, not a passing selected crate.

`explicit_writeback.rs` uses the actual core::mem::drop call; parent086 rejects
that resolved function before semantic verification. This frontend rejection
receives zero semantic-negative credit. Adding an assumed drop specification
would not solve the checked destruction-site effect problem.

The ordinary pinned-rustc fixture performs both operations and checks observed
values7 and9. Its exact compile/run is captured privately in drop-site-baseline-001.
Those two finite checks illustrate Rust behavior, not a universal Verus proof.
`wrong_borrow_frame.rs` is a future negative: writing through the target must not
be interpreted as writing a different borrowed value or preserving the target.
That control is authored but not yet launched or credited.

The first semantic implementation can start with universally checked, unconditional
no-unwind Drop bodies like this one: preserve existing Drop header restrictions,
check every applicable real destruction event, and apply the proved effect to the
correct loan only after supported glue. This tests the event/effect plumbing before
adding contextual domains. It does not solve RawLeafBuilder's incomplete-state or
panic-cleanup requirements. Contextual domains, real panic-state correspondence,
two-run native/ghost mapping and unsupported generic/escape coverage remain the
separately checked steps described by the research design.
