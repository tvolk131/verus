#!/usr/bin/env python3
import sys
sys.dont_write_bytecode=True
import json,shutil
from pathlib import Path
import probes as p
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require

def main():
 out=p.HERE/'compatibility';out.mkdir();snapshot=Path('/tmp/owned-cell-preflight-6e5fb54d-parent-snapshot')
 require(digest(snapshot/'receipt.json')=='25ccaf84cf2ae40c0e4c84c081ce9924b58ab1eb6257d43828158e1cde1d757b','preflight receipt')
 hs=json.loads((snapshot/'receipt.json').read_text())['identity']['tools_sha256'];g=ArtifactGuard([snapshot/'receipt.json',*[snapshot/n for n in hs]])
 baseline=p.HERE/'private/preflight-bundle';baseline.mkdir()
 for n,h in hs.items():
  require(digest(snapshot/n)==h,'preflight '+n);shutil.copy2(snapshot/n,baseline/n);require(digest(baseline/n)==h,'private '+n)
 (baseline/'verus-root').touch();g.check()
 bundle=p.HERE/'private/bundle';tools=ArtifactGuard([*[baseline/n for n in hs],*[bundle/n for n in hs]])
 cases=[
  ('header_true',p.source('fn pass(v:u64)->(r:u64) ensures r==v, {v}')),
  ('header_false',p.source('fn bad(v:u64)->(r:u64) ensures r!=v, {v}')),
  ('scalar_spec_helper',p.source('spec fn eq(a:u64,b:u64)->bool{a==b} fn pass(v:u64)->(r:u64) requires eq(v,v), ensures eq(r,v), {v}')),
  ('scalar_tuple',p.source('fn pass(v:((u64,bool),()))->(r:((u64,bool),())) ensures r==v, {v}')),
  ('scalar_quantifier',p.source('fn pass(v:u64)->(r:u64) ensures forall|n:u64| n==n, r==v, {v}')),
  ('overflow',p.source('fn bad(v:u64)->u64{v+1}')),
  ('builtin_reexport',p.raw('use verus_builtin::{ensures as E,constrain_type as T};#[verifier::verify] pub fn pass(v:u64)->u64{E(|r:u64|[T(r,pass(v)),r==v]);v}')),
  ('builtin_reexport_false',p.raw('use verus_builtin::{ensures as E,constrain_type as T};#[verifier::verify] pub fn pass(v:u64)->u64{E(|r:u64|[T(r,pass(v)),r!=v]);v}')),
 ]
 r=dict(commands=[],cases=[],baseline_tool_hashes=hs)
 with replay_receipt(out,r):
  for label,src in cases:
   for infer in [False,True]:
    observed=[]
    for name,b in [('preflight',baseline),('F1',bundle)]:
     d=out/f'{label}-{infer}-{name}';d.mkdir();s=d/'input.rs';s.write_text(src)
     argv=[str(b/'verus'),str(s),'--crate-type=lib','--crate-name=compatibility','--edition=2024','--no-cheating','--compile','--output-json','-o',str(d/'libcompatibility.rlib')]
     if infer:argv+=['--infer-goals']
     env=p.check.env(p.HERE/'private/environment');env.update(VERUS_ROOT=str(b),VERUS_Z3_PATH=str(b/'z3'),VERUS_INFER_CACHE=str(d/'fresh-cache'))
     code=LoggedCommands(d,p.ROOT,r['commands'],timeout=60)(label,argv,env)
     j=json.loads((d/f'{label}.stdout').read_text());vi=j['verus-infer'];err=(d/f'{label}.stderr').read_text()
     obs=dict(exit=code,verification=j['verification-results'],scope=vi['verification_scope'],postcondition_failure='postcondition not satisfied' in err,overflow_failure='possible arithmetic underflow/overflow' in err,functions=sorted(j['func-details']))
     # Verification durations are not compatibility semantics.
     obs['verification']={k:v for k,v in obs['verification'].items() if not k.startswith('time')}
     observed.append(obs)
    r['cases'].append(dict(label=label,infer=infer,equal=observed[0]==observed[1],observed=observed))
  tools.check();g.check();r['guard_passed']=True
  r['unexpected']=[c for c in r['cases'] if not c['equal']]
 print('comparisons',len(r['cases']),'different',len(r['unexpected']))
if __name__=='__main__':main()
