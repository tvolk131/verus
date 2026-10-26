use vstd::prelude::*;
verus! {
    #[verifier::infer(sound)]
    #[verifier::rlimit(0.000001)]
    pub fn multiply(x: u64, y: u64) -> u64 { x * y }
}
