"""Guarded exact-original artifact replay or independent fresh VIR --lib build."""
import importlib.util,json,os,sys,shutil,time
from pathlib import Path
sys.dont_write_bytecode=True
R=Path(__file__).resolve().parents[2];sys.path.insert(0,str(R/'source/tools'))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require
B=json.loads(Path(__file__).with_name('BINDING.json').read_text())
S=Path('/tmp/owned-cell-client-C1-20261112-freeze');TC=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
action=sys.argv[1];O=Path(sys.argv[2]).resolve()
require(digest(S/'receipt.json')==B['snapshot_receipt_sha256'],'freeze')
for n,h in B['snapshot']['tools_sha256'].items():require(digest(S/n)==h,'tool '+n)
require(digest(S/'native-client-test')==B['snapshot']['native_client_test_sha256'],'original test')
for p,h in B['source'].items():require(digest(R/p)==h,'source '+p)
inputs=[Path(__file__),Path(__file__).with_name('BINDING.json'),R/'source/tools/infer_eval_support.py',R/'source/tools/infer_experiment.py',S/'receipt.json',S/'native-client-test',*[S/n for n in B['snapshot']['tools_sha256']],*[R/p for p in B['source']]]
if action=='replay':
 O.mkdir(exist_ok=False)
 for n in ['tmp','cache']:(O/n).mkdir()
 r=dict(commands=[],builds=0,provenance=B['build_provenance'],full_parent_case=True)
 with replay_receipt(O,r):
  g=ArtifactGuard(inputs)
  env=dict(os.environ,PATH='/home/dev/.cargo/bin:/usr/bin:/bin',RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN='1.99.1',RUSTC_BOOTSTRAP='1',VERUS_IN_VARGO='1',VERUS_ROOT=str(S),VERUS_Z3_PATH=str(S/'z3'),RUST_MIN_STACK='20971520',LD_LIBRARY_PATH=str(TC/'lib')+':'+str(TC/'lib/rustlib/x86_64-unknown-linux-gnu/lib'),TMPDIR=str(O/'tmp'),XDG_CACHE_HOME=str(O/'cache'),VERUS_INFER_CACHE=str(O/'infer-cache'))
  env.pop('C1_PRIVATE_SOURCE_CASE',None)
  r['environment']={k:env[k] for k in ['PATH','RUSTUP_HOME','RUSTUP_TOOLCHAIN','RUSTC_BOOTSTRAP','VERUS_IN_VARGO','VERUS_ROOT','VERUS_Z3_PATH','RUST_MIN_STACK','LD_LIBRARY_PATH','TMPDIR','XDG_CACHE_HOME','VERUS_INFER_CACHE']}
  run=LoggedCommands(O,R/'source',r['commands'],timeout=240,guard=g)
  for label,name in [('native','checked_owned_cell::tests::loaded_native_methods_and_complete_mir_corruptions'),('clients','checked_owned_cell::clients::original_frontend_inventory_and_first_unproved_native_boundary')]:
   require(run(label,[str(S/'native-client-test'),'--exact',name,'--nocapture','--test-threads=1'],env)==0,label+' exit')
   out=(O/(label+'.stdout')).read_text();err=(O/(label+'.stderr')).read_text()
   require('1 passed; 0 failed' in out,label+' exact unit inventory');require('panicked at' not in err,'panic')
   sessions=1 if label=='native' else 2
   for role in ['Positive','FalseFirst','FalseFuture']:
    require(err.count('D2 complete native body query inventory: 3 bodies, 3 queries, query='+role)==sessions,label+' complete '+role)
    require(err.count('D2 ordinary full native body result (query='+role+')')==3*sessions,label+' per-body '+role)
   if label=='clients':
    require(err.count('C1 actual inventory: admitted=3, owners=6, canonical_headers=3, translated=3, native_helpers_installed=0, calls=7, stores=2')==1,'goals inventory')
    require(err.count('C1 actual inventory: admitted=3, owners=3, canonical_headers=0, translated=3, native_helpers_installed=0, calls=7, stores=2')==1,'raw inventory')
    require(err.count('C1 actual checked reconciliation: checked=0/3,')==2,'precise measured zero clients')
    require(err.count('C1 actual checked reconciliation: checked=1/1,')==1,'scalar WF only')
    r['wf_notes']=[line for line in err.splitlines() if 'C1 actual checked reconciliation:' in line]
   print(label+' PASS',flush=True)
  g.check();r.update(guard_passed=True,standalone_native_queries=9,repeated_native_queries_in_source_sessions=18,original_client_solver_queries=0,F2=0,F3=0,complete_checked_inventory=False,logs={p.name:digest(p) for p in O.glob('*.std*')})
