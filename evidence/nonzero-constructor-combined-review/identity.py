"""Freeze fresh native expansion and every translated original local body."""
import hashlib,re
from pathlib import Path as P
import review as e
import probes as p
W=p.W

def end(text,start):
    depth=0;quoted=False;escape=False
    for i in range(start,len(text)):
        c=text[i]
        if quoted:
            if escape:escape=False
            elif c=='\\':escape=True
            elif c=='"':quoted=False
        elif c=='"':quoted=True
        elif c=='(':depth+=1
        elif c==')':
            depth-=1
            if depth==0:return i+1
    raise RuntimeError('unbalanced expression')
def field(text,key):
    depth=0;quoted=False;escape=False
    for i,c in enumerate(text):
        if quoted:
            if escape:escape=False
            elif c=='\\':escape=True
            elif c=='"':quoted=False
        elif c=='"':quoted=True
        elif c=='(':depth+=1
        elif c==')':depth-=1
        elif depth==1 and text.startswith(':'+key+' ',i):
            start=i+len(key)+2
            return text[start:end(text,start)] if text[start]=='(' else text[start:].split()[0]
    raise RuntimeError('missing field '+key)
def main():
    d=W/'identity-v2';d.mkdir()
    source=d/'carrier.rs';source.write_text('use std::num::NonZero;\n'+p.DECL+'\n')
    provider=W/'probes-v3/macro';vir=provider/'vir/crate.vir'
    out,r=e.epoch(W,'body-identity',[e.TC/'bin/rustc',source,'--crate-type=lib','--edition=2021','-Zunpretty=expanded'],extra=[source,vir,e.UPSTREAM],envextra={'RUSTC_BOOTSTRAP':'1'},cwd=d,timeout=120)
    e.need(r['exit']==0,'native expansion failed')
    expansion=(out/'stdout').read_text();start=expansion.index('{',expansion.index('fn eq('));finish=expansion.index('}',start)+1;rust=expansion[start:finish]
    e.need(rust=='{ self.0 == other.0 }','native body changed')
    text=vir.read_text();rows=[]
    for m in re.finditer(r'\(Function\s',text):
        f=text[m.start():end(text,m.start())]
        name=field(f,'name')
        if not name.startswith('(Fun :path component::'):continue
        body=field(f,'body')
        rows.append(dict(name=name,body_sha256=hashlib.sha256(body.encode()).hexdigest(),body_present=body!='None'))
        if re.search(r'component::[^ )]+::eq\)',name):
            e.need('core::cmp::PartialEq::eq' in body,'original trait call absent')
            e.need('ReadPlace' in body,'original executable field read absent')
            (d/'original-eq.vir').write_text(body+'\n')
    e.need(sum('::eq)' in x['name'] for x in rows)==1,'eq identity ambiguous')
    e.need(sum(x['body_present'] and 'component::BorTag::ctor%)' in x['name'] for x in rows)==1,'generated constructor absent')
    result=dict(rust_generated_eq_body=rust,rust_generated_eq_body_sha256=hashlib.sha256(rust.encode()).hexdigest(),original_eq_body_file_sha256=e.sha(d/'original-eq.vir'),all_original_local_functions=rows,vir_sha256=e.sha(vir),source_sha256=e.sha(source),epoch=str(out),validation_passed=True)
    e.write(out/'validation.json',result)
    with (W/'validations.jsonl').open('a') as f:f.write(__import__('json').dumps(dict(path=str(out),validation_sha256=e.sha(out/'validation.json')))+'\n')
    print(result)
if __name__=='__main__':main()
