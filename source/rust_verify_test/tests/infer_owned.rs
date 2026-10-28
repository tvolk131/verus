#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const LIFECYCLE: &str = r#"use vstd::prelude::*;
verus! {
struct Snapshot<T:Copy> { slots:Vec<Option<T>>, epoch:u16, ready:bool }
impl<T:Copy> Snapshot<T> {
 #[verifier::infer] fn new(n:usize)->Self {
  Self { slots:vec![None;n], epoch:0, ready:true }
 }
 #[verifier::infer] fn next(&self)->Option<Self> {
  if self.epoch==u16::MAX {return None;}
  let mut out=Vec::with_capacity(self.slots.len());
  let mut i=0usize;
  while i<self.slots.len() {out.push(self.slots[i]);i+=1;}
  Some(Self {slots:out,epoch:self.epoch+1,ready:self.ready})
 }
}
fn initial<T:Copy>(n:usize)->(r:Snapshot<T>)
 ensures r.epoch==0,r.ready,r.slots@.len()==n,
  forall|i:int| 0<=i<n ==> r.slots@[i].is_none(),
{Snapshot::new(n)}
fn advance<T:Copy>(s:&Snapshot<T>)->(r:Option<Snapshot<T>>)
 ensures r.is_none()==(s.epoch==u16::MAX),
  r.is_some() ==> r.unwrap().slots@==s.slots@,
  r.is_some() ==> r.unwrap().epoch==s.epoch+1,
  r.is_some() ==> r.unwrap().ready==s.ready,
{s.next()}
}
"#;

fn name(source: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut h);
    format!("owned_{:x}", h.finish())
}
fn exact(source: &str, count: usize) {
    let d = verify_one_file(&name(source), source.into(), &["vstd", "--no-cheating"]).unwrap();
    for p in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
        assert_eq!(d.notes.iter().filter(|n| n.message.contains(p)).count(), count, "{:?}", d);
    }
}
fn unsupported(source: &str, reason: &str) {
    let e = verify_one_file(&name(source), source.into(), &["vstd", "--no-cheating"]).unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains(reason)), "{:?}", e);
}

#[test]
fn generic_copy_option_and_complete_record_lifecycle_are_exact() {
    exact(LIFECYCLE, 2);
}

#[test]
fn fill_loop_synthesizes_contents_and_termination() {
    exact(
        r#"use vstd::prelude::*;verus!{
 #[verifier::infer] fn fill(n:usize,x:u32)->Vec<u32>{
  let mut out=Vec::new();while out.len()<n{out.push(x);}out
 }
 fn caller(n:usize,x:u32)->(r:Vec<u32>)
 ensures r@.len()==n,forall|i:int|0<=i<n ==> r@[i]==x,{fill(n,x)}
}"#,
        1,
    );
}

#[test]
fn primitive_clone_and_nested_owned_returns_are_exact() {
    exact(
        r#"use vstd::prelude::*;verus!{
 struct Inner{data:Vec<Option<u16>>,stamp:u8}
 struct Outer{inner:Inner,enabled:bool}
 #[verifier::infer] fn copy(s:&Outer)->Outer{
  Outer{inner:Inner{data:s.inner.data.clone(),stamp:s.inner.stamp+1},enabled:s.enabled}
 }
 fn caller(s:&Outer)->(r:Outer) requires s.inner.stamp<255,
 ensures r.inner.data@==s.inner.data@,r.inner.stamp==s.inner.stamp+1,r.enabled==s.enabled,{copy(s)}
}"#,
        1,
    );
}

#[test]
fn partial_fill_expression_is_explicitly_unsupported() {
    unsupported(
        r#"use vstd::prelude::*;verus!{
 #[verifier::infer] fn fill(n:usize,x:u8)->Vec<u8>{
 let mut v=Vec::new();while v.len()<n{v.push(x+1);}v}
 fn empty()->Vec<u8>{fill(0,255)}
 fn nonempty()->Vec<u8>{fill(2,254)}
}"#,
        "fill value must be total",
    );
}

#[test]
fn arithmetic_before_fill_loop_keeps_its_exact_domain() {
    exact(
        r#"use vstd::prelude::*;verus!{
 #[verifier::infer] fn fill(n:usize,x:u8)->Vec<u8>{let value=x+1;
 let mut v=Vec::new();while v.len()<n{v.push(value);}v}
 fn caller()->Vec<u8>{fill(0,254)}
}"#,
        1,
    );
}

