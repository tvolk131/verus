import pathlib,shutil
import check as c
b=c.b;r=c.R
for dest in [r/'src/source/target-verus/release',r/'target-verus/debug']:
 dest.mkdir(parents=True,exist_ok=True)
 for n in b.TOOLS:shutil.copy2(c.B/n,dest/n)
 b.need(all(b.sha(dest/n)==b.sha(c.B/n) for n in b.TOOLS),'test-harness bundle binding')
for label,args in [('rust-verify-lib8-final',[b.TC/'bin/cargo','test','--offline','--locked','-j','2','-p','rust_verify','--lib','--','--test-threads=2']),('existing-constructors13',[b.TC/'bin/cargo','test','--offline','--locked','-j','2','-p','rust_verify_test','--test','constructor_function_values','--','--test-threads=2'])]:
 b.need(c.run(label,args)['exit']==0,label)
