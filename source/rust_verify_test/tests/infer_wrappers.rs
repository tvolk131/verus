#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const WRAPPERS: &str = r#"use vstd::prelude::*;
verus! {
#[derive(Copy,Clone)] pub struct Key(pub usize);
#[derive(Copy,Clone)] pub struct Count(pub usize);
pub struct Slots<T>(pub Vec<Option<T>>);
impl<T> Slots<T> {
 #[verifier::infer] pub fn len(&self)->usize{self.0.len()}
 #[verifier::infer] pub fn get(&self,k:Key)->Option<&T>{self.0[k.0].as_ref()}
 #[verifier::infer] pub fn insert(&mut self,k:Key,v:T){self.0[k.0]=Some(v);}
}
pub struct Registry<T> {pub slots:Slots<T>,pub count:Count,pub stable:bool}
impl<T:Copy> Registry<T> {
 #[verifier::infer] pub fn add(&mut self,k:Key,v:T){
  if k.0>=self.slots.len(){return;}
  let empty=match self.slots.get(k){None=>true,Some(_)=>false};
  if empty {self.slots.insert(k,v);self.count.0+=1;}
 }
}
fn check<T:Copy>(r:&mut Registry<T>,k:Key,v:T)
 requires old(r).count.0==old(r).slots.0@.count_prefix(|s:Option<T>|s.is_some(),old(r).slots.0@.len()),
 ensures final(r).count.0==final(r).slots.0@.count_prefix(|s:Option<T>|s.is_some(),final(r).slots.0@.len()), final(r).stable==old(r).stable,
{r.add(k,v);}
}
"#;

const PRIVATE_WRAPPERS: &str = r#"use vstd::prelude::*;
verus! {
pub mod storage {
 use vstd::prelude::*;
 pub struct Slots<T>(Vec<Option<T>>);
 impl<T> View for Slots<T> {
  type V=Seq<Option<T>>;
  closed spec fn view(&self)->Self::V{self.0@}
 }
 impl<T> Slots<T> {
  #[verifier::infer] pub fn len(&self)->usize{self.0.len()}
  #[verifier::infer] pub fn get(&self,i:usize)->Option<&T>{self.0[i].as_ref()}
  #[verifier::infer] pub fn put(&mut self,i:usize,v:T){self.0[i]=Some(v);}
 }
}
pub struct Registry<T>{pub slots:storage::Slots<T>,pub used:usize,pub stable:bool}
impl<T:Copy> Registry<T> {
 #[verifier::infer] pub fn add(&mut self,i:usize,v:T){
  if i>=self.slots.len(){return;}
  let empty=match self.slots.get(i){None=>true,Some(_)=>false};
  if empty{self.slots.put(i,v);self.used+=1;}
 }
}
fn check<T:Copy>(r:&mut Registry<T>,i:usize,v:T)
 requires old(r).used==old(r).slots@.count_prefix(|s:Option<T>|s.is_some(),old(r).slots@.len()),
 ensures final(r).used==final(r).slots@.count_prefix(|s:Option<T>|s.is_some(),final(r).slots@.len()),final(r).stable==old(r).stable,
{let _=r.slots.len();r.add(i,v);}
}
"#;

#[test]
fn private_sequence_views_preserve_accounting_across_modules() {
 let result=verify_one_file("private",PRIVATE_WRAPPERS.to_owned(),&["vstd","--infer-goals"]);
 assert!(result.is_ok(),"{:?}",result);
}

#[test]
fn private_view_helper_mutation_still_needs_to_satisfy_the_goal() {
 rejected("private_wrong_slot",PRIVATE_WRAPPERS.replace("self.0[i]=Some(v)","self.0[0]=Some(v)"));
}

#[test]
fn private_views_reject_an_extra_overflow_restriction_on_valid_inputs() {
 rejected("private_domain",PRIVATE_WRAPPERS.replace("  if i>=self.slots.len()", "  let extra=i+1; if i>=self.slots.len()"));
}

#[test]
fn private_views_preserve_unchanged_sibling_fields() {
 rejected("private_frame",PRIVATE_WRAPPERS.replace("self.used+=1;", "self.used+=1;self.stable=false;"));
}

