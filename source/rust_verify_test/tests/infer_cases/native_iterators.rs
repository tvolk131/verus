use super::*;
const PROVIDER: &str = r#"use vstd::prelude::*;use vstd::std_specs::iter::IteratorSpec;use vstd::std_specs::iter_adapters::*;
verus! {
broadcast use {vstd::std_specs::iter::group_iter_axioms,group_native_adapter_axioms,vstd::seq::group_seq_axioms,vstd::seq::group_seq_lemmas};
pub struct Slots<T>(Vec<Option<T>>);
impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}
impl<T> Slots<T>{
 pub closed spec fn entries_value<'a>(&'a self)->Seq<(usize,&'a T)>{flatten_options(Seq::new(self@.len(),|i:int|match self@[i]{None=>None,Some(v)=>Some((i as usize,&v))}))}
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
        assert_eq!(report["functions"].as_array().unwrap().len(), 3);
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
        ("s.iter().any(|(_,v)|{let mut x=*v;x+=1;x>round})", "pure scalar expressions"),
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
