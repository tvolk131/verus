#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const IMPLS: &str = r#"use vstd::prelude::*;
use vstd::std_specs::{ops::*,convert::*};
verus! {
#[derive(Copy,Clone)] pub struct Count(pub usize);
impl From<usize> for Count {
 #[verifier::infer] fn from(n:usize)->Self {Count(n)}
}
impl FromSpecImpl<usize> for Count {
 open spec fn obeys_from_spec()->bool {true}
 open spec fn from_spec(n:usize)->Self {Count(n)}
}
impl core::ops::AddAssign for Count {
 #[verifier::infer] fn add_assign(&mut self,rhs:Self){self.0+=rhs.0;}
}
impl AddAssignSpecImpl for Count {
 open spec fn obeys_add_assign_spec()->bool {true}
 open spec fn add_assign_req(&self,rhs:Self)->bool {self.0 as int+rhs.0 as int<=usize::MAX as int}
 open spec fn add_assign_spec(&self,rhs:Self)->&Self {&Count((self.0+rhs.0) as usize)}
}
#[verifier::infer] fn wrap(n:usize)->Count {n.into()}
pub struct State{pub count:Count,pub epoch:usize}
impl State{#[verifier::infer] pub fn bump(&mut self){self.count+=Count(1);}}
fn caller(s:&mut State)
 requires old(s).count.0<usize::MAX,
 ensures final(s).count.0==old(s).count.0+1,final(s).epoch==old(s).epoch,
{s.bump();}
fn conversion(n:usize)->(r:Count) ensures r.0==n,{wrap(n)}
}
"#;

#[test]
fn concrete_from_into_and_add_assign_compose_with_exact_checks() {
    let r = verify_one_file("wrapper_traits", IMPLS.into(), &["vstd", "--no-cheating"]);
    assert!(r.is_ok(), "{:?}", r);
    let d = r.unwrap();
    for p in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
        assert_eq!(d.notes.iter().filter(|n| n.message.contains(p)).count(), 4, "{:?}", d);
    }
}

#[test]
fn overrestricted_inherited_domains_fail_independent_boundary_check() {
    let s = IMPLS.replace("<=usize::MAX as int}", "<=usize::MAX as int/2}");
    let e = verify_one_file("restrictive_wrapper", s, &["vstd", "--no-cheating"]).unwrap_err();
    assert!(
        e.errors.iter().any(|d| d.message.contains("inherited trait precondition must equal")),
        "{:?}",
        e
    );
    assert!(e.notes.iter().any(|d| d.message.contains("soundness: PROVED")), "{:?}", e);
}

#[test]
fn false_inherited_domains_cannot_make_exact_inference_vacuous() {
    let s = IMPLS.replace("self.0 as int+rhs.0 as int<=usize::MAX as int}", "false}");
    let e = verify_one_file("false_wrapper", s, &["vstd", "--no-cheating"]).unwrap_err();
    assert!(
        e.errors.iter().any(|d| d.message.contains("inherited trait precondition must equal")),
        "{:?}",
        e
    );
}

#[test]
fn weak_inherited_domains_do_not_hide_overflow() {
    let s = IMPLS.replace("self.0 as int+rhs.0 as int<=usize::MAX as int}", "true}");
    let e = verify_one_file("weak_wrapper", s, &["vstd", "--no-cheating"]).unwrap_err();
    assert!(
        e.errors.iter().any(|d| d.message.contains("overflow")
            || d.message.contains("inherited trait precondition must equal")),
        "{:?}",
        e
    );
}

#[test]
fn wrong_extension_models_fail_ordinary_verification() {
    let e = verify_one_file(
        "wrong_wrapper",
        IMPLS.replace("&Count((self.0+rhs.0) as usize)", "&Count(self.0)"),
        &["vstd", "--no-cheating"],
    )
    .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("postcondition not satisfied")), "{:?}", e);
}

#[test]
fn conversion_names_do_not_supply_identity_semantics() {
    let s = r#"use vstd::prelude::*; verus! {
  pub struct Count(pub usize);
  impl From<usize> for Count{#[verifier::infer] fn from(n:usize)->Self{Count(0)}}
  impl vstd::std_specs::convert::FromSpecImpl<usize> for Count {
   open spec fn obeys_from_spec()->bool{true}
   open spec fn from_spec(n:usize)->Self{Count(0)}
  }
  #[verifier::infer] fn wrap(n:usize)->Count{Count::from(n)}
  fn caller(n:usize)->(r:Count) ensures r.0==n,{wrap(n)}
 }"#;
    let e =
        verify_one_file("changed_conversion", s.into(), &["vstd", "--no-cheating"]).unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("postcondition not satisfied")), "{:?}", e);
    assert_eq!(
        e.notes.iter().filter(|d| d.message.contains("output exactness: PROVED")).count(),
        2
    );
}

