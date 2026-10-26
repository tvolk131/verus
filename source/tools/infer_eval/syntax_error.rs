// Intentionally malformed: the runner must not treat a frontend error as a proof.
use vstd::prelude::*;
#[verus_infer]
pub fn malformed( -> u8 { 1 }
