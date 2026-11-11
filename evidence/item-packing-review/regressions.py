"""Focused compiler and complete VIR regressions against fresh private shipping tools."""
import review as t
import shutil,json
R=t.root()
def main():
 names=['constructor_function_values','infer_goals','infer_nonzero_derive','local_checked_foreign','local_checked_packing']
 args=[t.TC/'bin/cargo','test','--offline','--locked','-j','2','-p','rust_verify_test','--no-run','--message-format=json']
 for n in names:args.extend(['--test',n])
 j=t.run('focused-build',args);t.need(j['exit']==0,'focused build')
 executables={}
 for line in (R/'focused-build/stdout').read_text().splitlines():
  a=json.loads(line)
  if a.get('reason')=='compiler-artifact' and a.get('executable'):executables[a['target']['name']]=a['executable']
 t.need(set(executables)==set(names),'exact focused binaries')
 for profile in [R/'target-verus/debug',R/'cargo-build/debug']:
  profile.mkdir(parents=True,exist_ok=True)
  for n in t.TOOLS:shutil.copy2(R/'bundle'/n,profile/n)
  (profile/'verus-root').touch()
 # Only the private source copy is linked; own checkout stays untouched.
 link=R/'src/source/z3';t.need(not link.exists(),'private solver link absent');link.symlink_to(R/'bundle/z3')
 try:
  for n in names:
   j=t.run('tests-'+n,[executables[n],'--test-threads=2'],extra={'VERUS_TEST_LOG_DIR':str(R/'focused-test-logs')});t.need(j['exit']==0,'focused '+n)
 finally:link.unlink()
 j=t.run('tests-vir',[t.TC/'bin/cargo','test','--offline','--locked','-j','2','-p','vir','--lib','--','--test-threads=2']);t.need(j['exit']==0,'VIR')
 t.write(t.HERE/'TEST_BINARIES.json',{n:dict(path=p,sha256=t.sha(p)) for n,p in executables.items()})
if __name__=='__main__':main()
