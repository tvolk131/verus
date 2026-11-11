#![feature(rev_into_inner)]
// Type/lifetime witnesses only; this does not verify the fallback or a native law.
pub fn actual_rev<'a>(e: core::iter::Enumerate<core::slice::Iter<'a,u64>>)
    -> core::iter::Rev<core::iter::Enumerate<core::slice::Iter<'a,u64>>> { e.rev() }
pub fn actual_inner<'a>(r: core::iter::Rev<core::iter::Enumerate<core::slice::Iter<'a,u64>>>)
    -> core::iter::Enumerate<core::slice::Iter<'a,u64>> { r.into_inner() }
pub fn original<'a>(borrows: &'a Vec<u64>) { for (_,_) in borrows.iter().enumerate().rev() {} }
