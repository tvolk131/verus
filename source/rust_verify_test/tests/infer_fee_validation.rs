#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::verify_exact as checked;
use common::*;

const NATIVE: &str = r#"use vstd::prelude::*;
#[verus_verify(infer_derive(PartialEq,PartialOrd,Ord))]
#[derive(Clone,Copy,Eq,PartialEq,Ord,PartialOrd)]
pub struct Amount {
    pub msats: u64,
}
#[verus_verify]
impl Amount {
    #[verus_infer]
    pub fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            msats: self.msats.checked_add(other.msats)?,
        })
    }
}
#[verus_verify]
#[derive(Copy,Clone)]
pub struct PaymentFee {
    pub base: Amount,
    pub parts_per_million: u64,
}
#[verus_verify]
impl PaymentFee {
    #[verus_infer]
    pub fn is_within(&self, limit: &PaymentFee) -> bool {
        self.base <= limit.base && self.parts_per_million <= limit.parts_per_million
    }
    #[verus_infer]
    pub fn checked_add(self, rhs: Self) -> Option<PaymentFee> {
        Some(PaymentFee {
            base: self.base.checked_add(rhs.base)?,
            parts_per_million: self.parts_per_million.checked_add(rhs.parts_per_million)?,
        })
    }
}
"#;

#[test]
fn native_nested_options_and_independent_borrowed_roots() {
    checked(
        "native_fee_validation",
        format!(
            r#"{NATIVE}
verus! {{
fn addition(left:PaymentFee,right:PaymentFee) {{
    let result=left.checked_add(right);
    assert(match result {{
        None => (left.base.msats as int)+(right.base.msats as int)>u64::MAX ||
            (left.parts_per_million as int)+(right.parts_per_million as int)>u64::MAX,
        Some(value) => value.base.msats as int == (left.base.msats as int)+(right.base.msats as int) &&
            value.parts_per_million as int == (left.parts_per_million as int)+(right.parts_per_million as int),
    }});
}}
fn bounds(value:&PaymentFee,limit:&PaymentFee) {{
    let result=value.is_within(limit);
    assert(result==(value.base.msats<=limit.base.msats && value.parts_per_million<=limit.parts_per_million));
    let equal=value.is_within(value);
    assert(equal);
}}
}}
"#
        ),
        4,
    );
}

#[test]
fn either_overflow_and_exact_maximum_are_not_preconditions() {
    checked(
        "native_fee_maximum",
        format!(
            r#"{NATIVE}
verus! {{fn goal() {{
    let maximum=PaymentFee{{base:Amount{{msats:u64::MAX}},parts_per_million:u64::MAX}};
    let zero=PaymentFee{{base:Amount{{msats:0}},parts_per_million:0}};
    let exact=maximum.checked_add(zero);
    assert(exact matches Some(value) && value.base.msats==u64::MAX && value.parts_per_million==u64::MAX);
    let base=PaymentFee{{base:Amount{{msats:1}},parts_per_million:0}};
    let ppm=PaymentFee{{base:Amount{{msats:0}},parts_per_million:1}};
    let base_overflow=maximum.checked_add(base); assert(base_overflow is None);
    let ppm_overflow=maximum.checked_add(ppm); assert(ppm_overflow is None);
}}}}
"#
        ),
        4,
    );
}

#[test]
fn equality_cannot_establish_a_strict_native_bound() {
    let failure = verify_one_file(
        "native_fee_false_strict",
        format!(
            r#"{NATIVE}
verus! {{fn goal(value:&PaymentFee) {{
    let result=value.is_within(value); assert(result);
    assert(value.base.msats<value.base.msats);
}}}}
"#
        ),
        &["vstd", "--no-cheating", "--output-json"],
    )
    .unwrap_err();
    assert!(failure.errors.iter().any(|error| error.message.contains("assertion failed")));
    let report = &failure.json_output.as_ref().unwrap()["verus-infer"];
    assert_eq!(report["interface_written"], false);
    assert!(
        report["functions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|function| { function["required_checks_passed"] == true })
    );
}
