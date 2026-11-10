#!/usr/bin/env python3
"""Expand nested module/interface signatures omitted by Print Module's default.

This supplements (does not alter) the frozen normal-build/kernel controller.
"""
from review import *
def expand(key):
 base,_,_,prefix,logical=SPEC[key];work=ROOT/key;src=work/'src';inv=load(work/'inventory.json');out=work/'nested-v2';out.mkdir(exist_ok=True)
 done={('module',m):e['path'] for m,e in inv['signatures'].items()};pending={('module',m) for m in re.findall(r'checking module: (\S+)',(work/'kernel.log').read_text())}-set(done)
 epoch={'controller_sha256':sha(__file__),'review_sha256':sha(OWN/'review.py'),'inventory_sha256':sha(work/'inventory.json'),'kernel_sha256':sha(work/'kernel.log')};save(work/'nested-v2-pretrial.json',epoch)
 # Discover names directly from each fresh printed signature, following all
 # immediate nested modules/types until the finite qualified-name set closes.
 parsed=set();i=0;round_no=0
 while True:
  for (kind,m),path in list(done.items()):
   if (kind,m) in parsed:continue
   text=P(path).read_text();parsed.add((kind,m))
   found=list(re.finditer(r'\bModule( Type)?\s+([\w.\']+)',text))
   for mt in found[1:]:
    typ,name=mt.groups();child=m+'.'+name if '.' not in name else name
    if child!=m and (('type' if typ else 'module'),child) not in done:pending.add((('type' if typ else 'module'),child))
  pending-=set(done)
  if not pending:break
  todo=sorted(pending);pending=set();round_no+=1
  lines=['Require '+' '.join(inv['libraries'])+'.','Set Printing Width 1000000.','Set Printing All.']
  if key=='tb':lines+=['Set Printing Depth 1000000.']
  for kind,m in todo:
   dest=out/f'signature-{i}';i+=1;done[(kind,m)]=str(dest)+'.out';lines.append(f'Redirect "{dest}" Print Module '+('Type ' if kind=='type' else '')+m+'.')
  query=out/f'round-{round_no}.v';query.write_text('\n'.join(lines)+'\n')
  bin=base/prefix/'bin';args=['-q','-batch'] if key=='tb' else ['-quiet','-batch','-coqlib',str(base/prefix/'lib/coq')]
  run(key,f'nested-v2-round-{round_no}',([bin/'rocq','repl'] if key=='tb' else [bin/'coqtop'])+args+['-Q',str(src/'theories'),logical,'-l',str(query)])
  for kind,m in todo:
   p=P(done[(kind,m)]);assert p.is_file() and p.read_text().strip() and 'Error:' not in p.read_text(),(kind,m)
 save(work/'nested-inventory.json',{'epoch':epoch,'signatures':[{'kind':k,'name':m,'path':p,'sha256':sha(p)} for (k,m),p in sorted(done.items())]})
 assert sha(__file__)==epoch['controller_sha256']
 print(key,'expanded',len(done),'module/interface signatures',flush=True)
if __name__=='__main__':
 for k in (sys.argv[1:] or ['sb','tb']):
  assert k in SPEC
  expand(k)