#[test]
fn lossy_sequence_views_do_not_qualify_as_complete_state() {
 let source=PRIVATE_WRAPPERS.replace("{self.0@}","{self.0@.take(0)}");
 let e=verify_one_file("lossy",source,&["vstd","--infer-goals"]).unwrap_err();
 assert_vir_error_msg(e,"visible transparent record fields");
}

#[test]
fn sequence_views_cannot_hide_additional_state_fields() {
 let source=PRIVATE_WRAPPERS.replace("Slots<T>(Vec<Option<T>>)","Slots<T>(Vec<Option<T>>,u8)");
 let e=verify_one_file("hidden",source,&["vstd","--infer-goals"]).unwrap_err();
 assert_vir_error_msg(e,"visible transparent record fields");
}

test_verify_one_file_with_options! {
 #[test] private_named_vector_wrapper_moves_noncopy_values_and_preserves_frames ["vstd"] => verus_code! {
  use vstd::prelude::*;
  pub struct Payload(pub u64);
  mod buffer {
   use vstd::prelude::*;
   pub struct Buffer<T>{data:Vec<T>}
   impl<T> View for Buffer<T>{type V=Seq<T>;closed spec fn view(&self)->Seq<T>{self.data@}}
   impl<T> Buffer<T>{#[verifier::infer] pub fn replace(&mut self,i:usize,v:T){self.data[i]=v;}}
  }
  pub struct Outer<T>{pub buffer:buffer::Buffer<T>,pub sibling:buffer::Buffer<T>,pub stable:u8}
  impl<T> Outer<T>{
   #[verifier::infer] pub fn twice(&mut self,i:usize,j:usize,a:T,b:T){self.buffer.replace(i,a);self.buffer.replace(j,b);}
  }
  fn check(s:&mut Outer<Payload>,i:usize,j:usize)
   requires i<old(s).buffer@.len(),j<old(s).buffer@.len(),
   ensures final(s).buffer@==old(s).buffer@.update(i as int,Payload(1)).update(j as int,Payload(2)),
    final(s).sibling@==old(s).sibling@,final(s).stable==old(s).stable,
  {s.twice(i,j,Payload(1),Payload(2));}
 } => Ok(())
}

#[test]
fn nested_wrapper_calls_prove_accounting_and_all_exact_obligations() {
    let result = verify_one_file("nested", WRAPPERS.to_owned(), &["vstd", "--infer-goals"]);
    assert!(result.is_ok(), "{:?}", result);
}

fn rejected(name: &str, source: String) {
    let e = verify_one_file(name, source, &["vstd", "--infer-goals"]).unwrap_err();
    for label in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"]
    {
        assert_eq!(e.notes.iter().filter(|n| n.message.contains(label)).count(), 4, "{:?}", e);
    }
    assert!(
        e.errors.iter().any(|e| e.message.contains("precondition not satisfied")
            || e.message.contains("postcondition not satisfied")),
        "{:?}",
        e
    );
}

#[test]
fn wrong_helper_slot_is_rejected() {
    rejected("slot", WRAPPERS.replace("self.0[k.0]=Some(v)", "self.0[0]=Some(v)"));
}
#[test]
fn missing_caller_guard_is_rejected() {
    rejected("guard", WRAPPERS.replace("if k.0>=self.slots.len(){return;}", ""));
}
#[test]
fn repeated_accounting_is_rejected() {
    rejected("count", WRAPPERS.replace("self.count.0+=1", "self.count.0+=2"));
}
#[test]
fn changed_nested_frame_is_rejected() {
    rejected("frame", WRAPPERS.replace("self.count.0+=1;", "self.count.0+=1;self.stable=false;"));
}

