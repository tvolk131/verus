#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const REDB: &str = include_str!("../fixtures/infer/applications/redb_allocator.rs");
const QUORUM: &str = include_str!("../fixtures/infer/applications/fedimint_quorum.rs");

fn check(name: &str, source: &str, options: &[&str], error: Option<&str>) {
    let result = verify_one_file(name, source.to_string(), options);
    match error {
        None => assert!(result.is_ok(), "{name}: {result:?}"),
        Some(expected) => {
            let errors = result.expect_err("registered semantic/safety control must fail");
            assert_eq!(errors.errors.len(), 1, "{name}: {errors:?}");
            assert!(errors.errors[0].code.is_none(), "{name}: Rust/frontend error");
            assert!(errors.errors[0].message.contains(expected), "{name}: {errors:?}");
        }
    }
}

#[test]
fn redb_allocator_original_interval_and_conservation_contracts() {
    for goals in [false, true] {
        let mut options = vec!["vstd", "--no-cheating", "--cfg", "full_semantics"];
        if goals {
            options.push("--infer-goals");
        }
        check("redb_allocator", REDB, &options, None);
    }
}

#[test]
fn redb_allocator_semantic_and_safety_controls() {
    for (cfg, diagnostic) in [
        ("v2_negative_effect", "assertion failed"),
        ("v2_negative_merge", "assertion failed"),
        ("v2_negative_domain", "precondition not satisfied"),
        ("v2_negative_alloc_accept", "assertion failed"),
        ("v2_negative_alloc_effect", "assertion failed"),
        ("v2_negative_overlap", "assertion failed"),
    ] {
        check(
            cfg,
            REDB,
            &["vstd", "--no-cheating", "--infer-goals", "--cfg", "full_semantics", "--cfg", cfg],
            Some(diagnostic),
        );
    }
}

#[test]
fn fedimint_quorum_policy_and_honesty() {
    check("fedimint_quorum", QUORUM, &["vstd", "--no-cheating", "--infer-goals"], None);
}

#[test]
fn fedimint_false_intersection_and_honesty_controls() {
    for (old, new) in [
        (
            "ensures a.intersect(b).len() > fault(n),",
            "ensures a.intersect(b).len() > fault(n) + 1,",
        ),
        (
            "a.len() >= quorum(n), bad.len() <= fault(n),",
            "a.len() >= quorum(n), bad.len() <= quorum(n),",
        ),
    ] {
        assert_eq!(QUORUM.matches(old).count(), 1);
        check(
            "fedimint_false_policy",
            &QUORUM.replace(old, new),
            &["vstd", "--no-cheating", "--infer-goals"],
            Some("postcondition not satisfied"),
        );
    }
}
