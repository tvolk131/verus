import sys,shutil
import review as t
R=t.R;B=R/'bundle'
p=R/'target-verus/debug';p.parent.mkdir(exist_ok=True);
if not p.exists():p.symlink_to(B,target_is_directory=True)
for n in t.TOOLS:
 if n not in ['rust_verify','verus']:shutil.copy2(B/n,R/'target/debug'/n)
for target in ['local_checked_foreign','infer_goals','infer_nonzero_derive','constructor_function_values']:
 r=t.run(('test-local_checked_foreign-corrected' if target=='local_checked_foreign' else 'test-'+target),[t.TC/'bin/cargo','test','--offline','--locked','-j','2','-p','rust_verify_test','--test',target,'--','--test-threads=1'])
 t.need(r['exit']==0,'regression '+target)
r=t.run('test-vir',[t.TC/'bin/cargo','test','--offline','--locked','-j','2','-p','vir','--lib','--','--test-threads=2'])
t.need(r['exit']==0,'VIR tests')
