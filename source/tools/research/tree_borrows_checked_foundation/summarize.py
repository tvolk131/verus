"""Assemble evidence only after actual successful build, recursive check and queries.

No native theorem or source-safety claim is produced. Gates use durable executed
command receipts, checker output, required coverage, and actual final hash guards.
"""
import hashlib,json,pathlib,re,sys,datetime,ast
from guard import digest,check,canonical
r=pathlib.Path('/tmp/tree-borrows-checked-foundation-20261110');own=pathlib.Path(__file__).resolve().parent

def require(b,message):
    if not b:raise RuntimeError(message)
def save(p,x):p.write_text(json.dumps(x,indent=2,ensure_ascii=False)+'\n')
def receipt(name):
    p=r/'jobs'/(name+'.json');x=json.loads(p.read_text())
    require(x.get('exit_code')==0,'unsuccessful receipt '+name)
    require(digest(pathlib.Path(x['log']))==x['log_sha256'],'changed log '+name)
    return {'receipt':str(p),'sha256':digest(p),'details':x}
receipts={n:receipt(n) for n in ['bootstrap','build-gmp','download-dependencies','freeze-dependencies','install-dependencies-2','model-prepare','model-build','kernel','inventory']}
pre=json.loads((r/'model-pretrial.json').read_text());closure=json.loads((r/'closure.json').read_text())
require(digest(r/'closure.json')==pre['closure_sha256'],'closure changed')
require(digest(r/'installed-pretrial.json')==pre['installed_snapshot_sha256'],'installed initial snapshot changed')
source=json.loads((own/'upstream-source.json').read_text());guards={'upstream':check(r/'simuliris',source['files']),'support':check(own,pre['support_files']),'private_runners':check(r,pre['private_runner_files'])}
initial=json.loads((r/'installed-pretrial.json').read_text());guards['installed']={k:check(pathlib.Path(k),v) for k,v in initial.items()}
objects=json.loads((r/'model-objects.json').read_text());guards['model_objects']=check(r/'simuliris',objects)
require(set(objects)==set(closure['targets']),'object scope mismatch')
require('MODEL_OBJECTS_SHA256 '+digest(r/'model-objects.json') in (r/'jobs/model-build.log').read_text(),'objects not bound to normal build')
# Frozen retained sources and repository snapshots are checked independently of
# installed objects. Synthetic/generated .opam diffs are in build receipts.
deps=json.loads((own/'dependencies.json').read_text());guards['dependency_sources']={}
for pkg,v in deps.items():
    guards['dependency_sources'][pkg]={'metadata':check(pathlib.Path(v['metadata_root']),v['metadata_files'])}
    if 'source_root' in v:
        manifest=r/'source-manifests'/(pkg+'.json')
        require(digest(manifest)==v['source_manifest_sha256'],'source manifest changed '+pkg)
        guards['dependency_sources'][pkg]['source']=check(pathlib.Path(v['source_root']),json.loads(manifest.read_text()))
guards['repositories']={}
for name,v in json.loads((own/'repositories.json').read_text()).items():
    manifest=r/(name+'-tracked.json');require(digest(manifest)==v['manifest_sha256'],'repository manifest changed')
    guards['repositories'][name]=check(r/'repositories'/name,json.loads(manifest.read_text()))
guards['gmp_source']=check(r/'gmp-6.3.0',json.loads((r/'gmp-source.json').read_text()))
klog=(r/'jobs/kernel.log').read_text();require('Modules were successfully checked' in klog,'missing recursive checker success')
require('Admitting library:' not in klog,'unchecked library')
libs=sorted(set(re.findall(r'Checking library: ([^\s]+)',klog)))
require(set(closure['modules'])<=set(libs),'kernel omitted required module')
context=klog.split('CONTEXT SUMMARY',1)[-1];require(context!=klog,'missing context summary')
for s in ['Theory: Set is predicative','Theory: Rewrite rules are not allowed','Constants/Inductives relying on type-in-type: <none>','Constants/Inductives relying on unsafe (co)fixpoints: <none>','Inductives whose positivity is assumed: <none>']:
    require(s in context,'unsafe/missing context field '+s)
