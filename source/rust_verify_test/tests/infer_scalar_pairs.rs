#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

test_verify_one_file! {
    #[test] exact_pair_helpers_and_unrelated_arithmetic verus_code! {
        #[verifier::infer]
        fn split(bit:u32)->(usize,usize){((bit as usize)/64,(bit as usize)%64)}
        #[verifier::infer]
        fn arithmetic(x:u8,y:u8)->(u8,u8){(x+1,x/y)}
        #[verifier::infer]
        fn compose(x:u8,y:u8,b:bool)->(u8,u8){
            let pair=arithmetic(x,y);
            if b {(pair.1,pair.0)} else {pair}
        }
        fn caller(x:u32){
            let p=split(x);assert(p.0==(x as usize)/64 && p.1==(x as usize)%64);
            let p=split(0);assert(p.0==0 && p.1==0);
            let p=split(u32::MAX);assert(p.0==67108863 && p.1==63);
            let p=compose(254,2,true);assert(p.0==127 && p.1==255);
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] discarded_right_component_still_requires_safety verus_code! {
        #[verifier::infer]
        fn discard(x:u8,y:u8)->u8{let p=(x+1,x/y);p.0}
        fn caller(){let _=discard(1,0); /* FAILS */}
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] discarded_left_component_still_requires_safety verus_code! {
        #[verifier::infer]
        fn discard(x:u8,y:u8)->u8{let p=(x/y,x+1);p.1}
        fn caller(){let _=discard(1,0); /* FAILS */}
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] tuples_are_not_inputs verus_code! {
        #[verifier::infer] fn f(x:(u8,u8))->u8{x.0}
    } => Err(e) => assert_vir_error_msg(e,"Verus Infer unsupported")
}

test_verify_one_file! {
    #[test] nested_tuple_is_outside_profile verus_code! {
        #[verifier::infer] fn f(x:u8)->((u8,u8),u8){((x,x),x)}
    } => Err(e) => assert_vir_error_msg(e,"Verus Infer unsupported")
}

test_verify_one_file! {
    #[test] arity_three_is_outside_profile verus_code! {
        #[verifier::infer] fn f(x:u8)->(u8,u8,u8){(x,x,x)}
    } => Err(e) => assert_vir_error_msg(e,"Verus Infer unsupported")
}
