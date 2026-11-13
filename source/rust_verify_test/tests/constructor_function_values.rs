#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

// Real NonZero, unchanged constructor-function-item body, every u64.
#[test]
fn original_nonzero() {
    verify_one_file(
        "original_nonzero",
        include_str!("../fixtures/infer/constructor_functions/bortag.rs").to_string(),
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
        }
        fn false_second_field() {
            let f = Pair;
            let r = f(7u64, 9u64);
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
    } => Err(err) => assert_rust_error_msg(err, "cannot return value referencing local variable")
}

test_verify_one_file_with_options! {
    #[test] self_public_bound_const_generic_and_zero_arity ["vstd", "--no-cheating"] => verus_code! {
        use vstd::prelude::*;
        struct W<T>(T);
        impl<T> W<T> {
            fn make(x: T) -> (r: Option<Self>)
                ensures r.is_some(), r.unwrap().0 == x,
            { Some(x).map(Self) }
        }
        struct B<T: Copy>(T);
        fn bound<T: Copy>(x: T) {
            let r = Some(x).map(B::<T>);
            assert(r.unwrap().0 == x);
        }
        struct Bytes<const N: usize>([u8; N]);
        fn bytes<const N: usize>(x: [u8; N]) {
            let r = Some(x).map(Bytes::<N>);
            assert(r.unwrap().0 == x);
        }
        struct Zero();
        mod visible { pub struct V(pub u64); }
        fn test() {
            let f = Zero;
            let z = f();
            assert(z == Zero());
            let r = Some(7u64).map(visible::V);
            assert(r.unwrap().0 == 7);
        }
    } => Ok(())
}

#[test]
fn payload_get_roundtrip() {
    let source = include_str!("../fixtures/infer/constructor_functions/bortag.rs").to_string()
        + r#"
verus! {
impl BorTag {
    fn get(&self) -> (r: u64)
        ensures r == self.0@,
    { self.0.get() }
}
fn roundtrip(i: u64) -> (r: u64)
    ensures r == i,
{
    match BorTag::new(i) {
        Some(tag) => tag.get(),
        None => 0,
    }
}
}
"#;
    verify_one_file("payload_get_roundtrip", source, &["vstd", "--no-cheating"]).unwrap();
}
