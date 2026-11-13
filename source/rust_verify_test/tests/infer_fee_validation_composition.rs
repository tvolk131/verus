#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::{InferCrates, assert_exact_report, json_inference, verify_exact};
use common::*;

const PROVIDER: &str = r#"
use vstd::prelude::*;
#[verus_verify(infer_derive(PartialEq, PartialOrd, Ord))]
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Quantity { pub units: __WORD__ }
#[verus_verify]
#[derive(Copy, Clone)]
pub struct Budget { pub amount: Quantity, pub rate: __WORD__ }
#[verus_verify]
impl Quantity {
    #[verus_infer]
    pub fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self { units: self.units.checked_add(other.units)? })
    }
}
#[verus_verify]
impl Budget {
    #[verus_infer]
    pub fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            amount: self.amount.checked_add(other.amount)?,
            rate: self.rate.checked_add(other.rate)?,
        })
    }
    #[verus_infer]
    pub fn is_within(&self, limit: &Self) -> bool {
        self.amount <= limit.amount && self.rate <= limit.rate
    }
}
"#;

const GOALS: &str = r#"
verus! {
pub fn accept_sum(left: Budget, right: Budget, limit: Budget) -> (result: Option<Budget>)
    ensures match result {
        Some(value) =>
            value.amount.units as int == (left.amount.units as int) + (right.amount.units as int)
            && value.rate as int == (left.rate as int) + (right.rate as int)
            && value.amount.units <= limit.amount.units && value.rate <= limit.rate,
        None =>
            (left.amount.units as int) + (right.amount.units as int) > __WORD__::MAX
            || (left.rate as int) + (right.rate as int) > __WORD__::MAX
            || (left.amount.units as int) + (right.amount.units as int) > limit.amount.units
            || (left.rate as int) + (right.rate as int) > limit.rate,
    },
{
    match left.checked_add(right) {
        None => None,
        Some(value) => if value.is_within(&limit) { Some(value) } else { None },
    }
}
pub fn aliased_limit(value: &Budget) -> (result: bool)
    ensures result,
{
    value.is_within(value)
}
}
"#;

const FALSE_GOAL: &str = r#"
verus! {
pub fn base_order_is_not_enough() {
    let lower_base = Budget { amount: Quantity { units: 0 }, rate: 1 };
    let higher_base = Budget { amount: Quantity { units: 1 }, rate: 0 };
    let accepted = lower_base.is_within(&higher_base);
    assert(accepted);
}
}
"#;

#[test]
fn named_comparisons_two_roots_and_nested_option_compose_at_u8_and_u64() {
    for width in ["u8", "u64"] {
        verify_exact(
            &format!("fee_composition_{width}"),
            format!("{PROVIDER}{GOALS}").replace("__WORD__", width),
            4,
        );
    }
}

#[test]
fn composed_component_check_rejects_a_lexicographic_shortcut() {
    let error = verify_one_file(
        "fee_composition_false_policy",
        format!("{PROVIDER}{FALSE_GOAL}").replace("__WORD__", "u8"),
        &["vstd", "--no-cheating", "--output-json"],
    )
    .unwrap_err();
    assert!(error.errors.iter().any(|diagnostic| diagnostic.message.contains("assertion failed")));
    let report = &error.json_output.as_ref().unwrap()["verus-infer"];
    assert_exact_report(report, 4);
    assert_eq!(report["crate_success"], false);
    assert_eq!(report["interface_written"], false);
}

#[test]
fn imported_composition_checks_aliases_and_removes_a_failed_client_export() {
    let crates = InferCrates::new();
    let provider = PROVIDER.replace("__WORD__", "u64");
    let run =
        crates.build_source("provider", &provider, &[], &["--no-cheating", "--output-json"], true);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_exact_report(&json_inference(&run), 4);
    let prefix = "use vstd::prelude::*; use provider::{Budget, Quantity};\n";
    let client = format!("{prefix}{GOALS}").replace("__WORD__", "u64");
    let run = crates.build_source(
        "client",
        &client,
        &["provider"],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert!(crates.vir("client").is_file());
    let report = json_inference(&run);
    assert!(!report["imported_inference_dependencies"].as_array().unwrap().is_empty());
    let run = crates.build_source(
        "client",
        &format!("{prefix}{FALSE_GOAL}"),
        &["provider"],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("assertion failed"));
    assert!(!crates.vir("client").exists());
}
