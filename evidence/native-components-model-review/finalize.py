#!/usr/bin/env python3
"""Authenticate replay, complete inventories, exact products, pins and final guards."""
import json, re, subprocess, sys
from pathlib import Path
sys.dont_write_bytecode=True
from review import ROOT,HERE,RAW,SPECS,need,read,write,sha,mapsha,guards
from inventory import SCOPES

def main():
 guards();pins=read(RAW/'pretrial-inputs.json');scopes={}
 def retain(p,h=None):
  p=Path(p);value=sha(p);need(h is None or value==h,'final authentication failed '+str(p))
  need(str(p) not in pins or pins[str(p)]==value,'changed retained file '+str(p));pins[str(p)]=value
 def receipt(p):retain(p);r=read(p);need(r['success'] is True,'receipt failed '+str(p));return r
 for tag,(scope,freeze,worker,expected,old) in SPECS.items():
  worker=Path(worker);retain(worker/'receipt.json',expected);w=read(worker/'receipt.json')
  r=receipt(RAW/tag/'receipt.json');i=receipt(RAW/(tag+'-inventory')/'receipt.json')
  need(i['all_final_guards_before_success'] is True,'missing inventory final guards')
  for n,f in r['inputs'].items():retain(n,f['sha256'])
  for n,f in r['proof_artifacts'].items():retain(n,f['sha256'])
  for n,h in i['products'].items():retain(n,h)
  need(i['assumptions'].keys()=={m+'.'+n for m,ns in i['inventory'].items() for n in ns},'assumption coverage')
  # Independent full cst-set equality, beyond the supplemental controller subset check.
  csts=[]
  for m in SCOPES[tag][1]:
   text=(RAW/(tag+'-inventory')/(m+'.module.out')).read_text()
   csts += [m+'.'+n for n in re.findall(r'^     (?:Definition|Parameter)\s+(\w+)',text,re.M)]
  need(set(csts)==set(i['kernel_csts']) and len(csts)==len(i['kernel_csts']),'full kernel cst-set mismatch')
  for records,output in [(w['commands'],worker),(r['commands'],RAW/tag),(i['commands'],RAW/(tag+'-inventory'))]:
   for c in records:
    need(c['exit_code']==0,'failed formal command')
    argv=c['argv']
    need(not set(argv)&{'-admit','-noinit','-norec','-unsafe','-boot','-quick','-vos','-vok','-vio','-type-in-type','-impredicative-set'},'unsafe/excluded formal command')
    if Path(argv[1]).name=='coqchk':need('-o' in argv,'kernel assumptions summary absent')
    for ext in ['stdout','stderr']:retain(output/(c['label']+'.'+ext))
  for n in ['controller.stdout','controller.stderr','launch.json']:retain(RAW/(tag+'.'+n))
  need(r['assumptions']==w['assumptions'] and r['kernel_axioms']==w['kernel_axioms'],'assumptions changed')
  if tag=='caller':need(r['proof_terms']==w['proof_terms'],'printed proof terms changed')
  wp={Path(n).name:f['sha256'] for n,f in w['proof_artifacts'].items() if n.endswith(('.v','.vo'))}
  comparison={}
  for n,f in r['proof_artifacts'].items():
   p=Path(n)
   if p.suffix in ['.v','.vo']:
    comparison[p.name]={'worker_sha256':wp[p.name],'fresh_sha256':f['sha256'],'equal':wp[p.name]==f['sha256']}
  # Verify every delivered in-scope file is unchanged in the prepared checkout.
  delivered={'caller':'a8df10a0','constructor':'3d668d29','distance':'2c0b8f23'}[tag]
  changed=subprocess.check_output(['git','diff','--name-only',delivered,'HEAD','--','source/tools/research/'+scope],cwd=ROOT,text=True)
  need(not changed,'delivered scope changed')
  if tag=='constructor':need(r['shape_controls']==w['shape_controls'],'shape controls differ')
  if tag=='distance':need(r['source_controls']==w['source_controls'],'source controls differ')
  scopes[tag]={'delivered_commit':delivered,'scope':scope,'unchanged_delivered_scope':True,'worker_receipt':{'path':str(worker/'receipt.json'),'sha256':expected},'fresh_receipt':{'path':str(RAW/tag/'receipt.json'),'sha256':sha(RAW/tag/'receipt.json')},'inventory_receipt':{'path':str(RAW/(tag+'-inventory')/'receipt.json'),'sha256':sha(RAW/(tag+'-inventory')/'receipt.json')},'normal_coqc_recursive_coqchk':True,'declarations':i['inventory'],'per_declaration_assumptions':i['assumptions'],'source_declarations':i['source_count'],'inventory_helpers':i['inspection_helpers'],'program_obligations':[],'kernel_axioms':i['kernel_axioms'],'object_and_source_comparison':comparison,'deterministic_differences':[n for n,c in comparison.items() if not c['equal']],'commands':[{k:c[k] for k in ['label','exit_code','seconds']} for c in r['commands']],'supplement_commands':[{k:c[k] for k in ['label','exit_code','seconds']} for c in i['commands']]}
 ready_path=Path('/tmp/verus-infer-buffer-cycle-20261109/rocq-toolchain/baseline-ready.json');ready=read(ready_path)
 base_path=Path(ready['receipt']);need(base_path.name=='receipt.json' and base_path.parent.name=='durable-build-2','wrong baseline receipt')
 retain(ready_path);retain(base_path);b=read(base_path);need(b['success'] and ready['baseline_kernel_checked'],'unchecked baseline')
 need(ready['source_commit']=='115a618dd35ff8337112cf2081a7f5391d3973d5' and len(b['copied_sources'])==523,'wrong baseline source')
 commands={c['label']:c for c in b['commands']}
 for label in ['prover-version','build-baseline','kernel-check-baseline']:need(commands[label]['returncode']==0,'incomplete durable build')
 for c in b['commands']:
  if 'log' in c:retain(c['log'],c['log_sha256'])
 baseline_root=Path(ready['build']);objects={str(p.relative_to(baseline_root)):h for n,h in pins.items() for p in [Path(n)] if p.is_relative_to(baseline_root) and p.suffix=='.vo'}
 binding=read(ROOT/'source/tools/research/native_semantic_bridge/baseline-bindings.json')
 need(len(objects)==284 and mapsha(objects)==binding['compiled_module_map_sha256'],'baseline object map differs')
 gmp=Path('/tmp/verus-infer-buffer-cycle-20261109/rocq-toolchain/gmp-private/manifest.json');retain(gmp,'332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37')
 tools={n:Path(ready['switch'])/'_opam/bin'/n if n in ['coqc','coqchk'] else Path(ready[n]) for n in ['coqc','coqchk','runner','opam']}
 for n,p in tools.items():retain(p,binding['tools'][n])
 failed=RAW/'constructor-integration-drift/receipt.json';retain(failed);failure=read(failed)
 need(failure['success'] is False and failure['commands']==[] and failure['failure']['message']=='freeze mismatch source/tools/research/mutable_buffer_goal/TARGET.md','unexpected integration failure')
 for p in [HERE/'review.py',HERE/'pretrial-freeze.json',HERE/'inventory.py',HERE/'inventory-freeze.json',HERE/'finalize.py',HERE/'finalize-freeze.json',HERE/'INTERFACE.md',HERE/'REPORT.md',HERE/'report-freeze.json',RAW/'caller-history-audit.json']:retain(p)
 need(sha(HERE/'REPORT.md')==read(HERE/'report-freeze.json')['report_sha256'],'report changed')
 need(sha(HERE/'finalize.py')==read(HERE/'finalize-freeze.json')['script_sha256'],'finalizer changed')
 # No success flag is set until the complete retained set passes final guards.
 for n,h in pins.items():need(sha(n)==h,'last guard failed '+n)
 guards()
 write(RAW/'final-artifact-pins.json',pins)
 result={'scopes':scopes,'baseline':{'ready_sha256':sha(ready_path),'durable_build_2_sha256':sha(base_path),'source_commit':ready['source_commit'],'sources':523,'objects':284,'object_map_sha256':mapsha(objects),'GMP_manifest_sha256':sha(gmp),'tools':{n:{'path':str(p),'sha256':sha(p)} for n,p in tools.items()}},'constructor_integration_replay':{'success':False,'receipt':str(failed),'sha256':sha(failed),'failure':failure['failure']},'final_artifact_pins':{'path':str(RAW/'final-artifact-pins.json'),'sha256':sha(RAW/'final-artifact-pins.json'),'count':len(pins)},'all_final_guards_before_success':True,'native_correspondences':0,'minimal_open':44,'phase_open':3,'writer_effects_open':101,'publication':False,'goal_complete':False,'canonical':'6/0 inherited, not rerun','full_HMAC':'1/1 inherited, not rerun','success':True}
 result['acceptance']={
  'caller': {'component':'ACCEPT MODEL only','delivered_evidence':'ACCEPT'},
  'constructor': {'component':'ACCEPT exact delivered conditional composition','integration_replay':'REJECT at current head: frozen TARGET.md drift'},
  'distance': {'component':'ACCEPT conditional fragment after independent helper supplement','delivered_complete_inventory_claim':'REJECT: 154 generated helpers omitted'},
  'native_certificate':'REJECT for every scope: actual source obligations unproved'}
 result['report_sha256']=sha(HERE/'REPORT.md')
 write(HERE/'results.json',result)
 print('All final guards passed; results.json written')
if __name__=='__main__':main()
