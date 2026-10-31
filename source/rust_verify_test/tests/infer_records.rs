#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const POOL: &str = r#"
use vstd::prelude::*;
verus! {
pub struct Pool<T: Copy> { pub slots: Vec<Option<T>>, pub used: usize, pub epoch: u32 }
impl<T:Copy> Pool<T> {
    pub closed spec fn valid(&self)->bool {
        self.used == self.slots@.count_prefix(|slot:Option<T>|slot.is_some(),self.slots@.len())
    }
    #[verifier::infer]
    pub fn insert(&mut self,i:usize,value:T) {
        if i>=self.slots.len(){return;}
        let empty=match self.slots[i] {None=>true,Some(_)=>false};
        if empty {self.slots[i]=Some(value); self.used+=1;}
    }
    pub fn check_insert(&mut self,i:usize,value:T)
        requires old(self).valid(),
        ensures final(self).valid(), final(self).epoch==old(self).epoch,
            final(self).slots@.len()==old(self).slots@.len(),
    {self.insert(i,value);}
}
}
"#;

#[test]
fn occupied_slots_verify_the_whole_supplied_domain_through_inferred_effects() {
    let result = verify_one_file("pool", POOL.to_owned(), &["vstd", "--infer-goals"]);
    assert!(result.is_ok(), "{:?}", result);
}

fn rejected(name: &str, source: String) {
    let errors = verify_one_file(name, source, &["vstd", "--infer-goals"]).unwrap_err();
    assert!(
        errors.notes.iter().any(|d| d.message.contains("soundness: PROVED")),
        "candidate must remain checked: {:?}",
        errors
    );
    assert!(
        errors.errors.iter().any(|d| d.message.contains("precondition not satisfied")
            || d.message.contains("postcondition not satisfied")),
        "must fail a caller proof: {:?}",
        errors
    );
}

#[test]
fn wrong_counter_fails_coverage_or_preservation() {
    rejected("counter", POOL.replace("self.used+=1", "self.used+=2"));
}
#[test]
fn wrong_slot_fails_preservation() {
    rejected("slot", POOL.replace("self.slots[i]=Some(value)", "self.slots[0]=Some(value)"));
}
#[test]
fn changing_a_frame_field_is_detected() {
    rejected("frame", POOL.replace("self.used+=1;", "self.used+=1;self.epoch+=1;"));
}
#[test]
fn missing_accounting_is_not_inferred_or_assumed() {
    rejected("missing", POOL.replace("requires old(self).valid(),", "requires true,"));
}
#[test]
fn dropping_the_duplicate_guard_is_detected() {
    rejected("duplicate", POOL.replace("if empty {", "if true {"));
}

