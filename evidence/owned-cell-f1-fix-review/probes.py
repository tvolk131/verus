#!/usr/bin/env python3
"""Independent exact finding and bounded coverage probes; raw evidence private."""
import sys
sys.dont_write_bytecode=True
import json,shutil
from pathlib import Path
import recheck as r
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require
EARLY='frontend-skipped const_body/reveal_fn local item or ancestor is forbidden'
LATER='inactive: F1 admission passed;'
def raw(s):return 'use vstd::prelude::*;'+s
def empty(j):
 v=j.get('verus-infer',{});s=v.get('verification_scope',{})
 return j.get('func-details')=={} and v.get('functions')==[] and v.get('requested_functions')==0 and v.get('generated_functions')==0 and s.get('checked_exec_bodies')==0 and s.get('checked_proof_bodies')==0 and v.get('interface_written') is False

def cases():
 result=[]
 # Byte-for-byte copies of the six original finding/control source files.
 for label in ['checked_false_control','const_body_false','reveal_fn_false','const_body_native','reveal_fn_native','normal_native_control']:
  src=r.ROOT/f'evidence/owned-cell-f1-review/confirmation/{label}-False-True/input.rs'
  for infer in [False,True]:
   for flag in [False,True]:
    expected='early' if flag and label.startswith(('const_body','reveal_fn')) else 'later' if flag else 'postcondition' if label=='checked_false_control' else 'frontend_native' if label=='normal_native_control' else 'skipped'
    result.append((label,src.read_text(),infer,flag,expected))
 for marker in ['const_body','reveal_fn']:
  a=f'#[verus::internal({marker})]'
  shapes={
   'deep_ancestor':f'{a} mod a {{mod b {{mod c {{#[verifier::verify] pub fn hidden(v:u64)->u64{{v}}}}}}}}',
   'crate_inner_attr':f'#![verus::internal({marker})]\nuse vstd::prelude::*;#[verifier::verify] fn hidden(v:u64)->u64{{v}}',
   'unused_nested':f'#[verifier::verify] fn outer(v:u64)->u64{{ if false {{ {a} #[verifier::verify] fn hidden(v:u64)->u64{{v}} }} v }}',
   'marked_outer':f'{a} #[verifier::verify] fn outer(v:u64)->u64{{#[verifier::verify] fn inner(v:u64)->u64{{v}} inner(v)}}',
   'macro_item':f'macro_rules! emit {{()=>{{{a} #[verifier::verify] fn hidden(v:u64)->u64{{v}}}}}} emit!();',
   'macro_ancestor':f'macro_rules! emit {{()=>{{{a} mod a{{#[verifier::verify] fn hidden(v:u64)->u64{{v}}}}}}}} emit!();',
   'cfg_attr':f'#[cfg_attr(all(),verus::internal({marker}))] #[verifier::verify] fn hidden(v:u64)->u64{{v}}',
   'const_item':f'{a} #[verifier::verify] const K:u64=7;',
   'mixed_attributes':f'#[verifier::external_body] {a} #[verifier::verify] fn hidden(v:u64)->u64{{v}}',
   'scalar_spec':f'{a} #[verus::internal(spec)] fn hidden(v:u64)->bool{{v==v}}',
   'real_verus_macro':f'verus!{{{a} pub fn hidden(v:u64)->(r:u64) ensures r==v, {{v}}}}',
  }
  for name,s in shapes.items():
   for infer in [False,True]:result.append((marker+'_'+name,s if name=='crate_inner_attr' else raw(s),infer,True,'early'))
 for name,s,expected in [
  ('inactive_cfg_marker',raw('#[cfg_attr(any(),verus::internal(const_body))] #[verifier::verify] fn checked(v:u64)->u64{v}'),'later'),
  ('raw_scalar_const',raw('#[verifier::verify] const K:u64=7; #[verifier::verify] fn checked(v:u64)->u64{K}'),'later'),
  ('header_macro',raw('macro_rules! eq {($a:expr,$b:expr)=>{$a==$b};} verus!{pub fn checked(v:u64)->(r:u64) ensures eq!(r,v), {v}}'),'later'),
  ('nested_header',raw('verus!{pub fn outer(v:u64)->(r:u64) ensures r==v, {#[verifier::verify] fn inner(v:u64)->u64{v} inner(v)}}'),'later'),
  ('structural_const_wrapper',raw('#[verifier::verify] #[verus::internal(structural_const_wrapper)] const K:u64=7;'),'explore'),
  ('size_of_const',raw('#[verifier::verify] #[verus::internal(size_of)] const K:u64=7;'),'explore'),
  ('broadcast_const',raw('#[verifier::verify] #[verus::internal(item_broadcast_use)] const K:()=();'),'explore'),
  ('proxy_fn',raw('#[verifier::verify] #[verus::internal(uses_unerased_proxy)] fn checked(v:u64)->u64{v}'),'explore'),
 ]:
  for infer in [False,True]:result.append((name,s,infer,True,expected))
 return result

def run():
 out=r.WORK/'probes';out.mkdir()
 a=json.loads((r.HERE/'AUTHENTICATION.json').read_text());b=r.WORK/'bundle'
 paths=[b/n for n in a['tools']]+[Path(__file__),r.HERE/'recheck.py']
 g=ArtifactGuard(paths);receipt=dict(commands=[],cases=[])
 with replay_receipt(out,receipt):
  for i,(label,s,infer,flag,expected) in enumerate(cases()):
   d=out/f'{i:03}-{label}-{infer}-{flag}';d.mkdir();src=d/'input.rs';src.write_text(s)
   env=r.environment();env['VERUS_INFER_CACHE']=str(d/'infer-cache')
   argv=[str(b/'verus'),str(src),'--crate-type=lib','--crate-name=probe','--edition=2024','--no-cheating','--compile','--output-json','-o',str(d/'libprobe.rlib')]
   if flag:argv+=['--experimental-owned-unsafe-cell']
   if infer:argv+=['--infer-goals']
   code=LoggedCommands(d,r.ROOT,receipt['commands'],timeout=60)('probe',argv,env)
   err=(d/'probe.stderr').read_text();stdout=(d/'probe.stdout').read_text()
   try:j=json.loads(stdout)
   except ValueError:j={}
   actual='early' if EARLY in err else 'later' if LATER in err else 'postcondition' if 'postcondition not satisfied' in err else 'skipped' if code==0 and j.get('func-details')=={} else 'frontend_native' if 'not supported' in err else 'other'
   ok=expected==actual and (code!=0 if flag or expected in ['postcondition','frontend_native'] else code==0)
   if flag:ok=ok and empty(j) and j.get('verification-results',{}).get('success') is False
   row=dict(label=label,infer=infer,flag=flag,expected=expected,actual=actual,exit=code,empty_inventory=empty(j),source_sha256=digest(src),ok=ok,dir=str(d))
   receipt['cases'].append(row)
  g.check();receipt['guard_passed']=True
  receipt['unexpected']=[c for c in receipt['cases'] if not c['ok'] and c['expected']!='explore']
 print('cases',len(receipt['cases']),'unexpected',[(c['label'],c['infer'],c['actual']) for c in receipt['unexpected']])
 print('exploratory',[(c['label'],c['infer'],c['actual']) for c in receipt['cases'] if c['expected']=='explore'])
if __name__=='__main__':run()
