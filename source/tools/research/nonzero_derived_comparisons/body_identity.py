"""Freeze genuine compiler expansion and the verifier's original eq-body identity."""
import hashlib
import json
import os
import re
from pathlib import Path
import sys
import tempfile
sys.dont_write_bytecode=True
from trial import UPSTREAM,RUSTC,authenticate
sys.path.insert(0,str(Path(__file__).resolve().parents[2]))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require

def sexpr(text,start):
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
            if depth==0:return text[start:i+1]
    raise ValueError('unclosed sexpr')

provider=Path(sys.argv[1]);authenticate()
out=Path(tempfile.mkdtemp(prefix='nonzero-derive-body-',dir='/tmp'));out.chmod(0o700);print(out,flush=True)
r=dict(schema=1,commands=[])
with replay_receipt(out,r):
    original='#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]\npub struct BorTag(NonZero<u64>);'
    require(UPSTREAM.read_text().count(original)==1,'source declaration drift')
    src=out/'carrier.rs';src.write_text('use std::num::NonZero;\n'+original+'\n')
    vir=provider/'vir/crate.vir'
    inputs=[UPSTREAM,RUSTC,src,vir,provider/'receipt.json',Path(__file__)]
    guard=ArtifactGuard(inputs)
    env=dict(os.environ,RUSTUP_TOOLCHAIN='1.99.1',RUSTC_BOOTSTRAP='1',TMPDIR=str(out))
    argv=[str(RUSTC),str(src),'--crate-type=lib','--edition=2021','-Zunpretty=expanded']
    (out/'input-freeze.json').write_text(json.dumps(dict(inputs=guard.initial,argv=argv,before_first_trial=True),indent=2)+'\n');fg=ArtifactGuard([out/'input-freeze.json'])
    r['freeze_sha256']=digest(out/'input-freeze.json')
    run=LoggedCommands(out,out,r['commands'],timeout=120,guard=guard)
    try:
        rc=run('expand',argv,env);r['terminal_exit']=rc;require(rc==0,'native expansion failed')
        text=vir.read_text();matches=re.findall(r':name \(Fun :path component::[^ )]+::eq\)',text);require(len(matches)==1,'ambiguous raw eq identity');needle=matches[0]
        require(text.count(needle)==1,'ambiguous eq function')
        f=sexpr(text,text.rfind('(Function',0,text.index(needle)))
        depth=0;quoted=False;escape=False;offset=None
        for i,c in enumerate(f):
            if quoted:
                if escape:escape=False
                elif c=='\\':escape=True
                elif c=='"':quoted=False
            elif c=='"':quoted=True
            elif c=='(':depth+=1
            elif c==')':depth-=1
            elif depth==1 and f.startswith(':body ',i):offset=i+6;break
        require(offset is not None,'missing original body field')
        body=f[offset:];require(body.startswith('('),'missing original body')
        body=sexpr(body,0)
        (out/'original-eq.vir').write_text(body+'\n')
        expansion=(out/'expand.stdout').read_text()
        a=expansion.index('fn eq(');brace=expansion.index('{',a);end=expansion.index('}',brace)
        rust_body=expansion[brace:end+1];require('self.0 == other.0' in rust_body,'unexpected compiler eq body')
        r['rust_generated_eq_body']=rust_body
        r['rust_generated_eq_body_sha256']=hashlib.sha256(rust_body.encode()).hexdigest()
        r['vir_original_eq_body_sha256']=digest(out/'original-eq.vir')
        r['source_declaration_sha256']=hashlib.sha256(original.encode()).hexdigest()
    finally:
        guard.check();fg.check();r['final_guards_passed']=True
        r['products']={str(p.relative_to(out)):digest(p) for p in out.rglob('*') if p.is_file() and p.name!='receipt.json'}