# Every actual file interned by the checker must belong to, and match, an
# initial installed snapshot or the object snapshot bound to the normal build.
loaded={}
for name in sorted(set(re.findall(r'\[intern (.*?) \.\.\.',klog))):
    p=pathlib.Path(name);expected=None
    try:expected=objects.get(str(p.relative_to(r/'simuliris')))
    except ValueError:pass
    for base,files in initial.items():
        try:expected=files.get(str(p.relative_to(base)),expected)
        except ValueError:pass
    require(expected is not None,'unguarded loaded file '+name);require(digest(p)==expected,'loaded hash mismatch '+name);loaded[name]=expected
require(len(loaded)==len(libs),'loaded/check library coverage mismatch')
q=json.loads((r/'inventory-queries.json').read_text());require(q['all_checked_libraries']==libs,'module query scope changed')
require(q['all_kernel_constants']==sorted(set(re.findall(r'checking cst:([^\s]+)',klog))),'constant inventory incomplete')
require('Error:' not in (r/'jobs/inventory.log').read_text(),'query error')
modules={}
for i,m in enumerate(libs):
    p=r/'query-results'/('module-%05d.out'%i);require(p.is_file() and p.stat().st_size>0,'missing full compiled module '+m)
    modules[m]={'path':str(p),'sha256':digest(p),'bytes':p.stat().st_size}
classes={};individual={};types={}
for row in q['internal_constants']:
    name=row['name'];t=pathlib.Path(row['type']);a=pathlib.Path(row['assumptions'])
    require(t.is_file() and t.stat().st_size>0,'missing type '+name);require(a.is_file(),'missing assumptions '+name)
    text=a.read_text();require('Closed under the global context' in text or 'Axioms:' in text,'unrecognized assumptions '+name)
    require('Error:' not in text and 'Error:' not in t.read_text(),'query failure '+name)
    key=hashlib.sha256(text.encode()).hexdigest();classes[key]=text;individual[name]=key
    if any(name.startswith(m+'.') for m in closure['roots']):types[name]={'type':t.read_text(),'assumptions_class':key}
expected={c for c in q['all_kernel_constants'] if c.startswith('simuliris.')};require(expected==set(individual),'individual assumptions incomplete')
# Exact ordinary premises/bodies remain in compiled module inventories and the
# guarded upstream source; Print Assumptions inventories logical axioms only.
axioms={'individual_constant_classes':individual,'classes_verbatim':classes,'kernel_dependency_context_verbatim':'CONTEXT SUMMARY'+context,'ordinary_premises_notice':'Print Assumptions does not discharge ordinary hypotheses, typeclass contexts, state_interp, state_wf, pool_safe, SafeImplies, isat or logical-relation premises. sim_expr_inner consumes pool_safe; this baseline proves no actual source safety.'}
save(own/'ASSUMPTIONS.json',axioms);save(own/'PUBLIC_TYPES.json',types)
save(r/'loaded-objects.json',loaded);save(r/'final-guards.json',guards)
source_audit=[]
for m in libs:
    if m.startswith('simuliris.'):
        p=r/'simuliris/theories'/(m[len('simuliris.'):].replace('.','/')+'.v')
    else:
        candidates=[pathlib.Path(p).with_suffix('.v') for p in loaded if p.endswith('/'+m.split('.')[-1]+'.vo')]
        candidates=[p for p in candidates if p.is_file()]
        if len(candidates)!=1:continue
        p=candidates[0]
    for n,line in enumerate(p.read_text().splitlines(),1):
        if re.search(r'\b(Admitted|Axiom|Axioms|Conjecture)\b|Unset (Guard|Positivity|Universe) Checking|type-in-type|bypass',line):source_audit.append({'module':m,'path':str(p),'line':n,'text':line})
