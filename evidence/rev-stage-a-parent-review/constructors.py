import pathlib,shutil
import check as c
r=c.R;b=c.b
rel='source/tools/research/constructor_function_value/bortag.rs';p=r/'src'/rel;p.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(b.ROOT/rel,p)
b.write(r/'constructor-fixture.json',{rel:b.sha(p)})
b.need(c.run('existing-constructors13-final',[b.TC/'bin/cargo','test','--offline','--locked','-j','2','-p','rust_verify_test','--test','constructor_function_values','--','--test-threads=2'])['exit']==0,'constructor tests')
b.need(b.sha(p)==b.sha(b.ROOT/rel),'constructor fixture guard')
for label,args in [('target-spec',[b.TC/'bin/rustc','-Z','unstable-options','--print','target-spec-json','--target','x86_64-unknown-linux-gnu']),('compiler-version',[b.TC/'bin/rustc','-vV'])]:
 b.need(c.run(label,args)['exit']==0,label)
