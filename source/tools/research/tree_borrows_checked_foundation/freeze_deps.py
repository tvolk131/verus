"""Resolve only frozen opam actions, extract and inventory all source inputs."""
import hashlib,json,pathlib,subprocess,sys,shutil
from guard import snapshot,canonical,digest
r=pathlib.Path('/tmp/tree-borrows-checked-foundation-20261110'); own=pathlib.Path(__file__).resolve().parent
solution=json.loads((r/'dependency-solve-2.json').read_text())['solution']
pkgs=[v['install']['name']+'.'+v['install']['version'] for v in solution]
installed=subprocess.check_output(['opam','list','--switch=tb','--installed','--short','--columns=package'],text=True).split()
pkgs=sorted(set(pkgs+installed));(r/'frozen-sources').mkdir(exist_ok=True);(r/'source-manifests').mkdir(exist_ok=True)
rows={}
for pkg in pkgs:
 name=pkg.split('.')[0]; candidates=[r/'repositories'/repo/prefix/'packages'/name/pkg for repo,prefix in [('iris-dev',''),('coq-released','released'),('opam-repository','')]]
 dirs=[p for p in candidates if (p/'opam').is_file()]
 if not dirs:raise SystemExit('metadata missing '+pkg)
 d=dirs[0]; meta=(d/'opam').read_text(); mh=snapshot(d)
 entry={'package':pkg,'metadata_root':str(d),'metadata_files':mh,'opam':meta}
 if 'url {' in meta or 'extra-source ' in meta:
  dst=r/'frozen-sources'/pkg
  if not dst.exists():subprocess.run(['opam','source','--switch=tb','--dir='+str(dst),pkg],check=True)
  sm=snapshot(dst); mp=r/'source-manifests'/(pkg+'.json');mp.write_text(json.dumps(sm,indent=2)+'\n')
  entry.update(source_root=str(dst),source_files=len(sm),source_map_sha256=canonical(sm),source_manifest_sha256=digest(mp))
 rows[pkg]=entry
 print('frozen',pkg,flush=True)
(r/'dependency-inputs-full.json').write_text(json.dumps(rows,indent=2)+'\n')
(own/'dependencies.json').write_text(json.dumps(rows,indent=2)+'\n')
(r/'exact-packages.txt').write_text('\n'.join(pkg for pkg in pkgs if pkg not in installed)+'\n')
