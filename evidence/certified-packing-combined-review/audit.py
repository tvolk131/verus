"""Read-only acceptance gates; identical under normal and optimized Python."""
from trials import *
sys.path.insert(0,str(ROOT/'source/tools/research/item_tag_constructor_composition'))
from inventory import end,field,inventory

def field(t,key):
 depth=0;quoted=False;escape=False
 for i,c in enumerate(t):
  if quoted:
   if escape:escape=False
   elif c=='\\':escape=True
   elif c=='"':quoted=False
  elif c=='"':quoted=True
  elif c=='(':depth+=1
  elif c==')':depth-=1
  elif depth==1 and t.startswith(':'+key,i) and t[i+len(key)+1].isspace():
   a=i+len(key)+1
   while t[a].isspace():a+=1
   return t[a:end(t,a)] if t[a]=='(' else t[a:].split()[0]
 raise RuntimeError('missing field '+key)

def forms(d,crate='component'):
 f=json.loads((d/'freeze.json').read_text());paths=list(f['fixtures']);need(len(paths)==1,'one fixture');text=(d/'vir/crate.vir').read_text();out={}
 for m in re.finditer(r'\(Function\s',text):
  s=text[m.start():end(text,m.start())];n=field(s,'name');path=n[len('(Fun :path '):-1]
  if not path.startswith(crate+'::'):continue
  row={k:field(s,k) for k in ['mode','body','require','ensure','visibility','body_visibility']}
  out[path]=row
 return out,paths[0]
def token_hash(s,fixture):return hashlib.sha256(json.dumps(re.findall(r'"(?:\\.|[^"\\])*"|[()]|[^\s()]+',s.replace(fixture,'@FIXTURE@')),separators=(',',':')).encode()).hexdigest()
def helper(d,crate='component'):
 text=(d/'vir/root-final.air').read_text();a=text.index(';; Function-Def '+crate+'::checked_foreign_bool_to_u64%\n');b=text.find('\n;; Function-',a+1);return text[a:b if b>=0 else len(text)]
