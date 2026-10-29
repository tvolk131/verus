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
    #[test] isize_increment_has_verified_contract code! {
        #[verus_infer]
        fn f(x: isize) -> isize { x + 1 }
        verus! {fn caller(){let r=f(isize::MAX-1);assert(r==isize::MAX);}}
    } => Ok(())
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

#[test]
fn private_inferred_helpers_mark_exports_and_remove_stale_ordinary_outputs() {
    let crates = InferCrates::new();
    let source = "#[verifier::infer] fn helper(x:u8)->u8{x+1} pub fn property(x:u8)->(r:u8) requires x<255, ensures r as int==x as int+1, {helper(x)}";
    crates.succeeds("private_only", source, &[]);
    assert!(std::fs::read(crates.vir("private_only")).unwrap().ends_with(b"VRINFER1"));
    let run = crates.build("private_only", &source.replace("x as int+1", "x as int+2"), &[], &[]);
    assert!(!run.status.success());
    assert!(!crates.vir("private_only").exists());
    crates.succeeds("private_only", source, &[]);
    let run = crates.build("private_only", "pub fn broken( {", &[], &[]);
    assert!(!run.status.success());
    assert!(!crates.vir("private_only").exists());
    // Upgrade an unmarked ordinary artifact, then fail during generation.
    crates.succeeds("private_only", "pub fn property(x:u8)->u8{x}", &[]);
    assert!(!std::fs::read(crates.vir("private_only")).unwrap().ends_with(b"VRINFER1"));
    let unsupported = "#[verifier::infer] fn helper(x:u8)->u8{loop{}} pub fn property(x:u8)->u8{helper(x)}";
    let run = crates.build("private_only", unsupported, &[], &[]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("Verus Infer unsupported"));
    assert!(!crates.vir("private_only").exists());
}

