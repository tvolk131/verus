"""Independent arithmetic-only source controls in the private source copy."""
import review as t
import audit
import json,shutil,sys
R=t.root();P=R/'src/source/rust_verify/src/checked_foreign_packing.rs'
def health(label):
 d=R/label;j=json.loads((d/'stdout').read_text());h=j['func-details']['component::checked_foreign_bool_to_u64%'];t.need(not h['failed_proof_notes'],'healthy bool');audit.eq(d);t.need(audit.bodies(d)==audit.bodies(R/'full/cold'),'original typed bodies unchanged')
 a=json.loads((d/'freeze.json').read_text())['source'];b=json.loads((R/'full/cold/freeze.json').read_text())['source'];t.need(a.keys()==b.keys() and [k for k in a if a[k]!=b[k]]==['source/rust_verify/src/checked_foreign_packing.rs'],'sole arithmetic change')
 t.need(audit.helper(d)==audit.helper(R/'full/cold'),'identical healthy ordinary helper query')
def main(name):
 shipping=(t.ROOT/'source/rust_verify/src/checked_foreign_packing.rs').read_bytes();t.need(P.read_bytes()==shipping,'shipping start')
 source=shipping.decode();t.need(source.count('for fact in proposed {')==1,'one ablation site')
 needle='let rhs = SpannedTyped::new(&e.span, &e.typ, ExpX::Const(Constant::Int(value)));';t.need(source.count(needle)==1,'one false constant site')
 variant=source.replace('for fact in proposed {','for fact in Vec::<Exp>::new() {') if name=='ablation' else source.replace(needle,needle.replace('Int(value)','Int(value + 1)'))
 (R/(name+'.rs')).write_text(variant);shutil.copytree(R/'bundle',R/('bundle-before-'+name));P.write_text(variant)
 try:
  t.build(name+'-build');t.vstd(name+'-vstd');t.need('2250 verified, 0 errors' in (R/(name+'-vstd')/'stdout').read_text(),'vstd2250')
  for target in (['full','packet'] if name=='ablation' else ['full']):
   label=name+'-'+target;j=t.trial(label,R/target/'component.rs');t.need(j['exit']!=0 and j['verification']['errors']>0 and j['verification']['is-verifying-entire-crate'],'semantic rejection')
   if target=='full':health(label)
   if name=='false-constant':t.need('bitvector assertion not satisfied' in (R/label/'stderr').read_text(),'isolated failure propagates')
  shutil.copytree(R/'bundle',R/('bundle-'+name))
 finally:P.write_bytes(shipping)
if __name__=='__main__':main(sys.argv[1])
