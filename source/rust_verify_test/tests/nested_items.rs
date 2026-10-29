#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

test_verify_one_file! {
    #[test] test_ignorable_items verus_code! {
        mod m {
            // Verus can ignore 'use' item
            use super::*;

            pub open spec fn test() -> bool { true }
        }

        fn test_use_item() {
            use m::test as test2;
            assert(test2());
        }

        fn test_macro_item() {
            let mut x = 1;

            // Verus can ignore 'macro' item
            macro_rules! add_1 {
                () => {
                    x = x + 1;
                }
            }

            add_1!();

            assert(x == 2);
        }
    } => Ok(())
}

// TODO: spec fn, broadcast, etc.
// TODO: ownership check

test_verify_one_file! {
    #[test] test_attr_syntax_items_without_nested_spec code! {
        #[verus_spec(requires 1 + 1 == 2)]
        fn test() {
            const fn f1(i: u64) -> u64 {
                i / 2
            }
            fn f2(i: u64) -> u64 {
                i / 2
            }
            const C: u64 = 4 + 5;

            let x = f1(10);
            let y = f2(10);
            let z = C;
            proof! {
                assert(z == 9);
                assert(y == 5); // FAILS
            }
        }
    } => Err(err) => assert_one_fails(err)
}

test_verify_one_file! {
    #[test] test_attr_syntax_items_with_nested_spec code! {
        #[verus_spec(requires 1 + 1 == 2)]
        fn test() {
            #[verus_spec(r => ensures r == i / 2)]
            const fn f1(i: u64) -> u64 {
                i / 2
            }
            #[verus_spec(r => ensures r == i / 2)]
            fn f2(i: u64) -> u64 {
                i / 2
            }
            #[verus_spec()]
            const C: u64 = 4 + 5;

            let x = f1(10);
            let y = f2(10);
            let z = C;
            proof! {
                assert(x == 5);
                assert(y == 5);
                assert(z == 9);
            }
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] test_attr_syntax_items_ownership code! {
        struct S;

        verus! {
            #[verifier::external_body]
            proof fn produce() -> tracked S {
                panic!()
            }

            proof fn consume(tracked s: S) {
            }
        }

        #[verus_spec(requires 1 + 1 == 2)]
        fn test() {
            const C: u64 = {
                proof! {
                    let s = produce();
                    consume(s);
                    consume(s);
                }
                4 + 5
            };

            let z = C;
        }
    } => Err(err) => assert_rust_error_msg(err, "use of moved value")
}

test_verify_one_file! {
    #[test] test_verus_items_without_nested_spec verus_code! {
        fn test() {
            const fn f1(i: u64) -> u64 {
                i / 2
            }
            fn f2(i: u64) -> u64 {
                i / 2
            }
            const C: u64 = 4 + 5;

            let x = f1(10);
            let y = f2(10);
            let z = C;
            assert(z == 9);
            assert(y == 5); // FAILS
        }
    } => Err(err) => assert_one_fails(err)
}

test_verify_one_file! {
    #[test] test_verus_items_with_nested_spec verus_code! {
        fn test() {
            const fn f1(i: u64) -> (r: u64)
                ensures r == i / 2
            {
                i / 2
            }
            fn f2(i: u64) -> (r: u64)
                ensures r == i / 2
            {
                i / 2
            }
            const C: u64 = 4 + 5;

            let x = f1(10);
            let y = f2(10);
            let z = C;
            assert(x == 5);
            assert(y == 5);
            assert(z == 9);
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] test_verus_items_ownership verus_code! {
        struct S;

        #[verifier::external_body]
        proof fn produce() -> tracked S {
            panic!()
        }

        proof fn consume(tracked s: S) {
        }

        fn test() {
            const C: u64 = {
                let s = produce();
                consume(s);
                consume(s);
                4 + 5
            };

            let z = C;
        }
    } => Err(err) => assert_rust_error_msg(err, "use of moved value")
}

test_verify_one_file! {
    #[test] test_verus_items_ownership_nested verus_code! {
        fn test() {
            struct S;

            #[verifier::external_body]
            proof fn produce() -> tracked S {
                panic!()
            }

            proof fn consume(tracked s: S) {
            }

            const C: u64 = {
                let s = produce();
                consume(s);
                consume(s);
                4 + 5
            };

            let z = C;
        }
    } => Err(err) => assert_rust_error_msg(err, "use of moved value")
}

test_verify_one_file! {
    #[test] test_ghost_items verus_code! {
        proof fn test() {
            spec fn f(i: int) -> int
                decreases i
            {
                if i <= 0 {
                    0
                } else {
                    f(i - 1) + 1
                }
            }

            broadcast proof fn p(i: int)
                ensures #[trigger] f(i) >= 0
                decreases i
            {
                if i > 0 {
                    p(i - 1);
                }
            }

            assert(f(200) >= 0) by {
                broadcast use p;
            }
            assert(f(100) >= 0); // FAILS
        }
    } => Err(err) => assert_one_fails(err)
}

// Codec derives put independent implementations in unnamed constant wrappers.
// Rust's disambiguator must remain part of each implementation's VIR identity.
#[test]
fn unnamed_constants_keep_distinct_trait_implementation_identities() {
    let code=r#"use vstd::prelude::*;verus!{
trait Read {fn value(&self)->(r:u8) ensures r>0;}
struct A;
struct B;
}
const _: ()={#[verus_verify] impl Read for A {fn value(&self)->u8 {1}}};
const _: ()={#[verus_verify] impl Read for B {fn value(&self)->u8 {2}}};
"#;
    verify_one_file("anonymous_impls",code.into(),&["vstd","--no-cheating"]).unwrap();
    for (label,source) in [("first",code.replace("{1}","{0}")),("second",code.replace("{2}","{0}"))] {
        let e=verify_one_file(&format!("anonymous_bad_{label}"),source,&["vstd","--no-cheating"]).unwrap_err();
        assert_eq!(e.errors.len(), 1);
        assert!(e.errors[0].message.contains("postcondition not satisfied"), "{:?}", e);
    }
}