#[test]
fn collection_views_and_effects_cross_crates_preserve_honest_precision() {
    let crates = InferCrates::new();
    let base = r#"
        use vstd::prelude::*;
        #[verifier::infer] pub fn length<T>(v:&Vec<T>)->usize {v.len()}
        #[verifier::infer] pub fn read<T:Copy>(v:&Vec<T>,i:usize)->T {v[i]}
        #[verifier::infer_precision_rlimit(1)]
        #[verifier::infer(sound)] pub fn replace<T:Copy>(v:&mut Vec<T>,i:usize,x:T){v[i]=x;}
    "#;
    let caller = r#"
        use vstd::prelude::*;
        #[verifier::infer] fn own_length<T>(v:&Vec<T>)->usize {v.len()}
        pub fn property(v:&mut Vec<u8>,i:usize,x:u8)
            requires i<old(v)@.len(),
            ensures final(v)@==old(v)@.update(i as int,x),
        {
            let n=collection_base::length(&*v); assert(n==v@.len());
            let own=own_length(&*v); assert(own==n);
            let old=collection_base::read(&*v,i); assert(old==v@[i as int]);
            collection_base::replace(v,i,x);
        }
    "#;
    for _ in 0..2 {
        let run = crates.build("collection_base", base, &[], &["--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        for f in report["functions"].as_array().unwrap() {
            assert_eq!(f["required_checks_passed"], true);
            assert_eq!(f["metrics"]["cache_policy"], "regenerated_template");
            assert_eq!(f["metrics"]["cache"], "miss");
            if f["function"].as_str().unwrap().ends_with("::replace") {
                assert_eq!(f["summary_kind"], "collection_state");
                for p in ["maximality", "exactness"] {
                    assert_eq!(f["proofs"][p]["status"], "not_established");
                    assert_eq!(f["proofs"][p]["required"], false);
                    assert_eq!(f["proofs"][p]["reason"], "state_precision_not_supported");
                }
            } else {
                for p in ["soundness", "maximality", "exactness"] {
                    assert_eq!(f["proofs"][p]["status"], "proved");
                }
            }
        }
        crates.succeeds("collection_consumer", caller, &["collection_base"]);
    }
    let wrong = caller.replace(".update(i as int,x)", ".update(i as int,x.wrapping_add(1))");
    let run = crates.build("collection_consumer", &wrong, &["collection_base"], &[]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("postcondition not satisfied"));
    assert!(!crates.vir("collection_consumer").exists());
}

#[test]
fn scalar_newtypes_cross_crate_cache_reproof_and_failed_export() {
    let crates = InferCrates::new();
    let base = r#"
        pub struct Count { pub value: u8 }
        impl Count {
            #[verifier::infer]
            pub fn increment(&self) -> Count { Count { value: self.value + 1 } }
        }
    "#;
    let middle = r#"
        #[verifier::infer]
        pub fn twice(x: &wrapper_base::Count) -> wrapper_base::Count {
            x.increment().increment()
        }
    "#;
    let app = r#"
        pub fn property(x: wrapper_base::Count) -> (r: wrapper_base::Count)
            requires x.value <= 253,
            ensures r.value as int == x.value as int + 2,
        { wrapper_middle::twice(&x) }
    "#;
    for expected_cache in ["miss", "hit"] {
        let run = crates.build("wrapper_base", base, &[], &["--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        assert_eq!(report["functions"][0]["metrics"]["cache"], expected_cache);
        assert_eq!(report["functions"][0]["required_checks_passed"], true);
    }
    let files: Vec<_> = std::fs::read_dir(crates.directory.path().join("candidate-cache")).unwrap().collect();
    assert_eq!(files.len(), 1);
    let base_cache = files.into_iter().next().unwrap().unwrap().path();
    for expected_cache in ["miss", "hit"] {
        let run = crates.build("wrapper_middle", middle, &["wrapper_base"], &["--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        let f = &report["functions"][0];
        assert_eq!(f["metrics"]["cache"], expected_cache);
        assert_eq!(f["required_checks_passed"], true);
        assert_ne!(f["result_type"], "u8");
        crates.succeeds("wrapper_app", app, &["wrapper_base", "wrapper_middle"]);
    }
    // A cache entry is only a proposal: dropping the overflow condition must
    // fail against the original wrapper body and remove its old certificate.
    let path = base_cache;
    let mut candidate: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    candidate["pre"] = serde_json::json!({"Bool": true});
    std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
    let run = crates.build("wrapper_base", base, &[], &["--output-json"]);
    assert!(!run.status.success());
    let report = json_inference(&run);
    assert_eq!(report["functions"][0]["metrics"]["cache"], "hit");
    assert_eq!(report["functions"][0]["proofs"]["soundness"]["status"], "not_proved");
    assert!(!crates.vir("wrapper_base").exists());
}

#[test]
fn scalar_newtype_definition_changes_invalidate_candidate_cache() {
    let crates = InferCrates::new();
    let source = "pub struct Count { pub value:u8 } #[verifier::infer] pub fn identity(x:Count)->Count{x}";
    for (body, expected_cache) in [(source.to_owned(), "miss"), (source.to_owned(), "hit"),
        (source.replace("value", "other"), "miss")] {
        let run = crates.build("wrapper_layout", &body, &[], &["--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        assert_eq!(report["functions"][0]["metrics"]["cache"], expected_cache);
        assert_eq!(report["functions"][0]["required_checks_passed"], true);
    }
    let record = source.replace("pub value:u8", "pub value:u8, pub extra:u8");
    let run = crates.build("wrapper_layout", &record, &[], &["--output-json"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let report = json_inference(&run);
    assert_ne!(report["functions"][0]["metrics"]["cache"], "hit");
    assert_eq!(report["functions"][0]["required_checks_passed"], true);
    let fields = (0..9).map(|i|format!("pub field{i}:u8")).collect::<Vec<_>>().join(",");
    let unsupported = source.replace("pub value:u8", &fields);
    let run = crates.build("wrapper_layout", &unsupported, &[], &["--output-json"]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("parameters and return values"));
    assert!(!crates.vir("wrapper_layout").exists());
}
#[test]
fn goal_assistance_uses_checked_imports_and_failed_goals_remove_stale_exports() {
    let crates = InferCrates::new();
    let base = "#[verifier::infer] pub fn bucket(x:u8)->u8{x & 15}";
    let run = crates.build("goals_base", base, &[], &[]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let caller = "pub fn property(x:u8){let y=goals_base::bucket(x);assert(y<16);}";
    let run = crates.build("goals_consumer", caller, &["goals_base"], &[]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("assertion failed"));
    let run = crates.build("goals_consumer", caller, &["goals_base"], &["--infer-goals"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert!(crates.vir("goals_consumer").exists());
    let wrong = caller.replace("y<16", "y==16");
    let run = crates.build("goals_consumer", &wrong, &["goals_base"], &["--infer-goals"]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("bitvector assertion not satisfied"));
    assert!(!crates.vir("goals_consumer").exists());
    let run = crates.build("goals_consumer", caller, &["goals_base"], &["--infer-goals"]);
    assert!(run.status.success());
    let run = crates.build("goals_consumer", "pub fn broken( {", &["goals_base"], &["--infer-goals"]);
    assert!(!run.status.success());
    assert!(!crates.vir("goals_consumer").exists());
}

#[test]
fn counting_assistance_uses_imported_invariants_and_removes_failed_exports() {
    let crates = InferCrates::new();
    let base = r#"
        use vstd::prelude::*;
        pub open spec fn used(s:Seq<bool>)->nat {s.count_prefix(|b:bool|b,s.len())}
        #[verifier::infer(sound)] pub fn replace(v:&mut Vec<bool>,i:usize,value:bool){v[i]=value;}
    "#;
    crates.succeeds("counting_base", base, &[]);
    let caller = r#"
        use vstd::prelude::*;
        pub fn occupy(v:&mut Vec<bool>,total:&mut usize,i:usize)
            requires i<old(v)@.len(),!old(v)@[i as int],*old(total)==counting_base::used(old(v)@),
            ensures *final(total)==counting_base::used(final(v)@),
        {let n=v.len(); counting_base::replace(v,i,true); *total+=1;}
    "#;
    let plain = crates.build("counting_app", caller, &["counting_base"], &[]);
    assert!(!plain.status.success());
    assert!(String::from_utf8_lossy(&plain.stderr).contains("possible arithmetic"));
    let run = crates.build("counting_app", caller, &["counting_base"], &["--infer-goals", "--output-json"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let report = json_inference(&run);
    assert_eq!(report["goal_assistance"].as_array().unwrap().len(), 1);
    assert_eq!(report["goal_assistance"][0]["strategy"], "counting_broadcast");
    assert_eq!(report["goal_assistance"][0]["lemmas"].as_array().unwrap().len(), 4);
    assert!(std::fs::read(crates.vir("counting_app")).unwrap().ends_with(b"VRINFER1"));
    let run = crates.build("counting_app", &caller.replace("*total+=1", "*total+=2"),
        &["counting_base"], &["--infer-goals"]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("postcondition not satisfied"));
    assert!(!crates.vir("counting_app").exists());
}

#[test]
fn record_state_summaries_cross_crates_preserve_frames_and_honest_precision() {
    let crates=InferCrates::new();
    let base=r#"
        use vstd::prelude::*;
        pub struct Pool {pub slots:Vec<bool>,pub used:usize,pub epoch:u8}
        impl Pool {
            pub open spec fn valid(&self)->bool {self.used==self.slots@.count_prefix(|b:bool|b,self.slots@.len())}
            #[verifier::infer] pub fn insert(&mut self,i:usize) {
                if i<self.slots.len() && !self.slots[i] {self.slots[i]=true;self.used+=1;}
            }
        }
    "#;
    for _ in 0..2 {
        let run=crates.build("record_base",base,&[],&["--output-json"]);
        assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
        let report=json_inference(&run);let f=&report["functions"][0];
        assert_eq!(f["summary_kind"],"record_state");
        assert_eq!(f["metrics"]["cache_policy"],"regenerated_state");
        assert_eq!(f["metrics"]["cache"],"miss");
        assert_eq!(f["proofs"]["soundness"]["status"],"proved");
        for p in ["maximality","exactness"] {
            assert_eq!(f["proofs"][p]["status"],"proved");
            assert_eq!(f["proofs"][p]["required"],true);
        }
    }
    let caller=r#"
        use vstd::prelude::*;
        pub fn check(s:&mut record_base::Pool,i:usize)
            requires old(s).valid(),
            ensures final(s).valid(),final(s).epoch==old(s).epoch,
        {s.insert(i);}
    "#;
    let run=crates.build("record_app",caller,&["record_base"],&["--infer-goals"]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let wrong=caller.replace("final(s).epoch==old(s).epoch","final(s).used==old(s).used");
    let run=crates.build("record_app",&wrong,&["record_base"],&["--infer-goals"]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("postcondition not satisfied"));
    assert!(!crates.vir("record_app").exists());
}

#[test]
fn record_precision_timeouts_block_exact_publication_but_remain_optional_in_sound_mode() {
    let crates = InferCrates::new();
    let source = r#"
        pub struct Counter { pub count:u8, pub stable:bool }
        impl Counter { #[verifier::infer] pub fn bump(&mut self) {self.count+=1;} }
    "#;
    assert!(crates.build("record_budget", source, &[], &["--output-json"]).status.success());
    assert!(crates.vir("record_budget").exists());
    let limited = source.replace("#[verifier::infer]", "#[verifier::infer_precision_rlimit(0.000001)] #[verifier::infer]");
    let run = crates.build("record_budget", &format!("{limited} fn caller(){{assert(false);}}"), &[], &["--output-json"]);
    assert!(!run.status.success());
    let report = json_inference(&run);
    assert_eq!(report["status"], "required_proof_incomplete");
    let f = &report["functions"][0];
    assert_eq!(f["proofs"]["soundness"]["status"], "proved");
    assert_eq!(f["proofs"]["exactness"]["status"], "incomplete");
    assert_eq!(f["available_to_same_crate_callers"], false);
    assert_eq!(f["exported"], false);
    assert!(!String::from_utf8_lossy(&run.stderr).contains("assertion failed"));
    assert!(!crates.vir("record_budget").exists());
    let run = crates.build("record_budget", &limited.replace("#[verifier::infer]", "#[verifier::infer(sound)]"), &[], &["--output-json"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let report = json_inference(&run);
    let f = &report["functions"][0];
    assert_eq!(f["proofs"]["soundness"]["status"], "proved");
    assert_eq!(f["proofs"]["exactness"]["status"], "incomplete");
    assert_eq!(f["proofs"]["exactness"]["required"], false);
    assert_eq!(f["exported"], true);
    assert!(crates.vir("record_budget").exists());
}

#[test]
fn generic_wrapper_interfaces_reprove_queries_updates_and_composed_callers() {
    let crates=InferCrates::new();
    let base=r#"
        use vstd::prelude::*;
        pub struct Slots<T>(pub Vec<Option<T>>);
        impl<T> Slots<T>{
            #[verifier::infer] pub fn len(&self)->usize{self.0.len()}
            #[verifier::infer] pub fn get(&self,i:usize)->Option<&T>{self.0[i].as_ref()}
            #[verifier::infer] pub fn put(&mut self,i:usize,v:T){self.0[i]=Some(v);}
        }
    "#;
    for _ in 0..2 {
        let run=crates.build("wrapper_base",base,&[],&["--output-json"]);
        assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
        let report=json_inference(&run);let fs=report["functions"].as_array().unwrap();
        assert_eq!(fs.len(),3);
        for f in fs {
            assert!(matches!(f["summary_kind"].as_str(),Some("record_view"|"record_state")),"{}",f);
            assert_eq!(f["metrics"]["cache_policy"],"regenerated_state");
            assert_eq!(f["metrics"]["cache"],"miss");
            for p in ["soundness","maximality","exactness"]{assert_eq!(f["proofs"][p]["status"],"proved","{f}");}
        }
    }
    let caller=r#"
        use vstd::prelude::*;
        pub struct Outer<T>{pub map:wrapper_base::Slots<T>,pub used:usize,pub stable:u8}
        impl<T:Copy> Outer<T>{
            pub open spec fn valid(&self)->bool{self.used==self.map.0@.count_prefix(|s:Option<T>|s.is_some(),self.map.0@.len())}
            #[verifier::infer] pub fn add(&mut self,i:usize,v:T){
                if i>=self.map.len(){return;}
                let empty=match self.map.get(i){None=>true,Some(_)=>false};
                if empty{self.map.put(i,v);self.used+=1;}
            }
        }
        pub fn check<T:Copy>(s:&mut Outer<T>,i:usize,v:T)
            requires old(s).valid(), ensures final(s).valid(),final(s).stable==old(s).stable,
        {s.add(i,v);}
    "#;
    for _ in 0..2 {
        let run=crates.build("wrapper_app",caller,&["wrapper_base"],&["--infer-goals","--output-json"]);
        assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
        assert_eq!(json_inference(&run)["functions"][0]["proofs"]["exactness"]["status"],"proved");
    }
    let wrong=base.replace("self.0[i].as_ref()","self.0[0].as_ref()");
    let run=crates.build("wrapper_base",&wrong,&[],&["--output-json"]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let run=crates.build("wrapper_app",caller,&["wrapper_base"],&["--infer-goals","--output-json"]);
    assert!(!run.status.success());
    let report=json_inference(&run);
    assert_eq!(report["status"],"caller_proof_failed");
    for p in ["soundness","maximality","exactness"]{assert_eq!(report["functions"][0]["proofs"][p]["status"],"proved");}
    assert!(!crates.vir("wrapper_app").exists());
    // A precision failure in a generic helper cannot publish or start callers.
    let limited=base.replace("#[verifier::infer] pub fn put", "#[verifier::infer_precision_rlimit(0.000001)] #[verifier::infer] pub fn put");
    let run=crates.build("wrapper_base",&format!("{limited} fn blocked(){{assert(false);}}"),&[],&["--output-json"]);
    assert!(!run.status.success());
    assert_eq!(json_inference(&run)["status"],"required_proof_incomplete");
    assert!(!String::from_utf8_lossy(&run.stderr).contains("assertion failed"));
    assert!(!crates.vir("wrapper_base").exists());
}

#[test]
fn private_sequence_view_interfaces_compose_without_exporting_the_definition() {
    let crates=InferCrates::new();
    let base=r#"
        use vstd::prelude::*;
        pub struct Slots<T>(Vec<Option<T>>);
        impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}
        impl<T> Slots<T>{
            #[verifier::infer] pub fn len(&self)->usize{self.0.len()}
            #[verifier::infer] pub fn get(&self,i:usize)->Option<&T>{self.0[i].as_ref()}
            #[verifier::infer] pub fn put(&mut self,i:usize,v:T){self.0[i]=Some(v);}
        }
    "#;
    let caller=r#"
        use vstd::prelude::*;
        pub struct Outer<T>{pub map:private_base::Slots<T>,pub sibling:private_base::Slots<T>,pub used:usize,pub stable:u8}
        impl<T:Copy> Outer<T>{
            pub open spec fn valid(&self)->bool{self.used==self.map@.count_prefix(|s:Option<T>|s.is_some(),self.map@.len())}
            #[verifier::infer] pub fn add(&mut self,i:usize,v:T){
                if i>=self.map.len(){return;}
                let empty=match self.map.get(i){None=>true,Some(_)=>false};
                if empty{self.map.put(i,v);self.used+=1;}
            }
        }
        pub fn check<T:Copy>(s:&mut Outer<T>,i:usize,v:T)
            requires old(s).valid(), ensures final(s).valid(),final(s).stable==old(s).stable,final(s).sibling@==old(s).sibling@,
        {let _=s.map.len();s.add(i,v);}
    "#;
    for _ in 0..2 {
        let run=crates.build("private_base",base,&[],&["--output-json"]);
        assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
        for f in json_inference(&run)["functions"].as_array().unwrap(){for p in ["soundness","maximality","exactness"]{assert_eq!(f["proofs"][p]["status"],"proved","{f}");}}
        let run=crates.build("private_app",caller,&["private_base"],&["--infer-goals","--output-json"]);
        assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
        for p in ["soundness","maximality","exactness"]{assert_eq!(json_inference(&run)["functions"][0]["proofs"][p]["status"],"proved");}
    }
    // The closed body supplies no automatic representation facts to clients.
    // Calling the checked len query above is what exposes this bound.
    let hidden=r#"use vstd::prelude::*; pub fn hidden(s:&private_base::Slots<u8>){ assert(s@.len()<=usize::MAX); }"#;
    let run=crates.build("private_hidden",hidden,&["private_base"],&[]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("assertion failed"));
    let raw="use vstd::prelude::*;pub fn raw(s:&private_base::Slots<u8>){let _=&s.0;}";
    let run=crates.build("private_field",raw,&["private_base"],&[]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("is private"),"{}",String::from_utf8_lossy(&run.stderr));
    // A precisely certified wrong helper must still fail application intent.
    let wrong=base.replace("self.0[i]=Some(v)","self.0[0]=Some(v)");
    let run=crates.build("private_base",&wrong,&[],&["--output-json"]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let run=crates.build("private_app",caller,&["private_base"],&["--infer-goals","--output-json"]);
    assert!(!run.status.success());
    assert_eq!(json_inference(&run)["status"],"caller_proof_failed");
    assert!(!crates.vir("private_app").exists());
    // Changing the supplied view to a lossy one removes structural eligibility.
    let lossy=base.replace("{self.0@}","{self.0@.take(0)}");
    let run=crates.build("private_base",&lossy,&[],&["--output-json"]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let run=crates.build("private_app",caller,&["private_base"],&["--infer-goals","--output-json"]);
    assert!(!run.status.success());
    assert!(json_inference(&run)["generation_failure"].is_string(),"{}",String::from_utf8_lossy(&run.stderr));
    assert!(!crates.vir("private_app").exists());
    // The view evidence must not survive removal of the inference certificate.
    let run=crates.build("private_base",base,&[],&[]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let mut bytes=std::fs::read(crates.vir("private_base")).unwrap();
    let start=bytes.windows(b"{\"schema\":".len()).rposition(|s|s==b"{\"schema\":").unwrap();
    bytes.truncate(start);std::fs::write(crates.vir("private_base"),bytes).unwrap();
    let run=crates.build("private_app",caller,&["private_base"],&[]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("missing or invalid inference proof metadata"));
}

#[test]
fn private_sequence_view_evidence_requires_metadata_even_without_public_inferred_methods() {
    let crates=InferCrates::new();
    let base=r#"use vstd::prelude::*;
      pub struct Buffer(Vec<u8>);
      impl View for Buffer{type V=Seq<u8>;closed spec fn view(&self)->Seq<u8>{self.0@}}
      impl Buffer{#[verifier::infer] fn put(&mut self,i:usize,v:u8){self.0[i]=v;}}
    "#;
    let run=crates.build("view_only",base,&[],&["--output-json"]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let caller="use vstd::prelude::*;pub fn unchanged(s:&view_only::Buffer) ensures s@==s@, {}";
    let run=crates.build("view_client",caller,&["view_only"],&["--infer-goals"]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let mut bytes=std::fs::read(crates.vir("view_only")).unwrap();
    let start=bytes.windows(b"{\"schema\":".len()).rposition(|s|s==b"{\"schema\":").unwrap();
    bytes.truncate(start);std::fs::write(crates.vir("view_only"),bytes).unwrap();
    let run=crates.build("view_client",caller,&["view_only"],&["--infer-goals"]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("missing or invalid inference proof metadata"));
    assert!(!crates.vir("view_client").exists());
}

#[test]
fn certified_trait_getters_cross_crates_bind_models_and_reject_failed_providers() {
    let crates=InferCrates::new();
    let base=r#"use vstd::prelude::*;
      pub trait Family{type Id:Copy;}
      pub trait Event{
       type Family:Family;
       spec fn key_value(&self)->usize;
       spec fn payload_value(&self)-><Self::Family as Family>::Id;
       fn key(&self)->(r:usize) ensures r==self.key_value();
       fn payload(&self)->(r:<Self::Family as Family>::Id) ensures r==self.payload_value();
      }
      pub struct F;impl Family for F{type Id=u64;}
      pub struct Message{pub key:usize,pub payload:u64}
      impl Event for Message{
       type Family=F;
       open spec fn key_value(&self)->usize{self.key}
       open spec fn payload_value(&self)->u64{self.payload}
       fn key(&self)->usize{self.key}
       fn payload(&self)->u64{self.payload}
      }
    "#;
    let consumer=r#"use vstd::prelude::*;use getter_base::{Family,Event};
      pub struct Table<F:Family>{pub slots:Vec<Option<F::Id>>,pub used:usize}
      impl<F:Family> Table<F>{
       pub open spec fn valid(&self)->bool{self.used==self.slots@.count_prefix(|s:Option<F::Id>|s.is_some(),self.slots@.len())}
       #[verifier::infer] pub fn add<U:Event<Family=F>>(&mut self,u:&U){
        let i=u.key();if i>=self.slots.len(){return;}
        let empty=match self.slots[i]{None=>true,Some(_)=>false};
        if empty{self.slots[i]=Some(u.payload());self.used+=1;}
       }
      }
      pub fn invariant<F:Family,U:Event<Family=F>>(t:&mut Table<F>,u:&U)
       requires old(t).valid(),ensures final(t).valid(),{t.add(u);}
    "#;
    let app=r#"use vstd::prelude::*;
      pub fn apply(t:&mut getter_consumer::Table<getter_base::F>,m:&getter_base::Message)
       requires old(t).valid(),m.key<old(t).slots@.len(),old(t).slots@[m.key as int].is_none(),
       ensures final(t).slots@[m.key as int]==Some(m.payload),
      {t.add(m);}
    "#;
    for _ in 0..2 {
        let run=crates.build("getter_base",base,&[],&["--infer-goals","--output-json","--no-cheating"]);
        assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
        let run=crates.build("getter_consumer",consumer,&["getter_base"],&["--infer-goals","--output-json","--no-cheating"]);
        assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
        for p in ["soundness","maximality","exactness"]{assert_eq!(json_inference(&run)["functions"][0]["proofs"][p]["status"],"proved");}
        let run=crates.build("getter_app",app,&["getter_base","getter_consumer"],&["--infer-goals","--no-cheating"]);
        assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    }
    // Changing the observation together with its implementation is a legitimate
    // new API. The old consumer certificate must not survive that change.
    let changed=base.replace("fn key_value(&self)->usize{self.key}","fn key_value(&self)->usize{0}")
        .replace("fn key(&self)->usize{self.key}","fn key(&self)->usize{0}");
    let run=crates.build("getter_base",&changed,&[],&["--infer-goals","--no-cheating"]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let run=crates.build("getter_app","use vstd::prelude::*;pub fn import_only(){}",&["getter_base","getter_consumer"],&["--infer-goals"]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("stale or missing inferred dependency fingerprint"),"{}",String::from_utf8_lossy(&run.stderr));
    assert!(!crates.vir("getter_app").exists());
    let run=crates.build("getter_consumer",consumer,&["getter_base"],&["--infer-goals","--output-json"]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let run=crates.build("getter_app",app,&["getter_base","getter_consumer"],&["--infer-goals"]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("postcondition not satisfied"));
    // A wrong implementation cannot publish its otherwise well-shaped contract.
    let bad=base.replace("fn key(&self)->usize{self.key}","fn key(&self)->usize{0}");
    let run=crates.build("getter_base",&bad,&[],&["--infer-goals"]);
    assert!(!run.status.success());assert!(!crates.vir("getter_base").exists());
    // A contracts-only provider must also reject verification bypasses/filters.
    for options in [&["--infer-goals","--no-verify"][..],&["--infer-goals","--verify-root"][..]]{
        let run=crates.build("getter_base",base,&[],options);
        assert!(!run.status.success());
        assert!(String::from_utf8_lossy(&run.stderr).contains("requires full-crate verification"));
        assert!(!crates.vir("getter_base").exists());
    }
    let run=crates.build("getter_base",base,&[],&["--infer-goals"]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let mut bytes=std::fs::read(crates.vir("getter_base")).unwrap();
    let start=bytes.windows(b"{\"schema\":".len()).rposition(|s|s==b"{\"schema\":").unwrap();
    bytes.truncate(start);std::fs::write(crates.vir("getter_base"),bytes).unwrap();
    let run=crates.build("getter_consumer",consumer,&["getter_base"],&["--infer-goals"]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("missing or invalid inference proof metadata"));
    // Ordinary legacy interfaces cannot supply compiler-certified getter facts.
    let run=crates.build("getter_base",base,&[],&[]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let run=crates.build("getter_consumer",consumer,&["getter_base"],&["--infer-goals"]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("trait inference requires a certified precondition-free getter"));
    assert!(!crates.vir("getter_consumer").exists());
}

#[test]
fn record_precision_covers_all_integer_widths_and_both_pointer_models() {
    let crates = InferCrates::new();
    let mut source = String::new();
    for (i, typ) in ["u8","u16","u32","u64","u128","usize","i8","i16","i32","i64","i128","isize"].iter().enumerate() {
        source.push_str(&format!("pub struct State{i} {{pub count:{typ},pub stable:bool}} impl State{i} {{ #[verifier::infer] pub fn update(&mut self,skip:bool){{if skip{{return;}} self.count+=1;}} }}"));
    }
    for bytes in [4,8] {
        let run = crates.verify("record_widths", &format!("global size_of usize == {bytes};{source}"), &[], &["--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        let functions = report["functions"].as_array().unwrap();
        assert_eq!(functions.len(),12);
        for f in functions { for p in ["soundness","maximality","exactness"] {
            assert_eq!(f["proofs"][p]["status"], "proved", "{f}");
            assert_eq!(f["proofs"][p]["required"], true);
        }}
    }
}

#[test]
fn wrapper_trait_certificates_compose_across_crates_and_reject_changed_boundaries() {
    let crates=InferCrates::new();
    let provider=r#"use vstd::prelude::*;use vstd::std_specs::{ops::*,convert::*};
      #[derive(Copy,Clone)] pub struct Count(pub usize);
      impl From<usize> for Count{#[verifier::infer] fn from(n:usize)->Self{Count(n)}}
      impl FromSpecImpl<usize> for Count{
       open spec fn obeys_from_spec()->bool{true}
       open spec fn from_spec(n:usize)->Self{Count(n)}
      }
      impl core::ops::AddAssign for Count{
       #[verifier::infer] fn add_assign(&mut self,rhs:Self){self.0+=rhs.0;}
      }
      impl AddAssignSpecImpl for Count{
       open spec fn obeys_add_assign_spec()->bool{true}
       open spec fn add_assign_req(&self,rhs:Self)->bool{self.0 as int+rhs.0 as int<=usize::MAX as int}
       open spec fn add_assign_spec(&self,rhs:Self)->&Self{&Count((self.0+rhs.0) as usize)}
      }
    "#;
    let consumer=r#"use vstd::prelude::*;use wrapper_base::Count;
      pub struct State{pub count:Count,pub epoch:u64}
      impl State{#[verifier::infer] pub fn bump(&mut self){self.count+=Count(1);}}
      pub struct Slots<T>(Vec<Option<T>>);
      impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}
      impl<T> Slots<T>{#[verifier::infer] pub fn size(&self)->Count{self.0.len().into()}}
    "#;
    let app=r#"use vstd::prelude::*;use wrapper_consumer::*;
      fn update(s:&mut State) requires old(s).count.0<usize::MAX,
       ensures final(s).count.0==old(s).count.0+1,final(s).epoch==old(s).epoch,{s.bump();}
      fn length<T>(s:&Slots<T>)->(r:wrapper_base::Count) ensures r.0==s@.len(),{s.size()}
    "#;
    for _ in 0..2 {
        for (name,body,deps) in [("wrapper_base",provider,&[][..]),("wrapper_consumer",consumer,&["wrapper_base"][..]),("wrapper_app",app,&["wrapper_base","wrapper_consumer"][..])] {
            let run=crates.build(name,body,deps,&["--no-cheating","--output-json"]);
            assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
            if name!="wrapper_app" {for f in json_inference(&run)["functions"].as_array().unwrap(){
                for p in ["soundness","maximality","exactness"]{assert_eq!(f["proofs"][p]["status"],"proved","{f}");}
            }}
        }
    }
    // Native impl attributes introduce specification impls. Compiled erased
    // declarations must preserve the same implementation identities on import.
    let native=r#"use vstd::prelude::*;
      #[verus_verify] #[derive(Copy,Clone)] pub struct Count(pub usize);
      #[verus_infer] impl From<usize> for Count{fn from(n:usize)->Self{Count(n)}}
      #[verus_infer] impl core::ops::AddAssign for Count{fn add_assign(&mut self,rhs:Self){self.0+=rhs.0;}}
    "#;
    let run=crates.build_source("native_wrapper_base",native,&[],&["--no-cheating"],true);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let run=crates.build("native_wrapper_consumer",&consumer.replace("wrapper_base","native_wrapper_base"),&["native_wrapper_base"],&["--no-cheating"]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let run=crates.build("native_wrapper_app",&app.replace("wrapper_base","native_wrapper_base").replace("wrapper_consumer","native_wrapper_consumer"),&["native_wrapper_base","native_wrapper_consumer"],&["--no-cheating"]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    // A legitimate changed operator gets new evidence; old consumers cannot
    // silently retain the old arithmetic interpretation.
    let changed=provider.replace("self.0+=rhs.0;","self.0=rhs.0;")
        .replace("self.0 as int+rhs.0 as int<=usize::MAX as int}","true}")
        .replace("&Count((self.0+rhs.0) as usize)","&Count(rhs.0)");
    let run=crates.build("wrapper_base",&changed,&[],&["--no-cheating"]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let run=crates.build("wrapper_app","use vstd::prelude::*;fn import_only(){}",&["wrapper_base","wrapper_consumer"],&[]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("stale or missing inferred dependency fingerprint"),"{}",String::from_utf8_lossy(&run.stderr));
    let run=crates.build("wrapper_consumer",consumer,&["wrapper_base"],&["--no-cheating"]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let run=crates.build("wrapper_app",app,&["wrapper_base","wrapper_consumer"],&["--no-cheating"]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("postcondition not satisfied"));
    // A sound but restrictive inherited operator domain fails maximality and
    // cannot publish an inferred interface.
    let restrictive=provider.replace("<=usize::MAX as int}","<=usize::MAX as int/2}");
    let run=crates.build("wrapper_base",&restrictive,&[],&["--output-json","--no-cheating"]);
    assert!(!run.status.success());assert!(!crates.vir("wrapper_base").exists());
    assert!(String::from_utf8_lossy(&run.stderr).contains("inherited trait precondition must equal"));
    let report=json_inference(&run);
    let op=report["functions"].as_array().unwrap().iter().find(|f|f["function"].as_str().unwrap().ends_with("::add_assign")).unwrap();
    assert_eq!(op["proofs"]["soundness"]["status"],"proved");
    assert_eq!(op["required_checks_passed"],false);
    // Missing proof evidence also rejects a syntactically valid trait interface.
    let run=crates.build("wrapper_base",provider,&[],&["--no-cheating"]);
    assert!(run.status.success());
    let mut bytes=std::fs::read(crates.vir("wrapper_base")).unwrap();
    let start=bytes.windows(b"{\"schema\":".len()).rposition(|s|s==b"{\"schema\":").unwrap();
    bytes.truncate(start);std::fs::write(crates.vir("wrapper_base"),bytes).unwrap();
    let run=crates.build("wrapper_consumer",consumer,&["wrapper_base"],&[]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("missing or invalid inference proof metadata"));
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
        self.build_mode(name, body, dependencies, options, true)
    }
    // These model-only tests skip erased native-code checks: global size
    // assertions would otherwise reject the other pointer width on this host.
    fn verify(
        &self,
        name: &str,
        body: &str,
        dependencies: &[&str],
        options: &[&str],
    ) -> std::process::Output {
        self.build_mode(name, body, dependencies, options, false)
    }
    fn build_mode(
        &self,
        name: &str,
        body: &str,
        dependencies: &[&str],
        options: &[&str],
        compile: bool,
    ) -> std::process::Output {
        self.build_source(name,&format!("use verus_builtin::*; use verus_builtin_macros::*; verus! {{ {body} }}"),dependencies,options,compile)
    }
    fn build_source(&self,name:&str,body:&str,dependencies:&[&str],options:&[&str],compile:bool)->std::process::Output {
        let source = self.directory.path().join(format!("{name}.rs"));
        std::fs::write(&source,body).unwrap();
        let mut command = std::process::Command::new(
            self.profile.join(format!("rust_verify{}", std::env::consts::EXE_SUFFIX)),
        );
        command
            .args([
                "--internal-test-mode",
                "--crate-type=lib",
                "--edition=2021",
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
        if compile {
            command.arg("--compile");
        } else {
            command.arg("--no-erasure-check");
        }
        if body.contains("vstd::") || body.contains("global size_of") {
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
    #[test] match_guards_are_supported code! {
        use vstd::prelude::*;
        #[verus_infer] fn f(x: Option<u8>) -> u8 { match x { Some(v) if v > 0 => v, _ => 0 } }
        verus! {fn caller(){let r=f(Some(7));assert(r==7);let r=f(None);assert(r==0);}}
    } => Ok(())
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
        enum Option { Some(u8), None, Other }
        #[verifier::infer]
        fn f(x: u8) -> Option { if x==0 {Option::Other}else{Option::Some(x)} }
        fn goal()->(r:Option) ensures r==Option::Other,{f(0)}
    } => Ok(())
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
    } => Err(e) => assert_vir_error_msg(e, "as do arrays")
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
    #[test] array_return_values_are_supported code! {
        use vstd::prelude::*;
        #[verus_infer] fn identity(values: [u8; 2]) -> [u8; 2] { values }
        verus! {fn caller(){let r=identity([1,255]);assert(r[0]==1 && r[1]==255);}}
    } => Ok(())
}

test_verify_one_file! {
    #[test] shared_slice_first_has_exact_contract code! {
        use vstd::prelude::*;
        #[verus_infer] fn first(values: &[u8]) -> u8 { values[0] }
        verus! { fn caller() { let r=first(&[9, 12]); assert(r==9); } }
    } => Ok(diags) => {
        for label in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
            assert!(diags.notes.iter().any(|d| d.message.contains(label)));
        }
    }
}

test_verify_one_file! {
    #[test] nested_arrays_are_explicitly_unsupported code! {
        use vstd::prelude::*;
        #[verus_infer] fn first(values: [[u8; 2]; 2]) -> u8 { values[0][0] }
    } => Err(e) => assert_vir_error_msg(e, "parameters and return values must be")
}

test_verify_one_file! {
    #[test] array_indices_support_checked_usize_arithmetic code! {
        use vstd::prelude::*;
        #[verus_infer] fn shifted(values: [u8; 2], index: usize) -> u8 { values[index + 1] }
        verus! { fn caller() { let r=shifted([4,9],0); assert(r==9); } }
    } => Ok(())
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
    } => Err(e) => assert_vir_error_msg(e, "as do arrays")
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
    #[test] question_mark_supports_lossless_error_conversions code! {
        use vstd::prelude::*;
        #[verus_infer] fn convert(input: Result<u8, u8>) -> Result<u8, u16> { Ok(input?) }
        verus! {fn caller(){let r=convert(Err(255));assert(r==Err(255u16));let r=convert(Ok(7));assert(r==Ok(7));}}
    } => Ok(())
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

fn json_inference(run: &std::process::Output) -> serde_json::Value {
    let output: serde_json::Value = serde_json::from_slice(&run.stdout).unwrap_or_else(|e| {
        panic!(
            "invalid JSON: {e}: {}\n{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        )
    });
    assert_eq!(output["verus-infer"]["schema_version"], 1);
    output["verus-infer"].clone()
}

#[test]
fn structured_report_cold_and_warm_candidates_have_fresh_proofs() {
    let crates = InferCrates::new();
    let source = "#[verifier::infer] pub fn increment(x:u8)->u8{x+1} fn caller(){let r=increment(2);assert(r==3);}";
    let mut first_contract = None;
    for cache in ["miss", "hit"] {
        let run = crates.build("json_cache", source, &[], &["--output-json", "--num-threads", "4"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        assert_eq!(report["status"], "verified");
        assert_eq!(report["requested_functions"], 1);
        assert_eq!(report["interface_written"], true);
        let f = &report["functions"][0];
        assert_eq!(f["metrics"]["cache"], cache);
        assert_eq!(f["required_checks_passed"], true);
        assert_eq!(f["exported"], true);
        assert_eq!(f["contract"]["requires"], "(x <= 254)");
        assert_eq!(f["contract"]["ensures"], "result == (x + 1)");
        for obligation in ["soundness", "maximality", "exactness"] {
            assert_eq!(f["proofs"][obligation]["status"], "proved");
        }
        assert_eq!(f["candidate_id"].as_str().unwrap().len(), 64);
        if let Some(contract) = first_contract {
            assert_eq!(f["contract"], contract);
        }
        first_contract = Some(f["contract"].clone());
    }
}

#[test]
fn structured_report_optional_timeouts_and_skipped_precision_are_honest() {
    let crates = InferCrates::new();
    let source = r#"
        #[verifier::infer(sound)] #[verifier::infer_precision_rlimit(0.000001)]
        pub fn leaf(b:bool)->u8 {if b{1}else{2}}
        #[verifier::infer(sound)] pub fn caller(b:bool)->u8 {leaf(b)+1}
    "#;
    let run = crates.build("json_optional", source, &[], &["--output-json"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let report = json_inference(&run);
    assert_eq!(report["status"], "verified");
    let functions = report["functions"].as_array().unwrap();
    let leaf =
        functions.iter().find(|f| f["function"].as_str().unwrap().ends_with("::leaf")).unwrap();
    assert_eq!(leaf["proofs"]["exactness"]["status"], "incomplete");
    assert_eq!(leaf["proofs"]["exactness"]["reason"], "resource_limit");
    let caller =
        functions.iter().find(|f| f["function"].as_str().unwrap().ends_with("::caller")).unwrap();
    assert_eq!(caller["proofs"]["maximality"]["status"], "not_established");
    assert_eq!(caller["proofs"]["maximality"]["reason"], "sound_mode_dependency");
    assert_eq!(caller["dependencies"][0]["candidate_id"], leaf["candidate_id"]);
    for f in functions {
        assert_eq!(f["proofs"]["soundness"]["status"], "proved");
    }
}

#[test]
fn structured_report_caller_failure_blocks_export_after_valid_inference() {
    let crates = InferCrates::new();
    let run = crates.build("json_caller", "#[verifier::infer] pub fn identity(x:u8)->u8{x} fn wrong(){let r=identity(1);assert(r==2);}", &[], &["--output-json"]);
    assert!(!run.status.success());
    let report = json_inference(&run);
    assert_eq!(report["status"], "caller_proof_failed");
    assert_eq!(report["interface_written"], false);
    assert_eq!(report["functions"][0]["required_checks_passed"], true);
    assert_eq!(report["functions"][0]["available_to_same_crate_callers"], true);
    assert_eq!(report["functions"][0]["exported"], false);
    assert!(!crates.vir("json_caller").exists());
}

#[test]
fn structured_report_unsupported_and_required_timeout_do_not_publish() {
    let crates = InferCrates::new();
    for (body, expected) in [
        ("#[verifier::infer] pub fn looping(x:u8)->u8{loop{}}", "unsupported"),
        (
            "#[verifier::infer] #[verifier::infer_precision_rlimit(0.000001)] pub fn select(b:bool)->u8{if b{1}else{2}}",
            "required_proof_incomplete",
        ),
    ] {
        let run = crates.build("json_failure", body, &[], &["--output-json"]);
        assert!(!run.status.success());
        let report = json_inference(&run);
        assert_eq!(report["status"], expected, "{report}");
        assert_eq!(report["interface_written"], false);
        for f in report["functions"].as_array().unwrap() {
            assert_eq!(f["available_to_same_crate_callers"], false);
            assert_eq!(f["exported"], false);
        }
    }
}

#[test]
fn structured_report_imported_identity_and_model_fingerprints() {
    let crates = InferCrates::new();
    let base = crates.build(
        "json_base",
        "#[verifier::infer] pub fn base(x:u8)->u8{x+1}",
        &[],
        &["--output-json"],
    );
    assert!(base.status.success());
    let base = json_inference(&base);
    let run = crates.build(
        "json_middle",
        "#[verifier::infer] pub fn next(x:u8)->u8{json_base::base(x)+1}",
        &["json_base"],
        &["--output-json"],
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let report = json_inference(&run);
    assert_eq!(report["imported_inference_dependencies"].as_array().unwrap().len(), 1);
    assert_eq!(
        report["functions"][0]["dependencies"][0]["candidate_id"],
        base["functions"][0]["candidate_id"]
    );
    let run = crates.build("json_model", "use vstd::prelude::*; #[verifier::infer] pub fn checked(x:u8)->Option<u8>{x.checked_add(1)}", &[], &["--output-json"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let report = json_inference(&run);
    assert!(!report["trusted_models"].as_array().unwrap().is_empty());
    assert!(report["trusted_models"][0]["fingerprint"].as_str().unwrap().len() == 64);
}

#[test]
fn structured_report_poisoned_cache_is_reproved_and_stale_export_removed() {
    let crates = InferCrates::new();
    let body = "#[verifier::infer] pub fn increment(x:u8)->u8{x+1}";
    let run = crates.build("json_poison", body, &[], &["--output-json"]);
    assert!(run.status.success());
    let original = json_inference(&run);
    let cache = crates.directory.path().join("candidate-cache");
    for entry in std::fs::read_dir(cache).unwrap() {
        let path = entry.unwrap().path();
        let mut candidate: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        candidate["pre"] = serde_json::json!({"Bool": true});
        std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
    }
    let run = crates.build("json_poison", body, &[], &["--output-json"]);
    assert!(!run.status.success());
    let report = json_inference(&run);
    assert_eq!(report["status"], "required_proof_failed");
    let f = &report["functions"][0];
    assert_eq!(f["metrics"]["cache"], "hit");
    assert_eq!(f["proofs"]["soundness"]["status"], "not_proved");
    assert_eq!(f["required_checks_passed"], false);
    assert_eq!(f["available_to_same_crate_callers"], false);
    assert_eq!(f["exported"], false);
    assert_ne!(f["candidate_id"], original["functions"][0]["candidate_id"]);
    assert!(!crates.vir("json_poison").exists());
}

test_verify_one_file! {
    #[test] slices_guarded_indexing_length_and_composition code! {
        use vstd::prelude::*;
        #[verus_infer] fn at(xs:&[u8], i:usize)->u8 {xs[i]}
        #[verus_infer] fn get(xs:&[u8], i:usize)->Option<u8> {
            let n=xs.len(); if i<n {Some(at(xs,i))} else {None}
        }
        #[verus_infer] fn checked(xs:&[u8], i:usize, extra:u8)->Option<u8> {
            Some(get(xs,i)?.checked_add(extra)?)
        }
        #[verus_infer] fn uniform(x:u8, i:usize)->Option<u8> {get(&[x;3],i)}
        #[verus_infer] fn length_byte(xs:&[u8])->u8 {xs.len() as u8}
        verus! {fn caller(){
            let r=get(&[],0);assert(r==None);
            let r=get(&[4,8],1);assert(r==Some(8u8));
            let r=get(&[4,8],2);assert(r==None);
            let r=checked(&[250],0,5);assert(r==Some(255u8));
            let r=checked(&[250],0,6);assert(r==None);
            let r=uniform(9,2);assert(r==Some(9u8));
            let r=uniform(9,3);assert(r==None);
            assert(300usize as u8 == 44) by(bit_vector);
            let r=length_byte(&[0u8;300]);assert(r==44);
        }}
    } => Ok(diags) => {
        for label in ["soundness: PROVED", "safe-domain maximality: PROVED", "output exactness: PROVED"] {
            assert_eq!(diags.notes.iter().filter(|d|d.message.contains(label)).count(),5,"{label}: {:?}",diags.notes);
        }
    }
}

test_verify_one_file! {
    #[test] slices_signed_reads_branches_and_short_circuit code! {
        use vstd::prelude::*;
        #[verus_infer] fn negate(xs:&[i8],i:usize)->i8 {-xs[i]}
        #[verus_infer] fn positive(xs:&[i8],i:usize)->bool {i<xs.len()&&xs[i]>0}
        #[verus_infer] fn selected(a:&[i8],b:&[i8],pick:bool,i:usize)->i8 {
            let xs=if pick {a}else{b};let alias=&*xs; alias[i]
        }
        #[verus_infer] fn same_length(a:&[bool],b:&[bool])->bool {a.len()==b.len()}
        verus! {fn caller(){
            let r=negate(&[-127i8],0);assert(r==127);
            let r=positive(&[],0);assert(!r);
            let r=positive(&[1i8],0);assert(r);
            let r=selected(&[2i8],&[-3i8,4],false,1);assert(r==4);
            let r=same_length(&[true,false],&[false,true]);assert(r);
        }}
    } => Ok(())
}

test_verify_one_file! {
    #[test] slices_out_of_bounds_caller_is_rejected code! {
        use vstd::prelude::*;
        #[verus_infer] fn at(xs:&[u8],i:usize)->u8 {xs[i]}
        verus! {fn caller(){let _=at(&[4],1); /* FAILS */}}
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] slices_overflowing_element_operation_is_rejected code! {
        use vstd::prelude::*;
        #[verus_infer] fn increment(xs:&[u8],i:usize)->u8 {xs[i]+1}
        verus! {fn caller(){let _=increment(&[255],0); /* FAILS */}}
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] slices_length_relations_early_returns_and_boolean_elements code! {
        use vstd::prelude::*;
        #[verus_infer] fn zipped(xs:&[u8],ys:&[u8],i:usize)->Result<u8,u8> {
            if xs.len()!=ys.len(){return Err(1);}
            if i>=xs.len(){return Err(2);}
            Ok(xs[i]+ys[i])
        }
        #[verus_infer] fn flag(xs:&[bool],i:usize)->Option<bool> {
            if i<xs.len() {Some(xs[i])}else{None}
        }
        verus! {fn caller(){
            let r=zipped(&[1,2],&[3,4],1);assert(r==Ok::<u8,u8>(6));
            let r=zipped(&[255],&[],0);assert(r==Err::<u8,u8>(1));
            let r=zipped(&[],&[],0);assert(r==Err::<u8,u8>(2));
            let r=flag(&[true,false],1);assert(r==Some(false));
            let r=flag(&[],usize::MAX);assert(r==None);
        }}
    } => Ok(())
}

test_verify_one_file! {
    #[test] slices_array_coercions_preserve_eager_operand_safety code! {
        use vstd::prelude::*;
        #[verus_infer] fn get(xs:&[u8],i:usize)->Option<u8> {
            if i<xs.len(){Some(xs[i])}else{None}
        }
        #[verus_infer] fn empty(x:u8)->Option<u8> {get(&[x+1;0],0)}
        verus!{fn caller(){let _=empty(255); /* FAILS */}}
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] slices_sound_mode_is_explicitly_unsupported code! {
        use vstd::prelude::*;
        #[verus_infer(precision="sound")] fn at(xs:&[u8])->u8 {xs[0]}
    } => Err(e) => assert_vir_error_msg(e,"as do arrays and shared slices")
}

test_verify_one_file! {
    #[test] slices_mutable_references_are_unsupported code! {
        use vstd::prelude::*;
        #[verus_infer] fn at(xs:&mut[u8])->u8 {xs[0]}
    } => Err(e) => assert_vir_error_msg(e,"parameters and return values must be")
}

test_verify_one_file! {
    #[test] slices_nested_elements_are_unsupported code! {
        use vstd::prelude::*;
        #[verus_infer] fn at(xs:&[&[u8]])->u8 {xs[0][0]}
    } => Err(e) => assert_vir_error_msg(e,"parameters and return values must be")
}

test_verify_one_file! {
    #[test] slices_returns_are_unsupported code! {
        use vstd::prelude::*;
        #[verus_infer] fn identity(xs:&[u8])->&[u8] {xs}
    } => Err(e) => assert_vir_error_msg(e,"slice returns are also unsupported")
}

test_verify_one_file! {
    #[test] slices_reslicing_is_supported code! {
        use vstd::prelude::*;
        #[verus_infer] fn subslice(xs:&[u8],i:usize)->u8 {let part=&xs[i..];part[0]}
        verus!{fn caller(){let r=subslice(&[1,7,255],2);assert(r==255);}}
    } => Ok(())
}

test_verify_one_file! {
    #[test] slices_different_array_coercion_branches_are_explicitly_unsupported code! {
        use vstd::prelude::*;
        #[verus_infer] fn selected(x:u8,b:bool,i:usize)->u8 {
            let xs:&[u8]=if b {&[x;1]}else{&[x;2]};xs[i]
        }
    } => Err(e) => assert_vir_error_msg(e,"invalid summary conditional")
}

const SLICE_BASE: &str = r#"
    use vstd::prelude::*;
    #[verifier::infer] pub fn at(xs:&[i8],i:usize)->i8 {xs[i]}
    #[verifier::infer] pub fn get(xs:&[i8],i:usize)->Result<i8,u8> {
        if i<xs.len(){Ok(at(xs,i))}else{Err(7)}
    }
"#;
const SLICE_MIDDLE: &str = r#"
    use vstd::prelude::*;
    #[verifier::infer] pub fn increment(xs:&[i8],i:usize)->Result<i8,u8> {
        Ok(slice_base::get(xs,i)?+1)
    }
"#;

#[test]
fn slices_cross_crate_caches_reports_and_transitive_model_provenance() {
    let crates = InferCrates::new();
    let vstd_path = crates.directory.path().join("vstd.vir");
    std::fs::copy(crates.profile.join("vstd.vir"), &vstd_path).unwrap();
    for (name, body, deps) in
        [("slice_base", SLICE_BASE, vec![]), ("slice_middle", SLICE_MIDDLE, vec!["slice_base"])]
    {
        for cache in ["miss", "hit"] {
            let run = crates.build(name, body, &deps, &["--output-json", "--num-threads", "4"]);
            assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
            let report = json_inference(&run);
            for f in report["functions"].as_array().unwrap() {
                assert_eq!(f["metrics"]["cache"], cache);
                assert_eq!(f["exported"], true);
                for o in ["soundness", "maximality", "exactness"] {
                    assert_eq!(f["proofs"][o]["status"], "proved");
                }
            }
            for model in ["spec_slice_len", "spec_slice_index"] {
                assert!(
                    report["trusted_models"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|m| m["function"].as_str().unwrap().ends_with(model))
                );
            }
        }
    }
    crates.succeeds("slice_app",r#"
        use vstd::prelude::*;
        #[verifier::infer] pub fn top(xs:&[i8],i:usize)->Result<i8,u8> {slice_middle::increment(xs,i)}
        pub fn caller(){
            let r=top(&[-128,126],1);assert(r==Ok::<i8,u8>(127));
            let r=top(&[],0);assert(r==Err::<i8,u8>(7));
            let r=top(&[1i8;300],299);assert(r==Ok::<i8,u8>(2));
        }
    "#,&["slice_base","slice_middle"]);
    let mut changed = std::fs::read(&vstd_path).unwrap();
    changed.push(b' ');
    std::fs::write(&vstd_path, changed).unwrap();
    let run = crates.build("slice_middle", SLICE_MIDDLE, &["slice_base"], &["--output-json"]);
    assert!(!run.status.success());
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("stale or missing trusted-model fingerprint")
    );
    assert_eq!(json_inference(&run)["interface_written"], false);
    assert!(!crates.vir("slice_middle").exists());
}

#[test]
fn slices_poisoned_cache_bounds_and_outputs_are_independently_rejected() {
    for poison in ["permissive", "restrictive", "wrong_index"] {
        let crates = InferCrates::new();
        let source =
            r#"use vstd::prelude::*;#[verifier::infer] pub fn at(xs:&[u8],i:usize)->u8{xs[i]}"#;
        let run = crates.build("slice_cache", source, &[], &["--output-json"]);
        assert!(run.status.success());
        for file in std::fs::read_dir(crates.directory.path().join("candidate-cache")).unwrap() {
            let path = file.unwrap().path();
            let mut candidate: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            match poison {
                "permissive" => candidate["pre"] = serde_json::json!({"Bool":true}),
                "restrictive" => candidate["pre"] = serde_json::json!({"Logical":[
                    vir::ast::LogicalOp::And, candidate["pre"].clone(),
                    {"Binary":[vir::ast::BinaryOp::Inequality(vir::ast::InequalityOp::Le),
                        {"SliceLen":{"Param":0}}, vir::ast::Constant::Int(64.into())]}
                ]}),
                _ => {
                    candidate["post"]["Binary"][2]["Binary"][2] =
                        serde_json::to_value(&vir::ast::Constant::Int(0.into())).unwrap()
                }
            }
            std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
        }
        let run = crates.build("slice_cache", source, &[], &["--output-json"]);
        assert!(!run.status.success(), "{poison}: {}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        assert_eq!(report["status"], "required_proof_failed", "{poison}: {report}");
        let f = &report["functions"][0];
        assert_eq!(f["metrics"]["cache"], "hit");
        assert_eq!(f["required_checks_passed"], false);
        assert_eq!(f["exported"], false);
        if poison == "restrictive" {
            assert_eq!(f["proofs"]["soundness"]["status"], "proved");
            assert_eq!(f["proofs"]["maximality"]["status"], "not_proved");
        } else {
            assert_eq!(f["proofs"]["soundness"]["status"], "not_proved");
        }
        assert!(!crates.vir("slice_cache").exists());
    }
}

test_verify_one_file! {
    #[test] counted_loops_fixed_widths_alias_names_and_composition code! {
        #[verus_infer] fn add8(x:u8,n:u8)->u8 {
            let mut r=x;let mut i=0u8;while i<n {r+=1;i+=1;}r
        }
        #[verus_infer] fn add16(x:u16,n:u16)->u16 {
            let mut i=0u16;let mut total=x;while i<n {total=total+1;i=i+1;}total
        }
        #[verus_infer] fn add32(x:u32,n:u32)->u32 {
            let mut x=x;let mut i=0u32;while i<n {x+=1;i+=1;}x
        }
        #[verus_infer] fn add64(n:u64,seed:u64)->u64 {
            let mut total=seed;let mut tick=0u64;while tick<n {total+=1;tick+=1;}total
        }
        #[verus_infer] fn add128(x:u128,n:u128)->u128 {
            let mut r=x;let mut i=0u128;while i<n {r+=1;i+=1;}r
        }
        #[verus_infer] fn composed(x:u8,n:u8)->u8 {add8(add8(x,n),n)}
        verus!{fn caller(){
            let r=add8(255,0);assert(r==255);
            let r=add8(0,255);assert(r==255);
            let r=add16(65534,1);assert(r==65535);
            let r=add32(u32::MAX,0);assert(r==u32::MAX);
            let r=add64(1,4);assert(r==5);
            let r=add128(u128::MAX-1,1);assert(r==u128::MAX);
            let r=composed(251,2);assert(r==255);
        }}
    } => Ok(diags) => {
        for label in ["soundness: PROVED","safe-domain maximality: PROVED","output exactness: PROVED"] {
            assert_eq!(diags.notes.iter().filter(|n|n.message.contains(label)).count(),6,"{label}");
        }
    }
}

test_verify_one_file! {
    #[test] counted_loops_unsafe_caller_is_rejected code! {
        #[verus_infer] fn add(x:u8,n:u8)->u8 {
            let mut r=x;let mut i=0u8;while i<n {r+=1;i+=1;}r
        }
        verus!{fn caller(){let _=add(255,1); /* FAILS */}}
    } => Err(e) => assert_one_fails(e)
}

test_verify_one_file! {
    #[test] counted_loops_constant_step_is_supported code! {
        #[verus_infer] fn add(x:u8,n:u8)->u8 {
            let mut r=x;let mut i=0u8;while i<n {r+=2;i+=1;}r
        }
        verus!{fn caller(){let r=add(1,127);assert(r==255);}}
    } => Ok(())
}

test_verify_one_file! {
    #[test] counted_loops_wrong_update_order_is_unsupported code! {
        #[verus_infer] fn add(x:u8,n:u8)->u8 {
            let mut r=x;let mut i=0u8;while i<n {i+=1;r+=1;}r
        }
    } => Err(e) => assert_vir_error_msg(e,"loops currently require one counted")
}

test_verify_one_file! {
    #[test] counted_loops_extra_body_statement_is_unsupported code! {
        #[verus_infer] fn add(x:u8,n:u8)->u8 {
            let mut r=x;let mut i=0u8;while i<n {let hidden=1u8;r+=1;i+=1;}r
        }
    } => Err(e) => assert_vir_error_msg(e,"loops currently require one counted")
}

test_verify_one_file! {
    #[test] counted_loops_handwritten_invariants_are_unsupported verus_code! {
        #[verifier::infer] fn add(x:u8,n:u8)->u8 {
            let mut r=x;let mut i=0u8;
            while i<n invariant i<=n, {r+=1;i+=1;}r
        }
    } => Err(e) => assert_vir_error_msg(e,"no handwritten loop proofs")
}

test_verify_one_file! {
    #[test] counted_loops_termination_assumptions_are_unsupported verus_code! {
        #[verifier::infer] #[verifier::assume_termination]
        fn add(x:u8,n:u8)->u8 {
            let mut r=x;let mut i=0u8;while i<n {r+=1;i+=1;}r
        }
    } => Err(e) => assert_vir_error_msg(e,"loops currently require one counted")
}

