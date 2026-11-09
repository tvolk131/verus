#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const SOURCE: &str = r#"
use vstd::prelude::*;
verus! {
    #[verifier::external]
    #[derive(PartialEq)]
    pub struct Foreign([u8; 32]);
    #[verifier::external_type_specification]
    #[verifier::external_body]
    pub struct ExForeign(Foreign);
    pub struct Request { pub hash: Foreign, pub n: u32, pub other: Option<u64> }
    impl Request {
        #[verifier::infer]
        pub fn query(&self, height: Option<u32>) -> u32 {
            height.map_or(0, |h| self.n.saturating_sub(h))
        }
    }
    fn check(r: &Request) -> (out: u32) ensures out == 0,
    { r.query(Some(r.n)) }
}
"#;

#[test]
fn explicitly_opaque_unused_field_retains_original_readonly_body() {
    let result = verify_one_file("unused", SOURCE.to_owned(), &["vstd", "--no-cheating"]);
    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn opaque_transport_rejects_by_value_receivers_and_their_implicit_drop() {
    let source = SOURCE
        .replace("pub fn query(&self, height: Option<u32>) -> u32", "pub fn query(self) -> [u8; 4]")
        .replace("height.map_or(0, |h| self.n.saturating_sub(h))", "[0u8; 4]")
        .replace("    fn check(r: &Request) -> (out: u32) ensures out == 0,\n    { r.query(Some(r.n)) }\n", "");
    let error = verify_one_file("by_value", source, &["vstd", "--no-cheating"]).unwrap_err();
    assert!(
        error.errors.iter().any(|d| d.message.contains("exact borrowed readonly record")),
        "{:?}",
        error
    );
}

#[test]
fn opaque_access_aliases_traits_mutation_and_invariants_fail_closed() {
    let mut invariant = SOURCE.replace(
        "pub struct Request { pub hash: Foreign, pub n: u32, pub other: Option<u64> }",
        "mod guarded { use super::*; pub struct Request { hash: Foreign, n: u32, other: Option<u64> }",
    ).replace("impl Request {", "impl Request { #[verifier::type_invariant] pub closed spec fn valid(&self) -> bool { self.n < 100 }");
    invariant.insert(invariant.rfind('}').unwrap(), '}');
    for (label, source) in [
        ("read", SOURCE.replace("height.map_or", "&self.hash; height.map_or")),
        (
            "alias",
            SOURCE
                .replace("height.map_or", "let alias = self; let _v = &alias.hash; height.map_or"),
        ),
        (
            "mutate",
            SOURCE
                .replace("pub fn query(&self", "pub fn query(&mut self")
                .replace("fn check(r: &Request)", "fn check(r: &mut Request)"),
        ),
        (
            "trait",
            SOURCE.replace("height.map_or", "let _v = self.hash == self.hash; height.map_or"),
        ),
        ("invariant", invariant),
        (
            "missing_proxy",
            SOURCE.replace("#[verifier::external_type_specification]", "#[verifier::external]"),
        ),
        ("transparent", SOURCE.replace("#[verifier::external_body]", "")),
    ] {
        let error = verify_one_file(label, source, &["vstd", "--no-cheating"]).unwrap_err();
        let diagnostic = match label {
            "read" => "opaque external record fields must remain unused",
            "alias" => "record local aliases",
            "mutate" => "Verus Infer unsupported",
            "trait" => "trait inference requires",
            "invariant" => "record state requires visible transparent record fields",
            "missing_proxy" => "cannot use type",
            "transparent" => "private fields not supported",
            _ => unreachable!(),
        };
        assert!(
            error.errors.iter().any(|d| d.message.contains(diagnostic)),
            "{}: {:?}",
            label,
            error
        );
    }
}

#[test]
fn scalar_domain_checks_are_not_weakened_by_an_unread_opaque_field() {
    let source = SOURCE
        .replace("self.n.saturating_sub(h)", "self.n - h")
        .replace("r.query(Some(r.n))", "r.query(Some(1))");
    let error = verify_one_file("coverage", source, &["vstd", "--no-cheating"]).unwrap_err();
    assert!(
        error.errors.iter().any(|d| d.message.contains("precondition not satisfied")),
        "{:?}",
        error
    );
    assert!(
        error.notes.iter().any(|d| d.message.contains("safe-domain maximality: PROVED")),
        "{:?}",
        error
    );
}
