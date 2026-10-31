#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::{source_name, verify_exact};
use common::*;

fn name(source: &str) -> String {
    source_name("traversal", source)
}
fn exact(source: &str, count: usize) {
    verify_exact(&name(source), source.into(), count);
}
const PREFIX: &str = r#"use vstd::prelude::*;verus!{
struct Bag { values:Vec<u8>, stamp:u16 }
impl Bag {
 #[verifier::infer] fn bump(&mut self,n:usize) {
  let mut i=0usize;
  while i<n {self.values[i]+=1;i+=1;}
 }
 #[verifier::infer] fn reset(&mut self,n:usize) {
  let mut i=0usize;
  while i<n {self.values[i]=0;i=i+1;}
 }
}
fn goal(b:&mut Bag,n:usize)
 requires n<=old(b).values@.len(),
  forall|j:int|0<=j<n ==> old(b).values@[j]<255,
 ensures final(b).stamp==old(b).stamp,
  final(b).values@.len()==old(b).values@.len(),
  forall|j:int|0<=j<old(b).values@.len() ==> #[trigger] final(b).values@[j] as int ==
   if j<n {old(b).values@[j]+1} else {old(b).values@[j] as int},
{b.bump(n);}
fn empty(b:&mut Bag) ensures final(b).values@==old(b).values@ {b.bump(0);}
fn clear(b:&mut Bag,n:usize) requires n<=old(b).values@.len(),
 ensures final(b).stamp==old(b).stamp,
  final(b).values@.len()==old(b).values@.len(),
  forall|j:int|0<=j<old(b).values@.len() ==> #[trigger] final(b).values@[j] ==
   if j<n {0u8} else {old(b).values@[j]},
{b.reset(n);}
}"#;

#[test]
fn prefix_updates_have_exact_domain_complete_contents_and_frames() {
    exact(PREFIX, 2);
}

#[test]
fn signed_updates_and_empty_prefix_do_not_restrict_unvisited_arithmetic() {
    exact(
        r#"use vstd::prelude::*;verus!{
 struct Bag { values:Vec<i8> }
 impl Bag {
  #[verifier::infer] fn subtract(&mut self,n:usize,x:i8) {
   let mut i=0usize;while i<n {self.values[i]-=x;i+=1;}
  }
  #[verifier::infer] fn fill(&mut self,n:usize,x:i8) {
   let mut i=0usize;while i<n {self.values[i]=x+1;i+=1;}
  }
 }
 fn empty(b:&mut Bag) ensures final(b).values@==old(b).values@ {b.fill(0,127);}
 fn goal(b:&mut Bag,n:usize,x:i8)
 requires n<=old(b).values@.len(), x<127,
 ensures final(b).values@.len()==old(b).values@.len(),
  forall|j:int|0<=j<n ==> #[trigger] final(b).values@[j]==x+1,
 {b.fill(n,x);}
}"#,
        2,
    );
}

const TRANSFER: &str = r#"use vstd::prelude::*;verus!{
struct Accounts { balances:Vec<u64>, stamp:u8 }
impl Accounts {
 #[verifier::infer] fn transfer(&mut self,src:usize,dst:usize,amount:u64) {
  self.balances[src]-=amount;
  self.balances[dst]+=amount;
 }
 fn try_transfer(&mut self,src:usize,dst:usize,amount:u64)->(r:Result<(),u8>)
 ensures final(self).stamp==old(self).stamp,
  final(self).balances@.len()==old(self).balances@.len(),
  r.is_err() ==> final(self).balances@==old(self).balances@,
  r.is_ok() == (src<old(self).balances@.len() && dst<old(self).balances@.len()
   && amount<=old(self).balances@[src as int]
   && (src==dst || old(self).balances@[dst as int]<=u64::MAX-amount)),
  r.is_ok() ==> final(self).balances@==old(self).balances@.update(src as int,
   (old(self).balances@[src as int]-amount) as u64).update(dst as int,
   (if src==dst {old(self).balances@[src as int] as int} else {old(self).balances@[dst as int]+amount}) as u64),
 {
  if src>=self.balances.len() || dst>=self.balances.len() {return Err(0);}
  if self.balances[src]<amount {return Err(1);}
  if src!=dst && self.balances[dst]>u64::MAX-amount {return Err(2);}
  self.transfer(src,dst,amount);Ok(())
 }
}
fn same(a:&mut Accounts,i:usize,amount:u64)
 requires i<old(a).balances@.len(), amount<=old(a).balances@[i as int],
 ensures final(a).balances@==old(a).balances@, final(a).stamp==old(a).stamp,
{a.transfer(i,i,amount);}
}"#;
#[test]
fn transfer_handles_overlap_and_checked_errors_leave_state_unchanged() {
    exact(TRANSFER, 1);
}