test_verify_one_file_with_options! {
    #[test] counted_loops_termination_suppression_option_is_unsupported ["exec_allows_no_decreases_clause"] => verus_code! {
        #[verifier::infer] fn add(x:u8,n:u8)->u8 {
            let mut r=x;let mut i=0u8;while i<n {r+=1;i+=1;}r
        }
    } => Err(e) => assert_vir_error_msg(e,"loops currently require one counted")
}

test_verify_one_file! {
    #[test] counted_loops_sound_mode_is_unsupported code! {
        #[verus_infer(precision="sound")] fn add(x:u8,n:u8)->u8 {
            let mut r=x;let mut i=0u8;while i<n {r+=1;i+=1;}r
        }
    } => Err(e) => assert_vir_error_msg(e,"loops currently require one counted")
}

test_verify_one_file! {
    #[test] counted_loops_user_break_is_unsupported code! {
        #[verus_infer] fn add(x:u8,n:u8)->u8 {
            let mut r=x;let mut i=0u8;while i<n {if i==1{break;}r+=1;i+=1;}r
        }
    } => Err(e) => assert_vir_error_msg(e,"loops currently require one counted")
}

test_verify_one_file! {
    #[test] counted_loops_signed_counters_are_explicitly_unsupported code! {
        #[verus_infer] fn add(x:i8,n:i8)->i8 {
            let mut r=x;let mut i=0i8;while i<n {r+=1;i+=1;}r
        }
    } => Err(e) => assert_vir_error_msg(e,"loops currently require one counted")
}

const COUNTED_LOOP_BASE: &str = r#"
    #[verifier::infer] pub fn add(x:u8,n:u8)->u8 {
        let mut r=x;let mut i=0u8;while i<n {r+=1;i+=1;}r
    }
"#;

