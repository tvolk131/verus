#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const REDB: &str = include_str!("../fixtures/infer/applications/redb_allocator.rs");
const QUORUM: &str = include_str!("../fixtures/infer/applications/fedimint_quorum.rs");

fn check(name: &str, source: &str, options: &[&str], error: Option<(&str, &str, usize)>) {
    // These frozen inputs import their own prelude. Preserve their source bytes
    // so diagnostic coordinates refer to the registered function below.
    let mut options = options.to_vec();
    options.push("no-auto-import-verus_builtin");
    let result = verify_one_file(name, source.to_string(), &options);
    match error {
        None => assert!(result.is_ok(), "{}: {:?}", name, result),
        Some((expected, function, count)) => {
            let errors = result.expect_err("registered semantic/safety control must fail");
            assert_eq!(errors.errors.len(), count, "{}: {:?}", name, errors);
            let start =
                source.find(&format!("fn {function}(")).expect("registered failing function");
            let end = start + source[start..].find("\n}").expect("function end") + 2;
            let first_line = source[..start].bytes().filter(|b| *b == b'\n').count() + 1;
            let last_line = source[..end].bytes().filter(|b| *b == b'\n').count() + 1;
            for diagnostic in &errors.errors {
                assert!(diagnostic.code.is_none(), "{}: Rust/frontend error", name);
                assert!(diagnostic.message.contains(expected), "{}: {:?}", name, errors);
                assert!(
                    diagnostic.spans.iter().any(|span| span.is_primary
                        && first_line <= span.line_start
                        && span.line_start <= last_line),
                    "{}: failure outside {}",
                    name,
                    function
                );
            }
        }
    }
}

#[test]
fn redb_allocator_original_interval_and_conservation_contracts() {
    for goals in [false, true] {
        let mut options = vec!["vstd", "--no-cheating", "--cfg full_semantics"];
        if goals {
            options.push("--infer-goals");
        }
        check("redb_allocator", REDB, &options, None);
    }
}

#[test]
fn redb_allocator_semantic_and_safety_controls() {
    for (cfg, function, diagnostic) in [
        ("v2_negative_effect", "wrong_free_effect", "assertion failed"),
        ("v2_negative_merge", "wrong_free_merge", "assertion failed"),
        ("v2_negative_domain", "wrong_free_domain", "precondition not satisfied"),
        ("v2_negative_alloc_accept", "wrong_alloc_accept", "assertion failed"),
        ("v2_negative_alloc_effect", "wrong_alloc_effect", "assertion failed"),
        ("v2_negative_overlap", "wrong_overlap", "assertion failed"),
    ] {
        check(
            cfg,
            REDB,
            &[
                "vstd",
                "--no-cheating",
                "--infer-goals",
                "--cfg full_semantics",
                &format!("--cfg {cfg}"),
            ],
            Some((diagnostic, function, 1)),
        );
    }
}

#[test]
fn fedimint_quorum_policy_and_honesty() {
    check("fedimint_quorum", QUORUM, &["vstd", "--no-cheating", "--infer-goals"], None);
}

#[test]
fn fedimint_false_intersection_and_honesty_controls() {
    for (old, new, function, count) in [
        (
            "ensures a.intersect(b).len() > fault(n),",
            "ensures a.intersect(b).len() > fault(n) + 1,",
            "quorum_intersection",
            1,
        ),
        (
            "a.len() >= quorum(n), bad.len() <= fault(n),",
            "a.len() >= quorum(n), bad.len() <= quorum(n),",
            "quorum_has_honest",
            2,
        ),
    ] {
        assert_eq!(QUORUM.matches(old).count(), 1);
        check(
            "fedimint_false_policy",
            &QUORUM.replace(old, new),
            &["vstd", "--no-cheating", "--infer-goals"],
            Some(("postcondition not satisfied", function, count)),
        );
    }
}
