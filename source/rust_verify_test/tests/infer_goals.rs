#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

test_verify_one_file! {
    #[test] exact_equation_alone_does_not_prove_mask_bound code! {
        #[verus_infer] fn bucket(key: u64) -> u8 { (key & 15) as u8 }
        verus! { fn caller(key: u64) { let r = bucket(key); assert(r < 16); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file_with_options! {
    #[test] demanded_mask_bound_and_modulo ["vstd", "--infer-goals"] => code! {
        #[verus_infer] fn bucket(key: u64) -> u8 { (key & 15) as u8 }
        verus! {
            fn caller(key: u64) {
                let r = bucket(key);
                assert(r < 16);
                assert(r as u64 == key % 16);
                let a = [0u8; 16];
                let _ = a[r as usize];
            }
        }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] vote_schedule_and_multiple_inferred_layers ["--infer-goals"] => code! {
        #[verus_infer] fn common(round: u16) -> bool {
            if round == 3 { return false; }
            if round <= 4 { return true; }
            round % 2 == 1
        }
        #[verus_infer] fn relay(round: u16) -> bool { common(round) }
        verus! {
            fn caller(round: u16) {
                let vote = relay(round);
                assert(vote == (round != 3 && (round <= 4 || (round & 1u16) == 1u16)));
            }
        }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] branch_assumption_is_used_locally ["--infer-goals"] => code! {
        #[verus_infer] fn identity(x: u8) -> u8 { x }
        verus! {
            fn caller(x: u8) {
                let r = identity(x);
                if x < 16 { assert((r & 15u8) == r); }
            }
        }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] caller_precondition_is_used ["--infer-goals"] => code! {
        #[verus_infer] fn identity(x: u8) -> u8 { x }
        verus! {
            fn caller(x: u8) requires x < 16, {
                let r = identity(x);
                assert((r & 15u8) == r);
            }
        }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] branch_assumption_does_not_escape ["--infer-goals"] => code! {
        #[verus_infer] fn identity(x: u8) -> u8 { x }
        verus! {
            fn caller(x: u8) {
                let r = identity(x);
                if x < 16 { assert((r & 15u8) == r); }
                assert((r & 15u8) == r); // FAILS
            }
        }
    } => Err(e) => assert!(!e.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] false_goal_is_not_assumed ["--infer-goals"] => code! {
        #[verus_infer] fn bucket(x: u8) -> u8 { x & 15 }
        verus! { fn caller(x: u8) { let r=bucket(x); assert((r & 15u8) == 16u8); /* FAILS */ } }
    } => Err(e) => assert!(!e.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] unchanged_original_goal_catches_stale_substitution ["--infer-goals"] => code! {
        #[verus_infer] fn identity(x: u8) -> u8 { x }
        verus! {
            fn caller(x: u8) {
                let mut key = x;
                let r = identity(key);
                key = 0;
                assert((r & 255u8) == key); // FAILS
            }
        }
    } => Err(e) => assert!(!e.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] callee_safety_requirement_remains ["--infer-goals"] => code! {
        #[verus_infer] fn next(x: u8) -> u8 { x + 1 }
        verus! {
            fn caller(x: u8) {
                let r = next(x); // FAILS
                assert((r & 255u8) == r);
            }
        }
    } => Err(e) => assert!(!e.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] signed_leaf_bounds_bridge_the_two_encodings ["--infer-goals"] => code! {
        #[verus_infer] fn identity(x: i8) -> i8 { x }
        verus! { fn caller(x: i8) { let r=identity(x); assert((r & 15i8) >= 0); } }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] request_budget_does_not_assume_the_unattempted_goal ["--infer-goals"] => code! {
        #[verus_infer] fn bucket(x: u8) -> u8 { x & 15 }
        verus! {
            fn caller(a:u8,b:u8,c:u8,d:u8,e:u8,f:u8,g:u8,h:u8,i:u8) {
                let x=bucket(a); assert(x<16);
                let x=bucket(b); assert(x<16);
                let x=bucket(c); assert(x<16);
                let x=bucket(d); assert(x<16);
                let x=bucket(e); assert(x<16);
                let x=bucket(f); assert(x<16);
                let x=bucket(g); assert(x<16);
                let x=bucket(h); assert(x<16);
                let x=bucket(i); assert(x<16); // FAILS
            }
        }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file_with_options! {
    #[test] sound_summary_is_not_reinterpreted_as_exact ["--infer-goals"] => code! {
        #[verus_infer(precision="sound")] fn bucket(x: u8) -> u8 { x & 15 }
        verus! { fn caller(x: u8) { let r=bucket(x); assert(r == (x & 15u8)); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}
