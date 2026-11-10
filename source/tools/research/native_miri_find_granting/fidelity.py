"""Strict lexical source check for the bounded cache component; not a native theorem."""
import hashlib,json,re
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
def need(b,m):
 if not b:raise ValueError(m)
def uncomment(s):return re.sub(r'//[^\n]*|/\*.*?\*/','',s,flags=re.S)
def tokens(s):return re.findall(r'"(?:[^"\\]|\\.)*"|\w+|[^\s]',uncomment(s))
def end(s,i):
 d=1;j=i+1
 while d:
  d+=(s[j]=='{')-(s[j]=='}');j+=1
 return j
def body(s,name,annotated=False):
 need(len(re.findall(r'\bfn '+name+r'\(',s))==1,'method identity '+name)
 a=re.search(r'\bfn '+name+r'\(',s).start()
 if annotated:
  m=re.search(r'\n[ \t]*\{\n',s[a:]);need(m is not None,'body marker '+name)
  i=s.index('{',a+m.start())
 else:i=s.index('{',a)
 t=uncomment(s[i:end(s,i)])
 while (m:=re.search(r'\bproof\s*\{',t)):
  i=t.index('{',m.start());t=t[:m.start()]+t[end(t,i):]
 return t
def check(path):
 text=Path(path).read_text();p=json.loads((ROOT/'source/tools/research/native_miri_policy_pin/pins.json').read_text())
 src={}
 for n in ['item.rs','stack.rs']:
  q=p['sources']['src/tools/miri/src/borrow_tracker/stacked_borrows/'+n]
  need(hashlib.sha256(Path(q['path']).read_bytes()).hexdigest()==q['sha256'],'primary '+n)
  src[n]=Path(q['path']).read_text()
 rows={}
 for n,file in [('add','stack.rs'),('new','stack.rs'),('perm','item.rs'),('from_bits','item.rs')]:
  # Select Stack::new, not other methods in stack.rs. Item::new not selected.
  raw=src[file]
  a=body(raw,n)
  a=re.sub(r'#\[cfg\(feature = "stack-cache"\)\]','',a)
  b=body(text,n,True)
  need(tokens(a)==tokens(b),'executable drift '+n)
  rows[n]=dict(exact_executable_tokens=True,token_count=len(tokens(a)))
 for pat in [r'\bassume\s*\(',r'\badmit\s*\(',r'\bexternal_body\b',r'\baxiom\b',r'\bassume_specification\b',r'\bexec_allows_no_decreases_clause\b',r'\bunsafe\b']:
  need(re.search(pat,text) is None,'forbidden '+pat)
 for d in ['pub struct Item(u64);','pub struct BorTag(NonZero<u64>);','borrows: Vec<Item>','unknown_bottom: Option<BorTag>','cache: StackCache','unique_range: Range<usize>','items: [Item; CACHE_LEN]','idx: [usize; CACHE_LEN]','const CACHE_LEN: usize = 32;','const PERM_MASK: u64 = 0x3 << 61;','const PERM_SHIFT: u64 = 61;']:
  need(d in text,'representation '+d)
 return rows
if __name__=='__main__':print(json.dumps(check(HERE/'cache_component.rs'),indent=2))
