#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::{InferCrates, assert_exact_report, json_inference, verify_exact};

const PROVIDER: &str = r#"
use vstd::prelude::*;
verus! {
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExternalIpv4(std::net::Ipv4Addr);
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExternalIpv6(std::net::Ipv6Addr);
pub struct Packet {
    pub from: std::net::Ipv4Addr,
    pub to: std::net::Ipv6Addr,
    pub ttl: u8,
}
impl Packet {
    #[verifier::infer]
    pub fn budget(&self, hops: u8) -> u8 { self.ttl.saturating_sub(hops) }
}
fn policy(packet: &Packet, hops: u8) -> (result: u8)
    ensures result as int == if packet.ttl >= hops { packet.ttl as int - hops } else { 0 },
{ packet.budget(hops) }
}
"#;

#[test]
fn explicit_foreign_opaque_fields_preserve_scalar_queries() {
    verify_exact("foreign_opaque_queries", PROVIDER.to_owned(), 1);
}

#[test]
fn foreign_opaque_imports_and_cold_warm_checks() {
    let crates = InferCrates::new();
    for _ in 0..2 {
        let result = crates.build_source(
            "packets",
            PROVIDER,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        assert_exact_report(&json_inference(&result), 1);
        let consumer = crates.build_source(
            "consumer",
            r#"
use vstd::prelude::*;
use packets::Packet;
verus! {
fn use_query(p: &Packet) {
    let result = p.budget(p.ttl);
    assert(result == 0);
}
}
"#,
            &["packets"],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(consumer.status.success(), "{}", String::from_utf8_lossy(&consumer.stderr));
    }
}

#[test]
fn opaque_fields_do_not_gain_equality_laws_or_publish_false_goals() {
    let crates = InferCrates::new();
    let seed =
        crates.build_source("packets", PROVIDER, &[], &["--no-cheating", "--output-json"], true);
    assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
    let source = format!(
        r#"{PROVIDER}
verus! {{ fn false_identity(a: &Packet, b: &Packet) {{ assert(a.from == b.from); }} }}
"#
    );
    let failed =
        crates.build_source("packets", &source, &[], &["--no-cheating", "--output-json"], true);
    assert!(!failed.status.success());
    assert_exact_report(&json_inference(&failed), 1);
    assert!(!crates.vir("packets").exists());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("assertion failed"));
}

#[test]
fn boundary_requires_declaration_and_does_not_grant_methods_or_mutation() {
    let crates = InferCrates::new();
    for source in [
        PROVIDER.replace(
            "pub struct ExternalIpv4(std::net::Ipv4Addr);",
            "pub struct ExternalIpv4(std::net::SocketAddrV4);",
        ),
        PROVIDER.replace("self.ttl.saturating_sub(hops)", "self.from.octets()[0]"),
        PROVIDER
            .replace(
                "pub fn budget(&self, hops: u8) -> u8 { self.ttl.saturating_sub(hops) }",
                "pub fn budget(&mut self, hops: u8) { self.ttl = hops; }",
            )
            .split("fn policy(")
            .next()
            .unwrap()
            .to_owned()
            + "}",
    ] {
        let seed = crates.build_source(
            "boundary",
            PROVIDER,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        let failed = crates.build_source(
            "boundary",
            &source,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(!failed.status.success());
        assert!(!crates.vir("boundary").exists());
        let stderr = String::from_utf8_lossy(&failed.stderr);
        assert!(!stderr.contains("panicked at"), "{}", stderr);
        assert!(
            stderr.contains("not supported")
                || stderr.contains("Verus Infer unsupported")
                || stderr.contains("duplicate"),
            "{}",
            stderr
        );
    }
}
