"""Ordinary false properties/ranges/body controls; no race/security trials."""
import run as t
R=t.root()
BOOL='use vstd::prelude::*; verus! { fn flag(b:bool)->(r:u64) ensures r==0u64, {u64::from(b)} }'
def check(label,source,on=True,success=False):
 j=t.trial(label,source,on=on);v=j.get('verification',{})
 t.need(v.get('is-verifying-entire-crate') and v.get('success')==success and (j['exit']==0)==success,'outcome '+label)
 if not success:t.need(v.get('errors',0)>0,'ordinary proof rejection '+label)
 return j
if __name__=='__main__':
 full=(R/'full.rs').read_text();reuse=(t.HERE/'reuse.rs').read_text()
 check('final-full-off',full,False);check('final-reuse-off',reuse,False)
 check('false-result',BOOL)
 controls=[
  ('full-false-contract', full.replace('r.word() & (u64::MAX >> 3)==tag.value()', 'r.word() & (u64::MAX >> 3)!=tag.value()')),
  ('full-original-range', full.replace('requires tag.value() <= (u64::MAX >> 3),','')),
  ('full-original-shift', full.replace('perm.to_bits() << PERM_SHIFT','perm.to_bits() << (PERM_SHIFT - 1)')),
  ('full-original-flag',full.replace('u64::from(protected)','u64::from(!protected)')),
  ('reuse-range',reuse.replace('payload < 4096','payload < 8192')),
  ('reuse-channel',reuse.replace('channel < 8','channel < 16')),
  ('reuse-shift',reuse.replace('channel << 12u64','channel << 11u64')),
  ('reuse-flag',reuse.replace('== urgent','== !urgent')),
  ('reuse-xor-overlap',reuse.replace('payload < 4096','payload < 8192').replace('payload |','payload ^')),
  ('reuse-or-overlap',reuse.replace('payload < 4096','payload < 8192')),
 ]
 for name,s in controls:t.need(s not in [full,reuse],'effective mutation');check(name,s)