#[test]
fn counted_loops_cache_reproves_annotations_and_cross_crate_contracts() {
    let crates = InferCrates::new();
    for cache in ["miss", "hit"] {
        let run = crates.build(
            "loop_base",
            COUNTED_LOOP_BASE,
            &[],
            &["--output-json", "--num-threads", "4"],
        );
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        let f = &report["functions"][0];
        assert_eq!(f["metrics"]["cache"], cache);
        assert_eq!(f["loop_proofs"][0]["loop_isolation"], false);
        assert_eq!(f["loop_proofs"][0]["invariants"].as_array().unwrap().len(), 2);
        assert_eq!(f["loop_proofs"][0]["decreases"][0], "(n - i)");
        for o in ["soundness", "maximality", "exactness"] {
            assert_eq!(f["proofs"][o]["status"], "proved");
        }
    }
    for _ in 0..2 {
        crates.succeeds(
            "loop_middle",
            r#"
            #[verifier::infer] pub fn twice(x:u8,n:u8)->u8 {loop_base::add(loop_base::add(x,n),n)}
        "#,
            &["loop_base"],
        );
    }
    crates.succeeds(
        "loop_app",
        r#"
        #[verifier::infer] pub fn top(x:u8,n:u8)->u8 {loop_middle::twice(x,n)}
        pub fn caller(){let r=top(253,1);assert(r==255);let r=loop_base::add(255,0);assert(r==255);}
    "#,
        &["loop_base", "loop_middle"],
    );
}

#[test]
fn counted_loops_wrong_cached_candidates_block_dependents_and_remove_exports() {
    for kind in ["restrictive", "permissive", "wrong_output"] {
        let crates = InferCrates::new();
        let body = format!("{COUNTED_LOOP_BASE} fn caller(){{let _=add(0,0);}}");
        let run = crates.build("loop_cache", &body, &[], &["--output-json"]);
        assert!(run.status.success());
        for file in std::fs::read_dir(crates.directory.path().join("candidate-cache")).unwrap() {
            let path = file.unwrap().path();
            let mut c: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            match kind {
                "restrictive" => c["pre"] = serde_json::json!({"Bool":false}),
                "permissive" => c["pre"] = serde_json::json!({"Bool":true}),
                _ => {
                    c["post"]["Binary"][2] =
                        serde_json::to_value(vir::ast::Constant::Int(0.into())).unwrap()
                }
            }
            std::fs::write(path, serde_json::to_vec(&c).unwrap()).unwrap();
        }
        let run = crates.build("loop_cache", &body, &[], &["--output-json"]);
        assert!(!run.status.success(), "{kind}: {}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        let f = &report["functions"][0];
        assert_eq!(f["metrics"]["cache"], "hit");
        assert_eq!(f["available_to_same_crate_callers"], false);
        assert_eq!(f["exported"], false);
        if kind == "restrictive" {
            assert_eq!(f["proofs"]["soundness"]["status"], "proved");
            assert_eq!(f["proofs"]["maximality"]["status"], "not_proved");
        } else {
            assert_eq!(f["proofs"]["soundness"]["status"], "not_proved");
        }
        assert!(!crates.vir("loop_cache").exists());
    }
}

#[test]
fn counted_loops_precision_timeout_blocks_dependents_and_exports() {
    let crates = InferCrates::new();
    let body = format!(
        "#[verifier::infer_precision_rlimit(0.000001)] {COUNTED_LOOP_BASE} fn caller(){{assert(false);}}"
    );
    let run = crates.build("loop_timeout", &body, &[], &["--output-json"]);
    assert!(!run.status.success());
    let report = json_inference(&run);
    assert_eq!(report["status"], "required_proof_incomplete");
    assert_eq!(report["functions"][0]["proofs"]["soundness"]["status"], "proved");
    assert!(!String::from_utf8_lossy(&run.stderr).contains("assertion failed"));
    assert!(!crates.vir("loop_timeout").exists());
}

const USIZE_OPERATIONS: &str = r#"
    use vstd::prelude::*;
    #[verifier::infer] pub fn add(x:usize,y:usize)->usize{x+y}
    #[verifier::infer] pub fn sub(x:usize,y:usize)->usize{x-y}
    #[verifier::infer] pub fn mul(x:usize,y:usize)->usize{x*y}
    #[verifier::infer] pub fn div(x:usize,y:usize)->usize{x/y}
    #[verifier::infer] pub fn rem(x:usize,y:usize)->usize{x%y}
    #[verifier::infer] pub fn cast(x:u64)->usize{x as usize}
    #[verifier::infer] pub fn signed(x:i64)->usize{x as usize}
    #[verifier::infer] pub fn narrow(x:usize)->u8{x as u8}
    #[verifier::infer] pub fn maybe(x:usize)->Option<usize>{if x<usize::MAX{Some(x+1)}else{None}}
    #[verifier::infer] pub fn propagate(x:usize)->Option<usize>{let n=maybe(x)?;Some(n)}
    #[verifier::infer] pub fn result(x:usize)->Result<usize,usize>{if x<usize::MAX{Ok(x+1)}else{Err(x)}}
    #[verifier::infer] pub fn error(x:usize)->Result<usize,usize>{let n=result(x)?;Ok(n)}
    #[verifier::infer] pub fn counted(x:usize,n:usize)->usize{
        let mut r=x;let mut i=0usize;while i<n{r+=1;i+=1;}r
    }
    #[verifier::infer] pub fn twice(x:usize,n:usize)->usize{counted(counted(x,n),n)}
"#;

#[test]
fn usize_exact_arithmetic_enum_cast_loop_boundaries_on_32_and_64_bits() {
    for bytes in [4, 8] {
        let crates = InferCrates::new();
        let expected = if bytes == 4 { 1u64 } else { 4294967297u64 };
        let body = format!(
            r#"
            global size_of usize == {bytes};
            {USIZE_OPERATIONS}
            fn caller(){{
                let r=add(usize::MAX-1,1);assert(r==usize::MAX);
                let r=sub(usize::MAX,usize::MAX);assert(r==0);
                let r=mul(usize::MAX,1);assert(r==usize::MAX);
                let r=div(usize::MAX,1);assert(r==usize::MAX);
                let r=rem(usize::MAX,2);assert(r==1);
                assert((4294967297u64 as usize)=={expected}) by(bit_vector);
                let r=cast(4294967297);assert(r=={expected});
                assert((-1i64 as usize)==usize::MAX) by(bit_vector);
                let r=signed(-1);assert(r==usize::MAX);
                assert((257usize as u8)==1) by(bit_vector);
                let r=narrow(257);assert(r==1);
                let r=maybe(usize::MAX);assert(r==None);
                let r=propagate(usize::MAX-1);assert(r==Some(usize::MAX));
                let r=error(usize::MAX);assert(r==Err(usize::MAX));
                let r=counted(0,usize::MAX);assert(r==usize::MAX);
                let r=counted(usize::MAX,0);assert(r==usize::MAX);
                let r=twice(usize::MAX-2,1);assert(r==usize::MAX);
            }}
        "#
        );
        for cache in ["miss", "hit"] {
            let run =
                crates.verify("usize_ops", &body, &[], &["--output-json", "--num-threads", "4"]);
            assert!(run.status.success(), "{bytes}: {}", String::from_utf8_lossy(&run.stderr));
            let report = json_inference(&run);
            assert_eq!(report["functions"].as_array().unwrap().len(), 14);
            for f in report["functions"].as_array().unwrap() {
                assert_eq!(f["metrics"]["cache"], cache);
                for obligation in ["soundness", "maximality", "exactness"] {
                    assert_eq!(f["proofs"][obligation]["status"], "proved");
                }
            }
        }
    }
}

test_verify_one_file! {
    #[test] usize_portable_slices_arrays_lengths_and_index_arithmetic code! {
        use vstd::prelude::*;
        #[verus_infer] fn length(values:&[usize])->usize{values.len()}
        #[verus_infer] fn next(values:&[usize],i:usize)->usize{values[i+1]}
        #[verus_infer] fn fixed(values:[usize;2])->Option<usize>{Some(values[1]+1)}
        #[verus_infer] fn bounded(x:usize)->Option<usize>{if x<usize::MAX {Some(x+1)}else{None}}
        #[verus_infer] fn repeated(x:usize,i:usize)->usize{[x+1;2][i]}
        #[verus_infer] fn literal(x:usize)->usize{[0usize,x+1][1]}
        #[verus_infer] fn bits()->u32{usize::BITS}
        verus!{fn caller(){
            let r=length(&[usize::MAX,2]);assert(r==2);
            let r=next(&[3,usize::MAX],0);assert(r==usize::MAX);
            let r=fixed([0,usize::MAX-1]);assert(r==Some(usize::MAX));
            let r=bounded(usize::MAX);assert(r==None);
            let r=repeated(usize::MAX-1,1);assert(r==usize::MAX);
            let r=literal(usize::MAX-1);assert(r==usize::MAX);
            let r=bits();assert(r==usize::BITS);
        }}
    } => Ok(diags) => {
        assert_eq!(diags.notes.iter().filter(|n|n.message.contains("output exactness: PROVED")).count(),7);
    }
}

#[test]
fn usize_unsafe_operations_fail_callers_on_both_architectures() {
    for bytes in [4, 8] {
        for (name, expression) in [
            ("add", "usize::MAX+1usize"),
            ("sub", "0usize-1usize"),
            ("mul", "usize::MAX*2usize"),
            ("div", "1usize/0usize"),
            ("rem", "1usize%0usize"),
        ] {
            // Keep arithmetic in the inferred function so Rust's constant-folding
            // lint cannot preempt the Verus precondition check.
            let op = match name {
                "add" => "+",
                "sub" => "-",
                "mul" => "*",
                "div" => "/",
                _ => "%",
            };
            let args = match name {
                "add" => "usize::MAX,1",
                "sub" => "0,1",
                "mul" => "usize::MAX,2",
                _ => "1,0",
            };
            let crates = InferCrates::new();
            let body = format!(
                "global size_of usize == {bytes}; #[verifier::infer] pub fn op(x:usize,y:usize)->usize{{x{op}y}} fn caller(){{let _=op({args});}}"
            );
            let run = crates.verify("usize_bad", &body, &[], &["--output-json"]);
            assert!(!run.status.success(), "{} {}", bytes, expression);
            assert!(
                String::from_utf8_lossy(&run.stderr).contains("precondition not satisfied"),
                "{}",
                String::from_utf8_lossy(&run.stderr)
            );
            assert_eq!(json_inference(&run)["status"], "caller_proof_failed");
            assert!(!crates.vir("usize_bad").exists());
        }
    }
}

#[test]
fn usize_cache_architecture_changes_and_cross_crate_compatibility() {
    let crates = InferCrates::new();
    let ops = r#"
        use vstd::prelude::*;
        #[verifier::infer] pub fn add(x:usize,n:usize)->usize{
            let mut r=x;let mut i=0usize;while i<n{r+=1;i+=1;}r
        }
        #[verifier::infer] pub fn cast(x:u64)->usize{x as usize}
        #[verifier::infer] pub fn wrapped(x:usize)->Option<usize>{Some(add(x,1))}
    "#;
    // Native rlib metadata describes the same executable functions; model-specific
    // VIR is exported separately. These cross-width tests never execute the rlib.
    crates.succeeds("usize_base", ops, &[]);
    for bytes in [4, 8] {
        let base = format!("global size_of usize == {bytes}; {ops}");
        for cache in ["miss", "hit"] {
            let run = crates.verify("usize_base", &base, &[], &["--output-json"]);
            assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
            for f in json_inference(&run)["functions"].as_array().unwrap() {
                assert_eq!(f["metrics"]["cache"], cache);
            }
        }
        // Architecture is inherited from the dependency and enters the cache key.
        let middle = r#"#[verifier::infer] pub fn twice(x:usize,n:usize)->usize{usize_base::add(usize_base::add(x,n),n)}"#;
        crates.succeeds("usize_middle", middle, &["usize_base"]);
        let app = format!(
            r#"global size_of usize == {bytes};
            fn caller(){{let r=usize_middle::twice(usize::MAX-2,1);assert(r==usize::MAX);
                let r=usize_base::wrapped(usize::MAX-1);assert(r==Some(usize::MAX));}}
        "#
        );
        let run = crates.verify("usize_app", &app, &["usize_base", "usize_middle"], &[]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let mismatch = format!("global size_of usize == {}; fn caller(){{}}", 12 - bytes);
        let run = crates.verify("usize_mismatch", &mismatch, &["usize_base", "usize_middle"], &[]);
        assert!(!run.status.success());
        assert!(String::from_utf8_lossy(&run.stderr).contains("compatible arch_word_bits"));
    }
    // A portable producer can be consumed at either width.
    crates.succeeds("usize_portable", ops, &[]);
    for bytes in [4, 8] {
        let run=crates.verify("usize_consumer",&format!("global size_of usize == {bytes}; fn caller(){{let r=usize_portable::add(usize::MAX,0);assert(r==usize::MAX);}}"),&["usize_portable"],&[]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    }
}

#[test]
fn usize_wrong_host_width_cached_bound_is_reproved_and_rejected() {
    let crates = InferCrates::new();
    let body = "global size_of usize == 4; #[verifier::infer] pub fn add(x:usize,y:usize)->usize{x+y} fn caller(){let _=add(0,0);}";
    let run = crates.verify("usize_poison", body, &[], &["--output-json"]);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    for entry in std::fs::read_dir(crates.directory.path().join("candidate-cache")).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("USizeMax"));
        let max = serde_json::to_string(&vir::ast::Constant::Int(u64::MAX.into())).unwrap();
        std::fs::write(path, text.replace("\"USizeMax\"", &max)).unwrap();
    }
    let run = crates.verify("usize_poison", body, &[], &["--output-json"]);
    assert!(!run.status.success());
    let report = json_inference(&run);
    let f = &report["functions"][0];
    assert_eq!(f["metrics"]["cache"], "hit");
    assert_eq!(f["proofs"]["soundness"]["status"], "not_proved");
    assert_eq!(f["available_to_same_crate_callers"], false);
    assert!(!crates.vir("usize_poison").exists());
}

test_verify_one_file! {
    #[test] usize_sound_mode_supports_scalar_signatures code! {
        #[verus_infer(precision="sound")] fn identity(x:usize)->usize{x}
        verus! {fn caller(){let r=identity(usize::MAX);assert(r<=usize::MAX);}}
    } => Ok(diags) => {
        assert!(diags.notes.iter().any(|d|d.message.contains("soundness: PROVED")));
        assert!(diags.notes.iter().any(|d|d.message.contains("exact output: NOT ESTABLISHED")));
    }
}

test_verify_one_file! {
    #[test] isize_exact_scalar_signature code! {
        #[verus_infer] fn identity(x:isize)->isize{x}
        verus! {fn caller(){let r=identity(isize::MIN);assert(r==isize::MIN);}}
    } => Ok(())
}

const PLATFORM_BASE: &str = r#"
    use vstd::prelude::*;
    #[verifier::infer] pub fn add(x:isize,y:isize)->isize{x+y}
    #[verifier::infer] pub fn sub(x:isize,y:isize)->isize{x-y}
    #[verifier::infer] pub fn mul(x:isize,y:isize)->isize{x*y}
    #[verifier::infer] pub fn div(x:isize,y:isize)->isize{x/y}
    #[verifier::infer] pub fn rem(x:isize,y:isize)->isize{x%y}
    #[verifier::infer] pub fn neg(x:isize)->isize{-x}
    #[verifier::infer] pub fn cast(x:usize)->isize{x as isize}
    #[verifier::infer] pub fn unsigned(x:isize)->usize{x as usize}
    #[verifier::infer] pub fn from_wide(x:i128)->isize{x as isize}
    #[verifier::infer] pub fn narrow(x:isize)->i8{x as i8}
    #[verifier::infer] pub fn checked(x:isize,y:isize)->Option<isize>{x.checked_add(y)}
    #[verifier::infer] pub fn checked_unsigned(x:usize,y:usize)->Option<usize>{x.checked_add(y)}
    #[verifier::infer] pub fn propagate(x:isize,y:isize)->Result<isize,isize>{
        match checked(x,y){Some(z)=>Ok(z),None=>Err(x)}
    }
    #[verifier::infer] pub fn slice(values:&[isize],i:usize)->isize{values[i]}
    #[verifier::infer(sound)] pub fn scale(x:usize)->usize{x/10}
    #[verifier::infer(sound)] pub fn signed_scale(x:isize)->isize{x/10}
    #[verifier::infer(sound)] pub fn remainder(x:isize)->isize{x%10}
"#;

#[test]
fn platform_exact_boundaries_casts_payloads_and_sound_ranges_on_both_widths() {
    for bytes in [4, 8] {
        let crates = InferCrates::new();
        let cast = if bytes == 4 { 1i128 } else { 4294967297i128 };
        let body = format!(
            r#"
            global size_of usize == {bytes}; {PLATFORM_BASE}
            fn caller(){{
                let r=add(isize::MAX-1,1);assert(r==isize::MAX);
                let r=sub(isize::MIN+1,1);assert(r==isize::MIN);
                let r=mul(isize::MIN,1);assert(r==isize::MIN);
                let r=div(-10,3);assert(r == -3);
                let r=rem(-10,3);assert(r == -1);
                let r=neg(isize::MAX);assert(r == -isize::MAX);
                assert((usize::MAX as isize)== -1) by(bit_vector);
                let r=cast(usize::MAX);assert(r == -1);
                assert((-1isize as usize)==usize::MAX) by(bit_vector);
                let r=unsigned(-1);assert(r==usize::MAX);
                assert((4294967297i128 as isize)=={cast}) by(bit_vector);
                let r=from_wide(4294967297);assert(r=={cast});
                assert((-129isize as i8)==127) by(bit_vector);
                let r=narrow(-129);assert(r==127);
                let r=checked(isize::MAX,1);assert(r==None);
                let r=checked(isize::MIN,-1);assert(r==None);
                let r=checked_unsigned(usize::MAX,1);assert(r==None);
                let r=propagate(isize::MAX,1);assert(r==Err(isize::MAX));
                let r=slice(&[isize::MIN,isize::MAX],0);assert(r==isize::MIN);
                let r=scale(usize::MAX);assert(r<=usize::MAX/10);
                let r=signed_scale(isize::MIN);assert(r>=isize::MIN/10 && r<=isize::MAX/10);
                let r=remainder(isize::MIN);assert(-9<=r && r<=9);
            }}
        "#
        );
        for cache in ["miss", "hit"] {
            let run = crates.verify("platform_bounds", &body, &[], &["--output-json"]);
            assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
            for f in json_inference(&run)["functions"].as_array().unwrap() {
                assert_eq!(f["metrics"]["cache"], cache);
                assert_eq!(f["proofs"]["soundness"]["status"], "proved");
                if f["mode"] == "exact" {
                    assert_eq!(f["proofs"]["maximality"]["status"], "proved");
                    assert_eq!(f["proofs"]["exactness"]["status"], "proved");
                }
            }
        }
    }
}

