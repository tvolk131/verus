use super::*;

#[test]
fn borrowed_projection_chains_keep_complete_objects_across_imports() {
    let crates = InferCrates::new();
    let provider = r#"use vstd::prelude::*;
pub struct Buffers{pub left:Vec<u8>,pub right:Vec<u8>}
impl Buffers{
 #[verifier::infer] pub fn left(&self)->&Vec<u8>{&self.left}
}
pub struct Cached{data:Buffers,epoch:u8}
impl Cached{
 #[verifier::type_invariant] closed spec fn valid(&self)->bool{self.epoch>0}
 #[verifier::infer] pub fn buffers(&self)->&Buffers{&self.data}
 pub closed spec fn left_value(&self)->Vec<u8>{self.data.left}
 pub fn take_left(&self)->(r:&Vec<u8>) ensures r==self.left_value(),{self.buffers().left()}
}
"#;
    let consumer = r#"use vstd::prelude::*;
pub enum Error{Missing,Other}
#[verifier::infer] pub fn pass(c:borrow_provider::Cached)->Result<borrow_provider::Cached,Error>{Ok(c)}
pub fn goal(c:&borrow_provider::Cached)->(r:&Vec<u8>)
 ensures r==c.left_value(),
{c.take_left()}
pub fn repeated(c:&borrow_provider::Cached){
 let a=c.buffers().left();let b=c.buffers().left();assert(a==b);
}
pub fn preserved(c:borrow_provider::Cached)->(r:Result<borrow_provider::Cached,Error>)
 ensures r==Ok::<borrow_provider::Cached,Error>(c),
{pass(c)}
"#;
    for _ in 0..2 {
        for (name, source, deps, count) in [
            ("borrow_provider", provider, vec![], 2),
            ("borrow_consumer", consumer, vec!["borrow_provider"], 1),
        ] {
            let r = crates.build(name, source, &deps, &["--no-cheating", "--output-json"]);
            assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
            let report = json_inference(&r);
            let fs = report["functions"].as_array().unwrap();
            assert_eq!(fs.len(), count);
            for f in fs {
                for proof in ["soundness", "maximality", "exactness"] {
                    assert_eq!(f["proofs"][proof]["status"], "proved", "{report}");
                }
            }
        }
    }
    let wrong = provider.replace("{&self.left}", "{&self.right}");
    let r = crates.build("borrow_provider", &wrong, &[], &["--no-cheating"]);
    assert!(!r.status.success());
    assert!(!crates.vir("borrow_provider").exists());
    assert!(String::from_utf8_lossy(&r.stderr).contains("postcondition not satisfied"));
}
