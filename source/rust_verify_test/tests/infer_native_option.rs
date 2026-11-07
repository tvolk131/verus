#![feature(rustc_private)]
#[macro_use]
mod common;
use common::infer::{InferCrates, assert_exact_report, json_inference, verify_exact};

const PROVIDER: &str = r#"
use vstd::prelude::*;
verus! {
pub struct Reading { pub value: u8, pub marker: bool }
impl Reading {
    #[verifier::infer]
    pub fn eager(&self, condition: bool) -> Option<u8> {
        condition.then_some(self.value + 1)
    }
    #[verifier::infer]
    pub fn lazy(&self, input: Option<u8>) -> u8 {
        input.map_or(self.value, |value| value + 1)
    }
    #[verifier::infer]
    pub fn eager_default(&self, input: Option<u8>) -> u8 {
        input.map_or(self.value + 1, |value| value)
    }
}
}
"#;

#[test]
fn native_combinators_keep_eager_argument_and_lazy_callback_domains() {
    let source = PROVIDER.to_owned()
        + r#"
verus! {
fn safe(record: &Reading, condition: bool, input: u8)
    requires record.value < 255, input < 255,
{
    let eager = record.eager(condition);
    assert(eager == if condition { Some((record.value + 1) as u8) } else { None });
    let lazy = record.lazy(Some(input));
    assert(lazy == input + 1);
    let default = record.eager_default(Some(input));
    assert(default == input);
}
fn lazy_none(record: &Reading) {
    let value = record.lazy(None);
    assert(value == record.value);
}
}
"#;
    verify_exact("native_option_evaluation", source, 3);
}

