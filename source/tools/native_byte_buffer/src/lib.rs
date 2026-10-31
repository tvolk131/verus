#![cfg_attr(verus_keep_ghost, feature(proc_macro_hygiene))]
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

#[cfg(verus_keep_ghost)]
verus! {
pub open spec fn result_policy(input:Seq<Result<u8,u8>>,r:Result<Vec<u8>,u8>)->bool {
    match r {
        Ok(v)=>v@.len()==input.len() && forall|i:int| 0<=i<input.len() ==> input[i]==Ok(v@[i]),
        Err(e)=>exists|i:int| 0<=i<input.len() && input[i]==Err(e) &&
            (forall|j:int| 0<=j<i ==> input[j].is_ok()),
    }
}
}

/// Fill a fresh byte buffer, returning the first error in the input stream.
/// Extra requested capacity demonstrates that the proof does not equate
/// allocation capacity with either the request or the eventual output length.
#[cfg_attr(verus_keep_ghost, verus_verify(byte_buffer))]
#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures result_policy(input@,r)))]
pub fn fill(input: Vec<Result<u8, u8>>) -> Result<Vec<u8>, u8> {
    let iter = input.into_iter();
    let len = iter.len();
    let capacity = len.saturating_add(8);
    let mut output = Vec::with_capacity(capacity);
    let mut ptr = output.as_mut_ptr();
    for byte in iter {
        unsafe {
            core::ptr::write(ptr, byte?);
            ptr = ptr.add(1);
        }
    }
    unsafe { output.set_len(len); }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::fill;
    #[test]
    fn values_and_first_error() {
        assert_eq!(fill(vec![]), Ok(vec![]));
        assert_eq!(fill(vec![Ok(0), Ok(128), Ok(255)]), Ok(vec![0,128,255]));
        assert_eq!(fill(vec![Err(3), Err(4)]), Err(3));
        assert_eq!(fill(vec![Ok(17), Err(42), Err(9)]), Err(42));
    }
}
