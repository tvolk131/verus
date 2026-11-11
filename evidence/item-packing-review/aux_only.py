"""False-constant control whose ordinary return relations need no arithmetic theorem."""
import review as t
import json,shutil
R=t.root();P=R/'src/source/rust_verify/src/checked_foreign_packing.rs';shipping=P.read_bytes()
p=R/'aux-only.rs';t.need(not p.exists(),'immutable aux fixture')
p.write_text('''use vstd::prelude::*;
verus! {
const MASK:u64=3u64 << 2u64;
fn flag(b:bool)->(r:u64) ensures r==if b {1u64} else {0u64} {u64::from(b)}
fn values(x:u64)->(r:(u64,u64)) ensures r.0==x, r.1==x & MASK,
{ let masked=x & MASK; (x,masked) }
}
''')
j=t.trial('aux-only-shipping',p);t.need(j['exit']==0 and j['verification']['errors']==0,'ordinary valid source')
try:
 P.write_bytes((R/'false-constant.rs').read_bytes());old=json.loads((R/'false-constant-full/freeze.json').read_text());t.need(t.manifest(R/'src')==old['source'],'exact archived false compiler source')
 b=R/'bundle-false-constant';t.need({n:t.sha(b/n) for n in t.TOOLS}==old['tools'],'exact archived fresh source-matching tools/vstd')
 j=t.trial('aux-only-false',p,b=b);v=j['verification'];s=(R/'aux-only-false/stderr').read_text();t.need(j['exit']!=0 and v['errors']==1 and not v['encountered-vir-error'] and v['is-verifying-entire-crate'],'one ordinary required isolated failure')
 t.need('bitvector assertion not satisfied' in s and 'postcondition not satisfied' not in s and 'precondition not satisfied' not in s,'only isolated failure')
 t.write(t.HERE/'AUX_ONLY.json',dict(positive=json.loads((R/'aux-only-shipping/receipt.json').read_text()),negative=j,ordinary_return_contracts='r.0==x; r.1==x & MASK; exact flag return',isolated_failures=1,ordinary_failure_diagnostics=0))
finally:P.write_bytes(shipping)
t.need(t.manifest(R/'src')==t.manifest(t.ROOT),'shipping bytes restored')
