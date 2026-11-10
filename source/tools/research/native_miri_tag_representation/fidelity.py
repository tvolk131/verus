"""Token fidelity and private native comparisons; finite execution is NOT certification."""
import hashlib
import json
from pathlib import Path
import re
import tempfile
from source_selection import selection, block, require
from annotate import generate
HERE=Path(__file__).resolve().parent

def tokens(s):
 return re.findall(r'\w+|[^\s]',re.sub(r'//[^\n]*','',s))

def annotated_body(text,marker):
 start=text.index(marker)
 brace=text.index('\n    {',start)+5
 require(text[brace]=='{','annotation body anchor differs')
 depth=0
 for end in range(brace,len(text)):
  depth+=(text[end]=='{')-(text[end]=='}')
  if depth==0:
   return text[brace:end+1]
 raise ValueError('unclosed annotated body')

def erase_proofs(s):
 while 'proof {' in s:
  part=block(s,'proof {')
  s=s.replace(part,'',1)
 return s

def fidelity():
 meta,original=selection();annotated=generate()
 require(meta==json.loads((HERE/'selection.json').read_bytes()),'selection drift')
 markers=['pub fn new(i:', 'pub fn get(', 'pub fn inner(', 'pub fn one(', 'fn default(',
          'pub fn new(tag:', 'pub fn tag(', 'pub fn perm(', 'pub fn protected(',
          'pub fn set_permission(', 'fn to_bits(', 'fn from_bits(']
 result=[]
 for marker in markers:
  before=block(original,marker);before=before[before.index('{'):]
  after=erase_proofs(annotated_body(annotated,marker))
  adapter=marker=='pub fn new(i:'
  if adapter:
   after=after.replace('|n: NonZero<u64>| -> (r: BorTag) ensures r.0 == n, { BorTag(n) }','|n: NonZero<u64>| { BorTag(n) }')
   expected=before.replace('.map(BorTag)','.map(|n: NonZero<u64>| { BorTag(n) })')
  else:expected=before
  require(tokens(after)==tokens(expected),'runtime-erased body differs: '+marker)
  result.append(dict(marker=marker,exact=not adapter,
                     original_sha256=hashlib.sha256(json.dumps(tokens(before)).encode()).hexdigest(),
                     erased_sha256=hashlib.sha256(json.dumps(tokens(after)).encode()).hexdigest()))
 return dict(bodies=result,exact_bodies=11,explicit_adapters=1,
             excluded_succ='unsupported NonZero::checked_add and constructor-as-function',
             declaration_adapters=['public tuple fields and constants for contract visibility',
                                   'Permission Structural marker', 'verus namespace and return binders'],
             native_equivalence_proved=False)

HARNESS=r'''
    pub fn observe(x: u64, pi: usize, protected: bool) -> (Option<u64>, Option<u64>, bool, bool, bool, u64, u64, u64, bool) {
        let p=[Permission::Unique,Permission::SharedReadWrite,Permission::SharedReadOnly,Permission::Disabled][pi];
        let new= BorTag::new(x);
        let next=new.and_then(|t| t.succ()).map(|t|t.get());
        let got=new.map(|t|t.get());
        let basic= new.map(|t|t.inner().get()==x).unwrap_or(x==0);
        let one=BorTag::one().get()==1 && BorTag::default().get()==1;
        // Exercise the EXISTING assertion above the Item range; this is supporting
        // native behavior only, never a semantic negative or a proof of unwind.
        let constructed=if let Some(t)=new {
            std::panic::catch_unwind(|| Item::new(t,p,protected))
        } else { std::panic::catch_unwind(|| panic!("no zero BorTag")) };
        let is_normal=constructed.is_ok();
        if x!=0 { assert_eq!(is_normal,x<=TAG_MASK); }
        if let Ok(i)=constructed {
            assert_eq!(i.tag().get(),x);assert_eq!(i.perm(),p);assert_eq!(i.protected(),protected);
        }
        // Arbitrary words are allowed for the update. tag() is observed only on
        // its actual normal-return domain, not assumed total on word zero.
        let mut i=Item(x);let old_tag=x&TAG_MASK;let old_prot=x&PROTECTED_MASK;
        i.set_permission(p);
        assert_eq!(i.0&TAG_MASK,old_tag);assert_eq!(i.0&PROTECTED_MASK,old_prot);assert_eq!(i.perm(),p);
        let back=if old_tag!=0 {i.tag().get()} else {0};
        (got,next,basic,one,is_normal,i.0,back,old_prot,i.protected())
    }
    pub fn comparisons(a: u64,b:u64)->Option<(bool,bool,std::cmp::Ordering,u64)> {
        use std::hash::{Hash,Hasher};
        let a=BorTag::new(a)?;let b=BorTag::new(b)?;
        let mut h=std::collections::hash_map::DefaultHasher::new();a.hash(&mut h);
        assert_eq!(a==b,a.get()==b.get());assert_eq!(a<b,a.get()<b.get());assert_eq!(a.cmp(&b),a.get().cmp(&b.get()));
        Some((a==b,a<b,a.cmp(&b),h.finish()))
    }
'''

if __name__=='__main__':
 out=Path(tempfile.mkdtemp(prefix='native-miri-tag-fidelity-',dir='/tmp'));out.chmod(0o700)
 report=fidelity();(out/'fidelity.json').write_text(json.dumps(report,indent=2)+'\n')
 _,original=selection()
 adapter=original.replace('.map(BorTag)','.map(|n: NonZero<u64>| { BorTag(n) })')
 require(adapter!=original,'adapter unchanged')
 src='#![allow(dead_code)]\nmod original {\n'+original+HARNESS+'\n}\n'
 src+='mod adapter {\n'+adapter+HARNESS+'\n}\n'
 src+=r'''
fn main(){
 std::panic::set_hook(Box::new(|_|{}));
 let mask=u64::MAX>>3;
 let mut values=vec![0,1,2,3,mask-1,mask,mask+1,mask+2,1u64<<61,1u64<<62,1u64<<63,u64::MAX-1,u64::MAX];
 let mut seed=0xdeadbeef12345678u64;
 for _ in 0..64 {seed=seed.wrapping_mul(6364136223846793005).wrapping_add(1);values.push(seed);}
 let mut observations=0;
 for &x in &values {for p in 0..4 {for b in [false,true] {
   assert_eq!(original::observe(x,p,b),adapter::observe(x,p,b));observations+=1;
 }}}
 let mut comparisons=0;
 for &a in &values {for &b in &values {assert_eq!(original::comparisons(a,b),adapter::comparisons(a,b));comparisons+=1;}}
 println!("supporting native observations={observations}, comparison_pairs={comparisons}; NOT a source or permission certificate");
}
'''
 (out/'runtime.rs').write_text(src)
 print(out)
