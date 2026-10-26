#![feature(rustc_private)]

#[macro_use]
mod common;
use common::*;

test_verify_one_file! {
    #[test] arithmetic_and_composition code! {
        #[verus_infer]
        fn compute(x: u64) -> u64 { (x + 10) * 2 }

        #[verus_infer]
        fn debit(balance: u64, amount: u64) -> u64 { balance - amount }

        #[verus_infer]
        fn abs_diff(a: u64, b: u64) -> u64 {
            if a >= b { a - b } else { b - a }
        }

        #[verus_infer]
        fn fee(amount: u64) -> u64 { amount / 100 }

        verus! {
            fn caller(x: u64) -> (r: u64)
                requires x <= 9_223_372_036_854_775_797,
                ensures r as int == (x as int + 10) * 2 - 10,
            {
                let n = compute(x);
                debit(n, 10)
            }

            fn difference(a: u64, b: u64) -> (r: u64)
                ensures r == if a >= b { a - b } else { b - a },
            { abs_diff(a, b) }

            fn small_fee(amount: u64) -> (r: u64)
                requires amount < 100,
                ensures r == 0,
            { fee(amount) }
        }
    } => Ok(diags) => {
        for label in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
            assert_eq!(diags.notes.iter().filter(|d| d.message.contains(label)).count(), 4, "{label}: {:?}", diags.notes);
        }
    }
}

test_verify_one_file! {
    #[test] branch_local_and_short_circuit code! {
        #[verus_infer]
        fn branch(x: u8, b: bool) -> u8 {
            let offset = if b { 3u8 } else { 1u8 };
            let answer = x + offset;
            answer
        }

        #[verus_infer]
        fn guarded_division(x: u8, y: u8) -> bool { y != 0 && x / y > 2 }

        #[verus_infer]
        fn guarded_or(x: u8, y: u8) -> bool { y == 0 || x / y < 10 }

        #[verus_infer]
        fn quotient(x: u8, y: u8) -> u8 { x / y }

        #[verus_infer]
        fn remainder(x: u8, y: u8) -> u8 { x % y }

        verus! {
            #[verifier::infer]
            fn nested(a: u8, b: u8) -> u8 {
                if a == 0 { 0 } else if a >= b { a - b } else { b - a }
            }

            fn boundary() {
                let r = branch(252, true); assert(r == 255);
                let r = branch(254, false); assert(r == 255);
                let r = guarded_division(255, 0); assert(!r);
                let r = guarded_or(255, 0); assert(r);
                let r = quotient(255, 1); assert(r == 255);
                let r = remainder(255, 2); assert(r == 1);
                let r = nested(0, 255); assert(r == 0);
            }
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] unsafe_caller_is_rejected code! {
        #[verus_infer]
        fn debit(balance: u8, amount: u8) -> u8 { balance - amount }
        verus! { fn caller() { let _ = debit(0, 1); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] overflow_caller_is_rejected code! {
        #[verus_infer]
        fn compute(x: u64) -> u64 { (x + 10) * 2 }
        verus! { fn caller() { let _ = compute(9_223_372_036_854_775_798); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] division_by_zero_caller_is_rejected code! {
        #[verus_infer]
        fn quotient(x: u8, y: u8) -> u8 { x / y }
        verus! { fn caller() { let _ = quotient(1, 0); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] unsupported_loop code! {
        #[verus_infer]
        fn f(x: u8) -> u8 { loop {} }
    } => Err(e) => assert_vir_error_msg(e, "Verus Infer unsupported: loops")
}

test_verify_one_file! {
    #[test] early_returns_guards_and_nested_operands code! {
        #[verus_infer]
        fn quotient(x: u8, y: u8) -> u8 {
            if y == 0 { return 0; }
            x / y
        }
        #[verus_infer]
        fn increment(x: u8) -> u8 { return x + 1; }
        #[verus_infer]
        fn branch(x: u8) -> u8 {
            if x == 0 { return 3; } else { return x - 1; }
        }
        #[verus_infer]
        fn operand(x: u8) -> u8 {
            let y = if x == 0 { return 4; } else { x - 1 };
            y + 1
        }
        #[verus_infer]
        fn short_circuit(x: u8) -> u8 {
            let b = x == 0 && { return 7; };
            if b { 1 } else { 2 }
        }
        verus! {
            fn caller(x: u8, y: u8)
                requires x < 255,
            {
                let r = quotient(x, y);
                assert(r == if y == 0 { 0 } else { x / y });
                let r = increment(x); assert(r as int == x as int + 1);
                let r = branch(x); assert(r == if x == 0 { 3 } else { x - 1 });
                let r = operand(x); assert(r == if x == 0 { 4 } else { x });
                let r = short_circuit(x); assert(r == if x == 0 { 7u8 } else { 2u8 });
            }
        }
    } => Ok(diags) => {
        for label in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
            assert_eq!(diags.notes.iter().filter(|d| d.message.contains(label)).count(), 5, "{}", label);
        }
    }
}

test_verify_one_file! {
    #[test] early_returns_checked_matches_and_borrowed_arrays code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn debit(balance: u8, amount: u8) -> Result<u8, u8> {
            let remaining = match balance.checked_sub(amount) {
                None => return Err(1),
                Some(value) => value,
            };
            if remaining == 0 { return Err(2); }
            Ok(remaining)
        }
        #[verus_infer]
        fn read(values: &[u8; 2], index: usize) -> Option<u8> {
            if index >= values.len() { return None; }
            Some(values[index])
        }
        verus! {
            fn caller(balance: u8, amount: u8, values: &[u8; 2], index: usize) {
                let r = debit(balance, amount);
                assert(r == if amount > balance { Err::<u8, u8>(1u8) }
                    else if amount == balance { Err::<u8, u8>(2u8) }
                    else { Ok::<u8, u8>((balance - amount) as u8) });
                let r = read(values, index);
                assert(r == if index < 2 { Some(values[index as int]) } else { None });
            }
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] early_returns_preserve_evaluation_order code! {
        #[allow(unreachable_code)]
        #[verus_infer]
        fn before(x: u8) -> u8 { (x + 1) / { return 5; 0u8 } }
        #[allow(unreachable_code)]
        #[verus_infer]
        fn after(x: u8) -> u8 {
            return x;
            x + 1
        }
        #[allow(unreachable_code)]
        #[verus_infer]
        fn array_before(x: u8) -> u8 {
            let values: [u8; 2] = [x + 1, { return 9; }];
            values[0]
        }
        #[allow(unreachable_code)]
        #[verus_infer]
        fn array_after(x: u8) -> u8 {
            let values: [u8; 2] = [{ return 9; }, x + 1];
            values[0]
        }
        verus! {
            fn caller(x: u8) {
                let r = after(x); assert(r == x);
                let r = array_after(x); assert(r == 9);
                if x < 255 {
                    let r = before(x); assert(r == 5);
                    let r = array_before(x); assert(r == 9);
                }
            }
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] early_returns_boolean_conditions_and_unit_payloads code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn condition(x: u8) -> u8 {
            if (if x == 0 { return 9; } else { x < 4 }) { x + 1 } else { x - 1 }
        }
        #[verus_infer]
        fn boolean(x: u8) -> bool { if x == 0 { return true; } x < 5 }
        #[verus_infer]
        fn short_or(x: u8) -> u8 {
            let b = x == 0 || { return 7; };
            if b { 2 } else { 1 }
        }
        #[verus_infer]
        fn unit_payload(b: bool) -> Result<(), u8> {
            if b { return Err(1); }
            Ok(())
        }
        #[verus_infer]
        fn unit_match(x: u8) -> u8 {
            match x { 0 => return 9, _ => () };
            x
        }
        #[verus_infer]
        fn unit_if(b: bool, x: u8) -> u8 {
            if b { return 1; } else { () };
            x
        }
        #[verus_infer]
        fn payload_return(b: bool) -> Result<(), u8> {
            Ok(if b { return Err(1); } else { () })
        }
        verus! {
            fn caller(x: u8, b: bool) {
                let r = condition(x);
                assert(r == if x == 0 { 9 } else if x < 4 { x + 1 } else { x - 1 });
                let r = boolean(x); assert(r == (x < 5));
                let r = short_or(x); assert(r == if x == 0 { 2u8 } else { 7u8 });
                let r = unit_payload(b); assert(r == if b { Err(1u8) } else { Ok(()) });
                let r = unit_match(x); assert(r == if x == 0 { 9u8 } else { x });
                let r = unit_if(b, x); assert(r == if b { 1u8 } else { x });
                let r = payload_return(b); assert(r == if b { Err(1u8) } else { Ok(()) });
            }
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] early_return_does_not_hide_prior_overflow code! {
        #[allow(unreachable_code)]
        #[verus_infer]
        fn before(x: u8) -> u8 { (x + 1) / { return 5; 0u8 } }
        verus! { fn caller() { let _ = before(255); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] early_returns_remain_exact_mode_only code! {
        #[verus_infer(precision = "sound")]
        fn f(x: u8) -> u8 { if x == 0 { return 1; } x }
    } => Err(e) => assert_vir_error_msg(e, "early returns currently require exact mode")
}

test_verify_one_file! {
    #[test] early_returns_do_not_bypass_source_eligibility verus_code! {
        #[allow(unreachable_code)]
        #[verifier::infer]
        fn f(x: u8) -> u8 { return x; assume(false); x }
    } => Err(e) => assert_vir_error_msg(e, "proof statements and assumptions")
}

test_verify_one_file! {
    #[test] inferred_call code! {
        #[verus_infer]
        fn f(x: u8) -> u8 { g(x) }
        #[verus_infer]
        fn g(x: u8) -> u8 { x }
    } => Ok(())
}

test_verify_one_file! {
    #[test] unsupported_assumption verus_code! {
        #[verifier::infer]
        fn f(x: u8) -> u8 { assume(false); x + 1 }
    } => Err(e) => assert_vir_error_msg(e, "proof statements and assumptions")
}

test_verify_one_file! {
    #[test] unsupported_mutation code! {
        #[verus_infer]
        fn f(x: u8) -> u8 { let mut y = x; y += 1; y }
    } => Err(e) => assert_vir_error_msg(e, "mutable or nonscalar locals")
}

test_verify_one_file! {
    #[test] unsupported_isize code! {
        #[verus_infer]
        fn f(x: isize) -> isize { x + 1 }
    } => Err(e) => assert_vir_error_msg(e, "parameters and return values must be")
}

test_verify_one_file! {
    #[test] no_silent_overwrite verus_code! {
        #[verifier::infer]
        fn f(x: u8) -> (r: u8) requires x < 10, ensures r == x, { x }
    } => Err(e) => assert_vir_error_msg(e, "combining inference with handwritten contracts")
}

test_verify_one_file_with_options! {
    #[test] no_unverified_inference ["--no-verify"] => code! {
        #[verus_infer]
        fn f(x: u8) -> u8 { x + 1 }
    } => Err(e) => assert_vir_error_msg(e, "Verus Infer requires full-crate verification")
}

test_verify_one_file! {
    #[test] invalid_options code! {
        #[verus_infer(precision = "bogus")]
        fn f(x: u8) -> u8 { x }
    } => Err(e) => assert_vir_error_msg(e, "precision must be")
}

test_verify_one_file! {
    #[test] solver_limit_prevents_publication verus_code! {
        #[verifier::infer]
        #[verifier::rlimit(0.000001)]
        fn multiply(x: u64, y: u64) -> u64 { x * y }

        fn caller() { assert(false); }
    } => Err(e) => {
        assert!(e.errors.iter().any(|d| d.message.contains("Resource limit")));
        assert!(!e.errors.iter().any(|d| d.message.contains("assertion failed")));
        assert!(e.notes.iter().any(|d| d.message.contains("INCOMPLETE (resource limit)")));
        assert!(e.notes.iter().any(|d| d.message.contains("candidates were not published")));
        assert!(!e.notes.iter().any(|d| d.message.contains("contracts available")));
    }
}

#[test]
fn formula_budget_is_bounded() {
    let mut source = String::from("#[verus_infer] fn f(x: u64) -> u64 { let a0 = x; ");
    for i in 1..=16 {
        source.push_str(&format!("let a{i} = a{} + a{}; ", i - 1, i - 1));
    }
    source.push_str("a16 }");
    let errors = verify_one_file("formula_budget_is_bounded", source, &[]).unwrap_err();
    assert_vir_error_msg(errors, "Verus Infer incomplete: expanded formula budget exceeded");
}

test_verify_one_file! {
    #[test] widths_casts_and_booleans code! {
        #[verus_infer]
        fn small(x: u16) -> u16 { x + 1 }
        #[verus_infer]
        fn medium(x: u32) -> u32 { x * 2 }
        #[verus_infer]
        fn large(x: u128) -> u128 { x - 1 }
        #[verus_infer]
        fn truncate(x: u16) -> u8 { x as u8 }
        #[verus_infer]
        fn booleans(a: bool, b: bool) -> bool { !(a ^ b) | (a & b) }
        verus! {
            fn caller() {
                let a = small(65534); assert(a == 65535);
                let b = medium(2147483647); assert(b == 4294967294);
                let c = large(340282366920938463463374607431768211455);
                assert(c == 340282366920938463463374607431768211454);
                assert(256u16 as u8 == 0) by(bit_vector);
                let d = truncate(256); assert(d == 0);
                let e = booleans(true, false); assert(!e);
            }
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] cross_module_composition verus_code! {
        mod arithmetic {
            #[verifier::infer]
            pub fn double(x: u64) -> u64 { x * 2 }
            #[verifier::infer]
            pub fn increment(x: u64) -> u64 { x + 1 }
        }
        mod consumer {
            use super::*;
            #[verifier::infer]
            fn composed(x: u64) -> u64 {
                super::arithmetic::increment(super::arithmetic::double(x))
            }
            fn caller(x: u64) -> (r: u64)
                requires x <= 9_223_372_036_854_775_807,
                ensures r as int == (x as int) * 2 + 1,
            {
                composed(x)
            }
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] nested_diamond_forward_and_branch_calls code! {
        #[verus_infer]
        fn top(x: u8, b: bool) -> u8 { if b { left(x) } else { right(x) } }
        #[verus_infer]
        fn right(x: u8) -> u8 { increment(increment(x)) }
        #[verus_infer]
        fn left(x: u8) -> u8 { increment(x) * 2 }
        #[verus_infer]
        fn increment(x: u8) -> u8 { x + 1 }
        verus! {
            fn caller() {
                let a = top(126, true); assert(a == 254);
                let b = top(253, false); assert(b == 255);
            }
        }
    } => Ok(diags) => {
        for label in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
            assert_eq!(diags.notes.iter().filter(|d| d.message.contains(label)).count(), 4);
        }
    }
}

