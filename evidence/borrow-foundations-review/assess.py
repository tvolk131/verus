#!/usr/bin/env python3
"""Terminal acceptance, exact object/type/assumption comparisons and compact indices."""
from review import *
import collections,ast
out={};receipts={}
for key,(base,oldsrc,delivery,prefix,logical) in SPEC.items():
 w=ROOT/key;inv=load(w/'inventory.json');nested=load(w/'signature-api-inventory.json');klog=(w/'kernel.log').read_text()
 assert 'Modules were successfully checked' in klog and 'Admitting library:' not in klog
 assert 'CONTEXT SUMMARY' in klog
 context='CONTEXT SUMMARY'+klog.split('CONTEXT SUMMARY',1)[1]
 assert 'Set is predicative' in context
 baseline_context=('CONTEXT SUMMARY'+load(base/'completed-build-receipt.json')['kernel_context_summary']) if key=='sb' else load(base/'baseline-ready.json')['kernel_context_verbatim']
 assert context.strip()==baseline_context.strip(),(key,'dependency-context mismatch')
 if key=='tb':
  for x in ['Rewrite rules are not allowed','Constants/Inductives relying on type-in-type: <none>','Constants/Inductives relying on unsafe (co)fixpoints: <none>','Inductives whose positivity is assumed: <none>']:assert x in context
 # Compare exact upstream objects: equality includes terms, not just theorem names.
 objects=load(w/'objects.json');objdiff={}
 for p,h in objects.items():
  old=sha(oldsrc/p)
  if old!=h:objdiff[p]={'delivered':old,'fresh':h}
 assert not objdiff,(key,objdiff)
 if key=='sb':
  delivered=load(base/'manifests/declaration-inventory.json');expected=set(delivered)
 else:
  delivered=load(base/'inventory-queries.json');expected={e['name'] for e in delivered['internal_constants']}
 fresh=dict(inv['declarations']);fresh.update(load(w/'extra-declarations.json'));assert expected<=set(fresh)
 assert set(fresh)==set(load(w/'inductive-pretrial.json')['found']),(key,'complete model binding coverage')
 classes={};rows=[];diffs={'types':[],'assumptions':[],'declarations':[]};oldlib={}
 if key=='tb':oldlib=load(delivery/'INVENTORY.json')['compiled_module_signatures']
 for n,e in sorted(fresh.items()):
  a=P(e['assumptions_path']).read_text();h=sha(e['assumptions_path']);classes[h]=a
  rows.append([n,e['kind'],h,e['type_sha256'],e.get('declaration_sha256','-')])
  if n not in expected:continue
  old=delivered[n] if key=='sb' else next(x for x in delivered['internal_constants'] if x['name']==n)
  for suff in ['type','assumptions']+(['declaration'] if key=='sb' and old.get('print_full_declaration') else []):
   oldp=base/f'declaration-output/{old["index"]}-{suff}.out' if key=='sb' else P(old[suff])
   if sha(oldp)!=e[suff+'_sha256']:diffs[{'type':'types','assumptions':'assumptions','declaration':'declarations'}[suff]].append(n)
 (OWN/(key+'-declarations.tsv')).write_text('name\tkind\tassumptions_sha256\ttype_sha256\tfull_declaration_sha256\n'+'\n'.join('\t'.join(row) for row in rows)+'\n')
 save(OWN/(key+'-assumptions.json'),classes)
 # Inventory all actual loaded object/source identities from fresh checker walk.
 loads={}
 for text in re.findall(r'\[intern (.*?) \.\.\.',klog):
  p=P(text);p=p if p.is_absolute() else w/'src'/p
  assert p.is_relative_to(w/'src') or p.is_relative_to(base/prefix),str(p)
  s=p.with_suffix('.v');loads[str(p)]={'object_sha256':sha(p),'source_sha256':sha(s) if s.exists() else None}
 assert len(loads)==len(inv['libraries'])
 save(w/'loaded-objects.json',loads)
 # Normalize source/object paths only for mapping to delivered ones. Contents are
 # not normalized; complete binary equality was required above.
 if key=='sb':oldlibs=load(base/'manifests/kernel-coverage.json')['checked_modules']
 else:oldlibs=delivered['all_checked_libraries']
 assert inv['libraries']==sorted(oldlibs)
 if key=='tb':assert inv['constants']==delivered['all_kernel_constants']
 nestedrows=[r.split('\t') for r in P(nested['index']).read_text().splitlines()[1:]]
 # Complete dependency field inventories and raw exact type payloads stay private.
 save(OWN/(key+'-signatures.json'),nested)
 for p in sorted(w.glob('*.json')):
  d=load(p)
  if isinstance(d,dict) and 'argv' in d:
   assert d['state']=='finished' and (d['exit_code']==0 or p.stem in ['inventory-rejected','nested-round-1','nested-v2-round-2','signature-api-query','signature-api-query-rocq']),(p,d)
   checked(d['log'],d['log_sha256']);receipts[key+'/'+p.stem]={'argv':d['argv'],'cwd':d['cwd'],'pid':d['pid'],'pgid':d['pgid'],'startticks':d['startticks'],'exit':d['exit_code'],'receipt':str(p),'receipt_sha256':sha(p),'log_sha256':d['log_sha256'],'controller_sha256':d['controller_sha256'],'classification':('query_setup_failure' if d['exit_code'] else 'successful_command'),'duration_seconds':round(d['finished_unix']-d['started_unix'],3)}
 assert sha(OWN/'signatures.py')==nested['controller_sha256']
 assert sha(OWN/'signature_walk.template')==nested['template_sha256']
 if 'loader_source' in nested:assert sha(nested['loader_source'])==nested['loader_sha256']
 assert key in load(ROOT/'postguards.json')['guards']
 out[key]={'normal_build':True,'full_recursive_kernel':True,'exact_object_equality':len(objects),'object_differences':objdiff,'libraries':len(inv['libraries']),'all_kernel_constants':len(inv['constants']),'individual_bindings':len(fresh),'delivered_individual_bindings':len(expected),'supplemental_individual_bindings':sorted(set(fresh)-expected),'assumption_classes':{h:sum(e['assumptions_sha256']==h for e in fresh.values()) for h in classes},'display_differences':diffs,'structural_signature_fields':len(nestedrows),'structural_signature_counts':nested['counts'],'kernel_context':context,'private_outputs':{p.name:{'path':str(p),'sha256':sha(p)} for p in [w/'pretrial.json',w/'source.json',w/'installed-before.json',w/'objects.json',w/'inventory.json',(w/'discovery-v2.json' if key=='sb' else w/'discovery.json'),w/'signature-api-pretrial.json',w/'signature-api-inventory.json',w/'inductive-pretrial.json',w/'extra-declarations.json',w/'loaded-objects.json',w/'final-guards.json']}}
save(ROOT/'assessment.json',out);save(OWN/'RESULTS.json',{'scope':'independently checked external MODEL only','models':out,'native_correspondences':0,'minimal_open':44,'phase_open':3,'writer_effects_open':101,'canonical':'inherited 6/0; full original domain, not rerun','HMAC':'inherited 1/1 UNPROVED; full original domain, XOR prefix AND untouched suffix, not rerun','publication':False,'goal_complete':False})
save(OWN/'COMMANDS.json',receipts)
save(OWN/'SUPPORT.json',{p.name:{'sha256':sha(p),'definitions':[n.name for n in ast.walk(ast.parse(p.read_text())) if isinstance(n,(ast.FunctionDef,ast.ClassDef,ast.AsyncFunctionDef))]} for p in sorted(OWN.glob('*.py'))})
print(json.dumps({k:{n:v for n,v in d.items() if n in ['exact_object_equality','libraries','individual_bindings','structural_signature_fields','assumption_classes']} for k,d in out.items()},indent=2))