#[test]
fn platform_all_checked_methods_use_existing_models_and_reprove_cache() {
    let crates = InferCrates::new();
    let mut body = String::from("use vstd::prelude::*; ");
    for (prefix, typ) in [("u", "usize"), ("i", "isize")] {
        for method in ["add", "sub", "mul", "div", "rem", "div_euclid", "rem_euclid"] {
            body.push_str(&format!("#[verifier::infer] fn {prefix}_{method}(x:{typ},y:{typ})->Option<{typ}>{{x.checked_{method}(y)}}\n"));
        }
    }
    body.push_str(
        r#"fn caller(){
        let r=u_add(usize::MAX,1);assert(r==None);let r=u_sub(0,1);assert(r==None);
        let r=u_mul(usize::MAX,2);assert(r==None);let r=u_div(1,0);assert(r==None);
        let r=u_rem(1,0);assert(r==None);let r=u_div_euclid(10,3);assert(r==Some(3));
        let r=u_rem_euclid(10,3);assert(r==Some(1));
        let r=i_add(isize::MAX,1);assert(r==None);let r=i_sub(isize::MIN,1);assert(r==None);
        let r=i_mul(isize::MIN,-1);assert(r==None);let r=i_div(isize::MIN,-1);assert(r==None);
        let r=i_rem(isize::MIN,-1);assert(r==None);let r=i_div(-10,3);assert(r==Some(-3isize));
        let r=i_rem(-10,3);assert(r==Some(-1isize));let r=i_div_euclid(-10,-3);assert(r==Some(4));
        let r=i_rem_euclid(-10,-3);assert(r==Some(2));
    }"#,
    );
    for cache in ["miss", "hit"] {
        let run =
            crates.build("platform_checked", &body, &[], &["--output-json", "--num-threads", "4"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        assert_eq!(report["functions"].as_array().unwrap().len(), 14);
        for f in report["functions"].as_array().unwrap() {
            assert_eq!(f["metrics"]["cache"], cache);
            for proof in ["soundness", "maximality", "exactness"] {
                assert_eq!(f["proofs"][proof]["status"], "proved");
            }
            assert!(!report["trusted_models"].as_array().unwrap().is_empty());
        }
    }
}

#[test]
fn platform_invalid_isize_operations_fail_callers_and_remove_exports() {
    for bytes in [4, 8] {
        for (op, args) in [
            ("+", "isize::MAX,1"),
            ("-", "isize::MIN,1"),
            ("*", "isize::MIN,-1"),
            ("/", "isize::MIN,-1"),
            ("%", "isize::MIN,-1"),
            ("/", "1,0"),
            ("%", "1,0"),
        ] {
            let crates = InferCrates::new();
            let body = format!(
                "global size_of usize == {bytes}; use vstd::prelude::*; #[verifier::infer] fn op(x:isize,y:isize)->isize{{x{op}y}} fn caller(){{let _=op({args});}}"
            );
            let run = crates.verify("platform_bad", &body, &[], &["--output-json"]);
            assert!(!run.status.success());
            assert!(
                String::from_utf8_lossy(&run.stderr).contains("precondition not satisfied"),
                "{}",
                String::from_utf8_lossy(&run.stderr)
            );
            assert_eq!(json_inference(&run)["status"], "caller_proof_failed");
            assert!(!crates.vir("platform_bad").exists());
        }
    }
}

#[test]
fn platform_sound_composition_and_exact_calls_across_three_crates_at_both_widths() {
    let crates = InferCrates::new();
    for cache in ["miss", "hit"] {
        let run = crates.build("platform_base", PLATFORM_BASE, &[], &["--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        for f in json_inference(&run)["functions"].as_array().unwrap() {
            assert_eq!(f["metrics"]["cache"], cache);
        }
    }
    let middle = r#"
        use vstd::prelude::*;
        #[verifier::infer(sound)] pub fn shifted(x:usize)->usize{platform_base::scale(x)+1}
        #[verifier::infer(sound)] pub fn signed_shifted(x:isize)->isize{platform_base::signed_scale(x)+1}
        #[verifier::infer(sound)] pub fn guarded(x:isize)->isize{
            let r=platform_base::remainder(x);let alias=r;if alias!=0{alias+1}else{0}
        }
        #[verifier::infer] pub fn exact(x:isize,y:isize)->Option<isize>{Some(platform_base::checked(x,y)?)}
    "#;
    for cache in ["miss", "hit"] {
        let run = crates.build("platform_middle", middle, &["platform_base"], &["--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        for f in json_inference(&run)["functions"].as_array().unwrap() {
            assert_eq!(f["metrics"]["cache"], cache);
        }
    }
    for bytes in [4, 8] {
        let body = format!(
            r#"use vstd::prelude::*;global size_of usize == {bytes};fn caller(){{
            let r=platform_middle::shifted(usize::MAX);assert(1<=r && r<=usize::MAX/10+1);
            let r=platform_middle::signed_shifted(isize::MIN);assert(isize::MIN/10+1<=r && r<=isize::MAX/10+1);
            let r=platform_middle::guarded(isize::MIN);assert(-8<=r && r<=10);
            let r=platform_middle::exact(isize::MAX,1);assert(r==None);
        }}"#
        );
        let run = crates.verify("platform_app", &body, &["platform_base", "platform_middle"], &[]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    }
}

#[test]
fn platform_wrong_cached_sound_architecture_range_is_reproved() {
    let crates = InferCrates::new();
    let body = "use vstd::prelude::*;#[verifier::infer(sound)] pub fn scale(x:usize)->usize{x/10} fn caller(){let _=scale(10);}";
    let run = crates.build("platform_cache", body, &[], &["--output-json"]);
    assert!(run.status.success());
    for entry in std::fs::read_dir(crates.directory.path().join("candidate-cache")).unwrap() {
        let path = entry.unwrap().path();
        let mut c: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        // Keep a well-typed interval shape but falsely claim a constant result on 32-bit.
        assert!(c["post"]["If"].is_array());
        c["post"]["If"][1] = serde_json::json!({"Binary":[{"Eq":"Spec"},{"Param":1},serde_json::to_value(vir::ast::Constant::Int(0.into())).unwrap()]});
        std::fs::write(path, serde_json::to_vec(&c).unwrap()).unwrap();
    }
    let run = crates.build("platform_cache", body, &[], &["--output-json"]);
    assert!(!run.status.success());
    let f = &json_inference(&run)["functions"][0];
    assert_eq!(f["metrics"]["cache"], "hit");
    assert_eq!(f["proofs"]["soundness"]["status"], "not_proved");
    assert_eq!(f["available_to_same_crate_callers"], false);
    assert!(!crates.vir("platform_cache").exists());
}
#[test]
fn integer_operations_all_types_exact_proofs_and_cache_reuse() {
    let crates = InferCrates::new();
    let mut body = String::from("use vstd::prelude::*;\n");
    let mut callers = String::from("fn caller(){\n");
    for (unsigned, signed) in [
        ("u8", "i8"),
        ("u16", "i16"),
        ("u32", "i32"),
        ("u64", "i64"),
        ("u128", "i128"),
        ("usize", "isize"),
    ] {
        for typ in [unsigned, signed] {
            for (name, expr) in [("and", "x&y"), ("or", "x|y"), ("xor", "x^y")] {
                body.push_str(&format!(
                    "#[verifier::infer] fn {typ}_{name}(x:{typ},y:{typ})->{typ}{{{expr}}}\n"
                ));
            }
            body.push_str(&format!("#[verifier::infer] fn {typ}_not(x:{typ})->{typ}{{!x}}\n"));
            for (name, symbol) in [("shl", "<<"), ("shr", ">>")] {
                body.push_str(&format!(
                    "#[verifier::infer] fn {typ}_{name}(x:{typ},n:i32)->{typ}{{x{symbol}n}}\n"
                ));
            }
            for policy in ["wrapping", "saturating"] {
                for operation in ["add", "sub", "mul"] {
                    body.push_str(&format!("#[verifier::infer] fn {typ}_{policy}_{operation}(x:{typ},y:{typ})->{typ}{{x.{policy}_{operation}(y)}}\n"));
                }
            }
            body.push_str(&format!(
                "#[verifier::infer] fn {typ}_neg(x:{typ})->Option<{typ}>{{x.checked_neg()}}\n"
            ));
            let (other, suffix) =
                if typ == unsigned { (signed, "signed") } else { (unsigned, "unsigned") };
            for operation in ["add", "sub"] {
                body.push_str(&format!("#[verifier::infer] fn {typ}_checked_{operation}(x:{typ},y:{other})->Option<{typ}>{{x.checked_{operation}_{suffix}(y)}}\n"));
            }
            callers.push_str(&format!(
                r#"
                assert((3{typ}&5{typ})==1) by(bit_vector);let r={typ}_and(3,5);assert(r==1);
                assert((3{typ}|5{typ})==7) by(bit_vector);let r={typ}_or(3,5);assert(r==7);
                assert((3{typ}^5{typ})==6) by(bit_vector);let r={typ}_xor(3,5);assert(r==6);
                let r={typ}_shl(1,0);assert(r==(1{typ}<<0));
                let r={typ}_shr(1,0);assert(r==(1{typ}>>0));
                let r={typ}_saturating_add({typ}::MAX,1);assert(r=={typ}::MAX);
                let r={typ}_saturating_sub({typ}::MIN,1);assert(r=={typ}::MIN);
                let r={typ}_saturating_mul({typ}::MAX,2);assert(r=={typ}::MAX);
                let r={typ}_neg(0);assert(r==Some(0{typ}));
                let r={typ}_checked_add({typ}::MAX,1);assert(r==None);
                let r={typ}_checked_sub({typ}::MIN,1);assert(r==None);
            "#
            ));
        }
    }
    callers.push('}');
    body.push_str(&callers);
    for cache in ["miss", "hit"] {
        let run =
            crates.build("integer_matrix", &body, &[], &["--output-json", "--num-threads", "4"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        assert_eq!(report["functions"].as_array().unwrap().len(), 180);
        for f in report["functions"].as_array().unwrap() {
            assert_eq!(f["metrics"]["cache"], cache);
            for proof in ["soundness", "maximality", "exactness"] {
                assert_eq!(f["proofs"][proof]["status"], "proved");
            }
        }
    }
}

#[test]
fn integer_operations_32_64_signedness_edges_and_safe_shift_domains() {
    for bytes in [4, 8] {
        let crates = InferCrates::new();
        let body = format!(
            r#"use vstd::prelude::*;global size_of usize == {bytes};
            #[verifier::infer] fn shift(x:usize,n:i64)->usize{{x<<n}}
            #[verifier::infer] fn right(x:isize,n:u32)->isize{{x>>n}}
            #[verifier::infer] fn inv(x:usize)->usize{{!x}}
            #[verifier::infer] fn wrap(x:isize,y:isize)->isize{{x.wrapping_add(y)}}
            #[verifier::infer] fn mixed(x:usize,y:isize)->Option<usize>{{x.checked_sub_signed(y)}}
            #[verifier::infer] fn neg(x:isize)->Option<isize>{{x.checked_neg()}}
            fn caller(){{
                assert((usize::MAX<<1)==usize::MAX-1) by(bit_vector);
                let r=shift(usize::MAX,1);assert(r==usize::MAX-1);
                assert((-2isize>>1)== -1) by(bit_vector);
                let r=right(-2,1);assert(r == -1);
                assert(!0usize==usize::MAX) by(bit_vector);
                let r=inv(0);assert(r==usize::MAX);
                let r=wrap(isize::MAX,1);assert(r==isize::MIN);
                let r=mixed(0,-1);assert(r==Some(1usize));
                let r=mixed(usize::MAX,-1);assert(r==None);
                let r=neg(isize::MIN);assert(r==None);
            }}"#
        );
        let run = crates.verify("integer_arch", &body, &[], &["--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        for count in ["-1", "usize::BITS as i64", "64"] {
            let bad = format!("{body} fn invalid(){{let _=shift(0,{count});}}");
            let run = crates.verify("integer_bad_shift", &bad, &[], &["--output-json"]);
            assert!(!run.status.success());
            assert!(String::from_utf8_lossy(&run.stderr).contains("precondition not satisfied"));
            assert!(!crates.vir("integer_bad_shift").exists());
        }
    }
}

#[test]
fn integer_operations_composition_retypes_bitwise_operands_and_imports() {
    let crates = InferCrates::new();
    let base = r#"use vstd::prelude::*;
        #[verifier::infer] pub fn mask(x:usize)->usize{x&15}
        #[verifier::infer] pub fn shift(x:usize,n:u32)->usize{x<<n}
        #[verifier::infer] pub fn wrap(x:isize,y:isize)->isize{x.wrapping_add(y)}
        #[verifier::infer] pub fn negate(x:isize)->Option<isize>{x.checked_neg()}
        #[verifier::infer] pub fn bounded(x:isize,y:isize)->isize{x.saturating_mul(y)}
    "#;
    for _ in 0..2 {
        crates.succeeds("integer_base", base, &[]);
    }
    let middle = r#"use vstd::prelude::*;
        #[verifier::infer] pub fn masked(x:usize)->usize{integer_base::mask(integer_base::shift(x,1))}
        #[verifier::infer] pub fn increment_mask(x:usize)->usize{integer_base::mask(x+1)}
        #[verifier::infer] pub fn carry(x:isize,y:isize)->Option<isize>{integer_base::negate(integer_base::wrap(x,y))}
        #[verifier::infer] pub fn sat(x:isize,y:isize)->isize{integer_base::bounded(x,y)}
    "#;
    for _ in 0..2 {
        crates.succeeds("integer_middle", middle, &["integer_base"]);
    }
    for bytes in [4, 8] {
        let app = format!(
            r#"use vstd::prelude::*;global size_of usize == {bytes};fn caller(){{
            assert(((usize::MAX<<1)&15)==14) by(bit_vector);
            let r=integer_middle::masked(usize::MAX);assert(r==14);
            assert(((((14usize+1) as usize)&15))==15) by(bit_vector);
            let r=integer_middle::increment_mask(14);assert(r==15);
            let r=integer_middle::carry(isize::MAX,1);assert(r==None);
            let r=integer_middle::sat(isize::MIN,2);assert(r==isize::MIN);
        }}"#
        );
        let run = crates.verify("integer_app", &app, &["integer_base", "integer_middle"], &[]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    }
}

test_verify_one_file! {
    #[test] integer_operations_sound_intervals_and_guarded_composition code! {
        use vstd::prelude::*;
        #[verus_infer(precision="sound")] fn scale(x:u8)->u8{x/100}
        #[verus_infer(precision="sound")] fn shifted(x:u8)->u8{1u8<<scale(x)}
        #[verus_infer(precision="sound")] fn mixed(x:u8)->u8{scale(x).wrapping_add(1)&15}
        #[verus_infer(precision="sound")] fn guarded(x:isize,y:isize)->isize {
            let v=x.saturating_add(y);if v<isize::MAX{v+1}else{0}
        }
        #[verus_infer(precision="sound")] fn complemented(x:usize)->usize{!x}
        verus!{fn caller(){let r=shifted(255);assert(r<=255);let r=mixed(255);assert(r<=255);let r=guarded(isize::MAX,1);assert(isize::MIN<=r && r<=isize::MAX);}}
    } => Ok(diags) => {assert_eq!(diags.notes.iter().filter(|n|n.message.contains("soundness: PROVED")).count(),5);}
}

#[test]
fn integer_operations_incorrect_cached_candidates_never_publish() {
    for kind in [
        "permissive_shift",
        "restrictive_shift",
        "wrong_wrapping",
        "wrong_saturating",
        "wrong_checked",
    ] {
        let crates = InferCrates::new();
        let function = match kind {
            "permissive_shift" | "restrictive_shift" => {
                "#[verifier::infer] pub fn op(x:u8,y:i32)->u8{x<<y}"
            }
            "wrong_wrapping" => "#[verifier::infer] pub fn op(x:u8,y:u8)->u8{x.wrapping_add(y)}",
            "wrong_saturating" => {
                "#[verifier::infer] pub fn op(x:u8,y:u8)->u8{x.saturating_mul(y)}"
            }
            _ => "#[verifier::infer] pub fn op(x:u8,y:i8)->Option<u8>{x.checked_add_signed(y)}",
        };
        let body = format!("use vstd::prelude::*; {function}");
        let run = crates.build("integer_poison", &body, &[], &["--output-json"]);
        assert!(run.status.success());
        for file in std::fs::read_dir(crates.directory.path().join("candidate-cache")).unwrap() {
            let path = file.unwrap().path();
            let mut c: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            match kind {
                "permissive_shift" => c["pre"] = serde_json::json!({"Bool":true}),
                "restrictive_shift" => c["pre"] = serde_json::json!({"Bool":false}),
                "wrong_checked" => {
                    let typ = c["post"]["Binary"][2]["If"][1]["Ctor"][0].clone();
                    assert!(!typ.is_null());
                    c["post"]["Binary"][2] = serde_json::json!({"Ctor":[typ,"None",null]});
                }
                _ => {
                    c["post"]["Binary"][2] =
                        serde_json::to_value(vir::ast::Constant::Int(0.into())).unwrap()
                }
            }
            std::fs::write(path, serde_json::to_vec(&c).unwrap()).unwrap();
        }
        let run = crates.build("integer_poison", &body, &[], &["--output-json"]);
        assert!(!run.status.success(), "{}", kind);
        let report = json_inference(&run);
        let f = &report["functions"][0];
        assert_eq!(f["metrics"]["cache"], "hit");
        assert_eq!(f["available_to_same_crate_callers"], false);
        assert_eq!(f["exported"], false);
        assert_eq!(
            f["proofs"][if kind == "restrictive_shift" { "maximality" } else { "soundness" }]["status"],
            "not_proved"
        );
        assert!(!crates.vir("integer_poison").exists());
    }
}

#[test]
fn integer_operations_malformed_cached_bit_widths_are_rejected_before_reproof() {
    let crates = InferCrates::new();
    let body = "#[verifier::infer] pub fn invert(x:u8)->u8{!x}";
    let run = crates.build("integer_bad_width", body, &[], &["--output-json"]);
    assert!(run.status.success());
    for width in
        [serde_json::Value::Null, serde_json::json!({"Width":16}), serde_json::json!({"Width":256})]
    {
        for file in std::fs::read_dir(crates.directory.path().join("candidate-cache")).unwrap() {
            let path = file.unwrap().path();
            let mut candidate: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            assert!(candidate["post"]["Binary"][2]["BitNot"].is_array());
            candidate["post"]["Binary"][2]["BitNot"][0] = width.clone();
            std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
        }
        let run = crates.build("integer_bad_width", body, &[], &["--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        let f = &report["functions"][0];
        assert_eq!(f["metrics"]["cache"], "miss");
        for proof in ["soundness", "maximality", "exactness"] {
            assert_eq!(f["proofs"][proof]["status"], "proved");
        }
    }
}
// Generate source separately from the recognizer and inspect every required proof.
#[test]
fn counted_step_families_all_unsigned_widths_and_pointer_models() {
    for bytes in [4, 8] {
        let crates = InferCrates::new();
        let mut body = format!("use vstd::prelude::*; global size_of usize == {bytes};");
        let mut calls = String::from("fn caller(){");
        for typ in ["u8", "u16", "u32", "u64", "u128", "usize"] {
            for subtract in [false, true] {
                for countdown in [false, true] {
                    for param in [false, true] {
                        let name = format!("f_{typ}_{subtract}_{countdown}_{param}");
                        let third = if param { format!(",step:{typ}") } else { String::new() };
                        let step = if param { "step" } else { "2" };
                        let op = if subtract { "-" } else { "+" };
                        let init = if countdown { "n".into() } else { format!("0{typ}") };
                        let guard = if countdown { "i>0" } else { "i<n" };
                        let update = if countdown { "i=i-1" } else { "i+=1" };
                        body.push_str(&format!("#[verifier::infer] pub fn {name}(x:{typ},n:{typ}{third})->{typ}{{let mut r=x;let mut i={init};while {guard}{{r=r{op}{step};{update};}}r}}"));
                        let third_arg = if param { ",2" } else { "" };
                        let (x, n, result) =
                            if subtract { ("254", "127", "0") } else { ("1", "127", "255") };
                        calls.push_str(&format!("let r={name}({x},{n}{third_arg});assert(r=={result});let r={name}({typ}::MAX,0{third_arg});assert(r=={typ}::MAX);"));
                        if param {
                            calls.push_str(&format!("let r={name}({typ}::MAX,{typ}::MAX,0);assert(r=={typ}::MAX);let r={name}({typ}::MAX,0,{typ}::MAX);assert(r=={typ}::MAX);"));
                            let start = if subtract { format!("{typ}::MAX") } else { "0".into() };
                            let result = if subtract { "0".into() } else { format!("{typ}::MAX") };
                            calls.push_str(&format!(
                                "let r={name}({start},1,{typ}::MAX);assert(r=={result});"
                            ));
                        }
                    }
                }
            }
        }
        body.push_str(&calls);
        body.push('}');
        // A symbolic platform constant and aliased step parameters have no host-width specialization.
        body.push_str("#[verifier::infer] fn platform_constant(x:usize,n:usize)->usize{let mut r=x;let mut i=n;while i>0{r+=usize::MAX;i-=1;}r}");
        body.push_str("#[verifier::infer] fn alias_initial(x:u8,n:u8)->u8{let mut r=x;let mut i=0u8;while i<n{r+=x;i+=1;}r}");
        body.push_str("#[verifier::infer] fn alias_bound(x:u8,n:u8)->u8{let mut r=x;let mut i=n;while i>0{r-=n;i-=1;}r}");
        let run =
            crates.verify("family_matrix", &body, &[], &["--output-json", "--num-threads", "4"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        assert_eq!(report["functions"].as_array().unwrap().len(), 51);
        for f in report["functions"].as_array().unwrap() {
            assert_eq!(f["loop_proofs"].as_array().unwrap().len(), 1);
            for proof in ["soundness", "maximality", "exactness"] {
                assert_eq!(f["proofs"][proof]["status"], "proved", "{f}");
            }
        }
    }
}

const COUNTED_FAMILY_BASE: &str = r#"
    #[verifier::infer] pub fn up_add(x:u8,n:u8,s:u8)->u8{let mut r=x;let mut i=0u8;while i<n{r+=s;i+=1;}r}
    #[verifier::infer] pub fn up_sub(x:u8,n:u8,s:u8)->u8{let mut r=x;let mut i=0u8;while i<n{r-=s;i+=1;}r}
    #[verifier::infer] pub fn down_add(x:u8,n:u8,s:u8)->u8{let mut i=n;let mut r=x;while i>0{r=r+s;i=i-1;}r}
    #[verifier::infer] pub fn down_sub(x:u8,n:u8,s:u8)->u8{let mut i=n;let mut r=x;while i>0{r=r-s;i=i-1;}r}
    #[verifier::infer] pub fn platform_debit(x:usize,n:usize,s:usize)->usize{let mut r=x;let mut i=n;while i>0{r-=s;i-=1;}r}
"#;

#[test]
fn counted_step_families_cache_and_three_crate_composition() {
    let crates = InferCrates::new();
    for cache in ["miss", "hit"] {
        let run = crates.build("family_base", COUNTED_FAMILY_BASE, &[], &["--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        for f in json_inference(&run)["functions"].as_array().unwrap() {
            assert_eq!(f["metrics"]["cache"], cache);
            assert_eq!(f["loop_proofs"][0]["arithmetic_prover"], "nonlinear");
            for proof in ["soundness", "maximality", "exactness"] {
                assert_eq!(f["proofs"][proof]["status"], "proved");
            }
        }
    }
    crates.succeeds("family_middle", r#"
        #[verifier::infer] pub fn twice(x:u8,n:u8,s:u8)->u8{family_base::up_add(family_base::down_add(x,n,s),n,s)}
        #[verifier::infer] pub fn debit(x:u8,n:u8,s:u8)->u8{family_base::down_sub(family_base::up_sub(x,n,s),n,s)}
        #[verifier::infer] pub fn platform(x:usize,n:usize,s:usize)->usize{family_base::platform_debit(x,n,s)}
    "#, &["family_base"]);
    crates.succeeds("family_app", r#"
        #[verifier::infer] pub fn top(x:u8,n:u8,s:u8)->u8{family_middle::twice(x,n,s)}
        fn caller(){let r=top(3,63,2);assert(r==255);let r=family_middle::debit(252,63,2);assert(r==0);}
    "#, &["family_base", "family_middle"]);
    for bytes in [4, 8] {
        let body = format!(
            r#"use vstd::prelude::*;global size_of usize == {bytes};
            #[verifier::infer] pub fn top(x:usize,n:usize,s:usize)->usize{{family_middle::platform(x,n,s)}}
            fn caller(){{let r=top(usize::MAX,1,usize::MAX);assert(r==0);let r=top(usize::MAX,usize::MAX,0);assert(r==usize::MAX);}}
        "#
        );
        let run =
            crates.verify("family_platform_app", &body, &["family_base", "family_middle"], &[]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    }
}

#[test]
fn counted_step_families_poisoned_candidates_and_incomplete_proofs_never_publish() {
    for kind in ["restrictive", "permissive", "wrong_output", "weak_output"] {
        let crates = InferCrates::new();
        let body = "#[verifier::infer] pub fn op(x:u8,n:u8,s:u8)->u8{let mut r=x;let mut i=n;while i>0{r-=s;i-=1;}r}fn caller(){let _=op(0,0,0);}";
        assert!(crates.build("family_poison", body, &[], &["--output-json"]).status.success());
        for file in std::fs::read_dir(crates.directory.path().join("candidate-cache")).unwrap() {
            let path = file.unwrap().path();
            let mut c: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            match kind {
                "restrictive" => c["pre"] = serde_json::json!({"Bool":false}),
                "permissive" => c["pre"] = serde_json::json!({"Bool":true}),
                "weak_output" => c["post"] = serde_json::json!({"Bool":true}),
                _ => {
                    c["post"]["Binary"][2] =
                        serde_json::to_value(vir::ast::Constant::Int(0.into())).unwrap()
                }
            }
            std::fs::write(path, serde_json::to_vec(&c).unwrap()).unwrap();
        }
        let run = crates.build("family_poison", body, &[], &["--output-json"]);
        if kind == "weak_output" {
            // Cache recipes require result equality. A malformed relation is
            // discarded, then source inference and all proofs run afresh.
            assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
            let report = json_inference(&run);
            assert_eq!(report["functions"][0]["metrics"]["cache"], "miss");
            for proof in ["soundness", "maximality", "exactness"] {
                assert_eq!(report["functions"][0]["proofs"][proof]["status"], "proved");
            }
            continue;
        }
        assert!(!run.status.success(), "{kind}: {}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        let f = &report["functions"][0];
        assert_eq!(f["metrics"]["cache"], "hit");
        assert_eq!(f["available_to_same_crate_callers"], false);
        assert_eq!(f["exported"], false);
        let failed = match kind {
            "restrictive" => "maximality",
            "weak_output" => "exactness",
            _ => "soundness",
        };
        assert_eq!(f["proofs"][failed]["status"], "not_proved", "{f}");
        assert!(!crates.vir("family_poison").exists());
    }
    let crates = InferCrates::new();
    let body = "#[verifier::infer_precision_rlimit(0.000001)] #[verifier::infer] pub fn op(x:u8,n:u8,s:u8)->u8{let mut r=x;let mut i=n;while i>0{r+=s;i-=1;}r}fn caller(){assert(false);}";
    let run = crates.build("family_timeout", body, &[], &["--output-json"]);
    assert!(!run.status.success());
    let report = json_inference(&run);
    assert_eq!(report["status"], "required_proof_incomplete");
    assert_eq!(report["functions"][0]["proofs"]["soundness"]["status"], "proved");
    assert!(!String::from_utf8_lossy(&run.stderr).contains("assertion failed"));
    assert!(!crates.vir("family_timeout").exists());
}

#[test]
fn counted_step_families_reject_unsafe_calls_and_unsupported_source_shapes() {
    let crates = InferCrates::new();
    for (function, call) in [
        ("up_add", "255,1,1"),
        ("down_add", "1,255,2"),
        ("up_sub", "0,1,1"),
        ("down_sub", "254,128,2"),
    ] {
        let body = format!("{COUNTED_FAMILY_BASE} fn caller(){{let _={function}({call});}}");
        let run = crates.verify("family_unsafe", &body, &[], &[]);
        assert!(!run.status.success(), "{}", function);
        assert!(String::from_utf8_lossy(&run.stderr).contains("precondition not satisfied"));
    }
    for source in [
        "let mut r=x;let mut i=n;while i>0{r+=s;i+=1;}r",
        "let mut r=x;let mut i=0u8;while i<n{r+=s;i-=1;}r",
        "let mut r=x;let mut i=n;while i>0{r+=s;i-=2;}r",
        "let mut r=x;let mut i=n;while i>0{r+=i;i-=1;}r",
        "let mut r=x;let mut i=n;while i>0{r+=r;i-=1;}r",
        "let mut r=x;let mut i=n;while i>0{r+=s+1;i-=1;}r",
        "let mut r=x;let mut i=n;while i>0{r=r.wrapping_add(s);i-=1;}r",
        "let mut r=x;let mut i=n;while i>=1{r+=s;i-=1;}r",
        "let mut r=x;let mut i=n;while i>0{r+=s;s-=1;i-=1;}r",
    ] {
        let mutable = if source.contains("s-=1") { "mut " } else { "" };
        let run = crates.verify(
            "family_shape",
            &format!("#[verifier::infer] fn op(x:u8,n:u8,{mutable}s:u8)->u8{{{source}}}"),
            &[],
            &[],
        );
        assert!(!run.status.success(), "{}", source);
    }
}
test_verify_one_file! {
    #[test] coverage_control_flow_patterns_and_array_outputs code! {
        use vstd::prelude::*;
        #[verus_infer] fn optional(x:Option<u8>)->[u8;2]{let Some(v)=x else{return [0,1];};[v,v+1]}
        #[verus_infer] fn pattern(x:Result<u8,u8>)->u8{match x{Ok(0|1)=>0,Ok(v)|Err(v)=>v}}
        #[verus_infer] fn guards(x:Option<u8>,d:u8)->u8{match x{Some(v) if d!=0 && v/d>0=>v/d,Some(v) if v<255=>v+1,_=>0}}
        #[verus_infer] fn guard_exit(x:Option<u8>,d:u8)->u8{match x{Some(v) if {if d==0{return v;}v/d>0}=>v/d,_=>0}}
        #[verus_infer] fn error(x:Result<u8,i8>)->Result<u8,i128>{let v=x?;Ok(v+1)}
        #[verus_infer] fn arithmetic_guard(x:Option<u8>,y:u8)->u8{match x{Some(v) if v+y>0=>v,_=>0}}
        #[verus_infer] fn arrays(x:u8,b:bool)->[u8;2]{if b{return [x,0];}[x,x+1]}
        #[verus_infer] fn copied(x:[u8;2])->[u8;2]{let y=x; y}
        #[verus_infer] fn empty(x:u8)->[u8;0]{[x+1;0]}
        #[verus_infer] fn full(x:u8)->[u8;64]{[x;64]}
        verus!{fn caller(){
            let r=optional(None);assert(r[0]==0 && r[1]==1);
            let r=optional(Some(254));assert(r[1]==255);
            let r=pattern(Err(255));assert(r==255);
            let r=guards(Some(254),0);assert(r==255);
            let r=guard_exit(Some(255),0);assert(r==255);
            let r=guard_exit(None,0);assert(r==0);
            let r=error(Err(-128));assert(r==Err(-128i128));
            let r=arithmetic_guard(None,255);assert(r==0);
            let r=arrays(255,true);assert(r[0]==255 && r[1]==0);
            let r=copied([255,7]);assert(r[0]==255 && r[1]==7);
            let r=empty(254);assert(r.len()==0);
            let r=full(255);assert(r[0]==255 && r[63]==255);
        }}
    } => Ok(())
}

test_verify_one_file! {
    #[test] coverage_shared_subslices_and_array_composition code! {
        use vstd::prelude::*;
        #[verus_infer] fn at(s:&[u8],i:usize)->u8{s[i]}
        #[verus_infer] fn pair(x:u8)->[u8;2]{[x,x+1]}
        #[verus_infer] fn offset(s:&[u8],a:usize,b:usize,i:usize)->u8{let t=&s[a..b];at(t,i)}
        #[verus_infer] fn open(s:&[u8],a:usize,b:usize)->usize{let t=&s[a..];let u=&t[..b];let v=&u[..];v.len()}
        #[verus_infer] fn nested(s:&[u8],a:usize,b:usize,c:usize,i:usize)->u8{let t=&s[a..b];let u=&t[c..];at(u,i)}
        #[verus_infer] fn build_read(x:u8)->u8{let a=pair(x);let s=&a;at(s,1)}
        #[verus_infer] fn from_ref(a:&[u8;2])->[u8;2]{*a}
        #[verus_infer] fn coerced_local(x:u8)->u8{let a=pair(x);let s:&[u8]=&a;let w=&s[1..];at(w,0)}
        #[verus_infer] fn combined(s:&[u8],bounds:Option<u8>)->[u8;2]{let Some(a)=bounds else{return [0,0];};let tail=&s[a as usize..];let n=tail.len();if n==0{return [0,0];}[at(tail,0),n as u8]}
        verus!{fn caller(){
            let r=offset(&[1,3,7,255],1,4,2);assert(r==255);
            let r=open(&[1,3,7],3,0);assert(r==0);
            let r=nested(&[1,3,7,255],1,4,1,1);assert(r==255);
            let r=build_read(254);assert(r==255);
            let a=from_ref(&[1,255]);assert(a[0]==1 && a[1]==255);
            let r=coerced_local(254);assert(r==255);
            let r=combined(&[3,7,255],Some(1));assert(r[0]==7 && r[1]==2);
            let r=combined(&[],None);assert(r[0]==0 && r[1]==0);
        }}
    } => Ok(())
}

#[test]
fn coverage_primitive_error_conversion_matrix_and_pointer_models() {
    let crates = InferCrates::new();
    let mut body = String::from("use vstd::prelude::*;");
    let mut caller = String::from("fn caller(){");
    let mut count = 0;
    let conversions = [
        ("u8", vec!["u16", "u32", "u64", "u128", "usize", "i16", "i32", "i64", "i128", "isize"]),
        ("u16", vec!["u32", "u64", "u128", "usize", "i32", "i64", "i128"]),
        ("u32", vec!["u64", "u128", "i64", "i128"]),
        ("u64", vec!["u128", "i128"]),
        ("i8", vec!["i16", "i32", "i64", "i128", "isize"]),
        ("i16", vec!["i32", "i64", "i128", "isize"]),
        ("i32", vec!["i64", "i128"]),
        ("i64", vec!["i128"]),
    ];
    for (from, tos) in conversions {
        for to in tos {
            let name = format!("convert_{from}_{to}");
            body.push_str(&format!(
                "#[verifier::infer] fn {name}(x:Result<u8,{from}>)->Result<u8,{to}>{{Ok(x?)}}"
            ));
            caller.push_str(&format!("let r={name}(Err({from}::MIN));assert(r==Err({from}::MIN as {to}));let r={name}(Err({from}::MAX));assert(r==Err({from}::MAX as {to}));let r={name}(Ok(255));assert(r==Ok(255));"));
            count += 1;
        }
    }
    body.push_str(&caller);
    body.push('}');
    for bytes in [4, 8] {
        let run = crates.verify(
            "coverage_conversions",
            &format!("global size_of usize == {bytes};{body}"),
            &[],
            &["--output-json", "--num-threads", "4"],
        );
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        assert_eq!(report["functions"].as_array().unwrap().len(), count);
        for f in report["functions"].as_array().unwrap() {
            for p in ["soundness", "maximality", "exactness"] {
                assert_eq!(f["proofs"][p]["status"], "proved", "{f}");
            }
        }
    }
}

#[test]
fn coverage_unsafe_control_flow_arrays_and_slice_ranges_are_rejected() {
    let crates = InferCrates::new();
    for (source, call) in [
        (
            "#[verifier::infer] fn op(x:Option<u8>)->u8{let Some(v)=x else{return 0;};v+1}",
            "op(Some(255))",
        ),
        (
            "#[verifier::infer] fn op(x:Option<u8>,y:u8)->u8{match x{Some(v) if v+y>0=>v,_=>0}}",
            "op(Some(255),1)",
        ),
        ("#[verifier::infer] fn op(x:u8)->[u8;2]{[x,x+1]}", "op(255)"),
        ("#[verifier::infer] fn op(x:u8)->[u8;0]{[x+1;0]}", "op(255)"),
        (
            "#[verifier::infer] fn op(s:&[u8],a:usize,b:usize)->usize{let t=&s[a..b];t.len()}",
            "op(&[1,2],2,1)",
        ),
        (
            "#[verifier::infer] fn op(s:&[u8],a:usize,b:usize)->usize{let t=&s[a..b];t.len()}",
            "op(&[1,2],0,3)",
        ),
        (
            "#[verifier::infer] fn op(s:&[u8],a:usize,i:usize)->u8{let t=&s[a..];t[i]}",
            "op(&[1,2],2,0)",
        ),
    ] {
        let body = format!("use vstd::prelude::*;{source} fn caller(){{let _={call};}}");
        let run = crates.verify("coverage_unsafe", &body, &[], &[]);
        assert!(!run.status.success(), "{}", body);
        assert!(
            String::from_utf8_lossy(&run.stderr).contains("precondition not satisfied"),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
    }
}
const COVERAGE_BASE: &str = r#"
use vstd::prelude::*;
#[verifier::infer] pub fn pair(x:u8)->[u8;2]{[x,x+1]}
#[verifier::infer] pub fn selected(x:Option<u8>)->[u8;2]{let Some(v)=x else{return [0,0];};pair(v)}
#[verifier::infer] pub fn widened(x:Result<u8,u8>)->Result<u8,u16>{Ok(x?)}
#[verifier::infer] pub fn window(s:&[u8],start:usize,end:usize,i:usize)->u8{let w=&s[start..end];w[i]}
"#;

#[test]
fn coverage_cache_cross_crate_and_pointer_models_preserve_all_guarantees() {
    let crates = InferCrates::new();
    for cache in ["miss", "hit"] {
        let run = crates.build("coverage_base", COVERAGE_BASE, &[], &["--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        for f in json_inference(&run)["functions"].as_array().unwrap() {
            assert_eq!(f["metrics"]["cache"], cache);
            for proof in ["soundness", "maximality", "exactness"] {
                assert_eq!(f["proofs"][proof]["status"], "proved", "{f}");
            }
        }
    }
    crates.succeeds("coverage_middle",r#"use vstd::prelude::*;
        #[verifier::infer] pub fn chosen(x:Option<u8>)->[u8;2]{coverage_base::selected(x)}
        #[verifier::infer] pub fn read(s:&[u8],input:Result<u8,u8>)->Result<u8,u16>{let i=coverage_base::widened(input)?;Ok(coverage_base::window(s,i as usize,s.len(),0))}
        #[verifier::infer] pub fn early(input:Result<u8,u8>)->[u8;2]{match input{Ok(v) if v<255=>coverage_base::pair(v),Ok(v)|Err(v)=>[v,0]}}
    "#,&["coverage_base"]);
    for bytes in [4, 8] {
        let body = format!(
            r#"use vstd::prelude::*;global size_of usize == {bytes};
            #[verifier::infer] pub fn top(x:Option<u8>)->[u8;2]{{coverage_middle::chosen(x)}}
            fn caller(){{let a=top(Some(254));assert(a[0]==254 && a[1]==255);let a=top(None);assert(a[0]==0 && a[1]==0);
            let r=coverage_middle::read(&[1,7,255],Ok(2));assert(r==Ok(255));let r=coverage_middle::read(&[],Err(255));assert(r==Err(255u16));
            let a=coverage_middle::early(Ok(255));assert(a[0]==255 && a[1]==0);}}
        "#
        );
        let run = crates.verify("coverage_app", &body, &["coverage_base", "coverage_middle"], &[]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    }
}

#[test]
fn coverage_poisoned_array_and_range_candidates_never_publish() {
    for kind in ["permissive", "restrictive", "wrong_array", "malformed_array"] {
        let crates = InferCrates::new();
        let body = "use vstd::prelude::*;#[verifier::infer] pub fn window(s:&[u8],start:usize,end:usize)->[u8;2]{let w=&s[start..end];[w[0],w[1]]}fn caller(){let _=window(&[1,2],0,2);}";
        assert!(crates.build("coverage_poison", body, &[], &["--output-json"]).status.success());
        for file in std::fs::read_dir(crates.directory.path().join("candidate-cache")).unwrap() {
            let path = file.unwrap().path();
            let mut c: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            match kind {
                "permissive" => c["pre"] = serde_json::json!({"Bool":true}),
                "restrictive" => c["pre"] = serde_json::json!({"Bool":false}),
                "wrong_array" => {
                    c["post"]["Binary"][2]["Array"][2][1] =
                        serde_json::to_value(vir::ast::Constant::Int(0.into())).unwrap()
                }
                _ => c["post"]["Binary"][2]["Array"][1] = serde_json::json!(65),
            }
            std::fs::write(path, serde_json::to_vec(&c).unwrap()).unwrap();
        }
        let run = crates.build("coverage_poison", body, &[], &["--output-json"]);
        let report = json_inference(&run);
        let f = &report["functions"][0];
        if kind == "malformed_array" {
            assert!(run.status.success());
            assert_eq!(f["metrics"]["cache"], "miss");
            continue;
        }
        assert!(!run.status.success(), "{}", kind);
        assert_eq!(f["metrics"]["cache"], "hit");
        assert_eq!(f["available_to_same_crate_callers"], false);
        assert_eq!(f["exported"], false);
        assert_eq!(
            f["proofs"][if kind == "restrictive" { "maximality" } else { "soundness" }]["status"],
            "not_proved",
            "{f}"
        );
        assert!(!crates.vir("coverage_poison").exists());
    }
}

#[test]
fn coverage_source_effects_and_unsupported_interfaces_are_rejected() {
    let crates = InferCrates::new();
    for source in [
        "#[verifier::infer] fn op(input:Option<u8>)->u8{let Some(v)=input else{assume(false);return 0;};v}",
        "#[verifier::infer] fn op(input:Option<u8>)->u8{match input{Some(v) if {assume(false);true}=>v,_=>0}}",
        "#[verifier::infer] fn op(s:&[u8])->usize{let w=&s[{assume(false);0}..];w.len()}",
        "#[verifier::infer] fn op(s:&[u8])->&[u8]{let w=&s[..];w}",
        "#[verifier::infer] fn op(x:u8)->[u8;65]{[x;65]}",
        "#[verifier::infer] fn op(x:Option<u8>)->Option<[u8;2]>{Some([0,1])}",
        "#[verifier::infer] fn op(s:&[u8],i:usize)->usize{let w=&s[..=i];w.len()}",
    ] {
        let run =
            crates.verify("coverage_source", &format!("use vstd::prelude::*;{source}"), &[], &[]);
        assert!(!run.status.success(), "{}", source);
        assert!(
            String::from_utf8_lossy(&run.stderr).contains("unsupported"),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
    }
}
#[test]
fn coverage_array_outputs_all_scalar_types_lengths_and_pointer_models() {
    let crates = InferCrates::new();
    let mut body = String::from("use vstd::prelude::*;");
    let mut caller = String::from("fn caller(){");
    for typ in [
        "bool", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128",
        "isize",
    ] {
        for n in [0, 1, 2, 64] {
            let name = format!("array_{typ}_{n}");
            body.push_str(&format!(
                "#[verifier::infer] pub fn {name}(value:{typ})->[{typ};{n}]{{[value;{n}]}}"
            ));
            for v in if typ == "bool" {
                vec!["false".into(), "true".into()]
            } else {
                vec![format!("{typ}::MIN"), format!("{typ}::MAX")]
            } {
                caller.push_str(&format!("let a={name}({v});assert(a.len()=={n});"));
                if n > 0 {
                    caller.push_str(&format!("assert(a[0]=={v} && a[{}]=={v});", n - 1));
                }
            }
        }
    }
    body.push_str(&caller);
    body.push('}');
    for bytes in [4, 8] {
        let run = crates.verify(
            "coverage_arrays",
            &format!("global size_of usize == {bytes};{body}"),
            &[],
            &["--output-json", "--num-threads", "4"],
        );
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        let report = json_inference(&run);
        assert_eq!(report["functions"].as_array().unwrap().len(), 52);
        assert!(
            report["trusted_models"]
                .as_array()
                .unwrap()
                .iter()
                .any(|m| m["function"].as_str().unwrap().ends_with("axiom_array_ext_equal"))
        );
        for f in report["functions"].as_array().unwrap() {
            for p in ["soundness", "maximality", "exactness"] {
                assert_eq!(f["proofs"][p]["status"], "proved", "{f}");
            }
        }
    }
}

#[test]
fn coverage_incomplete_array_proofs_remove_stale_exports_and_block_callers() {
    let crates = InferCrates::new();
    let function = "#[verifier::infer] pub fn array(value:u8)->[u8;2]{[value,value+1]}";
    assert!(crates.build("coverage_timeout", function, &[], &["--output-json"]).status.success());
    assert!(crates.vir("coverage_timeout").exists());
    let body = format!(
        "#[verifier::infer_precision_rlimit(0.000001)]{function} fn caller(){{assert(false);}}"
    );
    let run = crates.build("coverage_timeout", &body, &[], &["--output-json"]);
    assert!(!run.status.success());
    let report = json_inference(&run);
    assert_eq!(report["status"], "required_proof_incomplete");
    let f = &report["functions"][0];
    assert_eq!(f["proofs"]["soundness"]["status"], "proved");
    assert_eq!(f["available_to_same_crate_callers"], false);
    assert_eq!(f["exported"], false);
    assert!(!String::from_utf8_lossy(&run.stderr).contains("assertion failed"));
    assert!(!crates.vir("coverage_timeout").exists());
}


#[test]
fn precision_ranges_all_integer_types_and_pointer_models_are_freshly_proved() {
    let crates = InferCrates::new();
    let mut body = String::from("use vstd::prelude::*;");
    let mut caller = String::from("fn caller(){");
    for typ in [
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
    ] {
        body.push_str(&format!(
            "#[verifier::infer(sound)] pub fn mask_{typ}(x:{typ})->{typ}{{(x&15)>>1}}"
        ));
        body.push_str(&format!("#[verifier::infer(sound)] pub fn branch_{typ}(x:{typ})->{typ}{{let copy=x;let small=copy>=0 && copy<16;if small{{x>>1}}else{{0}}}}"));
        caller.push_str(&format!("let r=mask_{typ}({typ}::MAX);assert(0<=r && r<=7);let r=branch_{typ}({typ}::MIN);assert(0<=r && r<=7);"));
    }
    body.push_str(&caller);
    body.push('}');
    for bytes in [4, 8] {
        for cache in ["miss", "hit"] {
            let run = crates.verify(
                &format!("precision_matrix_{bytes}"),
                &format!("global size_of usize == {bytes};{body}"),
                &[],
                &["--output-json", "--num-threads", "4"],
            );
            assert!(
                run.status.success(),
                "{}",
                String::from_utf8_lossy(&run.stderr)
            );
            let report = json_inference(&run);
            assert_eq!(report["functions"].as_array().unwrap().len(), 24);
            for f in report["functions"].as_array().unwrap() {
                assert_eq!(f["metrics"]["cache"], cache, "{f}");
                assert_eq!(f["proofs"]["soundness"]["status"], "proved", "{f}");
                assert_eq!(f["mode"], "sound");
            }
        }
    }
}

#[test]
fn precision_truncation_arithmetic_aliases_and_contract_simplification() {
    let crates = InferCrates::new();
    let body = r#"use vstd::prelude::*;
    #[verifier::infer(sound)] pub fn code(x:i16)->i8{if x>=128 && x<=130{x as i8}else{-128}}
    #[verifier::infer(sound)] pub fn wrap(x:u8)->u8{if x>250{x.wrapping_add(10)}else{0}}
    #[verifier::infer(sound)] pub fn shift(n:i16)->u8{1u8<<n}
    #[verifier::infer(sound)] pub fn sat(x:u8)->u8{if x<10{x.saturating_add(1)}else{0}}
    #[verifier::infer] pub fn cancel(x:u8)->u8{(x+1)-x}
    #[verifier::infer] pub fn zero(x:u8)->u8{x-x}
    #[verifier::infer] pub fn window(s:&[u8],i:usize)->u8{let w=&s[i..];w[0]}
    fn caller(x:u8){let r=shift(7);assert(1<=r && r<=128);let r=wrap(x);assert(r<=9);let r=sat(x);assert(r<=10);let r=code(130);assert(-128<=r && r<=-126);let r=cancel(254);assert(r==1);}
    "#;
    for cache in ["miss", "hit"] {
        let run = crates.build("precision_arithmetic", body, &[], &["--output-json"]);
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        let report = json_inference(&run);
        for f in report["functions"].as_array().unwrap() {
            assert_eq!(f["metrics"]["cache"], cache);
            let name = f["function"].as_str().unwrap();
            if name.ends_with("::cancel") {
                assert_eq!(f["contract"]["requires"], "(x <= 254)");
                assert_eq!(f["contract"]["ensures"], "result == 1");
            }
            if name.ends_with("::zero") {
                assert_eq!(f["contract"]["requires"], "true");
                assert_eq!(f["contract"]["ensures"], "result == 0");
            }
            if name.ends_with("::window") {
                assert_eq!(f["contract"]["requires"], "(i < (s).len())");
            }
        }
    }
    let unsafe_body = body.replace("let r=cancel(254)", "let r=cancel(255)");
    let run = crates.verify("precision_unsafe", &unsafe_body, &[], &["--output-json"]);
    assert!(!run.status.success());
    assert_eq!(json_inference(&run)["status"], "caller_proof_failed");
}

#[test]
fn precision_poisoned_mask_bounds_cannot_borrow_checked_hint_evidence() {
    let crates = InferCrates::new();
    let body = "use vstd::prelude::*;#[verifier::infer(sound)] pub fn masked(x:u8)->u8{x&15}fn caller(){let _=masked(255);}";
    let run = crates.build("precision_mask_poison", body, &[], &["--output-json"]);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    for file in std::fs::read_dir(crates.directory.path().join("candidate-cache")).unwrap() {
        let path = file.unwrap().path();
        let mut c: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        c["post"]["Logical"][2]["Binary"][2] =
            serde_json::to_value(vir::ast::Constant::Int(14.into())).unwrap();
        std::fs::write(path, serde_json::to_vec(&c).unwrap()).unwrap();
    }
    let run = crates.build("precision_mask_poison", body, &[], &["--output-json"]);
    assert!(!run.status.success());
    let report = json_inference(&run);
    let f = &report["functions"][0];
    assert_eq!(f["metrics"]["cache"], "hit");
    assert_eq!(f["proofs"]["soundness"]["status"], "not_proved");
    assert_eq!(f["available_to_same_crate_callers"], false);
    assert!(!crates.vir("precision_mask_poison").exists());
}

#[test]
fn precision_bitvector_resource_limits_never_publish_or_start_callers() {
    let crates = InferCrates::new();
    let body = "use vstd::prelude::*;#[verifier::infer(sound)]#[verifier::rlimit(0.000001)]pub fn masked(x:u128)->u128{x&15}fn caller(){assert(false);}";
    let run = crates.build("precision_hint_timeout", body, &[], &["--output-json"]);
    assert!(!run.status.success());
    let report = json_inference(&run);
    assert_eq!(report["status"], "required_proof_incomplete");
    assert_eq!(
        report["functions"][0]["proofs"]["soundness"]["status"],
        "incomplete"
    );
    assert_eq!(
        report["functions"][0]["available_to_same_crate_callers"],
        false
    );
    assert!(!crates.vir("precision_hint_timeout").exists());
    assert!(!String::from_utf8_lossy(&run.stderr).contains("assertion failed"));
}

#[test]
fn owned_private_factories_and_copy_loops_cross_crates_with_exact_evidence() {
    let crates = InferCrates::new();
    let provider = r#"
 use vstd::prelude::*;
 pub struct Slots<T>(Vec<Option<T>>);
 impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}
 impl<T:Clone> Slots<T>{#[verifier::infer] pub fn new(n:usize)->Self{Slots(vec![None;n])}}
 impl<T:Copy> Slots<T>{
  #[verifier::infer] pub fn copy_contents(&self)->Self{
   let mut out=Vec::with_capacity(self.0.len());let mut i=0usize;
   while i<self.0.len(){out.push(self.0[i]);i+=1;}Slots(out)
  }
 }
"#;
    let consumer = r#"
 use vstd::prelude::*;
 pub struct State<T>{pub slots:owned_provider::Slots<T>,pub epoch:u16,pub used:usize}
 impl<T:Copy> State<T>{
  #[verifier::infer] pub fn new(n:usize)->Self{Self{slots:owned_provider::Slots::new(n),epoch:0,used:0}}
  #[verifier::infer] pub fn next(&self)->Option<Self>{
   if self.epoch==u16::MAX{return None;}
   Some(Self{slots:self.slots.copy_contents(),epoch:self.epoch+1,used:0})
  }
 }
 pub fn initial<T:Copy>(n:usize)->(r:State<T>)
  ensures r.slots@.len()==n,r.epoch==0,r.used==0,forall|i:int|0<=i<n ==> r.slots@[i].is_none(),{State::new(n)}
 pub fn next<T:Copy>(s:&State<T>)->(r:Option<State<T>>)
  ensures r.is_none()==(s.epoch==u16::MAX),r.is_some() ==> r.unwrap().slots@==s.slots@,
   r.is_some() ==> r.unwrap().epoch==s.epoch+1,r.is_some() ==> r.unwrap().used==0,{s.next()}
"#;
    let check = |run: &std::process::Output| {
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        let report = json_inference(run);
        let fs = report["functions"].as_array().unwrap();
        assert_eq!(fs.len(), 2);
        for f in fs {
            assert_eq!(f["summary_kind"], "owned_value");
            for p in ["soundness", "maximality", "exactness"] {
                assert_eq!(f["proofs"][p]["status"], "proved", "{f}");
            }
        }
    };
    for _ in 0..2 {
        check(&crates.build(
            "owned_provider",
            provider,
            &[],
            &["--no-cheating", "--output-json"],
        ));
        check(&crates.build(
            "owned_consumer",
            consumer,
            &["owned_provider"],
            &["--no-cheating", "--output-json"],
        ));
    }
    // Precisely infer a changed implementation; the downstream application goal
    // must reject its changed initial length even on a warm candidate cache.
    check(&crates.build(
        "owned_provider",
        &provider.replace("vec![None;n]", "vec![None;0]"),
        &[],
        &["--output-json"],
    ));
    let run = crates.build(
        "owned_consumer",
        consumer,
        &["owned_provider"],
        &["--output-json"],
    );
    assert!(!run.status.success());
    assert_eq!(json_inference(&run)["status"], "caller_proof_failed");
    assert!(!crates.vir("owned_consumer").exists());
    check(&crates.build("owned_provider", provider, &[], &["--output-json"]));
    // Neither a partial output summary nor an uncertified export is usable.
    let mut bytes = std::fs::read(crates.vir("owned_provider")).unwrap();
    let start = bytes
        .windows(b"{\"schema\":".len())
        .rposition(|s| s == b"{\"schema\":")
        .unwrap();
    bytes.truncate(start);
    std::fs::write(crates.vir("owned_provider"), bytes).unwrap();
    let run = crates.build("owned_consumer", consumer, &["owned_provider"], &[]);
    assert!(!run.status.success());
    assert!(
        String::from_utf8_lossy(&run.stderr)
            .contains("missing or invalid inference proof metadata")
    );
    assert!(!crates.vir("owned_consumer").exists());
    let broken = format!("{provider} pub fn false_goal(){{assert(false);}}");
    let run = crates.build("owned_provider", &broken, &[], &[]);
    assert!(!run.status.success());
    assert!(!crates.vir("owned_provider").exists());
}

#[test]
fn borrowed_result_summaries_cross_private_crate_boundaries() {
    let crates = InferCrates::new();
    let provider = r#"
 use vstd::prelude::*;
 pub struct Slots<T>(Vec<Option<T>>);
 impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}
 pub struct Table<T>{pub slots:Slots<T>,pub ready:bool}
 impl<T> Table<T>{
  #[verifier::infer] pub fn query(&self)->Result<&Slots<T>,u8>{
   if self.ready{Ok(&self.slots)}else{Err(1)}
  }
 }
"#;
    let consumer = r#"
 use vstd::prelude::*;
 #[verifier::infer] pub fn forward<T>(s:&selection_provider::Table<T>)->Result<&selection_provider::Slots<T>,u8>{s.query()}
 pub fn check<T>(s:&selection_provider::Table<T>)->(r:Result<&selection_provider::Slots<T>,u8>)
  ensures r.is_ok()==s.ready,r.is_ok() ==> r.unwrap()@==s.slots@,!s.ready ==> r==Err::<&selection_provider::Slots<T>,u8>(1),{forward(s)}
"#;
    for _ in 0..2 {
        for (name, source, deps) in [
            ("selection_provider", provider, vec![]),
            ("selection_consumer", consumer, vec!["selection_provider"]),
        ] {
            let run = crates.build(name, source, &deps, &["--no-cheating", "--output-json"]);
            assert!(
                run.status.success(),
                "{}",
                String::from_utf8_lossy(&run.stderr)
            );
            let report = json_inference(&run);
            let fs = report["functions"].as_array().unwrap();
            assert_eq!(fs.len(), 1);
            assert_eq!(fs[0]["summary_kind"], "owned_value");
            for p in ["soundness", "maximality", "exactness"] {
                assert_eq!(fs[0]["proofs"][p]["status"], "proved");
            }
        }
    }
    let changed = provider.replace("Err(1)", "Err(2)");
    crates.succeeds("selection_provider", &changed, &[]);
    let run = crates.build(
        "selection_consumer",
        consumer,
        &["selection_provider"],
        &["--output-json"],
    );
    assert!(!run.status.success());
    assert_eq!(json_inference(&run)["status"], "caller_proof_failed");
    assert!(!crates.vir("selection_consumer").exists());
}

#[test]
fn native_clone_only_providers_require_complete_checked_exports() {
    let crates = InferCrates::new();
    let provider = r#"use vstd::prelude::*;
#[verus_verify(clone_via_view)] #[derive(Clone)] pub struct Slots<T>(Vec<Option<T>>);
verus!{impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}}
"#;
    let consumer = r#"use vstd::prelude::*;
use vstd::std_specs::clone::{CloneIdentity,lemma_cloned_identity};
verus!{broadcast use lemma_cloned_identity;}
#[verus_infer] pub fn copy<T:Copy+CloneIdentity>(v:&native_clone_provider::Slots<(T,u16)>)->native_clone_provider::Slots<(T,u16)>{v.clone()}
verus!{pub fn goal(v:&native_clone_provider::Slots<(u64,u16)>)->(r:native_clone_provider::Slots<(u64,u16)>) ensures r@==v@,{copy(v)}}
"#;
    for _ in 0..2 {
        for (name, source, deps) in [
            ("native_clone_provider", provider, vec![]),
            ("native_clone_consumer", consumer, vec!["native_clone_provider"]),
        ] {
            let run=crates.build_source(name,source,&deps,&["--no-cheating","--output-json"],true);
            assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
            assert!(crates.vir(name).exists());
        }
    }
    for opts in [vec!["--no-verify"],vec!["--verify-root","--verify-function","clone"]] {
        let run=crates.build_source("native_clone_provider",provider,&[],&opts,true);
        assert!(!run.status.success());
        assert!(!crates.vir("native_clone_provider").exists());
        assert!(String::from_utf8_lossy(&run.stderr).contains("requires full-crate verification"));
    }
    let broken=format!("{provider} verus!{{pub fn false_goal(){{assert(false);}}}}");
    let run=crates.build_source("native_clone_provider",&broken,&[],&[],true);
    assert!(!run.status.success());
    assert!(!crates.vir("native_clone_provider").exists());
    let false_relation=provider.replace("#[verus_verify(clone_via_view)]","#[verus_verify]").to_owned()
        + "verus!{impl<T:Clone> Slots<T>{pub open spec fn __verus_clone_relation(&self,r:Self)->bool{false}}}";
    let run=crates.build_source("native_clone_provider",&false_relation,&[],&[],true);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("postcondition not satisfied"));
    assert!(!crates.vir("native_clone_provider").exists());
    let run=crates.build_source("native_clone_provider",provider,&[],&[],true);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let mut bytes=std::fs::read(crates.vir("native_clone_provider")).unwrap();
    let start=bytes.windows(b"{\"schema\":".len()).rposition(|s|s==b"{\"schema\":").unwrap();
    bytes.truncate(start);
    std::fs::write(crates.vir("native_clone_provider"),bytes).unwrap();
    let run=crates.build_source("native_clone_consumer",consumer,&["native_clone_provider"],&[],true);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("missing or invalid inference proof metadata"));
    assert!(!crates.vir("native_clone_consumer").exists());
}

#[test]
fn private_view_bounds_are_checked_and_selected_only_for_demanded_goals() {
    let crates=InferCrates::new();
    let provider=r#"use vstd::prelude::*;
#[verus_verify(clone_via_view)] #[derive(Clone)] pub struct Slots<T>(Vec<Option<T>>);
verus!{impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}}
"#;
    let consumer=r#"use vstd::prelude::*;verus!{
 fn bump(v:&view_bound_provider::Slots<u8>,n:usize)->(r:usize)
 requires n<v@.len(),ensures r as int==n as int+1,{n+1}
 fn unrelated(n:usize)->usize{n}
}"#;
    let run=crates.build_source("view_bound_provider",provider,&[],&["--no-cheating"],true);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let run=crates.build_source("view_bound_consumer",consumer,&["view_bound_provider"],&["--no-cheating"],true);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("overflow"));
    for _ in 0..2 {
        let run=crates.build_source("view_bound_consumer",consumer,&["view_bound_provider"],&["--infer-goals","--no-cheating","--output-json"],true);
        assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
        let report=json_inference(&run);
        let assists=report["goal_assistance"].as_array().unwrap();
        assert_eq!(assists.len(),1,"{report}");
        assert_eq!(assists[0]["strategy"],"sequence_view_bound");
        assert!(assists[0]["function"].as_str().unwrap().ends_with("::bump"));
    }
    let wrong_view=provider.replace("{self.0@}","{Seq::new(usize::MAX as nat+1,|i:int|None)}");
    let run=crates.build_source("view_bound_provider",&wrong_view,&[],&["--no-cheating"],true);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("postcondition not satisfied"));
    assert!(!crates.vir("view_bound_provider").exists());
    // A same-named vacuous theorem does not satisfy the selection interface.
    let fake=r#"use vstd::prelude::*;verus!{
 pub struct Slots<T>(Vec<Option<T>>);
 impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}
 impl<T> Slots<T>{pub broadcast proof fn __verus_infer_view_len_bound(&self)
  requires false,ensures #[trigger] self@.len()<=usize::MAX as int,{}}
}"#;
    let run=crates.build_source("view_bound_provider",fake,&[],&[],true);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let run=crates.build_source("view_bound_consumer",consumer,&["view_bound_provider"],&["--infer-goals","--output-json"],true);
    assert!(!run.status.success());
    assert!(json_inference(&run)["goal_assistance"].as_array().unwrap().is_empty());
}

#[test]
fn generic_error_values_and_propagation_cross_certified_crate_boundaries() {
    let crates=InferCrates::new();
    let provider=r#"use vstd::prelude::*;
pub enum Error<T:Copy>{Pair(T,u16),Other}
#[verifier::infer] pub fn leaf<T:Copy>(fail:bool,value:T,round:u16)->Result<(),Error<T>>{
 if fail{return Err(Error::Pair(value,round));}Ok(())
}"#;
    let consumer=r#"use vstd::prelude::*;
#[verifier::infer] pub fn bridge<T:Copy>(fail:bool,value:T,round:u16)->Result<u16,error_provider::Error<T>>{
 error_provider::leaf(fail,value,round)?;Ok(round+1)
}

