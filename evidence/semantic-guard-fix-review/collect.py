#!/usr/bin/env python3
"""Package small, hashed review artifacts; raw proofs/logs stay in private paths."""
import argparse, json, shutil, sys
from pathlib import Path
sys.dont_write_bytecode=True
from recheck import ROOT,BRIDGE,sha,need,save

def main(work,out):
 final=json.loads((work/'final/validation.json').read_text());need(final['success'] is True,'incomplete validation')
 copies={
  'identities.json':'identities/identities.json',
  'probes.json':'probes/probes.json',
  'controller-refusals.json':'probes/controller-refusals.json',
  'omission-results.json':'omission/omission-results.json',
  'history-validation.json':'history/history.json',
  'suite.json':'suite/suite.json',
  'assumption-inventory.stdout':'proofs/assumption-inventory.stdout',
  'kernel-summary.stderr':'proofs/kernel-check.stderr',
  'statements-and-assumptions.stdout':'final/independent-inventory.stdout',
  'IndependentInventory.v':'final/IndependentInventory.v'}
 for name,rel in copies.items():shutil.copyfile(work/rel,out/name)
 controls={}
 for label,directory in [('source',work/'suite/source-controls'),('readiness',work/'suite/readiness-controls'),('acceptance',work/'guard-controls')]:
  p=directory/'receipt.json';r=json.loads(p.read_text());need(r['success'] is True,'failed control suite')
  cases=[]
  for case in r['cases']:
   case=dict(case)
   if 'receipt' in case:
    p=Path(case['receipt']);nested=json.loads(p.read_text());case.update(actual_receipt_sha256=sha(p),failure=nested.get('failure'),commands=nested.get('commands',[]),logs={str(q):sha(q) for q in p.parent.iterdir() if q.suffix in ['.stdout','.stderr']})
    need(nested['success'] is False and nested['publication'] is False,'control acquired positive status')
   cases.append(case)
  controls[label]=dict(cases=cases,receipt=str(directory/'receipt.json'),sha256=sha(directory/'receipt.json'))
 save(out/'controls.json',controls)
 # Bind every retained receipt and every command log, including refused trials.
 raw={}
 for p in work.rglob('*'):
  if p.is_file() and (p.name=='receipt.json' or p.suffix in ['.stdout','.stderr'] or p.name in ['baseline-snapshot.json','inventory-run.json','mirror.json','omitted-binding-freeze.json','full-freeze.json']):raw[str(p)]=sha(p)
 final.update(review_base='e84eed6c79d9c104cefada569de0c99221dc2b51',fix_commit='1efbd004',scope='independent fixed-guard review; exact frozen model replay accepted; incomplete source-audit freeze remains a finding',raw_root=str(work),raw_evidence=raw,packaged_files={n:sha(out/n) for n in [*copies,'controls.json']},review_scripts={p.name:sha(p) for p in out.glob('*.py')},counts=dict(normal_units=8,optimized_units=8,source_controls=12,readiness_controls=4,acceptance_refusals=3,additional_parser_source_cases=67,additional_proof_controller_refusals=13,source_binding_omission_false_acceptances=2),semantic_negative_count=0,native_input_embedding_proved=False,verus_CF_import_proved=False,publication=False)
 save(out/'validation.json',final)
 print(json.dumps(dict(validation_sha256=sha(out/'validation.json'),raw_evidence_items=len(raw),files=len(final['packaged_files']))))

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--work',type=Path,default=Path('/tmp/semantic-guard-fix-review-e84eed6c'));p.add_argument('--output',type=Path,default=Path(__file__).resolve().parent);a=p.parse_args();main(a.work.resolve(),a.output.resolve())