#[test]
fn checks_after_mutation_do_not_establish_error_atomicity() {
    let source=TRANSFER.replace("if src!=dst && self.balances[dst]>u64::MAX-amount {return Err(2);}",
  "self.balances[src]-=amount; if src!=dst && self.balances[dst]>u64::MAX-amount {return Err(2);}");
    let e = verify_one_file(&name(&source), source, &["vstd", "--no-cheating"]).unwrap_err();
    assert!(e.notes.iter().any(|n| n.message.contains("output exactness: PROVED")), "{:?}", e);
    assert!(e.errors.iter().any(|e| e.message.contains("postcondition not satisfied")), "{:?}", e);
}
#[test]
fn wrong_caller_claims_and_missing_input_safety_are_rejected() {
    for source in [
        PREFIX.replace("old(b).values@[j]<255", "old(b).values@[j]<=255"),
        PREFIX.replace("{b.bump(n);}", "{b.reset(n);}"),
    ] {
        let e = verify_one_file(&name(&source), source, &["vstd", "--no-cheating"]).unwrap_err();
        assert_eq!(
            e.notes.iter().filter(|n| n.message.contains("output exactness: PROVED")).count(),
            2,
            "{:?}",
            e
        );
        assert!(
            e.errors.iter().any(|e| e.message.contains("precondition not satisfied")
                || e.message.contains("postcondition not satisfied")),
            "{:?}",
            e
        );
    }
}
#[test]
fn unsupported_traversals_fail_closed() {
    for body in [
        "let mut i=1usize;while i<n {self.values[i]=0;i+=1;}",
        "let mut i=0usize;while i<n {self.values[i]=0;i+=2;}",
        "let mut i=0usize;while i<n {self.values[i]=self.values[i+1];i+=1;}",
        "let mut i=0usize;while i<n {self.values[i]=0;self.stamp+=1;i+=1;}",
        "let mut i=0usize;while i<n {self.values[i]=0;break;}",
        "let mut i=0usize;while i<n invariant i<=n, {self.values[i]=0;i+=1;}",
        "let mut i=0usize;while i<n decreases n-i, {self.values[i]=0;i+=1;}",
        "let mut i=0usize;while i<n {let mut j=0usize;while j<n {self.values[j]=0;j+=1;}i+=1;}",
        "let mut i=0usize;while i<n {self.values[i]=0;i+=1;}let mut j=0usize;while j<n {self.values[j]=0;j+=1;}",
    ] {
        let source = format!(
            "use vstd::prelude::*;verus!{{struct Bag{{values:Vec<u8>,stamp:u8}}impl Bag{{#[verifier::infer] fn update(&mut self,n:usize){{{body}}}}}}}"
        );
        let e = verify_one_file(&name(&source), source, &["vstd", "--no-cheating"]).unwrap_err();
        assert!(e.errors.iter().any(|d| d.message.contains("Verus Infer unsupported")), "{:?}", e);
        assert!(
            !e.notes.iter().any(|d| d.message.contains("verified contracts available")),
            "{:?}",
            e
        );
    }
}

#[test]
fn traversal_termination_bypasses_and_sound_mode_are_rejected() {
    for (source, options) in [
        (
            PREFIX.replace(
                "#[verifier::infer]",
                "#[verifier::infer] #[verifier::assume_termination]",
            ),
            vec!["vstd", "--no-cheating"],
        ),
        (PREFIX.into(), vec!["vstd", "exec_allows_no_decreases_clause"]),
        (
            PREFIX.replace("#[verifier::infer]", "#[verifier::infer(sound)]"),
            vec!["vstd", "--no-cheating"],
        ),
    ] {
        let e = verify_one_file(&name(&source), source, &options).unwrap_err();
        assert!(e.errors.iter().any(|d| d.message.contains("Verus Infer unsupported")), "{:?}", e);
        assert!(
            !e.notes.iter().any(|d| d.message.contains("verified contracts available")),
            "{:?}",
            e
        );
    }
}

#[test]
fn collection_length_bounds_and_index_dependent_values_are_exact() {
    exact(
        r#"use vstd::prelude::*;verus!{
        struct Indexes { values:Vec<usize>, stamp:bool }
        impl Indexes {
            #[verifier::infer] fn ordinal(&mut self) {
                let mut i=0usize;
                while i<self.values.len() {self.values[i]=i+1;i+=1;}
            }
        }
        fn check(b:&mut Indexes)
            ensures final(b).stamp==old(b).stamp,
                final(b).values@.len()==old(b).values@.len(),
                forall|i:int|0<=i<old(b).values@.len() ==> #[trigger] final(b).values@[i]==i+1,
        {b.ordinal();}
    }"#,
        1,
    );
}