pub fn goal(value:u64)->(result:Result<u16,error_provider::Error<u64>>)
 ensures result==Err::<u16,error_provider::Error<u64>>(error_provider::Error::Pair(value,u16::MAX)),
{bridge(true,value,u16::MAX)}
"#;
    for _ in 0..2 {
        for (name,body,deps) in [("error_provider",provider,vec![]),("error_consumer",consumer,vec!["error_provider"])] {
            let run=crates.build(name,body,&deps,&["--no-cheating","--output-json"]);
            assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
            let report=json_inference(&run);
            let fs=report["functions"].as_array().unwrap();assert_eq!(fs.len(),1);
            for check in ["soundness","maximality","exactness"]{assert_eq!(fs[0]["proofs"][check]["status"],"proved");}
        }
    }
    let changed=provider.replace("Err(Error::Pair(value,round))","Err(Error::Other)");
    let run=crates.build("error_provider",&changed,&[],&["--no-cheating"]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let run=crates.build("error_consumer",consumer,&["error_provider"],&["--no-cheating","--output-json"]);
    assert!(!run.status.success());assert!(!crates.vir("error_consumer").exists());
    assert_eq!(json_inference(&run)["status"],"caller_proof_failed");
    assert!(String::from_utf8_lossy(&run.stderr).contains("postcondition not satisfied"));
}

