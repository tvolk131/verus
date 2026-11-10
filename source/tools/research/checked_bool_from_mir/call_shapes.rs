pub fn probe_actual(b: bool) -> u64 { u64::from(b) }
pub fn probe_other_output(b: bool) -> u32 { u32::from(b) }
pub fn probe_generic<T: From<bool>>(b: bool) -> T { T::from(b) }

mod custom {
    pub trait FromLike<T> { fn from(x: T) -> Self; }
    impl FromLike<bool> for u64 { fn from(_: bool) -> Self { 7 } }
}
pub fn probe_user_trait(b: bool) -> u64 { <u64 as custom::FromLike<bool>>::from(b) }

pub fn probe_function_pointer(f: fn(bool) -> u64, b: bool) -> u64 { f(b) }
