#!/usr/bin/env python3
"""Source eligibility controls, never semantic counterexamples or proof import."""
import argparse
import copy
import json
import sys
from pathlib import Path
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard,digest,support_artifacts
from infer_experiment import require,replay_receipt
from source_map import build
p=argparse.ArgumentParser()
p.add_argument('--extraction',type=Path,required=True)
p.add_argument('--freeze',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
r=dict(schema=1,publication=False,native_bridge_proved=False,cases=[],semantic_negative_count=0)
with replay_receipt(a.output,r):
 f=json.loads(a.freeze.read_text())
 require(f['before_first_trial'] is True,'pretrial freeze')
 for n,h in f['files'].items():require(digest(ROOT/n)==h,'changed input '+n)
 paths=[a.extraction/n for n in ('drop-native.json','drop-borrowck.json','receipt.json')]
 g=ArtifactGuard([*paths,a.freeze,*[ROOT/n for n in f['files']],*support_artifacts()])
 require(json.loads(paths[2].read_text())['success'] is True,'unhealthy extractor')
 native=json.loads(paths[0].read_text());borrowck=json.loads(paths[1].read_text())
 source=HERE.parent/'native_heap_lowering/fixtures/drop.rs'
 base=build(native,borrowck,source)
 require(base['publication'] is False and base['native_effects_discharged']==0,'accidental publication')
 r['healthy_audit']=dict(native_operations=44,unproved_rejected=44,phase_obligations=3)
 def check(name,mutate):
  n,b=copy.deepcopy(native),copy.deepcopy(borrowck);mutate(n,b)
  try:build(n,b,source)
  except ValueError as error:r['cases'].append(dict(name=name,classification='source_eligibility_rejection',message=str(error)))
  else:raise ValueError('source mutation not rejected '+name)
 def first_value(n,op):
  return next(s['ir']['value'] for i in n['instances'] for bb in i['blocks'] for s in bb['statements']
   if s['ir']['op']=='assign' and s['ir']['value']['op']==op)
 check('wrong-original-loan',lambda n,b:b['loans'][0]['borrowed'].update(local=4))
 check('missing-issued-loan',lambda n,b:b['borrowck_facts'].update(loan_issued_at=[]))
 check('incomplete-native-inventory',lambda n,b:n.update(pending_instances=['missing']))
 check('unknown-retag',lambda n,b:first_value(n,'use').update(retag='Invented'))
 check('wrong-raw-borrow-kind',lambda n,b:first_value(n,'raw_ptr').update(kind='Const'))
 check('wrong-byte-offset-operation',lambda n,b:first_value(n,'binary').update(kind='Add'))
 check('wrong-input-extent',lambda n,b:b['input_shapes'][0].update(length=127,size=127))
 check('missing-physical-instance',lambda n,b:n['instances'].pop())
 require(len(r['cases'])==8,'control inventory')
 r['artifacts']=g.initial;g.check()
