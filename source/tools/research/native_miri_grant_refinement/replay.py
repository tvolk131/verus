"""Run this component's bounded evidence set. Raw outputs remain private."""
import json, pathlib, re, subprocess, sys, tempfile
from replay_support import *
from fidelity import strict_check, block_end
from controls import CONTROLS

def main():
    strict_check(HERE/'component.rs')
    out=pathlib.Path(tempfile.mkdtemp(prefix='suite-',dir=private_root()))
    base=(HERE/'component.rs').read_text()
    no_manual=base
    while (m:=re.search(r'\bproof\s*\{',no_manual)):
        i=no_manual.index('{',m.start());no_manual=no_manual[:m.start()]+no_manual[block_end(no_manual,i):]
    (out/'no_manual.rs').write_text(no_manual)
    for name,body in CONTROLS.items():(out/(name+'.rs')).write_text(base.rstrip()[:-1]+body+'}\n')
    sources={'component':HERE/'component.rs','literal':HERE/'literal.rs','no_manual':out/'no_manual.rs',**{n:out/(n+'.rs') for n in CONTROLS}}
    freeze,bundle,files=authenticate();rec={'output':str(out),'commands':[],'sources':{n:{'path':str(p),'sha256':digest(p)} for n,p in sources.items()},'fidelity':strict_check(HERE/'component.rs')}
    (out/'input-freeze.json').write_text(json.dumps(rec,indent=2)+'\n')
    with replay_receipt(out,rec):
        guard=ArtifactGuard(files+list(sources.values()));rec['artifacts']=guard.initial
        run=LoggedCommands(out,HERE,rec['commands'],timeout=180,guard=guard,status_key='returncode')
        for name,src in sources.items():
            modes=[False,True] if name in ['component','no_manual'] else [False]
            for off in modes:
                kind={'component':'positive','literal':'frontend','no_manual':'no_manual'}.get(name,'negative')
                cmd=[sys.executable,'-B',str(HERE/'trial.py'),str(src),'--kind',kind]
                if off:cmd+=['--off']
                if kind=='negative':cmd+=['--extra',name]
                label=name+('-off' if off else '-on');require(run(label,cmd,environment(bundle,out))==0,'epoch gate failed '+label)
                rec.setdefault('epochs',{})[label]=(out/(label+'.stdout')).read_text().splitlines()[0]
        for name in ['native_compare']:
            require(run(name,[sys.executable,'-B',str(HERE/(name+'.py'))],environment(bundle,out))==0,name+' failed')
            rec[name]=(out/(name+'.stdout')).read_text().splitlines()[0]
        guard.check();rec['artifacts_after']=guard.snapshot()
    print(out)
if __name__=='__main__':main()
