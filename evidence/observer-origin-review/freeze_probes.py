"""Create all adversarial inputs before any probe is run (no production writes)."""
import json, hashlib, shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
RAW=Path('/tmp/observer-origin-review-d35c31fa')
repros=HERE/'repros'; repros.mkdir(exist_ok=True)
cases=[]
def add(label,text,execs=('client',),on='success',off='success',selection=()):
    p=repros/(label+'.rs'); p.write_text(text.rstrip()+'\n')
    copy=RAW/'probes/sources'/p.name;copy.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,copy)
    cases.append(dict(label=label,source=str(copy),committed_source=str(p.relative_to(ROOT)),sha256=hashlib.sha256(p.read_bytes()).hexdigest(),execs=list(execs),proofs=[],helpers=[],on=on,off=off,selection=list(selection) if selection is not None else None))
def simple(body,params='buf: &mut [u8;10]',helpers=''):
    return 'use vstd::prelude::*;\nverus! {\n'+helpers+'\npub fn client('+params+') {\n'+body+'\n}\n}\n'
loop='for b in buf.iter_mut() { *b = 1; }'
for p in sorted((ROOT/'evidence/shared-buffer-fix-review/repros').glob('*.rs')):
    if p.stem in ['skipped-lifetime','skipped-safety','unrelated-prefix','numeric-observer-composition','ghost-read']:continue
    names={'before-write':['before_write'],'after-write':['after_write'],'helper-before':['write','client'],'alias-after':['write','client'],'projection-helper':['write','client'],'returned-helper':['id','client']}.get(p.stem,['client'])
    add('prior-'+p.stem,p.read_text(),execs=names,selection=('client',) if p.stem=='simple' else ())
