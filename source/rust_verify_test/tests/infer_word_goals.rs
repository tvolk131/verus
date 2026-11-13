#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

test_verify_one_file_with_options! {
    #[test] word_view_grant_and_count ["vstd", "--infer-goals", "--no-cheating"] => code! {
        verus! {
            fn grant(w: u64, i: u64) -> (r: u64)
                requires i < 64,
                ensures
                    vstd::word::view(r, 64) == vstd::word::view(w, 64).update(i as int, true),
                    vstd::word::view(r, 64).count_prefix(|b: bool| b, 64) ==
                        vstd::word::view(w, 64).count_prefix(|b: bool| b, 64)
                        + if vstd::word::view(w, 64)[i as int] { 0nat } else { 1nat },
            { w | (1u64 << i) }
        }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] xor_is_not_a_grant ["vstd", "--infer-goals", "--no-cheating"] => code! {
        verus! {
            fn grant(w: u64, i: u64) -> (r: u64)
                requires i < 64,
                ensures vstd::word::view(r, 64) == vstd::word::view(w, 64).update(i as int, true), // FAILS
            { w ^ (1u64 << i) }
        }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file_with_options! {
    #[test] no_word_automation_without_flag ["vstd", "--no-cheating"] => code! {
        verus! {
            fn grant(w: u64, i: u64) -> (r: u64)
                requires i < 64,
                ensures vstd::word::view(r, 64) == vstd::word::view(w, 64).update(i as int, true), // FAILS
            { w | (1u64 << i) }
        }
    } => Err(e) => assert_one_fails(e)
}
