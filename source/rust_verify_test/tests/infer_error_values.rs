#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const ERRORS: &str = r#"use vstd::prelude::*;verus!{
pub trait Family{type Hash:Copy;}
pub struct Count(pub usize);
pub struct Index(pub usize);
pub enum Error<H:Family>{Count(Count),Hashes(H::Hash,H::Hash),Missing(Index),Wrong(u16),Few(u16),Future(u16)}
pub struct Validator<H:Family>{pub expected:H::Hash,pub threshold:usize}
impl<H:Family> Validator<H>{
 #[verifier::infer] pub fn query(&self,kind:u8,round:u16,count:usize,node:usize,hash:H::Hash)->Result<(),Error<H>>{
  if kind==0{return Err(Error::Count(Count(count)));}
  if kind==1{return Err(Error::Hashes(hash,self.expected));}
  if kind==2{return Err(Error::Missing(Index(node)));}
  if kind==3{return Err(Error::Wrong(round));}
  if kind==4{return Err(Error::Few(round));}
  if kind==5{return Err(Error::Future(round));}
  Ok(())
 }
}
fn policy<H:Family>(v:&Validator<H>,kind:u8,round:u16,count:usize,node:usize,hash:H::Hash)->(r:Result<(),Error<H>>)
 ensures r==if kind==0{Err(Error::Count(Count(count)))}else if kind==1{Err(Error::Hashes(hash,v.expected))}
 else if kind==2{Err(Error::Missing(Index(node)))}else if kind==3{Err(Error::Wrong(round))}
 else if kind==4{Err(Error::Few(round))}else if kind==5{Err(Error::Future(round))}else{Ok(())},
{v.query(kind,round,count,node,hash)}
}"#;

#[test]
fn six_variant_generic_errors_preserve_every_tag_and_payload() {
    let d = verify_one_file("error_values", ERRORS.into(), &["vstd", "--no-cheating"]).unwrap();
    for p in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
        assert_eq!(d.notes.iter().filter(|d| d.message.contains(p)).count(), 1, "{:?}", d);
    }
}

#[test]
fn changed_error_payload_is_exactly_inferred_but_rejected_by_policy() {
    let source = ERRORS.replace(
        "return Err(Error::Hashes(hash,self.expected))",
        "return Err(Error::Hashes(self.expected,hash))",
    );
    let e =
        verify_one_file("changed_error_payload", source, &["vstd", "--no-cheating"]).unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("postcondition not satisfied")), "{:?}", e);
    assert!(e.notes.iter().any(|d| d.message.contains("output exactness: PROVED")), "{:?}", e);
}

#[test]
fn enum_matches_read_named_and_positional_payloads() {
    let source = r#"use vstd::prelude::*;verus!{
 pub enum Error{Missing{node:usize},Bounds(u16,u16),Empty}
 #[verifier::infer] fn code(e:Error)->usize{
  match e{Error::Missing{node}=>node,Error::Bounds(lo,hi) if lo<hi=>lo as usize,Error::Bounds(_,hi)=>hi as usize,Error::Empty=>0}
 }
 fn goal(node:usize)->(r:usize) ensures r==node,{code(Error::Missing{node})}
}"#;
    verify_one_file("enum_payload_matches", source.into(), &["vstd", "--no-cheating"]).unwrap();
}

#[test]
fn error_payload_arithmetic_keeps_its_safety_domain() {
    let source = r#"use vstd::prelude::*;verus!{
 enum Error{Previous(u16),Other}
 #[verifier::infer] fn checked(round:u16)->Result<(),Error>{Err(Error::Previous(round-1))}
 fn unsafe_caller(){let _=checked(0);}
}"#;
    let e = verify_one_file("error_payload_underflow", source.into(), &["vstd", "--no-cheating"])
        .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("precondition not satisfied")), "{:?}", e);
    assert!(
        e.notes.iter().any(|d| d.message.contains("safe-domain maximality: PROVED")),
        "{:?}",
        e
    );
}

