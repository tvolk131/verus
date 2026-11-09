#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::{InferCrates, assert_exact_report, json_inference};
use common::*;

#[test]
fn nested_record_division_imports_and_retains_its_safety_requirement() {
    for structured in [false, true] {
        let crates = InferCrates::new();
        let result_type = if structured { "Value" } else { "u64" };
        let result_body = if structured {
            "Value { quotient: self.inner.numerator.saturating_div(divisor) }"
        } else {
            "self.inner.numerator.saturating_div(divisor)"
        };
        let provider = format!(
            r#"
            use vstd::prelude::*;
            pub struct Holder {{ pub numerator: u64 }}
            pub struct Ratio {{ pub inner: Holder }}
            pub struct Value {{ pub quotient: u64 }}
            impl Ratio {{
                #[verifier::infer]
                pub fn calculate(&self, divisor: u64) -> {result_type} {{ {result_body} }}
            }}
        "#
        );
        let run = crates.build("provider", &provider, &[], &["--no-cheating", "--output-json"]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 1);
        let result_field = if structured { "result.quotient" } else { "result" };
        for divisor in ["divisor", "0"] {
            let client = format!(
                r#"
                use vstd::prelude::*;
                use provider::{{Ratio, Value}};
                pub fn client(ratio: &Ratio, divisor: u64) -> (result: {result_type})
                    requires divisor != 0,
                    ensures {result_field} == ratio.inner.numerator / divisor,
                {{ ratio.calculate({divisor}) }}
            "#
            );
            let run =
                crates.build("client", &client, &["provider"], &["--no-cheating", "--output-json"]);
            if divisor == "0" {
                assert!(!run.status.success());
                assert!(
                    String::from_utf8_lossy(&run.stderr).contains("precondition not satisfied")
                );
                assert!(!crates.vir("client").exists());
            } else {
                assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
                assert!(crates.vir("client").exists());
            }
        }
    }
}

test_verify_one_file_with_options! {
    #[test] native_zero_division_remains_a_safety_error ["vstd"] => verus_code! {
        fn zero()->u64 {10u64.saturating_div(0)}
    } => Err(error) => {
        assert_eq!(error.errors.len(),1);
        assert!(error.errors[0].message.contains("precondition not satisfied"));
    }
}

test_verify_one_file_with_options! {
    #[test] unsigned_division_keeps_nonzero_obligation ["vstd"] => verus_code! {
        #[verifier::infer]
        fn divide(left:u64, right:u64)->u64 {left.saturating_div(right)}
        fn exact(left:u64,right:u64)->(result:u64)
            requires right!=0,
            ensures result==left/right,
        {divide(left,right)}
        fn zero()->u64 {divide(u64::MAX,0)}
    } => Err(error) => {
        assert_eq!(error.errors.len(),1);
        assert!(error.errors[0].message.contains("precondition not satisfied"));
    }
}

test_verify_one_file_with_options! {
    #[test] record_division_keeps_nonzero_obligation ["vstd"] => verus_code! {
        struct Ratio { numerator:u64 }
        impl Ratio {
            #[verifier::infer]
            fn divide(&self,denominator:u64)->u64 {
                self.numerator.saturating_div(denominator)
            }
        }
        fn zero(ratio:&Ratio)->u64 {ratio.divide(0)}
    } => Err(error) => {
        assert_eq!(error.errors.len(),1);
        assert!(error.errors[0].message.contains("precondition not satisfied"));
    }
}

test_verify_one_file_with_options! {
    #[test] sound_division_keeps_nonzero_obligation ["vstd"] => verus_code! {
        #[verifier::infer(sound)]
        fn divide(left:u64, right:u64)->u64 {left.saturating_div(right)}
        fn zero()->u64 {divide(u64::MAX,0)}
    } => Err(error) => {
        assert_eq!(error.errors.len(),1);
        assert!(error.errors[0].message.contains("precondition not satisfied"));
    }
}

test_verify_one_file_with_options! {
    #[test] owned_record_return_uses_standard_saturating_models ["vstd"] => verus_code! {
        struct Amount {msats:u64}
        struct Rates {base:Amount,ppm:u64}
        impl Rates {
            #[verifier::infer]
            fn calculate(&self,amount:u64,denominator:u64)->Amount {
                Amount {msats:amount.saturating_mul(self.ppm).saturating_div(denominator)
                    .saturating_add(self.base.msats)}
            }
        }
        fn exact(rate:&Rates,amount:u64,denominator:u64)->(result:Amount)
            requires denominator>0,rate.ppm==0,
            ensures result.msats==rate.base.msats,
        {rate.calculate(amount,denominator)}
        fn zero(rate:&Rates)->Amount {rate.calculate(1,0)}
    } => Err(error) => {
        assert_eq!(error.errors.len(),1);
        assert!(error.errors[0].message.contains("precondition not satisfied"));
    }
}

test_verify_one_file_with_options! {
    #[test] unsigned_saturating_division_all_widths ["vstd"] => verus_code! {
        #[verifier::infer] fn d8(left:u8,right:u8)->u8 {left.saturating_div(right)}
        #[verifier::infer] fn d16(left:u16,right:u16)->u16 {left.saturating_div(right)}
        #[verifier::infer] fn d32(left:u32,right:u32)->u32 {left.saturating_div(right)}
        #[verifier::infer] fn d64(left:u64,right:u64)->u64 {left.saturating_div(right)}
        #[verifier::infer] fn d128(left:u128,right:u128)->u128 {left.saturating_div(right)}
        #[verifier::infer] fn ds(left:usize,right:usize)->usize {left.saturating_div(right)}
        fn exact(left:u64,right:u64)->(result:u64)
            requires right>0,
            ensures result==left/right,
        {d64(left,right)}
    } => Ok(())
}
