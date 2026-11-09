#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const MANUAL: &str = include_str!("../../tools/research/varint_canonical_goal/candidate-004.rs");
const AUTOMATIC: &str = include_str!("../../tools/research/varint_canonical_goal/candidate-008.rs");

fn check(name: &str, text: String, goals: bool, success: bool) {
    let options = if goals {
        vec!["vstd", "--infer-goals", "--no-cheating"]
    } else {
        vec!["vstd", "--no-cheating"]
    };
    let text = text.replace("#![feature(proc_macro_hygiene)]", "");
    let result = verify_one_file(name, text, &options);
    assert_eq!(result.is_ok(), success, "{name}: {result:?}");
}

#[test]
fn ordinary_canonical_every_u64_arbitrary_initialized_buffer() {
    check("ordinary_canonical", MANUAL.into(), false, true);
}
#[test]
fn assisted_canonical_every_u64_arbitrary_initialized_buffer() {
    check("assisted_canonical", AUTOMATIC.into(), true, true);
}
#[test]
fn disabled_assistance_rejects() {
    check("disabled", AUTOMATIC.into(), false, false);
}
#[test]
fn safety_length_is_a_separately_named_result() {
    check(
        "safety_length",
        AUTOMATIC
            .replace("canonical(number, encoded@)", "1 <= encoded@.len() <= 10")
            .replace("pub fn u64(", "pub fn safety_length("),
        true,
        true,
    );
}
#[test]
fn zero_early_break_and_maximum_input_callers() {
    let caller = r#"
verus! {
    fn extremes(a: &mut [u8;10], b: &mut [u8;10]) {
        let z = u64(0,a);
        assert(z@.len()==1);
        assert(z@[0]==0);
        let m = u64(u64::MAX,b);
        assert(canonical(u64::MAX,m@));
    }
}
"#;
    check("extremes", format!("{AUTOMATIC}\n{caller}"), true, true);
}
#[test]
fn wrong_source_steps_and_effects_reject() {
    for (name, from, to) in [
        ("wrong_shift", "n >>= 7;", "n >>= 6;"),
        ("wrong_break", "if n == 0", "if n == 1"),
        ("wrong_counter", "i += 1", "i += 2"),
        ("wrong_payload", "*b = n as u8 | 0x80;", "*b = 0x80;"),
        ("wrong_continuation", "*b &= 0x7f;", "*b &= 0xff;"),
        ("wrong_range", "&$buf[0..=i]", "&$buf[0..i]"),
    ] {
        check(name, AUTOMATIC.replace(from, to), true, false);
    }
}
#[test]
fn unsupported_width_step_and_capacity_do_not_claim_generality() {
    // The first profile is explicitly u64/7/[u8;10]. Unsupported shapes retain
    // ordinary obligations; these are not proofs of semantic negatives.
    for (name, text) in [
        ("capacity9", AUTOMATIC.replace("U64_LEN: usize = 10", "U64_LEN: usize = 9")),
        ("step8", AUTOMATIC.replace("n >>= 7;", "n >>= 8;")),
        ("width32", AUTOMATIC.replace("number: u64", "number: u32")),
    ] {
        check(name, text, true, false);
    }
}
