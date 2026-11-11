"""Append-only whole-crate ordinary trials, with immutable private inputs."""
import hashlib,json,re,shutil,subprocess,sys,time
from pathlib import Path
import build as b
import source as s
R=b.root();B=R/'bundle';H=b.HERE

def toolchain():
    j=json.loads((R/'toolchain.json').read_text());j['files']['/home/dev/.cargo/bin/rustup']=b.sha('/home/dev/.cargo/bin/rustup');return j

def freeze():
    p=R/'application-freeze-v2.json';b.need(not p.exists(),'immutable application freeze')
    b.need(json.loads((R/'vstd/receipt.json').read_text())['exit']==0,'vstd required')
    b.write(p,dict(source=b.manifest(),controllers=b.controllers(),tools={n:b.sha(B/n) for n in b.TOOLS},toolchain=toolchain(),before_applications=True))

def app(label,text,goals=False,options=(),fidelity=None):
    frozen=json.loads((R/'application-freeze-v2.json').read_text());b.need(frozen['source']==b.manifest(),'source drift');b.need(all(b.sha(B/n)==h for n,h in frozen['tools'].items()),'tool drift')
    p=R/('source-'+hashlib.sha256(text.encode()).hexdigest()+'.rs')
    if p.exists():b.need(p.read_text()==text,'immutable source')
    else:p.write_text(text)
    label=('v2-' if label.split('-',1)[-1] in ['forward_loop','slice_reverse_loop','forward_laws','reverse_law','reverse_decrease','reverse_remaining','reverse_inner','reverse_loop'] else 'v4-')+label
    d=R/label;d.mkdir();argv=list(map(str,[B/'verus',p,'--crate-type=lib','--crate-name','tagged','--no-cheating','--compile','-o',d/'libtagged.rlib','--output-json','--multiple-errors','30','--log-all','--log-dir',d/'vir',*(['--infer-goals'] if goals else []),*options]))
    controllers=b.controllers();archive=R/'controllers';archive.mkdir(exist_ok=True)
    for n,h in controllers.items():
        q=archive/(h+'.py')
        if q.exists():b.need(b.sha(q)==h,'controller archive')
        else:shutil.copyfile(H/n,q)
    f=dict(argv=argv,cwd=str(b.ROOT/'source'),environment=b.env(),source=b.manifest(),controllers=controllers,tools=frozen['tools'],toolchain=toolchain(),fixture={str(p):b.sha(p)},fidelity=fidelity,goals=goals)
    b.write(d/'freeze.json',f);fh=b.sha(d/'freeze.json');start=time.monotonic()
    with (d/'stdout').open('wb') as o,(d/'stderr').open('wb') as e:r=subprocess.run(argv,cwd=f['cwd'],env=f['environment'],stdout=o,stderr=e)
    guards=f['source']==b.manifest() and controllers==b.controllers() and all(b.sha(B/n)==h for n,h in f['tools'].items()) and all(b.sha(n)==h for n,h in f['toolchain']['files'].items()) and b.sha(p)==f['fixture'][str(p)] and b.sha(d/'freeze.json')==fh
    j=dict(exit=r.returncode,seconds=time.monotonic()-start,guards=guards,freeze_sha256=fh,stdout_sha256=b.sha(d/'stdout'),stderr_sha256=b.sha(d/'stderr'),source_sha256=b.sha(p),goals=goals)
    try:
        data=json.loads((d/'stdout').read_text());j.update(verification=data['verification-results'],inference=data.get('verus-infer'),native=data.get('verus-native'))
    except (ValueError,KeyError):pass
    b.write(d/'receipt.json',j);print(label,json.dumps({k:v for k,v in j.items() if k not in ['native','inference']}),flush=True);b.need(guards,'trial input drift');return j

def probes():
    for n in ['forward_loop','slice_reverse_loop','forward_laws','reverse_law','reverse_decrease','reverse_remaining','reverse_inner','reverse_loop']:
        for g in [False,True]:
            j=app(('goals-' if g else 'plain-')+n,s.probe(n),g)
            b.need('errors' in j.get('verification',{}),'probe frontend/setup failure; stop batch')

def components():
    for n in ['dependencies','fallback_plain','fallback','full_original']:
        text=s.component(n);f=s.fidelity(text,n)
        for g in [False,True]:
            j=app(('goals-' if g else 'plain-')+n,text,g,['--cfg','feature="stack-cache"'],f)
            if n=='dependencies':b.need(j.get('verification',{}).get('success') is True,'dependency health failure; stop batch')
            elif n!='full_original':b.need(j.get('verification',{}).get('errors',0)>0 or j['exit']==0,'component frontend/setup failure; stop batch')

def controls():
    text=s.component('dependencies');f=s.fidelity(text,'dependencies')
    wrong=text.replace('ensures r==(p==Permission::Unique || p==Permission::SharedReadWrite)',
        'ensures r!=(p==Permission::Unique || p==Permission::SharedReadWrite)')
    b.need(wrong!=text,'wrong property edit')
    mutation=text.replace('self != Permission::Disabled\n', 'self == Permission::Disabled\n')
    b.need(mutation!=text,'runtime mutation edit')
    for label,t in [('wrong_write_property',wrong),('mutated_grants',mutation)]:
        for g in [False,True]:app(('goals-' if g else 'plain-')+label,t,g,['--cfg','feature="stack-cache"'],dict(base=f,control=label))

if __name__=='__main__':
    b.need(sys.flags.dont_write_bytecode,'use -B')
    {'freeze':freeze,'probes':probes,'components':components,'controls':controls}[sys.argv[1]]()
