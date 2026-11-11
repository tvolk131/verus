// SPECIFICATION CORRECTION: implement the existing FromSpecImpl extension.
// The original executable From body must prove this equation; no assumption.
verus! {
impl vstd::std_specs::convert::FromSpecImpl<usize> for NumPeers {
    open spec fn obeys_from_spec() -> bool { true }
    closed spec fn from_spec(value: usize) -> Self { NumPeers(value) }
}
}
