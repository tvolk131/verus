use vstd::prelude::*;
#[verus_infer(precision="sound")]
#[verifier::rlimit(0.000001)] pub fn kind(flags:u128)->u128{flags&15}
