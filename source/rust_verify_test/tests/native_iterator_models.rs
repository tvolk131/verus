#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const SPARSE: &str = r#"use vstd::prelude::*;
use vstd::std_specs::iter::IteratorSpec;
use vstd::std_specs::iter_adapters::*;
verus!{
broadcast use {vstd::std_specs::iter::group_iter_axioms,group_native_adapter_axioms,vstd::seq::group_seq_axioms,vstd::seq::group_seq_lemmas};
pub open spec fn slots<'a,T>(values:Seq<Option<T>>)->Seq<(usize,&'a T)>{
 flatten_options(Seq::new(values.len(),|i:int|match values[i]{None=>None,Some(v)=>Some((i as usize,&v))}))
}
pub fn entries<'a,T>(values:&'a [Option<T>])->(result:impl Iterator<Item=(usize,&'a T)>+'a)
 ensures finite_native_view(result,slots(values@)),result.obeys_prophetic_iter_laws(),native_count_supported(result),result.decrease().is_some(),result.decrease().unwrap()<=usize::MAX as nat,
 result.remaining().len()<=values.len(),result.will_return_none() ==> result.remaining()==slots(values@),
{
 let indexed=values.iter().enumerate();
 let f=|(idx,maybe_value):(usize,&'a Option<T>)|->(r:Option<(usize,&'a T)>)
  ensures r==match maybe_value {None=>None,Some(v)=>Some((idx,v))},
 {Some((idx,maybe_value.as_ref()?))};
 let result=indexed.filter_map(f);
 proof {
  filter_map_postcondition::<_,(usize,&'a T),_>(indexed,f,result);
  assert(filter_map_iter(result)==indexed);
  assert(result.obeys_prophetic_iter_laws());
  let full=Seq::new(values@.len(),|i:int|match values@[i]{None=>None,Some(v)=>Some((i as usize,&v))});
  assert(filter_map_outputs(result).is_prefix_of(full));
  flatten_options_prefix(filter_map_outputs(result),full);
  assert(result.will_return_none() ==> filter_map_outputs(result) =~= Seq::new(values.len() as nat,|i:int|match values@[i]{None=>None,Some(v)=>Some((i as usize,&v))}));
 }
 result
}
fn number<T>(values:&[Option<T>])->(r:usize) ensures r==slots(values@).len(),
{ entries(values).count() }
fn exhaust<T>(values:&[Option<T>]) { for _entry in entries(values) {} }
}
"#;

#[test]
fn sparse_native_chain_preserves_indices_and_complete_count() {
    verify_one_file("native_sparse_chain", SPARSE.into(), &["vstd", "--no-cheating"]).unwrap();
}

#[test]
fn a_wrong_sparse_callback_fails_the_supplied_index_contract() {
    let source =
        SPARSE.replace("Some((idx,maybe_value.as_ref()?))", "Some((0,maybe_value.as_ref()?))");
    assert_ne!(source, SPARSE);
    let e = verify_one_file("wrong_native_sparse_index", source, &["vstd", "--no-cheating"])
        .unwrap_err();
    assert!(
        e.errors.iter().any(|e| e.message.contains("postcondition not satisfied")
            || e.message.contains("unable to prove post-condition of closure")),
        "{:?}",
        e
    );
}

#[test]
fn count_rejects_unrestricted_iterator_implementations() {
    let source = r#"use vstd::prelude::*;use vstd::std_specs::iter::IteratorSpec;verus!{
fn count_unknown<I:Iterator>(iter:I)->usize
 requires iter.obeys_prophetic_iter_laws(),iter.decrease().is_some(),iter.remaining().len()<=usize::MAX as nat,
{iter.count()}
}"#;
    let e = verify_one_file("unrestricted_native_count", source.into(), &["vstd", "--no-cheating"])
        .unwrap_err();
    assert!(e.errors.iter().any(|e| e.message.contains("precondition not satisfied")), "{:?}", e);
}

#[test]
fn enumerate_requires_a_bounded_nonprophetic_metric() {
    let source = r#"use vstd::prelude::*;use vstd::std_specs::iter::IteratorSpec;use vstd::std_specs::iter_adapters::native_count_supported;verus!{
fn unknown<I:Iterator>(iter:I)
 requires native_count_supported(iter),iter.obeys_prophetic_iter_laws(),iter.decrease().is_some(),
{let _=iter.enumerate();}
}"#;
    let e =
        verify_one_file("unbounded_enumerate_counter", source.into(), &["vstd", "--no-cheating"])
            .unwrap_err();
    assert!(e.errors.iter().any(|e| e.message.contains("precondition not satisfied")), "{:?}", e);
}

