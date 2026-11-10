#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

fn one_error(e: TestErr, expected: &str) {
    assert_eq!(e.errors.len(), 1);
    assert!(e.errors[0].code.is_none());
    assert!(e.errors[0].message.contains(expected));
}

const ZIP: &str = include_str!("../../tools/research/buffer_observation_goal/prefix-zip.rs");
const PLAIN: &str = include_str!("../../tools/research/buffer_observation_goal/plain.rs");

test_verify_one_file_with_options! {
    #[test] zip_prefix_checked ["vstd", "--infer-goals", "--no-cheating"] => ZIP.to_string() => Ok(())
}
test_verify_one_file_with_options! {
    #[test] plain_checked ["vstd", "--infer-goals", "--no-cheating"] => PLAIN.to_string() => Ok(())
}
test_verify_one_file_with_options! {
    #[test] zip_disabled ["vstd", "--no-cheating"] => ZIP.to_string() => Err(e) => one_error(e, "postcondition not satisfied")
}
test_verify_one_file_with_options! {
    #[test] plain_disabled ["vstd", "--no-cheating"] => PLAIN.to_string() => Ok(())
}
test_verify_one_file_with_options! {
    #[test] full_suffix_remains_unproved ["vstd", "--infer-goals", "--no-cheating"] =>
        include_str!("../../tools/research/buffer_observation_goal/full-zip.rs").to_string() => Err(e) => one_error(e, "postcondition not satisfied")
}
test_verify_one_file_with_options! {
    #[test] two_matching_functions ["vstd", "--infer-goals", "--no-cheating"] => {
        let second=PLAIN.replace("pub fn plain", "pub fn second");
        format!("{}\n{}", PLAIN.replace("fn main() {}", ""), second.replace("use vstd::prelude::*;", ""))
    } => Ok(())
}
test_verify_one_file_with_options! {
    #[test] wrong_value ["vstd", "--infer-goals", "--no-cheating"] =>
        ZIP.replace("old(ipad)@[j] ^ key@[j]", "old(ipad)@[j] | key@[j]") => Err(e) => one_error(e, "postcondition not satisfied")
}
test_verify_one_file_with_options! {
    #[test] safety_preserved ["vstd", "--infer-goals", "--no-cheating"] => code! {
        verus! { fn overflow(buf: &mut [u8; 10]) { for b in buf.iter_mut() { *b += 1; } } }
    } => Err(e) => one_error(e, "possible arithmetic underflow/overflow")
}
test_verify_one_file_with_options! {
    #[test] custom_lookalike_is_not_native ["vstd", "--infer-goals", "--no-cheating"] => code! {
        use vstd::prelude::*;
        verus! {
            struct Custom { value: [u8; 10] }
            impl Custom {
                fn iter__mut(&mut self) -> (r: &[u8; 10]) { &self.value }
            }
            fn client(c: &mut Custom) { let r=c.iter__mut(); assert(r@[0] == 7); } // FAILS
        }
    } => Err(e) => one_error(e, "assertion failed")
}
