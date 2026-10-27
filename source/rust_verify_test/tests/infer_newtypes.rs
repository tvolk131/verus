#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

test_verify_one_file! {
    #[test] quorum_method_accepts_every_machine_input verus_code! {
        use vstd::prelude::*;
        #[derive(Copy, Clone)] pub struct Count(pub usize);
        impl Count {
            #[verifier::infer]
            pub fn threshold(&self) -> Count {
                Count((self.0 / 3) * 2 + ((self.0 % 3) * 2) / 3 + 1)
            }
        }
        #[verifier::infer]
        fn scalar(n: usize) -> usize { Count(n).threshold().0 }
        fn properties(n: usize) -> (r: usize)
            ensures r as int == 2 * (n as int) / 3 + 1,
                r > 0, n > 0 ==> r <= n,
                3 * r as int > 2 * n as int,
                3 * (r as int - 1) <= 2 * n as int,
        { scalar(n) }
    } => Ok(diags) => {
        assert_eq!(diags.notes.iter().filter(|d| d.message.contains("output exactness: PROVED")).count(), 2);
    }
}

test_verify_one_file! {
    #[test] original_arithmetic_restriction_is_preserved verus_code! {
        struct Count { value: usize }
        impl Count {
            #[verifier::infer]
            fn threshold(&self) -> Count { Count { value: self.value * 2 / 3 + 1 } }
        }
        fn safe(n: Count) -> (r: Count)
            requires n.value <= usize::MAX / 2,
            ensures r.value as int == 2 * n.value as int / 3 + 1,
        { n.threshold() }
        fn unsafe_caller() { let _ = Count { value: usize::MAX }.threshold(); /* FAILS */ }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] signed_boolean_named_fields_and_branches verus_code! {
        use vstd::prelude::*;
        #[derive(Copy, Clone)] struct Signed { value: i8 }
        struct Flag(bool);
        impl Signed {
            #[verifier::infer]
            fn half(&self) -> Signed { Signed { value: self.value / 2 } }
        }
        #[verifier::infer]
        fn pick(a: Signed, b: Signed, flag: Flag) -> Signed { if flag.0 { a.half() } else { b } }
        fn caller() {
            let r = pick(Signed { value: -7 }, Signed { value: 9 }, Flag(true));
            assert(r.value == -3);
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] shared_receiver_and_immutable_wrapper_locals verus_code! {
        use vstd::prelude::*;
        #[derive(Copy, Clone)] struct Amount(u16);
        impl Amount {
            #[verifier::infer] fn difference(&self, other: &Amount) -> Amount {
                let a = *self;
                let b = *other;
                Amount(if a.0 >= b.0 { a.0 - b.0 } else { b.0 - a.0 })
            }
        }
        fn caller(x: Amount, y: Amount) -> (r: Amount)
            ensures r.0 as int == if x.0 >= y.0 { x.0 - y.0 } else { y.0 - x.0 },
        { x.difference(&y) }
    } => Ok(())
}

test_verify_one_file! {
    #[test] mutable_wrappers_are_rejected verus_code! {
        struct Count(usize);
        #[verifier::infer] fn increment(x: &mut Count) -> usize { x.0 += 1; x.0 }
    } => Err(e) => assert_vir_error_msg(e, "Verus Infer unsupported")
}

test_verify_one_file! {
    #[test] multiple_fields_are_rejected verus_code! {
        struct Pair { a: u8, b: u8 }
        #[verifier::infer] fn first(x: Pair) -> u8 { x.a }
    } => Err(e) => assert_vir_error_msg(e, "parameters and return values")
}

test_verify_one_file! {
    #[test] nested_wrappers_are_rejected verus_code! {
        struct Inner(u8); struct Outer(Inner);
        #[verifier::infer] fn nested(x: Outer) -> u8 { x.0.0 }
    } => Err(e) => assert_vir_error_msg(e, "parameters and return values")
}

test_verify_one_file! {
    #[test] generic_wrappers_are_rejected verus_code! {
        struct Count<T>(T);
        #[verifier::infer] fn get(x: Count<u8>) -> u8 { x.0 }
    } => Err(e) => assert_vir_error_msg(e, "parameters and return values")
}

test_verify_one_file! {
    #[test] returning_wrapper_references_is_rejected verus_code! {
        struct Count(u8);
        #[verifier::infer] fn borrow(x: &Count) -> &Count { x }
    } => Err(e) => assert_vir_error_msg(e, "returning wrapper references")
}

test_verify_one_file! {
    #[test] operator_preconditions_are_not_installed_on_trait_impls code! {
        use std::ops::Mul;
        verus! {
            struct Count(usize);
            impl Mul<usize> for Count {
                type Output = Self;
                #[verifier::infer]
                fn mul(self, rhs: usize) -> Self { Count(self.0 * rhs) }
            }
        }
    } => Err(e) => assert_vir_error_msg(e, "preconditions are inherited from the trait")
}

test_verify_one_file! {
    #[test] sound_mode_does_not_silently_project_wrappers verus_code! {
        struct Count(u8);
        #[verifier::infer(sound)] fn get(x: Count) -> u8 { x.0 }
    } => Err(e) => assert_vir_error_msg(e, "parameters and return values")
}