test_verify_one_file! {
    #[test] forwarding_composed_summary_preserves_intermediate_safety code! {
        #[verus_infer]
        fn compute(x: u64) -> u64 { (x + 10) * 2 }
        #[verus_infer]
        fn debit(balance: u64, amount: u64) -> u64 { balance - amount }
        #[verus_infer]
        fn middle(x: u64) -> u64 { debit(compute(x), 10) }
        #[verus_infer]
        fn top(x: u64) -> u64 { middle(x) }
        verus! {
            fn caller(x: u64) -> (r: u64)
                requires x <= 9_223_372_036_854_775_797,
                ensures r as int == (x as int + 10) * 2 - 10,
            { top(x) }
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] guarded_call_domain code! {
        #[verus_infer]
        fn quotient(x: u8, y: u8) -> u8 { x / y }
        #[verus_infer]
        fn guarded(x: u8, y: u8) -> bool { y == 0 || quotient(x, y) < 10 }
        verus! { fn caller() { let r = guarded(255, 0); assert(r); } }
    } => Ok(())
}

test_verify_one_file! {
    #[test] transitive_unsafe_caller code! {
        #[verus_infer]
        fn increment(x: u8) -> u8 { x + 1 }
        #[verus_infer]
        fn twice(x: u8) -> u8 { increment(increment(x)) }
        verus! { fn caller() { let _ = twice(254); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] recursive_inference code! {
        #[verus_infer]
        fn a(x: u8) -> u8 { b(x) }
        #[verus_infer]
        fn b(x: u8) -> u8 { a(x) }
    } => Err(e) => assert_vir_error_msg(e, "recursive inferred call graph")
}

test_verify_one_file! {
    #[test] missing_inferred_contract verus_code! {
        #[verifier::infer]
        fn a(x: u8) -> u8 { b(x) }
        fn b(x: u8) -> u8 { x }
    } => Err(e) => assert_vir_error_msg(e, "same-crate inferred callee")
}

test_verify_one_file! {
    #[test] failed_callee_blocks_inferred_caller verus_code! {
        #[verifier::infer]
        fn top(x: u64, y: u64) -> u64 { multiply(x, y) }
        #[verifier::infer]
        #[verifier::rlimit(0.000001)]
        fn multiply(x: u64, y: u64) -> u64 { x * y }
        fn caller() { assert(false); }
    } => Err(e) => {
        assert!(e.errors.iter().any(|d| d.message.contains("Resource limit")));
        assert!(!e.errors.iter().any(|d| d.message.contains("assertion failed")));
        assert!(!e.notes.iter().any(|d| d.message.contains("top: soundness: PROVED")));
        assert!(e.notes.iter().any(|d| d.message.contains("dependent verification was not started")));
    }
}

#[test]
fn persistent_cache_candidates_are_reproved_before_dependent_publication() {
    let directory = tempfile::tempdir().unwrap();
    let cache = directory.path().join("cache");
    let source = directory.path().join("test.rs");
    std::fs::write(
        &source,
        r#"
        use verus_builtin::*;
        use verus_builtin_macros::*;
        verus! {
            #[verifier::infer]
            fn top(x: u8) -> u8 { increment(x) }
            #[verifier::infer]
            fn increment(x: u8) -> u8 { x + 1 }
            fn caller() { let r = top(1); assert(r == 2); }
        }
    "#,
    )
    .unwrap();
    let executable = std::env::current_exe().unwrap();
    let profile = executable.parent().unwrap().parent().unwrap();
    let extension = std::env::consts::DLL_EXTENSION;
    let prefix = std::env::consts::DLL_PREFIX;
    let run = || {
        std::process::Command::new(
            profile.join(format!("rust_verify{}", std::env::consts::EXE_SUFFIX)),
        )
        .arg("--internal-test-mode")
        .arg("--crate-type=lib")
        .arg("--edition=2021")
        .arg("--cfg=verus_no_vstd")
        .arg("--extern")
        .arg(format!("verus_builtin={}", profile.join("libverus_builtin.rlib").display()))
        .arg("--extern")
        .arg(format!(
            "verus_builtin_macros={}",
            profile.join(format!("{prefix}verus_builtin_macros.{extension}")).display()
        ))
        .arg("-L")
        .arg(format!("dependency={}", profile.display()))
        .arg("--emit=metadata")
        .arg("-o")
        .arg(directory.path().join("out.rmeta"))
        .arg(&source)
        .env("VERUS_INFER_CACHE", &cache)
        .env(
            "VERUS_Z3_PATH",
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join(if cfg!(windows) { "z3.exe" } else { "z3" }),
        )
        .output()
        .unwrap()
    };
    let first = run();
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    let second = run();
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    assert_eq!(std::fs::read_dir(&cache).unwrap().count(), 2);
    // A syntactically valid but too-restrictive cached domain is sound vacuously.
    // Only the independent maximality proof rejects it; no dependent may run.
    for entry in std::fs::read_dir(&cache).unwrap() {
        let path = entry.unwrap().path();
        let mut candidate: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        candidate["pre"] = serde_json::json!({"Bool": false});
        std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
    }
    let poisoned = run();
    let diagnostics = String::from_utf8_lossy(&poisoned.stderr);
    assert!(!poisoned.status.success(), "poisoned cache unexpectedly accepted");
    assert!(diagnostics.contains("safe-domain maximality"), "{}", diagnostics);
    assert!(diagnostics.contains("candidates were not published"), "{}", diagnostics);
    assert!(!diagnostics.contains("top: soundness: PROVED"), "{}", diagnostics);
    assert!(!diagnostics.contains("contracts available"), "{}", diagnostics);
}

// Exercise the actual compiler's binary export/import boundary, rather than
// constructing imported ASTs in the test process.
struct InferCrates {
    directory: tempfile::TempDir,
    profile: std::path::PathBuf,
}
impl InferCrates {
    fn new() -> Self {
        let executable = std::env::current_exe().unwrap();
        Self {
            directory: tempfile::tempdir().unwrap(),
            profile: executable.parent().unwrap().parent().unwrap().into(),
        }
    }
    fn vir(&self, name: &str) -> std::path::PathBuf {
        self.directory.path().join(format!("{name}.vir"))
    }
    fn build(
        &self,
        name: &str,
        body: &str,
        dependencies: &[&str],
        options: &[&str],
    ) -> std::process::Output {
        let source = self.directory.path().join(format!("{name}.rs"));
        std::fs::write(
            &source,
            format!("use verus_builtin::*; use verus_builtin_macros::*; verus! {{ {body} }}"),
        )
        .unwrap();
        let mut command = std::process::Command::new(
            self.profile.join(format!("rust_verify{}", std::env::consts::EXE_SUFFIX)),
        );
        command
            .args([
                "--internal-test-mode",
                "--crate-type=lib",
                "--edition=2021",
                "--compile",
                "--crate-name",
                name,
            ])
            .arg("--export")
            .arg(self.vir(name))
            .arg("-o")
            .arg(self.directory.path().join(format!("lib{name}.rlib")))
            .arg("--extern")
            .arg(format!("verus_builtin={}", self.profile.join("libverus_builtin.rlib").display()))
            .arg("--extern")
            .arg(format!(
                "verus_builtin_macros={}",
                self.profile
                    .join(format!(
                        "{}verus_builtin_macros.{}",
                        std::env::consts::DLL_PREFIX,
                        std::env::consts::DLL_EXTENSION
                    ))
                    .display()
            ))
            .arg("-L")
            .arg(format!("dependency={}", self.profile.display()))
            .arg("-L")
            .arg(format!("dependency={}", self.directory.path().display()))
            .env(
                "VERUS_Z3_PATH",
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .join(if cfg!(windows) { "z3.exe" } else { "z3" }),
            );
        if body.contains("vstd::") {
            let override_vir = self.directory.path().join("vstd.vir");
            let vstd_vir =
                if override_vir.exists() { override_vir } else { self.profile.join("vstd.vir") };
            command
                .arg("--extern")
                .arg(format!("vstd={}", self.profile.join("libvstd.rlib").display()))
                .arg("--import")
                .arg(format!("vstd={}", vstd_vir.display()));
        } else {
            command.arg("--cfg=verus_no_vstd");
        }
        for dependency in dependencies {
            command
                .arg("--extern")
                .arg(format!(
                    "{dependency}={}",
                    self.directory.path().join(format!("lib{dependency}.rlib")).display()
                ))
                .arg("--import")
                .arg(format!("{dependency}={}", self.vir(dependency).display()));
        }
        // Repeated builds exercise candidate reuse; proof results are never cached.
        command.env("VERUS_INFER_CACHE", self.directory.path().join("candidate-cache"));
        command.args(options).arg(source).output().unwrap()
    }
    fn succeeds(&self, name: &str, body: &str, dependencies: &[&str]) -> std::process::Output {
        let run = self.build(name, body, dependencies, &[]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        run
    }
    fn edit_metadata(&self, name: &str, edit: impl FnOnce(&mut serde_json::Value)) {
        let path = self.vir(name);
        let mut bytes = std::fs::read(&path).unwrap();
        let start =
            bytes.windows(b"{\"schema\":".len()).rposition(|s| s == b"{\"schema\":").unwrap();
        let mut metadata: serde_json::Value =
            serde_json::from_slice(&bytes[start..bytes.len() - 8]).unwrap();
        edit(&mut metadata);
        bytes.truncate(start);
        bytes.extend(serde_json::to_vec(&metadata).unwrap());
        bytes.extend(b"VRINFER1");
        std::fs::write(path, bytes).unwrap();
    }
}

const INFER_BASE: &str = r#"
    #[verifier::infer]
    fn secret_helper(x: u8) -> u8 { x + 1 }
    #[verifier::infer]
    pub fn increment(x: u8) -> u8 { secret_helper(x) }
"#;
const INFER_MIDDLE: &str = r#"
    #[verifier::infer]
    pub fn twice(x: u8) -> u8 { infer_base::increment(infer_base::increment(x)) }
"#;
const INFER_APP: &str = r#"
    #[verifier::infer]
    pub fn top(x: u8) -> u8 { infer_middle::twice(x) }
    pub fn application_property() { let r = top(253); assert(r == 255); }
"#;

#[test]
fn cross_crate_inferred_chain_and_stale_dependencies() {
    let crates = InferCrates::new();
    crates.succeeds("infer_base", INFER_BASE, &[]);
    let exported = std::fs::read(crates.vir("infer_base")).unwrap();
    assert!(!exported.windows(b"secret_helper".len()).any(|s| s == b"secret_helper"));
    crates.succeeds("infer_middle", INFER_MIDDLE, &["infer_base"]);
    let run = crates.succeeds("infer_app", INFER_APP, &["infer_base", "infer_middle"]);
    assert!(String::from_utf8_lossy(&run.stderr).contains("output exactness: PROVED"));
    // Keep Rust metadata compatible while replacing just the Verus artifact;
    // otherwise rustc rejects the stale rlib before the inference boundary runs.
    let rlib_path = crates.directory.path().join("libinfer_base.rlib");
    let original_rlib = std::fs::read(&rlib_path).unwrap();
    crates.succeeds("infer_base", &INFER_BASE.replace("x + 1", "x + 2"), &[]);
    std::fs::write(&rlib_path, original_rlib).unwrap();
    let stale = crates.build("infer_app", INFER_APP, &["infer_base", "infer_middle"], &[]);
    assert!(!stale.status.success());
    assert!(
        String::from_utf8_lossy(&stale.stderr)
            .contains("stale or missing inferred dependency fingerprint"),
        "{}",
        String::from_utf8_lossy(&stale.stderr)
    );
    assert!(!crates.vir("infer_app").exists());
    crates.succeeds("infer_base", &INFER_BASE.replace("x + 1", "x + 2"), &[]);
    crates.succeeds("infer_middle", INFER_MIDDLE, &["infer_base"]);
    crates.succeeds(
        "infer_app",
        &INFER_APP.replace("top(253)", "top(251)"),
        &["infer_base", "infer_middle"],
    );
}

#[test]
fn imported_inference_requires_complete_compatible_metadata() {
    let crates = InferCrates::new();
    for (field, value, expected) in [
        ("schema", serde_json::json!(999), "incompatible inference schema"),
        ("policy", serde_json::json!("wrapping"), "incompatible inference schema"),
        ("producer", serde_json::json!("incompatible compiler"), "incompatible inference schema"),
        ("soundness", serde_json::json!(false), "incomplete inference proof guarantees"),
        ("maximality", serde_json::json!(false), "incomplete inference proof guarantees"),
        ("exactness", serde_json::json!(false), "incomplete inference proof guarantees"),
        ("summaries", serde_json::json!("corrupted"), "inferred contract fingerprint mismatch"),
    ] {
        crates.succeeds("infer_base", INFER_BASE, &[]);
        crates.edit_metadata("infer_base", |metadata| metadata[field] = value);
        let run = crates.build("infer_middle", INFER_MIDDLE, &["infer_base"], &[]);
        assert!(!run.status.success(), "accepted incompatible metadata: {}", field);
        assert!(
            String::from_utf8_lossy(&run.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert!(!crates.vir("infer_middle").exists());
    }
    crates.succeeds("infer_base", INFER_BASE, &[]);
    let path = crates.vir("infer_base");
    let mut bytes = std::fs::read(&path).unwrap();
    let start = bytes.windows(b"{\"schema\":".len()).rposition(|s| s == b"{\"schema\":").unwrap();
    bytes.truncate(start);
    std::fs::write(path, bytes).unwrap();
    let run = crates.build("infer_middle", INFER_MIDDLE, &["infer_base"], &[]);
    assert!(!run.status.success());
    assert!(
        String::from_utf8_lossy(&run.stderr)
            .contains("missing or invalid inference proof metadata"),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn failed_or_partial_verification_never_exports_inference() {
    let crates = InferCrates::new();
    for options in [vec!["--no-verify"], vec!["--verify-root", "--verify-function", "increment"]] {
        crates.succeeds("infer_base", INFER_BASE, &[]);
        let run = crates.build("infer_base", INFER_BASE, &[], &options);
        assert!(!run.status.success());
        assert!(
            !crates.vir("infer_base").exists(),
            "options {:?}: {}",
            options,
            String::from_utf8_lossy(&run.stderr)
        );
    }
    crates.succeeds("infer_base", INFER_BASE, &[]);
    let run = crates.build(
        "infer_base",
        &format!("{} pub fn failed() {{ assert(false); }}", INFER_BASE),
        &[],
        &[],
    );
    assert!(!run.status.success());
    assert!(!crates.vir("infer_base").exists());
    crates.succeeds("infer_base", INFER_BASE, &[]);
    let invalid = crates.build("infer_base", "this is not valid Rust;", &[], &[]);
    assert!(!invalid.status.success());
    assert!(!crates.vir("infer_base").exists());
    crates.succeeds("infer_base", INFER_BASE, &[]);
    let limited = r#"
        #[verifier::infer]
        #[verifier::rlimit(0.000001)]
        pub fn multiply(x: u64, y: u64) -> u64 { x * y }
    "#;
    let run = crates.build("infer_base", limited, &[], &[]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("Resource limit"));
    assert!(!crates.vir("infer_base").exists());
}

#[test]
fn ordinary_library_exports_remain_compatible() {
    let crates = InferCrates::new();
    crates.succeeds(
        "ordinary_base",
        r#"
        pub fn increment(x: u8) -> (r: u8)
            requires x < 255,
            ensures r as int == x as int + 1,
        { x + 1 }
    "#,
        &[],
    );
    assert!(!std::fs::read(crates.vir("ordinary_base")).unwrap().ends_with(b"VRINFER1"));
    crates.succeeds(
        "ordinary_user",
        r#"
        pub fn caller() { let r = ordinary_base::increment(254); assert(r == 255); }
    "#,
        &["ordinary_base"],
    );
}

test_verify_one_file! {
    #[test] sound_interval_is_useful_without_exactness code! {
        #[verus_infer(precision = "sound")]
        fn fee(amount: u8) -> u8 { amount / 100 }
        verus! {
            fn range_property(amount: u8) -> (r: u8)
                ensures r <= 2,
            { fee(amount) }
        }
    } => Ok(diags) => {
        assert!(diags.notes.iter().any(|d| d.message.contains("soundness: PROVED")));
        assert!(diags.notes.iter().any(|d| d.message.contains("safe-domain maximality: PROVED")));
        assert!(diags.notes.iter().any(|d| d.message.contains("output exactness: NOT PROVED")));
        assert!(diags.notes.iter().any(|d| d.message.contains("VERIFIED SOUND")));
        assert!(!diags.notes.iter().any(|d| d.message.contains("output exactness: PROVED")));
    }
}

test_verify_one_file! {
    #[test] sound_summary_does_not_grant_an_exact_result code! {
        #[verus_infer(precision = "sound")]
        fn fee(amount: u8) -> u8 { amount / 100 }
        verus! { fn caller() { let r = fee(255); assert(r == 2); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] exact_mode_rejects_sound_callee code! {
        #[verus_infer(precision = "sound")]
        fn fee(amount: u8) -> u8 { amount / 100 }
        #[verus_infer]
        fn wrapper(amount: u8) -> u8 { fee(amount) }
    } => Err(e) => assert_vir_error_msg(e, "exact inference requires exact-mode callees")
}

test_verify_one_file! {
    #[test] sound_mode_composes_without_assuming_callee_maximality code! {
        #[verus_infer(precision = "sound")]
        fn fee(amount: u8) -> u8 { amount / 100 }
        #[verus_infer(precision = "sound")]
        fn wrapper(amount: u8) -> u8 { fee(amount) + 1 }
        verus! { fn caller() { let r = wrapper(1); assert(r >= 1 && r <= 3); } }
    } => Ok(diags) => {
        assert!(diags.notes.iter().any(|d| d.message.contains("NOT ESTABLISHED (sound-mode dependency)")));
        assert!(!diags.notes.iter().any(|d| d.message.contains("wrapper: safe-domain maximality: PROVED")));
    }
}

test_verify_one_file! {
    #[test] explicit_exact_mode_is_supported code! {
        #[verus_infer(precision = "exact")]
        fn increment(x: u8) -> u8 { x + 1 }
        verus! { fn caller() { let r = increment(254); assert(r == 255); } }
    } => Ok(())
}

#[test]
fn interval_fallback_outlives_exact_formula_expansion_budget() {
    let mut source =
        String::from("#[verus_infer(precision = \"sound\")] fn f(x: u64) -> u64 { let a0 = x; ");
    for i in 1..=16 {
        source.push_str(&format!("let a{i} = a{} + a{}; ", i - 1, i - 1));
    }
    source.push_str("a16 } verus! { fn caller(x: u64) -> (r: u64) requires x <= u64::MAX / 65536, ensures r <= u64::MAX, { f(x) } }");
    let diags =
        verify_one_file("interval_fallback_outlives_exact_formula_expansion_budget", source, &[])
            .unwrap();
    assert!(diags.notes.iter().any(|d| d.message.contains("soundness: PROVED")));
    assert!(diags.notes.iter().any(|d| d.message.contains("safe-domain maximality: PROVED")));
    assert!(diags.notes.iter().any(|d| d.message.contains("output exactness: NOT PROVED")));
}

test_verify_one_file! {
    #[test] sound_mode_still_requires_successful_soundness verus_code! {
        #[verifier::infer(sound)]
        #[verifier::rlimit(0.000001)]
        fn multiply(x: u64, y: u64) -> u64 { x * y }
        fn caller() { assert(false); }
    } => Err(e) => {
        assert!(e.errors.iter().any(|d| d.message.contains("Resource limit")));
        assert!(!e.errors.iter().any(|d| d.message.contains("assertion failed")));
        assert!(!e.notes.iter().any(|d| d.message.contains("contracts available")));
    }
}

test_verify_one_file! {
    #[test] optional_precision_timeout_preserves_soundness verus_code! {
        #[verifier::infer(sound)]
        #[verifier::infer_precision_rlimit(0.000001)]
        fn select_value(b: bool) -> u8 { if b { 1 } else { 2 } }
        fn caller() { let r = select_value(true); assert(r >= 1 && r <= 2); }
    } => Ok(diags) => {
        assert!(diags.notes.iter().any(|d| d.message.contains("soundness: PROVED")));
        assert!(diags.notes.iter().any(|d| d.message.contains("output exactness: INCOMPLETE")));
        assert!(diags.notes.iter().any(|d| d.message.contains("VERIFIED SOUND")));
    }
}

const SOUND_BASE: &str = r#"
    #[verifier::infer(sound)]
    pub fn fee(x: u8) -> u8 { x / 100 }
"#;
const SOUND_MIDDLE: &str = r#"
    #[verifier::infer(sound)]
    pub fn wrapped(x: u8) -> u8 { sound_base::fee(x) + 1 }
"#;

#[test]
fn cross_crate_sound_precision_is_preserved_and_exact_callers_are_rejected() {
    let crates = InferCrates::new();
    let run = crates.succeeds("sound_base", SOUND_BASE, &[]);
    assert!(String::from_utf8_lossy(&run.stderr).contains("output exactness: NOT PROVED"));
    crates.edit_metadata("sound_base", |metadata| {
        assert_eq!(metadata["soundness"], true);
        assert_eq!(metadata["maximality"], true);
        assert_eq!(metadata["exactness"], false);
        let proofs = metadata["proofs"].as_array().unwrap();
        assert_eq!(proofs.len(), 1);
        assert_eq!(proofs[0][1]["soundness"], true);
        assert_eq!(proofs[0][1]["exactness"], false);
    });
    crates.succeeds("sound_middle", SOUND_MIDDLE, &["sound_base"]);
    crates.edit_metadata("sound_middle", |metadata| {
        assert_eq!(metadata["soundness"], true);
        assert_eq!(metadata["maximality"], false);
        assert_eq!(metadata["exactness"], false);
    });
    crates.succeeds(
        "sound_app",
        r#"
        pub fn range_property(x:u8) { let r = sound_middle::wrapped(x); assert(r >= 1 && r <= 3); }
    "#,
        &["sound_base", "sound_middle"],
    );
    let exact = crates.build(
        "exact_app",
        r#"
        #[verifier::infer]
        pub fn exact_wrapper(x: u8) -> u8 { sound_middle::wrapped(x) }
    "#,
        &["sound_base", "sound_middle"],
        &[],
    );
    assert!(!exact.status.success());
    assert!(
        String::from_utf8_lossy(&exact.stderr)
            .contains("exact inference requires exact-mode callees"),
        "{}",
        String::from_utf8_lossy(&exact.stderr)
    );
    assert!(!crates.vir("exact_app").exists());
    // Removing the mandatory soundness proof is fatal even for sound mode.
    crates.edit_metadata("sound_middle", |metadata| {
        metadata["proofs"][0][1]["soundness"] = serde_json::json!(false)
    });
    let unsound =
        crates.build("sound_app", "pub fn caller() {}", &["sound_base", "sound_middle"], &[]);
    assert!(!unsound.status.success());
    assert!(
        String::from_utf8_lossy(&unsound.stderr).contains("incomplete inference proof guarantees")
    );
}

#[test]
fn cached_sound_candidates_are_reproved_and_cannot_publish_wrong_bounds() {
    let crates = InferCrates::new();
    crates.succeeds("sound_base", SOUND_BASE, &[]);
    let second = crates.succeeds("sound_base", SOUND_BASE, &[]);
    assert!(String::from_utf8_lossy(&second.stderr).contains("soundness: PROVED"));
    assert!(String::from_utf8_lossy(&second.stderr).contains("output exactness: NOT PROVED"));
    let cache = crates.directory.path().join("candidate-cache");
    let entries: Vec<_> = std::fs::read_dir(&cache).unwrap().map(|e| e.unwrap().path()).collect();
    assert_eq!(entries.len(), 1);
    let path = &entries[0];
    let mut candidate: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let zero = candidate["post"]["Logical"][1]["Binary"][1].clone();
    assert!(!zero.is_null());
    candidate["post"] = serde_json::json!({"Binary": [{"Eq": "Spec"}, {"Param": 1}, zero]});
    std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
    let wrong = crates.build("sound_base", SOUND_BASE, &[], &[]);
    let diagnostics = String::from_utf8_lossy(&wrong.stderr);
    assert!(!wrong.status.success(), "invalid cached bounds were accepted");
    assert!(diagnostics.contains("soundness: NOT PROVED"), "{}", diagnostics);
    assert!(!diagnostics.contains("VERIFIED SOUND"), "{}", diagnostics);
    assert!(!crates.vir("sound_base").exists());
}

test_verify_one_file! {
    #[test] native_precision_budget_is_independent_of_soundness code! {
        #[verus_infer(precision = "sound", precision_rlimit = 1e-40)]
        fn selected_value(b: bool) -> u8 { if b { 1 } else { 2 } }
        verus! { fn caller() { let r = selected_value(true); assert(r >= 1 && r <= 2); } }
    } => Ok(diags) => {
        assert!(diags.notes.iter().any(|d| d.message.contains("soundness: PROVED")));
        assert!(diags.notes.iter().any(|d| d.message.contains("output exactness: INCOMPLETE")));
    }
}

