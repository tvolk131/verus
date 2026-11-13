#![feature(rustc_private)]
//! Named views of inferred contracts: `#[verus_infer]` declares
//! `<f>_requires(params)` and `<f>_ensures(params, ret)`, whose bodies are the
//! installed contract. They are definitions, not assumptions.

#[macro_use]
mod common;
use common::*;

test_verify_one_file! {
    #[test] views_name_inferred_contracts_in_lemmas code! {
        use vstd::prelude::*;

        #[verus_verify]
        #[derive(Clone, Copy)]
        pub struct Region { pages: u32, size: u32 }

        impl Region {
            #[verus_infer]
            pub(crate) fn len(&self) -> u64 { u64::from(self.pages) * u64::from(self.size) }

            #[verus_infer]
            pub(crate) fn halve(total: u64, unit: u64) -> u64 { total / unit / 2 }
        }

        #[verus_infer]
        fn scaled(x: u32, k: u32) -> u32 { x * k + 1 }

        verus! {
        proof fn lemma_scaled(x: u32, k: u32)
            requires x <= 100, k <= 100,
            ensures scaled_requires(x, k),
        {
            assert(x * k <= 10000) by(nonlinear_arith) requires x <= 100, k <= 100;
        }

        proof fn lemma_len(r: Region)
            ensures r.len_requires(),
        {
            let (p, s) = (r.pages as int, r.size as int);
            assert(0 <= p * s <= 0xffff_ffff * 0xffff_ffff) by(nonlinear_arith)
                requires 0 <= p <= 0xffff_ffff, 0 <= s <= 0xffff_ffff;
        }

        fn client(r: Region, x: u32, k: u32) -> (out: u64)
            requires x <= 100, k <= 100,
            ensures out == r.pages * r.size,
        {
            proof { lemma_scaled(x, k); }
            let y = scaled(x, k);
            assert(scaled_ensures(x, k, y));
            assert(Region::halve_requires(10, 3));
            proof { lemma_len(r); }
            let l = r.len();
            assert(r.len_ensures(l));
            l
        }
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] views_are_definitions_not_assumptions code! {
        use vstd::prelude::*;

        #[verus_infer]
        fn halve(total: u64, unit: u64) -> u64 { total / unit / 2 }

        #[verus_infer]
        fn scaled(x: u32, k: u32) -> u32 { x * k + 1 }

        verus! {
        proof fn too_wide(x: u32, k: u32)
            requires x <= 100_000, k <= 100_000,
            ensures scaled_requires(x, k), // FAILS
        {
        }

        proof fn zero_divisor() {
            assert(halve_requires(10, 0)); // FAILS
        }

        proof fn wrong_output() {
            assert(halve_ensures(10, 3, 2)); // FAILS
        }
        }
    } => Err(e) => assert_fails(e, 3)
}

test_verify_one_file! {
    #[test] public_views_are_usable_from_other_modules code! {
        use vstd::prelude::*;

        mod geometry {
            use vstd::prelude::*;
            #[verus_verify]
            #[derive(Clone, Copy)]
            pub struct Pages { pub count: u32, pub size: u32 }
            impl Pages {
                #[verus_infer]
                pub fn bytes(&self) -> u64 { self.count as u64 * self.size as u64 }
                #[verus_infer]
                pub fn per_page(total: u64, size: u64) -> u64 { total / size }
            }
        }

        verus! {
        use geometry::Pages;
        proof fn uses(p: Pages)
            requires p.count == 0,
        {
            assert(Pages::per_page_requires(10, 2));
            assert(Pages::per_page_ensures(10, 2, 5));
            assert(p.count as u64 * p.size as u64 == 0);
            assert(p.bytes_ensures(0));
        }
        }
    } => Ok(())
}