#[test]
fn executable_restrictions_are_distinct_from_intended_totality() {
    let s = LIFECYCLE
        .replace("Self { slots:vec![None;n]", "let _unused=n+1; Self { slots:vec![None;n]");
    let e = verify_one_file("restricted_constructor", s, &["vstd", "--no-cheating"]).unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("precondition not satisfied")), "{:?}", e);
    for p in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
        assert_eq!(e.notes.iter().filter(|n| n.message.contains(p)).count(), 2, "{:?}", e);
    }
}

#[test]
fn exact_inference_does_not_certify_wrong_application_round() {
    let s = LIFECYCLE.replace("epoch:0, ready:true", "epoch:1, ready:true");
    let e = verify_one_file("wrong_round", s, &["vstd", "--no-cheating"]).unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("postcondition not satisfied")), "{:?}", e);
    assert_eq!(
        e.notes.iter().filter(|n| n.message.contains("output exactness: PROVED")).count(),
        2,
        "{:?}",
        e
    );
}

#[test]
fn copy_bound_does_not_authorize_arbitrary_clone_equality() {
    unsupported(
        r#"use vstd::prelude::*;verus!{
 #[verifier::infer] fn copy<T:Copy>(v:&Vec<T>)->Vec<T>{v.clone()}
}"#,
        "generic Clone does not establish value equality",
    );
}

#[test]
fn repeated_opaque_value_requires_identity_clone_evidence() {
    unsupported(
        r#"use vstd::prelude::*;verus!{
 #[verifier::infer] fn repeat<T:Copy>(n:usize,x:T)->Vec<T>{vec![x;n]}
}"#,
        "repeat requires checked identity cloning or None",
    );
}

#[test]
fn handwritten_constructor_contracts_are_rejected() {
    unsupported(
        r#"use vstd::prelude::*;verus!{
 #[verifier::infer] fn new(n:usize)->(r:Vec<u8>) ensures r@.len()==0,{Vec::with_capacity(n)}
}"#,
        "no handwritten contracts",
    );
}

#[test]
fn loop_annotations_are_rejected() {
    unsupported(
        r#"use vstd::prelude::*;verus!{
 #[verifier::infer] fn new(n:usize)->Vec<u8>{let mut v=Vec::new();
 while v.len()<n invariant v@.len()<=n, decreases n-v.len(),{v.push(0);}v}
}"#,
        "fill or prefix-copy pattern",
    );
}

#[test]
fn loop_extra_effects_are_rejected() {
    unsupported(
        r#"use vstd::prelude::*;verus!{
 #[verifier::infer] fn new(n:usize)->Vec<u8>{let mut v=Vec::new();
 while v.len()<n{v.push(0);v.push(1);}v}
}"#,
        "fill or prefix-copy pattern",
    );
}

#[test]
fn copied_index_must_match_counter() {
    unsupported(
        r#"use vstd::prelude::*;verus!{
 #[verifier::infer] fn copy(v:&Vec<u8>)->Vec<u8>{let mut out=Vec::new();let mut i=0usize;
 while i<v.len(){out.push(v[0]);i+=1;}out}
}"#,
        "fill or prefix-copy pattern",
    );
}

#[test]
fn proof_assumptions_are_rejected() {
    unsupported(
        r#"use vstd::prelude::*;verus!{
 #[verifier::infer] fn new()->Vec<u8>{proof{assume(false);}Vec::new()}
}"#,
        "owned expression outside",
    );
}

#[test]
fn owned_sound_mode_is_explicitly_unsupported() {
    unsupported(
        r#"use vstd::prelude::*;verus!{
 #[verifier::infer(sound)] fn new()->Vec<u8>{Vec::new()}
}"#,
        "owned inference requires",
    );
}

#[test]
fn owned_output_observation_budget_is_enforced() {
    unsupported(
        r#"use vstd::prelude::*;verus!{
 struct Triple{a:Vec<u8>,b:Vec<u8>,c:Vec<u8>}
 struct Many{a:Triple,b:Triple,c:Triple,d:Triple,e:Triple,f:Triple}
 #[verifier::infer] fn make()->Many{loop{}}
}"#,
        "owned output exceeds 16 observations",
    );
}
