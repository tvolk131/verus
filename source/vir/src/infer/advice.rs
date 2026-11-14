//! What to change after a rejection.
//!
//! A rejection names the unsupported shape in the engine's terms. The help
//! line added here says what a user can do about it, for the rejections that
//! come up most often; every other rejection gets the general fallback.

/// The fallback that always works: a hand-written contract.
const BY_HAND: &str = "remove `#[verus_infer]` from this function and write its contract by hand (`requires`/`ensures`); other functions can stay inferred";

/// Advice for a rejection, chosen by the start of its message.
pub(super) fn advice(feature: &str) -> String {
    // Attributes cannot be added to the standard library, so a rejected call
    // into it needs other advice than a rejected call into this crate.
    if feature.starts_with("calls require an acyclic same-crate inferred callee")
        && ["`core::", "`alloc::", "`std::"].iter().any(|krate| feature.contains(krate))
    {
        return format!(
            "inference models only some standard-library functions, and this is not one of them. To keep the code as it is, {BY_HAND}"
        );
    }
    let specific: &[(&str, &str)] = &[
        (
            "calls require an acyclic same-crate inferred callee",
            "the called function has no contract that inference can use: add `#[verus_infer]` to it, or give it a hand-written contract of the form `ensures ret == <expression>` (or `returns <expression>`). If it already has `#[verus_infer]`, it was rejected itself (fix its own error first) or the calls are recursive",
        ),
        (
            "record state requires visible transparent record fields",
            "every struct this function works on must be declared to Verus (`#[verus_verify]` on the struct) with fields this function can see",
        ),
        (
            "record state requires one transparent record",
            "inference handles a method on one struct declared to Verus (`#[verus_verify]` on the struct) whose other parameters are passed by value",
        ),
        (
            "combining inference with handwritten contracts",
            "this function has both `#[verus_infer]` and its own `requires`, `ensures` or `decreases`; keep one of the two",
        ),
        (
            "only static, nongeneric inferred calls",
            "this call is generic or goes through a trait (for example a closure, an iterator, `clone` or `try_from`), which inference does not model",
        ),
        (
            "generic calls",
            "this call is generic or goes through a trait, which inference does not model",
        ),
        (
            "loops currently require",
            "inference handles only simple counted loops; a function with another loop needs a hand-written contract and loop invariants",
        ),
        (
            "parameters and return values must be",
            "a parameter or the return type is outside what inference can describe",
        ),
        (
            "? requires identical errors",
            "`?` is inferred only when the error type is unchanged or converted by a checked `From`",
        ),
    ];
    match specific.iter().find(|(start, _)| feature.starts_with(start)) {
        Some((start, text)) if start.starts_with("calls require") => text.to_string(),
        Some((start, text)) if start.starts_with("combining inference") => text.to_string(),
        Some((_, text)) => format!("{text}. Otherwise, {BY_HAND}"),
        None => format!("to keep the code as it is, {BY_HAND}"),
    }
}

#[cfg(test)]
mod tests {
    use super::advice;

    #[test]
    fn specific_advice_wins_and_everything_else_gets_the_fallback() {
        assert!(
            advice("calls require an acyclic same-crate inferred callee: `f`")
                .contains("ensures ret ==")
        );
        let std = advice(
            "calls require an acyclic same-crate inferred callee or certified imported summary: `core::slice::impl&%0::get`",
        );
        assert!(std.contains("standard-library") && !std.contains("add `#[verus_infer]`"));
        assert!(advice("generic calls").contains("write its contract by hand"));
        assert!(advice("anything else").starts_with("to keep the code as it is"));
    }
}
