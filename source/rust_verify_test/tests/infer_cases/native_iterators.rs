use super::*;
const PROVIDER: &str = r#"use vstd::prelude::*;use vstd::std_specs::iter::IteratorSpec;use vstd::std_specs::iter_adapters::*;
verus! {
broadcast use {vstd::std_specs::iter::group_iter_axioms,group_native_adapter_axioms,vstd::seq::group_seq_axioms,vstd::seq::group_seq_lemmas};
pub struct Slots<T>(Vec<Option<T>>);
impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}
impl<T> Slots<T>{
 pub open spec fn entries_value<'a>(&'a self)->Seq<(usize,&'a T)>{vstd::seq_update::sparse_entries(self@,|i:int|(i as usize,&self@[i].unwrap()))}
 pub fn iter<'a>(&'a self)->(result:impl Iterator<Item=(usize,&'a T)>+'a)
 ensures finite_native_view(result,self.entries_value()),
 {
  let values=self.0.as_slice();
  let indexed=values.iter().enumerate();
  let f=|(idx,maybe_value):(usize,&'a Option<T>)|->(r:Option<(usize,&'a T)>)
   ensures r==match maybe_value{None=>None,Some(v)=>Some((idx,v))},
  {Some((idx,maybe_value.as_ref()?))};
  let result=indexed.filter_map(f);
  proof{
   filter_map_postcondition::<_,(usize,&'a T),_>(indexed,f,result);
   let full=Seq::new(values@.len(),|i:int|match values@[i]{None=>None,Some(v)=>Some((i as usize,&v))});
   vstd::seq_update::lemma_sparse_entries_eq(self@,|i:int|(i as usize,&self@[i].unwrap()),full);
   assert(self.entries_value()==flatten_options(full));
   assert(filter_map_outputs(result).is_prefix_of(full));
   flatten_options_prefix(filter_map_outputs(result),full);
   assert(result.will_return_none() ==> filter_map_outputs(result) =~= full);
  }
  result
 }

}

}
"#;

#[test]
fn native_factory_views_compose_across_cold_and_warm_exports() {
    let crates = InferCrates::new();
    let consumer = r#"use vstd::prelude::*;
 #[verifier::infer] pub fn count<T>(s:&native_factory::Slots<T>)->usize{s.iter().count()}
 pub fn goal<T>(s:&native_factory::Slots<T>)->(r:usize) ensures r==s.entries_value().len(),{count(s)}
 #[verifier::infer] pub fn filtered(s:&native_factory::Slots<u16>,round:u16)->usize{
     s.iter().filter(|&(_,value)|*value==round-1).count()
 }
 #[verifier::infer] pub fn future(s:&native_factory::Slots<u16>,round:u16)->bool{
     s.iter().any(|(_,value)|*value>round-1)
 }
 #[verifier::infer] pub fn partial_filter(s:&native_factory::Slots<u16>,round:u16)->usize{
     s.iter().filter(|&(_,value)|*value-1>round).count()
 }
 pub fn partial_goal(s:&native_factory::Slots<u16>,round:u16)->(r:usize)
     requires forall|i:int|0<=i<s.entries_value().len() ==> *s.entries_value()[i].1>0,
     ensures r==s.entries_value().filter(|e:(usize,&u16)|((*e.1 as int-1) as u16)>round).len(),
 {partial_filter(s,round)}
 pub fn filtered_goal(s:&native_factory::Slots<u16>,round:u16)->(r:usize)
     requires round>0,
     ensures r==s.entries_value().filter(|e:(usize,&u16)|*e.1==round as int-1).len(),
 {filtered(s,round)}
 pub fn future_goal(s:&native_factory::Slots<u16>,round:u16)->(r:bool)
     requires round>0,
     ensures r==(s.entries_value().filter(|e:(usize,&u16)|*e.1>round as int-1).len()>0),
 {future(s,round)}
 pub fn empty_zero(s:&native_factory::Slots<u16>) requires s.entries_value().len()==0,
 {let count=filtered(s,0);let any=future(s,0);assert(count==0);assert(!any);}

 "#;
    for _ in 0..2 {
        let r = crates.build("native_factory", PROVIDER, &[], &["--no-cheating", "--output-json"]);
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let r = crates.build(
            "native_count",
            consumer,
            &["native_factory"],
            &["--no-cheating", "--output-json"],
        );
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let report = json_inference(&r);
        assert_eq!(report["functions"].as_array().unwrap().len(), 4);
        for f in report["functions"].as_array().unwrap() {
            for p in ["soundness", "maximality", "exactness"] {
                assert_eq!(f["proofs"][p]["status"], "proved");
            }
        }
    }
    let r = crates.build("native_factory", PROVIDER, &[], &["--no-verify"]);
    assert!(!r.status.success());
    assert!(!crates.vir("native_factory").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("requires full-crate verification"));
    let wrong =
        PROVIDER.replace("Some((idx,maybe_value.as_ref()?))", "Some((0,maybe_value.as_ref()?))");
    let r = crates.build("native_factory", &wrong, &[], &["--no-cheating"]);
    assert!(!r.status.success());
    assert!(!crates.vir("native_factory").exists());
    assert!(
        String::from_utf8_lossy(&r.stderr).contains("postcondition not satisfied")
            || String::from_utf8_lossy(&r.stderr).contains("assertion failed")
            || String::from_utf8_lossy(&r.stderr)
                .contains("unable to prove post-condition of closure"),
        "{}",
        String::from_utf8_lossy(&r.stderr)
    );
}

