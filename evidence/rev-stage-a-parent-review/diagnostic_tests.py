"""Private test-only copy; baseline compiler/library source stays frozen."""
import json,pathlib,shutil
import build as b
import check as c
r=c.R;src=r/'diagnostic-vir-v8';shutil.copytree(r/'src/source/vir/src',src)
append=b.HERE/'independent.rs'
p=src/'infer_checks/tests/internal_representation.rs';p.write_bytes(p.read_bytes()+b'\n'+append.read_bytes())
manifest={str(p.relative_to(src)):b.sha(p) for p in src.rglob('*') if p.is_file()}
b.write(r/'diagnostic-source.json',manifest)
args=[b.TC/'bin/rustc','--crate-name','vir_review','--edition=2024','--test',src/'lib.rs','-L','dependency='+str(c.DEPS),'-C','debuginfo=0','-o',r/'diagnostic-tests-wf-final']
for crate in ['air','im','indexmap','num_bigint','num_traits','serde','serde_json','sha2','sise','vir_macros','tempfile','bincode_next']:
 ps=list(c.DEPS.glob('lib'+crate+'-*.rlib')) if crate!='vir_macros' else list(c.DEPS.glob('libvir_macros-*.so'))
 if len(ps)!=1:
  fp=c.DEPS.parent/'.fingerprint'
  base=next(fp.glob('vir-c3b90db594511844/lib-vir.json'))
  wanted={v[1]:v[3] for v in json.loads(base.read_text())['deps']}
  target=wanted.get(crate)
  if target is None:
   pattern='vir-*/test-lib-vir.json' if crate=='tempfile' else 'rust_verify-*/lib-rust_verify.json'
   bases=list(fp.glob(pattern));b.need(len(bases)==1,'unique extra fingerprint '+crate+' '+str(bases))
   target={v[1]:v[3] for v in json.loads(bases[0].read_text())['deps']}.get(crate)
  matches=[]
  for candidate in ps:
   suffix=candidate.name[3:].split('.')[0].rsplit('-',1)[1];stamps=list(fp.glob('*-'+suffix+'/lib-'+crate));stamp=stamps[0] if len(stamps)==1 else fp/'absent'
   if stamp.exists() and int.from_bytes(bytes.fromhex(stamp.read_text()),'little')==target:matches.append(candidate)
  ps=matches
 b.need(len(ps)==1,'one dependency '+crate+' '+str(ps));args+=['--extern',crate+'='+str(ps[0])]
j=c.run('independent-test-build-wf2-final',args)
b.need(j['exit']==0,'diagnostic build')
j=c.run('independent-tests-wf2-final',[r/'diagnostic-tests-wf-final','independent_review','--test-threads=2','--nocapture'])
b.need(j['exit']==0,'diagnostic tests')
b.need(all(b.sha(src/p)==h for p,h in manifest.items()),'diagnostic immutable source')
