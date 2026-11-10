#!/usr/bin/env python3
"""Serial private normal-check controller. Never invokes baseline builders."""
import hashlib, json, os, re, shutil, subprocess, sys, time
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = Path(os.environ.get('STBOR_ENTRY_OUTPUT_ROOT', '/tmp/stbor-entry-handoff-20261110'))
BASE = Path('/tmp/stbor-checked-foundation-20261110')
PREFIX = BASE/'opamroot/legacy'
UP = BASE/'src/stacked-borrows/theories'
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(p,x): p.write_text(json.dumps(x,indent=2,sort_keys=True)+'\n')
def load(p): return json.loads(p.read_text())
def require_check(condition, message):
    if not condition:
        raise RuntimeError(message)

CONTROLLER_SHA = sha(__file__)
def foundation(full=False):
    pins = {'baseline-ready.json':'9259b8f303e52e38f1b47709429f2be23a81a4e44ed557b6058184a95e653e51',
      'completed-build-receipt.json':'0d85e9ec05f29413973137f03bc840ccb1e22170d50169b2837331e58770f649'}
    checked = {}
    def guard(p,h):
        p=Path(p); require_check(p.is_relative_to(BASE), str(p))
        require_check(sha(p)==h, str(p))
        checked[str(p)]=h
    for p,h in pins.items(): guard(BASE/p,h)
    ready=load(BASE/'baseline-ready.json'); require_check(ready['fully_ready'] is True, 'Required proof/artifact check failed at original line 23')
    receipt=load(BASE/'completed-build-receipt.json')
    for m in receipt['maps'].values(): guard(m['path'],m['sha256'])
    for n in ['upstream-build','recursive-kernel-check']:
        r=receipt['build_and_check_receipts'][n]
        guard(r['path'],r['sha256']); x=load(Path(r['path']))
        require_check(x['state']=='finished' and x['exit_status']==0, 'Required proof/artifact check failed at original line 29')
        guard(x['log'],x['log_sha256'])
    for t in receipt['tools'].values(): guard(t['path'],t['sha256'])
    frozen=load(BASE/'manifests/upstream-tool-freeze.json')
    # Build-tree snapshots are authenticated by the completed manifest; they are
    # not runtime inputs. Recheck installed plugin/runtime binaries and all
    # actually loaded proof objects/sources, without rereading old build trees.
    for p,h in (frozen['installed_files'].items() if full else []):
        if not p.startswith('.opam-switch/') and Path(p).suffix in ('.cmxs','.cma','.cmxa','.so'):
            guard(Path(frozen['installed_prefix'])/p,h)
    reader=load(BASE/'manifests/object-dependency-map.json')
    guard(reader['reader'],reader['reader_sha256'])
    guard(Path(reader['reader']).parent/'vo_summary.ml',reader['reader_source_sha256'])
    for p,x in load(BASE/'manifests/kernel-loaded-objects.json').items():
        guard(p,x['object_sha256'])
        if x['source_sha256']: guard(x['source'],x['source_sha256'])
    return checked

