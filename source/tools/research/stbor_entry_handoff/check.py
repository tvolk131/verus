#!/usr/bin/env python3
"""Serial private normal-check controller. Never invokes baseline builders."""
import hashlib, json, os, re, shutil, subprocess, sys, time
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = Path('/tmp/stbor-entry-handoff-20261110')
BASE = Path('/tmp/stbor-checked-foundation-20261110')
PREFIX = BASE/'opamroot/legacy'
UP = BASE/'src/stacked-borrows/theories'
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(p,x): p.write_text(json.dumps(x,indent=2,sort_keys=True)+'\n')
def load(p): return json.loads(p.read_text())
def foundation():
    pins = {'baseline-ready.json':'9259b8f303e52e38f1b47709429f2be23a81a4e44ed557b6058184a95e653e51',
      'completed-build-receipt.json':'0d85e9ec05f29413973137f03bc840ccb1e22170d50169b2837331e58770f649'}
    checked = {}
    def guard(p,h):
        p=Path(p); assert p.is_relative_to(BASE), str(p)
        assert sha(p)==h, str(p)
        checked[str(p)]=h
    for p,h in pins.items(): guard(BASE/p,h)
    ready=load(BASE/'baseline-ready.json'); assert ready['fully_ready'] is True
    receipt=load(BASE/'completed-build-receipt.json')
    for m in receipt['maps'].values(): guard(m['path'],m['sha256'])
    for n in ['upstream-build','recursive-kernel-check']:
        r=receipt['build_and_check_receipts'][n]
        guard(r['path'],r['sha256']); x=load(Path(r['path']))
        assert x['state']=='finished' and x['exit_status']==0
        guard(x['log'],x['log_sha256'])
    for t in receipt['tools'].values(): guard(t['path'],t['sha256'])
    for p,x in load(BASE/'manifests/kernel-loaded-objects.json').items():
        guard(p,x['object_sha256'])
        if x['source_sha256']: guard(x['source'],x['source_sha256'])
    return checked

def run(name, argv, directory, env, inputs):
    log=directory/(name+'.log'); rec=directory/(name+'.json')
    assert not log.exists() and not rec.exists()
    x=dict(argv=argv,cwd=str(directory),environment=env,inputs=inputs,
           runner_sha256=sha(__file__),started=time.time(),state='running',log=str(log))
    save(rec,x)
    with log.open('wb') as f:
        p=subprocess.Popen(argv,cwd=directory,env=env,stdout=f,stderr=subprocess.STDOUT,start_new_session=True)
        x.update(pid=p.pid,pgid=p.pid,startticks=Path('/proc/%d/stat'%p.pid).read_text().split()[21]); save(rec,x)
        code=p.wait()
    x.update(exit_status=code,state='finished',finished=time.time(),log_sha256=sha(log)); save(rec,x)
    print(name,code,str(log),flush=True)
    return code

def main():
    name=sys.argv[1]; mode=sys.argv[2] if len(sys.argv)>2 else 'build'
    ROOT.mkdir(exist_ok=True); d=ROOT/name; d.mkdir()
    for n in ['tmp','cache']: (d/n).mkdir()
    env={'PATH':str(PREFIX/'bin')+':/usr/bin:/bin','TMPDIR':str(d/'tmp'),
         'XDG_CACHE_HOME':str(d/'cache'),'LANG':'C.UTF-8','LC_ALL':'C.UTF-8',
         'PYTHONDONTWRITEBYTECODE':'1','OPAMJOBS':'2','MAKEFLAGS':'-j2'}
    inputs=foundation(); save(d/'foundation-guard.json',inputs)
    sources={}
    for p in sorted(HERE.glob('*.v')):
        shutil.copyfile(p,d/p.name); sources[p.name]=sha(p)
    for p in sorted(HERE.glob('*freeze.json')): shutil.copyfile(p,d/p.name)
    save(d/'pretrial-freeze.json',dict(sources=sources,controller_sha256=sha(__file__),
        foundation_guard_sha256=sha(d/'foundation-guard.json'),mode=mode))
    args=['-coqlib',str(PREFIX/'lib/coq'),'-Q',str(UP),'stbor','-Q',str(d),'handoff']
    if mode=='build':
        code=run('compile',[str(PREFIX/'bin/coqc')]+args+[str(d/'candidate.v')],d,env,sources)
    else: raise SystemExit('unknown mode')
    assert foundation()==inputs
    products={str(p.relative_to(d)):sha(p) for p in d.iterdir() if p.is_file()}
    save(d/'products.json',products)
    sys.exit(code)
if __name__=='__main__': main()