#[test]
fn reference_next_algorithms_check_arithmetic_and_first_some_without_assumptions() {
    let source = r#"use vstd::prelude::*;use vstd::std_specs::iter::IteratorSpec;verus!{
broadcast use vstd::seq::group_seq_lemmas;
fn enumerate_step<I:Iterator>(iter:&mut I,index:&mut usize)->(result:Option<(usize,I::Item)>)
 requires old(iter).obeys_prophetic_iter_laws(),old(iter).decrease().is_some(),
          *old(index) as int+old(iter).decrease().unwrap()<=usize::MAX as nat,
 ensures result.is_some()==(old(iter).remaining().len()>0),
         result.is_some() ==> result.unwrap()==(*old(index),old(iter).remaining()[0]),
         *final(index) as int==*old(index) as int+if result.is_some(){1int}else{0int},
{
 match iter.next(){None=>None,Some(value)=>{let i=*index;*index+=1;Some((i,value))}}
}
fn filter_map_step<I:Iterator,B,F:Fn(I::Item)->Option<B>>(iter:I,f:F)->(result:Option<B>)
 requires iter.obeys_prophetic_iter_laws(),iter.decrease().is_some(),
          forall|i:int|0<=i<iter.remaining().len() ==> f.requires((#[trigger] iter.remaining()[i],)),
 ensures result.is_none() ==> forall|i:int|0<=i<iter.remaining().len() ==> f.ensures((#[trigger] iter.remaining()[i],),None),
         result.is_none() ==> iter.will_return_none(),
         result.is_some() ==> exists|i:int|0<=i<iter.remaining().len()
           && f.ensures((iter.remaining()[i],),result)
           && forall|j:int|0<=j<i ==> f.ensures((#[trigger] iter.remaining()[j],),None),
{
 let mut rest=iter;
 let ghost snapshot=iter;
 let ghost mut seen:nat=0;
 loop
  invariant snapshot==iter,rest.obeys_prophetic_iter_laws(),rest.decrease().is_some(),
    seen<=snapshot.remaining().len(),
    rest.remaining()==snapshot.remaining()[seen as int..],
    rest.will_return_none()==snapshot.will_return_none(),
    forall|i:int|0<=i<snapshot.remaining().len() ==> f.requires((#[trigger] snapshot.remaining()[i],)),
    forall|j:int|0<=j<seen ==> f.ensures((#[trigger] snapshot.remaining()[j],),None),
  decreases rest.decrease().unwrap(),
 {
  match rest.next(){
   None=>return None,
   Some(value)=>{
    let output=f(value);
    if output.is_some(){
     proof {
      assert(seen<snapshot.remaining().len());
      assert(f.ensures((snapshot.remaining()[seen as int],),output));
      assert(exists|i:int|0<=i<snapshot.remaining().len() && f.ensures((snapshot.remaining()[i],),output)
        && forall|j:int|0<=j<i ==> f.ensures((#[trigger] snapshot.remaining()[j],),None));
     }
     return output;
    }else{proof{seen=seen+1;}}
   }
  }
 }
}
}"#;
    verify_one_file("native_adapter_reference_next", source.into(), &["vstd", "--no-cheating"])
        .unwrap();
}

#[test]
fn native_map_count_delegates_without_claiming_mapper_exhaustion() {
    let source = r#"use vstd::prelude::*;use vstd::std_specs::iter::IteratorSpec;use vstd::std_specs::iter_adapters::*;verus!{
fn delegated<'a,F:FnMut(&'a u8)->u8>(values:&'a [u8],f:F)->(r:usize)
 requires forall|i:int|0<=i<values.len() ==> f.requires((#[trigger] &values@[i],)),
 ensures r==values.len(),
 { values.iter().map(f).count() }
fn no_completion<'a,F:FnMut(&'a u8)->u8>(values:&'a [u8],f:F)
 requires forall|i:int|0<=i<values.len() ==> f.requires((#[trigger] &values@[i],)),
 {
   let iter=values.iter().map(f);let ghost snapshot=iter;
   let count=iter.count();
   assert(snapshot.will_return_none());
 }
}"#;
    let error = verify_one_file("map_count_completion", source.into(), &["vstd", "--no-cheating"])
        .unwrap_err();
    assert_eq!(error.errors.len(), 1, "{:?}", error);
    assert!(error.errors[0].message.contains("assertion failed"), "{:?}", error);
}
