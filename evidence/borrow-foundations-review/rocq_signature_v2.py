#!/usr/bin/env python3
"""Read the Rocq environment in its documented Interp phase; no proof state writes."""
from review import *
base,_,_,prefix,logical=SPEC['tb'];w=ROOT/'tb';old=w/'signature-api-v1';out=w/'signature-api-v2';out.mkdir(exist_ok=False);inv=load(w/'inventory.json')
checked(old/'review_signature_walk.ml',load(w/'signature-api-pretrial.json')['generated_source_sha256'])
text=(old/'review_signature_walk.ml').read_text().replace(json.dumps(str(old)),json.dumps(str(out)))
text=text.replace('let () =\n  output_string index','let inspect () =\n  output_string index')
text+='\nlet () = Mltop.declare_cache_obj_full (Mltop.interp_only_obj inspect) "review_signature_walk"\n'
source=out/'review_signature_walk.ml';source.write_text(text)
meta=out/'findlib/review_signature_walk/META';meta.parent.mkdir(parents=True);meta.write_text('directory = '+json.dumps(str(out))+'\nrequires = "rocq-runtime.library"\nplugin(native) = "review_signature_walk.cmxs"\n')
query=out/'inspect.v';query.write_text('Require '+' '.join(inv['libraries'])+'.\nDeclare ML Module "review_signature_walk".\n')
save(w/'signature-api-v2-pretrial.json',{'controller_sha256':sha(__file__),'source_sha256':sha(source),'descriptor_sha256':sha(meta),'query_sha256':sha(query),'kernel_receipt_sha256':sha(w/'kernel.json'),'original_generated_source_sha256':sha(old/'review_signature_walk.ml')})
b=base/prefix/'bin';plugin=out/'review_signature_walk.cmxs'
run('tb','signature-api-v2-compile',[b/'ocamlfind','ocamlopt','-rectypes','-thread','-package','rocq-runtime.toplevel','-shared',source,'-o',plugin])
run('tb','signature-api-v2-query',['/usr/bin/env','OCAMLPATH='+str(out/'findlib'),b/'rocq','repl','-q','-batch','-Q',str(w/'src/theories'),logical,'-l',query])
rows=(out/'declarations.tsv').read_text().splitlines();assert rows[0]=='name\tkind\toffset\tbytes';counts={}
for row in rows[1:]:
 n,k,o,l=row.split('\t');counts[k]=counts.get(k,0)+1;assert int(o)>=0 and int(l)>0
save(w/'signature-api-inventory.json',{'rows':len(rows)-1,'counts':counts,'libraries':inv['libraries'],'index':str(out/'declarations.tsv'),'index_sha256':sha(out/'declarations.tsv'),'types':str(out/'types.marshal'),'types_sha256':sha(out/'types.marshal'),'plugin_sha256':sha(plugin),'source_sha256':sha(source),'controller_sha256':sha(OWN/'signatures.py'),'template_sha256':sha(OWN/'signature_walk.template'),'loader_sha256':sha(__file__),'loader_source':str(OWN/'rocq_signature_v2.py'),'notice':'Typed read-only environment walk in documented Interp callback. No proof declarations, instantiations or checker bypass. Private findlib loading descriptor; no installation. /$arg and /$result are lexical functor contexts. Every range binds exact marshaled types/context. Opaque proof terms are bound by .vo identity.'})
print('tb complete structural signature rows',len(rows)-1,flush=True)