add('tuple-move-alias',simple(loop+' let (alias,) = (buf,); alias[0]=42;'))
add('match-reborrow-alias',simple(loop+' match &mut *buf { alias => {alias[0]=42;} }'))
add('tuple-ref-mut-alias',simple(loop+' let (ref mut alias,) = (*buf,); alias[0]=42;'),selection=('client',)) # copies array: unrelated value
add('target-scalar-before',simple('let _x=buf[0]; '+loop),selection=('client',))
add('target-array-copy-after',simple(loop+' let _x=*buf;'),selection=('client',))
add('index-local-before',simple('let _x=other[{buf[0]=42; 0}]; '+loop,'buf:&mut[u8;10], other:&[u8;10]'))
add('index-target-before',simple('let _x=buf[{buf[0]=42; 0}]; '+loop))
add('index-local-after',simple(loop+' let _x=other[{buf[0]=42; 0}];','buf:&mut[u8;10], other:&[u8;10]'))
add('temporary-copy-before',simple('let _x=({buf[0]=42; *other})[0]; '+loop,'buf:&mut[u8;10], other:&[u8;10]'))
add('temporary-copy-after',simple(loop+' let _x=({buf[0]=42; *other})[0];','buf:&mut[u8;10], other:&[u8;10]'))
write='fn write(buf:&mut[u8;10])->(r:usize) ensures r==0 {buf[0]=42; 0}'
add('index-helper-before',simple('let _x=other[write(buf)]; '+loop,'buf:&mut[u8;10], other:&[u8;10]',write),execs=('write','client'))
add('constructor-effect-before',simple('for b in {buf[0]=42; buf.iter_mut()} {*b=1;}'))
add('constructor-effect-nested',simple('for b in {let _x=other[{buf[0]=42;0}]; buf.iter_mut()} {*b=1;}','buf:&mut[u8;10],other:&[u8;10]'))
base=(ROOT/'source/tools/research/mutable_buffer_goal/constructor-loop-mismatch.rs').read_text()
add('constructor-mismatch',base,execs=('make','client'))
add('constructor-mismatch-nested',base.replace('let _unused = buf.iter_mut();\n    for b in make(other)','for b in { let _unused = buf.iter_mut(); make(other) }'),execs=('make','client'))
add('constructor-mismatch-conditional',base.replace('other: &mut [u8; 10]) {','other: &mut [u8; 10], third: &mut [u8; 10], choose: bool) {').replace('for b in make(other)','for b in if choose {make(other)} else {make(third)}'),execs=('make','client'))
add('constructor-mismatch-zip-unused',base.replace('other: &mut [u8; 10]) {','other: &mut [u8; 10], key:&[u8]) {').replace('buf.iter_mut();','buf.iter_mut().zip(key);'),execs=('make','client'))
add('constructor-mismatch-shadow',base.replace('for b in make(other) { *b = 1; }','for b in {let buf=other; make(buf)} { *b=1; }'),execs=('make','client'))
add('multiple-buffers-loops',simple(loop+' for b in other.iter_mut(){*b=2;}','buf:&mut[u8;10],other:&mut[u8;10]'))
add('multiple-buffers-unused',simple('let _unused=other.iter_mut(); '+loop,'buf:&mut[u8;10],other:&mut[u8;10]'))
add('binder-shadow',simple('let VERUS_iter_init=1u8; let VERUS_iter_exec=2u8; let VERUS_it=3u8; '+loop+' let _x=VERUS_iter_init+VERUS_iter_exec+VERUS_it;'),selection=('client',))
prefix=(ROOT/'source/tools/research/buffer_observation_goal/prefix-zip.rs').read_text().replace('fn main() {}','')
add('prefix-readonly',prefix.replace('    for (b_i, b_h)','    let _x=ipad[0]; let _all=*ipad;\n    for (b_i, b_h)'),execs=('xor_key',),off='postcondition',selection=('xor_key',))
add('prefix-key-temporary',prefix.replace('ipad.iter_mut().zip(key)','ipad.iter_mut().zip({key})'),execs=('xor_key',),off='postcondition',selection=('xor_key',))
add('prefix-unrelated-before-after',prefix.replace('key: &[u8])','key: &[u8], other: &mut [u8;10])').replace('    for (b_i, b_h)','    other[0]=42;\n    for (b_i, b_h)').replace('        *b_i ^= *b_h;\n    }','        *b_i ^= *b_h;\n    }\n    other[0]=43;'),execs=('xor_key',),off='postcondition',selection=('xor_key',))
keyfn='fn make_key(key:&[u8])->(r:&[u8]) ensures r@==key@ {key}'
add('zip-helper-key',simple('for (b,h) in buf.iter_mut().zip(make_key(key)) {*b=1;}', 'buf:&mut[u8;10],key:&[u8]',keyfn),execs=('make_key','client'),selection=None)
add('zip-key-local-reassigned',simple('let mut k=key; for (b,h) in buf.iter_mut().zip(k) {k=other; *b=1;}', 'buf:&mut[u8;10],key:&[u8],other:&[u8]'),selection=None)
add('zip-key-parameter-reassigned',simple('for (b,h) in buf.iter_mut().zip(key) {key=other; *b=1;}', 'buf:&mut[u8;10],mut key:&[u8],other:&[u8]'),selection=None)
add('zip-key-conditional',simple('for (b,h) in buf.iter_mut().zip(if choose {key} else {other}) {*b=1;}', 'buf:&mut[u8;10],key:&[u8],other:&[u8],choose:bool'),selection=None)
numeric=(ROOT/'source/tools/research/varint_canonical_goal/candidate-008.rs').read_text().replace('fn main() {}','')
add('numeric-prefix-composition',numeric+'\n'+prefix.replace('use vstd::prelude::*;',''),execs=('u64','xor_key'),off='diagnostic',selection=('u64','xor_key'))
add('invalid-overlap',simple(loop+' let a=&mut *buf; let b=&mut *buf; a[0]=1; b[0]=2;'),off='frontend',on='frontend')
add('overflow-after-skip',simple('buf[0]=42; for b in buf.iter_mut(){*b+=1;}'),off='safety',on='safety')
(RAW/'probes/freeze.json').write_text(json.dumps(cases,indent=2)+'\n')
(HERE/'probe-freeze.json').write_text(json.dumps(cases,indent=2)+'\n')
print(len(cases),'frozen probes')
supplement_source=repros/'zip-key-parameter-same-lifetime.rs'
text=(repros/'zip-key-parameter-reassigned.rs').read_text().replace('fn client(buf:&mut[u8;10],mut key:&[u8],other:&[u8])',"fn client<'a>(buf:&mut[u8;10],mut key:&'a[u8],other:&'a[u8])")
supplement_source.write_text(text)
out=RAW/'supplement';(out/'sources').mkdir(parents=True,exist_ok=True)
private=out/'sources'/supplement_source.name;shutil.copy2(supplement_source,private)
supplement=[dict(label=supplement_source.stem,source=str(private),committed_source=str(supplement_source.relative_to(ROOT)),sha256=hashlib.sha256(supplement_source.read_bytes()).hexdigest(),execs=['client'],proofs=[],helpers=[],off='success',on='success',selection=None)]
(out/'freeze.json').write_text(json.dumps(supplement,indent=2)+'\n')
(HERE/'supplement-freeze.json').write_text(json.dumps(supplement,indent=2)+'\n')
print('1 frozen lifetime-corrected supplement')
