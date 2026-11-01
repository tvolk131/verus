#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

const COUNT: &str = include_str!("../../../examples/guide/macro_loop_adapter.rs");

test_verify_one_file_with_options! {
    #[test] macro_loop_adapter_count ["no-auto-import-verus_builtin", "--no-cheating"]
    => COUNT.to_string() => Ok(())
}

test_verify_one_file_with_options! {
    #[test] macro_loop_adapter_wrong_result ["no-auto-import-verus_builtin", "--no-cheating"] =>
    COUNT.replace("    result\n}", "    result ^ 1\n}")
    => Err(err) => {
        assert_eq!(err.errors.len(), 1);
        assert!(err.errors[0].message.contains("postcondition not satisfied"));
    }
}
