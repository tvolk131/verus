#![feature(rustc_private)]
//! Regressions for the engine fixes from the redb layout pilot (2026-11-13):
//! one-pass rejection reporting, owned division and mutable locals, dead-branch
//! joins (formerly an ill-typed AIR crash), callees with functional
//! specifications (vstd std specs and hand-written contracts), and diverging
//! `assert_eq!`.

#[macro_use]
mod common;
use common::*;

fn proved(diags: &TestErr, expected: usize) {
    for label in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"]
    {
        assert_eq!(
            diags.notes.iter().filter(|d| d.message.contains(label)).count(),
            expected,
            "{label}: {:?}",
            diags.notes
        );
    }
}

test_verify_one_file! {
    #[test] every_rejection_is_reported_in_one_run code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn first(x: u8) -> u8 { loop {} }

        #[verus_infer]
        fn second(o: Option<u64>) -> u64 { o.unwrap_or_default() }

        #[verus_infer]
        fn blocked(x: u8) -> u8 { first(x) }
    } => Err(e) => {
        assert!(e.errors.len() >= 3, "{:?}", e.errors);
        assert!(e.errors.iter().any(|d| d.message.contains("loops")), "{:?}", e.errors);
        assert!(e.errors.iter().any(|d| d.message.contains("unwrap_or_default")), "{:?}", e.errors);
        assert!(e.errors.iter().any(|d| d.message.contains("blocked: this calls")), "{:?}", e.errors);
    }
}

test_verify_one_file! {
    #[test] owned_division_and_reassigned_locals code! {
        use vstd::prelude::*;
        #[verus_verify]
        #[derive(Clone, Copy)]
        pub struct Split { whole: u64, rest: u64 }

        #[verus_infer]
        fn split(total: u64, unit: u64, header: u64) -> Split {
            let mut remaining = total - header;
            let whole = remaining / unit;
            remaining -= whole * unit;
            let rest = if remaining > 0 {
                remaining -= 1;
                remaining + 1
            } else {
                0
            };
            Split { whole, rest }
        }
    } => Ok(diags) => proved(&diags, 1)
}

test_verify_one_file! {
    #[test] dead_panic_branch_joins_without_air_crash code! {
        use vstd::prelude::*;
        #[verus_verify]
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct Region { pages: u32, header: u32 }
        #[verus_verify]
        #[derive(Clone, Copy)]
        pub struct Layout { full: Region, trailing: Option<Region> }
        impl Layout {
            #[verus_infer]
            pub(crate) fn trailing(&self) -> Region {
                match self.trailing { Some(v) => v, None => panic!("x") }
            }
            #[verus_infer]
            pub(crate) fn pick(&self, c: bool) -> Region {
                let o = if c { self.trailing } else { Some(self.full) };
                match o { Some(v) => v, None => panic!("x") }
            }
        }
    } => Ok(diags) => proved(&diags, 2)
}

test_verify_one_file! {
    #[test] vstd_functional_specs_and_assert_eq code! {
        use vstd::prelude::*;
        #[verus_verify]
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct Region { pages: u32, header: u32 }

        #[verus_infer]
        fn round_up(value: u64, multiple: u64) -> u64 {
            if value.is_multiple_of(multiple) { value } else { value + multiple - value % multiple }
        }

        #[verus_infer]
        fn pick(full: Region, trailing: Option<Region>, last: bool) -> Region {
            if last { trailing.unwrap() } else { full }
        }

        #[verus_infer]
        fn same_header(a: Region, b: Region) -> Region {
            assert_eq!(a.header, b.header);
            a
        }
    } => Ok(diags) => proved(&diags, 3)
}

test_verify_one_file! {
    #[test] inferred_caller_of_functional_hand_written_contract code! {
        use vstd::prelude::*;
        #[verus_verify]
        #[derive(Clone, Copy)]
        pub struct Region { pages: u32, size: u32 }
        #[verus_verify]
        #[derive(Clone, Copy)]
        pub struct Layout { full: Region, trailing: Option<Region>, count: u32 }
        impl Region {
            #[verus_infer]
            pub(crate) fn len(&self) -> u64 { u64::from(self.pages) * u64::from(self.size) }
        }
        impl Layout {
            // Inferred, although its callee keeps a hand-written contract.
            #[verus_infer]
            pub(crate) fn last_len(&self) -> u64 { self.region(self.count).len() }
        }
        verus! {
        impl Layout {
            pub(crate) fn region(&self, i: u32) -> (r: Region)
                requires i == self.count ==> self.trailing is Some,
                ensures r == if i == self.count { self.trailing->0 } else { self.full },
            {
                if i == self.count { self.trailing.unwrap() } else { self.full }
            }
        }
        }
    } => Ok(diags) => proved(&diags, 2)
}