#[test]
fn add_assign_names_do_not_supply_addition_semantics() {
    let s = IMPLS
        .replace("self.0+=rhs.0;", "self.0=rhs.0;")
        .replace("self.0 as int+rhs.0 as int<=usize::MAX as int}", "true}")
        .replace("&Count((self.0+rhs.0) as usize)", "&Count(rhs.0)");
    let e = verify_one_file("replacement_operator", s, &["vstd", "--no-cheating"]).unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("postcondition not satisfied")), "{:?}", e);
    assert_eq!(
        e.notes.iter().filter(|d| d.message.contains("output exactness: PROVED")).count(),
        4
    );
}

#[test]
fn partial_from_implementations_cannot_introduce_a_new_trait_precondition() {
    let s = r#"use vstd::prelude::*; verus! {
  pub struct Count(pub usize);
  impl From<usize> for Count{#[verifier::infer] fn from(n:usize)->Self{Count(n+1)}}
 }"#;
    let e = verify_one_file("partial_from", s.into(), &["vstd", "--no-cheating"]).unwrap_err();
    assert!(
        e.errors.iter().any(|d| d.message.contains("overflow")
            || d.message.contains("inherited trait precondition must equal")),
        "{:?}",
        e
    );
}

#[test]
fn native_impl_attributes_generate_checked_extension_definitions() {
    let s = r#"use vstd::prelude::*;
 #[verus_verify] #[derive(Copy,Clone)] pub struct Count(pub usize);
 #[verus_infer] impl From<usize> for Count{fn from(n:usize)->Self{Count(n)}}
 #[verus_infer] impl From<Count> for usize{fn from(n:Count)->Self{n.0}}
 #[verus_infer] impl core::ops::AddAssign for Count{fn add_assign(&mut self,rhs:Self){self.0+=rhs.0;}}
 #[verus_infer] fn roundtrip(n:usize)->usize{let c:Count=n.into();c.into()}
 #[verus_verify] struct State{count:Count}
 #[verus_verify] impl State{#[verus_infer] fn bump(&mut self){self.count+=Count(1);}}
 verus! {
  fn check(n:usize)->(r:usize) ensures r==n,{roundtrip(n)}
  fn increment(s:&mut State) requires old(s).count.0<usize::MAX,
   ensures final(s).count.0==old(s).count.0+1,{s.bump();}
 }"#;
    let r = verify_one_file("native_wrapper", s.into(), &["vstd", "--no-cheating"]);
    assert!(r.is_ok(), "{:?}", r);
}

#[test]
fn multi_field_trait_implementations_are_outside_the_wrapper_profile() {
    let s = r#"use vstd::prelude::*;verus! {
  struct Count{a:usize,b:usize}
  impl core::ops::AddAssign for Count{#[verifier::infer] fn add_assign(&mut self,rhs:Self){self.a+=rhs.a;}}
 }"#;
    let e = verify_one_file("multi_wrapper", s.into(), &["vstd"]).unwrap_err();
    assert_vir_error_msg(e, "single scalar field");
}

#[test]
fn unsupported_native_impl_bodies_fail_with_a_profile_diagnostic() {
    let s = r#"use vstd::prelude::*;
 #[verus_verify] struct Count(usize);
 #[verus_infer] impl From<usize> for Count{fn from(n:usize)->Self{Count(n+1)}}"#;
    let e = verify_one_file("unsupported_native_wrapper", s.into(), &["vstd"]).unwrap_err();
    assert!(
        e.errors.iter().any(|d| d.message.contains("Verus Infer wrapper impl supports")),
        "{:?}",
        e
    );
}

#[test]
fn signed_unsigned_and_primitive_rhs_wrappers_use_their_actual_ranges() {
    for typ in ["u8", "i8", "usize"] {
        let source=r#"use vstd::prelude::*;
   #[verus_verify] pub struct Count(pub NUMBER);
   #[verus_infer] impl From<NUMBER> for Count{fn from(n:NUMBER)->Self{Count(n)}}
   #[verus_infer] impl core::ops::AddAssign<NUMBER> for Count{fn add_assign(&mut self,n:NUMBER){self.0+=n;}}
   #[verus_verify] struct State{count:Count,stable:bool}
   #[verus_verify] impl State{#[verus_infer] fn bump(&mut self){self.count+=1;}}
   verus! {
    fn goal(s:&mut State) requires old(s).count.0<NUMBER::MAX,
     ensures final(s).count.0 as int==old(s).count.0 as int+1,final(s).stable==old(s).stable,
    {s.bump();}
   }"#.replace("NUMBER",typ);
        let r =
            verify_one_file(&format!("wrapper_width_{typ}"), source, &["vstd", "--no-cheating"]);
        assert!(r.is_ok(), "{:?}", r);
    }
}
