use crate::prelude::*;

verus! {

#[cfg_attr(verus_keep_ghost, rustc_diagnostic_item = "verus::vstd::native_option::then_some")]
pub fn then_some<Value>(condition: bool, value: Value) -> (result: Option<Value>)
    ensures
        result == if condition {
            Some(value)
        } else {
            None
        },
{
    if condition {
        Some(value)
    } else {
        None
    }
}

#[cfg_attr(verus_keep_ghost, rustc_diagnostic_item = "verus::vstd::native_option::map_or")]
pub fn map_or<Value, Output, Callback: FnOnce(Value) -> Output>(
    option: Option<Value>,
    default: Output,
    callback: Callback,
) -> (result: Output)
    requires
        option is Some ==> callback.requires((option->0,)),
    ensures
        option is None ==> result == default,
        option is Some ==> callback.ensures((option->0,), result),
{
    option.map(callback).unwrap_or(default)
}

} // verus!