def run(name, argv, directory, env, inputs):
    log=directory/(name+'.log'); rec=directory/(name+'.json')
    require_check(not log.exists() and not rec.exists(), 'Required proof/artifact check failed at original line 49')
    x=dict(argv=argv,cwd=str(directory),environment=env,inputs=inputs,
           runner_sha256=CONTROLLER_SHA,started=time.time(),state='running',log=str(log))
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
    shutil.copyfile(__file__, d/'controller.py')
    require_check(sha(d/'controller.py') == CONTROLLER_SHA, 'Required proof/artifact check failed at original line 65')
    for n in ['tmp','cache']: (d/n).mkdir()
    env={'PATH':str(PREFIX/'bin')+':/usr/bin:/bin','TMPDIR':str(d/'tmp'),
         'XDG_CACHE_HOME':str(d/'cache'),'LANG':'C.UTF-8','LC_ALL':'C.UTF-8',
         'PYTHONDONTWRITEBYTECODE':'1','OPAMJOBS':'2','MAKEFLAGS':'-j2'}
    inputs=foundation(full=(mode=='accept')); save(d/'foundation-guard.json',inputs)
    sources={}
    for p in sorted(HERE.glob('*.v')):
        shutil.copyfile(p,d/p.name); sources[p.name]=sha(p)
    for p in sorted(HERE.glob('*freeze.json')): shutil.copyfile(p,d/p.name)
    save(d/'pretrial-freeze.json',dict(sources=sources,controller_sha256=CONTROLLER_SHA,
        foundation_guard_sha256=sha(d/'foundation-guard.json'),mode=mode))
    args=['-coqlib',str(PREFIX/'lib/coq'),'-Q',str(UP),'stbor','-Q',str(d),'handoff']
    own_modules=['candidate']+(['composition'] if (d/'composition.v').exists() else [])
    if mode in ('build','accept'):
        code=0
        for module in own_modules:
            code=run('compile-'+module,[str(PREFIX/'bin/coqc')]+args+[str(d/(module+'.v'))],d,env,sources)
            if code: break
    else: raise SystemExit('unknown mode')
    if code==0 and mode=='accept':
        declarations={}
        for module in own_modules:
            for line in (d/(module+'.glob')).read_text().splitlines():
                m=re.match(r'^(def|prf|ind|constr|rec|proj|inst) (\d+):(\d+) (\S+) (\S+)$',line)
                if m:
                    kind,a,b,scope,base=m.groups()
                    name='handoff.'+module+'.'+('' if scope=='<>' else scope+'.')+base
                    declarations[name]=dict(glob_kind=kind,source=module+'.v',source_start=int(a),source_end=int(b))
        require_check(declarations, 'Required proof/artifact check failed at original line 94')
        (d/'declarations').mkdir()
        query=['Require '+' '.join('handoff.'+m for m in own_modules)+'.', 'Print LoadPath.', 'Set Printing Width 1000000.', 'Set Printing All.', 'Set Printing Universes.']
        for i,(name,x) in enumerate(sorted(declarations.items())):
            x['index']=i
            query += [f'Redirect "{d}/declarations/{i}-type" Check {name}.',
                      f'Redirect "{d}/declarations/{i}-assumptions" Print Assumptions {name}.']
        (d/'inventory.v').write_text('\n'.join(query)+'\n')
        code=run('inventory',[str(PREFIX/'bin/coqtop'),'-quiet','-batch']+args+['-l',str(d/'inventory.v')],d,env,sources)
        require_check(code==0, 'Required proof/artifact check failed at original line 103')
        for name,x in declarations.items():
            for kind in ['type','assumptions']:
                p=d/f'declarations/{x["index"]}-{kind}.out'
                t=p.read_text(); require_check(t.strip() and 'Error:' not in t, 'Required proof/artifact check failed at original line 107')
                x[kind+'_text']=t; x[kind+'_sha256']=sha(p)
            require_check('Closed under the global context' in x['assumptions_text'], 'Required proof/artifact check failed at original line 109')
        save(d/'declaration-inventory.json',declarations)
        baseline_map=load(BASE/'manifests/object-dependency-map.json')
        modules=baseline_map['modules']
        reader=baseline_map['reader']
        code=run('object-summary',[reader]+[str(d/(m+'.vo')) for m in own_modules],d,env,sources)
        require_check(code==0, 'Required proof/artifact check failed at original line 115')
        combined=dict(modules)
        own={}
        for row in (d/'object-summary.log').read_text().splitlines():
            name,file,deps,imports=row.split('\t')
            own[name]=dict(object=file,object_sha256=sha(file),source_sha256=sha(Path(file).with_suffix('.v')),
                recorded_compiled_dependencies=dict(x.split('=') for x in deps.split(';') if x),
                recorded_imports=list(filter(None,imports.split(';'))))
        require_check(set(own)=={'handoff.'+m for m in own_modules}, 'Required proof/artifact check failed at original line 123')
        combined.update(own)
        expected=set(own)
        while True:
            expanded=expected|{dep for name in expected for dep in combined[name]['recorded_compiled_dependencies']}
            if expanded==expected: break
            expected=expanded
        save(d/'object-dependency-map.json',dict(modules={m:combined[m] for m in sorted(expected)},
            reader_sha256=sha(reader),source='serialized checked-object summaries; not a substitute for kernel checking'))
        code=run('kernel',[str(PREFIX/'bin/coqchk')]+args+['-o']+['handoff.'+m for m in own_modules],d,env,sources)
        require_check(code==0, 'Required proof/artifact check failed at original line 133')
        text=(d/'kernel.log').read_text()
        checked=set(re.findall(r'^Checking library: (\S+)\s*$',text,re.M))
        own=set(re.findall(r'^\s*checking cst:(handoff\.\S+)\s*$',text,re.M))
        require_check(checked==expected, (checked-expected,expected-checked))
        require_check(own==set(declarations), (own-set(declarations),set(declarations)-own))
        require_check('Modules were successfully checked' in text, 'Required proof/artifact check failed at original line 139')
        objects={}
        for m in re.finditer(r'\[intern (.*?) \.\.\.',text):
            p=Path(m.group(1)); p=p if p.is_absolute() else d/p
            objects[str(p)]=dict(object_sha256=sha(p),source_sha256=sha(p.with_suffix('.v')) if p.with_suffix('.v').exists() else None)
        save(d/'loaded-objects.json',objects)
        save(d/'acceptance.json',dict(declarations=len(declarations),all_individually_closed=True,
             checked_modules=sorted(checked),objects=len(objects),kernel_context=text[text.rfind('==============='):],
             compiled_sources={m:sha(d/(m+'.v')) for m in own_modules},
             compiled_objects={m:sha(d/(m+'.vo')) for m in own_modules}))
    require_check(foundation(full=(mode=='accept'))==inputs, 'Required proof/artifact check failed at original line 149')
    products={str(p.relative_to(d)):sha(p) for p in d.iterdir() if p.is_file()}
    save(d/'products.json',products)
    sys.exit(code)
if __name__=='__main__':
    try: main()
    except SystemExit as error:
        d=ROOT/sys.argv[1]
        if d.exists() and not (d/'controller-exit.json').exists():
            save(d/'controller-exit.json',dict(exit_status=error.code,state='finished',time=time.time()))
        raise
    except BaseException as error:
        import traceback
        d=ROOT/sys.argv[1]
        if d.exists():
            save(d/'controller-exit.json',dict(error=repr(error),traceback=traceback.format_exc(),
                 controller_sha256=CONTROLLER_SHA,time=time.time()))
        raise
