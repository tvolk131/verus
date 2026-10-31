#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const MODEL: &str = r#"use vstd::prelude::*;use vstd::rwlock::*;verus!{
pub struct Allowed;
impl RwLockPredicate<Option<u64>> for Allowed {
 open spec fn inv(self,v:Option<u64>)->bool{v.is_none() || v==Some(7)}
}
"#;

#[test]
fn finite_read_attempts_preserve_the_configured_predicate() {
    let source = format!(
        r#"{MODEL}
fn check(lock:&RwLock<Option<u64>,Allowed>){{
 let value=lock.try_read_or(None);assert(value.is_none() || value==Some(7));
 match lock.try_acquire_read(){{None=>{{}},Some(handle)=>{{
  let value=*handle.borrow();assert(value.is_none() || value==Some(7));handle.release_read();
 }}}}
}}}}"#
    );
    verify_one_file("try_read_predicate", source, &["vstd", "--no-cheating"]).unwrap();
}

#[test]
fn fallback_must_satisfy_the_predicate() {
    let source = format!(
        "{MODEL} fn wrong(lock:&RwLock<Option<u64>,Allowed>){{let _=lock.try_read_or(Some(8));}}}}"
    );
    let e =
        verify_one_file("try_read_bad_fallback", source, &["vstd", "--no-cheating"]).unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("precondition not satisfied")), "{:?}", e);
}

#[test]
fn try_read_does_not_promise_success() {
    let source = format!(
        "{MODEL} fn wrong(lock:&RwLock<Option<u64>,Allowed>){{let r=lock.try_read_copy();assert(r.is_some());}}}}"
    );
    let e = verify_one_file("try_read_not_always_success", source, &["vstd", "--no-cheating"])
        .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("assertion failed")), "{:?}", e);
}