#[test]
fn native_short_circuit_partial_callbacks_fail_closed() {
    let crates = InferCrates::new();
    let r = crates.build("native_factory", PROVIDER, &[], &["--no-cheating"]);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    for (body, diagnostic) in [
        ("s.iter().any(|(_,v)|*v - 1 > 2)", "item-independent callback safety"),
        ("s.iter().filter(|&(_,v)|*v==round-1).any(|(_,v)|*v>0)", "search after a partial filter"),
        ("s.iter().any(|(_,v)|{let mut x=*v;x+=1;x>round})", "immutable expressions"),
    ] {
        let source = format!(
            "use vstd::prelude::*;#[verifier::infer] pub fn search(s:&native_factory::Slots<u16>,round:u16)->bool{{{body}}}"
        );
        let r =
            crates.build("unsupported_search", &source, &["native_factory"], &["--no-cheating"]);
        assert!(!r.status.success());
        assert!(!crates.vir("unsupported_search").exists());
        assert!(
            String::from_utf8_lossy(&r.stderr).contains(diagnostic),
            "{}",
            String::from_utf8_lossy(&r.stderr)
        );
    }
}

fn mutation_provider() -> String {
    let provider = PROVIDER.replace(
        "ensures finite_native_view(result,self.entries_value()),",
        "ensures finite_native_view(result,self.entries_value()), forall|i:int|0<=i<self.entries_value().len() ==> #[trigger] self.entries_value()[i].0<self@.len(),",
    ).replace("   assert(filter_map_outputs(result).is_prefix_of(full));",r#"
   assert(filter_map_outputs(result).is_prefix_of(full));
   assert forall|k:int|0<=k<self.entries_value().len() implies #[trigger] self.entries_value()[k].0<self@.len() by {
     flatten_options_witness(full,k);
     let i=choose|i:int|0<=i<full.len() && full[i]==Some(flatten_options(full)[k]);
     assert(full[i].unwrap().0==i);
   }
"#);
    provider
        + r#"verus! {
impl<T:Copy> Slots<T> {
 #[verifier::infer] pub fn with_size(size:usize)->Self {Self(vec![None;size])}
 #[verifier::infer] pub fn insert(&mut self,index:usize,value:T){self.0[index]=Some(value);}
 #[verifier::infer] pub fn size(&self)->usize{self.0.len()}
}
pub broadcast proof fn __verus_infer_view_len_bound<T>(s:&Slots<T>)
 ensures #[trigger] s@.len() <= usize::MAX,
{vstd::std_specs::vec::axiom_spec_len(&s.0);}
}"#
}

