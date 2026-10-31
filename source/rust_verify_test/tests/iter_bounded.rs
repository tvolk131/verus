#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const PRELUDE: &str = "use vstd::prelude::*;use vstd::iter::{SparseIter,sparse_entries,count,count_where,find,map_sparse,collect};use vstd::std_specs::iter::IteratorSpec;";

#[test]
fn finite_sparse_traversal_count_search_and_construction() {
    let source = format!(
        r#"{PRELUDE} verus!{{
fn count_slots<T>(values:&[Option<T>])->(result:usize)
 ensures result==values@.count_prefix(|v:Option<T>|v.is_some(),values@.len()),
{{
 let iter=SparseIter::new(values);
 proof{{vstd::iter::lemma_sparse_count::<'_,T>(values@,0);}}
 count(iter)
}}
fn count_values(values:&[u16],round:u16)->(result:usize)
 ensures result==values@.as_ref().count_prefix(|v:&u16|*v==round,values@.len()),
{{let _=values.len();count_where(values.iter(),|value:&&u16|->(r:bool) ensures r==(**value==round),{{**value==round}},Ghost(|value:&u16|*value==round))}}
fn copy_entries<T>(values:&[Option<T>])->(result:Vec<(usize,&T)>)
 ensures result@==sparse_entries(values@,0),
{{collect(SparseIter::new(values))}}
fn first(values:&[u16],round:u16)->(r:Option<&u16>)
 ensures r.is_some()==exists|i:int|0<=i<values.len() && values@[i]>round,
{{find(values.iter(),|v:&&u16|->(b:bool) ensures b==(**v>round),{{**v>round}},Ghost(|v:&u16|*v>round))}}
fn project<T>(values:&[Option<(T,u16)>])->(result:Vec<Option<u16>>)
 ensures result.len()==values.len(),forall|i:int|0<=i<values.len() ==> result@[i]==match values@[i]{{None=>None,Some((_,r))=>Some(r)}},
{{map_sparse(values,|v:&(T,u16)|->(r:u16) ensures r==v.1,{{v.1}})}}
}}"#
    );
    verify_one_file("bounded_sparse_consumers", source, &["vstd", "--no-cheating"]).unwrap();
}

#[test]
fn callback_result_must_match_the_supplied_mathematical_predicate() {
    let source = format!(
        r#"{PRELUDE} verus!{{
fn wrong(values:&[u16]){{let _=values.len();let _=count_where(values.iter(),
 |v:&&u16|->(b:bool) ensures b==true,{{true}},Ghost(|v:&u16|false));}}
}}"#
    );
    let e = verify_one_file("dishonest_iterator_predicate", source, &["vstd", "--no-cheating"])
        .unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("precondition not satisfied")), "{:?}", e);
}

#[test]
fn generic_consumers_require_a_nonprophetic_termination_metric() {
    let source = format!(
        r#"{PRELUDE} verus!{{
fn wrong<I:Iterator>(iter:I)->usize
 requires iter.obeys_prophetic_iter_laws(),iter.remaining().len()<=usize::MAX as nat,
{{count(iter)}}
}}"#
    );
    let e =
        verify_one_file("iterator_missing_metric", source, &["vstd", "--no-cheating"]).unwrap_err();
    assert!(e.errors.iter().any(|d| d.message.contains("precondition not satisfied")), "{:?}", e);
}
