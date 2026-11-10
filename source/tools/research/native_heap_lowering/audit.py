#!/usr/bin/env python3
"""Source/eligibility controls, not semantic proofs or native publication."""
import argparse,json,sys
from copy import deepcopy
from pathlib import Path
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard,digest,support_artifacts
from infer_experiment import require,replay_receipt
from lower import Lower

def validate_plan(d,b,h,plan):
 require(plan==Lower(d,b,h).build(),'forged_or_incomplete_lowering_plan')
def publish(*_): raise ValueError('Rust_reference_and_observation_interpretation_unproved')
def load(path):return json.loads(path.read_text())
def main():
 p=argparse.ArgumentParser();p.add_argument('--extraction',type=Path,required=True);p.add_argument('--freeze',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
 r=dict(schema=1,publication=False,native_bridge_proved=False,cases=[])
 with replay_receipt(a.output,r):
  f=load(a.freeze);require(f['before_first_trial'] is True,'pretrial freeze')
  for n,h in f['files'].items():require(digest(ROOT/n)==h,'frozen source changed '+n)
  g=ArtifactGuard([*[ROOT/n for n in f['files']],*a.extraction.glob('*.json'),*support_artifacts(),a.freeze]);r['artifacts']=g.initial
  require(load(a.extraction/'receipt.json')['success'] is True,'extraction incomplete')
  for name in ['drop','same-allocation','tail-write','helper-write','drop-write','wrong-original']:
   d=load(a.extraction/(name+'-native.json'));b=load(a.extraction/(name+'-borrowck.json'));h=digest(HERE/'fixtures'/(name+'.rs'))
   try:plan=Lower(d,b,h).build()
   except ValueError as e:
    require(name not in ('drop','same-allocation'),'positive extraction rejected: '+str(e))
    r['cases'].append(dict(case=name,source_sha256=h,classification='source_eligibility_rejection',reason=str(e),semantic_negative=False))
   else:
    require(name in ('drop','same-allocation'),'changed source accepted')
    r['cases'].append(dict(case=name,source_sha256=h,classification='fragment_candidate_only',operations=plan['operation_count'],steps=len(plan['steps']),publication=False))
  d=load(a.extraction/'drop-native.json');b=load(a.extraction/'drop-borrowck.json');h=digest(HERE/'fixtures/drop.rs');plan=Lower(d,b,h).build();validate_plan(d,b,h,plan)
  def reject(name,fn):
   try:fn()
   except ValueError as e:r['cases'].append(dict(case=name,classification='eligibility_or_publication_rejection',reason=str(e),semantic_negative=False))
   else:raise ValueError('false input accepted '+name)
  for name,change in [
   ('wrong-source',lambda x:x.update(input_sha256='0'*64)),
   ('missing-helper',lambda x:x['instances'].pop()),
   ('missing-effect',lambda x:x['instances'][0]['blocks'][0]['statements'].pop()),
   ('unknown-instruction',lambda x:x['instances'][0]['blocks'][0]['statements'][0].update(ir=dict(op='unchecked_token'))),
   ('unknown-cleanup',lambda x:x['instances'][0]['blocks'][0]['term_ir'].update(unwind=dict(action='cleanup',target=1))),
   ('wrong-original-place',lambda x:x['instances'][0]['blocks'][0]['statements'][3]['ir']['value']['place'].update(local=2)),
   ('wrong-layout',lambda x:next(l for i in x['instances'] for l in i.get('locals',[]) if l['shape'].get('definition')=='std::slice::IterMut')['layout'].update(field_offsets=[8,0,16])),
   ('missing-exit',lambda x:x['instances'][0]['blocks'][1].update(term_ir=dict(op='unreachable'))),
   ('unsupported-retag',lambda x:x['instances'][0]['blocks'][1]['statements'][1]['ir']['value'].update(retag='Unknown')),
   ('unsupported-arithmetic',lambda x:next(s for i in x['instances'] for bb in i.get('blocks',[]) for s in bb['statements'] if s['ir'].get('value',{}).get('kind')=='Offset')['ir']['value'].update(kind='Mul')),
  ]:
   x=deepcopy(d);change(x)
   # Mutated typed payload is consumed, not rejected by a stale whole-IR hash.
   reject(name,lambda:Lower(x,b,h).build())
  for name,key in [('omitted-step','steps'),('omitted-exit','coverage'),('lost-unreachable-branch','excluded')]:
   x=deepcopy(plan);x[key].pop();reject(name,lambda:validate_plan(d,b,h,x))
  for name,change in [('wrong-loan',lambda x:x['loans'][0].update(loan=999)),('duplicate-loan',lambda x:x['loans'].append(deepcopy(x['loans'][0]))),('wrong-current-future-array',lambda x:x['input_shapes'][0].update(length=127))]:
   x=deepcopy(b);change(x);reject(name,lambda:Lower(d,x,h).build())
  for status in ['failed','incomplete','timeout','resource','proved']:
   reject('forged-native-'+status,lambda:publish(dict(status=status,native_bridge_proved=True,publication=True,proofs=[],plan=plan)))
  # Copyable model state/bytes do not constitute Rust authority, even with true VM proofs.
  reject('ghost-only-copyable-authority',lambda:publish(dict(model_state=deepcopy(plan),reference_embedding='not supplied')))
  g.check();print(json.dumps(dict(cases=len(r['cases']),publication=False)))
if __name__=='__main__':main()