#[test]
fn borrowed_tuple_patterns_and_same_named_generic_substitutions_are_exact() {
    let source = r#"use vstd::prelude::*;
verus! {
#[derive(Copy,Clone)] pub struct Key(pub usize);
#[derive(Copy,Clone)] pub struct Count(pub usize);
pub struct Slots<T>(pub Vec<Option<T>>);
impl<T> Slots<T> {
 #[verifier::infer] pub fn len(&self)->usize{self.0.len()}
 #[verifier::infer] pub fn get(&self,k:Key)->Option<&T>{self.0[k.0].as_ref()}
 #[verifier::infer] pub fn insert(&mut self,k:Key,v:T){self.0[k.0]=Some(v);}
}
pub struct Registry<T> {pub slots:Slots<(T,u16)>,pub count:Count,pub stable:bool}
impl<T:Copy> Registry<T> {
 #[verifier::infer] pub fn add(&mut self,k:Key,v:T,round:u16){
  if k.0>=self.slots.len(){return;}
  let replace=match self.slots.get(k){None=>true,Some((_,prev))=>*prev<round};
  if replace {self.slots.insert(k,(v,round));self.count.0+=1;}
 }
}
}
"#;
    let result = verify_one_file("borrowed", source.to_owned(), &["vstd"]);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn assigning_a_whole_scalar_wrapper_preserves_nested_frames() {
    let source = WRAPPERS.replace("self.count.0+=1;", "self.count=Count(self.count.0+1);");
    let result = verify_one_file("whole", source, &["vstd", "--infer-goals"]);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn exact_callers_reject_sound_mode_mutable_helpers() {
    let source = WRAPPERS
        .replace("#[verifier::infer] pub fn insert", "#[verifier::infer(sound)] pub fn insert");
    let e = verify_one_file("sound", source, &["vstd", "--infer-goals"]).unwrap_err();
    assert_vir_error_msg(e, "exact record inference requires exact-mode callees");
}

test_verify_one_file_with_options! {
    #[test] repeated_generic_updates_read_the_current_state_and_preserve_siblings ["vstd"] => verus_code! {
        use vstd::prelude::*;
        pub struct Slots<T>(pub Vec<Option<T>>);
        impl<T> Slots<T> {
            #[verifier::infer] pub fn put(&mut self,i:usize,v:T){self.0[i]=Some(v);}
            #[verifier::infer] pub fn get(&self,i:usize)->Option<&T>{self.0[i].as_ref()}
            #[verifier::infer] pub fn twice(&mut self,i:usize,j:usize,a:T,b:T){self.put(i,a);self.put(j,b);}
        }
        pub struct Outer<T>{pub entries:Slots<T>,pub sibling:Slots<T>,pub stable:u8}
        impl<T:Copy> Outer<T>{
            #[verifier::infer] pub fn put_and_read(&mut self,i:usize,j:usize,a:T,b:T){
                self.entries.twice(i,j,a,b);
                let present=match self.entries.get(j){None=>false,Some(_)=>true};
                if present {self.stable=1;}
            }
        }
        fn check<T:Copy>(s:&mut Outer<T>,i:usize,j:usize,a:T,b:T)
            requires i<old(s).entries.0@.len(),j<old(s).entries.0@.len(),
            ensures final(s).entries.0@==old(s).entries.0@.update(i as int,Some(a)).update(j as int,Some(b)),
                final(s).sibling.0@==old(s).sibling.0@, final(s).stable==1,
        {s.put_and_read(i,j,a,b);}
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] moved_noncopy_payloads_use_the_original_rust_borrow_checks ["vstd"] => verus_code! {
        use vstd::prelude::*;
        pub struct Payload(pub u64);
        pub struct Slots<T>(pub Vec<Option<T>>);
        impl<T> Slots<T>{
            #[verifier::infer] pub fn put(&mut self,i:usize,v:T){self.0[i]=Some(v);}
            #[verifier::infer] pub fn get(&self,i:usize)->Option<&T>{self.0[i].as_ref()}
        }
        fn check(s:&mut Slots<Payload>) requires old(s).0@.len()>0,
            ensures final(s).0@[0]==Some(Payload(7)),
        {s.put(0,Payload(7));let p=s.get(0);assert(p.is_some());assert(p.unwrap().0==7);}
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] read_only_unit_queries_fail_closed ["vstd"] => verus_code! {
        pub struct Slots<T>(pub Vec<T>);
        impl<T> Slots<T>{#[verifier::infer] fn nothing(&self){}}
    } => Err(e) => assert_vir_error_msg(e,"read-only record queries require a value return")
}