test_verify_one_file! {
    #[test] sound_boolean_and_widths code! {
        #[verus_infer(precision = "sound")]
        fn even(x: u8) -> bool { x % 2 == 0 }
        #[verus_infer(precision = "sound")]
        fn truncate(x: u16) -> u8 { x as u8 }
        #[verus_infer(precision = "sound")]
        fn large_fee(x: u128) -> u128 { x / 100 }
        verus! {
            fn caller(x: u8) {
                let b = even(x); assert(b || !b);
                let r = truncate(65535); assert(r <= 255);
                let r = large_fee(1000);
                assert(r <= 3402823669209384634633746074317682114u128);
            }
        }
    } => Ok(diags) => {
        assert_eq!(diags.notes.iter().filter(|d| d.message.contains("VERIFIED SOUND")).count(), 3);
    }
}

test_verify_one_file! {
    #[test] conflicting_inference_annotations_are_rejected verus_code! {
        #[verifier::infer(sound)]
        #[verifier::infer]
        fn f(x: u8) -> u8 { x }
    } => Err(e) => assert_vir_error_msg(e, "duplicate inference modes")
}

test_verify_one_file! {
    #[test] exact_option_result_constructors_and_matches code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn maybe_increment(x: u8, yes: bool) -> Option<u8> {
            if yes { Some(x + 1) } else { None }
        }
        #[verus_infer]
        fn divide(x: u8, y: u8) -> Result<u8, ()> {
            if y == 0 { Err(()) } else { Ok(x / y) }
        }
        #[verus_infer]
        fn consume(x: Option<u8>) -> Result<u8, bool> {
            match x { Some(v) => Ok(v), None => Err(true) }
        }
        verus! {
            fn caller() {
                let r = maybe_increment(254, true); assert(r == Some(255u8));
                let r = maybe_increment(255, false); assert(r == None);
                let r = divide(7, 2); assert(r == Ok::<u8, ()>(3));
                let r = divide(7, 0); assert(r == Err::<u8, ()>(()));
                let r = consume(Some(9)); assert(r == Ok::<u8, bool>(9));
                let r = consume(None); assert(r == Err::<u8, bool>(true));
            }
        }
    } => Ok(diags) => {
        for label in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
            assert_eq!(diags.notes.iter().filter(|d| d.message.contains(label)).count(), 3);
        }
    }
}

test_verify_one_file! {
    #[test] exact_checked_debit_and_result_wrapper code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn debit(balance: u64, amount: u64) -> Option<u64> { balance.checked_sub(amount) }
        #[verus_infer]
        fn validated(balance: u64, amount: u64) -> Result<u64, u8> {
            match debit(balance, amount) { Some(v) => Ok(v), None => Err(1) }
        }
        verus! {
            fn caller() {
                let r = debit(0, 1); assert(r == None);
                let r = debit(10, 3); assert(r == Some(7u64));
                let r = validated(0, 1); assert(r == Err::<u64, u8>(1));
                let r = validated(10, 3); assert(r == Ok::<u64, u8>(7));
            }
        }
    } => Ok(diags) => {
        for label in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
            assert_eq!(diags.notes.iter().filter(|d| d.message.contains(label)).count(), 2);
        }
    }
}

test_verify_one_file! {
    #[test] checked_arithmetic_all_cases_and_widths code! {
        use vstd::prelude::*;
        #[verus_infer] fn add(x: u8, y: u8) -> Option<u8> { x.checked_add(y) }
        #[verus_infer] fn sub(x: u16, y: u16) -> Option<u16> { x.checked_sub(y) }
        #[verus_infer] fn mul(x: u32, y: u32) -> Option<u32> { x.checked_mul(y) }
        #[verus_infer] fn div(x: u64, y: u64) -> Option<u64> { x.checked_div(y) }
        #[verus_infer] fn rem(x: u128, y: u128) -> Option<u128> { x.checked_rem(y) }
        verus! {
            fn caller() {
                let r = add(255, 1); assert(r == None);
                let r = add(254, 1); assert(r == Some(255u8));
                let r = sub(0, 1); assert(r == None);
                let r = sub(65535, 1); assert(r == Some(65534u16));
                let r = mul(4294967295, 2); assert(r == None);
                let r = mul(4294967295, 1); assert(r == Some(4294967295u32));
                let r = div(7, 0); assert(r == None);
                let r = div(7, 2); assert(r == Some(3u64));
                let r = rem(7, 0); assert(r == None);
                let r = rem(7, 2); assert(r == Some(1u128));
            }
        }
    } => Ok(diags) => {
        assert_eq!(diags.notes.iter().filter(|d| d.message.contains("output exactness: PROVED")).count(), 5);
        assert_eq!(diags.notes.iter().filter(|d| d.message.contains("trusted standard models from vstd")).count(), 5);
    }
}

test_verify_one_file! {
    #[test] match_safety_and_literal_priority code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn classify(value: Option<u8>) -> Result<u8, ()> {
            match value { Some(0) => Err(()), Some(x) => Ok(10 / x), None => Ok(0) }
        }
        #[verus_infer]
        fn guarded_overflow(value: Option<u8>) -> u8 {
            match value { None => 0, Some(x) => x + 1 }
        }
        verus! {
            fn caller() {
                let r = classify(Some(0)); assert(r == Err::<u8, ()>(()));
                let r = classify(Some(2)); assert(r == Ok::<u8, ()>(5));
                let r = classify(None); assert(r == Ok::<u8, ()>(0));
                let r = guarded_overflow(None); assert(r == 0);
                let r = guarded_overflow(Some(254)); assert(r == 255);
            }
        }
    } => Ok(diags) => {
        assert_eq!(diags.notes.iter().filter(|d| d.message.contains("output exactness: PROVED")).count(), 2);
    }
}