elif action=='vir':
 spec=importlib.util.spec_from_file_location('c1_setup',R/'source/tools/research/redb_crash_protocol/native_state_link/implementation/check.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 require(shutil.disk_usage(O.parent).free>=25*1024**3,'25GiB pre-setup floor')
 m.setup(O)
 r=dict(commands=[],provenance='Fresh complete VIR --lib from prepared local source; private authenticated offline dependencies, no copied build tree, no rust_verify or vstd rebuild.')
 with replay_receipt(O,r):
  src=[p for base in ['source/vir','source/air','source/vir_macros'] for p in (R/base).rglob('*') if p.is_file()]+[R/'source/Cargo.toml',R/'source/Cargo.lock',R/'source/tools/research/redb_crash_protocol/native_state_link/implementation/check.py']
  g=ArtifactGuard(inputs+src)
  env=m.env(O);env.update(CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0',CARGO_BUILD_JOBS='2')
  require(shutil.disk_usage(O).free>=25*1024**3,'25GiB build floor')
  r['started_utc']=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime())
  run=LoggedCommands(O,R/'source',r['commands'],timeout=1200,guard=g)
  require(run('complete-vir-lib',[str(TC/'bin/cargo'),'test','--offline','--locked','-p','vir','--lib','--','--test-threads=1'],env)==0,'VIR --lib')
  out=(O/'complete-vir-lib.stdout').read_text();require('172 passed; 0 failed' in out,'172 complete VIR units')
  g.check();r.update(guard_passed=True,source={str(p.relative_to(R)):digest(p) for p in src},logs={p.name:digest(p) for p in O.glob('*.std*')},free_bytes=shutil.disk_usage(O).free,bootstrap_sha256=digest(O/'bootstrap.json'),finished_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()))
elif action=='vir-library':
 spec=importlib.util.spec_from_file_location('c1_setup',R/'source/tools/research/redb_crash_protocol/native_state_link/implementation/check.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 target=O/'role-oracle-library';target.mkdir(exist_ok=False)
 r=dict(commands=[],provenance='Linkable VIR library only, in this review existing private build; needed to inspect exact Invalid assertion messages in a synthetic oracle.')
 with replay_receipt(target,r):
  src=[p for base in ['source/vir','source/air','source/vir_macros'] for p in (R/base).rglob('*') if p.is_file()]+[R/'source/Cargo.toml',R/'source/Cargo.lock']
  g=ArtifactGuard(inputs+src);env=m.env(O);env.update(CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0',CARGO_BUILD_JOBS='2')
  require(shutil.disk_usage(O).free>=25*1024**3,'25GiB floor')
  run=LoggedCommands(target,R/'source',r['commands'],timeout=300,guard=g)
  require(run('vir-library',[str(TC/'bin/cargo'),'build','--offline','--locked','-p','vir','--lib'],env)==0,'linkable VIR library')
  g.check();r.update(guard_passed=True,logs={p.name:digest(p) for p in target.glob('*.std*')})
 O=target
else:raise ValueError(action)
print(json.dumps(dict(output=str(O),success=r['success'],receipt_sha256=digest(O/'receipt.json'))))
