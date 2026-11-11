"""Two exact one-line pass ablations; restore original source after each build."""
import os,json,sys,shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];O=Path('/tmp/practical-word-combined-review-5ecea5fe');D=O/'ablations';D.mkdir(exist_ok=False)
sys.path[:0]=[str(ROOT/'source/tools'),str(ROOT/'source/tools/research/practical_redb_fedimint/common_set')]
from infer_experiment import LoggedCommands,replay_receipt,validate_run,require_exact,require
from infer_eval_support import digest,ArtifactGuard
from replay_s1 import require_native_scope,body_inventory
assert (O/'ready').exists()
auth=json.loads((Path(__file__).parent/'AUTHENTICATION.json').read_text())['own_source']
p=ROOT/'source/vir/src/infer/pipeline.rs';original=p.read_bytes()
env=dict(os.environ,**json.loads((O/'env.json').read_text()),PYTHONDONTWRITEBYTECODE='1')
guard=ArtifactGuard([ROOT/n for n in auth if ROOT/n != p])
receipt={'commands':[],'outcomes':[],'mutations':[]}
with replay_receipt(D,receipt):
 run=LoggedCommands(D,ROOT,receipt['commands'],timeout=7200,guard=guard)
 for kind,line in [('set','        let functions = set_goals::prepare(&functions, self.imported, report)?;'),('word','        functions = word_sequence_goals::prepare(&functions, self.imported, report)?;')]:
  assert digest(p)==auth[str(p.relative_to(ROOT))]
  old=line.encode();assert original.count(old)==1
  disabled=original.replace(old,('        // Independent review ablation: '+kind+' prepare disabled.').encode())
  record={'profile':kind,'before':digest(p),'exact_removed_line':line,'restored':False}
  receipt['mutations'].append(record)
  try:
   p.write_bytes(disabled);record['disabled']=digest(p)
   cmd=[env['RUSTC'].replace('rustc','cargo'),'build','--release','--offline','--manifest-path','source/Cargo.toml','--config','build.build-dir="'+str(O/'build')+'"','--config','build.target-dir="'+str(O/'target')+'"','-p','rust_verify']
   assert run('build-'+kind,cmd,env)==0
   assert p.read_bytes()==disabled
   bundle=D/(kind+'-disabled');shutil.copytree(O/'bundle',bundle)
   shutil.copy2(O/'target/release/rust_verify',bundle/'rust_verify')
   tools={n:digest(bundle/n) for n in json.loads((O/'bundle/receipt.json').read_text())['tools']}
   (bundle/'receipt.json').write_text(json.dumps({'tools':tools,'source_delta':record},indent=2)+'\n')
   record['tools']=tools
  finally:
   p.write_bytes(original);assert digest(p)==record['before'];record['after']=digest(p);record['restored']=True
  for case in ['r2','f2']:
   source=ROOT/'source/tools/research/practical_redb_fedimint/common_set'/('r2-combined.rs' if case=='r2' else 'f2-reduced.rs')
   failed=(['finish_mutation'] if case=='r2' else ['quorum_intersection','quorum_has_honest','one_honest_margin']) if kind=='set' else (['U64GroupedBitmap::set','U64GroupedBitmap::clear'] if case=='r2' else [])
   label=kind+'_'+case
   eg=dict(env,VERUS_Z3_PATH=str(bundle/'z3'),VERUS_INFER_CACHE=str(D/(label+'-cache')))
   sg=ArtifactGuard([source,*[bundle/n for n in tools]])
   cmd=[str(bundle/'verus'),str(source),'--crate-type=lib','--crate-name',label,'--edition=2024','--no-cheating','--infer-goals','--output-json','--time','--multiple-errors','20']
   code=run(label,cmd,eg);sg.check()
   stdout=(D/(label+'.stdout')).read_text();stderr=(D/(label+'.stderr')).read_text()
   data=validate_run(dict(returncode=code,stdout=stdout,stderr=stderr),not failed);inf=data['verus-infer'];require_exact(inf.get('functions',[]));require(not inf.get('functions',[]),'no inferred helpers')
   pairs={} if kind=='set' else ({'finish_mutation':20} if case=='r2' else dict(zip(['quorum_intersection','quorum_has_honest','one_honest_margin'],[20,6,6])))
   ex,proof=body_inventory(case+'_cold',True);require_native_scope(data,source,ex,proof,failed,pairs)
   native={b['function']:b['rust_function'] for b in inf['verification_scope']['native_functions']}
   frames=inf.get('word_sequence_assistance',[])
   require(sorted(native[f['function']] for f in frames)==(['U64GroupedBitmap::clear','U64GroupedBitmap::set'] if kind=='set' and case=='r2' else []),'exact other profile retained')
   require(all(f['snapshots']==f['quantified_blocks']==f['library_calls']==1 for f in frames),'word template')
   require(data['verification-results']['is-verifying-entire-crate'],'full crate')
   if failed:
    require('postcondition not satisfied' in stderr,'postcondition failure')
    require(not any(s in stderr for s in ['error[E','precondition not satisfied','possible arithmetic underflow/overflow','assertion failed','invariant not satisfied']),'healthy dependency failure kind')
   receipt['outcomes'].append({'label':label,'verification':data['verification-results'],'failed':failed,'pairs':inf.get('set_pair_assistance',[]),'frames':frames,'scope':inf['verification_scope'],'times':data['times-ms'],'stdout_sha256':digest(D/(label+'.stdout')),'stderr_sha256':digest(D/(label+'.stderr'))})
 for name,h in auth.items():assert digest(ROOT/name)==h,name
print('BOTH NARROW CAUSAL ABLATIONS COMPLETE; SOURCE EXACTLY RESTORED')