test_verify_one_file! {
    #[test] unsafe_enum_payload_evaluation_is_rejected code! {
        use vstd::prelude::*;
        #[verus_infer] fn unsafe_error(x: u8) -> Result<(), u8> { Err(x + 1) }
        verus! { fn caller() { let _ = unsafe_error(255); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] exact_enum_caller_cannot_use_approximate_payload code! {
        use vstd::prelude::*;
        #[verus_infer(precision = "sound")] fn fee(x: u8) -> u8 { x / 100 }
        #[verus_infer] fn result(x: u8) -> Option<u8> { Some(fee(x)) }
    } => Err(e) => assert_vir_error_msg(e, "exact inference requires exact-mode callees")
}

test_verify_one_file! {
    #[test] enum_sound_mode_is_explicitly_unsupported code! {
        use vstd::prelude::*;
        #[verus_infer(precision = "sound")] fn maybe(x: u8) -> Option<u8> { Some(x) }
    } => Err(e) => assert_vir_error_msg(e, "Option/Result inference currently requires exact mode")
}

test_verify_one_file! {
    #[test] match_guards_are_explicitly_unsupported code! {
        use vstd::prelude::*;
        #[verus_infer] fn f(x: Option<u8>) -> u8 { match x { Some(v) if v > 0 => v, _ => 0 } }
    } => Err(e) => assert_vir_error_msg(e, "match guards")
}

test_verify_one_file! {
    #[test] lookalike_checked_method_is_not_approved verus_code! {
        fn checked_sub(x: u8, y: u8) -> u8 { x - y }
        #[verifier::infer] fn f(x: u8, y: u8) -> u8 { checked_sub(x, y) }
    } => Err(e) => assert_vir_error_msg(e, "same-crate inferred callee")
}

const ENUM_BASE: &str = r#"
    use vstd::prelude::*;
    #[verifier::infer]
    pub fn debit(x: u8, y: u8) -> Option<u8> { x.checked_sub(y) }
"#;
const ENUM_MIDDLE: &str = r#"
    use vstd::prelude::*;
    #[verifier::infer]
    pub fn validated(x: u8, y: u8) -> Result<u8, ()> {
        match enum_base::debit(x, y) { Some(v) => Ok(v), None => Err(()) }
    }
"#;

#[test]
fn enum_cross_crate_summaries_caches_and_checked_model_provenance() {
    let crates = InferCrates::new();
    crates.succeeds("enum_base", ENUM_BASE, &[]);
    crates.succeeds("enum_base", ENUM_BASE, &[]); // cached enum constructors
    crates.edit_metadata("enum_base", |metadata| {
        assert_eq!(metadata["trusted_models"].as_array().unwrap().len(), 1)
    });
    crates.succeeds("enum_middle", ENUM_MIDDLE, &["enum_base"]);
    crates.succeeds("enum_middle", ENUM_MIDDLE, &["enum_base"]); // cached selectors/match conditions
    crates.succeeds(
        "enum_app",
        r#"
        use vstd::prelude::*;
        pub fn property() {
            let r = enum_middle::validated(0, 1); assert(r == Err::<u8, ()>(()));
            let r = enum_middle::validated(10, 3); assert(r == Ok::<u8, ()>(7));
        }
    "#,
        &["enum_base", "enum_middle"],
    );
    crates.edit_metadata("enum_base", |metadata| {
        metadata["trusted_models"][0][1] = serde_json::json!("stale")
    });
    let run = crates.build("enum_middle", ENUM_MIDDLE, &["enum_base"], &[]);
    assert!(!run.status.success());
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("stale or missing trusted-model fingerprint"),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(!crates.vir("enum_middle").exists());
}

#[test]
fn wrong_cached_enum_tag_or_payload_fails_independent_verification() {
    for wrong_tag in [true, false] {
        let crates = InferCrates::new();
        crates.succeeds("enum_base", ENUM_BASE, &[]);
        let cache = crates.directory.path().join("candidate-cache");
        let entries: Vec<_> =
            std::fs::read_dir(&cache).unwrap().map(|e| e.unwrap().path()).collect();
        assert_eq!(entries.len(), 1);
        let path = &entries[0];
        let mut candidate: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        if wrong_tag {
            candidate["post"]["Binary"][2] =
                serde_json::json!({"Ctor": [{"Option": {"Unsigned": 8}}, "None", null]});
        } else {
            let zero = serde_json::to_value(vir::ast::Constant::Int(0.into())).unwrap();
            candidate["post"]["Binary"][2]["If"][1]["Ctor"][2] = zero;
        }
        std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
        let run = crates.build("enum_base", ENUM_BASE, &[], &[]);
        assert!(!run.status.success(), "incorrect enum summary was accepted");
        assert!(
            String::from_utf8_lossy(&run.stderr).contains("soundness: NOT PROVED"),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert!(!crates.vir("enum_base").exists());
    }
}

test_verify_one_file! {
    #[test] sound_mode_rejects_enum_temporaries_without_panicking code! {
        use vstd::prelude::*;
        #[verus_infer(precision = "sound")]
        fn f(x: u8, y: u8) -> u8 { let value = x.checked_sub(y); 0 }
    } => Err(e) => assert_vir_error_msg(e, "enum expressions currently require exact inference")
}

test_verify_one_file! {
    #[test] user_enum_named_option_is_not_core_option verus_code! {
        enum Option { Some(u8), None }
        #[verifier::infer]
        fn f(x: u8) -> Option { Option::Some(x) }
    } => Err(e) => assert_vir_error_msg(e, "parameters and return values must be")
}

#[test]
fn malformed_inference_metadata_blocks_import_and_invalidates_old_exports() {
    let crates = InferCrates::new();
    crates.succeeds("infer_base", INFER_BASE, &[]);
    let path = crates.vir("infer_base");
    let original = std::fs::read(&path).unwrap();
    for corruption in 0..7 {
        std::fs::write(&path, &original).unwrap();
        crates.succeeds("infer_middle", INFER_MIDDLE, &["infer_base"]);
        match corruption {
            0 => {
                std::fs::write(&path, &original[..original.len() - 1]).unwrap();
            }
            1 => {
                let mut bytes = original[..original.len() - 8].to_vec();
                bytes.extend(b"trailing garbageVRINFER1");
                std::fs::write(&path, bytes).unwrap();
            }
            2 => {
                let mut bytes = original[..original.len() - 8].to_vec();
                bytes.extend(vec![b' '; 1024 * 1024 + 1]);
                bytes.extend(b"VRINFER1");
                std::fs::write(&path, bytes).unwrap();
            }
            3 => crates.edit_metadata("infer_base", |metadata| {
                let duplicate = metadata["proofs"][0].clone();
                metadata["proofs"].as_array_mut().unwrap().push(duplicate);
            }),
            4 => crates.edit_metadata("infer_base", |metadata| {
                metadata["proofs"] = serde_json::json!([]);
            }),
            5 => crates.edit_metadata("infer_base", |metadata| {
                for proof in metadata["proofs"].as_array_mut().unwrap() {
                    proof[1]["soundness"] = serde_json::json!(false);
                }
            }),
            6 => crates.edit_metadata("infer_base", |metadata| {
                metadata.as_object_mut().unwrap().remove("trusted_models");
            }),
            _ => unreachable!(),
        }
        let run = crates.build("infer_middle", INFER_MIDDLE, &["infer_base"], &[]);
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(!run.status.success(), "accepted corruption {}", corruption);
        assert!(
            stderr.contains("Verus Infer:") && stderr.contains("rebuild the library"),
            "{}",
            stderr
        );
        assert!(!stderr.contains("panicked"), "{}", stderr);
        assert!(
            !crates.vir("infer_middle").exists(),
            "old export survived corruption {}",
            corruption
        );
    }
}

#[test]
fn cached_nonmaximal_candidate_blocks_inferred_and_ordinary_dependents() {
    let crates = InferCrates::new();
    let source = r#"
        #[verifier::infer]
        pub fn identity(x: u8) -> u8 { x }
        #[verifier::infer]
        pub fn wrapper(x: u8) -> u8 { identity(x) }
        pub fn ordinary() { let r = identity(1); assert(r == 1); }
    "#;
    let options = ["--num-threads", "4"];
    let seed = crates.build("infer_base", source, &[], &options);
    assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
    let cache = crates.directory.path().join("candidate-cache");
    let entries: Vec<_> = std::fs::read_dir(&cache).unwrap().map(|e| e.unwrap().path()).collect();
    assert_eq!(entries.len(), 2);
    let leaf = entries
        .iter()
        .find(|path| {
            let candidate: serde_json::Value =
                serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            candidate["dependencies"].as_array().unwrap().is_empty()
        })
        .unwrap();
    let mut candidate: serde_json::Value =
        serde_json::from_slice(&std::fs::read(leaf).unwrap()).unwrap();
    // Vacuous soundness is insufficient to publish an exact interface.
    candidate["pre"] = serde_json::json!({"Bool": false});
    std::fs::write(leaf, serde_json::to_vec(&candidate).unwrap()).unwrap();
    let run = crates.build("infer_base", source, &[], &options);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(!run.status.success(), "nonmaximal exact candidate was published");
    assert!(stderr.contains("identity: soundness: PROVED"), "{}", stderr);
    assert!(stderr.contains("identity: safe-domain maximality: NOT PROVED"), "{}", stderr);
    assert!(stderr.contains("dependent verification was not started"), "{}", stderr);
    assert!(!stderr.contains("wrapper: soundness:"), "{}", stderr);
    assert!(!stderr.contains("precondition not satisfied"), "{}", stderr);
    assert!(!stderr.contains("contracts available"), "{}", stderr);
    assert!(!crates.vir("infer_base").exists());
}

test_verify_one_file! {
    #[test] exact_array_reads_literals_and_composition code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn at(values: [u8; 4], index: usize) -> u8 { values[index] }
        #[verus_infer]
        fn guarded(values: [u8; 4], index: usize) -> u8 {
            if index < 4 { at(values, index) } else { 0 }
        }
        #[verus_infer]
        fn pair_sum(values: [u8; 2]) -> u8 { values[0] + values[1] }
        #[verus_infer]
        fn literal(x: u8, first: bool) -> u8 {
            let values = [x + 1, x];
            let copy = values;
            if first { copy[0] } else { copy[1] }
        }
        verus! {
            fn caller() {
                let r = at([10, 20, 30, 40], 0); assert(r == 10);
                let r = at([10, 20, 30, 40], 3); assert(r == 40);
                let r = guarded([10, 20, 30, 40], 4); assert(r == 0);
                let r = guarded([10, 20, 30, 40], usize::MAX); assert(r == 0);
                let r = guarded([10, 20, 30, 40], 2); assert(r == 30);
                let r = pair_sum([127, 128]); assert(r == 255);
                let r = literal(254, true); assert(r == 255);
                let r = literal(254, false); assert(r == 254);
            }
        }
    } => Ok(diags) => {
        for label in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
            assert_eq!(diags.notes.iter().filter(|d| d.message.contains(label)).count(), 4);
        }
    }
}

test_verify_one_file! {
    #[test] array_bounds_are_guarded_by_branches_and_short_circuit code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn enabled(values: [bool; 2], index: usize, enabled: bool) -> bool {
            enabled && values[index]
        }
        #[verus_infer]
        fn selected(a: [u64; 2], b: [u64; 2], first: bool, i: usize, j: usize) -> u64 {
            let values = if first { a } else { b };
            values[if first { i } else { j }]
        }
        #[verus_infer]
        fn optional(values: [u16; 2], index: usize) -> Option<u16> {
            if index < 2 { Some(values[index]) } else { None }
        }
        verus! {
            fn caller() {
                let r = enabled([true, false], 999, false); assert(!r);
                let r = enabled([true, false], 0, true); assert(r);
                let r = selected([3, 4], [5, 6], true, 1, 999); assert(r == 4);
                let r = selected([3, 4], [5, 6], false, 999, 0); assert(r == 5);
                let r = optional([7, 8], 1); assert(r == Some(8u16));
                let r = optional([7, 8], 2); assert(r == None);
            }
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] out_of_bounds_array_caller_is_rejected code! {
        use vstd::prelude::*;
        #[verus_infer] fn at(values: [u8; 2], i: usize) -> u8 { values[i] }
        verus! { fn caller() { let _ = at([1, 2], 2); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] unselected_array_literal_elements_still_require_safe_evaluation code! {
        use vstd::prelude::*;
        #[verus_infer] fn first(x: u8) -> u8 { let values = [x, x + 1]; values[0] }
        verus! { fn caller() { let _ = first(255); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] array_sound_mode_is_explicitly_unsupported code! {
        use vstd::prelude::*;
        #[verus_infer(precision = "sound")]
        fn at(values: [u8; 2]) -> u8 { values[0] }
    } => Err(e) => assert_vir_error_msg(e, "arrays and usize indices")
}

test_verify_one_file! {
    #[test] oversized_arrays_are_explicitly_unsupported code! {
        use vstd::prelude::*;
        #[verus_infer] fn at(values: [u8; 65]) -> u8 { values[0] }
    } => Err(e) => assert_vir_error_msg(e, "arrays of length 0..=64")
}

test_verify_one_file! {
    #[test] array_mutation_is_not_in_the_immutable_subset code! {
        use vstd::prelude::*;
        #[verus_infer] fn changed(values: [u8; 2]) -> u8 {
            let mut copy = values; copy[0] = 3; copy[0]
        }
    } => Err(e) => assert_vir_error_msg(e, "mutable or nonscalar locals")
}

test_verify_one_file! {
    #[test] empty_and_maximum_arrays_and_wide_elements code! {
        use vstd::prelude::*;
        #[verus_infer] fn impossible(values: [u8; 0], index: usize) -> u8 { values[index] }
        #[verus_infer] fn last(values: [u128; 64]) -> u128 { values[63] }
        #[verus_infer] fn temporary(value: u32) -> u32 { [0u32, value][1] }
        verus! {
            fn caller() {
                let r = last([9u128; 64]); assert(r == 9);
                let r = temporary(u32::MAX); assert(r == u32::MAX);
            }
        }
    } => Ok(diags) => {
        assert_eq!(diags.notes.iter().filter(|d| d.message.contains("output exactness: PROVED")).count(), 3);
    }
}

test_verify_one_file! {
    #[test] array_return_values_are_explicitly_unsupported code! {
        use vstd::prelude::*;
        #[verus_infer] fn identity(values: [u8; 2]) -> [u8; 2] { values }
    } => Err(e) => assert_vir_error_msg(e, "array/usize returns are unsupported")
}

test_verify_one_file! {
    #[test] array_slice_parameters_are_explicitly_unsupported code! {
        use vstd::prelude::*;
        #[verus_infer] fn first(values: &[u8]) -> u8 { values[0] }
    } => Err(e) => assert_vir_error_msg(e, "parameters and return values must be")
}

test_verify_one_file! {
    #[test] nested_arrays_are_explicitly_unsupported code! {
        use vstd::prelude::*;
        #[verus_infer] fn first(values: [[u8; 2]; 2]) -> u8 { values[0][0] }
    } => Err(e) => assert_vir_error_msg(e, "parameters and return values must be")
}

