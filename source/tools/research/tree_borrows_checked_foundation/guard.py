"""SHA256 snapshots and explicit non-asserting source/product guards."""
import argparse, hashlib, json, pathlib, sys

def digest(p):
    if p.is_symlink(): return 'symlink:'+str(p.readlink())
    h=hashlib.sha256()
    with p.open('rb') as f:
        for b in iter(lambda:f.read(1024*1024),b''): h.update(b)
    return h.hexdigest()

def snapshot(root):
    return {str(p.relative_to(root)):digest(p) for p in sorted(root.rglob('*')) if (p.is_file() or p.is_symlink()) and '.git' not in p.relative_to(root).parts}

def canonical(x): return hashlib.sha256(json.dumps(x,sort_keys=True,separators=(',',':')).encode()).hexdigest()

def check(root,files):
    errors=[]
    for n,h in files.items():
        p=root/n
        if not (p.is_file() or p.is_symlink()) or digest(p)!=h: errors.append(n)
    if errors: raise RuntimeError('guard mismatch '+repr(errors[:30]))
    return {'guarded_files':len(files),'sha256_map':canonical(files),'matched':True}

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('action',choices=['snapshot','check']);p.add_argument('root',type=pathlib.Path);p.add_argument('manifest',type=pathlib.Path);a=p.parse_args()
    if a.action=='snapshot':
        x=snapshot(a.root);a.manifest.write_text(json.dumps(x,indent=2)+'\n');print(canonical(x),len(x))
    else: print(json.dumps(check(a.root,json.loads(a.manifest.read_text()))))
