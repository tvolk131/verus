"""Freeze old regressions and new adversarial inputs before any invocation."""
import json,hashlib,shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
RAW=Path('/tmp/stable-key-review-e84eed6c');out=RAW/'probes';out.mkdir(parents=True,exist_ok=True)
repros=HERE/'repros';repros.mkdir(exist_ok=True);(out/'sources').mkdir(exist_ok=True)
cases=[]
def add(label,text,execs=('client',),on='success',off='success',selection=(),proofs=()):
    p=repros/(label+'.rs');p.write_text(text.rstrip()+'\n');copy=out/'sources'/p.name;shutil.copy2(p,copy)
    cases.append(dict(label=label,source=str(copy),committed_source=str(p.relative_to(ROOT)),sha256=hashlib.sha256(p.read_bytes()).hexdigest(),execs=list(execs),proofs=list(proofs),helpers=[],on=on,off=off,selection=list(selection) if selection is not None else None))
# Exact old repro bytes, with current intended outcome recorded before rechecking.
prior=ROOT/'evidence/observer-origin-review'
for old in json.loads((prior/'probe-freeze.json').read_text())+json.loads((prior/'supplement-freeze.json').read_text()):
    name=old['label'];sel=old['selection'];on=old['on'];off=old['off']
    if name in ['zip-helper-key','zip-key-local-reassigned','zip-key-parameter-same-lifetime','zip-key-conditional']:sel=[];on='success'
    if name=='zip-key-parameter-reassigned':sel=[];on=off='frontend'
    # A braces-only expression is normalized to a bare read by the frontend.
    add('prior-'+name,(ROOT/old['committed_source']).read_text(),old['execs'],on,off,sel,old['proofs'])
def simple(body,params='buf:&mut[u8;10],key:&[u8]',helpers='',generics=''):
    return 'use vstd::prelude::*;\nverus! {\n'+helpers+'\npub fn client'+generics+'('+params+') {\n'+body+'\n}\n}\n'
loop=lambda key: 'for (b,h) in buf.iter_mut().zip('+key+') { *b=1; }'
add('immutable-local-alias',simple('let k=key; '+loop('k')))
add('same-name-immutable-shadow',simple('let key=key; '+loop('key')))
add('same-name-mutable-shadow',simple('let mut key=key; for (b,h) in buf.iter_mut().zip(key) {key=other; *b=1;}',"buf:&mut[u8;10],key:&'a[u8],other:&'a[u8]",generics="<'a>"))
add('shadow-after-constructor',simple('for (b,h) in buf.iter_mut().zip(key) {let mut key=other; key=other; *b=1;}', 'buf:&mut[u8;10],key:&[u8],other:&[u8]'),selection=('client',))
add('nested-shadow-expression',simple(loop('{let key=other; key}'),'buf:&mut[u8;10],key:&[u8],other:&[u8]'))
add('shared-reference-projection',simple(loop('*key'),'buf:&mut[u8;10],key:&&[u8]'))
add('tuple-parameter-projection',simple(loop('key.0'),'buf:&mut[u8;10],key:(&[u8],)'))
add('struct-parameter-projection',simple(loop('holder.key'),'buf:&mut[u8;10],holder:Holder', 'pub struct Holder<\'a> {pub key:&\'a[u8]}'))
add('block-local-capture',simple(loop('{let k=key; k}')))
add('match-key-expression',simple(loop('match choose {true=>key,false=>other}'),'buf:&mut[u8;10],key:&[u8],other:&[u8],choose:bool'))
add('mut-parameter-unassigned',simple(loop('key'),'buf:&mut[u8;10],mut key:&[u8]'))
add('mut-parameter-preassigned',simple('key=other; '+loop('key'),"buf:&mut[u8;10],mut key:&'a[u8],other:&'a[u8]",generics="<'a>"))
add('mut-parameter-alias-reassign',simple('for (b,h) in buf.iter_mut().zip(key) {let alias=&mut key; *alias=other; *b=1;}',"buf:&mut[u8;10],mut key:&'a[u8],other:&'a[u8]",generics="<'a>"))
add('mut-ref-parameter-reassign',simple('for (b,h) in buf.iter_mut().zip(*key) {*key=other; *b=1;}',"buf:&mut[u8;10],key:&mut &'a[u8],other:&'a[u8]",generics="<'a>"))
add('block-counter-side-effect',simple('let mut counter=0u8; '+loop('{counter=7; key}')))
helper="fn make_key<'a>(key:&'a[u8],counter:&mut u8)->(r:&'a[u8]) ensures r@==key@, *final(counter)==7 { *counter=7; key }"
add('helper-counter-side-effect',simple('let mut counter=0u8; '+loop('make_key(key,&mut counter)'),helpers=helper),execs=('make_key','client'))
helper="pub mod keys {use vstd::prelude::*; verus! {pub fn zip(key:&[u8])->(r:&[u8]) ensures r@==key@ {key}}}"
add('qualified-key-lookalike',simple(loop('keys::zip(key)'),helpers=helper),execs=('zip','client'))
add('qualified-native-zip',simple('for (b,h) in core::iter::Iterator::zip(buf.iter_mut(),key) {*b=1;}'),selection=('client',))
add('qualified-native-iter',simple('for (b,h) in <[u8]>::iter_mut(buf).zip(key) {*b=1;}'),selection=('client',))
add('mut-key-with-proof-body',simple(loop('key'),'buf:&mut[u8;10],mut key:&[u8]', 'proof fn checked_identity() { assert(1int==1int); }'),proofs=('checked_identity',))
add('mutable-skipped-false-assert',simple(loop('key')+' assert(false);','buf:&mut[u8;10],mut key:&[u8]'),on='assertion',off='assertion')
add('helper-skipped-false-post',simple(loop('make_key(key)'),helpers='fn make_key(key:&[u8])->(r:&[u8]) ensures false {key}'),execs=('make_key','client'),on='postcondition',off='postcondition')
# Frontend lifetime and borrow failures are controls only, never semantic negatives.
add('immutable-param-mutation-invalid',simple('for (b,h) in buf.iter_mut().zip(key) {let alias=&mut key; *alias=other; *b=1;}',"buf:&mut[u8;10],key:&'a[u8],other:&'a[u8]",generics="<'a>"),on='frontend',off='frontend',selection=None)
add('key-reborrow-lifetime-invalid',simple('for (b,h) in buf.iter_mut().zip(*key) {*key=other; *b=1;}','buf:&mut[u8;10],key:&mut &[u8],other:&[u8]'),on='frontend',off='frontend',selection=None)
# Preserve prefix domain while changing unsupported source shape only.
prefix=(ROOT/'source/tools/research/buffer_observation_goal/prefix-zip.rs').read_text().replace('fn main() {}','')
add('prefix-immutable-local-skip',prefix.replace('    for (b_i, b_h)','    let k=key;\n    for (b_i, b_h)').replace('zip(key)','zip(k)'),execs=('xor_key',),off='postcondition',on='postcondition')
add('prefix-mutable-param-skip',prefix.replace('key: &[u8])','mut key: &[u8])'),execs=('xor_key',),off='postcondition',on='postcondition')
(out/'freeze.json').write_text(json.dumps(cases,indent=2)+'\n');(HERE/'probe-freeze.json').write_text(json.dumps(cases,indent=2)+'\n')
print(len(cases),'frozen probes')
