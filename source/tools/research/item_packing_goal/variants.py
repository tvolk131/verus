"""Preregistered source controls; own checkout and private completed outputs."""
import shutil,sys
import run as t
import check as audit
R=t.root();P=t.ROOT/'source/rust_verify/src/checked_foreign_packing.rs'
def vstd(label):
 j=t.run(label,[R/'target/debug/vstd_build',R/'bundle'],outputs=('vstd.vir','libvstd.rlib'));t.need(j['exit']==0 and '2250 verified, 0 errors' in (R/label/'stdout').read_text(),'matching vstd')
def full_health(label):
 import json
 j=json.loads((R/label/'stdout').read_text());d=j['func-details'];t.need('component::checked_foreign_bool_to_u64%' in d and not d['component::checked_foreign_bool_to_u64%']['failed_proof_notes'],'healthy bool helper')
 eq=j['verus-infer']['functions'];t.need(len(eq)==1 and eq[0]['contract']['requires']=='true','original Eq domain')
 for prop in ['soundness','maximality','exactness']:t.need(eq[0]['proofs'][prop]['status']=='proved','Eq '+prop)
 t.need(audit.source_bodies(R/label)==audit.source_bodies(R/'stable-full/cold'),'identical ordinary source bodies')
 a=json.loads((R/label/'freeze.json').read_text())['source'];b=json.loads((R/'stable-full/cold/freeze.json').read_text())['source']
 t.need(set(a)==set(b) and [k for k in a if a[k]!=b[k]]==['source/rust_verify/src/checked_foreign_packing.rs'],'only arithmetic source changed')
def variant(name):
 t.need(P.read_bytes()==(R/'shipping.rs').read_bytes(),'shipping starting bytes')
 shutil.copytree(R/'bundle',R/('bundle-before-'+name))
 P.write_bytes((R/(name+'.rs')).read_bytes());t.build('build-'+name);vstd('vstd-'+name)
 j=t.trial(name+'-full',(R/'full.rs').read_text());t.need(j['exit']!=0 and j['verification']['errors']>0,'control full rejection');full_health(name+'-full')
 if name=='ablation':
  j=t.trial(name+'-reuse',(t.HERE/'reuse.rs').read_text());t.need(j['exit']!=0 and j['verification']['errors']>0,'reuse requires arithmetic')
 else:
  t.need('bitvector assertion not satisfied' in (R/(name+'-full')/'stderr').read_text(),'isolated BV failure propagated')
 shutil.copytree(R/'bundle',R/('bundle-'+name));P.write_bytes((R/'shipping.rs').read_bytes())
if __name__=='__main__':
 t.need(sys.flags.dont_write_bytecode,'use -B')
 if not (R/'shipping.rs').exists():
  source=P.read_text();(R/'shipping.rs').write_text(source)
  t.need(source.count('for fact in proposed {')==1,'ablation site')
  (R/'ablation.rs').write_text(source.replace('for fact in proposed {','for fact in Vec::<Exp>::new() {'))
  needle='let rhs = SpannedTyped::new(&e.span, &e.typ, ExpX::Const(Constant::Int(value)));'
  t.need(source.count(needle)==1,'false-constant site')
  (R/'false-constant.rs').write_text(source.replace(needle,needle.replace('Int(value)','Int(value + 1)')))
 variant(sys.argv[1])
