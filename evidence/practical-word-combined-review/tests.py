"""Fresh full compiler-library and focused compiler units; verify actual paths."""
import os,json,shutil,sys,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];O=Path('/tmp/practical-word-combined-review-5ecea5fe');B=O/'bundle';D=O/'test-checks';D.mkdir(exist_ok=False)
sys.path.insert(0,str(ROOT/'source/tools'));from infer_experiment import LoggedCommands,replay_receipt
from infer_eval_support import ArtifactGuard,digest
assert (O/'ready').exists()
env=dict(os.environ,**json.loads((O/'env.json').read_text()),PYTHONDONTWRITEBYTECODE='1');env['RUSTC_WRAPPER']=str(ROOT/'evidence/practical-word-combined-review/test-build-wrapper.py')
fixture=O/'library-fixtures/target-verus';fixture.mkdir(parents=True,exist_ok=True);(fixture/'release').symlink_to(B)
# common/mod.rs resolves three ancestors above profile, then target-verus/profile.
loc=O/'test-layout/target/release/deps';loc.mkdir(parents=True);rt=O/'test-layout/target-verus';rt.mkdir();(rt/'release').symlink_to(B)
receipt={'commands':[],'test_paths':[]}
g=ArtifactGuard([B/n for n in json.loads((B/'receipt.json').read_text())['tools']])
with replay_receipt(D,receipt):
 run=LoggedCommands(D,ROOT,receipt['commands'],timeout=7200,guard=g)
 cargo=[env['RUSTC'].replace('rustc','cargo'),'test','--release','--offline','--manifest-path','source/Cargo.toml','--config','build.build-dir="'+str(O/'build')+'"','--config','build.target-dir="'+str(O/'target')+'"','--no-run','--message-format=json']
 for label,args in [('library',['-p','rust_verify','--lib']),('focused',['-p','rust_verify_test','--test','infer_word_sequence_goals','--test','infer_word_goals'])]:
  assert run('build-'+label,cargo+args,env)==0
  artifacts=[json.loads(l) for l in (D/('build-'+label+'.stdout')).read_text().splitlines() if l.startswith('{')]
  tests=[a for a in artifacts if a.get('reason')=='compiler-artifact' and a.get('profile',{}).get('test') and a.get('executable')]
  assert len(tests)==(1 if label=='library' else 2),tests
  for a in tests:
   src=Path(a['executable']);assert src.is_relative_to(O) and src.is_file()
   dst=loc/src.name;shutil.copy2(src,dst);assert digest(src)==digest(dst)
   expected=dst.parent.parent.parent.parent/'target-verus/release'
   assert expected.resolve()==B.resolve()
   receipt['test_paths'].append({'target':a['target']['name'],'emitted':str(src),'actual':str(dst),'sha256':digest(dst),'resolved_runtime':str(expected.resolve()),'tools':{n:digest(B/n) for n in json.loads((B/'receipt.json').read_text())['tools']}})
   assert run('list-'+a['target']['name'],[str(dst),'--list'],env)==0
   assert run('run-'+a['target']['name'],[str(dst),'--test-threads=2'],env)==0
print('FULL LIBRARY AND FOCUSED CHECKS COMPLETE')
