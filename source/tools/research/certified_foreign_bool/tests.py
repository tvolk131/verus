from run import *
need(sys.flags.dont_write_bytecode,'use -B')
p=R/'target-verus/debug';p.parent.mkdir(exist_ok=True)
if not p.exists():p.symlink_to(R/'bundle',target_is_directory=True)
for n in TOOLS:
 for profile in [R/'target/debug',R/'build/debug']:
  dst=profile/n;src=R/'bundle'/n
  if dst.exists() and sha(dst)==sha(src):continue
  if dst.exists():dst.unlink()
  shutil.copy2(src,dst)
  need(sha(dst)==sha(src),'test tool mismatch '+str(dst))
write(R/'test-harness-final-products.json',{str(profile/n):sha(profile/n) for profile in [R/'target/debug',R/'build/debug'] for n in TOOLS})
solver=ROOT/'source/z3'
need(not solver.exists() or solver.resolve()==R/'bundle/z3','own solver link')
if not solver.exists():solver.symlink_to(R/'bundle/z3')
for target in sys.argv[1:] or ['certified_foreign_bool','local_checked_foreign','infer_goals','infer_nonzero_derive','constructor_function_values','infer']:
 p=run('test-'+target,[TC/'bin/cargo','test','--offline','--locked','-j','2','-p','rust_verify_test','--test',target,'--','--test-threads=1'],{'VERUS_INFER_CACHE':str(R/'test-candidate-cache')})
 need(p['exit']==0,'regression '+target)