test_verify_one_file_with_options! {
    #[test] scalar_frames_early_returns_and_read_after_write ["vstd"] => verus_code! {
        pub struct State { pub x:u8, pub y:u8, pub stable:bool }
        impl State {
            #[verifier::infer] fn update(&mut self,skip:bool,x:u8) {
                if skip {return;}
                self.x=x;
                self.y=self.x;
            }
        }
        fn check(s:&mut State,skip:bool,x:u8)
            ensures final(s).x==if skip{old(s).x}else{x},
                final(s).y==if skip{old(s).y}else{x}, final(s).stable==old(s).stable,
        {s.update(skip,x);}
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] unsafe_record_calls_are_rejected ["vstd"] => verus_code! {
        pub struct State {pub n:u8}
        impl State {#[verifier::infer] fn bump(&mut self){self.n+=1;}}
        fn unrestricted(s:&mut State){s.bump(); /* FAILS */}
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file_with_options! {
    #[test] exact_state_checks_cover_unit_return_mutation ["vstd"] => verus_code! {
        pub struct State {pub n:u8}
        impl State {#[verifier::infer] fn bump(&mut self){self.n+=1;}}
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] exact_generic_records_cover_multiple_vectors_and_copy_fields ["vstd"] => verus_code! {
        use vstd::prelude::*;
        pub struct State<T:Copy,U:Copy> {
            pub left:Vec<T>, pub right:Vec<U>, pub last:Option<(T,u16)>, pub stable:bool,
        }
        impl<T:Copy,U:Copy> State<T,U> {
            #[verifier::infer] pub fn update(&mut self,i:usize,j:usize,a:T,b:U,n:u16) {
                if i>=self.left.len() || j>=self.right.len() {return;}
                self.left[i]=a;
                self.right[j]=b;
                self.last=Some((a,n));
            }
            #[verifier::infer] pub fn idle(&mut self) {}
        }
        fn check<T:Copy,U:Copy>(s:&mut State<T,U>,i:usize,j:usize,a:T,b:U,n:u16)
            requires i<old(s).left@.len(),j<old(s).right@.len(),
            ensures final(s).left@==old(s).left@.update(i as int,a),
                final(s).right@==old(s).right@.update(j as int,b),
                final(s).last==Some((a,n)),final(s).stable==old(s).stable,
        {s.update(i,j,a,b,n);s.idle();}
    } => Ok(())
}

test_verify_one_file_with_options! {
    #[test] second_mutable_reference_is_unsupported ["vstd"] => verus_code! {
        pub struct State {pub n:u8}
        impl State {#[verifier::infer] fn bump(&mut self,n:&mut u8){self.n=*n;}}
    } => Err(e) => assert_vir_error_msg(e,"record state requires")
}

test_verify_one_file_with_options! {
    #[test] opaque_record_fields_are_rejected_before_contract_lowering ["vstd"] => verus_code! {
        mod data {pub struct State<T:Copy>{value:T}}
        #[verifier::infer] fn idle<T:Copy>(s:&mut data::State<T>) {}
    } => Err(e) => assert_vir_error_msg(e,"transparent record")
}

test_verify_one_file_with_options! {
    #[test] proof_assumptions_are_unsupported ["vstd"] => verus_code! {
        pub struct State {pub n:u8}
        impl State {#[verifier::infer] fn bump(&mut self){assume(self.n<255);self.n+=1;}}
    } => Err(e) => assert_vir_error_msg(e,"record expression")
}

test_verify_one_file_with_options! {
    #[test] vector_aliases_are_unsupported ["vstd"] => verus_code! {
        use vstd::prelude::*;
        pub struct State {pub slots:Vec<u8>}
        impl State {#[verifier::infer] fn set(&mut self,i:usize,value:u8){let alias=&mut self.slots;alias[i]=value;}}
    } => Err(e) => assert_vir_error_msg(e,"local aliases")
}

test_verify_one_file_with_options! {
    #[test] signed_field_bounds_remain_required ["vstd"] => verus_code! {
        pub struct State {pub n:i8,pub flag:bool}
        impl State {#[verifier::infer] fn decrement(&mut self){self.n-=1;}}
        fn check(s:&mut State) requires old(s).n>i8::MIN,
            ensures final(s).n==old(s).n-1, final(s).flag==old(s).flag,
        {s.decrement();}
    } => Ok(())
}

#[test]
fn all_closed_primitive_from_widenings_keep_exact_proofs() {
    let cases = [
        ("u8", vec!["u16", "u32", "u64", "usize", "u128", "i16", "i32", "i64", "i128", "isize"]),
        ("u16", vec!["u32", "u64", "usize", "u128", "i32", "i64", "i128"]),
        ("u32", vec!["u64", "u128", "i64", "i128"]),
        ("u64", vec!["u128", "i128"]),
        ("i8", vec!["i16", "i32", "i64", "isize", "i128"]),
        ("i16", vec!["i32", "i64", "isize", "i128"]),
        ("i32", vec!["i64", "i128"]),
        ("i64", vec!["i128"]),
    ];
    let mut source = String::from("use vstd::prelude::*; verus! {");
    let mut n = 0;
    for (from, tos) in cases {
        for to in tos {
            source.push_str(&format!("#[verifier::infer] fn widen_{n}(x:{from})->{to}{{{to}::from(x)}}\nfn check_{n}(x:{from})->(r:{to}) ensures r as int==x as int,{{widen_{n}(x)}}\n"));
            n += 1;
        }
    }
    source.push('}');
    assert_eq!(n, 35);
    let result = verify_one_file("widenings", source, &["vstd"]);
    assert!(result.is_ok(), "{:?}", result);
}