#[test]
fn eager_false_and_eager_default_overflow_cannot_publish_even_after_seeding() {
    let crates = InferCrates::new();
    for goal in [
        "fn bad(record: &Reading) { let value = record.eager(false); }",
        "fn bad(record: &Reading) { let value = record.eager_default(Some(7)); }",
        "fn bad(record: &Reading) { let value = record.lazy(Some(255)); }",
    ] {
        let seed = crates.build_source(
            "evaluation",
            PROVIDER,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        assert!(crates.vir("evaluation").exists());
        let failed = crates.build_source(
            "evaluation",
            &format!("{PROVIDER}\nverus! {{ {goal} }}"),
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(!failed.status.success());
        assert_exact_report(&json_inference(&failed), 3);
        assert!(!crates.vir("evaluation").exists());
        assert!(String::from_utf8_lossy(&failed.stderr).contains("precondition not satisfied"));
    }
}

#[test]
fn literal_tuple_constant_and_existing_option_predicate_are_exact() {
    verify_exact(
        "native_tuple_constant",
        r#"
use vstd::prelude::*;
verus! {
pub const SENTINEL: (u64, u64) = (0, 0);
pub struct Circuit { pub first: u64, pub second: u64 }
impl Circuit {
    #[verifier::infer]
    pub fn circuit(&self) -> Option<(u64, u64)> {
        let circuit = (self.first, self.second);
        (circuit != SENTINEL).then_some(circuit)
    }
    #[verifier::infer]
    pub fn present(&self) -> bool { self.circuit().is_some() }
}
fn check(record: &Circuit) {
    let present = record.present();
    assert(present == (record.first != 0 || record.second != 0));
}
}
"#
        .to_owned(),
        2,
    );
}

#[test]
fn lookalike_methods_do_not_receive_canonical_models() {
    let crates = InferCrates::new();
    let source = r#"
use vstd::prelude::*;
verus! {
pub struct Fake { pub value: bool }
impl Fake {
    pub fn then_some(&self, value: u8) -> (result: Option<u8>)
        ensures result == None,
    { None }
    #[verifier::infer]
    pub fn query(&self) -> Option<u8> { self.then_some(7) }
}
}
"#;
    let failed =
        crates.build_source("lookalike", source, &[], &["--no-cheating", "--output-json"], true);
    assert!(!failed.status.success());
    assert!(!crates.vir("lookalike").exists());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("Verus Infer unsupported"));
}

#[test]
fn independently_observed_std_evaluation_order_matches_bridge_policy() {
    let mut events = Vec::new();
    let option = false.then_some({
        events.push("argument");
        7u8
    });
    assert_eq!(option, None);
    assert_eq!(events, ["argument"]);
    events.clear();
    let selected = Some(3u8).map_or(
        {
            events.push("default");
            9
        },
        |value| {
            events.push("callback");
            value
        },
    );
    assert_eq!(selected, 3);
    assert_eq!(events, ["default", "callback"]);
    events.clear();
    let absent = None::<u8>.map_or(
        {
            events.push("default");
            9
        },
        |value| {
            events.push("callback");
            value
        },
    );
    assert_eq!(absent, 9);
    assert_eq!(events, ["default"]);
    for value in 0..=u8::MAX {
        assert_eq!(
            std::panic::catch_unwind(|| false.then_some(value.checked_add(1).unwrap())).is_ok(),
            value != u8::MAX
        );
        assert!(
            std::panic::catch_unwind(
                || None::<u8>.map_or(value, |payload| payload.checked_add(1).unwrap())
            )
            .is_ok()
        );
    }
}

#[test]
fn effectful_callback_construction_and_authored_callback_contracts_fail_closed() {
    let crates = InferCrates::new();
    for callback in [
        "{ let captured = self.value + 1; |value| value }",
        "|value: u8| -> (result: u8) ensures result == value { value }",
        "|value: u8| { return value; }",
    ] {
        let seed = crates.build_source(
            "callback_source",
            PROVIDER,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        let source = format!(
            "use vstd::prelude::*; verus! {{ pub struct Reading {{ pub value: u8, pub marker: bool }} impl Reading {{ #[verifier::infer] pub fn query(&self, input: Option<u8>) -> u8 {{ input.map_or(0, {callback}) }} }} }}"
        );
        let failed = crates.build_source(
            "callback_source",
            &source,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(!failed.status.success());
        assert!(!crates.vir("callback_source").exists());
        assert!(
            String::from_utf8_lossy(&failed.stderr).contains("Verus Infer unsupported"),
            "{}",
            String::from_utf8_lossy(&failed.stderr)
        );
    }
}

#[test]
fn imported_native_bridge_domains_and_exactness_are_rechecked() {
    let crates = InferCrates::new();
    let provider = crates.build_source(
        "native_provider",
        PROVIDER,
        &[],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(provider.status.success(), "{}", String::from_utf8_lossy(&provider.stderr));
    assert_exact_report(&json_inference(&provider), 3);
    let client = r#"
use vstd::prelude::*;
extern crate native_provider;
use native_provider::Reading;
verus! {
#[verifier::infer]
pub fn forward(record: &Reading, input: Option<u8>) -> u8 { record.lazy(input) }
fn none(record: &Reading) {
    let result = forward(record, None);
    assert(result == record.value);
}

}
"#;
    for _ in 0..2 {
        let run = crates.build_source(
            "native_client",
            client,
            &["native_provider"],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 1);
    }
    let failed = crates.build_source(
        "native_client",
        &(client.to_owned()
            + "verus! { fn bad(record: &Reading) { let value = forward(record, Some(255)); } }"),
        &["native_provider"],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(!failed.status.success());
    assert_exact_report(&json_inference(&failed), 1);
    assert!(!crates.vir("native_client").exists());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("precondition not satisfied"));
}

#[test]
fn scalar_wrapper_projection_uses_the_checked_native_option_profile() {
    let source = r#"
use vstd::prelude::*;
verus! {
pub struct Wrapper { pub value: u8 }
impl Wrapper {
    #[verifier::infer]
    pub fn query(&self, condition: bool) -> Option<u8> { condition.then_some(self.value) }
}
}
"#;
    verify_exact("scalar_wrapper", source.to_owned(), 1);
}

const ARRAY_CALLBACKS: &str = r#"
use vstd::prelude::*;
verus! {
#[verifier::infer]
pub fn query(input: Option<u64>) -> [u8; 8] {
    input.map_or(0u64.to_le_bytes(), |value| value.to_le_bytes())
}
#[verifier::infer]
pub fn decode(input: Option<u64>) -> u64 {
    u64::from_le_bytes(input.map_or(0u64.to_le_bytes(), |value| value.to_le_bytes()))
}
#[verifier::infer]
pub fn increment(input: Option<u64>) -> [u8; 8] {
    input.map_or(0u64.to_le_bytes(), |value| (value + 1).to_le_bytes())
}
fn policy(input: Option<u64>) {
    let result = query(input);
    assert(result@ == vstd::bytes::spec_u64_to_le_bytes(input.unwrap_or(0)));
    let decoded = decode(input);
    let absent = increment(None);
    assert(absent@ == vstd::bytes::spec_u64_to_le_bytes(0));
}
}
"#;

#[test]
fn native_option_array_callbacks_check_complete_outputs_and_model_dependencies() {
    let crates = InferCrates::new();
    let options = &["--no-cheating", "--output-json"];
    for phase in ["cold", "warm"] {
        let run = crates.build_source("array_provider", ARRAY_CALLBACKS, &[], options, true);
        assert!(run.status.success(), "{phase}: {}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 3);
        crates.edit_metadata("array_provider", |metadata| {
            let models = metadata["trusted_models"].to_string();
            assert!(models.contains("axiom_array_ext_equal"), "{}", models);
        });
    }
    let client = r#"
use vstd::prelude::*;
verus! {
fn policy(input: Option<u64>) {
    let result = array_provider::query(input);
    assert(result@ == vstd::bytes::spec_u64_to_le_bytes(input.unwrap_or(0)));
    let decoded = array_provider::decode(input);
}
}
"#;
    let run = crates.build_source("array_client", client, &["array_provider"], options, true);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    for (goal, diagnostic) in [
        (
            "fn bad(input: Option<u64>) { let result = query(input); assert(result[7] == 0); }",
            "assertion failed",
        ),
        (
            "fn bad() { let result = increment(Some(0xffff_ffff_ffff_ffff)); }",
            "precondition not satisfied",
        ),
    ] {
        let failed = crates.build_source(
            "array_provider",
            &format!("{ARRAY_CALLBACKS}\nverus! {{ {goal} }}"),
            &[],
            options,
            true,
        );
        assert!(!failed.status.success());
        assert_exact_report(&json_inference(&failed), 3);
        assert!(!crates.vir("array_provider").exists());
        assert!(
            String::from_utf8_lossy(&failed.stderr).contains(diagnostic),
            "{}",
            String::from_utf8_lossy(&failed.stderr)
        );
        let seed = crates.build_source("array_provider", ARRAY_CALLBACKS, &[], options, true);
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
    }
}

#[test]
fn scalar_returns_with_array_callbacks_fingerprint_existing_array_equality() {
    let crates = InferCrates::new();
    let provider = r#"
use vstd::prelude::*;
verus! {
#[verifier::infer]
pub fn decode(input: Option<u64>) -> u64 {
    u64::from_le_bytes(input.map_or(0u64.to_le_bytes(), |value| value.to_le_bytes()))
}
}
"#;
    let run = crates.build_source(
        "scalar_array_provider",
        provider,
        &[],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    assert_exact_report(&json_inference(&run), 1);
    crates.edit_metadata("scalar_array_provider", |metadata| {
        let model = metadata["trusted_models"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|entry| entry[0].to_string().contains("axiom_array_ext_equal"))
            .expect("scalar caller must fingerprint its array callback observation model");
        model[1] = "stale-array-observation".into();
    });
    let failed = crates.build_source(
        "scalar_array_client",
        "use vstd::prelude::*; verus! { fn call() { let result = scalar_array_provider::decode(None); } }",
        &["scalar_array_provider"], &["--no-cheating", "--output-json"], true,
    );
    assert!(!failed.status.success());
    assert!(!crates.vir("scalar_array_client").exists());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("model"));
}

const WRAPPER_HELPERS: &str = r#"
use vstd::prelude::*;
verus! {
pub struct Wrapper { pub value: u8 }
#[verifier::infer]
pub fn base(wrapper: &Wrapper) -> u8 { wrapper.value }
#[verifier::infer]
pub fn eager(wrapper: &Wrapper, condition: bool) -> Option<u8> {
    condition.then_some(base(wrapper) + 1)
}
#[verifier::infer]
pub fn lazy(wrapper: &Wrapper, input: Option<u8>) -> u8 {
    input.map_or(0, |value| value + base(wrapper))
}
fn policy(wrapper: &Wrapper, value: u8)
    requires wrapper.value < 255, value as int + wrapper.value as int <= 255,
{
    let eager_result = eager(wrapper, true);
    assert(eager_result == Some((wrapper.value + 1) as u8));
    let lazy_result = lazy(wrapper, Some(value));
    assert(lazy_result == value + wrapper.value);
    let absent_result = lazy(wrapper, None);
    assert(absent_result == 0);
}
}
"#;

#[test]
fn projected_native_option_callers_compose_with_checked_wrapper_helpers() {
    let crates = InferCrates::new();
    for phase in ["cold", "warm"] {
        let run = crates.build_source(
            "wrapper_helpers",
            WRAPPER_HELPERS,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(run.status.success(), "{phase}: {}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 3);
        assert!(crates.vir("wrapper_helpers").exists());
    }
    for goal in [
        "fn bad(wrapper: &Wrapper) { let result = eager(wrapper, false); }",
        "fn bad(wrapper: &Wrapper) { let result = lazy(wrapper, Some(255)); }",
    ] {
        let seed = crates.build_source(
            "wrapper_helpers",
            WRAPPER_HELPERS,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(seed.status.success());
        let failed = crates.build_source(
            "wrapper_helpers",
            &format!("{WRAPPER_HELPERS}\nverus! {{ {goal} }}"),
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(!failed.status.success());
        assert_exact_report(&json_inference(&failed), 3);
        assert!(!crates.vir("wrapper_helpers").exists());
        assert!(String::from_utf8_lossy(&failed.stderr).contains("precondition not satisfied"));
    }
}

#[test]
fn projected_native_option_callers_compose_with_imported_wrapper_helpers() {
    let crates = InferCrates::new();
    let provider = r#"
use vstd::prelude::*;
verus! {
pub struct Wrapper { pub value: u8 }
#[verifier::infer]
pub fn base(wrapper: &Wrapper) -> u8 { wrapper.value }
}
"#;
    let client = WRAPPER_HELPERS.replace("pub struct Wrapper { pub value: u8 }\n#[verifier::infer]\npub fn base(wrapper: &Wrapper) -> u8 { wrapper.value }", "use wrapper_base::{Wrapper, base};");
    for phase in ["cold", "warm"] {
        let supplied = crates.build_source(
            "wrapper_base",
            provider,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(
            supplied.status.success(),
            "{phase}: {}",
            String::from_utf8_lossy(&supplied.stderr)
        );
        assert_exact_report(&json_inference(&supplied), 1);
        let run = crates.build_source(
            "wrapper_client",
            &client,
            &["wrapper_base"],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(run.status.success(), "{phase}: {}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 2);
        assert!(crates.vir("wrapper_client").exists());
    }
    let failed = crates.build_source(
        "wrapper_client",
        &format!("{client}\nverus! {{ fn bad(wrapper: &Wrapper) {{ let result = lazy(wrapper, Some(255)); }} }}"),
        &["wrapper_base"],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(!failed.status.success());
    assert_exact_report(&json_inference(&failed), 2);
    assert!(!crates.vir("wrapper_client").exists());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("precondition not satisfied"));
}

#[test]
fn sound_only_records_cannot_gain_exact_callback_assumptions() {
    let crates = InferCrates::new();
    let seed =
        crates.build_source("sound_only", PROVIDER, &[], &["--no-cheating", "--output-json"], true);
    assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
    let source = PROVIDER.replace("#[verifier::infer]", "#[verifier::infer(sound)]");
    let failed =
        crates.build_source("sound_only", &source, &[], &["--no-cheating", "--output-json"], true);
    assert!(!failed.status.success());
    assert!(!crates.vir("sound_only").exists());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("Verus Infer unsupported"));
}

const CROSS_PROFILE: &str = r#"
use vstd::prelude::*;
verus! {
#[verifier::infer]
pub fn scalar_eager(condition: bool, value: u8) -> Option<u8> {
    condition.then_some(value + 1)
}
#[verifier::infer]
pub fn scalar_lazy(input: Option<u8>, captured: u8) -> u8 {
    input.map_or(captured, |captured| captured + 1)
}
#[verifier::infer]
pub fn scalar_default(input: Option<u8>, value: u8) -> u8 {
    input.map_or(value + 1, |item| item)
}
pub struct Wrapper { pub value: u8 }
impl Wrapper {
    #[verifier::infer]
    pub fn eager(&self, condition: bool) -> Option<u8> {
        condition.then_some(self.value + 1)
    }
    #[verifier::infer]
    pub fn lazy(&self, input: Option<u8>) -> u8 {
        input.map_or(self.value, |value| value + 1)
    }
    #[verifier::infer]
    pub fn capture(&self, input: Option<u8>) -> u8 {
        let offset = self.value;
        input.map_or(0, |value| { let offset = offset + 1; value + offset })
    }
    #[verifier::infer]
    pub fn present(&self, condition: bool) -> bool {
        condition.then_some(self.value).is_some()
    }
}
pub struct Reading { pub value: u8, pub marker: bool }
impl Reading {
    #[verifier::infer]
    pub fn capture(&self, input: Option<u8>) -> u8 {
        let offset = self.value;
        input.map_or(0, |value| { let offset = offset + 1; value + offset })
    }
}
fn safe(wrapper: &Wrapper, reading: &Reading) {
    let absent = scalar_lazy(None, 255);
    assert(absent == 255);
    let shadowed = scalar_lazy(Some(2), 255);
    assert(shadowed == 3);
    let absent = wrapper.capture(None);
    assert(absent == 0);
    let absent = reading.capture(None);
    assert(absent == 0);
    let present = wrapper.present(true);
    assert(present);
}
fn captured(wrapper: &Wrapper, reading: &Reading)
    requires wrapper.value < 10, reading.value < 10,
{
    let first = wrapper.capture(Some(2));
    assert(first == wrapper.value + 3);
    let second = reading.capture(Some(2));
    assert(second == reading.value + 3);
}
}
"#;

#[test]
fn shared_profiles_recheck_cold_warm_imports_and_metadata() {
    let crates = InferCrates::new();
    for phase in ["cold", "warm"] {
        let run = crates.build_source(
            "shared_provider",
            CROSS_PROFILE,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(run.status.success(), "{phase}: {}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 8);
        crates.edit_metadata("shared_provider", |metadata| {
            let models = metadata["trusted_models"].to_string();
            assert!(models.contains("then_some"), "{}", models);
            assert!(models.contains("map_or"), "{}", models);
            assert!(models.contains("is_some"), "{}", models);
        });
    }
    let client = r#"
use vstd::prelude::*;
extern crate shared_provider;
use shared_provider::{Wrapper, Reading};
verus! {
#[verifier::infer]
pub fn scalar(input: Option<u8>) -> u8 { shared_provider::scalar_lazy(input, 255) }
#[verifier::infer]
pub fn wrapper(record: &Wrapper, input: Option<u8>) -> u8 { record.capture(input) }
#[verifier::infer]
pub fn reading(record: &Reading, input: Option<u8>) -> u8 { record.capture(input) }
fn none(first: &Wrapper, second: &Reading) {
    let result = scalar(None); assert(result == 255);
    let result = wrapper(first, None); assert(result == 0);
    let result = reading(second, None); assert(result == 0);
}
}
"#;
    for phase in ["cold", "warm"] {
        let run = crates.build_source(
            "shared_client",
            client,
            &["shared_provider"],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(run.status.success(), "{phase}: {}", String::from_utf8_lossy(&run.stderr));
        assert_exact_report(&json_inference(&run), 3);
    }
    for goal in [
        "fn bad() { let result = shared_provider::scalar_eager(false, 255); }",
        "fn bad() { let result = shared_provider::scalar_default(Some(0), 255); }",
        "fn bad() { let result = shared_provider::scalar_lazy(Some(255), 0); }",
        "fn bad(record: &Wrapper) { let result = record.eager(false); }",
        "fn bad(record: &Wrapper) { let result = record.capture(Some(255)); }",
        "fn bad(record: &Reading) { let result = record.capture(Some(255)); }",
    ] {
        let seed = crates.build_source(
            "shared_client",
            client,
            &["shared_provider"],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        let failed = crates.build_source(
            "shared_client",
            &format!("{client}\nverus! {{ {goal} }}"),
            &["shared_provider"],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(!failed.status.success());
        assert_exact_report(&json_inference(&failed), 3);
        assert!(!crates.vir("shared_client").exists());
        assert!(String::from_utf8_lossy(&failed.stderr).contains("precondition not satisfied"));
    }
    let seed = crates.build_source(
        "shared_provider",
        CROSS_PROFILE,
        &[],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(seed.status.success());
    crates.edit_metadata("shared_provider", |metadata| {
        metadata["policy"] = "candidate-bound-evidence-v58".into()
    });
    let failed = crates.build_source(
        "shared_client",
        client,
        &["shared_provider"],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(!failed.status.success());
    assert!(!crates.vir("shared_client").exists());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("incompatible inference schema"));
}

#[test]
fn native_option_composes_with_bytes_conversions_and_early_returns() {
    verify_exact("native_option_composition", r#"
use vstd::prelude::*;
verus! {
#[verifier::infer]
pub fn scalar(value: u128, condition: bool) -> u64 {
    if !condition { return 0; }
    let option = condition.then_some(u64::try_from(value).unwrap());
    option.map_or(0, |number| u64::from_le_bytes(number.to_le_bytes()))
}
pub struct Wrapper { pub value: u128 }
impl Wrapper {
    #[verifier::infer]
    pub fn query(&self, condition: bool) -> u64 {
        if !condition { return 0; }
        condition.then_some(u64::try_from(self.value).unwrap()).map_or(0, |number| u64::from_le_bytes(number.to_le_bytes()))
    }
}
pub struct Reading { pub value: u128, pub marker: bool }
impl Reading {
    #[verifier::infer]
    pub fn query(&self, condition: bool) -> u64 {
        if !condition { return 0; }
        condition.then_some(u64::try_from(self.value).unwrap()).map_or(0, |number| u64::from_le_bytes(number.to_le_bytes()))
    }
}
#[verifier::infer]
pub fn no_parameters() -> u8 { None::<u8>.map_or(9, |value| value + 1) }
#[verifier::infer]
pub fn option_bytes(input: Option<u64>) -> [u8; 8] {
    input.map_or(0, |number| number).to_le_bytes()
}
#[verifier::infer]
pub fn result_bytes(input: Result<u64, u64>) -> [u8; 8] {
    let number = match input { Ok(number) => number, Err(number) => number };
    true.then_some(number).map_or(0, |value| value).to_le_bytes()
}
fn safe(wrapper: &Wrapper, reading: &Reading) {
    let result = scalar(u128::MAX, false); assert(result == 0);
    let result = wrapper.query(false); assert(result == 0);
    let result = reading.query(false); assert(result == 0);
    let result = no_parameters(); assert(result == 9);
}
}
"#.to_owned(), 6);
}

#[test]
fn scalar_and_wrapper_native_callbacks_remain_bounded_and_exact_only() {
    let crates = InferCrates::new();
    for signature in [
        "#[verifier::infer(sound)] pub fn query(input: Option<u8>) -> u8 { input.map_or(0, |value| value) }",
        "#[verifier::infer] pub fn query(input: Option<u8>) -> u8 { input.map_or(0, { let offset = 1u8; |value| value + offset }) }",
        "#[verifier::infer] pub fn query(input: Option<u8>) -> u8 { input.map_or(0, |value: u8| { return value; }) }",
        "#[verifier::infer] pub fn query(input: Option<u8>) -> u8 { input.map_or(0, |value: u8| -> (result: u8) ensures result == value { value }) }",
        "impl Wrapper { #[verifier::infer] pub fn query(&mut self, input: Option<u8>) { self.value = input.map_or(0, |value| value); } }",
        "impl Wrapper { #[verifier::infer] pub fn query(&self, input: Option<u8>) -> u8 { input.map_or(0, |value| { let mut state = value; state = state + 1; state }) } }",
    ] {
        let seed = crates.build_source(
            "bounded_callbacks",
            CROSS_PROFILE,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        let source = format!(
            "use vstd::prelude::*; verus! {{ pub struct Wrapper {{ pub value: u8 }} {signature} }}"
        );
        let failed = crates.build_source(
            "bounded_callbacks",
            &source,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(!failed.status.success());
        assert!(!crates.vir("bounded_callbacks").exists());
        assert!(
            String::from_utf8_lossy(&failed.stderr).contains("Verus Infer unsupported"),
            "{}",
            String::from_utf8_lossy(&failed.stderr)
        );
    }
}

#[test]
fn cached_scalar_forwarders_cannot_weaken_or_strengthen_native_callback_domains() {
    for (condition, failed_check) in [(true, "soundness"), (false, "maximality")] {
        let crates = InferCrates::new();
        let provider = crates.build_source(
            "shared_provider",
            CROSS_PROFILE,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(provider.status.success(), "{}", String::from_utf8_lossy(&provider.stderr));
        let client = "use vstd::prelude::*; verus! { #[verifier::infer] pub fn relay(input: Option<u8>) -> u8 { shared_provider::scalar_lazy(input, 0) } }";
        let seed = crates.build_source(
            "poisoned_client",
            client,
            &["shared_provider"],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(seed.status.success(), "{}", String::from_utf8_lossy(&seed.stderr));
        assert_exact_report(&json_inference(&seed), 1);
        for entry in std::fs::read_dir(crates.directory.path().join("candidate-cache")).unwrap() {
            let path = entry.unwrap().path();
            let mut candidate: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            candidate["pre"] = serde_json::json!({"Bool": condition});
            std::fs::write(path, serde_json::to_vec(&candidate).unwrap()).unwrap();
        }
        let failed = crates.build_source(
            "poisoned_client",
            client,
            &["shared_provider"],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(!failed.status.success());
        assert!(!crates.vir("poisoned_client").exists());
        let report = json_inference(&failed);
        let function = &report["functions"][0];
        assert_eq!(function["metrics"]["cache"], "hit", "{function}");
        assert_eq!(function["proofs"][failed_check]["status"], "not_proved", "{function}");
        assert_eq!(function["available_to_same_crate_callers"], false);
        assert_eq!(function["exported"], false);
    }
}

#[test]
fn scalar_presence_eager_receiver_and_by_value_wrapper_captures_are_checked() {
    let source = r#"
use vstd::prelude::*;
verus! {
#[verifier::infer]
pub fn present(input: Option<u8>) -> bool { input.is_some() }
#[verifier::infer]
pub fn receiver(value: u8) -> Option<u8> { (value + 1 < 10).then_some(0) }
pub struct Wrapper { pub value: u8 }
#[verifier::infer]
pub fn consume(record: Wrapper, input: Option<u8>) -> u8 {
    input.map_or(record.value, |value| value + record.value)
}
fn safe(record: Wrapper) requires record.value < 10,
{
    let captured = record.value;
    let result = consume(record, Some(2)); assert(result == captured + 2);
    let result = present(None); assert(!result);
    let result = present(Some(255)); assert(result);
}
}
"#;
    let crates = InferCrates::new();
    for phase in ["cold", "warm"] {
        let positive = crates.build_source(
            "value_queries",
            source,
            &[],
            &["--no-cheating", "--output-json"],
            true,
        );
        assert!(
            positive.status.success(),
            "{phase}: {}",
            String::from_utf8_lossy(&positive.stderr)
        );
        assert_exact_report(&json_inference(&positive), 3);
    }
    let failed = crates.build_source(
        "value_queries",
        &format!("{source}\nverus! {{ fn bad() {{ let result = receiver(255); }} }}"),
        &[],
        &["--no-cheating", "--output-json"],
        true,
    );
    assert!(!failed.status.success());
    assert_exact_report(&json_inference(&failed), 3);
    assert!(String::from_utf8_lossy(&failed.stderr).contains("precondition not satisfied"));
    assert!(!crates.vir("value_queries").exists());
}
