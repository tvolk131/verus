#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

test_verify_one_file_with_options! {
    #[test] subtraction_shift_membership ["vstd", "--infer-goals"] => code! {
        #[verus_infer] fn interval(low:u32,high:u32)->u64 {
            assert!(low<high && high<=64);
            (u64::MAX << low) & (u64::MAX >> (64-high))
        }
        verus! {
            fn membership(low:u32,high:u32,bit:u32)->(result:u64)
                requires low<high,high<=64,bit<64,
                ensures ((result>>bit)&1u64==1u64)==(low<=bit && bit<high),
            {interval(low,high)}
        }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] signed_computed_left_shift ["--infer-goals"] => code! {
        #[verus_infer] fn shifted(shift:i8)->u8 {1u8 << (7-shift)}
        verus! {
            fn caller(shift:i8)->(result:u8)
                requires 0<=shift,shift<=7,
                ensures result>0,
            {shifted(shift)}
        }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] negative_shift_is_still_unsafe ["--infer-goals"] => code! {
        #[verus_infer] fn shifted(shift:i8)->u8 {1u8 << (7-shift)}
        verus! {fn caller()->u8 {shifted(8)}}
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] subtraction_overflow_is_still_unsafe ["--infer-goals"] => code! {
        #[verus_infer] fn shifted(shift:u8)->u8 {1u8 << (7-shift)}
        verus! {fn caller()->u8 {shifted(8)}}
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] false_shift_goal_remains_checked ["--infer-goals"] => code! {
        #[verus_infer] fn shifted(shift:u8)->u8 {1u8 << (7-shift)}
        verus! {
            fn caller(shift:u8)->(result:u8)
                requires shift<=7,
                ensures result==0,
            {shifted(shift)}
        }
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_fixed_widths ["--infer-goals"] => code! {
        verus! {
            fn byte(count:u8)->(result:u8) requires count==7, ensures result==128, {3u8<<count}
            fn word(count:u16)->(result:u16) requires count==15, ensures result==32768, {3u16<<count}
            fn double(count:u32)->(result:u32) requires count==31, ensures result==2147483648u32, {3u32<<count}
            fn quad(count:u64)->(result:u64) requires count==63, ensures result==9223372036854775808u64, {3u64<<count}
            fn wide(count:u128)->(result:u128) requires count==127, ensures result==170141183460469231731687303715884105728u128, {3u128<<count}
        }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_usize_both_architectures ["--infer-goals"] => code! {
        verus! { fn mask(count:usize)->(result:u64) requires count==0, ensures result==0, {(1u64<<count)-1} }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_after_lowered_assertion ["--infer-goals"] => code! {
        verus! {
            fn mask(input:usize)->(result:u64) requires input==0, ensures result==0, {
                let count=input;
                assert(count==0);
                (1u64<<count)-1
            }
        }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_branch_guard ["--infer-goals"] => code! {
        verus! {
            fn mask(count:usize)->(result:u64) ensures result==0, {
                if count==0 {(1u64<<count)-1} else {0}
            }
        }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_boolean_alias_after_assertion ["--infer-goals"] => code! {
        verus! {
            fn mask(input:usize)->(result:u64) requires input==0, ensures result==0, {
                let count=input;
                let first=count==0;
                let second=first;
                assert(second);
                (1u64<<count)-1
            }
        }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_before_assertion_declines ["--infer-goals"] => code! {
        verus! {
            fn mask(input:usize)->(result:u64) requires input==0, ensures result==0, {
                let count=input;
                let result=(1u64<<count)-1;
                assert(count==0);
                result
            }
        }
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_unasserted_false_boolean_declines ["--infer-goals"] => code! {
        verus! {
            fn mask(input:usize)->(result:u64) requires input==1, ensures result==0, {
                let count=input;
                let equality=count==0;
                (1u64<<count)-1
            }
        }
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_stale_boolean_declines ["--infer-goals"] => code! {
        verus! {
            fn mask(input:usize)->(result:u64) requires input==0, ensures result==0, {
                let mut count=input;
                let equality=count==0;
                count=1;
                assert(equality);
                (1u64<<count)-1
            }
        }
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_shadowing_declines ["--infer-goals"] => code! {
        verus! {
            fn mask(count:usize)->(result:u64) requires count==0, ensures result==0, {
                let count=1usize;
                (1u64<<count)-1
            }
        }
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_entry_prestate_expires ["--infer-goals"] => code! {
        verus! {
            fn mask(mut count:usize)->(result:u64) requires count==0, ensures result==0, {
                count=1;
                (1u64<<count)-1
            }
        }
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_mutable_call_expires ["--infer-goals"] => code! {
        verus! {
            fn change(count:&mut usize) ensures *count==1, {*count=1;}
            fn mask(mut count:usize)->(result:u64) requires count==0, ensures result==0, {
                change(&mut count);
                (1u64<<count)-1
            }
        }
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_branch_fact_does_not_escape ["--infer-goals"] => code! {
        verus! {
            fn mask(count:usize)->(result:u64) requires count<64, ensures result==0, {
                if count==0 {assert(count==0);}
                (1u64<<count)-1
            }
        }
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_isolated_proof_does_not_seed ["--infer-goals"] => code! {
        verus! {
            fn mask(input:usize)->(result:u64) requires input==0, ensures result==0, {
                let count=input;
                assert(count==0) by(bit_vector);
                (1u64<<count)-1
            }
        }
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_loop_does_not_seed ["--infer-goals"] => code! {
        verus! {
            fn mask(input:usize)->(result:u64) requires input==0, ensures result==0, {
                let mut count=input;
                while false invariant count==0, decreases 0, {count=0;}
                (1u64<<count)-1
            }
        }
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_guarded_dangerous_arm_stays_guarded ["--infer-goals"] => code! {
        verus! {
            fn mask(count:usize)->(result:u64) requires count==0 || count==64, ensures result==0, {
                if count==0 {(1u64<<count)-1} else {0}
            }
        }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_count_width_is_unsafe ["--infer-goals"] => code! {
        verus! {fn shift(count:u8)->u8 requires count==8, {1u8<<count}}
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_range_is_not_equality ["--infer-goals"] => code! {
        verus! {fn mask(count:usize)->(result:u64) requires count<64, ensures result==0, {(1u64<<count)-1}}
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_wrong_result_is_rejected ["--infer-goals"] => code! {
        verus! {fn mask(count:usize)->(result:u64) requires count==0, ensures result==2, {1u64<<count}}
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_reassignment_after_assertion_declines ["--infer-goals"] => code! {
        verus! {
            fn mask(input:usize)->(result:u64) requires input==0, ensures result==0, {
                let mut count=input;
                assert(count==0);
                count=1;
                (1u64<<count)-1
            }
        }
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_cast_count_declines ["--infer-goals"] => code! {
        verus! {
            fn mask(count:usize)->(result:u64) requires count==0, ensures result==0, {
                (1u64<<(count as u8))-1
            }
        }
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] local_constant_shift_signed_count_declines ["--infer-goals"] => code! {
        verus! {
            fn mask(count:i8)->(result:u64) requires count==0, ensures result==0, {
                (1u64<<count)-1
            }
        }
    } => Err(error) => assert!(!error.errors.is_empty())
}

test_verify_one_file_with_options! {
    #[test] unsigned_shift_cast_does_not_erase_call_safety ["--infer-goals"] => code! {
        #[verus_infer] fn shifted(shift:u8)->u8 {1u8 << (7-shift)}
        verus! {
            fn caller(shift:u8)->(result:u8)
                ensures result>0,
            {shifted(shift)}
        }
    } => Err(error) => assert!(!error.errors.is_empty())
}