test_verify_one_file_with_options! {
    #[test] user_from_implementations_are_not_given_primitive_semantics ["vstd"] => verus_code! {
        use std::convert::From;
        pub struct Wrapped(pub u8);
        impl From<Wrapped> for u16 { fn from(x:Wrapped)->u16 {x.0 as u16+1} }
        #[verifier::infer] fn convert(x:Wrapped)->u16 {u16::from(x)}
    } => Err(e) => assert_vir_error_msg(e,"only static")
}

#[test]
fn record_source_budget_is_enforced_even_for_overwritten_values() {
    let source = format!(
        "use vstd::prelude::*; verus! {{pub struct State{{pub x:u8}} impl State{{#[verifier::infer] fn set(&mut self){{{}}}}}}}",
        "self.x=1;".repeat(1500)
    );
    let e = verify_one_file("source_budget", source, &["vstd"]).unwrap_err();
    assert!(
        e.errors.iter().any(|e| e.message.contains("record source budget exceeded")),
        "{:?}",
        e
    );
}

#[test]
fn record_expansion_budget_is_enforced() {
    let source = format!(
        "use vstd::prelude::*; verus! {{pub struct State{{pub x:u8}} impl State{{#[verifier::infer] fn double(&mut self){{{}}}}}}}",
        "self.x=self.x+self.x;".repeat(16)
    );
    let e = verify_one_file("expansion_budget", source, &["vstd"]).unwrap_err();
    assert!(e.errors.iter().any(|e| e.message.contains("expanded formula budget")), "{:?}", e);
}

#[test]
fn guarded_match_effects_preserve_order_frames_and_early_returns() {
    let source = r#"use vstd::prelude::*;verus!{
 pub struct State{pub value:u16,pub epoch:u16}
 impl State{
  #[verifier::infer] fn choose(&mut self,input:Option<bool>){
   match input{
    Some(x) if x=>{self.value=1;return;},
    Some(_)=>{self.value=2;},
    None=>{},
   }
   self.value=3;
  }
  #[verifier::infer] fn ordered(&mut self,input:Option<u8>){
   match input{
    Some(x) if x<10=>{self.value=x as u16;},
    Some(x) if x<20=>{self.value=(x as u16)+1;},
    _=>{},
   }
  }
 }
 fn first(s:&mut State,input:Option<bool>)
 ensures final(s).value as int==if input==Some(true){1int}else{3int},final(s).epoch==old(s).epoch,
 {s.choose(input);}
 fn second(s:&mut State,input:Option<u8>)
 ensures final(s).value as int==match input{Some(x) if x<10=>x as int,Some(x) if x<20=>(x as int)+1,_=>old(s).value as int},final(s).epoch==old(s).epoch,
 {s.ordered(input);}
}"#;
    let d =
        verify_one_file("guarded_state_match", source.into(), &["vstd", "--no-cheating"]).unwrap();
    assert_eq!(
        d.notes.iter().filter(|n| n.message.contains("output exactness: PROVED")).count(),
        2,
        "{:?}",
        d
    );
}

#[test]
fn guarded_match_safety_is_required_only_on_the_selected_arm() {
    let source = r#"use vstd::prelude::*;verus!{
 pub struct State{pub value:u8}
 impl State{#[verifier::infer] fn update(&mut self,input:Option<u8>,skip:bool){
  match input{Some(_) if skip=>{},Some(x)=>{self.value=x+1;},None=>{}}
 }}
 fn skip(s:&mut State){s.update(Some(u8::MAX),true);}
 fn bad(s:&mut State){s.update(Some(u8::MAX),false);}
}"#;
    let e = verify_one_file("selected_match_safety", source.into(), &["vstd", "--no-cheating"])
        .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("precondition not satisfied")), "{:?}", e);
    assert!(e.notes.iter().any(|d| d.message.contains("output exactness: PROVED")), "{:?}", e);
    assert_eq!(
        e.errors.iter().filter(|d| d.message.contains("precondition not satisfied")).count(),
        1,
        "{:?}",
        e
    );
}
