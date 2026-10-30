use super::*;

// The direct compiler test supplies the same proposed model as infer_derive;
// the Cargo fixture separately exercises the actual derive macro expansion.
const ASSIGN: &str = r#"use vstd::prelude::*;
#[verus_verify] #[derive(Copy,Clone)] pub struct Quantity(pub u8);
verus! {
 impl Quantity { pub open spec fn __verus_infer_derived_add_assign(&self)->bool {true} }
 impl vstd::std_specs::ops::AddAssignSpecImpl for Quantity {
  open spec fn obeys_add_assign_spec()->bool {true}
  open spec fn add_assign_req(&self,rhs:Self)->bool {
   self.0 as int + rhs.0 as int <= u8::MAX
  }
  open spec fn add_assign_spec(&self,rhs:Self)->&Self {&Quantity((self.0+rhs.0) as u8)}
 }
}
#[automatically_derived]
impl core::ops::AddAssign for Quantity {
 fn add_assign(&mut self,rhs:Self) {self.0.add_assign(rhs.0);}
}
"#;

#[test]
fn derived_assignment_composes_nested_state_and_reproves_across_crates() {
    let crates = InferCrates::new();
    let consumer = r#"use vstd::prelude::*;use assign_provider::Quantity;
#[verus_verify] pub struct State {pub count:Quantity,pub epoch:u64}
#[verus_verify] impl State {
 #[verus_infer] pub fn bump(&mut self,delta:Quantity) {
  self.count += delta;
  self.count += Quantity(1);
 }
}
"#;
    let client = r#"use vstd::prelude::*;use assign_provider::Quantity;use assign_consumer::State;
verus! { pub fn policy(s:&mut State,delta:Quantity)
 requires (old(s).count.0 as int + delta.0 as int) < u8::MAX,
 ensures final(s).count.0 as int == old(s).count.0 as int + delta.0 as int + 1,
  final(s).epoch == old(s).epoch,
 {s.bump(delta);}
}
"#;
    for _ in 0..2 {
        for (name, source, deps) in [
            ("assign_provider", ASSIGN, &[][..]),
            ("assign_consumer", consumer, &["assign_provider"][..]),
            ("assign_client", client, &["assign_provider", "assign_consumer"][..]),
        ] {
            let run =
                crates.build_source(name, source, deps, &["--no-cheating", "--output-json"], true);
            assert!(run.status.success(), "{name}: {}", String::from_utf8_lossy(&run.stderr));
            if name != "assign_client" {
                let report = json_inference(&run);
                let fs = report["functions"].as_array().unwrap();
                assert_eq!(fs.len(), 1);
                assert_eq!(fs[0]["required_checks_passed"], true);
                for check in ["soundness", "maximality", "exactness"] {
                    assert_eq!(fs[0]["proofs"][check]["status"], "proved");
                }
            }
        }
    }
    let run = crates.build_source(
        "assign_client",
        &client.replace("+ 1,", "+ 2,"),
        &["assign_provider", "assign_consumer"],
        &["--no-cheating"],
        true,
    );
    assert!(!run.status.success());
    assert!(!crates.vir("assign_client").exists());
}

#[test]
fn derived_assignment_rejects_wrong_bodies_domains_and_missing_evidence() {
    let crates = InferCrates::new();
    for body in ["self.0 = rhs.0;", "self.0.sub_assign(rhs.0);"] {
        let changed = ASSIGN.replace("self.0.add_assign(rhs.0);", body);
        let run = crates.build_source(
            "assign_provider",
            &changed,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(!run.status.success(), "wrong assignment unexpectedly verified");
        assert!(!crates.vir("assign_provider").exists());
    }
    for bound in ["< u8::MAX", "<= u8::MAX / 2", "<= 511"] {
        let source = ASSIGN.replace("<= u8::MAX", bound);
        let run = crates.build_source(
            "assign_provider",
            &source,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(!run.status.success(), "wrong inherited domain unexpectedly verified");
        assert!(!crates.vir("assign_provider").exists());
    }
    let missing = ASSIGN.replace("#[automatically_derived]\n", "");
    let run = crates.build_source("assign_provider", &missing, &[], &["--no-cheating"], true);
    assert!(!run.status.success());
    assert!(
        String::from_utf8_lossy(&run.stderr)
            .contains("derived trait request requires one checked implementation")
    );
    assert!(!crates.vir("assign_provider").exists());
    for flags in [vec!["--no-verify"], vec!["--verify-root", "--verify-function", "add_assign"]] {
        let run = crates.build_source("assign_provider", ASSIGN, &[], &flags, true);
        assert!(!run.status.success());
        assert!(!crates.vir("assign_provider").exists());
        assert!(String::from_utf8_lossy(&run.stderr).contains("requires full-crate verification"));
    }
}

#[test]
fn signed_derived_assignment_keeps_negative_inputs_and_both_overflow_edges() {
    let crates = InferCrates::new();
    let source = ASSIGN.replace("u8", "i16").replace(
        "self.0 as int + rhs.0 as int <= i16::MAX",
        "i16::MIN <= self.0 as int + rhs.0 as int && self.0 as int + rhs.0 as int <= i16::MAX",
    );
    let client = r#"use vstd::prelude::*;use signed_assign::Quantity;
verus! {pub fn signed_policy(q:&mut Quantity,r:Quantity)
 requires i16::MIN <= old(q).0 as int + r.0 as int,
  old(q).0 as int + r.0 as int <= i16::MAX,
 ensures final(q).0 as int == old(q).0 as int + r.0 as int,
 {*q += r;}}
"#;
    let run = crates.build_source(
        "signed_assign",
        &source,
        &[],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_eq!(json_inference(&run)["functions"][0]["required_checks_passed"], true);
    let run =
        crates.build_source("signed_client", client, &["signed_assign"], &["--no-cheating"], true);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
}
