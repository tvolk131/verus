use vstd::prelude::*;
verus! {
// No writes and no extra resources. A diagnostic of inherited native drop facts.
pub fn native_handoff(buf: &mut [u8; 128])
    ensures *final(buf) == *old(buf),
{ let _it = buf.iter_mut(); }
}