#[test]
fn ordinary_static_queries_require_checked_exports_and_compose_across_private_views(){
 let crates=InferCrates::new();
 let provider=r#"use vstd::prelude::*;
 pub struct Slots<T>(Vec<T>);
 impl<T> View for Slots<T>{type V=Seq<T>;closed spec fn view(&self)->Self::V{self.0@}}
 impl Slots<u8>{
  pub closed spec fn length_value(&self)->usize{self.0@.len() as usize}
  pub fn length_checked(&self)->(r:usize) ensures r==self.length_value(),{self.0.len()}
 }
 "#;
 let consumer=r#"use vstd::prelude::*;pub struct State{pub slots:query_provider::Slots<u8>}
 pub enum Error{First,Second}
 #[verifier::infer]pub fn query(s:&State,n:usize)->Result<usize,Error>{Ok(s.slots.length_checked()+n)}
 pub fn goal(s:&State)->(r:Result<usize,Error>) ensures r==Ok::<usize,Error>(s.slots.length_value()),{query(s,0)}
 "#;
 for _ in 0..2{
  let r=crates.build("query_provider",provider,&[],&["--infer-goals","--no-cheating","--output-json"]);
  assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
  assert!(json_inference(&r)["functions"].as_array().unwrap().is_empty());
  let r=crates.build("query_consumer",consumer,&["query_provider"],&["--no-cheating","--output-json"]);
  assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
  assert_eq!(json_inference(&r)["functions"][0]["proofs"]["exactness"]["status"],"proved");
 }
 let r=crates.build("query_provider",provider,&[],&["--infer-goals","--no-verify"]);
 assert!(!r.status.success());assert!(!crates.vir("query_provider").exists());
 assert!(String::from_utf8_lossy(&r.stderr).contains("requires full-crate verification"));
 let broken=provider.replace("{self.0.len()}","{0}");
 let r=crates.build("query_provider",&broken,&[],&["--infer-goals","--no-cheating"]);
 assert!(!r.status.success());assert!(!crates.vir("query_provider").exists());
 assert!(String::from_utf8_lossy(&r.stderr).contains("postcondition not satisfied"));
}

