use super::*;

const PROVIDER: &str = r#"use vstd::prelude::*;
#[verus_verify] pub struct Slots<T>(Vec<Option<T>>);
verus!{
impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}
impl<T> Slots<T>{
 pub open spec fn entries_value<'a>(&'a self)->Seq<(usize,&'a T)>{
  vstd::seq_update::sparse_entries(self@,|i:int|(i as usize,&self@[i].unwrap()))
 }
}
pub struct Entry{pub index:usize,pub value:u16}
impl Slots<u16>{
 pub open spec fn mapped_value(&self)->Seq<Entry>{
  self.entries_value().map_values(|e:(usize,&u16)|Entry{index:e.0,value:*e.1})
 }
}
}
impl<T> Slots<T>{
 #[verus_verify(iterator_via_view(self.entries_value()))]
 pub fn iter<'a>(&'a self)->impl Iterator<Item=(usize,&'a T)>+'a {
  self.0.iter().enumerate().filter_map(|(idx,maybe_value)|Some((idx,maybe_value.as_ref()?)))
 }
}
impl Slots<u16>{
 #[verus_verify(iterator_via_view(self.mapped_value()))]
 pub fn entries(&self)->impl Iterator<Item=Entry>+'_ {
  self.iter().map(|(index,&value)|Entry{index,value})
 }
}
"#;
const CONSUMER: &str = r#"use vstd::prelude::*;use factory_provider::{Slots,Entry};
#[verifier::infer] pub fn count<T>(s:&Slots<T>)->usize{s.iter().count()}
#[verifier::infer] pub fn mapped_count(s:&Slots<u16>)->usize{s.entries().count()}
#[verifier::infer] pub fn future(s:&Slots<u16>,round:u16)->bool{s.entries().any(|e|e.value>round)}
pub fn count_goal<T>(s:&Slots<T>)->(r:usize) ensures r==s.entries_value().len(),{count(s)}
pub fn map_goal(s:&Slots<u16>)->(r:usize) ensures r==s.mapped_value().len(),{mapped_count(s)}
pub fn any_goal(s:&Slots<u16>,round:u16)->(r:bool)
 ensures r==(s.mapped_value().filter(|e:Entry|e.value>round).len()>0),{future(s,round)}
pub fn noncopy(s:&Slots<Vec<u8>>)->(r:usize) ensures r==s.entries_value().len(),{count(s)}
"#;

fn provider(crates: &InferCrates, source: &str, options: &[&str]) -> std::process::Output {
    crates.build_source(
        "factory_provider",
        &format!("use verus_builtin::*;use verus_builtin_macros::*;{source}"),
        &[],
        options,
        true,
    )
}

#[test]
fn factory_goals_infer_original_callbacks_and_compose_across_cold_warm_imports() {
    let crates = InferCrates::new();
    for _ in 0..2 {
        let r = provider(&crates, PROVIDER, &["--infer-goals", "--no-cheating", "--output-json"]);
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let report = json_inference(&r);
        let fs = report["factory_assistance"].as_array().unwrap();
        assert_eq!(fs.len(), 2);
        assert!(fs.iter().all(|f| f["callbacks"] == 1));
        let r = crates.build(
            "factory_consumer",
            CONSUMER,
            &["factory_provider"],
            &["--no-cheating", "--output-json"],
        );
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let report = json_inference(&r);
        let fs = report["functions"].as_array().unwrap();
        assert_eq!(fs.len(), 3);
        for f in fs {
            for p in ["soundness", "maximality", "exactness"] {
                assert_eq!(f["proofs"][p]["status"], "proved", "{report}");
            }
        }
    }
    for (old, new) in [
        ("Some((idx,maybe_value.as_ref()?))", "Some((0,maybe_value.as_ref()?))"),
        ("Entry{index,value}", "Entry{index,value:0}"),
        ("Entry{index:e.0,value:*e.1}", "Entry{index:e.0,value:0}"),
    ] {
        let source = PROVIDER.replace(old, new);
        assert_ne!(source, PROVIDER);
        let r = provider(&crates, &source, &["--infer-goals", "--no-cheating"]);
        assert!(!r.status.success());
        assert!(!crates.vir("factory_provider").exists());
        assert!(
            String::from_utf8_lossy(&r.stderr).contains("precondition not satisfied"),
            "{}",
            String::from_utf8_lossy(&r.stderr)
        );
    }
}

#[test]
fn factory_goal_assistance_is_opt_in_and_cannot_publish_unchecked_factories() {
    let crates = InferCrates::new();
    let r = provider(&crates, PROVIDER, &["--infer-goals", "--no-cheating"]);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r = provider(&crates, PROVIDER, &["--no-cheating"]);
    assert!(!r.status.success());
    assert!(!crates.vir("factory_provider").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("precondition not satisfied"));
    let r = provider(&crates, PROVIDER, &["--infer-goals", "--no-verify"]);
    assert!(!r.status.success());
    assert!(!crates.vir("factory_provider").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("requires full-crate verification"));
}

#[test]
fn factory_goal_partial_and_mutating_callbacks_fail_closed() {
    let crates = InferCrates::new();
    for (body, diagnostic) in [
        ("Entry{index,value:value+1}", "factory callbacks require unrestricted safety"),
        ("{let mut x=value;x+=1;Entry{index,value:x}}", "immutable expressions"),
    ] {
        let source = PROVIDER.replace("Entry{index,value}", body);
        let r = provider(&crates, &source, &["--infer-goals", "--no-cheating"]);
        assert!(!r.status.success());
        assert!(!crates.vir("factory_provider").exists());
        assert!(
            String::from_utf8_lossy(&r.stderr).contains(diagnostic),
            "{}",
            String::from_utf8_lossy(&r.stderr)
        );
    }
}
