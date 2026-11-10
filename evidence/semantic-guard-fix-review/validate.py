#!/usr/bin/env python3
"""Independent receipt, source, baseline and fresh-object validation."""
import argparse, hashlib, json, os, re, shlex, subprocess, sys
from pathlib import Path
sys.dont_write_bytecode=True
from recheck import ROOT,BRIDGE,RAW,READY,FREEZE,sha,need,save,run
from probes import NAMES,CONSTANTS,FE

def suite(out):
 out.mkdir(parents=True,exist_ok=False)
 records=[]
 for label,cmd in [('unit',[sys.executable,'-B',BRIDGE/'test_proof_checks.py']),('unit-optimized',[sys.executable,'-B','-O',BRIDGE/'test_proof_checks.py'])]:
  r=run(out,label,cmd);need(r['returncode']==0,'unit failure');records.append(r)
 for label,tool,args in [('map','source_map.py',['--extraction',RAW]),('source-controls','audit.py',['--extraction',RAW]),('readiness-controls','gate_controls.py',[])]:
  result=out/label
  r=run(out,label,[sys.executable,'-B','-O',BRIDGE/tool,'--freeze',FREEZE,'--output',result,*args])
  receipt=json.loads((result/'receipt.json').read_text());need(r['returncode']==0 and receipt['success'] is True and receipt['publication'] is False,label+' failure')
  r.update(receipt_sha256=sha(result/'receipt.json'));records.append(r)
 save(out/'suite.json',records);print(json.dumps(records,indent=2))

