#!/usr/bin/env python3
"""Generate private literal trials; execute none. No replacement lookup algorithms."""
import hashlib,json,re,sys
from pathlib import Path
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]

def need(b,m):
 if not b: raise ValueError(m)
def block(t,m,body=False):
 need(t.count(m)==1,'ambiguous '+m)
 a=t.index(m);b=t.index('{',a);d=1;i=b+1
 while d:d+=(t[i]=='{')-(t[i]=='}');i+=1
 return t[b:i] if body else t[a:i]
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 out=Path(sys.argv[1]).resolve();out.mkdir(exist_ok=False,parents=True)
 pins=json.loads((ROOT/'source/tools/research/native_miri_policy_pin/pins.json').read_text())
 src={}
 for k,q in pins['sources'].items():
  need(sha(q['path'])==q['sha256'],'source identity '+k);src[k]=Path(q['path']).read_text()
 stack=src['src/tools/miri/src/borrow_tracker/stacked_borrows/stack.rs']
 item=src['src/tools/miri/src/borrow_tracker/stacked_borrows/item.rs']
 bor=src['src/tools/miri/src/borrow_tracker/mod.rs']
 mod=src['src/tools/miri/src/borrow_tracker/stacked_borrows/mod.rs']
 machine=src['src/tools/miri/src/machine.rs']
  # Tuple structs have no braces; declaration is captured explicitly and audited.
 decl='#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]\npub struct BorTag(NonZero<u64>);\n'
 tag=decl+'impl BorTag {\n'+block(bor,'pub fn new(i: u64)')+'\n'+block(bor,'pub fn get(&self)')+'\n}\n'
 types='#[derive(Copy,Clone,Hash,PartialEq,Eq)]\npub struct Item(u64);\n'
 types+='#[derive(Copy,Clone,Debug,Hash,PartialEq,Eq,Structural)]\n'+block(item,'pub enum Permission')+'\n'
 types+='#[derive(Copy,Clone,Debug,Hash,PartialEq,Eq,Structural)]\n'+block(bor,'pub enum AccessKind')+'\n'
 types+='#[derive(Copy,Clone)]\n'+block(machine,'pub enum ProvenanceExtra')+'\n'
 types+='const TAG_MASK:u64=u64::MAX>>3;\nconst PERM_MASK:u64=0x3<<61;\nconst PERM_SHIFT:u64=61;\n'
 types+='impl Item {\n'+block(item,'pub fn tag(self)')+'\n'+block(item,'pub fn perm(self)')+'\n}\n'
 types+='impl Permission { const UNIQUE:u64=0;const SHARED_READ_WRITE:u64=1;const SHARED_READ_ONLY:u64=2;const DISABLED:u64=3;\n'+block(item,'fn from_bits(')+'\n'+block(mod,'fn grants(')+'\n}\n'
 fields=block(stack,'pub struct Stack')
 cache=block(stack,'struct StackCache')
 add=block(stack,'fn add(')
 lookup='\n'.join((('#[cfg(feature = \"stack-cache\")]\n' if m == 'fn find_granting_cache(' else '')+block(stack,m).replace('pub(super) fn','pub fn')) for m in ['pub(super) fn find_granting(','fn find_granting_tagged(','fn find_granting_cache('])
 literal='#![feature(rustc_private)]\nextern crate rustc_data_structures;\nuse rustc_data_structures::fx::FxHashSet;\nuse std::num::NonZero;\nuse std::ops::Range;\nuse vstd::prelude::*;\nverus! {\n'+tag+types+'const CACHE_LEN:usize=32;\n'+fields+'\n'+cache+'\nimpl StackCache {\n'+add+'\n}\nimpl Stack {\n'+lookup+'\n}\n}\n'
 cases=[]
 def put(n,t,extra=None):
  p=out/(n+'.rs');p.write_text(t);cases.append({'name':n,'source':str(p),'source_sha256':sha(p),'extra':extra or []})
 put('literal-enabled',literal,['--cfg','feature="stack-cache"'])
 put('literal-no-cache',literal)
 # Dependency isolation retains genuine NonZero and all original selected bodies.
 put('literal-tag', 'use vstd::prelude::*;\nuse std::num::NonZero;\nverus! {\n'+tag+'}\n')
 # Cache add projection observes no fields or types other than original packed Item.
 put('cache-add-literal','use vstd::prelude::*;\nverus! {\n#[derive(Copy,Clone,Hash,PartialEq,Eq)] pub struct Item(u64);\nconst CACHE_LEN:usize=32;\n'+cache+'\nimpl StackCache {\n'+add+'\n}\n}\n')
 # Isolate the original iterator form without importing compiler-private libraries.
 # This is only a dependency probe: no tag/set/body substitution and no lookup claim.
 put('iterator-isolation','use vstd::prelude::*;\nverus! {\npub fn probe(v: &Vec<u64>) -> Option<usize> { v.iter().enumerate().rev().find_map(|(idx, item)| { if *item > 0 { Some(idx) } else { None } }) }\n}\n')
 plan={'schema':1,'epoch':'literal-001','cases':cases,'classification':'frontend_or_baseline_only','semantic_controls':False}
 (out/'plan.json').write_text(json.dumps(plan,indent=2)+'\n')
 print(out/'plan.json')
if __name__=='__main__':main()
