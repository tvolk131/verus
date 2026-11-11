"""Read-only completed worker authentication and exact formula inventory replay."""
from review import *
from baseline import audit as baseline_audit, queries
from worker_inventory import typed
E=ROOT/'source/tools/research/packing_dedup'
C=Path('/tmp/packing-dedup-hd1n_rpr'); A=Path('/tmp/packing-dedup-cb7lqob8')
def authenticate():
 seal=json.loads((E/'SEAL.json').read_text()); result=json.loads((E/'RESULTS.json').read_text())
 for n,h in seal['files'].items():need(sha(E/n)==h,'worker seal '+n)
 need(sha(E/'RESULTS.json')=='61eab46b4e15e811a351b9e80fbea1c4b0f87b2621e1d0d173476abffc2ba80a','results pinned')
 need(sha(ROOT/seal['production_file'])==seal['production_file_sha256'],'identical reviewed module')
 prereg='8c2445a0'; impl='a7ca94cf'
 subprocess.run(['git','merge-base','--is-ancestor',prereg,impl],cwd=ROOT,check=True)
 for name in ['PREREGISTERED.md','BASELINE.json']:
  historical=subprocess.check_output(['git','show',prereg+':source/tools/research/packing_dedup/'+name],cwd=ROOT)
  need(historical==(E/name).read_bytes(),'preregistered bytes '+name)
 need(baseline_audit()==json.loads((E/'BASELINE.json').read_text()),'complete baseline replay')
 total=0;fixtures=set();sources=0
 for d,toolkey,receiptkey in [(C,'tools','receipts'),(A,'ablation_tools','ablation_receipts')]:
  final=json.loads((d/'FINAL_TOOLS.json').read_text())
  need(final['tools']==result[toolkey],'tool manifest')
  for n,h in final['tools'].items():need(sha(d/'bundle'/n)==h,'frozen tool '+n)
  for n,h in final['source'].items():need(sha(d/'src'/n)==h,'frozen source '+n)
  if d==C:
   sources=len(final['source'])
   need(hashlib.sha256(json.dumps(final['source'],sort_keys=True).encode()).hexdigest()==seal['tested_source_manifest_sha256'],'canonical final source digest')
   for n,h in final['source'].items():
    data=subprocess.check_output(['git','show','b706074a:'+n],cwd=ROOT)
    need(hashlib.sha256(data).hexdigest()==h,'own Git historical shipping bytes '+n)
  for label,row in result[receiptkey].items():
   p=d/label;rec=json.loads((p/'receipt.json').read_text());f=json.loads((p/'freeze.json').read_text())
   need(sha(p/'receipt.json')==row['receipt_sha256'] and rec['guards'],'receipt '+label)
   for name in ['freeze','stdout','stderr']:need(sha(p/('freeze.json' if name=='freeze' else name))==row[name+'_sha256'],'raw receipt binding '+label+'/'+name)
   need(rec['exit']==row['exit'] and rec.get('verification')==row.get('verification'),'outcome '+label)
   need(f['source']==final['source'],'same trial source')
   if label in ['test-build','unit-build']:need(all(final['tools'][n]==h for n,h in f['tools'].items()),'test build existing prerequisite tools')
   elif label not in ['build','vstd']:need(f['tools']==final['tools'],'same frozen bundle '+label)
   for n,h in f['fixtures'].items():need(sha(n)==h,'fixture '+n);fixtures.add(n)
   total+=1
 need(total==44,'all worker receipts')
 cs=json.loads((C/'FINAL_TOOLS.json').read_text())['source']; ab=json.loads((A/'FINAL_TOOLS.json').read_text())['source']
 need(set(cs)==set(ab),'ablation source inventory keys')
 need({n for n in cs if cs[n]!=ab[n]}=={'source/rust_verify/src/checked_foreign_packing.rs'},'ablation single file')
 module='source/rust_verify/src/checked_foreign_packing.rs'; before=(C/'src'/module).read_text(); disabled=(A/'src'/module).read_text()
 needle='if closed_key(&fact).is_some_and(|key| !seen.insert(key)) {'
 need(before.count(needle)==1 and disabled==before.replace(needle,'if false && closed_key(&fact).is_some_and(|key| !seen.insert(key)) {'),'selector only ablation')
 comps={}
 for kind in ['full','packet']:
  old=queries(A/('stable-'+kind)/'cold/vir');new=queries(C/('stable-'+kind)/'cold/vir')
  baseline=json.loads((E/'BASELINE.json').read_text())['cases'][kind]['queries']
  need(old==result['comparisons'][kind]['queries_before'] and new==result['comparisons'][kind]['queries_after'],'raw AIR full inventory')
  sig=lambda rs:[(r['owner'],r['formula_sha256']) for r in rs]
  need(sig(old)==sig(baseline),'old inventory matches preregistration')
  need(sig([r for r in old if r['keep']])==sig(new),'all nonidentical formulas, order and multiplicity')
  need(typed(A/('stable-'+kind)/'cold')==typed(C/('stable-'+kind)/'cold')==result['comparisons'][kind]['typed'],'typed full function/native inventory')
  for d in [C,A]:
   comparison=json.loads((d/('stable-'+kind)/'comparison.json').read_text())
   for ep in ['cold','warm']:
    path=d/('stable-'+kind)/ep
    need({p.name:sha(p) for p in (path/'vir').glob('*.air')}==comparison['air_sha256'],'every raw AIR file '+kind+'/'+ep)
  comps[kind]=dict(original=len(old),kept=len(new),removed=[r for r in old if not r['keep']],typed_functions=len(result['comparisons'][kind]['typed']['functions']),native_classifications=len(result['comparisons'][kind]['typed']['native']))
 auth=Path('/tmp/verus-infer-buffer-cycle-20261109/packing-dedup-parent-auth-v3/receipt.json')
 need(sha(auth)=='161211d914c2d26e668f20d7bbf242da05d0f773694f51efda3cd050c6266a4f','root completed authentication')
 return dict(passed=True,source_bindings=sources,sealed_files=len(seal['files']),receipts=total,raw_logs_and_freezes=total*3,fixtures=len(fixtures),toolsets=2,comparisons=comps,worker_results_sha256=sha(E/'RESULTS.json'),root_authentication_sha256=sha(auth))
if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'python -B required')
 result=authenticate();p=HERE/'AUTHENTICATION.json'
 if '--write' in sys.argv:write(p,result)
 else:need(json.loads(p.read_text())==result,'deterministic authentication')
 print('PASS worker authentication: 44 receipts, full116->102, packet29->28')