def identities(out):
 out.mkdir(parents=True,exist_ok=False)
 f=json.loads(FREEZE.read_text());need(all(sha(ROOT/n)==h for n,h in f['files'].items()),'freeze mismatch')
 files=dict(f['files'])
 for n in ('check.py','proof_checks.py','source_map.py','audit.py','gate_controls.py','guard_controls.py','freeze.py','test_proof_checks.py','source-bindings.json','baseline-bindings.json','review-fix-freeze.json','REPORT.md','HISTORY.json'):
  files[str((BRIDGE/n).relative_to(ROOT))]=sha(BRIDGE/n)
 sources={}
 for m in NAMES:
  p=BRIDGE/'candidates/loan-v12'/(m+'.v');rel=str(p.relative_to(ROOT)); current=p.read_bytes()
  vals={}
  for c in ['8935534e','05581f4b','1efbd004','e84eed6c']:
   old=subprocess.check_output(['git','show',c+':'+rel],cwd=ROOT);need(old==current,'changed mathematical source');vals[c]=hashlib.sha256(old).hexdigest()
  actual=re.findall(r'^\s*(?:Lemma|Theorem)\s+(\w+)',current.decode(),re.M);need(actual==NAMES[m],'source inventory mismatch')
  sources[m]=dict(source_sha256=sha(p),git_sources=vals,named_constants=actual)
 baseline=json.loads((BRIDGE.parent/'mutable_buffer_goal/foundation-baseline.json').read_text());ready=json.loads(READY.read_text());receipt=Path(ready['receipt']);rb=json.loads(receipt.read_text())
 need(sha(READY)==baseline['readiness']['sha256'] and sha(receipt)==baseline['receipt']['sha256'] and rb['success'] is True,'unauthenticated baseline')
 build=Path(ready['build']);original=Path('/tmp/verus-infer-buffer-cycle-20261109/verus-belt-115a618d')
 need(subprocess.check_output(['git','rev-parse','HEAD'],cwd=original,text=True).strip()==baseline['source_commit'],'wrong baseline commit')
 tracked=subprocess.check_output(['git','ls-files','src'],cwd=original,text=True).splitlines();copied=rb['copied_sources']
 need(set(tracked)==set(copied) and len(copied)==523,'incomplete copied baseline')
 need(all(sha(build/n)==v==sha(original/n) for n,v in copied.items()),'changed baseline source')
 bindings=json.loads((BRIDGE/'baseline-bindings.json').read_text())
 objects={str(p.relative_to(build)):sha(p) for p in (build/'src').rglob('*.vo')}
 canonical=hashlib.sha256(json.dumps(objects,sort_keys=True,separators=(',',':')).encode()).hexdigest()
 need(len(objects)==284 and canonical==bindings['compiled_module_map_sha256'],'wrong baseline object map')
 tools={'coqc':Path(ready['switch'])/'_opam/bin/coqc','coqchk':Path(ready['switch'])/'_opam/bin/coqchk','runner':Path(ready['runner']),'opam':Path(ready['opam'])}
 need(all(sha(p)==bindings['tools'][n] for n,p in tools.items()),'wrong tool identities')
 # Stable full baseline snapshot, including auxiliary tools and loadpath metadata.
 guarded=[READY,receipt,*tools.values(),build/'_CoqProject',READY.parent/'gmp-private/manifest.json',Path(ready['switch'])/'_opam/bin/ocamlc']
 guarded += [build/n for n in copied]+list((build/'src').rglob('*.vo'))
 snapshot={str(p):dict(sha256=sha(p),size=p.stat().st_size,mtime_ns=p.stat().st_mtime_ns,inode=p.stat().st_ino) for p in guarded}
 for label,pin in [('parent',BRIDGE.parent/'mutable_buffer_goal/parent-semantic-foundation.json')]:
  parent=json.loads(pin.read_text())
  for key in ['original_recursive_kernel_check','review_fix_positive_receipt']:
   v=parent[key];need(sha(v['path'])==v['sha256'] and json.loads(Path(v['path']).read_text())['success'] is True,'bad claimed proof receipt')
  v=parent['review_fix_validation'];need(sha(v['receipt'])==v['sha256'],'bad parent replay receipt')
  replay=json.loads(Path(v['receipt']).read_text());need(replay['success'] is True,'failed parent replay')
  for c in replay['commands']:
   need(c['returncode']==0 and sha(c['log'])==c['log_sha256'],'parent replay log mismatch')
 archive=json.loads((BRIDGE/'HISTORY.json').read_text())
 # Accommodate the manifest's descriptive wrapper, using its explicit entries.
 save(out/'archive-manifest.json',archive)
 versions={}
 for label,path,args in [('coq',tools['coqc'],['--version']),('ocaml',Path(ready['switch'])/'_opam/bin/ocamlc',['-version'])]:
  r=run(out,label,[tools['runner'],path,*args]);need(r['returncode']==0,'version command failed');versions[label]=r
 save(out/'identities.json',dict(review_base=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),files=files,proof_sources=sources,baseline_ready_sha256=sha(READY),baseline_receipt_sha256=sha(receipt),copied_sources=copied,compiled_module_map=objects,compiled_module_map_sha256=canonical,tools={n:dict(path=str(p),sha256=sha(p)) for n,p in tools.items()},versions=versions,raw_extraction={p:sha(RAW/p) for p in ['receipt.json','drop-native.json','drop-borrowck.json']}))
 save(out/'baseline-snapshot.json',snapshot)
 print(json.dumps(dict(sources_identical=True,copied_sources=len(copied),compiled_objects=len(objects),tools_authenticated=True)))