#[test]
fn identity_error_propagation_preserves_payloads_and_short_circuit_domains() {
    let source = format!(
        r#"{ERRORS} verus!{{
 impl<H:Family> Validator<H>{{
  #[verifier::infer] fn chain(&self,kind:u8,round:u16,count:usize,node:usize,hash:H::Hash)->Result<u16,Error<H>>{{
   self.query(kind,round,count,node,hash)?;
   Ok(round+1)
  }}
 }}
 fn error_bypasses_later_overflow<H:Family>(v:&Validator<H>,hash:H::Hash)->(r:Result<u16,Error<H>>)
 ensures r==Err::<u16,Error<H>>(Error::Count(Count(7))),
 {{v.chain(0,u16::MAX,7,0,hash)}}
 fn successful_chain<H:Family>(v:&Validator<H>,round:u16,hash:H::Hash)->(r:Result<u16,Error<H>>)
 requires round<u16::MAX,
 ensures r==Ok::<u16,Error<H>>((round+1) as u16),
 {{v.chain(9,round,0,0,hash)}}
 }}"#
    );
    let d = verify_one_file("error_propagation", source, &["vstd", "--no-cheating"]).unwrap();
    for p in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
        assert_eq!(d.notes.iter().filter(|d| d.message.contains(p)).count(), 2, "{:?}", d);
    }
}

#[test]
fn propagation_of_an_always_error_value_preserves_the_exact_constructor() {
    let source = r#"use vstd::prelude::*;verus!{
 enum Error{Missing(u16),Other}
 #[verifier::infer] fn fail(round:u16)->Result<u32,Error>{let _=Err::<u8,Error>(Error::Missing(round))?;Ok(9)}
 fn check(round:u16)->(r:Result<u32,Error>) ensures r==Err::<u32,Error>(Error::Missing(round)),{fail(round)}
}"#;
    verify_one_file("always_error_propagation", source.into(), &["vstd", "--no-cheating"]).unwrap();
}

#[test]
fn oversized_and_heap_error_values_fail_with_bounded_diagnostics() {
    for (name, definition) in [
        ("too_many_variants", "enum Error{A,B,C,D,E,F,G,H,I}"),
        ("heap_payload", "enum Error{A(Vec<u8>),B}"),
        ("recursive_payload", "enum Error{A(Box<Error>),B}"),
    ] {
        let source = format!(
            "use vstd::prelude::*;verus!{{{definition} struct State{{n:usize}} impl State{{#[verifier::infer] fn check(&self)->Result<(),Error>{{Ok(())}}}}}}"
        );
        let e = verify_one_file(name, source, &["vstd", "--no-cheating"]).unwrap_err();
        assert!(
            e.errors.iter().any(|d| d.message.contains("unsupported owned input/output leaf")),
            "{:?}",
            e
        );
    }
}

#[test]
fn literal_runtime_assertions_keep_the_native_failure_domain() {
    let source = r#"use vstd::prelude::*;verus!{
 enum Error{Missing(u16),Other}
 struct State{n:u16}
 impl State{#[verifier::infer] fn check(&self)->Result<(),Error>{
  assert!(self.n>0,"Round must be greater than 0");Ok(())
 }}
 fn goal(s:&State)->(r:Result<(),Error>) requires s.n>0,ensures r.is_ok(),{s.check()}
}"#;
    let d = verify_one_file("literal_runtime_assert", source.into(), &["vstd", "--no-cheating"])
        .unwrap();
    for p in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
        assert!(d.notes.iter().any(|n| n.message.contains(p)), "{:?}", d);
    }
    let bad = source.replace("requires s.n>0,", "");
    let e =
        verify_one_file("unchecked_runtime_assert", bad, &["vstd", "--no-cheating"]).unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("precondition not satisfied")), "{:?}", e);
}

#[test]
fn proof_assertions_and_dynamic_panic_arguments_do_not_enter_the_runtime_model() {
    for (name, check) in [
        ("proof_assert", "assert(self.n>0);"),
        ("dynamic_panic", "assert!(self.n>0,\"round {}\",self.n);"),
    ] {
        let source = format!(
            "use vstd::prelude::*;verus!{{enum Error{{Bad(u16),Other}} struct State{{n:u16}} impl State{{#[verifier::infer] fn check(&self)->Result<(),Error>{{{check} Ok(())}}}}}}"
        );
        let e = verify_one_file(name, source, &["vstd", "--no-cheating"]).unwrap_err();
        assert!(e.errors.iter().any(|d| d.message.contains("unsupported")), "{:?}", e);
    }
}
