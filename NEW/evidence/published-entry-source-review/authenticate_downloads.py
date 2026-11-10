import base64,hashlib,json,pathlib,urllib.request
out=pathlib.Path('NEW/evidence/published-entry-source-review'); commit='58cf8f9c35b355199727627d07ae34a4e2846d67'; files=[]
for name,path in [('call.rs','compiler/rustc_const_eval/src/interpret/call.rs'),('marker.rs','library/core/src/marker.rs')]:
 url='https://api.github.com/repos/rust-lang/rust/contents/'+path+'?ref='+commit
 req=urllib.request.Request(url,headers={'User-Agent':'source-only-review','Accept':'application/vnd.github+json'})
 with urllib.request.urlopen(req) as r:meta=json.load(r)
 b=(out/name).read_bytes(); blob=hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
 if meta['sha']!=blob or base64.b64decode(meta['content'])!=b:raise ValueError(name)
 files.append(dict(path=str(out/name),upstream=path,commit=commit,sha256=hashlib.sha256(b).hexdigest(),git_blob_sha1=blob,bytes=len(b),raw_url='https://raw.githubusercontent.com/rust-lang/rust/'+commit+'/'+path,authentication_url=url))
(out/'downloaded-primary.json').write_text(json.dumps(files,indent=2)+'\n')
print('exact-commit primary API/blob/content matches:',len(files))
