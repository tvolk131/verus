"""Final immutable receipt index and guards. Execute only after restored shipping replay."""
import review as t
import audit as a
import json,re,hashlib
R=t.root()
def receipt(label):
 d=R/label;j=json.loads((d/'receipt.json').read_text());f=json.loads((d/'freeze.json').read_text())
 for n,key in [('freeze.json','freeze_sha256'),('stdout','stdout_sha256'),('stderr','stderr_sha256')]:t.need(t.sha(d/n)==j[key],'receipt '+label+' '+n)
 return dict(j,receipt_sha256=t.sha(d/'receipt.json'),tools=f['tools'],fixtures=f['fixtures'],source_manifest_sha256=hashlib.sha256(json.dumps(f['source'],sort_keys=True).encode()).hexdigest())
labels=[]
for d in R.iterdir():
 if d.is_dir() and (d/'receipt.json').exists():labels.append(d.name)
for name in ['full','packet','packe']:
 for p in ['cold','warm']:labels.append(name+'/'+p)
labels.extend('probes/'+p.name for p in (R/'probes').iterdir() if (p/'receipt.json').exists())
receipts={label:receipt(label) for label in sorted(labels)}
excluded={k:receipts[k] for k in ['shipping-build','shipping-build2','shipping-build3','shipping-build4']}
for label,j in receipts.items():
 if label not in excluded:t.need(j['guards'],'valid guards '+label)
expected={'ablation-full':(27,3),'ablation-packet':(2,1),'false-constant-full':(122,24),'restored-full':(146,0),'restored-packet':(32,0)}
for label,v in expected.items():
 j=receipts[label];t.need((j['verification']['verified'],j['verification']['errors'])==v,'exact outcome '+label);t.need(j['verification']['is-verifying-entire-crate'] and not j['verification']['encountered-vir-error'],'ordinary fullscope '+label)
for label in ['shipping-vstd','ablation-vstd','false-constant-vstd','restored-vstd']:t.need('2250 verified, 0 errors' in (R/label/'stdout').read_text() and receipts[label]['exit']==0,'vstd '+label)
for label,n in [('constructor_function_values',13),('infer_goals',12),('infer_nonzero_derive',3),('local_checked_foreign',5),('local_checked_packing',4),('vir',149)]:
 s=(R/('tests-'+label)/'stdout').read_text();t.need(f'test result: ok. {n} passed; 0 failed;' in s,'tests '+label)
probes=json.loads((t.HERE/'PROBES.json').read_text());t.need(len(probes)==52,'52 independent probes')
for label,row in probes.items():
 v=row['receipt']['verification'];t.need(v['is-verifying-entire-crate'] and not v['encountered-vir-error'],'semantic full scope '+label)
 if row['expected']=='conservative':
  t.need((v['verified'],v['errors'])==(78,2),'two valid omissions '+label)
  s=(R/'probes'/label/'stderr').read_text();t.need('fn edge(' in s and 'fn reversed(' in s,'exact valid omissions')
for label in ['restored-full','restored-packet']:
 target=label.removeprefix('restored-');t.need(a.bodies(R/label)==a.bodies(R/target/'cold'),'restored bodies '+target);t.need(a.helper(R/label)==a.helper(R/target/'cold'),'restored helper '+target)
 eq=a.eq(R/'restored-full')
t.need(t.manifest(t.ROOT)==t.manifest(R/'src'),'final shipping sources restored');t.need(not (R/'src/source/z3').exists(),'temporary private solver link removed')
out=dict(status='scoped_arithmetic_reduction_accepted_no_blocking_findings',base='6fb881c7485b12cd9fd2eb7f0aef66054cc41053',private_root=str(R),source_sha256=t.sha(t.ROOT/'source/rust_verify/src/checked_foreign_packing.rs'),full_sha256=t.sha(R/'full.rs'),packet_sha256=t.sha(R/'packet.rs'),tools={n:t.sha(R/'bundle'/n) for n in t.TOOLS},source=t.manifest(t.ROOT),receipts=receipts,excluded_setup=list(excluded),probe_summary=dict(finite_false=40,original_Item_false=4,supported=4,valid_conservative=4,initial_valid_conservative='probes/positive-8 retained separately'),fresh_validation=dict(vstd_per_variant=2250,focused_compiler=37,VIR=149),Eq_candidate_id=eq['candidate_id'],native=0,open='44 minimal +3 phase /101 writer effects',publication=False,goal_complete=False,certificate_and_global_reconciliation='parent pending')
t.write(t.HERE/'RESULTS.json',out);print('final receipt/source/tool/body/helper/vstd/controls guards passed;',len(receipts),'immutable receipts')
