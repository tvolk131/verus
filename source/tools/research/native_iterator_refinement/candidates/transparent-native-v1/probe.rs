use vstd::prelude::*;
verus! {
// Request actual Rust fields rather than inventing replacement fields.
#[verifier::external_type_specification]
pub struct InspectNativeIterMut<'a, T: 'a>(core::slice::IterMut<'a, T>);
pub proof fn check_body() {}
}
