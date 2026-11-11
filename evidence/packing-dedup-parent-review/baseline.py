"""Completed artifact audit only. No compiler execution; Python -B / -O -B."""
import hashlib,json,re,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
OLD=Path('/tmp/certified-packing-combined-review-uedtt2b_')
BUNDLE=Path('/tmp/verus-infer-buffer-cycle-20261109/parent-combined71-bundle-21eeabbb')
def need(x,m):
 if not x: raise RuntimeError(m)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def digest(t):return hashlib.sha256(t.encode()).hexdigest()
def queries(d):
 rows=[];first={}
 for p in sorted(d.glob('*-final.air')):
  text=p.read_text()
  if 'query spun off because: bitvector' not in text:continue
  owner=re.search(r';; Function-Def ([^\n]+)',text)[1]
  # Compiler-generated qid/skolemid labels are occurrence identifiers, not
  # logical expressions. Keep all typed BV nodes, literals and binder names.
  clean=re.sub(r';;[^\n]*','',text)
  clean=re.sub(r':(?:qid|skolemid) [^\s()]+','',clean)
  tokens=re.findall(r'"(?:\\.|[^"\\])*"|[()]|[^\s()]+',clean)
  key=digest(json.dumps(tokens,separators=(',',':')))
  slot=(owner,key);previous=first.setdefault(slot,p.name)
  rows.append(dict(file=p.name,owner=owner,raw_sha256=sha(p),formula_sha256=key,keep=previous==p.name,first=previous))
 return rows

def audit():
 need(sha(BUNDLE/'receipt.json')=='5c81930271eeae212f1a4d92cb3d7f226c8cf72a4028063ba01a9837e6b01ea5','bundle receipt')
 br=json.loads((BUNDLE/'receipt.json').read_text())
 for n,h in br['tools'].items():need(sha(BUNDLE/n)==h,'bundle '+n)
 result=dict(base='efd9fb5680ece75f871058677afe10c9ce7e64eb',production='21eeabbb6e0eade862798e4dbe022d69c4a564c5',bundle_receipt_sha256=sha(BUNDLE/'receipt.json'),tools=br['tools'],cases={})
 for kind in ['full','packet']:
  d=OLD/('stable-'+kind)/'cold';rows=queries(d/'vir');owners={}
  for r in rows:
   c=owners.setdefault(r['owner'],dict(proposed=0,kept=0,removed=0));c['proposed']+=1;c['kept']+=r['keep'];c['removed']+=not r['keep']
  rec=json.loads((d/'receipt.json').read_text());warm=json.loads((d.parent/'warm/receipt.json').read_text())
  result['cases'][kind]=dict(receipt_sha256=sha(d/'receipt.json'),verified=rec['verification'],cold_seconds=rec['seconds'],warm_seconds=warm['seconds'],owners=owners,queries=rows,proposed=len(rows),kept=sum(r['keep'] for r in rows),removed=sum(not r['keep'] for r in rows))
 return result
if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'use -B');j=audit();(HERE/'BASELINE.json').write_text(json.dumps(j,indent=2,sort_keys=True)+'\n')
 for k,v in j['cases'].items():print(k,{n:v[n] for n in ['proposed','kept','removed','owners']})
