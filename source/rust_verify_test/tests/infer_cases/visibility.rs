use super::*;

const COORDINATE: &str = r#"use vstd::prelude::*;
verus! {
pub mod coordinate {
    use super::*;
    pub struct Coord { round: u16, index: usize }
    impl View for Coord {
        type V = (u16, usize);
        closed spec fn view(&self) -> Self::V { (self.round, self.index) }
    }
    impl Coord {
        #[verifier::infer]
        pub fn new(round: u16, index: usize) -> Self { Self { round, index } }
    }
}
}
"#;

#[test]
fn private_value_constructor_contracts_use_a_checked_public_view_across_crates() {
    let crates = InferCrates::new();
    let client = r#"use vstd::prelude::*;
verus! {
pub fn build(round:u16,index:usize)->(result: coordinate_provider::coordinate::Coord)
    ensures result@==(round,index),
{coordinate_provider::coordinate::Coord::new(round,index)}
}
"#;
    for _ in 0..2 {
        let run = crates.build_source(
            "coordinate_provider",
            COORDINATE,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let fs = json_inference(&run)["functions"].as_array().unwrap().clone();
        assert_eq!(fs.len(), 1);
        assert!(fs[0]["required_checks_passed"] == true);
        let run = crates.build_source(
            "coordinate_client",
            client,
            &["coordinate_provider"],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    }
    let wrong = client.replace("result@==(round,index)", "result@==(round,0usize)");
    let run = crates.build_source(
        "coordinate_client",
        &wrong,
        &["coordinate_provider"],
        &["--no-cheating"],
        true,
    );
    assert!(!run.status.success());
    assert!(!crates.vir("coordinate_client").exists());
    // With no public observation the inferred raw constructor is subject to
    // exactly the same visibility rejection as a handwritten public ensures.
    let start = COORDINATE.find("    impl View for Coord {").unwrap();
    let end = start + COORDINATE[start..].find("    impl Coord {").unwrap();
    let unavailable = format!("{}{}", &COORDINATE[..start], &COORDINATE[end..]);
    let run = crates.build_source(
        "coordinate_provider",
        &unavailable,
        &[],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(!run.status.success());
    assert!(!crates.vir("coordinate_provider").exists());
    let message = String::from_utf8_lossy(&run.stderr);
    assert!(message.contains("constructor for an opaque datatype"), "{}", message);
    assert!(!message.contains("panicked"));
}

#[test]
fn local_private_constructor_can_keep_its_local_representation_contract() {
    let crates = InferCrates::new();
    let source = r#"use vstd::prelude::*;
verus! {
pub mod coordinate {
    pub struct Coord {round:u16,index:usize}
    #[verifier::infer] fn new(round:u16,index:usize)->Coord {Coord{round,index}}
    pub fn read_round(round:u16,index:usize)->(result:u16) ensures result==round,
    {let c=new(round,index);c.round}
}

}
"#;
    let run = crates.build_source(
        "local_coordinate",
        source,
        &[],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert!(json_inference(&run)["functions"][0]["required_checks_passed"] == true);
}

#[test]
fn nested_private_components_fail_cleanly_and_flat_views_remain_checked() {
    let crates = InferCrates::new();
    let source = r#"use vstd::prelude::*;
verus! {
pub mod data {
    use super::*;
    pub struct Payload {values:Vec<u8>,stamp:u16}
    impl View for Payload {type V=(Seq<u8>,u16);
        closed spec fn view(&self)->Self::V{(self.values@,self.stamp)}}
}
pub mod packet {
    use super::*;
    pub struct Coord {round:u16,index:usize}
    impl View for Coord {type V=(u16,usize);
        closed spec fn view(&self)->Self::V{(self.round,self.index)}}
    impl Coord {
        #[verifier::infer] pub fn new(round:u16,index:usize)->Self{Self{round,index}}
        #[verifier::infer] pub fn index(&self)->usize{self.index}
    }
    pub struct Packet {coord:Coord,payload:data::Payload}
    impl View for Packet {type V=(Coord,data::Payload);
        closed spec fn view(&self)->Self::V{(self.coord,self.payload)}}
    impl Packet {
        #[verifier::infer] pub fn new(payload:data::Payload,round:u16)->Self{
            Self{coord:Coord::new(round,0),payload}}
        #[verifier::infer] pub fn index(&self)->usize{self.coord.index()}
    }

}
pub fn build(payload:data::Payload,round:u16)->(result:packet::Packet)
    ensures result@.0@==(round,0usize),result@.1==payload,
{packet::Packet::new(payload,round)}
pub fn index(p:&packet::Packet)->(i:usize) ensures i==p@.0@.1, {p.index()}
}
"#;
    let flat = source
        .replace("type V=(Coord,data::Payload)", "type V=(u16,usize,data::Payload)")
        .replace("(self.coord,self.payload)", "(self.coord.round,self.coord.index,self.payload)")
        .replace("i==p@.0@.1", "i==p@.1")
        .replace(
            "result@.0@==(round,0usize),result@.1==payload",
            "result@.0==round,result@.1==0usize,result@.2==payload",
        );
    let run = crates.build_source(
        "private_component",
        &flat,
        &[],
        &["--infer-goals", "--no-cheating", "--output-json"],
        true,
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert!(
        json_inference(&run)["functions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|f| f["required_checks_passed"] == true)
    );
    let run = crates.build_source(
        "private_component",
        source,
        &[],
        &["--infer-goals", "--no-cheating", "--output-json"],
        true,
    );
    assert!(!run.status.success());
    assert!(!crates.vir("private_component").exists());
    let message = String::from_utf8_lossy(&run.stderr);
    assert!(
        message.contains("reconstructed private record requires scalar View components"),
        "{}",
        message
    );
    assert!(!message.contains("panicked"));
}

#[test]
fn private_heap_constructor_view_keeps_independent_structural_exactness() {
    let crates = InferCrates::new();
    let source = r#"use vstd::prelude::*;
verus! {
pub mod store {
    use super::*;
    pub struct Bag {items:Vec<u8>,stamp:u16}
    impl View for Bag {type V=(Seq<u8>,u16);
        closed spec fn view(&self)->Self::V{(self.items@,self.stamp)}}
    impl Bag {#[verifier::infer] pub fn new(items:Vec<u8>,stamp:u16)->Self{Self{items,stamp}}}
}
pub fn construct(items:Vec<u8>,stamp:u16)->(result:store::Bag)
    ensures result@==(items@,stamp),
{store::Bag::new(items,stamp)}
}
"#;
    let run =
        crates.build_source("private_heap", source, &[], &["--no-cheating", "--output-json"], true);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert!(json_inference(&run)["functions"][0]["required_checks_passed"] == true);
    let lossy = source.replace("(self.items@,self.stamp)", "(self.items@,0u16)");
    let run =
        crates.build_source("private_heap", &lossy, &[], &["--no-cheating", "--output-json"], true);
    assert!(!run.status.success());
    assert!(!crates.vir("private_heap").exists());
    let f = &json_inference(&run)["functions"][0];
    assert!(f["proofs"]["exactness"]["status"] != "proved");
    assert!(f["required_checks_passed"] == false);
}