#[path="infer_cases/native_iterators.rs"]
mod native_iterator_cases;

#[path="infer_cases/equality.rs"]
mod equality_cases;

#[path="infer_cases/result_composition.rs"]
mod result_composition_cases;

#[path="infer_cases/result_map.rs"]
mod result_map_cases;

#[path="infer_cases/borrowed_objects.rs"]
mod borrowed_object_cases;

#[path="infer_cases/factory_goals.rs"]
mod factory_goal_cases;

#[path="infer_cases/outer_validation.rs"]
mod outer_validation_cases;

#[test]
fn pointwise_record_loops_cross_crates_reprove_and_reject_false_callers() {
    let crates = InferCrates::new();
    let base = r#"
        use vstd::prelude::*;
        pub struct Bag { pub values: Vec<u8>, pub stamp: u8 }
        impl Bag {
            #[verifier::infer] pub fn bump(&mut self, n: usize) {
                let mut i=0usize;
                while i<n { self.values[i]+=1; i+=1; }
            }
        }
    "#;
    for _ in 0..2 {
        let run = crates.build(
            "prefix_base",
            base,
            &[],
            &["--no-cheating", "--output-json"],
        );
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        let report = json_inference(&run);
        let f = &report["functions"][0];
        assert_eq!(f["metrics"]["cache_policy"], "regenerated_state");
        assert_eq!(f["loop_proofs"].as_array().unwrap().len(), 1);
        for p in ["soundness", "maximality", "exactness"] {
            assert_eq!(f["proofs"][p]["status"], "proved");
            assert_eq!(f["proofs"][p]["required"], true);
        }
    }
    let middle = r#"
        use vstd::prelude::*;
        #[verifier::infer] pub fn relay(b:&mut prefix_base::Bag,n:usize) {b.bump(n);}
    "#;
    crates.succeeds("prefix_middle", middle, &["prefix_base"]);
    let app = r#"
        use vstd::prelude::*;
        pub fn check(b:&mut prefix_base::Bag,n:usize)
            requires n<=old(b).values@.len(),
                forall|i:int|0<=i<n ==> old(b).values@[i]<255,
            ensures final(b).stamp==old(b).stamp,
                final(b).values@.len()==old(b).values@.len(),
                forall|i:int|0<=i<old(b).values@.len() ==>
                    #[trigger] final(b).values@[i] as int == if i<n {old(b).values@[i]+1} else {old(b).values@[i] as int},
        {prefix_middle::relay(b,n);}
    "#;
    let run = crates.build(
        "prefix_app",
        app,
        &["prefix_base", "prefix_middle"],
        &["--infer-goals"],
    );
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    for wrong in [
        app.replace("old(b).values@[i]<255", "old(b).values@[i]<=255"),
        app.replace("old(b).values@[i]+1", "old(b).values@[i]+2"),
    ] {
        let run = crates.build(
            "prefix_app",
            &wrong,
            &["prefix_base", "prefix_middle"],
            &["--infer-goals", "--output-json"],
        );
        assert!(!run.status.success());
        assert!(!crates.vir("prefix_app").exists());
        assert_eq!(json_inference(&run)["interface_written"], false);
    }
}

#[test]
fn ordinary_growth_proofs_cross_crates_require_full_checks_and_block_failed_exports() {
    let crates = InferCrates::new();
    let base = r#"use vstd::prelude::*;
        pub struct History {pub values:Vec<u8>,pub tag:u8}
        pub fn grow(h:&mut History,n:usize)
            ensures final(h).values@.len()==if old(h).values@.len()>n {old(h).values@.len() as int}else{n as int},
                final(h).values@.take(old(h).values@.len() as int)==old(h).values@,
                final(h).tag==old(h).tag,
                forall|i:int|old(h).values@.len()<=i<final(h).values@.len() ==> #[trigger] final(h).values@[i]==0,
        {while h.values.len()<n {h.values.push(0);}}
    "#;
    for _ in 0..2 {
        let r = crates.build(
            "growth_base",
            base,
            &[],
            &["--infer-goals", "--no-cheating", "--output-json"],
        );
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
        let report = json_inference(&r);
        assert_eq!(
            report["growth_loop_assistance"].as_array().unwrap().len(),
            1
        );
        assert!(report["functions"].as_array().unwrap().is_empty());
        assert_eq!(report["interface_written"], true);
    }
    let app = r#"use vstd::prelude::*;
        pub fn goal(h:&mut growth_base::History,n:usize)
            ensures final(h).tag==old(h).tag, final(h).values@.len()>=n,
        {growth_base::grow(h,n);}
    "#;
    crates.succeeds("growth_app", app, &["growth_base"]);
    let bad = app.replace("final(h).values@.len()>=n", "final(h).values@.len()<n");
    let r = crates.build(
        "growth_app",
        &bad,
        &["growth_base"],
        &["--infer-goals", "--output-json"],
    );
    assert!(!r.status.success());
    assert!(!crates.vir("growth_app").exists());
    for flag in ["--no-verify", "--verify-root"] {
        let r = crates.build("growth_base", base, &[], &["--infer-goals", flag]);
        assert!(!r.status.success());
        assert!(!crates.vir("growth_base").exists());
        assert!(String::from_utf8_lossy(&r.stderr).contains("requires full-crate verification"));
    }
    let bad = base.replace("push(0)", "push(1)");
    let r = crates.build(
        "growth_base",
        &bad,
        &[],
        &["--infer-goals", "--no-cheating", "--output-json"],
    );
    assert!(!r.status.success());
    assert!(!crates.vir("growth_base").exists());
    assert_eq!(json_inference(&r)["interface_written"], false);
}

#[test]
fn ordinary_range_proofs_cross_crates_require_full_checks_and_block_failed_exports() {
    let crates=InferCrates::new();
    let base=r#"use vstd::prelude::*;
        pub struct Buffer {pub values:Vec<u8>,pub stamp:u8}
        pub fn fill(b:&mut Buffer,start:u16,end:u16)
          requires end<old(b).values@.len(),
          ensures final(b).values@.len()==old(b).values@.len(),final(b).stamp==old(b).stamp,
            forall|i:int|start<=i<=end ==> #[trigger] final(b).values@[i]==1,
            forall|i:int|0<=i<old(b).values@.len() && (i<start || i>end) ==> #[trigger] final(b).values@[i]==old(b).values@[i],
        {for i in start..=end {b.values[i as usize]=1;}}
    "#;
    for _ in 0..2 {
        let r=crates.build("range_base",base,&[],&["--infer-goals","--no-cheating","--output-json"]);
        assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));
        let report=json_inference(&r);
        assert_eq!(report["range_loop_assistance"].as_array().unwrap().len(),1);
        assert!(report["functions"].as_array().unwrap().is_empty());
        assert_eq!(report["interface_written"],true);
    }
    let app=r#"use vstd::prelude::*;
        pub fn goal(b:&mut range_base::Buffer)
          requires old(b).values@.len()>255,
          ensures final(b).values@[255]==1,final(b).stamp==old(b).stamp,
        {range_base::fill(b,255,255);}
    "#;
    crates.succeeds("range_app",app,&["range_base"]);
    for flag in ["--no-verify","--verify-root"] {
        let r=crates.build("range_base",base,&[],&["--infer-goals",flag]);
        assert!(!r.status.success());
        assert!(!crates.vir("range_base").exists());
        assert!(String::from_utf8_lossy(&r.stderr).contains("requires full-crate verification"));
    }
    let r=crates.build("range_base",&base.replace("=1;","=2;"),&[],&["--infer-goals","--no-cheating","--output-json"]);
    assert!(!r.status.success());
    assert!(!crates.vir("range_base").exists());
    assert_eq!(json_inference(&r)["interface_written"],false);
}

#[test]
fn ordinary_mutable_selectors_cross_crates_require_complete_checked_publication() {
    let crates=InferCrates::new();
    let provider=r#"use vstd::prelude::*;
        pub struct Pool {pub values:Vec<u8>,pub tag:u8}
        pub fn prepare(p:&mut Pool,i:usize)
          requires i<old(p).values@.len(),
          ensures final(p).tag==old(p).tag,
            final(p).values@.len()==old(p).values@.len(),
            final(p).values@.take(old(p).values@.len() as int)==old(p).values@,
        {}
        pub fn select(p:&mut Pool,i:usize)->(r:&mut u8)
          requires i<old(p).values@.len(),
          ensures *r==old(p).values@[i as int],
        {prepare(p,i);&mut p.values[i]}
    "#;
    for _ in 0..2 {
        let run=crates.build("selector_base",provider,&[],&["--infer-goals","--no-cheating","--output-json"]);
        assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
        let report=json_inference(&run);
        assert_eq!(report["mutable_selector_assistance"].as_array().unwrap().len(),1);
        assert!(report["functions"].as_array().unwrap().is_empty());
        assert_eq!(report["interface_written"],true);
    }
    let client=r#"use vstd::prelude::*;
        pub fn set(p:&mut selector_base::Pool,i:usize,value:u8)
          requires i<old(p).values@.len(),
          ensures final(p).values@[i as int]==value,final(p).tag==old(p).tag,
            final(p).values@.len()==old(p).values@.len(),
            forall|j:int|0<=j<old(p).values@.len() && j!=i ==> final(p).values@[j]==old(p).values@[j],
        {let r=selector_base::select(p,i);*r=value;}
    "#;
    crates.succeeds("selector_client",client,&["selector_base"]);
    let bad=client.replace("*r=value;","*r=0;");
    let run=crates.build("selector_client",&bad,&["selector_base"],&["--no-cheating","--output-json"]);
    assert!(!run.status.success());
    assert!(!crates.vir("selector_client").exists());
    assert_eq!(json_inference(&run)["interface_written"],false);
    for flag in ["--no-verify","--verify-root"] {
        let run=crates.build("selector_client",client,&["selector_base"],&[flag]);
        assert!(!run.status.success());
        assert!(!crates.vir("selector_client").exists());
        assert!(String::from_utf8_lossy(&run.stderr).contains("requires full-crate verification"));
    }
    crates.succeeds("selector_client",client,&["selector_base"]);
    let leaf=r#"use vstd::prelude::*;
        pub fn goal(p:&mut selector_base::Pool,i:usize,value:u8)
          requires i<old(p).values@.len(),
          ensures final(p).values@[i as int]==value,final(p).tag==old(p).tag,
        {selector_client::set(p,i,value);}
    "#;
    crates.succeeds("selector_leaf",leaf,&["selector_base","selector_client"]);
    // This intermediate API contains no inferred functions or selectors. Its
    // header must still require its transitive proof/dependency trailer.
    let path=crates.vir("selector_client");
    let mut bytes=std::fs::read(&path).unwrap();
    let start=bytes.windows(b"{\"schema\":".len()).rposition(|s|s==b"{\"schema\":").unwrap();
    bytes.truncate(start);
    std::fs::write(&path,bytes).unwrap();
    let run=crates.build("selector_leaf",leaf,&["selector_base","selector_client"],&["--output-json"]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("inference proof metadata"));
    assert!(!crates.vir("selector_leaf").exists());
    for flags in [vec!["--no-verify"],vec!["--verify-root"],vec!["--verify-root","--verify-function","select"]] {
        let mut args=vec!["--infer-goals"];
        args.extend(flags);
        let run=crates.build("selector_base",provider,&[],&args);
        assert!(!run.status.success());
        assert!(!crates.vir("selector_base").exists());
        assert!(String::from_utf8_lossy(&run.stderr).contains("requires full-crate verification"),"{}",String::from_utf8_lossy(&run.stderr));
    }
    let run=crates.build("selector_base",&provider.replace("&mut p.values[i]","&mut p.values[0]"),&[],&["--infer-goals","--no-cheating","--output-json"]);
    assert!(!run.status.success());
    assert!(!crates.vir("selector_base").exists());
    assert_eq!(json_inference(&run)["interface_written"],false);
    let run=crates.build("selector_base",provider,&[],&["--infer-goals","--no-cheating"]);
    assert!(run.status.success(),"{}",String::from_utf8_lossy(&run.stderr));
    let path=crates.vir("selector_base");
    let mut bytes=std::fs::read(&path).unwrap();
    bytes.truncate(bytes.len()-8);
    std::fs::write(&path,bytes).unwrap();
    let run=crates.build("selector_client",client,&["selector_base"],&["--output-json"]);
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("inference proof metadata"));
    assert!(!crates.vir("selector_client").exists());
}

#[test]
fn native_binary_wrappers_compose_through_private_queries_and_checked_exports() {
    let crates=InferCrates::new();
    let provider=r#"use vstd::prelude::*;
#[verus_verify] #[derive(Copy,Clone)] pub struct Amount(pub usize);
#[verus_infer] impl From<usize> for Amount {fn from(n:usize)->Self{Amount(n)}}
#[verus_infer] impl core::ops::Mul<usize> for Amount {type Output=Self;fn mul(self,n:usize)->Self{Amount(self.0*n)}}
#[verus_infer] impl core::ops::Div<usize> for Amount {type Output=Self;fn div(self,n:usize)->Self{Amount(self.0/n)}}
#[verus_infer] impl core::ops::Add for Amount {type Output=Self;fn add(self,n:Self)->Self{Amount(self.0+n.0)}}
#[verus_infer] pub fn quota(n:&Amount)->Amount {(*n*2)/3+Amount(1)}
"#;
    let consumer=r#"use vstd::prelude::*;use binary_base::Amount;
#[verus_verify] pub struct Slots<T>(Vec<Option<T>>);
verus!{impl<T> View for Slots<T>{type V=Seq<Option<T>>;closed spec fn view(&self)->Self::V{self.0@}}}
impl<T> Slots<T>{#[verus_infer] pub fn size(&self)->Amount{self.0.len().into()}}
verus!{
 pub fn goal<T>(s:&Slots<T>)->(r:Amount) requires s@.len()<=usize::MAX/2,
 ensures r.0==(2*s@.len())/3+1,
 {let size=s.size();binary_base::quota(&size)}
}
"#;
    for _ in 0..2 {
        let p=crates.build_source("binary_base",provider,&[],&["--no-cheating","--output-json"],true);
        assert!(p.status.success(),"{}",String::from_utf8_lossy(&p.stderr));
        assert_eq!(json_inference(&p)["functions"].as_array().unwrap().len(),5);
        let c=crates.build_source("binary_app",consumer,&["binary_base"],&["--no-cheating","--output-json"],true);
        assert!(c.status.success(),"{}",String::from_utf8_lossy(&c.stderr));
        assert_eq!(json_inference(&c)["functions"].as_array().unwrap().len(),1);
    }
    let bad=consumer.replace("r.0==(2*s@.len())/3+1","r.0==(2*s@.len())/3+2");
    let c=crates.build_source("binary_app",&bad,&["binary_base"],&["--no-cheating","--output-json"],true);
    assert!(!c.status.success());assert!(!crates.vir("binary_app").exists());
    assert!(json_inference(&c)["functions"].as_array().unwrap().iter().all(|f|f["required_checks_passed"]==true));
    for opts in [vec!["--no-verify"],vec!["--verify-root","--verify-function","quota"]] {
        let p=crates.build_source("binary_base",provider,&[],&opts,true);
        assert!(!p.status.success());assert!(!crates.vir("binary_base").exists());
        assert!(String::from_utf8_lossy(&p.stderr).contains("requires full-crate verification"));
    }
    let wrong=provider.replace("Amount(self.0+n.0)","Amount(self.0-n.0)");
    let p=crates.build_source("binary_base",&wrong,&[],&["--no-cheating","--output-json"],true);
    assert!(p.status.success(),"{}",String::from_utf8_lossy(&p.stderr));
    let c=crates.build_source("binary_app",consumer,&["binary_base"],&["--no-cheating","--output-json"],true);
    assert!(!c.status.success());assert!(!crates.vir("binary_app").exists());
}

#[test]
fn derived_ordering_without_exact_targets_is_certified_and_invalidates_stale_exports(){
    let crates=InferCrates::new();
    let provider=r#"use vstd::prelude::*;
#[verus_verify(infer_derive(Ord))]
#[derive(Copy,Clone,Eq,PartialEq,PartialOrd,Ord)] pub struct Amount(pub u8);
"#;
    let caller=r#"use vstd::prelude::*;use ordering_base::Amount;
verus!{pub fn goal(a:Amount,b:Amount){let r=a.cmp(&b);
 assert((r==core::cmp::Ordering::Less)==(a.0<b.0));}}
"#;
    for _ in 0..2 {
        let p=crates.build_source("ordering_base",provider,&[],&["--no-cheating","--output-json"],true);
        assert!(p.status.success(),"{}",String::from_utf8_lossy(&p.stderr));
        assert!(json_inference(&p)["functions"].as_array().unwrap().is_empty());
        assert!(std::fs::read(crates.vir("ordering_base")).unwrap().ends_with(b"VRINFER1"));
        let c=crates.build_source("ordering_app",caller,&["ordering_base"],&["--no-cheating"],true);
        assert!(c.status.success(),"{}",String::from_utf8_lossy(&c.stderr));
    }
    let prior_client=std::fs::read(crates.vir("ordering_app")).unwrap();
    assert!(prior_client.ends_with(b"VRINFER1"));
    crates.edit_metadata("ordering_base",|m|{assert!(m["proofs"].as_array().unwrap().is_empty());m["soundness"]=false.into();});
    let c=crates.build_source("ordering_app",caller,&["ordering_base"],&["--no-cheating"],true);
    assert!(!c.status.success());
    // Ordinary consumers now carry the dependency certificate too. A failed
    // import must invalidate that old API before any replacement is published.
    assert!(!crates.vir("ordering_app").exists());
    for opts in [vec!["--no-verify"],vec!["--verify-root","--verify-function","cmp"]] {
        let good=crates.build_source("ordering_base",provider,&[],&["--no-cheating"],true);
        assert!(good.status.success(),"{}",String::from_utf8_lossy(&good.stderr));
        let p=crates.build_source("ordering_base",provider,&[],&opts,true);
        assert!(!p.status.success());assert!(!crates.vir("ordering_base").exists());
        assert!(String::from_utf8_lossy(&p.stderr).contains("requires full-crate verification"));
    }
}

#[test]
fn native_derived_comparisons_compose_across_crates_and_reject_wrong_client_goals(){
    let crates=InferCrates::new();
    let provider=r#"use vstd::prelude::*;
#[verus_verify(infer_derive(PartialEq,PartialOrd,Ord))]
#[derive(Copy,Clone,Eq,PartialEq,PartialOrd,Ord)] pub struct Amount(pub i16);
"#;
    let caller=r#"use vstd::prelude::*;use compare_base::Amount;
#[verus_infer] pub fn earlier(a:Amount,b:Amount)->bool{a<b}
verus!{pub fn goal(a:Amount,b:Amount){let r=earlier(a,b);assert(r==(a.0<b.0));}}
"#;
    for _ in 0..2 {
        let p=crates.build_source("compare_base",provider,&[],&["--no-cheating","--output-json"],true);
        assert!(p.status.success(),"{}",String::from_utf8_lossy(&p.stderr));
        let c=crates.build_source("compare_app",caller,&["compare_base"],&["--no-cheating","--output-json"],true);
        assert!(c.status.success(),"{}",String::from_utf8_lossy(&c.stderr));
        assert_eq!(json_inference(&c)["functions"][0]["required_checks_passed"],true);
    }
    let bad=caller.replace("assert(r==(a.0<b.0))","assert(r==(a.0>b.0))");
    let c=crates.build_source("compare_app",&bad,&["compare_base"],&["--no-cheating","--output-json"],true);
    assert!(!c.status.success());assert!(!crates.vir("compare_app").exists());
    assert_eq!(json_inference(&c)["functions"][0]["required_checks_passed"],true);
}
