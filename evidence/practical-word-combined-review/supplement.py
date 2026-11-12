"""Exact new ordinary helper bodies and fresh combined metadata controls."""
import os,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];O=Path('/tmp/practical-word-combined-review-5ecea5fe');B=O/'bundle';D=O/'supplement';D.mkdir(exist_ok=False)
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_experiment import LoggedCommands,replay_receipt,validate_run,require,require_exact
from infer_eval_support import ArtifactGuard,digest
word=(ROOT/'source/vstd/word.rs').read_text();sett=(ROOT/'source/vstd/set_lib.rs').read_text()
# Exact literal bodies, with only module imports supplied in a new ordinary crate.
member=word[word.index('pub open spec fn member'):word.index('pub open spec fn view')]
w=word[word.index('pub open spec fn seq_word_bit'):word.index('} // verus!')]
s=sett[sett.index('pub proof fn lemma_set_pair_observations'):sett.index('/// If `s1` is a subset of finite set')]
helper=D/'new_helpers.rs';helper.write_text('use vstd::prelude::*; use vstd::set_lib::{lemma_set_intersect_union_lens,lemma_len_subset};\nverus!{\n'+member+w+s+'\n}\n')
env=dict(os.environ,**json.loads((O/'env.json').read_text()),PYTHONDONTWRITEBYTECODE='1',VERUS_INFER_CACHE=str(D/'cache'))
receipt={'commands':[],'outcomes':[],'source_sha256':{str(helper):digest(helper)}}
with replay_receipt(D,receipt):
 run=LoggedCommands(D,D,receipt['commands'],timeout=180,guard=ArtifactGuard([helper,*[B/n for n in json.loads((B/'receipt.json').read_text())['tools']]]))
 code=run('helpers',[str(B/'verus'),str(helper),'--crate-type=lib','--no-cheating','--edition=2024','--compile','-o',str(D/'libhelpers.rlib'),'--output-json','--time'],env)
 stdout=(D/'helpers.stdout').read_text();stderr=(D/'helpers.stderr').read_text();data=validate_run(dict(returncode=code,stdout=stdout,stderr=stderr),True);inf=data['verus-infer'];require_exact(inf.get('functions',[]));require(not inf.get('functions',[]),'no helper inference')
 scope=inf['verification_scope'];actual=[b['rust_function'] for b in scope['native_functions'] if b['body_status']=='checked_proof_body']
 require(sorted(actual)==sorted(['checked_word_bits','checked_seq_bit_update','checked_seq_bit_update_at','lemma_set_pair_observations']),'exact ordinary bodies')
 receipt['outcomes'].append({'label':'helpers','verification':data['verification-results'],'scope':scope,'times':data['times-ms']})
 R=O/'combined-replay-fresh';artifact=R/'r2.vir';rlib=R/'libr2.rlib';client=ROOT/'source/tools/research/practical_redb_fedimint/redb_tree/callers.rs'
 original=artifact.read_bytes();start=original.rfind(b'{"schema":');require(start>=0 and original.endswith(b'VRINFER1'),'trailer')
 metadata=json.loads(original[start:-8]);require(metadata['proofs']==[],'empty candidate inventory')
 g=ArtifactGuard([artifact,rlib,client])
 for label,key,value in [('schema','schema',38),('precision','exactness',False)]:
  changed=dict(metadata);changed[key]=value;p=D/(label+'.vir');p.write_bytes(original[:start]+json.dumps(changed,separators=(',',':')).encode()+b'VRINFER1');output=D/(label+'-consumer.vir')
  code=run(label,[str(B/'verus'),str(client),'--crate-type=lib','--edition=2024','--no-cheating','--infer-goals','--output-json','--extern','r2_api='+str(rlib),'--import','r2_api='+str(p),'--export',str(output)],env)
  err=(D/(label+'.stderr')).read_text();require(code!=0 and not output.exists() and 'Verus Infer' in err,'metadata rejection')
  require(not any(t in err.lower() for t in ['panicked at','resource limit','timed out']),'healthy metadata control')
  receipt['outcomes'].append({'label':label,'classification':'EXPECTED_METADATA_FRONTEND_REJECTION','stderr_sha256':digest(D/(label+'.stderr'))});g.check()
print('NEW ORDINARY HELPERS AND METADATA GATES COMPLETE')
