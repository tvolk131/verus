"""Original source/native compiler diagnostics only; no verifier/native proof."""
import ast, hashlib, json, re, subprocess, sys, time
from pathlib import Path
HERE=Path.cwd()/'source/tools/research/nonzero_wrapper_ordering'
sys.path.insert(0,str(HERE))
import build as b
import fixtures as f
b.need(sys.flags.dont_write_bytecode,'use -B')
r=b.root()
b.need(not (r/'diagnostics.json').exists(),'immutable diagnostic label')
fixtures={}
for mode in ['original','request','authored']:
    p=r/(mode+'.rs');p.write_text(f.source(mode));fixtures[mode]=dict(path=str(p),sha256=b.sha(p))
native=r/'native.rs'
native_text=('use std::num::NonZero;\n'+f.declaration()+'''\npub fn total(a:&BorTag,b:&BorTag)->core::cmp::Ordering {a.cmp(b)}
pub fn partial(a:&BorTag,b:&BorTag)->Option<core::cmp::Ordering> {a.partial_cmp(b)}
pub fn same_reference(a:&BorTag)->(core::cmp::Ordering,Option<core::cmp::Ordering>) {(a.cmp(a),a.partial_cmp(a))}
pub fn less(a:&BorTag,b:&BorTag)->bool {a<b}
pub fn less_equal(a:&BorTag,b:&BorTag)->bool {a<=b}
pub fn greater(a:&BorTag,b:&BorTag)->bool {a>b}
pub fn greater_equal(a:&BorTag,b:&BorTag)->bool {a>=b}
''')
b.need(not native.exists() or native.read_text()==native_text,'native source drift')
if not native.exists():native.write_text(native_text)
inputs=[Path(__file__),HERE/'fixtures.py',HERE/'build.py',native,*[Path(v['path']) for v in fixtures.values()]]
records={}
for label,flags in [('expanded',['-Zunpretty=expanded']),('mir',['--emit=mir','-o',str(r/'diagnostic-final.mir')])]:
    d=r/('diagnostic-final-'+label);d.mkdir()
    argv=[str(b.TC/'bin/rustc'),str(native),'--crate-type=lib','--edition=2021',*flags]
    freeze=dict(argv=argv,cwd=str(r),environment=b.env(),inputs={str(p):b.sha(p) for p in inputs},toolchain=json.loads((r/'toolchain.json').read_text()))
    b.write(d/'freeze.json',freeze);freeze_sha=b.sha(d/'freeze.json');start=time.monotonic()
    with (d/'stdout').open('wb') as out,(d/'stderr').open('wb') as err:
        result=subprocess.run(argv,cwd=r,env=freeze['environment'],stdout=out,stderr=err)
    guards=all(b.sha(p)==h for p,h in freeze['inputs'].items()) and all(b.sha(p)==h for p,h in freeze['toolchain']['files'].items()) and b.sha(d/'freeze.json')==freeze_sha
    receipt=dict(exit=result.returncode,guards=guards,seconds=time.monotonic()-start,freeze_sha256=freeze_sha,stdout_sha256=b.sha(d/'stdout'),stderr_sha256=b.sha(d/'stderr'))
    b.write(d/'receipt.json',receipt);b.need(result.returncode==0 and guards,'native compiler diagnostic failure')
    records[label]=dict(receipt_sha256=b.sha(d/'receipt.json'),**receipt)
expanded=(r/'diagnostic-final-expanded/stdout').read_text()
b.need('impl ::core::cmp::Ord for BorTag' in expanded and 'impl ::core::cmp::PartialOrd for BorTag' in expanded,'original derived impls absent')
mir=(r/'diagnostic-final.mir').read_text();starts=list(re.finditer(r'^fn (.*?)\n',mir,re.M));sections=[]
for i,m in enumerate(starts):
    s=mir[m.start():starts[i+1].start() if i+1<len(starts) else len(mir)]
    sections.append(dict(signature=m[1],section_sha256=hashlib.sha256(s.encode()).hexdigest(),verified=False))
b.need(len(sections)==15 and sum(s['signature'].startswith('BorTag(') for s in sections)==2,'six native derive methods, seven clients, runtime/CTFE constructor sections expected')
b.write(r/'diagnostics.json',dict(fixtures=fixtures,declaration_sha256=f.DECL_SHA,native_source_sha256=b.sha(native),mir_sha256=b.sha(r/'diagnostic-final.mir'),native_MIR_sections=sections,runs=records,source_proofs=0,native_correspondences=0))
print(json.dumps(dict(root=str(r),sections=len(sections),native_correspondences=0)),flush=True)
