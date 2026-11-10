#!/usr/bin/env python3
"""Normally compile/load a version-matched read-only signature inspection plugin."""
from review import *
def walk(key):
 base,_,_,prefix,logical=SPEC[key];w=ROOT/key;inv=load(w/'inventory.json');out=w/'signature-api-v1';out.mkdir(exist_ok=False)
 template=(OWN/'signature_walk.template').read_text()
 repl={'ROOT':json.dumps(str(out)),'LIBRARIES':'['+';'.join(json.dumps(m) for m in inv['libraries'])+']','OPEN':'open Mod_declarations' if key=='tb' else '', 'MODTYPE':'Mod_declarations.mod_type' if key=='tb' else 'fun b -> b.mod_type','LABEL':'Id.to_string' if key=='tb' else 'Label.to_string','ARITY':'p.mind_user_arity' if key=='tb' else 'p.mind_arity','EXTRACONST':'| Primitive _ -> "constant:primitive" | Symbol _ -> "constant:symbol"' if key=='tb' else '', 'EXTRAFIELD':'| SFBrules _ -> failwith "unexpected rewrite rules"' if key=='tb' else '', 'PACKET_RECORD':'''(match p.mind_record with
        | PrimRecord r -> Array.iteri (fun i id -> emit (path ^ "." ^ Id.to_string id) "primitive_projection" r.tys.(i)) r.projections
        | NotRecord | FakeRecord -> ());''' if key=='tb' else '', 'BLOCK_RECORD':'()' if key=='tb' else '''(match b.mind_record with
      | PrimRecord rs -> Array.iter (fun (_, labels, tys) -> Array.iteri (fun i id -> emit (path ^ "." ^ Label.to_string id) "primitive_projection" tys.(i)) labels) rs
      | NotRecord | FakeRecord -> ())'''}
 # Parenthesize the generic old record accessor at each call site.
 repl['MODTYPE']='(fun b -> b.mod_type)' if key=='sb' else 'Mod_declarations.mod_type'
 for k,v in repl.items():template=template.replace('@'+k+'@',v)
 assert not re.search(r'@[A-Z]+@',template)
 source=out/'review_signature_walk.ml';source.write_text(template)
 b=base/prefix/'bin';plugin=out/'review_signature_walk.cmxs'
 save(w/'signature-api-pretrial.json',{'controller_sha256':sha(__file__),'template_sha256':sha(OWN/'signature_walk.template'),'generated_source_sha256':sha(source),'inventory_sha256':sha(w/'inventory.json'),'normal_kernel_receipt_sha256':sha(w/'kernel.json')})
 run(key,'signature-api-compile',[b/'ocamlfind','ocamlopt','-rectypes','-thread','-package','rocq-runtime.toplevel' if key=='tb' else 'coq.toplevel','-shared',source,'-o',plugin])
 query=out/'inspect.v';query.write_text('Require '+' '.join(inv['libraries'])+'.\nAdd ML Path '+json.dumps(str(out))+'.\nDeclare ML Module "review_signature_walk".\n')
 argv=[b/'coqtop','-quiet','-batch','-coqlib',base/prefix/'lib/coq'] if key=='sb' else [b/'rocq','repl','-q','-batch']
 run(key,'signature-api-query',argv+['-Q',str(w/'src/theories'),logical,'-l',str(query)])
 rows=(out/'declarations.tsv').read_text().splitlines();assert rows[0]=='name\tkind\toffset\tbytes';counts={}
 for row in rows[1:]:
  n,k,o,l=row.split('\t');counts[k]=counts.get(k,0)+1;assert int(o)>=0 and int(l)>0
 save(w/'signature-api-inventory.json',{'rows':len(rows)-1,'counts':counts,'libraries':inv['libraries'],'index':str(out/'declarations.tsv'),'index_sha256':sha(out/'declarations.tsv'),'types':str(out/'types.marshal'),'types_sha256':sha(out/'types.marshal'),'plugin_sha256':sha(plugin),'source_sha256':sha(source),'controller_sha256':sha(__file__),'template_sha256':sha(OWN/'signature_walk.template'),'notice':'Inspection only, after full normal kernel acceptance. Lexical /$arg and /$result fields are uninstantiated functor contexts, not new global axioms. Every index range identifies the exact OCaml-marshaled type/context payload in types.marshal. Opaque proof bodies are bound by exact .vo identity, not reprinted here.'})
 print(key,'complete structural signature rows',len(rows)-1,flush=True)
if __name__=='__main__':
 for key in sys.argv[1:]:walk(key)
