#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const SIMPLE: &str = r#"use vstd::prelude::*;verus!{
struct History {values:Vec<u8>,tag:u8}
fn grow(h:&mut History,n:usize)
 ensures final(h).values@.len()==if old(h).values@.len()>n {old(h).values@.len() as int}else{n as int},
 final(h).values@.take(old(h).values@.len() as int)==old(h).values@,
 final(h).tag==old(h).tag,
 forall|i:int|old(h).values@.len()<=i<final(h).values@.len() ==> #[trigger] final(h).values@[i]==0,
{while h.values.len()<n {h.values.push(0);}}
}"#;
const STRUCTURED: &str = r#"use vstd::prelude::*;verus!{
struct Snapshot {items:Vec<u8>,round:u16}
fn advance(s:&Snapshot)->(r:Snapshot)
 requires s.round<65535,
 ensures r.round==s.round+1,r.items@==s.items@,
{Snapshot{items:s.items.clone(),round:s.round+1}}
struct History {values:Vec<Snapshot>,stamp:u8}
impl History {
 closed spec fn valid(&self)->bool {
  0<self.values@.len()<=65535 && forall|i:int|#![trigger self.values@[i]] 0<=i<self.values@.len() ==> self.values@[i].round==i+1
 }
 fn grow(&mut self,round:u16)
 requires old(self).valid(),round<65535,
 ensures final(self).valid(),final(self).stamp==old(self).stamp,
  final(self).values@.len()==if old(self).values@.len()>round as int+1 {old(self).values@.len() as int}else{round as int+1},
  final(self).values@.take(old(self).values@.len() as int)==old(self).values@,
 {
  while self.values.len()<=round as usize {
   let next=advance(self.values.last().unwrap());
   self.values.push(next);
  }
 }
}
}"#;
fn name(s: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    format!("growth_{:x}", h.finish())
}
fn succeeds(s: &str) {
    let d = verify_one_file(
        &name(s),
        s.into(),
        &["vstd", "--no-cheating", "--infer-goals", "--output-json"],
    )
    .unwrap();
    let r = &d.json_output.unwrap()["verus-infer"];
    assert_eq!(r["growth_loop_assistance"].as_array().unwrap().len(), 1, "{r}");
    assert!(
        r["functions"].as_array().unwrap().is_empty(),
        "ordinary proofs must not claim exact inference"
    );
}
#[test]
fn append_preserves_prefix_frames_and_describes_new_elements() {
    succeeds(SIMPLE);
}
#[test]
fn nested_collection_elements_compose_checked_transition_contracts() {
    succeeds(STRUCTURED);
}
#[test]
fn no_assistance_requires_manual_loop_proofs() {
    assert!(
        verify_one_file("unassisted_growth", SIMPLE.into(), &["vstd", "--no-cheating"]).is_err()
    );
}
#[test]
fn wrong_values_frames_prefixes_and_progress_are_rejected() {
    for s in [
        SIMPLE.replace("push(0)", "push(1)"),
        SIMPLE.replace("h.values.push(0);", "h.values.push(0);h.tag=0;"),
        SIMPLE.replace("h.values.push(0);", "h.values.push(0);h.values[0]=0;"),
        SIMPLE.replace("h.values.push(0);", "if false {h.values.push(0);}"),
        STRUCTURED.replace(
            "requires old(self).valid(),round<65535,",
            "requires old(self).valid(),round<=65535,",
        ),
        STRUCTURED.replace("round:s.round+1", "round:s.round"),
    ] {
        let e =
            verify_one_file(&name(&s), s, &["vstd", "--no-cheating", "--infer-goals"]).unwrap_err();
        assert!(
            e.errors.iter().any(|d| d.message.contains("invariant")
                || d.message.contains("postcondition")
                || d.message.contains("precondition")
                || d.message.contains("decreases")),
            "{:?}",
            e
        );
    }
}
#[test]
fn explicit_loop_proofs_are_preserved() {
    let s = SIMPLE.replace(
        "while h.values.len()<n",
        "while h.values.len()<n invariant false, decreases 0int, ",
    );
    let e =
        verify_one_file(&name(&s), s, &["vstd", "--no-cheating", "--infer-goals", "--output-json"])
            .unwrap_err();
    assert!(
        e.json_output.unwrap()["verus-infer"]["growth_loop_assistance"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(e.errors.iter().any(|d| d.message.contains("invariant")));
}
