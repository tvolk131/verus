"""Only the requested suites, on the own immutable final runtime."""
from build import *
R=root()
TARGETS={'certified_foreign_bool':3,'local_checked_foreign':5,'local_checked_packing':4,'infer_goals':12,'infer_nonzero_derive':3,'constructor_function_values':13}
def artifacts():
 result={}
 for label in ['focused-test-build','unit-build-03']:
  need(json.loads((R/label/'receipt.json').read_text())['exit']==0,'test build failure')
  for line in (R/label/'stdout').read_text().splitlines():
   try:j=json.loads(line)
   except ValueError:continue
   if j.get('reason')=='compiler-artifact' and j.get('executable') and j['profile']['test']:
    name=j['target']['name']+('-unit' if j['target']['name']=='rust_verify' and 'lib' in j['target']['kind'] else '')
    result[name]=j['executable']
 return result
if __name__=='__main__':
 a=artifacts();mode=sys.argv[1]
 if mode=='vir':targets={'vir':149}
 elif mode=='units':targets={'rust_verify-unit':8}
 elif mode=='focused':targets=TARGETS
 if mode!='vir':
  # Existing harness profiles are aliases/copies of the SAME own final runtime.
  for p in [R/'target-verus/debug',R/'src/source/target-verus/release']:
   p.parent.mkdir(exist_ok=True,parents=True)
   if p.is_symlink():p.unlink()
   need(not p.exists(),'unexpected harness profile')
   p.symlink_to(bundle(),target_is_directory=True)
  for profile in [R/'target/debug',R/'build/debug']:
   for n in TOOLS:
    dst=profile/n
    if dst.exists() and sha(dst)==sha(bundle()/n):continue
    if dst.exists():dst.unlink()
    shutil.copy2(bundle()/n,dst)
  if not (R/'src/source/z3').exists():(R/'src/source/z3').symlink_to(bundle()/'z3')
 for t,count in targets.items():
  rec=run('final-test-'+t,[a[t],'--test-threads=1'])
  need(rec['exit']==0,'test failed '+t)
  out=(R/('final-test-'+t)/'stdout').read_text()
  match=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',out)
  need(match and (int(match[1]),int(match[2]),int(match[3]))==(count,0,0),'test counts '+t)
