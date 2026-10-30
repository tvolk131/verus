use super::*;

const SOURCE: &str = r#"use vstd::prelude::*;
#[verus_verify] pub struct Slots<T>(Vec<Option<T>>);
verus! {
impl<T> View for Slots<T> {type V=Seq<Option<T>>;
 closed spec fn view(&self)->Self::V {self.0@}}
impl<T> Slots<T> {
 pub open spec fn entries_value(&self)->Seq<(usize,&T)> {
  vstd::seq_update::sparse_entries(self@,|i:int|(i as usize,&self@[i].unwrap()))
 }
}
}
#[verus_verify] impl<T> Slots<T> {
 #[verus_infer] pub fn size(&self)->usize {self.0.len()}
 #[verus_infer] pub fn insert(&mut self,index:usize,value:T) {self.0[index]=Some(value);}
 #[verus_infer] pub fn with_size(n:usize)->Self where T:Clone {Slots(vec![None;n])}
 #[verus_verify(iterator_via_view(self.entries_value()))]
 pub fn iter(&self)->impl Iterator<Item=(usize,&T)> {
  self.0.iter().enumerate().filter_map(|(i,v)|Some((i,v.as_ref()?)))
 }
}
#[verus_verify] pub trait Stamp {fn stamp()->u64;}
#[verus_verify] pub struct Packet {slots:Slots<u16>,stamp:u64}
verus! {
impl View for Packet {type V=(Seq<Option<u16>>,u64);
 closed spec fn view(&self)->Self::V {(self.slots@,self.stamp)}}
pub fn pack<H:Stamp,T:Copy>(source:&Slots<(T,u16)>)->(result:Packet)
 ensures result@.0 == source@.map_values(|slot:Option<(T,u16)>|
  match slot {None=>None,Some((_,n))=>Some(n)}),
{
 let mut out=Slots::with_size(source.size());
 for (index,(_,value)) in source.iter() {out.insert(index,*value);}
 Packet {slots:out,stamp:H::stamp()}
}
}
"#;

#[test]
fn ordinary_constructor_keeps_local_fold_and_opaque_suffix_without_proof_hints() {
    let crates = InferCrates::new();
    for _ in 0..2 {
        let run = crates.build_source(
            "local_fold",
            SOURCE,
            &[],
            &["--infer-goals", "--no-cheating", "--output-json"],
            true,
        );
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        assert!(
            report["functions"]
                .as_array()
                .unwrap()
                .iter()
                .all(|f| f["required_checks_passed"] == true)
        );
        assert!(
            !report["functions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f["function"].as_str().unwrap().ends_with("::pack"))
        );
    }
    for source in [
        SOURCE.replace("out.insert(index,*value)", "out.insert(index,0)"),
        SOURCE.replace("out.insert(index,*value)", "out.insert(index+1,*value)"),
        // The temporary prefix analysis must not hide safety obligations in
        // the original suffix, including the empty-input case.
        SOURCE.replace("stamp:H::stamp()", "stamp:100 / source.size() as u64"),
    ] {
        let run = crates.build_source(
            "local_fold",
            &source,
            &[],
            &["--infer-goals", "--no-cheating", "--output-json"],
            true,
        );
        assert!(!run.status.success(), "wrong or unsafe constructor unexpectedly verified");
        assert!(!crates.vir("local_fold").exists());
        assert!(!String::from_utf8_lossy(&run.stderr).contains("panicked at"));
        assert!(
            json_inference(&run)["functions"]
                .as_array()
                .unwrap()
                .iter()
                .all(|f| f["required_checks_passed"] == true)
        );
    }
    let run = crates.build_source("local_fold", SOURCE, &[], &["--no-cheating"], true);
    assert!(!run.status.success(), "missing loop assistance unexpectedly proved the complete goal");
    assert!(!crates.vir("local_fold").exists());
}
