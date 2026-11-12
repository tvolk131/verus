#!/usr/bin/env python3
import sys
sys.dont_write_bytecode=True
import json
from pathlib import Path
import probes as p
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt

def main():
 out=p.HERE/'confirmation';out.mkdir()
 b=p.HERE/'private/bundle';hs=json.loads((p.HERE/'AUTHENTICATION.json').read_text())['tools']
 guard=ArtifactGuard([b/n for n in hs]);r=dict(commands=[],cases=[])
 raw=p.raw
 examples=[
  ('checked_false_control',raw('#[verifier::verify] pub fn checked(v:u64)->u64{verus_builtin::ensures(|r:u64|[verus_builtin::constrain_type(r,checked(v)),false]);v}')),
  ('const_body_false',raw('#[verifier::verify] #[verus::internal(const_body)] pub fn skipped(v:u64)->u64{verus_builtin::ensures(|r:u64|[verus_builtin::constrain_type(r,skipped(v)),false]);v}')),
  ('reveal_fn_false',raw('#[verifier::verify] #[verus::internal(reveal_fn)] pub fn skipped(v:u64)->u64{verus_builtin::ensures(|r:u64|[verus_builtin::constrain_type(r,skipped(v)),false]);v}')),
  ('const_body_native',raw('#[verifier::verify] #[verus::internal(const_body)] pub fn skipped(v:u64)->u64{core::cell::UnsafeCell::new(v).into_inner()}')),
  ('reveal_fn_native',raw('#[verifier::verify] #[verus::internal(reveal_fn)] pub fn skipped(v:u64)->u64{core::cell::UnsafeCell::new(v).into_inner()}')),
  ('normal_native_control',raw('#[verifier::verify] pub fn checked(v:u64)->u64{core::cell::UnsafeCell::new(v).into_inner()}')),
 ]
 with replay_receipt(out,r):
  for label,s in examples:
   for infer in [False,True]:
    for flag in [False,True]:
     d=out/f'{label}-{infer}-{flag}';d.mkdir();src=d/'input.rs';src.write_text(s)
     argv=[str(b/'verus'),str(src),'--crate-type=lib','--crate-name=confirm','--edition=2024','--no-cheating','--compile','--output-json','-o',str(d/'libconfirm.rlib')]
     if flag:argv+=['--experimental-owned-unsafe-cell']
     if infer:argv+=['--infer-goals']
     env=p.check.env(p.HERE/'private/environment');env.update(VERUS_ROOT=str(b),VERUS_Z3_PATH=str(b/'z3'),VERUS_INFER_CACHE=str(d/'fresh-cache'))
     code=LoggedCommands(d,p.ROOT,r['commands'],timeout=60)(label,argv,env)
     err=(d/f'{label}.stderr').read_text();j=json.loads((d/f'{label}.stdout').read_text());vi=j['verus-infer']
     r['cases'].append(dict(label=label,infer=infer,flag=flag,exit=code,later=p.LATER in err,func_details=j['func-details'],scope=vi['verification_scope'],source_sha256=digest(src)))
  guard.check();r['guard_passed']=True
 print([(c['label'],c['infer'],c['flag'],c['exit'],c['later'],c['scope']['checked_exec_bodies']) for c in r['cases']])
if __name__=='__main__':main()