/// redb `src/tree_store/page_store/layout.rs` at redb commit
/// 3e82eb8828d4fd5f4a52ab00ce93f6ff52882e98 (MIT OR Apache-2.0, Copyright the redb
/// authors), with `#[verus_infer]` on every function and these source-preserving
/// edits only: `use vstd::prelude::*`, `#[verus_verify]` on the two structs,
/// `pub(super)` widened to `pub(crate)`, the test module removed, and the two
/// functions the 2026-11-13 pilot harness never reaches removed
/// (`reduce_last_region`, `DatabaseLayout::usable_bytes`). No executable line is
/// changed: all 19 functions infer exact contracts through vstd's specs for
/// `is_multiple_of`, `try_into().unwrap()`, `Option::unwrap` and `assert_eq!`.
const REDB_LAYOUT: &str = r#"use vstd::prelude::*;
use core::ops::Range;

#[verus_infer]

fn round_up_to_multiple_of(value: u64, multiple: u64) -> u64 {
    if value.is_multiple_of(multiple) {
        value
    } else {
        value + multiple - value % multiple
    }
}

// Regions are laid out starting with the allocator state header, followed by the pages aligned
// to the next page
#[verus_verify]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionLayout {
    num_pages: u32,
    // Offset where data pages start
    header_pages: u32,
    page_size: u32,
}

impl RegionLayout {
    #[verus_infer]
    pub(crate) fn new(num_pages: u32, header_pages: u32, page_size: u32) -> Self {
        assert!(num_pages > 0);
        Self {
            num_pages,
            header_pages,
            page_size,
        }
    }

    #[verus_infer]

    pub(crate) fn calculate(
        desired_usable_bytes: u64,
        page_capacity: u32,
        region_header_pages: u32,
        page_size: u32,
    ) -> RegionLayout {
        assert!(desired_usable_bytes <= u64::from(page_capacity) * u64::from(page_size));
        let num_pages =
            round_up_to_multiple_of(desired_usable_bytes, page_size.into()) / u64::from(page_size);

        Self {
            num_pages: num_pages.try_into().unwrap(),
            header_pages: region_header_pages,
            page_size,
        }
    }

    #[verus_infer]

    pub(crate) fn data_section(&self) -> Range<u64> {
        let header_bytes = u64::from(self.header_pages) * u64::from(self.page_size);
        header_bytes..(header_bytes + self.usable_bytes())
    }

    #[verus_infer]

    pub(crate) fn get_header_pages(&self) -> u32 {
        self.header_pages
    }

    #[verus_infer]

    pub(crate) fn num_pages(&self) -> u32 {
        self.num_pages
    }

    #[verus_infer]

    pub(crate) fn page_size(&self) -> u32 {
        self.page_size
    }

    #[verus_infer]

    pub(crate) fn len(&self) -> u64 {
        u64::from(self.header_pages) * u64::from(self.page_size) + self.usable_bytes()
    }

    #[verus_infer]

    pub(crate) fn usable_bytes(&self) -> u64 {
        u64::from(self.page_size) * u64::from(self.num_pages)
    }
}

#[verus_verify]
#[derive(Clone, Copy, Debug)]
pub(crate) struct DatabaseLayout {
    full_region_layout: RegionLayout,
    num_full_regions: u32,
    trailing_partial_region: Option<RegionLayout>,
}

impl DatabaseLayout {
    #[verus_infer]
    pub(crate) fn new(
        full_regions: u32,
        full_region: RegionLayout,
        trailing_region: Option<RegionLayout>,
    ) -> Self {
        Self {
            full_region_layout: full_region,
            num_full_regions: full_regions,
            trailing_partial_region: trailing_region,
        }
    }


    #[verus_infer]

