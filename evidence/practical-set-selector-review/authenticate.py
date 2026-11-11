"""Authenticate permitted completed evidence and own sources, without executing it."""
import json,hashlib,shutil,difflib,sys
from pathlib import Path
R=Path(__file__).resolve().parents[2];E=Path(__file__).resolve().parent;O=Path('/tmp/practical-set-review-902041b3');W=Path('/tmp/practical-set-s1-3b931894');C=R/'source/tools/research/practical_redb_fedimint';S=C/'common_set'
h=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
r=json.loads((S/'RESULTS_S1.json').read_text());m={'parent_head':'902041b304157577c01deb5e769060fcc5398382','own_source':{},'historical':{},'selected_sources':{},'annotation_diff':{}}
for n,v in r['production_source'].items():assert h(R/n)==v;m['own_source'][n]=v
for n,v in r['freeze'].items():assert h(S/n)==v
assert h(W/'final-bundle/receipt.json')==r['runtime_receipt_sha256']
b=json.loads((W/'final-bundle/receipt.json').read_text());assert b['source']==r['production_source']
for n,v in b['tools'].items():assert h(W/'final-bundle'/n)==v
m['historical']['final-bundle/receipt.json']=h(W/'final-bundle/receipt.json')
for n,v in r['other_evidence'].items():
 assert h(W/n)==v,n;m['historical'][n]=v
for name,folder in [('accepted75','baseline2'),('disable_S1_only','ablation2'),('final','final-trials')]:
 p=W/folder/'receipt.json';assert h(p)==r['replays'][name]['receipt_sha256'];d=json.loads(p.read_text());assert d['success'];m['historical'][str(p)]=h(p)
 expected=r['replays'][name]['outcomes'];assert [i['label'] for i in d['outcomes']]==[i['label'] for i in expected]
 for x,y in zip(d['outcomes'],expected):
  for k in ['verification','failed_functions','set_pairs']:
   if k in y:assert x[k]==y[k],(name,x['label'],k)
  for stream in ['stdout','stderr']:
   p=W/folder/(x['label']+'.'+stream)
   if stream+'_sha256' in x:assert h(p)==x[stream+'_sha256']
 (E/'receipts').mkdir(exist_ok=True);shutil.copy2(W/folder/'receipt.json',E/'receipts'/('worker-'+name+'.json'))
pipeline=(R/'source/vir/src/infer/pipeline.rs').read_bytes();needle=b'        let functions = set_goals::prepare(&functions, self.imported, report)?;'
assert pipeline.count(needle)==1
abl=(W/'ablation-pipeline.rs').read_bytes();m['ablation_diff']=''.join(difflib.unified_diff(pipeline.decode().splitlines(True),abl.decode().splitlines(True)))
assert abl==pipeline.replace(needle,b'        let functions = functions; // private S1-only disable-pass ablation')
a=json.loads((W/'ablation-build.json').read_text());assert a['before']==h(R/'source/vir/src/infer/pipeline.rs')==a['restored'];assert a['disabled']==h(W/'ablation-pipeline.rs');assert a['returncode']==0
sel=json.loads((C/'SELECTION_R2_F2.json').read_text())
for case in sel['sources'].values():
 for n,pin in case['files'].items():
  p=Path(case['root'])/n;assert h(p)==pin['sha256'] and p.stat().st_size==pin['bytes'];m['selected_sources'][str(p)]=pin
# Exact textual removals, all other bytes preserved, including known whitespace.
manual=(C/'redb_tree/manual.rs').read_text();reduced=(S/'r2-reduced.rs').read_text()
expected=manual.replace('    broadcast use vstd::set_lib::range_set_properties;','    ').replace('    assert(allocated(b) =~= (if value { allocated(a).insert(i as int) } else { allocated(a).remove(i as int) }));','    ')
assert expected==reduced;m['annotation_diff']['r2']=''.join(difflib.unified_diff(manual.splitlines(True),reduced.splitlines(True)))
f=(C/'fedimint_quorum/component.rs').read_text()
for k,v in json.loads((C/'fedimint_quorum/manual.json').read_text()).items():f=f.replace('// MANUAL_'+k+': unsolved baseline slot, never claimed as a proof.',v)
f+='\n'+(C/'fedimint_quorum/adapter.rs').read_text();m['annotation_diff']['f2']=''.join(difflib.unified_diff(f.splitlines(True),(S/'f2-reduced.rs').read_text().splitlines(True)))
(E/'AUTHENTICATION.json').write_text(json.dumps(m,indent=2)+'\n')
print('Authenticated sources, exact tool bundle, completed inventories and narrow ablation.')
