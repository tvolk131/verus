#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

fn one_error(e: TestErr, expected: &str) {
    assert_eq!(e.errors.len(), 1);
    assert!(e.errors[0].code.is_none());
    assert!(e.errors[0].message.contains(expected));
}

const ZIP: &str = include_str!("../fixtures/infer/buffer_observation/zip_prefix.rs");
const PLAIN: &str = include_str!("../fixtures/infer/buffer_observation/plain.rs");

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
    #[test] surrounding_direct_writes_are_not_decorated ["vstd", "--infer-goals", "--no-cheating"] => code! {
        verus! {
            fn before(buf: &mut [u8; 10]) {
                buf[0] = 42;
                for b in buf.iter_mut() { *b = 1; }
            }
            fn after(buf: &mut [u8; 10]) {
                for b in buf.iter_mut() { *b = 1; }
                buf[0] = 42;
            }
        }
    } => Ok(())
}
test_verify_one_file_with_options! {
    #[test] surrounding_helper_and_alias_writes_are_not_decorated ["vstd", "--infer-goals", "--no-cheating"] => code! {
        verus! {
            fn write(buf: &mut [u8; 10]) { buf[0] = 42; }
            fn before(buf: &mut [u8; 10]) {
                write(buf);
                for b in buf.iter_mut() { *b = 1; }
            }
            fn after(buf: &mut [u8; 10]) {
                for b in buf.iter_mut() { *b = 1; }
                let alias = &mut *buf;
                write(alias);
            }
        }
    } => Ok(())
}
test_verify_one_file_with_options! {
    #[test] escaping_mutable_buffer_is_not_decorated ["vstd", "--infer-goals", "--no-cheating"] => code! {
        verus! {
            fn escape(buf: &mut [u8; 10]) -> &mut [u8; 10] {
                for b in buf.iter_mut() { *b = 1; }
                buf
            }
        }
    } => Ok(())
}
test_verify_one_file_with_options! {
    #[test] unrelated_buffer_writes_keep_prefix_assistance ["vstd", "--infer-goals", "--no-cheating"] =>
        ZIP.replace("key: &[u8])", "key: &[u8], other: &mut [u8; 10])")
            .replace("    for (b_i, b_h)", "    other[0] = 42;\n    for (b_i, b_h)")
            .replace("        *b_i ^= *b_h;\n    }", "        *b_i ^= *b_h;\n    }\n    other[0] = 43;") => Ok(())
}
test_verify_one_file_with_options! {
    #[test] declaration_move_alias_is_not_decorated ["vstd", "--infer-goals", "--no-cheating"] =>
        include_str!("../fixtures/infer/buffer_observation/alias_write_after_loop.rs").to_string() => Ok(())
}
test_verify_one_file_with_options! {
    #[test] mutable_pattern_before_is_not_decorated ["vstd", "--infer-goals", "--no-cheating"] =>
        include_str!("../fixtures/infer/buffer_observation/pattern_alias_before_loop.rs").to_string() => Ok(())
}
test_verify_one_file_with_options! {
    #[test] mutable_pattern_after_is_not_decorated ["vstd", "--infer-goals", "--no-cheating"] =>
        include_str!("../fixtures/infer/buffer_observation/pattern_alias_after_loop.rs").to_string() => Ok(())
}
test_verify_one_file_with_options! {
    #[test] match_move_alias_is_not_decorated ["vstd", "--infer-goals", "--no-cheating"] =>
        include_str!("../fixtures/infer/buffer_observation/match_alias_after_loop.rs").to_string() => Ok(())
}
test_verify_one_file_with_options! {
    #[test] readonly_target_value_keeps_prefix_assistance ["vstd", "--infer-goals", "--no-cheating"] =>
        ZIP.replace("    for (b_i, b_h)", "    let _first = ipad[0];\n    for (b_i, b_h)") => Ok(())
}
const WRONG_LOOP: &str =
    include_str!("../fixtures/infer/buffer_observation/unrelated_loop_origin.rs");
test_verify_one_file_with_options! {
    #[test] reassigned_zip_key_local_is_not_decorated ["vstd", "--infer-goals", "--no-cheating"] =>
        include_str!("../fixtures/infer/buffer_observation/reassigned_local_key.rs").to_string() => Ok(())
}
test_verify_one_file_with_options! {
    #[test] reassigned_zip_key_parameter_is_not_decorated ["vstd", "--infer-goals", "--no-cheating"] =>
        include_str!("../fixtures/infer/buffer_observation/reassigned_parameter_key.rs").to_string() => Ok(())
}
test_verify_one_file_with_options! {
    #[test] executable_zip_key_is_not_decorated ["vstd", "--infer-goals", "--no-cheating"] =>
        include_str!("../fixtures/infer/buffer_observation/helper_key.rs").to_string() => Ok(())
}
test_verify_one_file_with_options! {
    #[test] unrelated_constructor_is_not_the_loop_origin ["vstd", "--infer-goals", "--no-cheating"] =>
        WRONG_LOOP.to_string() => Ok(())
}
test_verify_one_file_with_options! {
    #[test] unrelated_constructor_inside_iterator_expression_is_not_the_loop_origin ["vstd", "--infer-goals", "--no-cheating"] =>
        WRONG_LOOP.replace("    let _unused = buf.iter_mut();\n    for b in make(other)",
            "    for b in { let _unused = buf.iter_mut(); make(other) }") => Ok(())
}
test_verify_one_file_with_options! {
    #[test] full_suffix_remains_unproved ["vstd", "--infer-goals", "--no-cheating"] =>
        include_str!("../fixtures/infer/buffer_observation/full_suffix_unproved.rs").to_string() => Err(e) => one_error(e, "postcondition not satisfied")
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
