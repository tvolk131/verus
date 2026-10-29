use super::*;

const PROVIDER: &str = r#"use vstd::prelude::*;
pub enum Error{Number(u16),Other}
pub struct State{pub fail:bool}
impl State{
 #[verifier::infer] pub fn map(&self,x:u16)->Result<u8,Error>{
  let input:Result<u8,u16>=if self.fail{Err(x)}else{Ok(7)};
  input.map_err(|e|Error::Number(e+1))
 }
}
"#;
const CONSUMER: &str = r#"use vstd::prelude::*;
use mapping_provider::{State,Error};
pub enum Outer{Wrapped(Error),Other}
#[verifier::infer] pub fn bridge(s:&State,x:u16)->Result<u8,Outer>{
 s.map(x).map_err(|e|Outer::Wrapped(e))
}
pub fn success()->(r:Result<u8,Outer>) ensures r==Ok::<u8,Outer>(7),
{bridge(&State{fail:false},u16::MAX)}
pub fn error(x:u16)->(r:Result<u8,Outer>) requires x<u16::MAX,
 ensures r==Err::<u8,Outer>(Outer::Wrapped(Error::Number((x+1) as u16))),
{bridge(&State{fail:true},x)}
"#;

#[test]
fn error_mapping_checks_conditional_safety_complete_payloads_and_imports() {
    let crates = InferCrates::new();
    for _ in 0..2 {
        for (name, source, deps) in [
            ("mapping_provider", PROVIDER, vec![]),
            ("mapping_consumer", CONSUMER, vec!["mapping_provider"]),
        ] {
            let run = crates.build(name, source, &deps, &["--no-cheating", "--output-json"]);
            assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
            let report = json_inference(&run);
            let fs = report["functions"].as_array().unwrap();
            assert_eq!(fs.len(), 1);
            for proof in ["soundness", "maximality", "exactness"] {
                assert_eq!(fs[0]["proofs"][proof]["status"], "proved", "{report}");
            }
        }
    }
    let unsafe_caller = CONSUMER.replace("requires x<u16::MAX,", "");
    let run =
        crates.build("mapping_consumer", &unsafe_caller, &["mapping_provider"], &["--no-cheating"]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("precondition not satisfied"));
    assert!(!crates.vir("mapping_consumer").exists());
    let changed = PROVIDER.replace("Error::Number(e+1)", "Error::Other");
    let run = crates.build("mapping_provider", &changed, &[], &["--no-cheating"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let run = crates.build(
        "mapping_consumer",
        CONSUMER,
        &["mapping_provider"],
        &["--no-cheating", "--output-json"],
    );
    assert!(!run.status.success());
    assert_eq!(json_inference(&run)["status"], "caller_proof_failed");
    assert!(String::from_utf8_lossy(&run.stderr).contains("postcondition not satisfied"));
    assert!(!crates.vir("mapping_consumer").exists());
}

#[test]
fn mapped_errors_preserve_captured_record_clones_and_nested_collection_payloads() {
    let crates = InferCrates::new();
    let provider = r#"use vstd::prelude::*;
#[verus_verify(clone_via_view)]
#[derive(Clone)] pub struct Rows(Vec<u8>);
verus!{
 broadcast use {vstd::std_specs::clone::lemma_cloned_identity,vstd::std_specs::clone::lemma_cloned_view,vstd::seq::group_seq_axioms,vstd::seq::group_seq_lemmas};
 impl View for Rows{type V=Seq<u8>;closed spec fn view(&self)->Self::V{self.0@}}
 #[derive(Clone)] pub struct Snapshot{pub rows:Rows,pub epoch:u16}
 impl Snapshot{pub open spec fn __verus_clone_relation(&self,r:Self)->bool{r.rows@ =~= self.rows@ &&r.epoch==self.epoch}}
 pub enum Error{Snapshot(Snapshot,u16),Other}
 #[verifier::infer] pub fn map(fail:bool,s:&Snapshot,x:u16)->Result<u8,Error>{
  let input:Result<u8,u16>=if fail{Err(x)}else{Ok(7)};
  input.map_err(|e|if e==0{Error::Other}else{Error::Snapshot(s.clone(),e)})
 }
}"#;
    let consumer = r#"use vstd::prelude::*;use snapshot_provider::{Snapshot,Error};
pub enum Outer{Wrapped(Error),Other}
#[verifier::infer] pub fn wrap(fail:bool,s:&Snapshot,x:u16)->Result<u8,Outer>{
 snapshot_provider::map(fail,s,x).map_err(|e|Outer::Wrapped(e))
}
pub fn check(s:&Snapshot,x:u16)->(r:Result<u8,Outer>)
 requires x>0,
 ensures r.is_err(),r.get_Err_0() is Wrapped,r.get_Err_0()->Wrapped_0 is Snapshot,
  r.get_Err_0()->Wrapped_0->Snapshot_0.rows@==s.rows@,
  r.get_Err_0()->Wrapped_0->Snapshot_0.epoch==s.epoch,
  r.get_Err_0()->Wrapped_0->Snapshot_1==x,
{wrap(true,s,x)}
pub fn empty(s:&Snapshot)->(r:Result<u8,Outer>) ensures r.is_err(),r.get_Err_0() is Wrapped,r.get_Err_0()->Wrapped_0 is Other,{wrap(true,s,0)}
pub fn success(s:&Snapshot)->(r:Result<u8,Outer>) ensures r.is_ok(),r.get_Ok_0()==7,{wrap(false,s,0)}
"#;
    for _ in 0..2 {
        for (name, source, deps) in [
            ("snapshot_provider", provider, vec![]),
            ("snapshot_consumer", consumer, vec!["snapshot_provider"]),
        ] {
            let run = crates.build(name, source, &deps, &["--no-cheating", "--output-json"]);
            assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
            let r = json_inference(&run);
            for f in r["functions"].as_array().unwrap() {
                for p in ["soundness", "maximality", "exactness"] {
                    assert_eq!(f["proofs"][p]["status"], "proved", "{r}");
                }
            }
        }
    }
    let changed = provider.replace("s.clone(),e)", "s.clone(),0)");
    let run = crates.build("snapshot_provider", &changed, &[], &["--no-cheating"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let run =
        crates.build("snapshot_consumer", consumer, &["snapshot_provider"], &["--no-cheating"]);
    assert!(!run.status.success());
    assert!(!crates.vir("snapshot_consumer").exists());
    assert!(String::from_utf8_lossy(&run.stderr).contains("postcondition not satisfied"));
    // A checked but weak Clone contract cannot justify the closure's proposed
    // collection/frame equation, even though the actual derive still copies.
    let weak = provider.replace("r.rows@ =~= self.rows@ &&r.epoch==self.epoch", "true");
    let run = crates.build("snapshot_provider", &weak, &[], &["--no-cheating"]);
    assert!(!run.status.success());
    assert!(!crates.vir("snapshot_provider").exists());
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("post-condition of closure"),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
}
