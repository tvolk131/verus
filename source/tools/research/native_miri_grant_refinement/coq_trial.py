"""Normal Coq 8.9.1 compilation and recursive checking of fresh local proofs."""
import json, os, pathlib, sys, tempfile
HERE=pathlib.Path(__file__).resolve().parent
sys.dont_write_bytecode=True
sys.path.insert(0,str(HERE.parents[1]))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt, require
from replay_support import private_root
BASE=pathlib.Path('/tmp/stbor-checked-foundation-20261110')
def main():
    ready=BASE/'baseline-ready.json'; completed=BASE/'completed-build-receipt.json'
    if digest(ready)!='9259b8f303e52e38f1b47709429f2be23a81a4e44ed557b6058184a95e653e51' or digest(completed)!='0d85e9ec05f29413973137f03bc840ccb1e22170d50169b2837331e58770f649': raise ValueError('baseline authentication')
    rd=json.loads(ready.read_text()); files=[ready,completed]
    for name in ['artifact-modules.json','kernel-loaded-objects.json','upstream-tool-freeze.json']:
        entry=rd['maps'][name]; p=pathlib.Path(entry['path'])
        if digest(p)!=entry['sha256']: raise ValueError('map authentication')
        files.append(p)
    for name in ['coqc','coqchk']:
        t=rd['tools'][name]
        if digest(t['path'])!=t['sha256']: raise ValueError('tool authentication')
        files.append(pathlib.Path(t['path']))
    for obj,info in json.loads((BASE/'manifests/kernel-loaded-objects.json').read_text()).items():
        for p,h in [(obj,info['object_sha256']),(info['source'],info['source_sha256'])]:
            if digest(p)!=h: raise ValueError('loaded artifact authentication: '+p)
            files.append(pathlib.Path(p))
    for info in json.loads((BASE/'manifests/artifact-modules.json').read_text()).values():
        for p,h in [(info['object'],info['object_sha256']),(info['source'],info['source_sha256'])]:
            if digest(p)!=h: raise ValueError('historical source authentication')
            files.append(pathlib.Path(p))
    tf=json.loads((BASE/'manifests/upstream-tool-freeze.json').read_text()); prefix=pathlib.Path(tf['installed_prefix'])
    # Only installed Coq libraries/plugins and selected binaries; no old controller execution.
    for rel,h in tf['installed_files'].items():
        if rel.startswith('lib/coq/') or rel in ['bin/coqc','bin/coqchk']:
            p=prefix/rel
            if digest(p)!=h: raise ValueError('Coq dependency authentication: '+rel)
            files.append(p)
    private=private_root()
    out=pathlib.Path(tempfile.mkdtemp(prefix='coq-',dir=private)); src=out/'GrantBridge.v';src.write_bytes((HERE/'GrantBridge.v').read_bytes());controller=out/'coq_trial.py';controller.write_bytes(pathlib.Path(__file__).read_bytes());files += [src,controller,HERE/'RESUMPTION.json',HERE/'replay_support.py',HERE.parents[1]/'infer_eval_support.py',HERE.parents[1]/'infer_experiment.py']
    rec={'output':str(out),'commands':[],'source_sha256':digest(src),'controller_sha256':digest(controller)}
    (out/'input-freeze.json').write_text(json.dumps(rec,indent=2)+'\n')
    with replay_receipt(out,rec):
        guard=ArtifactGuard(files);rec['artifacts']=guard.initial
        run=LoggedCommands(out,out,rec['commands'],timeout=180,guard=guard,status_key='returncode')
        env=dict(os.environ,COQLIB=str(prefix/'lib/coq'),PATH=str(prefix/'bin')+':'+os.environ['PATH'],XDG_CACHE_HOME=str(out/'cache'),TMPDIR=str(out))
        args=['-coqlib',str(prefix/'lib/coq'),'-Q',str(BASE/'src/stacked-borrows/theories'),'stbor','-Q',str(out),'GrantRefinement']
        require(run('version',[rd['tools']['coqc']['path'],'--version'],env)==0)
        rc=run('compile',[rd['tools']['coqc']['path'],*args,str(src)],env);rec['compile_returncode']=rc
        if rc==0:
            rec['object_sha256']=digest(out/'GrantBridge.vo')
            rec['check_returncode']=run('coqchk',[rd['tools']['coqchk']['path'],*args,'GrantRefinement.GrantBridge'],env)
        require(rec['compile_returncode']==0 and rec.get('check_returncode')==0, 'normal compile/kernel failure')
        require('Modules were successfully checked' in (out/'coqchk.stderr').read_text(), 'kernel completion marker missing')
        require((out/'compile.stdout').read_text().count('Closed under the global context')==7, 'assumption coverage')
        import re
        declarations=re.findall(r'checking cst:GrantRefinement.GrantBridge\.([^\s]+)',(out/'coqchk.stderr').read_text())
        authored=re.findall(r'^(?:Definition|Lemma|Theorem) ([A-Za-z_0-9]+)',src.read_text(),re.M)
        require(len(declarations)==7 and set(declarations)==set(authored),'complete declaration coverage')
        require(not re.search(r'\b(?:Axiom|Admitted|admit|Parameter|Hypothesis)\b',src.read_text()), 'assumed bridge')
        query=out/'Inventory.v'
        lines=['From GrantRefinement Require Import GrantBridge.','Set Printing All.','Set Printing Universes.','Set Printing Depth 1000000.','Set Printing Width 1000000.']
        for i,name in enumerate(authored):
            for kind,cmd in [('type','Check'),('term','Print'),('assumptions','Print Assumptions')]:
                lines.append('Redirect "'+str(out/(str(i)+'-'+kind))+'" '+cmd+' '+name+'.')
        query.write_text('\n'.join(lines)+'\n')
        inventory_guard=ArtifactGuard([query,out/'GrantBridge.vo']);rec['inventory_inputs']=inventory_guard.initial
        require(run('inventory',[rd['tools']['coqc']['path'],*args,str(query)],env)==0,'inventory failed')
        entries={}
        for i,name in enumerate(authored):
            row={'kind':'definition' if name in ['succeeding_boundary','vector_boundary'] else 'theorem', 'source_sha256':digest(src),'object_sha256':digest(out/'GrantBridge.vo')}
            for kind in ['type','term','assumptions']:
                p=out/(str(i)+'-'+kind+'.out');require(p.is_file() and p.stat().st_size>0,'missing inventory output')
                row[kind+'_sha256']=digest(p);row[kind+'_path']=str(p)
                if kind!='term':row[kind+'_text']=p.read_text()
            require(row['assumptions_text'].strip()=='Closed under the global context','nonclosed authored item')
            entries[name]=row
        rec['declarations']=entries
        rec['kernel_libraries']=re.findall(r'Checking library: ([^\s]+)',(out/'coqchk.stderr').read_text())
        rec['kernel_constants']=re.findall(r'checking cst:([^\s]+)',(out/'coqchk.stderr').read_text())
        rec['kernel_inductive_blocks']=re.findall(r'checking mutind block: ([^\s]+)',(out/'coqchk.stderr').read_text())
        for p in out.iterdir():
            if p.suffix in ['.stdout','.stderr','.vo','.v','.out','.glob']:
                rec.setdefault('products_sha256',{})[p.name]=digest(p)
        inventory_guard.check();rec['inventory_inputs_after']=inventory_guard.snapshot()
        guard.check();rec['artifacts_after']=guard.snapshot()
    print(out);print((out/'compile.stdout').read_text());print((out/'compile.stderr').read_text());print({k:v for k,v in rec.items() if 'returncode' in k})
if __name__=='__main__':main()
