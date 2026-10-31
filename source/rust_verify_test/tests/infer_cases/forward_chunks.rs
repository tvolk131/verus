use super::*;

const PROVIDER: &str = r#"use vstd::prelude::*;
use vstd::std_specs::chunks_exact::chunks;
#[verus_verify] pub struct Triples<'a>{iter:core::slice::ChunksExact<'a,u8>}
verus!{impl<'a> Triples<'a>{
 #[verifier::type_invariant] pub closed spec fn valid(&self)->bool{
  chunks(self.iter).len()<=usize::MAX && (forall|i:int| 0<=i<chunks(self.iter).len() ==> chunks(self.iter)[i]@.len()==3)
 }
 pub closed spec fn items(&self)->Seq<&'a [u8]>{chunks(self.iter)}
}}
#[verus_verify] impl<'a> Triples<'a>{
 #[verus_spec(r => ensures r.items().len()<=usize::MAX, r.items().len()==input@.len()/3,
   forall|i:int| 0<=i<r.items().len() ==> r.items()[i]@==input@.subrange(3*i,3*i+3))]
 pub fn new(input:&'a [u8])->Self{Self{iter:input.chunks_exact(3)}}
 #[verus_spec(r => ensures r.is_some() <==> n<old(self).items().len(),
   r.is_some() ==> r->Some_0@==old(self).items()[n as int]@,
   final(self).items()==if n<old(self).items().len(){old(self).items().subrange(n as int+1,old(self).items().len() as int)}else{Seq::empty()})]
 pub fn select(&mut self,n:usize)->Option<[u8;3]>{self.iter.nth(n).map(|slice|slice.try_into().expect("triple"))}
}
"#;

const CONSUMER: &str = r#"use vstd::prelude::*;use triples_provider::Triples;
verus!{
fn original_bytes(input:&[u8])->(r:Option<[u8;3]>)
 ensures r.is_some() <==> input@.len()>=9,
    r.is_some() ==> r->Some_0@==input@.subrange(6,9),
{let mut blocks=Triples::new(input);blocks.select(2)}
fn exhaust(input:&[u8])->(r:Option<[u8;3]>) ensures r.is_none(),
{let mut blocks=Triples::new(input);let _=blocks.select(usize::MAX);blocks.select(0)}
}
"#;

#[test]
fn checked_triplet_iterator_composes_across_crates_and_publication_fails_closed() {
    let crates = InferCrates::new();
    for _ in 0..2 {
        let r = crates.build_source(
            "triples_provider",
            PROVIDER,
            &[],
            &["--infer-goals", "--no-cheating", "--output-json"],
            true,
        );
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let report = json_inference(&r);
        assert_eq!(report["array_callback_assistance"].as_array().unwrap().len(), 1);
        assert_eq!(report["type_invariant_entry_assistance"].as_array().unwrap().len(), 1);
        let r = crates.build_source(
            "triples_consumer",
            CONSUMER,
            &["triples_provider"],
            &["--infer-goals", "--no-cheating"],
            true,
        );
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    }
    let wrong = PROVIDER.replace("input.chunks_exact(3)", "input.chunks_exact(2)");
    let r = crates.build_source(
        "triples_provider",
        &wrong,
        &[],
        &["--infer-goals", "--no-cheating"],
        true,
    );
    assert!(!r.status.success());
    assert!(!crates.vir("triples_provider").exists());
    for flags in [
        vec!["--infer-goals", "--no-verify"],
        vec!["--infer-goals", "--verify-root", "--verify-function", "select"],
    ] {
        let r = crates.build_source("triples_provider", PROVIDER, &[], &flags, true);
        assert!(!r.status.success());
        assert!(!crates.vir("triples_provider").exists());
        assert!(String::from_utf8_lossy(&r.stderr).contains("requires full-crate verification"));
    }
}

#[test]
fn concrete_std_iterator_refinement_justifies_no_unwind_without_first_party_opt_in() {
    let source = r#"use vstd::prelude::*;verus!{
 fn skip(iter:&mut core::slice::ChunksExact<'_,u8>,n:usize) no_unwind {let _=iter.nth(n);}
 }"#;
    verify_one_file("concrete_std_unwind", source.into(), &["vstd", "--no-cheating"]).unwrap();
    let bad = r#"use vstd::prelude::*;verus!{
 trait Fake{fn next(&mut self);}
 struct State;
 impl Fake for State{fn next(&mut self) no_unwind {}}
 }"#;
    let e = verify_one_file("unchecked_unwind_refinement", bad.into(), &["vstd", "--no-cheating"])
        .unwrap_err();
    assert!(
        e.errors
            .iter()
            .any(|d| d.message.contains("trait method implementation cannot declare an unwind")),
        "{:?}",
        e
    );
}

#[test]
fn tuple_error_callback_with_conditional_saturation_is_checked_across_crates() {
    let crates = InferCrates::new();
    let provider = r#"use vstd::prelude::*;
#[verus_verify] pub struct Tagged { pub code:u8, pub offset:usize }
#[verus_verify]
#[verus_spec(r => ensures r==match input {
 Ok(v)=>Ok(v), Err((code,first))=>Err(Tagged {code,offset:if first {base} else if base<usize::MAX {(base+1) as usize} else {usize::MAX}})
})]
pub fn wrap(input:Result<u8,(u8,bool)>,base:usize)->Result<u8,Tagged> {
 input.map_err(|(code,first)|Tagged {code,offset:if first {base}else{base.saturating_add(1)}})
}
"#;
    let consumer = r#"use vstd::prelude::*; use tagged_provider::wrap;
verus! {
 fn boundary(code:u8)->(r:Result<u8,tagged_provider::Tagged>)
 ensures r.is_err(),r->Err_0.code==code,r->Err_0.offset==usize::MAX,
 {wrap(Err((code,false)),usize::MAX)}
 fn location(base:usize,code:u8)->(r:Result<u8,tagged_provider::Tagged>)
 requires base<usize::MAX,
 ensures r.is_err(),r->Err_0.code==code,r->Err_0.offset==base+1,
 {wrap(Err((code,false)),base)}
}"#;
    for _ in 0..2 {
        let r = crates.build_source(
            "tagged_provider",
            provider,
            &[],
            &["--infer-goals", "--no-cheating", "--output-json"],
            true,
        );
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        assert_eq!(json_inference(&r)["error_callback_assistance"].as_array().unwrap().len(), 1);
        let r = crates.build_source(
            "tagged_consumer",
            consumer,
            &["tagged_provider"],
            &["--infer-goals", "--no-cheating"],
            true,
        );
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    }
    let wrong = provider.replace("base.saturating_add(1)", "base.saturating_add(2)");
    let r = crates.build_source(
        "tagged_provider",
        &wrong,
        &[],
        &["--infer-goals", "--no-cheating"],
        true,
    );
    assert!(!r.status.success());
    assert!(!crates.vir("tagged_provider").exists());
}
