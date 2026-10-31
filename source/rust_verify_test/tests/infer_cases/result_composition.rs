use super::*;

const PROVIDER: &str = r#"use vstd::prelude::*;
pub enum Error<T:Copy>{First(T,u16),Second(usize),Third(u16),Fourth(T),Fifth(bool),Sixth}
pub struct State<T:Copy>{pub token:T,pub count:usize}
impl<T:Copy> State<T>{
 #[verifier::infer] fn leaf(&self,kind:u8,round:u16)->Result<(),Error<T>>{
  if kind==0{return Err(Error::First(self.token,round));}
  if kind==1{return Err(Error::Second(self.count));}
  if kind==2{return Err(Error::Third(round));}
  if kind==3{return Err(Error::Fourth(self.token));}
  if kind==4{return Err(Error::Fifth(true));}
  if kind==5{return Err(Error::Sixth);}
  Ok(())
 }
 #[verifier::infer] pub fn chain(&self,first:u8,second:u8,round:u16)->Result<(),Error<T>>{
  self.leaf(first,round)?;self.leaf(second,round)?;Ok(())
 }
 #[verifier::infer] pub fn dispatch(&self,single:bool,first:u8,second:u8,round:u16)->Result<(),Error<T>>{
  match single{true=>self.leaf(first,round),false=>self.chain(first,second,round)}
 }
}
#[verifier::infer] pub fn preserve<T:Copy>(input:Result<T,Error<T>>)->Result<T,Error<T>>{Ok(input?)}
"#;

const CONSUMER: &str = r#"use vstd::prelude::*;
use result_provider::{Error,State};
#[verifier::infer] pub fn forward<T:Copy>(s:&State<T>,single:bool,first:u8,second:u8,round:u16)->Result<u16,Error<T>>{
 s.dispatch(single,first,second,round)?;Ok(round+1)
}
pub fn first_error<T:Copy>(s:&State<T>,round:u16)->(r:Result<u16,Error<T>>)
 ensures r==Err::<u16,Error<T>>(Error::First(s.token,round)),
{forward(s,false,0,1,round)}
pub fn second_error<T:Copy>(s:&State<T>)->(r:Result<u16,Error<T>>)
 ensures r==Err::<u16,Error<T>>(Error::Second(s.count)),
{forward(s,false,9,1,u16::MAX)}
pub fn single_branch<T:Copy>(s:&State<T>)->(r:Result<u16,Error<T>>)
 ensures r==Err::<u16,Error<T>>(Error::Third(u16::MAX)),
{forward(s,true,2,1,u16::MAX)}
pub fn success<T:Copy>(s:&State<T>,round:u16)->(r:Result<u16,Error<T>>)
 requires round<u16::MAX,
 ensures r==Ok::<u16,Error<T>>((round+1) as u16),
{forward(s,false,9,9,round)}
pub fn preserve<T:Copy>(input:Result<T,Error<T>>)->(r:Result<T,Error<T>>)
 ensures r==input,
{result_provider::preserve(input)}
"#;

#[test]
fn complete_result_composition_preserves_imported_errors_domains_and_precision() {
    let crates = InferCrates::new();
    for _ in 0..2 {
        for (name, source, deps, count) in [
            ("result_provider", PROVIDER, vec![], 4),
            ("result_consumer", CONSUMER, vec!["result_provider"], 1),
        ] {
            let run = crates.build(name, source, &deps, &["--no-cheating", "--output-json"]);
            assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
            let report = json_inference(&run);
            let functions = report["functions"].as_array().unwrap();
            assert_eq!(functions.len(), count);
            for f in functions {
                for check in ["soundness", "maximality", "exactness"] {
                    assert_eq!(f["proofs"][check]["status"], "proved", "{f}");
                }
            }
        }
    }
    // Error paths may bypass overflow; a successful path must still prove it safe.
    let unsafe_caller = CONSUMER.replace("requires round<u16::MAX,", "");
    let run =
        crates.build("result_consumer", &unsafe_caller, &["result_provider"], &["--no-cheating"]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("precondition not satisfied"));
    assert!(!crates.vir("result_consumer").exists());

    // A changed, correctly inferred error order must fail the independent policy.
    let reordered = PROVIDER.replace(
        "self.leaf(first,round)?;self.leaf(second,round)?;",
        "self.leaf(second,round)?;self.leaf(first,round)?;",
    );
    let run = crates.build("result_provider", &reordered, &[], &["--no-cheating"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let run = crates.build(
        "result_consumer",
        CONSUMER,
        &["result_provider"],
        &["--no-cheating", "--output-json"],
    );
    assert!(!run.status.success());
    assert_eq!(json_inference(&run)["status"], "caller_proof_failed");
    assert!(String::from_utf8_lossy(&run.stderr).contains("postcondition not satisfied"));
    assert!(!crates.vir("result_consumer").exists());
}