test_verify_one_file! {
    #[test] array_indices_do_not_enable_general_usize_arithmetic code! {
        use vstd::prelude::*;
        #[verus_infer] fn shifted(values: [u8; 2], index: usize) -> u8 { values[index + 1] }
    } => Err(e) => assert_vir_error_msg(e, "Verus Infer unsupported")
}

const ARRAY_BASE: &str = r#"
    use vstd::prelude::*;
    #[verifier::infer]
    pub fn at(values: [u8; 2], index: usize) -> u8 { values[index] }
    #[verifier::infer]
    pub fn from_literal(x: u8, index: usize) -> u8 { let values = [x, x + 1]; values[index] }
"#;
const ARRAY_MIDDLE: &str = r#"
    use vstd::prelude::*;
    #[verifier::infer]
    pub fn twice(values: [u8; 2], index: usize) -> u8 {
        let amount = array_base::at(values, index); amount + amount
    }
"#;

#[test]
fn array_cross_crate_summaries_and_literal_caches_preserve_exactness() {
    let crates = InferCrates::new();
    crates.succeeds("array_base", ARRAY_BASE, &[]);
    crates.succeeds("array_base", ARRAY_BASE, &[]);
    crates.succeeds("array_middle", ARRAY_MIDDLE, &["array_base"]);
    crates.succeeds("array_middle", ARRAY_MIDDLE, &["array_base"]);
    crates.succeeds(
        "array_app",
        r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn top(values: [u8; 2], index: usize) -> u8 { array_middle::twice(values, index) }
        pub fn property() {
            let r = top([100, 120], 0); assert(r == 200);
            let r = top([100, 120], 1); assert(r == 240);
            let r = array_base::from_literal(254, 1); assert(r == 255);
        }
    "#,
        &["array_base", "array_middle"],
    );
}

#[test]
fn wrong_cached_array_index_is_rejected_by_independent_proofs() {
    let crates = InferCrates::new();
    let source = r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn first(values: [u8; 2]) -> u8 { values[0] }
    "#;
    crates.succeeds("array_base", source, &[]);
    let path = std::fs::read_dir(crates.directory.path().join("candidate-cache"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let mut candidate: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let one = serde_json::to_value(vir::ast::Constant::Int(1.into())).unwrap();
    candidate["post"]["Binary"][2]["Binary"][2] = one;
    std::fs::write(&path, serde_json::to_vec(&candidate).unwrap()).unwrap();
    let run = crates.build("array_base", source, &[], &[]);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(!run.status.success(), "wrong array index was accepted");
    assert!(stderr.contains("soundness: NOT PROVED"), "{}", stderr);
    assert!(!crates.vir("array_base").exists());
}

#[test]
fn precision_assumptions_cannot_hide_ordinary_overflow_or_bounds_failures() {
    for source in [
        r#"
            #[verifier::infer]
            pub fn increment(x: u8) -> u8 { x + 1 }
            fn caller() { let r = increment(1); assert(r == 2); }
        "#,
        r#"
            use vstd::prelude::*;
            #[verifier::infer]
            pub fn read(values: &[u8; 2], index: usize) -> u8 { values[index] }
            fn caller() { let r = read(&[3, 5], 0); assert(r == 3); }
        "#,
    ] {
        let crates = InferCrates::new();
        let options = ["--num-threads", "4"];
        let seed = crates.build("safety_base", source, &[], &options);
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        let path = std::fs::read_dir(crates.directory.path().join("candidate-cache"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let mut candidate: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        // P=true passes maximality but admits unsafe executable operations.
        // Only the original ordinary proof must decide contract soundness.
        candidate["pre"] = serde_json::json!({"Bool": true});
        std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
        let run = crates.build("safety_base", source, &[], &options);
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(!run.status.success(), "unsafe candidate was published: {}", stderr);
        assert!(stderr.contains("soundness: NOT PROVED"), "{}", stderr);
        assert!(stderr.contains("candidates were not published"), "{}", stderr);
        assert!(!stderr.contains("contracts available"), "{}", stderr);
        assert!(!crates.vir("safety_base").exists());
    }
}

#[test]
fn early_returns_compose_through_cached_crates_and_fail_closed() {
    let crates = InferCrates::new();
    let base = r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn debit(balance: u8, amount: u8) -> Result<u8, u8> {
            if amount > balance { return Err(1); }
            Ok(balance - amount)
        }
    "#;
    let middle = r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn transfer(balance: u8, amount: u8, fee: u8) -> Result<u8, u8> {
            let total = match amount.checked_add(fee) {
                None => return Err(2),
                Some(total) => total,
            };
            let remaining = match early_base::debit(balance, total) {
                Err(code) => return Err(code),
                Ok(remaining) => remaining,
            };
            Ok(remaining)
        }
    "#;
    let app = r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn pay(balance: u8, amount: u8, fee: u8) -> Result<u8, u8> {
            let remaining = match early_middle::transfer(balance, amount, fee) {
                Err(code) => return Err(code),
                Ok(remaining) => remaining,
            };
            Ok(remaining)
        }
        pub fn conservation(balance: u8, amount: u8, fee: u8) {
            match pay(balance, amount, fee) {
                Ok(left) => assert(left as int + amount as int + fee as int == balance as int),
                Err(code) => assert(code == 1 || code == 2),
            }
        }
    "#;
    for _ in 0..2 {
        crates.succeeds("early_base", base, &[]);
        crates.succeeds("early_middle", middle, &["early_base"]);
        crates.succeeds("early_app", app, &["early_base", "early_middle"]);
    }
    let cache = crates.directory.path().join("candidate-cache");
    // Fingerprints include diagnostic locations, so materializing an imported
    // cached summary can conservatively create additional dependent entries.
    assert!(std::fs::read_dir(&cache).unwrap().count() >= 3);
    for entry in std::fs::read_dir(cache).unwrap() {
        let path = entry.unwrap().path();
        let mut candidate: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        if candidate["dependencies"].as_array().unwrap().is_empty() {
            candidate["pre"] = serde_json::json!({"Bool": false});
            std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
        }
    }
    let run = crates.build("early_base", base, &[], &[]);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(!run.status.success(), "{}", stderr);
    assert!(stderr.contains("safe-domain maximality: NOT PROVED"), "{}", stderr);
    assert!(!crates.vir("early_base").exists());
    let run = crates.build("early_app", app, &["early_base", "early_middle"], &[]);
    assert!(!run.status.success());
    assert!(!crates.vir("early_app").exists());
}

test_verify_one_file! {
    #[test] shared_arrays_length_repetition_and_borrowed_composition code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn lookup(values: &[u8; 4], index: usize) -> Option<u8> {
            if index < values.len() { Some(values[index]) } else { None }
        }
        #[verus_infer]
        fn relay(values: &[u8; 4], index: usize) -> Option<u8> { lookup(values, index) }
        #[verus_infer]
        fn repeated(x: u8, index: usize) -> u8 {
            let values = [x; 4];
            if index < values.len() { values[index] } else { 0 }
        }
        #[verus_infer]
        fn borrowed(x: u8, index: usize) -> Option<u8> {
            let values = [x; 4]; let shared = &values; relay(shared, index)
        }
        verus! {
            fn caller() {
                let values = [10, 20, 30, 40];
                let r = lookup(&values, 0); assert(r == Some(10u8));
                let r = lookup(&values, 3); assert(r == Some(40u8));
                let r = lookup(&values, 4); assert(r == None);
                let r = relay(&values, usize::MAX); assert(r == None);
                let r = repeated(255, 2); assert(r == 255);
                let r = repeated(255, 99); assert(r == 0);
                let r = borrowed(255, 0); assert(r == Some(255u8));
                let r = borrowed(255, 4); assert(r == None);
            }
        }
    } => Ok(diags) => {
        for label in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
            assert_eq!(diags.notes.iter().filter(|d| d.message.contains(label)).count(), 4);
        }
    }
}

test_verify_one_file! {
    #[test] repeated_empty_arrays_preserve_safety_and_maximum_length code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn empty(x: u8) -> u8 { let values = [x + 1; 0]; values.len() as u8 }
        #[verus_infer]
        fn last(x: u128) -> u128 { let values = [x; 64]; values[63] }
        #[verus_infer]
        fn flags(flag: bool, index: usize) -> bool {
            let values = [flag; 2]; index < values.len() && values[index]
        }
        verus! {
            fn caller() {
                let r = empty(254); assert(r == 0);
                let r = last(u128::MAX); assert(r == u128::MAX);
                let r = flags(true, 1); assert(r);
                let r = flags(true, usize::MAX); assert(!r);
            }
        }
    } => Ok(diags) => {
        assert_eq!(diags.notes.iter().filter(|d| d.message.contains("output exactness: PROVED")).count(), 3);
    }
}

test_verify_one_file! {
    #[test] zero_length_repeat_does_not_hide_operand_overflow code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn empty(x: u8) -> u8 { let values = [x + 1; 0]; values.len() as u8 }
        verus! { fn caller() { let _ = empty(255); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] unguarded_shared_array_read_requires_bounds code! {
        use vstd::prelude::*;
        #[verus_infer] fn read(values: &[u8; 2], index: usize) -> u8 { values[index] }
        verus! { fn caller() { let values = [1, 2]; let _ = read(&values, 2); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] mutable_array_references_remain_unsupported code! {
        use vstd::prelude::*;
        #[verus_infer] fn read(values: &mut [u8; 2]) -> u8 { values[0] }
    } => Err(e) => assert_vir_error_msg(e, "parameters and return values must be")
}

test_verify_one_file! {
    #[test] shared_arrays_remain_exact_mode_only code! {
        use vstd::prelude::*;
        #[verus_infer(precision = "sound")] fn read(values: &[u8; 2]) -> u8 { values[0] }
    } => Err(e) => assert_vir_error_msg(e, "arrays and usize indices")
}

test_verify_one_file! {
    #[test] oversized_repetition_is_rejected code! {
        use vstd::prelude::*;
        #[verus_infer] fn oversized(x: u8) -> u8 { let values = [x; 65]; values[0] }
    } => Err(e) => assert_vir_error_msg(e, "unsupported instantiation of trusted array model")
}

test_verify_one_file! {
    #[test] lookalike_array_model_is_not_approved code! {
        use vstd::prelude::*;
        #[verus_verify]
        fn array_fill_for_copy_types(x: u8) -> [u8; 4] { [x; 4] }
        #[verus_infer] fn read(x: u8) -> u8 { let values = array_fill_for_copy_types(x); values[0] }
    } => Err(e) => assert_vir_error_msg(e, "same-crate inferred callee")
}

const SHARED_ARRAY_BASE: &str = r#"
    use vstd::prelude::*;
    #[verifier::infer]
    pub fn lookup(values: &[u8; 4], index: usize) -> Option<u8> {
        if index < values.len() { Some(values[index]) } else { None }
    }
    #[verifier::infer]
    pub fn repeated(x: u8, index: usize) -> Option<u8> {
        let values = [x; 4]; lookup(&values, index)
    }
"#;
const SHARED_ARRAY_MIDDLE: &str = r#"
    use vstd::prelude::*;
    #[verifier::infer]
    pub fn forward(values: &[u8; 4], index: usize) -> Option<u8> { shared_base::lookup(values, index) }
"#;

#[test]
fn shared_array_caches_imports_and_transitive_model_provenance() {
    let crates = InferCrates::new();
    let vstd_path = crates.directory.path().join("vstd.vir");
    std::fs::copy(crates.profile.join("vstd.vir"), &vstd_path).unwrap();
    crates.succeeds("shared_base", SHARED_ARRAY_BASE, &[]);
    crates.succeeds("shared_base", SHARED_ARRAY_BASE, &[]);
    crates.edit_metadata("shared_base", |metadata| {
        assert_eq!(metadata["trusted_models"].as_array().unwrap().len(), 3);
    });
    crates.succeeds("shared_middle", SHARED_ARRAY_MIDDLE, &["shared_base"]);
    crates.succeeds("shared_middle", SHARED_ARRAY_MIDDLE, &["shared_base"]);
    crates.succeeds(
        "shared_app",
        r#"
        use vstd::prelude::*;
        pub fn property() {
            let values = [1, 2, 3, 4];
            let r = shared_middle::forward(&values, 3); assert(r == Some(4u8));
            let r = shared_middle::forward(&values, 4); assert(r == None);
            let r = shared_base::repeated(255, 2); assert(r == Some(255u8));
        }
    "#,
        &["shared_base", "shared_middle"],
    );
    // The declaration identities remain unchanged; changing the containing
    // artifact must still invalidate certificates that depend on its axioms.
    let mut changed_vstd = std::fs::read(&vstd_path).unwrap();
    changed_vstd.push(b' ');
    std::fs::write(&vstd_path, changed_vstd).unwrap();
    let run = crates.build("shared_middle", SHARED_ARRAY_MIDDLE, &["shared_base"], &[]);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(!run.status.success(), "changed standard models were silently accepted");
    assert!(stderr.contains("stale or missing trusted-model fingerprint"), "{}", stderr);
    assert!(!crates.vir("shared_middle").exists());
    let cache = crates.directory.path().join("candidate-cache");
    let before = std::fs::read_dir(&cache).unwrap().count();
    // Same source, import path, and executable model declarations. A changed
    // model artifact must invalidate candidate identity as well as certificates.
    crates.succeeds("shared_base", SHARED_ARRAY_BASE, &[]);
    assert_eq!(std::fs::read_dir(&cache).unwrap().count(), before + 2);
    crates.succeeds("shared_middle", SHARED_ARRAY_MIDDLE, &["shared_base"]);
}

test_verify_one_file! {
    #[test] question_mark_option_checked_chain_and_safety code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn chain(x: u8, y: u8) -> Option<u8> {
            let first = x.checked_add(y)?;
            let second = first.checked_sub(y)?;
            Some(second)
        }
        #[verus_infer]
        fn guarded(input: Option<u8>, x: u8) -> Option<u8> {
            let value = input?;
            Some(value + x)
        }
        #[verus_infer]
        fn unit(input: Option<()>) -> Option<bool> { input?; Some(true) }
        #[verus_infer]
        fn flag(input: Option<bool>) -> Option<()> { if input? { return None; } Some(()) }
        verus! {
            fn caller() {
                let r = chain(255, 1); assert(r == None);
                let r = chain(254, 1); assert(r == Some(254u8));
                let r = guarded(None, 255); assert(r == None);
                let r = guarded(Some(254), 1); assert(r == Some(255u8));
                let r = unit(None); assert(r == None);
                let r = unit(Some(())); assert(r == Some(true));
                let r = flag(None); assert(r == None);
                let r = flag(Some(true)); assert(r == None);
                let r = flag(Some(false)); assert(r == Some(()));
            }
        }
    } => Ok(diags) => {
        for label in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
            assert_eq!(diags.notes.iter().filter(|d| d.message.contains(label)).count(), 4);
        }
    }
}

