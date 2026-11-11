from trials import *
need(sys.flags.dont_write_bytecode,'use -B')
src='''use vstd::prelude::*; verus! {
pub fn ordinary_packet(payload:u64,channel:u64,urgent:bool)->(word:u64)
 requires payload < 4096, channel < 8,
 ensures word & 4095u64 == payload, (word & component::CHANNEL_MASK) >> 12u64 == channel,
 (word & 32768u64 > 0) == urgent,
 {component::packet(payload,channel,urgent)}
}'''
require('packet-client',app('packet-client','packet_client',src,[('component',R/'stable-packet-public-api/cold')]),1)