def finish(out,work):
 out.mkdir(parents=True,exist_ok=True)
 receipt=work/'proofs/receipt.json';r=json.loads(receipt.read_text())
 need(r['success'] is True and r['model_proofs_kernel_checked'] is True,'fresh positive failed')
 need(len(r['assumption_inventory'])==25,'incomplete fresh assumptions')
 expected_axioms={n:([FE] if n.startswith('byte_array.') and n not in ['byte_array.byte_ty','byte_array.byte_list_length'] and '.byte_ty_obligation_' not in n else []) for n in CONSTANTS}
 need(r['assumption_inventory']==expected_axioms,'actual assumptions not exactly advertised')
 need(set(r['kernel_axioms'])=={FE,'Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq'},'unexpected actual kernel summary')
 old=json.loads(Path('/tmp/native-semantic-bridge-e4a8dcab/proof-v12/receipt.json').read_text())
 parent=json.loads(Path('/tmp/verus-infer-buffer-cycle-20261109/semantic-guard-validation-1efbd004/proofs/receipt.json').read_text())
 need(r['proof_terms']==old['proof_terms']==parent['proof_terms'],'fresh proof objects differ')
 for m in NAMES:
  need(sha(work/'proofs/proofs'/(m+'.v'))==sha(BRIDGE/'candidates/loan-v12'/(m+'.v')),'fresh source differs')
  need(sha(work/'proofs/proofs'/(m+'.vo'))==r['proof_terms'][m],'fresh object differs')
  need(sha(Path('/tmp/native-semantic-bridge-e4a8dcab/proof-v12/proofs')/(m+'.vo'))==r['proof_terms'][m],'original object differs')
  need(sha(Path('/tmp/verus-infer-buffer-cycle-20261109/semantic-guard-validation-1efbd004/proofs/proofs')/(m+'.vo'))==r['proof_terms'][m],'parent object differs')
 need(sha(work/'proofs/kernel-check.stderr')==sha('/tmp/native-semantic-bridge-e4a8dcab/proof-v12/kernel-check.stderr'),'kernel summary changed')
 # Independent normal compilation: all statements and all 25 assumptions with own delimiters.
 ready=json.loads(READY.read_text());baseline=Path(ready['build']);args=[]
 for line in (baseline/'_CoqProject').read_text().splitlines():
  words=shlex.split(line,comments=True)
  if words and words[0] in ['-R','-Q']:args += [words[0],str(baseline/words[1]),words[2]]
 args+=['-Q',str(work/'proofs/proofs'),'NativeSemanticBridge']
 src='From NativeSemanticBridge Require Import '+ ' '.join(NAMES)+'.\n'
 for n in CONSTANTS:
  src+=f'Goal True. idtac "AUDIT-BEGIN {n}". exact I. Qed.\nCheck NativeSemanticBridge.{n}.\nPrint Assumptions NativeSemanticBridge.{n}.\nGoal True. idtac "AUDIT-END {n}". exact I. Qed.\n'
 src+='Fail Check NativeSemanticBridge.byte_array.byte_ty_obligation_7.\n'
 p=out/'IndependentInventory.v'
 if (out/'inventory-run.json').exists():
  need(p.read_text()==src,'independent inventory source changed')
  record=json.loads((out/'inventory-run.json').read_text())
  need(sha(out/'independent-inventory.stdout')==record['stdout_sha256'] and sha(out/'independent-inventory.stderr')==record['stderr_sha256'],'saved inventory logs changed')
 else:
  p.write_text(src)
  record=run(out,'independent-inventory',[ready['runner'],Path(ready['switch'])/'_opam/bin/coqc','-q',*args,p]);need(record['returncode']==0,'independent inventory failed')
  save(out/'inventory-run.json',record)
 text=(out/'independent-inventory.stdout').read_text();sections=re.findall(r'^AUDIT-BEGIN (\S+)\n(.*?)^AUDIT-END (\S+)\s*$',text,re.M|re.S)
 need([a for a,_,_ in sections]==CONSTANTS and [b for _,_,b in sections]==CONSTANTS,'independent inventory missing coverage')
 actual={}
 for n,text,_ in sections:
  if 'Axioms:' not in text:
   need(text.count('Closed under the global context')==1,'missing closed context');actual[n]=[]
  else:
   body=text.split('Axioms:',1)[1]
   ax=re.findall(r'^\s*([\w.]+)\s*:',body,re.M)
   need(ax in [['functional_extensionality_dep'],['FunctionalExtensionality.functional_extensionality_dep'],[FE]],'unexpected assumption in independently emitted inventory');actual[n]=[FE]
 need(actual==expected_axioms,'independent assumptions disagree')
 snap=json.loads((work/'identities/baseline-snapshot.json').read_text())
 need(all(dict(sha256=sha(p),size=Path(p).stat().st_size,mtime_ns=Path(p).stat().st_mtime_ns,inode=Path(p).stat().st_ino)==v for p,v in snap.items()),'shared baseline changed')
 gc=work/'guard-controls/receipt.json'
 if not gc.exists():
  print('Independent inventory passed; acceptance-control run still pending. Re-run finish to bind its receipt.')
  return
 guards=json.loads(gc.read_text());need(guards['success'] is True and len(guards['cases'])==3,'guard controls did not pass')
 bound={}
 for label,directory in [('positive',work/'proofs'),('acceptance_controls',work/'guard-controls'),('map',work/'suite/map'),('source_controls',work/'suite/source-controls'),('readiness_controls',work/'suite/readiness-controls')]:
  q=directory/'receipt.json';data=json.loads(q.read_text());need(data['success'] is True,'failed required check');bound[label]=dict(path=str(q),sha256=sha(q),logs={p.name:sha(p) for p in directory.iterdir() if p.suffix in ['.stdout','.stderr']})
 save(out/'validation.json',dict(success=True,publication=False,native_correspondences_proved=0,proof_receipts=bound,proof_objects=r['proof_terms'],assumptions=actual,kernel_summary_sha256=sha(work/'proofs/kernel-check.stderr'),independent_inventory=record,baseline_unchanged=True,baseline_guarded_files=len(snap)))
 print(json.dumps(dict(success=True,all25_exact_assumptions=True,all4_objects_identical=True,baseline_unchanged=True)))