test_verify_one_file! {
    #[test] question_mark_result_identity_payloads_and_branches code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn change_success(input: Result<u8, u128>) -> Result<bool, u128> {
            let x = input?; Ok(x == 0)
        }
        #[verus_infer]
        fn unit(input: Result<(), bool>) -> Result<u16, bool> { input?; Ok(65535) }
        #[verus_infer]
        fn select_result(first: Result<u8, ()>, second: Result<u8, ()>, yes: bool) -> Result<u8, ()> {
            let x = if yes { first? } else { second? };
            if x == 0 { return Ok(0); }
            Ok(255 / x)
        }
        verus! {
            fn caller() {
                let r = change_success(Err(u128::MAX)); assert(r == Err::<bool, u128>(u128::MAX));
                let r = change_success(Ok(0)); assert(r == Ok::<bool, u128>(true));
                let r = unit(Err(true)); assert(r == Err::<u16, bool>(true));
                let r = unit(Ok(())); assert(r == Ok::<u16, bool>(65535));
                let r = select_result(Err(()), Ok(0), false); assert(r == Ok::<u8, ()>(0));
                let r = select_result(Err(()), Ok(0), true); assert(r == Err::<u8, ()>(()));
                let r = select_result(Ok(3), Err(()), true); assert(r == Ok::<u8, ()>(85));
            }
        }
    } => Ok(diags) => {
        for label in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
            assert_eq!(diags.notes.iter().filter(|d| d.message.contains(label)).count(), 3);
        }
    }
}

test_verify_one_file! {
    #[test] question_mark_operand_order_and_array_bounds code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn lookup(values: &[u8; 2], index: usize) -> Option<u8> {
            if index < values.len() { Some(values[index]) } else { None }
        }
        #[verus_infer]
        fn read(values: &[u8; 2], index: usize, input: Option<u8>) -> Option<u8> {
            Some(input? + values[index])
        }
        #[verus_infer]
        fn order(first: Option<u8>, x: u8) -> Option<u8> { Some(first? + (x + 1)) }
        #[verus_infer]
        fn nested(values: &[u8; 2], index: usize) -> Option<u8> {
            Some(lookup(values, index)?.checked_add(1)?)
        }
        verus! {
            fn caller() {
                let values = [1, 255];
                let r = read(&values, usize::MAX, None); assert(r == None);
                let r = read(&values, 0, Some(2)); assert(r == Some(3u8));
                let r = order(None, 255); assert(r == None);
                let r = order(Some(1), 253); assert(r == Some(255u8));
                let r = nested(&values, 0); assert(r == Some(2u8));
                let r = nested(&values, 1); assert(r == None);
                let r = nested(&values, 2); assert(r == None);
            }
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] question_mark_preserves_prefix_overflow code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn before(input: Option<u8>, x: u8) -> Option<u8> { Some((x + 1) + input?) }
        verus! { fn caller() { let _ = before(None, 255); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] question_mark_success_still_requires_safety code! {
        use vstd::prelude::*;
        #[verus_infer] fn after(input: Option<u8>) -> Option<u8> { Some(input? + 1) }
        verus! { fn caller() { let _ = after(Some(255)); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] question_mark_rejects_error_conversions code! {
        use vstd::prelude::*;
        #[verus_infer] fn convert(input: Result<u8, u8>) -> Result<u8, u16> { Ok(input?) }
    } => Err(e) => assert_vir_error_msg(e, "requires identical Result error types")
}

test_verify_one_file! {
    #[test] question_mark_cannot_hide_proof_assumptions verus_code! {
        use vstd::prelude::*;
        #[verifier::infer]
        fn hidden(input: Option<u8>) -> Option<u8> { Some({ assume(false); input }?) }
    } => Err(e) => assert_vir_error_msg(e, "proof statements and assumptions")
}

test_verify_one_file! {
    #[test] question_mark_result_does_not_invent_no_unwind verus_code! {
        use vstd::prelude::*;
        #[verifier::infer]
        fn propagate(input: Result<u8, u8>) -> Result<u8, u8>
            no_unwind
        { Ok(input?) }
    } => Err(e) => {
        assert!(e.notes.iter().any(|d| d.message.contains("soundness: NOT PROVED")));
        assert!(e.errors.iter().any(|d| d.message.contains("unwind")));
    }
}

test_verify_one_file! {
    #[test] question_mark_remains_exact_mode_only code! {
        use vstd::prelude::*;
        #[verus_infer(precision = "sound")]
        fn propagate(input: Option<u8>) -> Option<u8> { Some(input?) }
    } => Err(e) => assert_vir_error_msg(e, "currently requires exact mode")
}

#[test]
fn question_mark_cached_crate_chain_conservation_and_model_provenance() {
    let crates = InferCrates::new();
    let vstd_path = crates.directory.path().join("vstd.vir");
    std::fs::copy(crates.profile.join("vstd.vir"), &vstd_path).unwrap();
    let base = r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn debit(balance: u8, amount: u8) -> Result<u8, u8> {
            if amount > balance { return Err(1); }
            Ok(balance - amount)
        }
        #[verifier::infer]
        pub fn charge(balance: u8, amount: u8) -> Result<u8, u8> {
            Ok(debit(balance, amount)?)
        }
    "#;
    let middle = r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn transfer(balance: u8, amount: u8, fee: u8) -> Result<u8, u8> {
            let total = match amount.checked_add(fee) { None => return Err(2), Some(x) => x };
            Ok(try_base::charge(balance, total)?)
        }
    "#;
    let app = r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn pay(balance: u8, amount: u8, fee: u8) -> Result<u8, u8> {
            Ok(try_middle::transfer(balance, amount, fee)?)
        }
        pub fn conservation(balance: u8, amount: u8, fee: u8) {
            match pay(balance, amount, fee) {
                Ok(left) => assert(left as int + amount as int + fee as int == balance as int),
                Err(code) => assert(code == 1 || code == 2),
            }
        }
    "#;
    for _ in 0..2 {
        crates.succeeds("try_base", base, &[]);
        crates.succeeds("try_middle", middle, &["try_base"]);
        crates.succeeds("try_app", app, &["try_base", "try_middle"]);
    }
    crates.edit_metadata("try_base", |m| {
        assert_eq!(m["trusted_models"].as_array().unwrap().len(), 2);
    });
    // The identity helper is unchanged, but the library containing its trusted
    // specification is part of both certificate and candidate dependencies.
    let mut changed = std::fs::read(&vstd_path).unwrap();
    changed.push(b' ');
    std::fs::write(&vstd_path, changed).unwrap();
    let run = crates.build("try_middle", middle, &["try_base"], &[]);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(!run.status.success());
    assert!(stderr.contains("stale or missing trusted-model fingerprint"), "{}", stderr);
    assert!(!crates.vir("try_middle").exists());
    crates.succeeds("try_base", base, &[]);
    crates.succeeds("try_middle", middle, &["try_base"]);
    crates.succeeds("try_app", app, &["try_base", "try_middle"]);
}

#[test]
fn question_mark_cached_candidates_retain_all_required_proof_gates() {
    let source = r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn increment(input: Option<u8>) -> Option<u8> { Some(input? + 1) }
        pub fn caller() { let r = increment(None); assert(r == None); }
    "#;
    for mutation in ["restrict", "unsafe", "output"] {
        let crates = InferCrates::new();
        let options = ["--num-threads", "4"];
        let seed = crates.build("try_gates", source, &[], &options);
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        let entries: Vec<_> = std::fs::read_dir(crates.directory.path().join("candidate-cache"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert_eq!(entries.len(), 1);
        let path = &entries[0];
        let mut candidate: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        match mutation {
            "restrict" => candidate["pre"] = serde_json::json!({"Bool": false}),
            "unsafe" => candidate["pre"] = serde_json::json!({"Bool": true}),
            // input has the correct Option<u8> type, but is not the result.
            "output" => candidate["post"]["Binary"][2] = serde_json::json!({"Param": 0}),
            _ => unreachable!(),
        }
        std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
        let run = crates.build("try_gates", source, &[], &options);
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(!run.status.success(), "{}: {}", mutation, stderr);
        let label = if mutation == "restrict" {
            "safe-domain maximality: NOT PROVED"
        } else {
            "soundness: NOT PROVED"
        };
        assert!(stderr.contains(label), "{}: {}", mutation, stderr);
        assert!(stderr.contains("candidates were not published"), "{}", stderr);
        assert!(!stderr.contains("contracts available"), "{}", stderr);
        assert!(!crates.vir("try_gates").exists());
    }
}

test_verify_one_file! {
    #[test] signed_arithmetic_all_widths_and_literals code! {
        use vstd::prelude::*;
        #[verus_infer] fn add(x: i8, y: i8) -> i8 { x + y }
        #[verus_infer] fn sub(x: i16, y: i16) -> i16 { x - y }
        #[verus_infer] fn mul(x: i32, y: i32) -> i32 { x * y }
        #[verus_infer] fn div(x: i64, y: i64) -> i64 { x / y }
        #[verus_infer] fn rem(x: i128, y: i128) -> i128 { x % y }
        #[verus_infer] fn neg(x: i128) -> i128 { -x }
        #[verus_infer] fn literals() -> i8 { -128 }
        #[verus_infer] fn minimum() -> i128 { -170141183460469231731687303715884105728 }
        verus! {
            fn caller() {
                let r = add(-128, 1); assert(r == -127);
                let r = add(127, -1); assert(r == 126);
                let r = sub(-32768, -1); assert(r == -32767);
                let r = mul(-2147483648, 1); assert(r == -2147483648);
                let r = div(-7, 3); assert(r == -2);
                let r = rem(-7, 3); assert(r == -1);
                let r = neg(i128::MAX); assert(r == i128::MIN + 1);
                let r = literals(); assert(r == i8::MIN);
                let r = minimum(); assert(r == i128::MIN);
            }
        }
    } => Ok(diags) => {
        for label in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
            assert_eq!(diags.notes.iter().filter(|d| d.message.contains(label)).count(), 8);
        }
    }
}

test_verify_one_file! {
    #[test] signed_division_remainder_and_guarded_edge_cases code! {
        use vstd::prelude::*;
        #[verus_infer] fn quotient(x: i8, y: i8) -> i8 { x / y }
        #[verus_infer] fn remainder(x: i8, y: i8) -> i8 { x % y }
        #[verus_infer]
        fn guarded(x: i8, y: i8) -> Result<i8, i8> {
            if y == 0 { return Err(0); }
            if x == i8::MIN && y == -1 { return Err(-1); }
            Ok(x / y)
        }
        verus! {
            fn caller() {
                let r = quotient(-7, 3); assert(r == -2);
                let r = quotient(7, -3); assert(r == -2);
                let r = quotient(-7, -3); assert(r == 2);
                let r = remainder(-7, 3); assert(r == -1);
                let r = remainder(7, -3); assert(r == 1);
                let r = remainder(-7, -3); assert(r == -1);
                let r = quotient(i8::MIN, 1); assert(r == i8::MIN);
                let r = remainder(i8::MIN, 1); assert(r == 0);
                let r = guarded(i8::MIN, -1); assert(r == Err::<i8, i8>(-1i8));
                let r = guarded(1, 0); assert(r == Err::<i8, i8>(0));
                let r = guarded(-7, 3); assert(r == Ok::<i8, i8>(-2i8));
            }
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] signed_checked_methods_distinguish_rust_and_euclidean code! {
        use vstd::prelude::*;
        #[verus_infer] fn add(x: i8, y: i8) -> Option<i8> { x.checked_add(y) }
        #[verus_infer] fn sub(x: i16, y: i16) -> Option<i16> { x.checked_sub(y) }
        #[verus_infer] fn mul(x: i32, y: i32) -> Option<i32> { x.checked_mul(y) }
        #[verus_infer] fn div(x: i64, y: i64) -> Option<i64> { x.checked_div(y) }
        #[verus_infer] fn rem(x: i128, y: i128) -> Option<i128> { x.checked_rem(y) }
        #[verus_infer] fn euclidean_div(x: i8, y: i8) -> Option<i8> { x.checked_div_euclid(y) }
        #[verus_infer] fn euclidean_rem(x: i8, y: i8) -> Option<i8> { x.checked_rem_euclid(y) }
        verus! {
            fn caller() {
                let r = add(127, 1); assert(r == None);
                let r = add(-128, -1); assert(r == None);
                let r = add(-128, 127); assert(r == Some(-1i8));
                let r = sub(i16::MIN, 1); assert(r == None);
                let r = sub(i16::MAX, -1); assert(r == None);
                let r = mul(i32::MIN, -1); assert(r == None);
                let r = mul(-7, -3); assert(r == Some(21i32));
                let r = div(i64::MIN, -1); assert(r == None);
                let r = div(-7, 3); assert(r == Some(-2i64));
                let r = rem(i128::MIN, -1); assert(r == None);
                let r = rem(-7, 3); assert(r == Some(-1i128));
                let r = euclidean_div(-7, 3); assert(r == Some(-3i8));
                let r = euclidean_div(-7, -3); assert(r == Some(3i8));
                let r = euclidean_rem(-7, -3); assert(r == Some(2i8));
                let r = euclidean_div(i8::MIN, -1); assert(r == None);
                let r = euclidean_rem(i8::MIN, -1); assert(r == None);
                let r = euclidean_rem(7, 0); assert(r == None);
            }
        }
    } => Ok(diags) => {
        assert_eq!(diags.notes.iter().filter(|d| d.message.contains("output exactness: PROVED")).count(), 7);
    }
}

