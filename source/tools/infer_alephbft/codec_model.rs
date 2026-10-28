//! Explicit trusted SCALE boundary for the FullUnit experiment.
//! The supplied model must describe every encoded field. This library bridge
//! models the native derive's deterministic encoding and #[codec(skip)] policy;
//! it does not verify SCALE's bytes, length prefixing, allocation, or algorithm.
//! For payloads, DeepView must retain every encoded distinction. CloneView alone
//! proves view preservation, not that an arbitrary chosen view is encoding-complete.
use vstd::prelude::*;
verus! {
pub uninterp spec fn encoded_fields<V>(view: V) -> Seq<u8>;

#[verifier::external_body]
pub fn encode_with_view<T: View>(value: &T) -> (bytes: Vec<u8>)
    ensures bytes@ == encoded_fields(value@),
{ unimplemented!() }
}
