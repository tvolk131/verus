"""Read raw ordinary VIR, keep only compact typed inventories and hashes."""
import hashlib,json,re
from pathlib import Path
from source import need

def end(t,s):
    depth=0;quoted=False;escape=False
    for i in range(s,len(t)):
        c=t[i]
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
def field(t,key):
    depth=0;quoted=False;escape=False
    for i,c in enumerate(t):
        if quoted:
            if escape:escape=False
            elif c=='\\':escape=True
            elif c=='"':quoted=False
        elif c=='"':quoted=True
        elif c=='(':depth+=1
        elif c==')':depth-=1
        elif depth==1 and t.startswith(':'+key+' ',i):
            s=i+len(key)+2
            return t[s:end(t,s)] if t[s]=='(' else t[s:].split()[0]
    raise RuntimeError('missing field '+key)
def inventory(d,crate='component',log=None):
    d=Path(d);data=json.loads((d/'stdout').read_text());txt=Path(log or d/'vir/crate.vir').read_text();rows={}
    for m in re.finditer(r'\(Function\s',txt):
        f=txt[m.start():end(txt,m.start())];name=field(f,'name');path=name[len('(Fun :path '):-1]
        if not path.startswith(crate+'::'):continue
        body=field(f,'body');norm=body.replace(str(d),'@EPOCH@')
        norm=json.dumps(re.findall(r'"(?:\\.|[^"\\])*"|[()]|[^\s()]+',norm),separators=(',',':'))
        rows[path]=dict(mode=field(f,'mode'),body_present=body!='None',body_sha256=hashlib.sha256(body.encode()).hexdigest(),normalized_body_sha256=hashlib.sha256(norm.encode()).hexdigest())
    return dict(ordinary_original_functions=rows,local_function_details={k:v for k,v in data['func-details'].items() if k.startswith(crate+'::')},native_scope=data['verus-infer']['verification_scope'],helpers=data['verus-infer']['functions'],inference_interface_written=data['verus-infer']['interface_written'],failed_functions=data['verus-infer'].get('failed_functions',[]),verification=data['verification-results'])
def signature(d):
    inv=inventory(d)
    return dict(ordinary={k:{f:v[f] for f in ['mode','body_present','normalized_body_sha256']} for k,v in inv['ordinary_original_functions'].items()},function_details=sorted(inv['local_function_details']),native_functions=[{k:v for k,v in b.items() if k not in ['source','body_status']} for b in inv['native_scope']['native_functions']],helpers=[{k:h[k] for k in ['function','mode','summary_kind','parameters','result_type','contract']} for h in inv['helpers']])
if __name__=='__main__':
    import sys
    print(json.dumps(signature(sys.argv[1]),indent=2))