test_verify_one_file! {
    #[test] signed_casts_preserve_truncation_and_sign_extension code! {
        use vstd::prelude::*;
        #[verus_infer] fn narrow(x: i16) -> i8 { x as i8 }
        #[verus_infer] fn extend(x: i8) -> i128 { x as i128 }
        #[verus_infer] fn unsigned(x: i8) -> u8 { x as u8 }
        #[verus_infer] fn signed(x: u8) -> i8 { x as i8 }
        #[verus_infer] fn wide_unsigned(x: i8) -> u128 { x as u128 }
        #[verus_infer] fn wide_signed(x: u128) -> i128 { x as i128 }
        verus! {
            fn caller() {
                assert((-129i16 as i8) == 127) by(bit_vector);
                let r = narrow(-129); assert(r == 127);
                assert((128i16 as i8) == -128) by(bit_vector);
                let r = narrow(128); assert(r == -128);
                assert((-32768i16 as i8) == 0) by(bit_vector);
                let r = narrow(-32768); assert(r == 0);
                let r = extend(-128); assert(r == -128);
                assert((-1i8 as u8) == 255) by(bit_vector);
                let r = unsigned(-1); assert(r == 255);
                assert((255u8 as i8) == -1) by(bit_vector);
                let r = signed(255); assert(r == -1);
                assert((-1i8 as u128) == u128::MAX) by(bit_vector);
                let r = wide_unsigned(-1); assert(r == u128::MAX);
                assert((u128::MAX as i128) == -1) by(bit_vector);
                let r = wide_signed(u128::MAX); assert(r == -1);
            }
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] signed_values_compose_with_enums_propagation_and_arrays code! {
        use vstd::prelude::*;
        #[verus_infer]
        fn lookup(values: &[i16; 2], index: usize) -> Option<i16> {
            if index < values.len() { Some(values[index]) } else { None }
        }
        #[verus_infer]
        fn adjusted(values: &[i16; 2], index: usize, delta: i16) -> Option<i16> {
            Some(lookup(values, index)?.checked_add(delta)?)
        }
        #[verus_infer]
        fn repeated(x: i8, index: usize) -> Option<i8> {
            let values = [x - 1; 2];
            if index < values.len() { Some(values[index]) } else { None }
        }
        #[verus_infer]
        fn classify(input: Result<i8, i16>) -> Result<bool, i16> {
            let x = input?;
            match x { -1 => Ok(true), _ => Ok(false) }
        }
        verus! {
            fn caller() {
                let values = [-32768, 32767];
                let r = adjusted(&values, 0, 1); assert(r == Some(-32767i16));
                let r = adjusted(&values, 0, -1); assert(r == None);
                let r = adjusted(&values, 1, 1); assert(r == None);
                let r = adjusted(&values, usize::MAX, -32768); assert(r == None);
                let r = repeated(-127, 1); assert(r == Some(-128i8));
                let r = classify(Err(-32768)); assert(r == Err::<bool, i16>(-32768i16));
                let r = classify(Ok(-1)); assert(r == Ok::<bool, i16>(true));
                let r = classify(Ok(-128)); assert(r == Ok::<bool, i16>(false));
            }
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] signed_positive_overflow_requires_precondition code! {
        #[verus_infer] fn add(x: i8, y: i8) -> i8 { x + y }
        verus! { fn caller() { let _ = add(127, 1); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] signed_negative_overflow_requires_precondition code! {
        #[verus_infer] fn sub(x: i8, y: i8) -> i8 { x - y }
        verus! { fn caller() { let _ = sub(-128, 1); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] signed_negating_min_requires_precondition code! {
        #[verus_infer] fn neg(x: i8) -> i8 { -x }
        verus! { fn caller() { let _ = neg(i8::MIN); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] signed_division_min_minus_one_requires_precondition code! {
        use vstd::prelude::*;
        #[verus_infer] fn div(x: i8, y: i8) -> i8 { x / y }
        verus! { fn caller() { let _ = div(i8::MIN, -1); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] signed_remainder_min_minus_one_requires_precondition code! {
        use vstd::prelude::*;
        #[verus_infer] fn rem(x: i8, y: i8) -> i8 { x % y }
        verus! { fn caller() { let _ = rem(i8::MIN, -1); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] signed_zero_divisor_requires_precondition code! {
        use vstd::prelude::*;
        #[verus_infer] fn div(x: i128, y: i128) -> i128 { x / y }
        verus! { fn caller() { let _ = div(-1, 0); /* FAILS */ } }
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] signed_interval_subtraction_is_supported code! {
        #[verus_infer(precision = "sound")] fn sub(x: i8, y: i8) -> i8 { x - y }
        verus! { fn caller() { let r = sub(-128, -1); assert(-128 <= r && r <= 127); } }
    } => Ok(diags) => {
        assert!(diags.notes.iter().any(|d|d.message.contains("soundness: PROVED")));
    }
}

test_verify_one_file! {
    #[test] signed_division_without_vstd_is_rejected_cleanly code! {
        #[verus_infer] fn quotient(x: i8, y: i8) -> i8 { x / y }
    } => Err(e) => assert_vir_error_msg(e, "requires vstd arithmetic models")
}

test_verify_one_file! {
    #[test] signed_temporaries_in_interval_mode_are_supported code! {
        #[verus_infer(precision = "sound")] fn negative(x: u8) -> bool { (x as i8) < 0 }
        verus! { fn caller() { let r = negative(255); assert(r || !r); } }
    } => Ok(diags) => {
        assert!(diags.notes.iter().any(|d|d.message.contains("soundness: PROVED")));
    }
}

#[test]
fn signed_cached_crates_preserve_rust_division_and_model_provenance() {
    let crates = InferCrates::new();
    let vstd_path = crates.directory.path().join("vstd.vir");
    std::fs::copy(crates.profile.join("vstd.vir"), &vstd_path).unwrap();
    let base = r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn divide(x: i16, y: i16) -> i16 { x / y }
        #[verifier::infer]
        pub fn narrow(x: u16) -> i8 { x as i8 }
    "#;
    let middle = r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn shifted(x: i16, y: i16, delta: i16) -> Option<i16> {
            let quotient=signed_base::divide(x,y);
            Some(quotient.checked_add(delta)?)
        }
        #[verifier::infer]
        pub fn remainder(x: i16, y: i16) -> i16 { x % y }
    "#;
    let app = r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn quote(x: i16, y: i16, delta: i16) -> Option<i16> {
            Some(signed_middle::shifted(x,y,delta)?)
        }
        pub fn property(x: i16, y: i16, delta: i16)
            requires y != 0, x != i16::MIN || y != -1,
        {
            let quotient=signed_base::divide(x,y);
            let result=quote(x,y,delta);
            match result {
                Some(value)=>assert(value as int == quotient as int+delta as int),
                None=>{},
            }
            let r=quote(-7,3,-1); assert(r==Some(-3i16));
            let r=signed_middle::remainder(-7,3);assert(r==-1);
            assert(255u16 as i8 == -1) by(bit_vector);
            let r=signed_base::narrow(255);assert(r==-1);
        }
    "#;
    for _ in 0..2 {
        crates.succeeds("signed_base", base, &[]);
        crates.succeeds("signed_middle", middle, &["signed_base"]);
        crates.succeeds("signed_app", app, &["signed_base", "signed_middle"]);
    }
    crates.edit_metadata("signed_base", |m| {
        assert_eq!(m["trusted_models"].as_array().unwrap().len(), 1)
    });
    let mut changed = std::fs::read(&vstd_path).unwrap();
    changed.push(b' ');
    std::fs::write(&vstd_path, changed).unwrap();
    let run = crates.build("signed_middle", middle, &["signed_base"], &[]);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(!run.status.success());
    assert!(stderr.contains("stale or missing trusted-model fingerprint"), "{}", stderr);
    assert!(!crates.vir("signed_middle").exists());
    crates.succeeds("signed_base", base, &[]);
    crates.succeeds("signed_middle", middle, &["signed_base"]);
    crates.succeeds("signed_app", app, &["signed_base", "signed_middle"]);
}

#[test]
fn signed_cached_candidates_cannot_drop_min_minus_one_or_other_proof_gates() {
    let source = r#"
        use vstd::prelude::*;
        #[verifier::infer]
        pub fn remainder(x: i8,y: i8)->i8 {x%y}
        pub fn caller() {let r=remainder(-7,3);assert(r==-1);}
    "#;
    for mutation in ["restrict", "edge", "output"] {
        let crates = InferCrates::new();
        let options = ["--num-threads", "4"];
        let seed = crates.build("signed_gates", source, &[], &options);
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        let entries: Vec<_> = std::fs::read_dir(crates.directory.path().join("candidate-cache"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert_eq!(entries.len(), 1);
        let path = &entries[0];
        let mut candidate: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        match mutation {
            "restrict" => candidate["pre"] = serde_json::json!({"Bool":false}),
            "edge" => candidate["pre"] = candidate["pre"]["Logical"][1].clone(),
            "output" => candidate["post"]["Binary"][2] = serde_json::json!({"Param":0}),
            _ => unreachable!(),
        }
        std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
        let run = crates.build("signed_gates", source, &[], &options);
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(!run.status.success(), "{}: {}", mutation, stderr);
        let label = if mutation == "restrict" {
            "safe-domain maximality: NOT PROVED"
        } else {
            "soundness: NOT PROVED"
        };
        assert!(stderr.contains(label), "{}: {}", mutation, stderr);
        if mutation == "edge" {
            assert!(stderr.contains("possible overflow case for signed remainder"), "{}", stderr);
        }
        assert!(stderr.contains("candidates were not published"), "{}", stderr);
        assert!(!crates.vir("signed_gates").exists());
    }
}

test_verify_one_file! {
    #[test] signed_i128_boundary_arithmetic_and_checked_overflow code! {
        use vstd::prelude::*;
        #[verus_infer] fn add(x:i128,y:i128)->i128 {x+y}
        #[verus_infer] fn sub(x:i128,y:i128)->i128 {x-y}
        #[verus_infer] fn mul(x:i128,y:i128)->i128 {x*y}
        #[verus_infer] fn checked(x:i128,y:i128)->Option<i128> {x.checked_mul(y)}
        verus! {
            fn caller() {
                let r=add(i128::MIN,i128::MAX);assert(r==-1);
                let r=sub(i128::MIN,-1);assert(r==i128::MIN+1);
                let r=mul(i128::MIN,1);assert(r==i128::MIN);
                let r=mul(i128::MAX,-1);assert(r==i128::MIN+1);
                let r=checked(i128::MIN,-1);assert(r==None);
                let r=checked(i128::MAX,2);assert(r==None);
                let r=checked(i128::MIN,1);assert(r==Some(i128::MIN));
            }
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] signed_interval_scaling_remainders_and_negative_divisors code! {
        use vstd::prelude::*;
        #[verus_infer(precision = "sound")] fn scaled(x:i8)->i8 {x/10}
        #[verus_infer(precision = "sound")] fn negative_divisor(x:i8)->i8 {x / -10}
        #[verus_infer(precision = "sound")] fn remainder(x:i8)->i8 {x%10}
        #[verus_infer(precision = "sound")] fn negative_remainder(x:i8)->i8 {x % -10}
        verus! {
            fn property(x:i8) {
                let r=scaled(x);assert(-12<=r&&r<=12);
                let r=negative_divisor(x);assert(-12<=r&&r<=12);
                let r=remainder(x);assert(-9<=r&&r<=9);
                let r=negative_remainder(x);assert(-9<=r&&r<=9);
            }
        }
    } => Ok(diags) => {
        assert_eq!(diags.notes.iter().filter(|d| d.message.contains("soundness: PROVED")).count(),4);
        assert_eq!(diags.notes.iter().filter(|d| d.message.contains("safe-domain maximality: PROVED")).count(),4);
        assert_eq!(diags.notes.iter().filter(|d| d.message.contains("output exactness: NOT PROVED")).count(),4);
    }
}

test_verify_one_file! {
    #[test] signed_interval_range_does_not_grant_exact_result code! {
        use vstd::prelude::*;
        #[verus_infer(precision = "sound")] fn scaled(x:i8)->i8 {x/10}
        verus! {fn caller(){let r=scaled(30);assert(r==3); /* FAILS */}}
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] signed_interval_negation_casts_and_all_widths code! {
        use vstd::prelude::*;
        #[verus_infer(precision = "sound")] fn negate(x:i8)->i8 {-x}
        #[verus_infer(precision = "sound")] fn choices(b:bool)->i16 {if b {-20}else{10}}
        #[verus_infer(precision = "sound")] fn minimum()->i128 {-170141183460469231731687303715884105728}
        #[verus_infer(precision = "sound")] fn narrow(x:i32)->i16 {x as i16}
        #[verus_infer(precision = "sound")] fn widen(x:i8)->i64 {x as i64}
        #[verus_infer(precision = "sound")] fn unsigned(x:i8)->u128 {x as u128}
        #[verus_infer(precision = "sound")] fn signed(x:u128)->i128 {x as i128}
        verus! {
            fn property(x:i8,b:bool) {
                let r=choices(b);assert(-20<=r&&r<=10);
                let r=minimum();assert(r==i128::MIN);
                let r=narrow(-32769);assert(i16::MIN<=r&&r<=i16::MAX);
                let r=widen(x);assert(-128<=r&&r<=127);
                let r=unsigned(-1);assert(r<=u128::MAX);
                let r=signed(u128::MAX);assert(i128::MIN<=r&&r<=i128::MAX);
                let r=negate(-127);assert(-127<=r&&r<=127);
            }
        }
    } => Ok(diags) => {
        assert!(diags.notes.iter().any(|d|d.message.contains("soundness: PROVED")));
    }
}

test_verify_one_file! {
    #[test] signed_interval_negation_still_requires_safety code! {
        #[verus_infer(precision = "sound")] fn negate(x:i8)->i8 {-x}
        verus! {fn caller(){let _=negate(i8::MIN); /* FAILS */}}
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] signed_interval_remainder_still_excludes_min_minus_one code! {
        use vstd::prelude::*;
        #[verus_infer(precision = "sound")] fn remainder(x:i8,y:i8)->i8 {x%y}
        verus! {fn caller(){let _=remainder(i8::MIN,-1); /* FAILS */}}
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] signed_interval_callees_keep_full_domains_and_honest_precision code! {
        use vstd::prelude::*;
        #[verus_infer(precision = "sound")] fn scaled(x:i8)->i8 {x/10}
        #[verus_infer(precision = "sound")] fn wrapper(x:i8)->i8 {scaled(x)+1}
        verus! {
            fn caller(x:i8)->(r:i8)
                ensures -11<=r&&r<=13,
            {wrapper(x)}
        }
    } => Ok(diags) => {
        assert!(diags.notes.iter().any(|d| d.message.contains("NOT ESTABLISHED (sound-mode dependency)")));
        assert!(!diags.notes.iter().any(|d| d.message.contains("wrapper: safe-domain maximality: PROVED")));
    }
}

test_verify_one_file! {
    #[test] sound_identity_wrappers_accept_full_signed_domains code! {
        #[verus_infer(precision = "sound")] fn leaf(x:i8)->i8 {x}
        #[verus_infer(precision = "sound")] fn wrapper(x:i8)->i8 {leaf(x)}
        verus! {fn caller(x:i8){let r=wrapper(x);assert(i8::MIN<=r&&r<=i8::MAX);}}
    } => Ok(diags) => assert!(diags.notes.iter().any(|d|d.message.contains("soundness: PROVED")))
}

#[test]
fn signed_interval_fallback_survives_exact_expansion_budget() {
    let mut source =
        String::from("#[verus_infer(precision = \"sound\")] fn f(x:i64)->i64 {let a0=x;");
    for i in 1..=16 {
        source.push_str(&format!("let a{i}=a{}+a{};", i - 1, i - 1));
    }
    source.push_str("a16} verus! {fn caller(x:i64)->(r:i64) requires i64::MIN / 65536 <= x && x <= i64::MAX / 65536, ensures i64::MIN <= r && r <= i64::MAX, {f(x)}}");
    let diags =
        verify_one_file("signed_interval_fallback_survives_exact_expansion_budget", source, &[])
            .unwrap();
    assert!(diags.notes.iter().any(|d| d.message.contains("soundness: PROVED")));
    assert!(diags.notes.iter().any(|d| d.message.contains("safe-domain maximality: PROVED")));
    assert!(diags.notes.iter().any(|d| d.message.contains("output exactness: NOT PROVED")));
}

test_verify_one_file! {
    #[test] signed_interval_optional_timeout_preserves_soundness code! {
        #[verus_infer(precision = "sound", precision_rlimit = 1e-40)]
        fn selected(b:bool)->i8 {if b {-2}else{2}}
        verus! {fn caller(){let r=selected(true);assert(-2<=r&&r<=2);}}
    } => Ok(diags) => {
        assert!(diags.notes.iter().any(|d|d.message.contains("soundness: PROVED")));
        assert!(diags.notes.iter().any(|d|d.message.contains("output exactness: INCOMPLETE")));
    }
}

#[test]
fn signed_interval_cross_crate_precision_cache_and_model_dependencies() {
    let crates = InferCrates::new();
    let vstd_path = crates.directory.path().join("vstd.vir");
    let original_vstd = std::fs::read(crates.profile.join("vstd.vir")).unwrap();
    std::fs::write(&vstd_path, &original_vstd).unwrap();
    let base = r#"
        use vstd::prelude::*;
        #[verifier::infer(sound)] pub fn scaled(x:i8)->i8 {x/10}
        #[verifier::infer] pub fn identity(x:i16)->i16 {x}
    "#;
    let middle = r#"
        use vstd::prelude::*;
        #[verifier::infer(sound)] pub fn shifted(x:i8)->i8 {signed_range_base::scaled(x)+1}
    "#;
    let app = r#"
        use vstd::prelude::*;
        #[verifier::infer(sound)] pub fn top(x:i8)->i8 {signed_range_middle::shifted(x)}
        pub fn range_property(x:i8)->(r:i8)
            ensures -11<=r&&r<=13,
        {top(x)}
        #[verifier::infer] pub fn exact_identity(x:i16)->i16 {signed_range_base::identity(x)}
        pub fn exact_property(){let r=exact_identity(-32768);assert(r==-32768);}
    "#;
    for _ in 0..2 {
        crates.succeeds("signed_range_base", base, &[]);
        crates.succeeds("signed_range_middle", middle, &["signed_range_base"]);
        crates.succeeds("signed_range_app", app, &["signed_range_base", "signed_range_middle"]);
    }
    crates.edit_metadata("signed_range_base", |m| {
        assert_eq!(m["soundness"], true);
        assert_eq!(m["exactness"], false);
        let proofs = m["proofs"].as_array().unwrap();
        assert!(proofs.iter().any(|p| p[1]["exactness"] == true));
        assert!(proofs.iter().any(|p| p[1]["exactness"] == false));
        assert_eq!(m["trusted_models"].as_array().unwrap().len(), 1);
    });
    crates.edit_metadata("signed_range_middle", |m| {
        assert_eq!(m["soundness"], true);
        assert_eq!(m["maximality"], false);
        assert_eq!(m["exactness"], false);
    });
    let exact = crates.build(
        "signed_range_exact",
        r#"
        use vstd::prelude::*;
        #[verifier::infer] pub fn exact(x:i8)->i8 {signed_range_middle::shifted(x)}
    "#,
        &["signed_range_base", "signed_range_middle"],
        &[],
    );
    assert!(!exact.status.success());
    assert!(
        String::from_utf8_lossy(&exact.stderr)
            .contains("exact inference requires exact-mode callees")
    );
    assert!(!crates.vir("signed_range_exact").exists());
    let mut changed = original_vstd.clone();
    changed.push(b' ');
    std::fs::write(&vstd_path, changed).unwrap();
    let stale = crates.build("signed_range_middle", middle, &["signed_range_base"], &[]);
    assert!(!stale.status.success());
    assert!(
        String::from_utf8_lossy(&stale.stderr)
            .contains("stale or missing trusted-model fingerprint")
    );
    assert!(!crates.vir("signed_range_middle").exists());
    std::fs::write(&vstd_path, original_vstd).unwrap();
    crates.succeeds("signed_range_middle", middle, &["signed_range_base"]);
    crates.edit_metadata("signed_range_middle", |m| {
        m["proofs"][0][1]["soundness"] = serde_json::json!(false)
    });
    let unsound =
        crates.build("signed_range_app", app, &["signed_range_base", "signed_range_middle"], &[]);
    assert!(!unsound.status.success());
    assert!(
        String::from_utf8_lossy(&unsound.stderr).contains("incomplete inference proof guarantees")
    );
    assert!(!crates.vir("signed_range_app").exists());
}

#[test]
fn signed_interval_poisoned_cache_cannot_drop_negative_results_or_safety() {
    for mutation in ["lower_bound", "safety"] {
        let crates = InferCrates::new();
        let source = if mutation == "lower_bound" {
            r#"
            use vstd::prelude::*;
            #[verifier::infer(sound)] pub fn scaled(x:i8)->i8 {x/10}
        "#
        } else {
            r#"
            #[verifier::infer(sound)] pub fn negate(x:i8)->i8 {-x}
        "#
        };
        let options = ["--num-threads", "4"];
        for _ in 0..2 {
            let run = crates.build("signed_interval_gates", source, &[], &options);
            assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        }
        let entries: Vec<_> = std::fs::read_dir(crates.directory.path().join("candidate-cache"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert_eq!(entries.len(), 1);
        let path = &entries[0];
        let mut candidate: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        if mutation == "lower_bound" {
            candidate["post"]["Logical"][1]["Binary"][1] =
                serde_json::to_value(vir::ast::Constant::Int(0.into())).unwrap();
        } else {
            candidate["pre"] = serde_json::json!({"Bool":true});
        }
        std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
        let run = crates.build("signed_interval_gates", source, &[], &options);
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(!run.status.success(), "{}: {}", mutation, stderr);
        assert!(stderr.contains("soundness: NOT PROVED"), "{}", stderr);
        assert!(stderr.contains("candidates were not published"), "{}", stderr);
        assert!(!stderr.contains("VERIFIED SOUND"), "{}", stderr);
        assert!(!crates.vir("signed_interval_gates").exists());
    }
}

test_verify_one_file! {
    #[test] signed_interval_wide_ranges code! {
        use vstd::prelude::*;
        #[verus_infer(precision = "sound")] fn scaled16(x:i16)->i16 {x/1000}
        #[verus_infer(precision = "sound")] fn scaled32(x:i32)->i32 {x/1000000000}
        #[verus_infer(precision = "sound")] fn scaled64(x:i64)->i64 {x/1000000000000000000}
        #[verus_infer(precision = "sound")] fn scaled128(x:i128)->i128 {x/100000000000000000000000000000000000000}
        verus! {
            fn property(a:i16,b:i32,c:i64,d:i128) {
                let r=scaled16(a);assert(-32<=r&&r<=32);
                let r=scaled32(b);assert(-2<=r&&r<=2);
                let r=scaled64(c);assert(-9<=r&&r<=9);
                let r=scaled128(d);assert(-1<=r&&r<=1);
            }
        }
    } => Ok(diags) => {
        assert_eq!(diags.notes.iter().filter(|d|d.message.contains("soundness: PROVED")).count(),4);
    }
}

test_verify_one_file! {
    #[test] composed_safety_signed_and_unsigned_shifts_accept_all_inputs code! {
        use vstd::prelude::*;
        #[verus_infer(precision="sound")] fn scaled(x:i8)->i8 {x/10}
        #[verus_infer(precision="sound")] fn shifted(x:i8)->i8 {scaled(x)+1}
        #[verus_infer(precision="sound")] fn fee(x:u8)->u8 {x/100}
        #[verus_infer(precision="sound")] fn charged(x:u8)->u8 {fee(x)+1}
        verus! {
            fn caller(x:i8,y:u8) {
                let r=shifted(x);assert(-11<=r&&r<=13);
                let r=charged(y);assert(1<=r&&r<=3);
            }
        }
    } => Ok(diags) => {
        assert!(diags.notes.iter().any(|d|d.message.contains("shifted candidate")&&d.message.contains("requires true;")));
        assert!(diags.notes.iter().any(|d|d.message.contains("charged candidate")&&d.message.contains("requires true;")));
        assert!(diags.notes.iter().any(|d|d.message.contains("NOT ESTABLISHED (sound-mode dependency)")));
    }
}

test_verify_one_file! {
    #[test] composed_safety_derives_offset_limits_from_callee_range code! {
        use vstd::prelude::*;
        #[verus_infer(precision="sound")] fn scaled(x:i8)->i8 {x/10}
        #[verus_infer(precision="sound")] fn offset(x:i8,y:i8)->i8 {scaled(x)+y}
        verus! {
            fn caller(x:i8,y:i8)
                requires -116<=y&&y<=115,
            {let r=offset(x,y);assert(i8::MIN<=r&&r<=i8::MAX);}
        }
    } => Ok(diags) => assert!(diags.notes.iter().any(|d|d.message.contains("offset: soundness: PROVED")))
}

test_verify_one_file! {
    #[test] composed_safety_still_excludes_real_overflow code! {
        use vstd::prelude::*;
        #[verus_infer(precision="sound")] fn scaled(x:i8)->i8 {x/10}
        #[verus_infer(precision="sound")] fn offset(x:i8,y:i8)->i8 {scaled(x)+y}
        verus! {fn caller(){let _=offset(127,116); /* FAILS */}}
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] composed_safety_preserves_real_callee_requirements code! {
        #[verus_infer(precision="sound")] fn increment(x:u8)->u8 {x+1}
        #[verus_infer(precision="sound")] fn relay(x:u8)->u8 {increment(x)}
        verus! {fn caller(x:u8) requires x<255,{let r=relay(x);assert(1<=r&&r<=255);}}
    } => Ok(diags) => assert!(diags.notes.iter().any(|d|d.message.contains("relay: soundness: PROVED")))
}

test_verify_one_file! {
    #[test] composed_safety_does_not_drop_callee_requirements code! {
        #[verus_infer(precision="sound")] fn increment(x:u8)->u8 {x+1}
        #[verus_infer(precision="sound")] fn relay(x:u8)->u8 {increment(x)}
        verus! {fn caller(){let _=relay(255); /* FAILS */}}
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] composed_safety_checks_callee_requirements_for_approximate_arguments code! {
        #[verus_infer(precision="sound")] fn fee(x:u8)->u8 {x/100}
        #[verus_infer(precision="sound")] fn increment(x:u8)->u8 {x+1}
        #[verus_infer(precision="sound")] fn charged(x:u8)->u8 {increment(fee(x))}
        verus! {fn caller(x:u8){let r=charged(x);assert(1<=r&&r<=255);}}
    } => Ok(diags) => assert!(diags.notes.iter().any(|d|d.message.contains("charged candidate")&&d.message.contains("requires true;")))
}

test_verify_one_file! {
    #[test] composed_safety_guards_and_aliases_of_approximate_values code! {
        use vstd::prelude::*;
        #[verus_infer(precision="sound")] fn scaled(x:i8)->i8 {x/10}
        #[verus_infer(precision="sound")] fn identity(x:i8)->i8 {x}
        #[verus_infer(precision="sound")]
        fn quotient(x:i8)->bool {
            let q=scaled(x);let alias=q;
            if alias==0 {false}else{120/q>0}
        }
        #[verus_infer(precision="sound")]
        fn negate(x:i8)->i8 {
            let q=identity(x);
            if q==i8::MIN {0}else{-q}
        }
        #[verus_infer(precision="sound")]
        fn positive(x:i8)->bool {
            let q=scaled(x);q!=0&&120/q>0
        }
        verus! {fn caller(x:i8){let _=quotient(x);let _=negate(x);let b=positive(x);assert(b||!b);}}
    } => Ok(diags) => {
        for name in ["quotient candidate","negate candidate","positive candidate"] {
            assert!(diags.notes.iter().any(|d|d.message.contains(name)&&d.message.contains("requires true;")));
        }
    }
}

test_verify_one_file! {
    #[test] composed_safety_branch_facts_do_not_escape code! {
        use vstd::prelude::*;
        #[verus_infer(precision="sound")] fn identity(x:i8)->i8 {x}
        #[verus_infer(precision="sound")]
        fn invalid(x:i8)->i8 {
            let q=identity(x);
            if q!=0 {let ignored=120/q;}
            120/q
        }
    } => Err(e) => assert_vir_error_msg(e,"cannot establish a usable input domain")
}

test_verify_one_file! {
    #[test] composed_safety_multiplies_intervals_without_small_input_guesses verus_code! {
        #[verifier::infer(sound)] fn fee(x:u8)->u8 {x/100}
        #[verifier::infer(sound)]
        #[verifier::nonlinear]
        fn product(x:u8,factor:u8)->u8 {fee(x)*factor}
        fn caller(x:u8,factor:u8) requires factor<=127,{let r=product(x,factor);assert(r<=255);}
    } => Ok(diags) => assert!(diags.notes.iter().any(|d|d.message.contains("product: soundness: PROVED")))
}

test_verify_one_file! {
    #[test] composed_safety_conditional_callee_requirements_accept_all_inputs code! {
        #[verus_infer(precision="sound")] fn debit(x:u8)->u8 {x-10}
        #[verus_infer(precision="sound")] fn guarded(x:u8)->u8 {if x>=10{debit(x)}else{0}}
        verus! {fn caller(x:u8){let r=guarded(x);assert(r<=245);}}
    } => Ok(diags) => assert!(diags.notes.iter().any(|d|d.message.contains("guarded candidate")&&d.message.contains("requires true;")))
}

