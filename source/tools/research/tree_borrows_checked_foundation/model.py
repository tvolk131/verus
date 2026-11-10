"""Prepare and run the exact scoped upstream normal build; no theorem stubs."""
import argparse,hashlib,json,os,pathlib,subprocess,sys
from guard import check,snapshot,digest,canonical
own=pathlib.Path(__file__).resolve().parent;r=pathlib.Path('/tmp/tree-borrows-checked-foundation-20261110');src=r/'simuliris'
p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','build','kernel','inventory']);a=p.parse_args()
def run(argv,**kw):return subprocess.run(argv,check=True,**kw)
def upstream_guard():
    files=json.loads((own/'upstream-source.json').read_text())['files']
    expected={x for x in files if x.endswith('.v')}
    actual={str(x.relative_to(src)) for x in src.rglob('*.v')}
    if expected!=actual:raise RuntimeError('unexpected proof source file set')
    return check(src,files)
def support_guard():
    pre=json.loads((r/'model-pretrial.json').read_text())
    for file,key in [('closure.json','closure_sha256'),('installed-pretrial.json','installed_snapshot_sha256')]:
        if digest(r/file)!=pre[key]:raise RuntimeError('pretrial manifest changed '+file)
    check(r,pre['private_runner_files'])
    return check(own,pre['support_files'])
def object_guard():
    receipt=json.loads((r/'jobs/model-build.json').read_text())
    log=pathlib.Path(receipt['log'])
    if receipt.get('exit_code')!=0 or digest(log)!=receipt['log_sha256']:raise RuntimeError('invalid durable build receipt')
    if 'MODEL_OBJECTS_SHA256 '+digest(r/'model-objects.json') not in log.read_text():raise RuntimeError('object snapshot not bound to build receipt')
    return check(src,json.loads((r/'model-objects.json').read_text()))
def dependency_guard():
    support_guard()
    pre=json.loads((r/'installed-pretrial.json').read_text())
    return {base:check(pathlib.Path(base),files) for base,files in pre.items()}
if a.action=='prepare':
    upstream_guard()
    with (r/'dependencies.d').open('w') as o, (r/'dependencies.stderr').open('w') as e:run(['rocq','dep','-f','_RocqProject'],cwd=src,stdout=o,stderr=e)
    run([sys.executable,'-B',str(own/'closure.py'),str(src),str(r/'dependencies.d'),str(r/'closure.json')])
    installed={}
    for base in [r/'opamroot/tb/bin',r/'opamroot/tb/lib',r/'deps',r/'opamroot/download-cache']:
        installed[str(base)]=snapshot(base)
    (r/'installed-pretrial.json').write_text(json.dumps(installed,indent=2)+'\n')
    versions={}
    for argv in [['rocq','--version'],['rocq','--print-version'],['rocq','compile','-config'],['ocamlc','-version'],['ocamlfind','list'],['opam','list','--switch=tb','--installed','--columns=package','--short']]:
        x=subprocess.run(argv,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True);versions[' '.join(argv)]={'exit_code':x.returncode,'stdout':x.stdout}
        if x.returncode:raise SystemExit('tool identity query failed '+str(argv))
    with (r/'loadpath.log').open('w') as f:run(['rocq','repl','-q','-quiet','-Q',str(src/'theories'),'simuliris'],input='Print LoadPath.\n',text=True,stdout=f,stderr=subprocess.STDOUT)
    pre={'upstream_guard':upstream_guard(),'installed_snapshot_sha256':digest(r/'installed-pretrial.json'),'installed_roots':{k:{'files':len(v),'map_sha256':canonical(v)} for k,v in installed.items()},'versions':versions,'closure_sha256':digest(r/'closure.json'),'loadpath_sha256':digest(r/'loadpath.log'),'private_runner_files':{x:digest(r/x) for x in ['env.sh','model.sh','install-deps.sh','bootstrap.sh','build-gmp.sh']},'support_files':snapshot(own)}
    (r/'model-pretrial.json').write_text(json.dumps(pre,indent=2)+'\n')
    (own/'closure.json').write_bytes((r/'closure.json').read_bytes());(own/'model-pretrial.json').write_bytes((r/'model-pretrial.json').read_bytes())
elif a.action=='build':
    upstream_guard();dependency_guard();c=json.loads((r/'closure.json').read_text())
    run(['make','Makefile.rocq'],cwd=src)
    run(['make','-f','Makefile.rocq','-j2']+['theories/'+m[len('simuliris.'):].replace('.','/')+'.vo' for m in c['roots']],cwd=src)
    upstream_guard();dependency_guard()
    objects={p:digest(src/p) for p in c['targets']}
    (r/'model-objects.json').write_text(json.dumps(objects,indent=2)+'\n')
    print('MODEL_OBJECTS_SHA256 '+digest(r/'model-objects.json'),flush=True)
elif a.action=='kernel':
    upstream_guard();dependency_guard();c=json.loads((r/'closure.json').read_text())
    object_guard()
    # Full recursive checking; do NOT add -norec, -admit, or unsafe exclusions.
    run(['rocq','check','-o','-Q',str(src/'theories'),'simuliris']+c['roots'],cwd=src)
    upstream_guard();dependency_guard();object_guard()
elif a.action=='inventory':
    upstream_guard();dependency_guard();object_guard()
    run([sys.executable,'-B',str(own/'inventory.py'),str(r)])
    run(['rocq','repl','-q','-batch','-Q',str(src/'theories'),'simuliris','-l',str(r/'inventory.v')],cwd=src)
    upstream_guard();dependency_guard()
    if a.action=='inventory':object_guard()
