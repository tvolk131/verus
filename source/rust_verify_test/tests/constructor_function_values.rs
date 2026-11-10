#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

// Real NonZero, unchanged constructor-function-item body, every u64.
#[test]
fn original_nonzero() {
    verify_one_file(
        "original_nonzero",
        include_str!("../../tools/research/constructor_function_value/bortag.rs").to_string(),
        &["vstd", "--no-cheating"],
    )
    .unwrap();
}

test_verify_one_file_with_options! {
    #[test] generic_move_lifetime_variant_and_order ["vstd", "--no-cheating"] => verus_code! {
        use vstd::prelude::*;
        struct MoveOnly(u64);
        struct Wrap<T>(T);
        struct Borrowed<'a>(&'a u64);
        struct Pair(u64, u64);
        enum Choice<T> { First(T), Second(T), Empty }

        fn wraps<T>(x: T) -> (r: Option<Wrap<T>>)
            ensures r.is_some(), r.unwrap().0 == x,
        { Some(x).map(Wrap::<T>) }

        fn test(x: MoveOnly, y: u64, z: u64) {
            let w = Some(x).map(Wrap);
            assert(w.unwrap().0 == x);
            let a = Some(y).map(Choice::<u64>::Second);
            assert(a.unwrap() == Choice::<u64>::Second(y));
            assert(a.unwrap() !is First);
            let b = Some(&y).map(Borrowed);
            assert(*b.unwrap().0 == y);
            let f = Pair;
            let pair = f(y, z);
            assert(pair.0 == y && pair.1 == z);
            let absent = Option::<u64>::None.map(Wrap);
            assert(absent.is_none());
        }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] wrong_variant_field_order_and_false_post ["vstd", "--no-cheating"] => verus_code! {
        use vstd::prelude::*;
        struct Pair(u64, u64);
        enum E { A(u64), B(u64) }
        fn false_variant() {
            let r = Some(7u64).map(E::B);
            assert(r.unwrap() is A); // FAILS
        }
        fn false_fields() {
            let f = Pair;
            let r = f(7u64, 9u64);
            assert(r.0 == 9); // FAILS
            assert(r.1 == 7); // FAILS
        }
        fn false_post(x: u64) -> (r: Option<E>)
            ensures r.is_none(), // FAILS
        { Some(x).map(E::A) }
    } => Err(err) => assert_fails(err, 4)
}

test_verify_one_file_with_options! {
    #[test] higher_order_requires ["vstd", "--no-cheating"] => verus_code! {
        use vstd::prelude::*;
        fn unchecked<F: FnOnce(u64) -> u64>(f: F) -> u64 {
            f(0) // FAILS
        }
    } => Err(err) => assert_fails(err, 1)
}

test_verify_one_file_with_options! {
    #[test] higher_order_checked ["vstd", "--no-cheating"] => verus_code! {
        use vstd::prelude::*;
        struct W(u64);
        fn apply<F: FnOnce(u64) -> W>(f: F, x: u64) -> (r: W)
            requires f.requires((x,)),
            ensures f.ensures((x,), r),
        { f(x) }
        fn test(x: u64) {
            let r = apply(W, x);
            assert(r.0 == x);
        }
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] mixed_mode_rejected ["vstd", "--no-cheating"] => verus_code! {
        use vstd::prelude::*;
        struct W(ghost u64);
        fn test() { let _r = Some(1u64).map(W); }
    } => Err(err) => assert_vir_error_msg(err, "constructor function values require")
}

test_verify_one_file_with_options! {
    #[test] invariant_rejected ["vstd", "--no-cheating"] => verus_code! {
        use vstd::prelude::*;
        struct W(u64);
        impl W {
            #[verifier::type_invariant]
            spec fn inv(&self) -> bool { self.0 > 0 }
        }
        fn test() { let _r = Some(0u64).map(W); }
    } => Err(err) => assert_vir_error_msg(err, "constructor function values require")
}

test_verify_one_file_with_options! {
    #[test] fn_pointer_rejected ["vstd", "--no-cheating"] => verus_code! {
        struct W(u64);
        fn test() { let _f: fn(u64) -> W = W; }
    } => Err(err) => assert_vir_error_msg(err, "does not yet support")
}

test_verify_one_file_with_options! {
    #[test] visibility_checked ["vstd", "--no-cheating"] => verus_code! {
        mod private {
            pub struct W(u64);
        }
        fn test() { let _r = Some(0u64).map(private::W); }
    } => Err(err) => assert_rust_error_msg(err, "private")
}

test_verify_one_file_with_options! {
    #[test] move_checked ["vstd", "--no-cheating"] => verus_code! {
        struct M(u64);
        struct W(M);
        fn test(x: M) {
            let _r = Some(x).map(W);
            let _s = Some(x).map(W);
        }
    } => Err(err) => assert_rust_error_msg(err, "moved value")
}

test_verify_one_file_with_options! {
    #[test] lifetime_checked ["vstd", "--no-cheating"] => verus_code! {
        struct W<'a>(&'a u64);
        fn test<'a>() -> Option<W<'a>> {
            let x = 7u64;
            Some(&x).map(W)
        }
    } => Err(err) => assert_rust_error_msg(err, "does not live long enough")
}
