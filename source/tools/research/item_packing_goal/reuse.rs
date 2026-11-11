use vstd::prelude::*;
verus! {
const CHANNEL_MASK:u64=7u64 << 12u64;
// A packet header: 12-bit payload, 3-bit channel, and an urgent flag.
fn packet(payload:u64,channel:u64,urgent:bool)->(word:u64)
    requires payload < 4096, channel < 8,
    ensures word & 4095u64 == payload,
        (word & CHANNEL_MASK) >> 12u64 == channel,
        (word & 32768u64 > 0) == urgent,
{
    payload | (channel << 12u64) | (u64::from(urgent) << 15u64)
}
}
