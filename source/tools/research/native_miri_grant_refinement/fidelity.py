"""Strict lexical equality of every selected executable body after ghost erasure.

This is a source fidelity check, not a Rust/Verus semantics theorem.
"""
import json, re
from pathlib import Path
H=Path(__file__).resolve().parent

def uncomment(s):
    # These selected bodies contain no comment-like strings.
    return re.sub(r'//[^\n]*|/\*.*?\*/','',s,flags=re.S)
def block_end(s,i):
    depth=1;j=i+1
    while depth:
        depth+=(s[j]=='{')-(s[j]=='}');j+=1
    return j
def tokens(s): return re.findall(r'"(?:[^"\\]|\\.)*"|[A-Za-z_][A-Za-z_0-9]*|[0-9]+|[^\s]',uncomment(s))
def erased_body(text,name,annotated):
    m=re.search(r'(?:pub )?fn '+name+r'\b',text)
    if annotated: start=text.index('\n{',m.start())+1
    else: start=text.index('{',m.start())
    s=uncomment(text[start:block_end(text,start)])
    while (m:=re.search(r'\bproof\s*\{',s)):
        i=s.index('{',m.start());s=s[:m.start()]+s[block_end(s,i):]
    if annotated and name=='find_first_write_incompatible':
        s=re.sub(r'(while let Some\(item\) = self.get\(idx\))\s+invariant\b.*?\n\s*\{',r'\1 {',s,flags=re.S)
    return s

def check(path):
    selected=json.loads((H/'selected.json').read_text()); text=Path(path).read_text(); rows={}
    for name,original in selected.items():
        a=erased_body(original,name,False);b=erased_body(text,name,True)
        if tokens(a)!=tokens(b): raise ValueError('executable body drift: '+name+'\n'+b)
        rows[name]={'exact_executable_tokens':True,'token_count':len(tokens(a))}
    for forbidden in [r'\bassume\s*\(',r'\badmit\s*\(',r'\bexternal_body\b',r'\baxiom\b',r'\bassume_specification\b',r'\bexec_allows_no_decreases_clause\b']:
        if re.search(forbidden,text):raise ValueError('forbidden construct: '+forbidden)
    for definition in ['const PERM_MASK: u64 = 0x3 << 61;', 'const PROTECTED_MASK: u64 = 0x1 << 63;', 'const PERM_SHIFT: u64 = 61;', 'pub struct Item(pub u64);']:
        if definition not in text:raise ValueError('representation drift: '+definition)
    return rows
if __name__=='__main__': print(json.dumps(check(H/'component.rs'),indent=2))
