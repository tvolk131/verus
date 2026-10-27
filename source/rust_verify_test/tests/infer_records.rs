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
    #[verifier::infer(sound)]
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
            #[verifier::infer(sound)] fn update(&mut self,skip:bool,x:u8) {
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
        impl State {#[verifier::infer(sound)] fn bump(&mut self){self.n+=1;}}
        fn unrestricted(s:&mut State){s.bump(); /* FAILS */}
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file_with_options! {
    #[test] exact_state_claims_are_unsupported ["vstd"] => verus_code! {
        pub struct State {pub n:u8}
        impl State {#[verifier::infer] fn bump(&mut self){self.n+=1;}}
    } => Err(e) => assert_vir_error_msg(e,"sound mode")
}

test_verify_one_file_with_options! {
    #[test] second_mutable_reference_is_unsupported ["vstd"] => verus_code! {
        pub struct State {pub n:u8}
        impl State {#[verifier::infer(sound)] fn bump(&mut self,n:&mut u8){self.n=*n;}}
    } => Err(e) => assert_vir_error_msg(e,"record state requires")
}

test_verify_one_file_with_options! {
    #[test] proof_assumptions_are_unsupported ["vstd"] => verus_code! {
        pub struct State {pub n:u8}
        impl State {#[verifier::infer(sound)] fn bump(&mut self){assume(self.n<255);self.n+=1;}}
    } => Err(e) => assert_vir_error_msg(e,"record expression")
}

test_verify_one_file_with_options! {
    #[test] vector_aliases_are_unsupported ["vstd"] => verus_code! {
        use vstd::prelude::*;
        pub struct State {pub slots:Vec<u8>}
        impl State {#[verifier::infer(sound)] fn set(&mut self,i:usize,value:u8){let alias=&mut self.slots;alias[i]=value;}}
    } => Err(e) => assert_vir_error_msg(e,"local aliases")
}

test_verify_one_file_with_options! {
    #[test] signed_field_bounds_remain_required ["vstd"] => verus_code! {
        pub struct State {pub n:i8,pub flag:bool}
        impl State {#[verifier::infer(sound)] fn decrement(&mut self){self.n-=1;}}
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
        "use vstd::prelude::*; verus! {{pub struct State{{pub x:u8}} impl State{{#[verifier::infer(sound)] fn set(&mut self){{{}}}}}}}",
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
        "use vstd::prelude::*; verus! {{pub struct State{{pub x:u8}} impl State{{#[verifier::infer(sound)] fn double(&mut self){{{}}}}}}}",
        "self.x=self.x+self.x;".repeat(16)
    );
    let e = verify_one_file("expansion_budget", source, &["vstd"]).unwrap_err();
    assert!(e.errors.iter().any(|e| e.message.contains("expanded formula budget")), "{:?}", e);
}