const COPY_CONSUMER: &str = r#"use vstd::prelude::*;use native_mutation::Slots;
verus! {
broadcast use {vstd::seq::group_seq_axioms,vstd::seq::group_seq_lemmas,vstd::std_specs::iter_adapters::group_native_adapter_axioms};
#[verifier::infer] pub fn copied<T:Copy>(source:&Slots<T>)->Slots<T>{
 let mut out=Slots::with_size(source.size());
 for (index,value) in source.iter(){out.insert(index,*value);}
 out
}
pub fn goal<T:Copy>(source:&Slots<T>)->(result:Slots<T>)
 ensures result@==vstd::seq_update::scatter(source.entries_value(),Seq::new(source@.len(),|i:int|None::<T>),|entry:(usize,&T)|entry.0 as int,|entry:(usize,&T)|Some(*entry.1)),
{copied(source)}
}"#;

#[test]
fn total_native_folds_compose_mutable_summaries_across_cold_and_warm_exports() {
    let crates = InferCrates::new();
    for _ in 0..2 {
        let r = crates.build(
            "native_mutation",
            &mutation_provider(),
            &[],
            &["--no-cheating", "--output-json"],
        );
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let r = crates.build(
            "native_copy",
            COPY_CONSUMER,
            &["native_mutation"],
            &["--infer-goals", "--no-cheating", "--output-json"],
        );
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let report = json_inference(&r);
        let functions = report["functions"].as_array().unwrap();
        assert_eq!(functions.len(), 1);
        for proof in ["soundness", "maximality", "exactness"] {
            assert_eq!(functions[0]["proofs"][proof]["status"], "proved");
        }
        assert!(!functions[0]["loop_proofs"].as_array().unwrap().is_empty());
    }
    let unsafe_index =
        COPY_CONSUMER.replace("out.insert(index,*value)", "out.insert(index+1,*value)");
    let r = crates.build("native_copy", &unsafe_index, &["native_mutation"], &["--no-cheating"]);
    assert!(!r.status.success());
    assert!(!crates.vir("native_copy").exists());
    let error = String::from_utf8_lossy(&r.stderr);
    assert!(
        error.contains("invariant not satisfied") || error.contains("precondition not satisfied"),
        "{}",
        error
    );
    assert!(!error.contains("unsupported") && !error.contains("panicked at"), "{}", error);
}

#[test]
fn native_fold_observations_are_demanded_and_checked() {
    let crates = InferCrates::new();
    let r = crates.build("native_mutation", &mutation_provider(), &[], &["--no-cheating"]);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let source = COPY_CONSUMER.replace(
        "{copied(source)}",
        r#"{copied(source)}
pub fn natural_goal<T:Copy>(source:&Slots<T>)->(result:Slots<T>)
 ensures result@.len()==source@.len(),
 forall|i:int|0<=i<source@.len() ==> #[trigger] result@[i]==source@[i],
{copied(source)}
pub fn unrequested<T:Copy>(source:&Slots<T>){let _=copied(source);}
"#);
    for _ in 0..2 {
        let r = crates.build("native_copy", &source, &["native_mutation"],
            &["--infer-goals", "--no-cheating", "--output-json"]);
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let report = json_inference(&r);
        assert!(report["goal_assistance"].as_array().unwrap().iter().any(|row|
            row["strategy"] == "sequence_fold_observation"));
        assert!(!report["goal_assistance"].as_array().unwrap().iter().any(|row|
            row["strategy"] == "sequence_fold_observation" && row["function"].as_str().unwrap().ends_with("unrequested")));
        for proof in ["soundness", "maximality", "exactness"] {
            assert_eq!(report["functions"][0]["proofs"][proof]["status"], "proved");
        }
    }
    let r = crates.build("native_copy", &source, &["native_mutation"], &["--no-cheating","--output-json"]);
    assert!(!r.status.success());
    assert!(!crates.vir("native_copy").exists());
    for proof in ["soundness","maximality","exactness"] {
        assert_eq!(json_inference(&r)["functions"][0]["proofs"][proof]["status"],"proved");
    }
    let wrong = source.replace("out.insert(index,*value)", "out.insert(index,None::<T>.unwrap())");
    let r = crates.build("native_copy", &wrong, &["native_mutation"], &["--infer-goals", "--no-cheating"]);
    assert!(!r.status.success());
    assert!(!crates.vir("native_copy").exists());
    let false_goal = source.replace("result@[i]==source@[i]", "result@[i]!=source@[i]");
    let r = crates.build("native_copy", &false_goal, &["native_mutation"], &["--infer-goals", "--no-cheating"]);
    assert!(!r.status.success());
    assert!(!crates.vir("native_copy").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("postcondition not satisfied"));
}

#[test]
fn checked_view_queries_compose_with_constructed_receivers() {
    let crates = InferCrates::new();
    let provider = mutation_provider() + r#"verus! {impl<T> Slots<T>{
 pub open spec fn view_size(&self)->usize{self@.len() as usize}
 pub fn view_size_checked(&self)->(r:usize) ensures r==self.view_size(),{self.0.len()}
}}"#;
    let source = r#"use vstd::prelude::*;use native_mutation::Slots;