def history(out):
 out.mkdir(parents=True,exist_ok=False)
 archive=json.loads((BRIDGE/'HISTORY.json').read_text());lines=0;recovered={}
 for n,h in archive['files'].items():
  data=subprocess.check_output(['git','show','05581f4b:'+str(BRIDGE.relative_to(ROOT)/n)],cwd=ROOT)
  need(hashlib.sha256(data).hexdigest()==h,'archive mismatch '+n)
  lines+=len(data.splitlines());recovered[n]=h
 need(len(recovered)==41 and lines==2733,'archive incomplete')
 results=json.loads((BRIDGE/'results.json').read_text());receipts={}
 for n,v in results['evidence'].items():
  need(sha(v['path'])==v['sha256'] and json.loads(Path(v['path']).read_text())['success'] is True,'original evidence mismatch '+n)
  receipts[n]=v
 failed=json.loads((BRIDGE/'failed-trials.json').read_text())
 for v in failed['trials']:
  need(sha(v['receipt'])==v['receipt_sha256'] and sha(v['stderr'])==v['stderr_sha256'] and json.loads(Path(v['receipt']).read_text())['success'] is False,'failed trial evidence mismatch')
 fresh=json.loads((out.parent/'suite/map/source-obligations.json').read_text())
 committed=json.loads((BRIDGE/'source-obligations.json').read_text());instances=committed.pop('instances')
 for row in committed['native_operations']:row['instance']=instances[row['instance']]
 need(fresh==committed and len(instances)==6 and len(fresh['native_operations'])==44 and len(fresh['borrowck_phase_obligations'])==3,'source mapping differs')
 need(all(row['simulation_status']=='unproved_rejected' for row in fresh['native_operations']),'accidental native proof')
 save(out/'history.json',dict(archive_files=recovered,archive_count=41,archive_lines=lines,original_receipts=receipts,failed_trials=failed['trials'],source_mapping_matches=True,source_mapping_sha256=sha(out.parent/'suite/map/source-obligations.json')))
 print(json.dumps(dict(archive_count=41,archive_lines=lines,original_receipts=5,failed_trials=7,source_mapping_matches=True)))

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('mode',choices=['suite','identities','finish','history']);p.add_argument('--output',type=Path,required=True);p.add_argument('--work',type=Path,default=Path('/tmp/semantic-guard-fix-review-e84eed6c'));a=p.parse_args()
 if a.mode=='finish':finish(a.output.resolve(),a.work.resolve())
 else:globals()[a.mode](a.output.resolve())
