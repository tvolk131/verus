"""Fresh focused replay plus independent ordinary compiler probes."""
import json,os,sys,subprocess
from pathlib import Path
R=Path(__file__).resolve().parents[2]; E=Path(__file__).resolve().parent; O=Path('/tmp/practical-set-review-902041b3'); B=O/'bundle'
sys.path.insert(0,str(R/'source/tools'));from infer_experiment import LoggedCommands,replay_receipt,validate_run
from infer_eval_support import digest,ArtifactGuard
assert (B/'vstd.vir').exists() and json.loads((O/'receipt.json').read_text())['success']
env=dict(os.environ);env.update(json.loads((O/'env.json').read_text()));env['PATH']='/home/dev/.cargo/bin:'+env['PATH'];env['VERUS_INFER_CACHE']=str(O/'independent-cache')
D=O/'independent';D.mkdir(exist_ok=False);receipt={'commands':[],'outcomes':[],'inputs':{}}
with replay_receipt(D,receipt):
 guard=ArtifactGuard([B/n for n in ['verus','rust_verify','vstd.vir','libvstd.rlib','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','z3']]);run=LoggedCommands(D,D,receipt['commands'],timeout=180,guard=guard)
 def trial(label,src,goals,extra=()):
  cmd=[str(B/'verus'),str(src),'--crate-type=lib','--edition=2024','--crate-name','review_'+label,'--no-cheating','--output-json','--time','--multiple-errors','20']
  if goals:cmd+=['--infer-goals']
  cmd+=list(extra);receipt['inputs'][str(src)]=digest(src);sg=ArtifactGuard([src]);code=run(label,cmd,env);sg.check()
  out=(D/(label+'.stdout')).read_text();err=(D/(label+'.stderr')).read_text()
  data=json.loads(out);receipt['outcomes'].append({'label':label,'exit_code':code,'data':data,'stdout_sha256':digest(D/(label+'.stdout')),'stderr_sha256':digest(D/(label+'.stderr'))});print(label,code,data['verification-results'],err[:700],flush=True);return data
 text=(R/'source/vstd/set_lib.rs').read_text();body=text[text.index('pub proof fn lemma_set_pair_observations<A>'):text.index('/// If `s1` is a subset of finite set `s2`,',text.index('pub proof fn lemma_set_pair_observations<A>'))]
 helper=D/'helper.rs';helper.write_text('use vstd::prelude::*;\nuse vstd::set_lib::*;\nverus! {\n'+body+'}\n')
 d=trial('helper',helper,False);assert d['verification-results']['success'] and d['verification-results']['verified']==1
 for goals in [False,True]:trial('compatibility_'+str(goals).lower(),E/'compatibility.rs',goals)
 neg=D/'termination.rs';neg.write_text('use vstd::prelude::*;verus!{ proof fn recursive(a:Set<int>,b:Set<int>,n:nat) ensures a==a,b==b, decreases n {if n>0 {recursive(a,b,n);}} }')
 trial('termination',neg,True)
 neg=D/'bound_false.rs';neg.write_text('use vstd::prelude::*;verus!{proof fn bound_false() ensures forall|s:Set<int>| s.len()==0, {}}')
 trial('bound_false',neg,True)
 plain=D/'plain.rs';plain.write_text('use vstd::prelude::*;verus!{fn unchanged(x:u64)->(r:u64) ensures r==x,{x}}')
 for i in range(3):
  for goals in [False,True]:
   d=trial('plain_'+str(i)+'_'+str(goals).lower(),plain,goals);assert d['verification-results']['success'];assert d['verus-infer'].get('set_pair_assistance',[])==[]
 run=LoggedCommands(O,R,receipt['commands'],timeout=1200)
 assert run('focused-replay',['python3','-B',str(R/'source/tools/research/practical_redb_fedimint/common_set/replay_s1.py'),'--bundle',str(B),'--output',str(O/'fresh-replay')],env)==0