def main():
 need(sys.flags.dont_write_bytecode,'use -B');receipts={}
 for p in sorted(R.rglob('receipt.json')):
  # New own run receipts only. Copied registry crates may contain unrelated receipt files.
  if not (p.parent/'freeze.json').exists():continue
  j=json.loads(p.read_text());need(j['guards'],'receipt guards '+str(p));need(sha(p.parent/'freeze.json')==j['freeze_sha256'] and sha(p.parent/'stdout')==j['stdout_sha256'] and sha(p.parent/'stderr')==j['stderr_sha256'],'receipt authentication '+str(p))
  receipts[str(p.parent.relative_to(R))]=dict(j,receipt_sha256=sha(p))
 expected=['build2','vstd','test-vir','test-rust_verify-unit',*['test-'+n for n in ['certified_foreign_bool','local_checked_foreign','local_checked_packing','infer_goals','infer_nonzero_derive','constructor_function_values','fndef_types']], 'observer-cold','observer-warm','observer-false','constructor-protector-limit','packet-client','native-call-identity','cargo-provider','cargo-client','actual-target-mismatch','unused-reexport','unused-consumer','missing-transitive-typed','missing-unused-typed']
 for label in expected:need(label in receipts,'required completed trial '+label)
 from tests import TARGETS
 for t,count in {'vir':149,'rust_verify-unit':2,**TARGETS}.items():
  label='test-'+t;need(receipts[label]['exit']==0,'test exit '+t);text=(R/label/'stdout').read_text();m=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
  need(m and int(m[1])==count and int(m[2])==0 and int(m[3])==(1 if t=='fndef_types' else 0),'exact tests '+t)
 for shape in ['extra-filename','explicit-emit','single-output']:
  for stage in ['complete','parse','type']:
   label='cargo-path-'+shape+'-'+stage;need(label in receipts and (receipts[label]['exit']==0)==(stage=='complete'),'Cargo selected '+label)
   selected=R/label/('libcomponent-proof.vir' if shape=='extra-filename' else 'chosen.vir');need(selected.exists()==(stage=='complete'),'Cargo stale selection '+label)
 for cargo in ['explicit','cargo']:
  for opt in ['no-verify','no-lifetime','no-trait','no-erasure','filter','parse','type']:
   label=cargo+'-consumer-'+opt;need(label in receipts and receipts[label]['exit']!=0,'ordinary consumer refusal '+label);need(not (R/label/('libconsumer.vir' if cargo=='cargo' else 'consumer.vir')).exists(),'ordinary consumer stale output')
  for opt in ['verify-function','verify-only-module']:
   label=cargo+'-incomplete-valid-'+opt;need(label in receipts and receipts[label]['exit']!=0,'incomplete refusal '+label);need(not (R/label/('libprovider.vir' if cargo=='cargo' else 'provider.vir')).exists(),'incomplete stale')
 for boundary in ['qualified','alias','function-value','coercion']:need('boundary-'+boundary in receipts,'boundary completed')
 tools={n:sha(R/'bundle'/n) for n in TOOLS}
 artifacts=json.loads((R/'test-artifacts.json').read_text())
 for n,v in artifacts.items():need(sha(v['path'])==v['sha256'],'immutable test executable '+n)
 for label,j in receipts.items():
  if label in ['build','build2','vstd']:continue
  if label in ['test-build','unit-build']:
   f=json.loads((R/label/'freeze.json').read_text());need(all(tools[n]==h for n,h in f['tools'].items()),'immutable prerequisites '+label);continue
  f=json.loads((R/label/'freeze.json').read_text());need(f['tools']==tools,'SAME immutable tools '+label)
 for cargo in ['explicit','cargo']:
  for failure in ['failed-bv-auxiliary','erased-type-failure-v2']:
   label=cargo+'-'+failure;need(label in receipts and receipts[label]['exit']!=0,'required refusal '+label)
   need(not (R/label/('libprovider.vir' if cargo=='cargo' else 'provider.vir')).exists(),'stale output '+label)
  for opt in ['proof','parse','type','no-verify','no-lifetime','no-trait-conflicts','no-erasure-check','verify-root','verify-module-other']:
   label=cargo+'-'+opt+'-valid';need(label in receipts and receipts[label]['exit']!=0,'disabled rejection '+label);need(not (R/label/('libprovider.vir' if cargo=='cargo' else 'provider.vir')).exists(),'disabled stale '+label)
 summaries={};typed={}
 for kind in ['bool','full','packet','packet-public-api']:
  b=R/('stable-'+kind);c=b/'cold';w=b/'warm';need((b/'comparison.json').exists(),'comparison '+kind)
  cmp=json.loads((b/'comparison.json').read_text())
  for epoch in [c,w]:
   for p in (epoch/'vir').glob('*final.air'):
    if p.name!='root-final.air' and '__eq-' not in p.name:need('query spun off because: bitvector' in p.read_text() and p.read_text().count('(check-valid')==1,'real isolated BV file')
  need(json.loads((c/'freeze.json').read_text())==json.loads((w/'freeze.json').read_text()),'identical frozen inputs')
  need(helper(c)==helper(w) and helper(c).count('(check-valid')==1,'exact helper successful normal query')
  a,af=forms(c);z,zf=forms(w);need(a.keys()==z.keys(),'actual typed names '+kind);diagnostic_deltas=[]
  for n in a:
   for k in a[n]:
    if a[n][k]==z[n][k]:continue
    need(kind=='full' and n=='component::impl&%5::eq' and k=='ensure','no semantic form differences '+kind+' '+n+' '+k)
    before='@@ "'+af+':5:19: 5:31 (#18)"';after='@@ "'+zf+':4:29: 4:38 (#18)"'
    need(a[n][k].count(before)==1 and a[n][k].replace(before,after)==z[n][k],'only exact inferred Eq diagnostic span differs')
    diagnostic_deltas.append(dict(function=n,field=k,cold_sha256=hashlib.sha256(a[n][k].encode()).hexdigest(),warm_sha256=hashlib.sha256(z[n][k].encode()).hexdigest(),before=before,after=after,replacements=1))
  full_rows={n:dict(mode=v['mode'],body_present=v['body']!='None',**{k+'_sha256':token_hash(v[k],af) for k in ['body','require','ensure','visibility','body_visibility']}) for n,v in a.items()}
  if kind in ['full','packet']:
   old=Path('/tmp/item-packing-review-vr4p8_bo')/('full' if kind=='full' else 'packet')/'cold';oldforms,oldf=forms(old)
   prior=json.loads((ROOT/'evidence/item-packing-review/RESULTS.json').read_text())['receipts'][kind+'/cold'];need(sha(old/'receipt.json')==prior['receipt_sha256'] and sha(old/'freeze.json')==prior['freeze_sha256'] and sha(old/'stdout')==prior['stdout_sha256'],'committed accepted receipt authentication')
   accepted=json.loads((ROOT/'evidence/item-packing-review/INVENTORY.json').read_text())[kind]
   for n,v in oldforms.items():need(hashlib.sha256(v['body'].encode()).hexdigest()==accepted['ordinary_original_functions'][n]['body_sha256'],'committed accepted raw typed body '+n)
   need(a.keys()==oldforms.keys(),'accepted body inventory '+kind)
   current_native=inventory(c)['native_scope'];old_native=inventory(old)['native_scope']
   need(json.dumps(current_native,sort_keys=True).replace(af,'@FIXTURE@')==json.dumps(old_native,sort_keys=True).replace(oldf,'@FIXTURE@'),'accepted native owners/status/domain '+kind)
   for n,v in a.items():
    for k in ['mode','body','require','ensure','visibility','body_visibility']:need(token_hash(v[k],af)==token_hash(oldforms[n][k],oldf),'accepted original '+kind+' '+n+' '+k)
  cert=cmp['certificates'];need(cert[0]['foreign_dependencies']==cert[1]['foreign_dependencies'],'certificate C/W')
  summaries[kind]=dict(query_counts=cmp['query_counts'],helper_air_sha256=hashlib.sha256(helper(c).encode()).hexdigest(),foreign_certificate=cert[0]['foreign_dependencies'],api_summaries=[v['summaries'] for v in cert],Eq=[v['helpers'] for v in cmp['inventories']],all_air_identical=True,raw_form_diagnostic_deltas=diagnostic_deltas)
  iv=inventory(c);typed[kind]=dict(functions=full_rows,native_scope=iv['native_scope'],failed_functions=iv['failed_functions'])
  if kind=='full':need(len(full_rows)==38 and sum(v['mode']=='Exec' for v in full_rows.values())==19 and sum(v['mode']=='Spec' for v in full_rows.values())==19,'38 typed=19Exec+19Spec')
 # Required evidence survives private/unused pruning and ordinary exports.
 propagated={}
 for label,crate in [('bool-provider','provider'),('private-provider','private'),('private-client','private_client'),('reexport','bridge'),('transitive-client','third'),('unused-reexport','unused'),('unused-consumer','unused_client')]:
  cert=metadata(R/label/(crate+'.vir'))[1];need(len(cert['foreign_dependencies'])==1,'required foreign propagation '+label);propagated[label]=cert['foreign_dependencies']
 for label in ['observer-cold','observer-warm']:
  ff,_=forms(R/label);need(not any('checked_foreign_bool' in n for n in ff),'private helper pruned')
  for n,v in ff.items():
   if v['mode']=='Exec' or (v['mode']=='Spec' and 'item_word_wf' not in n):need(v['body']=='None','ordinary opacity '+n)
 need('protector' in (R/'observer-false/stderr').read_text(),'named false protector goal')
 for cargo in ['explicit','cargo']:
  j=json.loads((R/(cargo+'-failed-bv-auxiliary')/'stdout').read_text());need(j['verus-infer']['failed_functions']==[] and j['verification-results']['verified']==10 and j['verification-results']['errors']==1,'aux-only10/1')
  j=json.loads((R/(cargo+'-erased-type-failure-v2')/'stdout').read_text());need(j['verification-results']['verified']==2 and j['verification-results']['errors']==0 and 'E0308' in (R/(cargo+'-erased-type-failure-v2')/'stderr').read_text(),'erased-only2/0 E0308')
 from fixtures import fixture
 generated,_,fidelity=fixture();need(generated==(R/'full.rs').read_text(),'pure generator fidelity')
 need(sha(R/'full.rs')=='1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d','full bytes')
 need(not re.search(r'\bproof\b|\bassert\s*\(|snapshot|\bassume\s*\(|external_body|axiom', (R/'full.rs').read_text()),'proofless no extra assumptions')
 j=dict(base='4d5ce2b86ed5a176d327f9bdcd13b5c3a08b178b',production='21eeabbb6e0eade862798e4dbe022d69c4a564c5',status='accepted_bounded_certified_interface_and_composed_arithmetic',findings=[],schema=39,policy='candidate-bound-evidence-v71',cache='verus-infer-v74:checked-buffer-statements:4096',production_files={p:v['current_sha256'] for p,v in json.loads((R/'historical-authentication.json').read_text())['source'].items()},runtime_toolchain_sha256=sha(R/'runtime-toolchain.json'),strip_receipt_sha256=sha(R/'strip-receipt.json'),test_artifacts=artifacts,private_root=str(R),tools=tools,source_manifest_sha256=sha(R/'shipping-source.json'),toolchain_receipt_sha256=sha(R/'toolchain.json'),historical_authentication_sha256=sha(R/'historical-authentication.json'),fidelity=json.loads((R/'fidelity.json').read_text()),stable=summaries,propagated=propagated,receipts=receipts,excluded_setup=['build','test-rust_verify','explicit-incomplete-verify-function','stable-packet-public/run'],native_correspondences=0,publication=False,goal_complete=False)
 write(HERE/'RESULTS.json',j);write(HERE/'INVENTORY.json',typed);print('combined normal/optimized-safe audit passed:',len(receipts),'authenticated runs')
if __name__=='__main__':main()