save(own/'SOURCE_AUDIT.json',{'method':'Source keyword observations only; authoritative unsafe audit is full recursive checker context summary. Includes comments, not automatically axioms.','hits':source_audit})
# Complete declaration inventory is the full compiled Print Module signatures
# (constructors, eliminators, projections, generated helpers, nested modules),
# plus the checker constant/module walk. Full raw outputs stay private.
save(own/'INVENTORY.json',{'compiled_module_signatures':modules,'all_kernel_modules':q['all_kernel_modules'],'all_kernel_constants':{'path':str(r/'inventory-queries.json'),'key':'all_kernel_constants','sha256':digest(r/'inventory-queries.json'),'count':len(q['all_kernel_constants'])},'internal_constants':len(individual),'target_constants':len(types),'includes':'Full compiled signatures include all inductives/constructors/records/projections and generated helpers; full checker constant walk includes eliminators. No source-regex subset replaces those outputs.','raw_queries_sha256':digest(r/'inventory.v'),'raw_queries_manifest_sha256':digest(r/'inventory-queries.json')})
failed=[{'kind':'tool_policy_rejection','command':'guard edit with py_compile cleanup','result':'exec rejected an rm-style cleanup command before execution; retried with in-memory syntax compilation and no cleanup operation. No proof trial or deletion occurred.'},{'kind':'setup','command':'initial source-path/tool discovery','result':'Missing evidence path / _CoqProject corrected to repository evidence and actual _RocqProject; no proof attempted.'},{'kind':'setup','command':'dependency-solve.json','exit_code':5,'result':'New private switch initially selected only default repo; selected all three pinned local repositories, then solved successfully.'},{'kind':'setup_guard','command':'install-dependencies','exit_code':31,'result':'Three successful dune builds rejected by over-strict metadata guard (identical version header relocated and final newline normalized). Exact diffs retained; source guards now allow only that exact metadata equivalence. No formal source patch.'},{'kind':'setup_cli_discovery','command':'rocq chk --help','result':'Unknown subcommand chk; actual full checker is rocq check / rocqchk. No proof check occurred.'}]
save(own/'TRIALS.json',{'failed_setup_or_guard_trials':failed,'failed_model_proof_trials':[],'semantic_controls':[],'semantic_controls_notice':'No semantic negative or application-control claims; setup failures are not semantic controls.','initial_guard_rejections':[{**json.loads(p.read_text()),'receipt_sha256':digest(p)} for p in sorted((r/'build-guards').glob('*.json')) if 'guard_error' in json.loads(p.read_text())]})
# Record every helper in the support implementation as well as Coq inventory.
save(own/'SUPPORT.json',{p.name:{'sha256':digest(p),'definitions':[n.name for n in ast.walk(ast.parse(p.read_text())) if isinstance(n,(ast.FunctionDef,ast.AsyncFunctionDef,ast.ClassDef))]} for p in sorted(own.glob('*.py'))})
summary={'classification':'CHECKED_EXTERNAL_MODEL_BASELINE_ONLY','ready':True,'normal_compilation':True,'full_recursive_kernel_check':True,'native_source_validity':False,'source_safety_for_original_Rust':False,'publication':False,'goal_complete':False,'independent_acceptance':'pending parent integration and fresh combined acceptance','canonical':'6/0 unchanged, not rerun','HMAC':'1/1 unproved; full prefix AND suffix, unchanged domain','native_correspondences':0,'open_obligations':{'minimal':44,'phase':3,'writer_effects':101},'simuliris_commit':source['commit'],'internal_modules':len(closure['modules']),'checked_libraries':len(libs),'loaded_objects':len(loaded),'kernel_constants':len(q['all_kernel_constants']),'internal_constants':len(individual),'target_constants':len(types),'assumption_classes':len(classes),'receipts':receipts,'kernel_context_verbatim':'CONTEXT SUMMARY'+context,'hashes':{p.name:digest(p) for p in [r/'closure.json',r/'model-pretrial.json',r/'installed-pretrial.json',r/'model-objects.json',r/'loaded-objects.json',r/'final-guards.json',r/'inventory.v',r/'inventory-queries.json',own/'upstream-source.json',own/'dependencies.json',own/'ASSUMPTIONS.json',own/'INVENTORY.json',own/'PUBLIC_TYPES.json',own/'SOURCE_AUDIT.json',own/'TRIALS.json',own/'SUPPORT.json']},'runner':(r/'model.sh').read_text(),'private_environment':(r/'env.sh').read_text(),'pretrial_versions_and_loadpaths':pre,'final_guards':str(r/'final-guards.json'),'scope_limit':'An external published borrow calculus and its conditional progress/simulation/adequacy exports. No rustc operation/entry/ancestor/protector/phase/C/F bridge, no original source-safety or HMAC proof; Reserved-to-Active is not borrowck two-phase activation.','created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
save(r/'baseline-ready.json',summary);save(own/'results.json',summary)
print('READY only for checked EXTERNAL MODEL baseline:',len(closure['modules']),'internal modules,',len(libs),'recursive libraries,',len(individual),'individually inventoried internal constants')
