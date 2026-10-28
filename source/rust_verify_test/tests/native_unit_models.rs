#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

#[test]
fn copied_lock_operations_preserve_the_construction_predicate() {
    let s = r#"use vstd::prelude::*;use vstd::rwlock::RwLock;verus!{
 fn cache(expected:u64){
  let lock=RwLock::<Option<u64>,spec_fn(Option<u64>)->bool>::new(None::<u64>,Ghost(|v:Option<u64>|v.is_none() || v.unwrap()==expected));
  lock.replace(Some(expected));
  let observed=lock.read_copy();
  assert(observed.is_some() ==> observed.unwrap()==expected);
  lock.replace(None);
  let observed=lock.read_copy();
  assert(observed.is_some() ==> observed.unwrap()==expected);
 }
}"#;
    verify_one_file("copied_lock", s.into(), &["vstd", "--no-cheating"]).unwrap();
    let e = verify_one_file(
        "wrong_lock_write",
        s.replace("replace(Some(expected))", "replace(Some(0))"),
        &["vstd", "--no-cheating"],
    )
    .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("precondition not satisfied")), "{:?}", e);
}

#[test]
fn native_byte_order_matches_the_target_model() {
    let s = r#"use vstd::prelude::*;verus!{
 fn bytes(x:u64)->(r:[u8;8]) ensures r@==vstd::bytes::spec_u64_to_ne_bytes(x),{x.to_ne_bytes()}
 fn zero(){
  broadcast use {vstd::seq::group_seq_axioms,vstd::array::group_array_axioms};
  let r=bytes(0);
  proof{
   vstd::bytes::spec_u64_to_le_bytes_to_open(0);
   assert((0u64 & 0xff) as u8 == 0) by(bit_vector);
   assert(((0u64 >> 56) & 0xff) as u8 == 0) by(bit_vector);
  }
  assert(r@[0]==0);assert(r@[7]==0);
 }
}"#;
    verify_one_file("native_bytes", s.into(), &["vstd", "--no-cheating"]).unwrap();
}

#[test]
fn external_signatures_normalize_closed_array_lengths_but_reject_mismatch() {
    let s = r#"use vstd::prelude::*;verus!{
 #[verifier::external] pub fn original(x:u64)->[u8;{1+1}]{[0;2]}
 pub assume_specification[original](x:u64)->[u8;2];
}"#;
    verify_one_file("equal_const_lengths", s.into(), &["vstd"]).unwrap();
    let e = verify_one_file(
        "different_const_lengths",
        s.replace("(x:u64)->[u8;2];", "(x:u64)->[u8;3];"),
        &["vstd"],
    )
    .unwrap_err();
    assert!(
        e.errors.iter().any(|d| d.message.contains("function type signature")
            || d.message.contains("mismatched types")),
        "{:?}",
        e
    );
}