verus! {
pub struct Made{pub slots:Slots<u8>,pub count:usize}
#[verifier::infer]pub fn constructed(size:usize)->Made{
 let slots=Slots::with_size(size);let n=slots.view_size_checked();Made{slots,count:n}
}
#[verifier::infer]pub fn temporary(size:usize)->Made{
 let n=Slots::<u8>::with_size(size).view_size_checked();Made{slots:Slots::with_size(size),count:n}
}
pub fn goal(size:usize){let result=constructed(size);assert(result.count==size);
 let result=temporary(size);assert(result.count==size);}
}"#;
    for _ in 0..2 {
        let r=crates.build("native_mutation", &provider, &[], &["--infer-goals","--no-cheating"]);
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let r=crates.build("view_query", source, &["native_mutation"], &["--no-cheating","--output-json"]);
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        for f in json_inference(&r)["functions"].as_array().unwrap(){
            for proof in ["soundness","maximality","exactness"] {assert_eq!(f["proofs"][proof]["status"],"proved");}
        }
    }
    let hidden=provider.replace("pub open spec fn view_size", "pub closed spec fn view_size");
    let r=crates.build("native_mutation", &hidden, &[], &["--infer-goals","--no-cheating"]);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let r=crates.build("view_query", source, &["native_mutation"], &["--no-cheating"]);
    assert!(!r.status.success());assert!(!crates.vir("view_query").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("visible observation body"));
    let wrong=provider.replace("{self.0.len()}","{0}");
    let r=crates.build("native_mutation", &wrong, &[], &["--infer-goals","--no-cheating"]);
    assert!(!r.status.success());assert!(!crates.vir("native_mutation").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("postcondition not satisfied"));
}

#[test]
fn owned_constructors_compose_complete_mutable_views_without_a_loop() {
    let crates = InferCrates::new();
    let r = crates.build("native_mutation", &mutation_provider(), &[], &["--no-cheating"]);
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let consumer = r#"use vstd::prelude::*;use native_mutation::Slots;verus!{
#[verifier::infer] pub fn one(size:usize,index:usize,value:u16)->Slots<u16>{
 let mut out=Slots::with_size(size);out.insert(index,value);out
}
pub fn goal(size:usize,index:usize,value:u16)->(r:Slots<u16>)
 requires index<size,
 ensures r@==Seq::new(size as nat,|i:int|None::<u16>).update(index as int,Some(value)),
{one(size,index,value)}
}"#;
    let r = crates.build(
        "native_one",
        consumer,
        &["native_mutation"],
        &["--no-cheating", "--output-json"],
    );
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let report = json_inference(&r);
    for proof in ["soundness", "maximality", "exactness"] {
        assert_eq!(report["functions"][0]["proofs"][proof]["status"], "proved");
    }
}

