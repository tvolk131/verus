from trials import *
need(sys.flags.dont_write_bytecode,'use -B')
d=R/'native-call-identity';analyze_only=len(sys.argv)>1 and sys.argv[1]=='analyze';d.mkdir(exist_ok=analyze_only);s=fixture('native-call-identity.rs',BOOL)
args=[R/'bundle/verus',s,'--crate-name=identity','--crate-type=lib','--no-cheating','--infer-goals','--output-json','--compile','--emit=mir','-o',d/'identity.mir','--export',d/'identity.vir','--log-all','--log-dir',d/'vir']
if not analyze_only:
 r=run('native-call-identity',args);require('native-call-identity',r,2)
text=(d/'identity.mir').read_text();need('checked_foreign_bool' not in text and '<u64 as std::convert::From<bool>>::from(copy _1)' in text and text.count('::from(copy _1)')==1,'original runtime From call identity')
write(d/'inspection.json',dict(mir_sha256=sha(d/'identity.mir'),original_core_From_call_retained=True,synthetic_helper_in_native_mir=False,native_theorem=False))
