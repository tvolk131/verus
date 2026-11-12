#!/usr/bin/env python3
"""Confirm adjacent inventory/translation discrepancy, separating frontend errors."""
import sys
sys.dont_write_bytecode=True
import json
from pathlib import Path
import recheck as r
import probes as p
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt
CASES=[
 ('structural_const_wrapper',p.raw('#[verifier::verify] #[verus::internal(structural_const_wrapper)] const K:u64=7;')),
 ('structural_wrapper_with_client',p.raw('#[verifier::verify] #[verus::internal(structural_const_wrapper)] const K:u64=7; #[verifier::verify] pub fn client()->u64{K}')),
 ('empty_broadcast',p.raw('#[verifier::verify] #[verus::internal(item_broadcast_use)] const K:()={};')),
 ('size_of_malformed',p.raw('#[verifier::verify] #[verus::internal(size_of)] const K:u64=7;')),
 ('reveal_group_fn',p.raw('#[verifier::verify] #[verus::internal(reveal_group)] pub fn skipped(){}')),
 ('plain_unit_fn',p.raw('#[verifier::verify] pub fn checked(){}')),
 ('plain_scalar_const',p.raw('#[verifier::verify] const K:u64=7;')),
 ('reveal_group_false_contract',p.raw('#[verifier::verify] #[verus::internal(reveal_group)] pub fn skipped(v:u64)->u64{verus_builtin::ensures(|r:u64|[verus_builtin::constrain_type(r,skipped(v)),false]);v}')),
]
def main():
 out=r.WORK/'coverage';out.mkdir()
 a=json.loads((r.HERE/'AUTHENTICATION.json').read_text());b=r.WORK/'bundle'
 g=ArtifactGuard([b/n for n in a['tools']]+[Path(__file__)])
 rec=dict(commands=[],cases=[])
 with replay_receipt(out,rec):
  for label,s in CASES:
   for infer in [False,True]:
    for flag in [False,True]:
     d=out/f'{label}-{infer}-{flag}';d.mkdir();src=d/'input.rs';src.write_text(s)
     env=r.environment();env['VERUS_INFER_CACHE']=str(d/'infer-cache')
     argv=[str(b/'verus'),str(src),'--crate-type=lib','--crate-name=coverage','--edition=2024','--no-cheating','--compile','--output-json','-o',str(d/'libcoverage.rlib')]
     if flag:argv+=['--experimental-owned-unsafe-cell']
     if infer:argv+=['--infer-goals']
     code=LoggedCommands(d,r.ROOT,rec['commands'],timeout=60)('coverage',argv,env)
     err=(d/'coverage.stderr').read_text();stdout=(d/'coverage.stdout').read_text()
     try:j=json.loads(stdout)
     except ValueError:j={}
     rec['cases'].append(dict(label=label,infer=infer,flag=flag,exit=code,later=p.LATER in err,early=p.EARLY in err,empty_inventory=p.empty(j),source_sha256=digest(src),dir=str(d),result=j.get('verification-results'),func_details=j.get('func-details'),scope=j.get('verus-infer',{}).get('verification_scope')))
  g.check();rec['guard_passed']=True
 print([(x['label'],x['infer'],x['flag'],x['exit'],x['later']) for x in rec['cases']])
if __name__=='__main__':main()
