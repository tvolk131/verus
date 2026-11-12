#!/usr/bin/env python3
"""Fresh old-runtime executions compared to exact corrected witnesses."""
import sys
sys.dont_write_bytecode=True
import json,shutil
from pathlib import Path
import recheck as r
import probes as p
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require

def main():
 a=json.loads((r.ROOT/'evidence/owned-cell-f1-review/AUTHENTICATION.json').read_text())
 snap=Path(a['snapshot']);require(digest(snap/'receipt.json')==a['receipt_sha256'],'old receipt')
 for n,h in a['tools'].items():require(digest(snap/n)==h,'old '+n)
 b=r.WORK/'old-bundle';b.mkdir()
 for n,h in a['tools'].items():shutil.copy2(snap/n,b/n);require(digest(b/n)==h,'private old '+n)
 (b/'verus-root').touch()
 out=r.WORK/'compatibility';out.mkdir()
 receipt=dict(commands=[],cases=[],old_tools=a['tools'],old_receipt_sha256=a['receipt_sha256'],authenticated_before_copy=True)
 guard=ArtifactGuard([b/n for n in a['tools']])
 with replay_receipt(out,receipt):
  for i,(label,s,infer,flag,expected) in enumerate(p.cases()[:24]):
   d=out/f'{i:03}-{label}-{infer}-{flag}';d.mkdir();src=d/'input.rs';src.write_text(s)
   env=r.environment();env.update(VERUS_ROOT=str(b),VERUS_Z3_PATH=str(b/'z3'),VERUS_INFER_CACHE=str(d/'infer-cache'))
   argv=[str(b/'verus'),str(src),'--crate-type=lib','--crate-name=probe','--edition=2024','--no-cheating','--compile','--output-json','-o',str(d/'libprobe.rlib')]
   if flag:argv+=['--experimental-owned-unsafe-cell']
   if infer:argv+=['--infer-goals']
   code=LoggedCommands(d,r.ROOT,receipt['commands'],timeout=60)('old',argv,env)
   err=(d/'old.stderr').read_text();j=json.loads((d/'old.stdout').read_text())
   new=r.WORK/'probes'/d.name
   newerr=(new/'probe.stderr').read_text();nj=json.loads((new/'probe.stdout').read_text())
   # Compare stable semantics, no timings or source path labels.
   def norm(j,err):
    v=j['verus-infer'];scope=v['verification_scope'];f=scope.get('native_functions',[])
    return dict(verification={k:j['verification-results'].get(k) for k in ['success','verified','errors','encountered-error','encountered-vir-error']},postcondition='postcondition not satisfied' in err,native_unsupported='not supported' in err,func_details_count=len(j['func-details']),functions=v['functions'],checked_exec=scope['checked_exec_bodies'],checked_proof=scope['checked_proof_bodies'],native=[{k:x.get(k) for k in ['function','selected','body_status','ordinary_contract']} for x in f],interface_written=v['interface_written'])
   same=norm(j,err)==norm(nj,newerr)
   ok=(p.LATER in err and p.empty(j) and code!=0) if flag else same
   receipt['cases'].append(dict(label=label,infer=infer,flag=flag,ok=ok,old_later=p.LATER in err,new_later=p.LATER in newerr,new_early=p.EARLY in newerr,stable_default_off_match=same if not flag else None,old=norm(j,err),new=norm(nj,newerr),source_sha256=digest(src)))
  guard.check();receipt['guard_passed']=True
  receipt['unexpected']=[x for x in receipt['cases'] if not x['ok']]
 print('old calls',len(receipt['cases']),'unexpected',[(x['label'],x['infer'],x['flag']) for x in receipt['unexpected']])
if __name__=='__main__':main()
