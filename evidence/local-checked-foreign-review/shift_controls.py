from controls import raw,wrap
import review as t
zero='pub fn call(b:bool,x:u64)->(r:u64) ensures r==x, {let z=u64::from(b); x<<0u64}'
r=raw('semantic-shift-zero-off',wrap(zero),goals=False);t.need(r['exit']!=0 and r['verification']['errors']==1,'inherited zero-shift result')
helped=zero.replace(' x<<0u64}', ' assert(x<<0u64 == x) by(bit_vector); x<<0u64}')
r=raw('semantic-shift-zero-ordinary-bv',wrap(helped));t.need(r['exit']==0,'ordinary BV zero-shift semantics')
for n,body,success in [
 ('shift-overflow','pub fn call(b:bool,x:u64)->u64 {let z=u64::from(b);x<<64u64}',False),
 ('signed-false','pub fn call(b:bool,x:i64)->(r:i64) ensures r>=0, {let z=u64::from(b);x<<1u64}',False),
 ('constant-nested','const MASK:u64=((1u64<<63u64)^0xffffu64)>>1u64; pub fn call(b:bool,x:u64)->(r:bool) ensures r, {let z=u64::from(b);let y=x<<2u64;MASK==0x4000000000007fffu64}',True),
]:
 r=raw('semantic-'+n,wrap(body));t.need((r['exit']==0)==success,'shift/constant '+n)
