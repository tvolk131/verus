"""Read-only authentication: historical checkout paths are labels, never read."""
import json,subprocess,sys
from pathlib import Path
import review as t
W=t.W
from functools import lru_cache
t.sha=lru_cache(maxsize=None)(t.sha)  # The explicitly completed worker inputs are read-only.
results=json.loads((t.ROOT/'source/tools/research/local_checked_bool_from_mir/RESULTS.json').read_text())
allowed=[t.ROOT/'source/tools/research/local_checked_bool_from_mir/RESULTS.json']
base={}
for commit in ['15045979','8b265532','ff1c65e4','d393a373','3e4c8d94','d13e8b8b','35bc7b12']:
 tree=subprocess.check_output(['git','ls-tree','-r',commit],cwd=t.ROOT).decode().splitlines()
 for l in tree:
  meta,p=l.split('\t');typ=meta.split()[1]
  if typ=='blob':base.setdefault(p,{})[commit]=meta.split()[2]
cache=json.loads((t.R/"git-blob-sha-cache.json").read_text()) if (t.R/"git-blob-sha-cache.json").exists() else {}
def gitsha(oid):
 if oid not in cache:cache[oid]=__import__('hashlib').sha256(subprocess.check_output(['git','cat-file','blob',oid],cwd=t.ROOT)).hexdigest()
 return cache[oid]
def check(label):
 d=W/label;r=json.loads((d/'receipt.json').read_text());f=json.loads((d/'freeze.json').read_text())
 t.need(r['guards'],'worker invalid guard '+label)
 for n in ['freeze','stdout','stderr']:
  t.need(t.sha(d/(n+'.json' if n=='freeze' else n))==r[n+'_sha256'],'worker bytes '+label+n)
 source=[];unmatched=[]
 for p,h in f.get('source',{}).items():
  matches=[c for c,o in base.get(p,{}).items() if gitsha(o)==h]
  if not matches:unmatched.append(p)
  source.append((p,h,matches))
 # Intentional source controls retain their exact mutated source in W.
 mutations={}
 if label.startswith('fault-'):
  m=W/label.split('/')[0]/'mutated.rs'
  if m.exists():mutations[str(m)]=t.sha(m);unmatched=[p for p in unmatched if not (p.endswith('/'+('checked_foreign_packing.rs' if 'packing-proposal' in label else 'checked_foreign.rs')) and f['source'][p]==t.sha(m))]

 if unmatched and not label.startswith('fault-'):raise RuntimeError('unbound worker source '+label+str(unmatched))
 fixtures={}
 for p,h in f.get('fixtures',{}).items():
  pp=Path(p);t.need(pp.is_relative_to(W),'fixture outside allowed worker '+p);t.need(t.sha(pp)==h,'worker fixture '+p);fixtures[p]=h
 tools={};unavailable=[]
 for n,h in f.get('tools',{}).items():
  # Use actual command's mutated bundle for fault trials.
  bundle=(d.parent/'bundle') if label.startswith('fault-') else W/'bundle'
  candidates=[bundle/n]
  if label=='final-build':candidates += [W/('fault-'+x)/'bundle'/n for x in ['generated-body','generated-type','false-proposal','mir-local-type','mir-extra-block','packing-proposal']]
  matches=[str(p) for p in candidates if p.exists() and t.sha(p)==h]
  if not matches and label=='final-build' and n=='rust_verify':unavailable.append(dict(tool=n,sha256=h,reason='superseded prebuild snapshot; not an input to compiler build or any accepted final proof'))
  else:t.need(matches,'worker tool '+label+n)
  tools[n]=h
 return dict(unavailable_prebuild_snapshot_bytes=unavailable,prebuild_tools_are_not_final_proof_tools=(label=='final-build'),label=label,receipt_sha256=t.sha(d/'receipt.json'),exit=r['exit'],verification=r.get('verification'),unbound_source=unmatched,source_count=len(source),source_manifest_sha256=__import__('hashlib').sha256(json.dumps(source,sort_keys=True).encode()).hexdigest(),tools=tools,fixtures=fixtures,mutations=mutations)
labels=['final-build','final-cold','final-warm','final-false','final-off','final-full','final-full-warm','final-support','accepted-vstd','final-all-compiler-tests','runtime-execution']
labels +=[x['label'] for x in results['export_controls']['records']]
labels +=['local-'+n for n in ['no-verify','no-lifetime','no-erasure','no-traits','filter-function','filter-module','filter-only-module','filter-root']]
labels +=['fault-'+n+'/proof' for n in ['generated-body','generated-type','false-proposal','mir-local-type','mir-extra-block','packing-proposal']]
labels +=['stable-'+kind+'/'+temp for kind in ['actual','full-help'] for temp in ['cold','warm']]
rows=[]
for label in labels:
 rows.append(check(label))
 t.write(t.R/'git-blob-sha-cache.json',cache)
products=json.loads((W/'final-build-products.json').read_text())
t.need(t.sha(W/'final-build-products.json')==results['build_products_manifest_sha256'],'worker build products')
for n,h in products['final_tools'].items():t.need(t.sha(W/'bundle'/n)==h,'worker final product '+n)
for p,h in products['core'].items():t.need(t.sha(p)==h,'worker core identity '+p)
t.need(t.sha(t.TC/'bin/rustc')==products['rustc_sha256'],'worker rustc identity')
t.write(t.R/'worker-authentication.json',dict(build_products_manifest_sha256=t.sha(W/'final-build-products.json'),status='authenticated_completed_bytes_against_own_Git_objects',receipts=rows,excluded='No live historical checkout read; worker cold/warm Eq identity remains a disclosed failure. No worker result is fresh review evidence.'))
print('Authenticated',len(rows),'completed worker receipts')
