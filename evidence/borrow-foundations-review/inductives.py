#!/usr/bin/env python3
"""Complete model binding queries from the exhaustive typed signature walk."""
from review import *
for k,(base,_,_,prefix,logical) in SPEC.items():
 w=ROOT/k;inv=load(w/'inventory.json');sig=load(w/'signature-api-inventory.json');found={}
 for row in P(sig['index']).read_text().splitlines()[1:]:
  name,kind,offset,size=row.split('\t')
  if name.startswith(logical+'.') and '/$' not in name and (kind.startswith('constant:') or kind in ['inductive','constructor','primitive_projection']):found[name]=kind
 missing=sorted(set(found)-set(inv['declarations']))
 save(w/'inductive-pretrial.json',{'controller_sha256':sha(__file__),'signature_inventory_sha256':sha(w/'signature-api-inventory.json'),'found':found,'additional_queries':missing})
 extra={};lines=['Require '+' '.join(inv['libraries'])+'.','Set Printing Width 1000000.','Set Printing All.']
 if k=='sb':lines+=['Set Printing Universes.']
 else:lines+=['Set Printing Depth 1000000.']
 out=w/'extra-inductives';out.mkdir(exist_ok=False)
 for i,n in enumerate(missing):
  extra[n]={'kind':found[n]}
  for suff,cmd in [('type','Check '+n),('assumptions','Print Assumptions '+n),('declaration','Print '+n)]:
   p=out/f'{i}-{suff}';extra[n][suff+'_path']=str(p)+'.out';lines.append(f'Redirect "{p}" {cmd}.')
 if missing:
  query=out/'queries.v';query.write_text('\n'.join(lines)+'\n');b=base/prefix/'bin'
  argv=[b/'coqtop','-quiet','-batch','-coqlib',base/prefix/'lib/coq'] if k=='sb' else [b/'rocq','repl','-q','-batch']
  run(k,'extra-inductives',argv+['-Q',str(w/'src/theories'),logical,'-l',str(query)])
  for n,e in extra.items():
   for suff in ['type','assumptions','declaration']:
    p=P(e[suff+'_path']);assert p.read_text().strip() and 'Error:' not in p.read_text(),(n,suff);e[suff+'_sha256']=sha(p)
 save(w/'extra-declarations.json',extra)
 print(k,'compiled global model bindings',len(found),'additional binding queries',len(missing),flush=True)