#[test]
fn requested_map_values_equations_enable_checked_sparse_projection(){
 let crates=InferCrates::new();
 let r=crates.build("native_mutation",&mutation_provider(),&[],&["--no-cheating"]);
 assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
 let source=COPY_CONSUMER.replace("{copied(source)}",r#"{copied(source)}
pub fn projected<T:Copy>(source:&Slots<T>)->(result:Slots<T>)
 ensures result@ == source@.map_values(|slot:Option<T>|match slot {Some(v)=>Some(v),None=>None}),
{copied(source)}
pub fn no_request<T:Copy>(source:&Slots<T>){let _=copied(source);}
"#);
 for _ in 0..2 {
  let r=crates.build("native_copy",&source,&["native_mutation"],&["--infer-goals","--no-cheating","--output-json"]);
  assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
  let report=json_inference(&r);
  let goals=report["goal_assistance"].as_array().unwrap();
  assert!(goals.iter().any(|g|g["function"].as_str().unwrap().ends_with("projected") && g["strategy"]=="sequence_fold_observation"));
  assert!(!goals.iter().any(|g|g["function"].as_str().unwrap().ends_with("no_request")));
 }
 let wrong=source.replace("Some(v)=>Some(v)","Some(v)=>None");
 let r=crates.build("native_copy",&wrong,&["native_mutation"],&["--infer-goals","--no-cheating","--output-json"]);
 assert!(!r.status.success());assert!(!crates.vir("native_copy").exists());
 assert!(json_inference(&r)["functions"].as_array().unwrap().iter().all(|f|f["required_checks_passed"]==true));
}

#[test]
fn entry_independent_native_fold_readiness_gets_an_internal_trigger(){
 let crates=InferCrates::new();
 let r=crates.build("native_mutation",&mutation_provider(),&[],&["--no-cheating"]);
 assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
 let source=COPY_CONSUMER.replace("out.insert(index,*value)","out.insert(0,*value)");
 let r=crates.build("native_copy",&source,&["native_mutation"],&["--infer-goals","--no-cheating","--output-json"]);
 assert!(!r.status.success());assert!(!crates.vir("native_copy").exists());
 let error=String::from_utf8_lossy(&r.stderr);
 assert!(!error.contains("infer triggers") && !error.contains("unsupported") && !error.contains("panicked at"),"{}",error);
 assert!(error.contains("postcondition not satisfied"),"{}",error);
 assert!(json_inference(&r)["functions"].as_array().unwrap().iter().all(|f|f["required_checks_passed"]==true));
}

#[test]
fn composed_native_filters_keep_authored_predicate_vocabulary() {
    let crates=InferCrates::new();
    let provider=crates.build("observation_factory",PROVIDER,&[],&["--no-cheating"]);
    assert!(provider.status.success(),"{}",String::from_utf8_lossy(&provider.stderr));
    let consumer=r#"use vstd::prelude::*;
    pub struct Item{stamp:u16,flag:bool}
    impl Item{pub closed spec fn stamp_value(&self)->u16{self.stamp}}
    #[verifier::infer] pub fn matching(s:&observation_factory::Slots<Item>,stamp:u16)->usize{
        s.iter().filter(|&(_,value)|value.stamp==stamp).count()
    }
    #[verifier::infer] pub fn relay(s:&observation_factory::Slots<Item>,stamp:u16)->usize{matching(s,stamp)}
    fn goal(s:&observation_factory::Slots<Item>,stamp:u16)->(r:usize)
        ensures r==s.entries_value().filter(|e:(usize,&Item)|e.1.stamp==stamp).len(),
    {relay(s,stamp)}
    "#;
    for _ in 0..2 {
        let r=crates.build("observation_filter",consumer,&["observation_factory"],&["--no-cheating","--output-json"]);
        assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
        assert!(json_inference(&r)["functions"].as_array().unwrap().iter().all(|f|f["required_checks_passed"]==true));
    }
    let wrong=consumer.replace("value.stamp==stamp","value.stamp!=stamp");
    let r=crates.build("observation_filter",&wrong,&["observation_factory"],&["--no-cheating","--output-json"]);
    assert!(!r.status.success());assert!(!crates.vir("observation_filter").exists());
    assert!(json_inference(&r)["functions"].as_array().unwrap().iter().all(|f|f["required_checks_passed"]==true));
}