    pub(crate) fn recalculate(
        file_len: u64,
        region_header_pages_u32: u32,
        region_max_data_pages_u32: u32,
        page_size_u32: u32,
    ) -> Self {
        let page_size = u64::from(page_size_u32);
        let region_header_pages = u64::from(region_header_pages_u32);
        let region_max_data_pages = u64::from(region_max_data_pages_u32);
        // Super-header
        let mut remaining = file_len - page_size;
        let full_region_size = (region_header_pages + region_max_data_pages) * page_size;
        let full_regions = remaining / full_region_size;
        remaining -= full_regions * full_region_size;
        let trailing = if remaining >= (region_header_pages + 1) * page_size {
            remaining -= region_header_pages * page_size;
            let remaining: u32 = remaining.try_into().unwrap();
            let data_pages = remaining / page_size_u32;
            assert!(data_pages < region_max_data_pages_u32);
            Some(RegionLayout::new(
                data_pages,
                region_header_pages_u32,
                page_size_u32,
            ))
        } else {
            None
        };
        let full_layout = RegionLayout::new(
            region_max_data_pages_u32,
            region_header_pages_u32,
            page_size_u32,
        );

        Self {
            full_region_layout: full_layout,
            num_full_regions: full_regions.try_into().unwrap(),
            trailing_partial_region: trailing,
        }
    }

    #[verus_infer]

    pub(crate) fn calculate(
        desired_usable_bytes: u64,
        page_capacity: u32,
        region_header_pages: u32,
        page_size: u32,
    ) -> Self {
        let full_region_layout = RegionLayout::new(page_capacity, region_header_pages, page_size);
        // Exactly one region's worth is one full region, as `recalculate` reads it back from the
        // file's length, not a trailing region of the full size
        if desired_usable_bytes < full_region_layout.usable_bytes() {
            // Single region layout
            let region_layout = RegionLayout::calculate(
                desired_usable_bytes,
                page_capacity,
                region_header_pages,
                page_size,
            );
            DatabaseLayout {
                full_region_layout,
                num_full_regions: 0,
                trailing_partial_region: Some(region_layout),
            }
        } else {
            // Multi region layout
            let full_regions = desired_usable_bytes / full_region_layout.usable_bytes();
            let remaining_desired =
                desired_usable_bytes - full_regions * full_region_layout.usable_bytes();
            assert!(full_regions > 0);
            let trailing_region = if remaining_desired > 0 {
                Some(RegionLayout::calculate(
                    remaining_desired,
                    page_capacity,
                    region_header_pages,
                    page_size,
                ))
            } else {
                None
            };
            if let Some(ref region) = trailing_region {
                // All regions must have the same header size
                assert_eq!(region.header_pages, full_region_layout.header_pages);
            }
            DatabaseLayout {
                full_region_layout,
                num_full_regions: full_regions.try_into().unwrap(),
                trailing_partial_region: trailing_region,
            }
        }
    }

    #[verus_infer]

    pub(crate) fn full_region_layout(&self) -> &RegionLayout {
        &self.full_region_layout
    }

    #[verus_infer]

    pub(crate) fn trailing_region_layout(&self) -> Option<&RegionLayout> {
        self.trailing_partial_region.as_ref()
    }

    #[verus_infer]

    pub(crate) fn num_full_regions(&self) -> u32 {
        self.num_full_regions
    }

    #[verus_infer]

    pub(crate) fn num_regions(&self) -> u32 {
        if self.trailing_partial_region.is_some() {
            self.num_full_regions + 1
        } else {
            self.num_full_regions
        }
    }

    #[verus_infer]

    pub(crate) fn len(&self) -> u64 {
        let last = self.num_regions() - 1;
        self.region_base_address(last) + self.region_layout(last).len()
    }


    #[verus_infer]

    pub(crate) fn region_base_address(&self, region: u32) -> u64 {
        assert!(region < self.num_regions());
        u64::from(self.full_region_layout.page_size())
            + u64::from(region) * self.full_region_layout.len()
    }

    #[verus_infer]

    pub(crate) fn region_layout(&self, region: u32) -> RegionLayout {
        assert!(region < self.num_regions());
        if region == self.num_full_regions {
            self.trailing_partial_region.unwrap()
        } else {
            self.full_region_layout
        }
    }
}

"#;

#[test]
fn redb_layout_infers_without_executable_changes() {
    let run = verify_one_file("redb_layout", REDB_LAYOUT.to_string(), &["vstd", "--no-cheating"])
        .unwrap_or_else(|e| panic!("{:?}", e.errors));
    proved(&run, 19);
    // The largest contracts are reported, with their named views.
    let large = |name: &str| {
        run.notes.iter().any(|d| {
            d.message.contains(&format!("DatabaseLayout::{name}` is large"))
                && d.message.contains(&format!("DatabaseLayout::{name}_requires`"))
        })
    };
    assert!(large("calculate") && large("len"), "{:?}", run.notes);
}
