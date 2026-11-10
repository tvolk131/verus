#!/usr/bin/env python3
"""Rocq 9.2 local-plugin loader, using a private findlib descriptor, no install."""
from review import *
key='tb';base,_,_,prefix,logical=SPEC[key];w=ROOT/key;out=w/'signature-api-v1';inv=load(w/'inventory.json');plugin=out/'review_signature_walk.cmxs'
assert load(w/'signature-api-compile.json')['exit_code']==0
metadata=out/'findlib/review_signature_walk/META';metadata.parent.mkdir(parents=True,exist_ok=False)
metadata.write_text('directory = '+json.dumps(str(out))+'\nrequires = "rocq-runtime.library"\nplugin(native) = "review_signature_walk.cmxs"\n')
query=out/'inspect-rocq.v';query.write_text('Require '+' '.join(inv['libraries'])+'.\nDeclare ML Module "review_signature_walk".\n')
save(w/'signature-api-loader-pretrial.json',{'controller_sha256':sha(__file__),'generator_sha256':sha(OWN/'signatures.py'),'plugin_sha256':sha(plugin),'generated_source_sha256':sha(out/'review_signature_walk.ml'),'descriptor_sha256':sha(metadata),'query_sha256':sha(query)})
b=base/prefix/'bin'
run(key,'signature-api-query-rocq',['/usr/bin/env','OCAMLPATH='+str(out/'findlib'),b/'rocq','repl','-q','-batch','-Q',str(w/'src/theories'),logical,'-l',query])
rows=(out/'declarations.tsv').read_text().splitlines();assert rows[0]=='name\tkind\toffset\tbytes';counts={}
for row in rows[1:]:
 n,k,o,l=row.split('\t');counts[k]=counts.get(k,0)+1;assert int(o)>=0 and int(l)>0
save(w/'signature-api-inventory.json',{'rows':len(rows)-1,'counts':counts,'libraries':inv['libraries'],'index':str(out/'declarations.tsv'),'index_sha256':sha(out/'declarations.tsv'),'types':str(out/'types.marshal'),'types_sha256':sha(out/'types.marshal'),'plugin_sha256':sha(plugin),'source_sha256':sha(out/'review_signature_walk.ml'),'controller_sha256':sha(OWN/'signatures.py'),'template_sha256':sha(OWN/'signature_walk.template'),'loader_sha256':sha(__file__),'notice':'Read-only normal local plugin loading; no installation or proof declarations. /$arg and /$result are uninstantiated functor contexts. Index ranges bind exact marshaled type/context payloads. Opaque proof terms are bound by exact .vo identity.'})
print('tb complete structural signature rows',len(rows)-1)