#[test]
fn composed_safety_cached_domains_are_reproved_before_publication() {
    let crates = InferCrates::new();
    let source = r#"
        #[verifier::infer(sound)] pub fn leaf(b:bool)->i8 {if b{-12}else{12}}
        #[verifier::infer(sound)] pub fn offset(b:bool,delta:i8)->i8 {leaf(b)+delta}
    "#;
    let options = ["--num-threads", "4"];
    for _ in 0..2 {
        let run = crates.build("composed_safety", source, &[], &options);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    }
    let mut poisoned = 0;
    for entry in std::fs::read_dir(crates.directory.path().join("candidate-cache")).unwrap() {
        let path = entry.unwrap().path();
        let mut candidate: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        if !candidate["dependencies"].as_array().unwrap().is_empty() {
            candidate["pre"] = serde_json::json!({"Bool":true});
            std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
            poisoned += 1;
        }
    }
    assert_eq!(poisoned, 1);
    let run = crates.build("composed_safety", source, &[], &options);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(!run.status.success(), "{}", stderr);
    assert!(stderr.contains("offset: soundness: NOT PROVED"), "{}", stderr);
    assert!(stderr.contains("candidates were not published"), "{}", stderr);
    assert!(!crates.vir("composed_safety").exists());
}

#[test]
fn composed_safety_budget_failure_reports_incomplete_and_removes_stale_export() {
    let crates = InferCrates::new();
    let head = r#"
        #[verifier::infer(sound)] fn anchor(b:bool)->bool {b}
        #[verifier::infer(sound)] pub fn compute(x:u64,b:bool)->u64 {let ignored=anchor(b);
    "#;
    crates.succeeds("safety_budget", &format!("{head} x }}"), &[]);
    assert!(crates.vir("safety_budget").exists());
    let mut expanded = format!("{head} let a0=x;");
    for i in 1..=16 {
        expanded.push_str(&format!("let a{i}=a{}*a{};", i - 1, i - 1));
    }
    expanded.push_str("a16 }");
    let run = crates.build("safety_budget", &expanded, &[], &[]);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(!run.status.success(), "{}", stderr);
    assert!(
        stderr.contains("Verus Infer incomplete") && stderr.contains("budget exceeded"),
        "{}",
        stderr
    );
    assert!(!stderr.contains("contracts available"), "{}", stderr);
    assert!(!crates.vir("safety_budget").exists());
}
