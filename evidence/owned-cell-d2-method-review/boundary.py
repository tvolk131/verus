"""Challenge new VIR entry APIs against our own fresh library, no compiler rebuild.

Canonical fixture scaffolding is extracted from frozen builder/old descriptor
fixture and adapted to synthetic spans/IDs. Only the independent tail is a test.
This cannot authenticate native correspondence or add a semantic solver control.
"""
import json,os,sys
from pathlib import Path
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require
O=Path(sys.argv[1]);O.mkdir(exist_ok=False)
build=Path(sys.argv[2]);deps=build/'build/debug/deps'
TC=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
lower=ROOT/'source/rust_verify/src/checked_owned_cell_lowering.rs';old=ROOT/'evidence/owned-cell-d1-review/boundary.rs';tail=Path(__file__).with_name('boundary_tail.rs')
binding=json.loads(Path(__file__).with_name('BINDING.json').read_text());require(digest(lower)==binding['source'][str(lower.relative_to(ROOT))],'frozen builder')
source=lower.read_text();builder=source[source.index('fn build('):source.index('fn matches_type(')]
builder='fn build(native:u64, op:Operation, d:&Datatype, spans:&Spans)->Function {\n'+builder.split(') -> Function {\n',1)[1]
builder=builder.replace('span(tcx, native)','span()').replace('crate::rust_to_vir_base::mk_crate_id(tcx, rustc_hir::def_id::LOCAL_CRATE)','local().krate.clone()').replace('tcx.def_path_hash(native).local_hash().as_u64()','native')
helpers=source[source.index('fn u64t()'):source.index('fn datatype(')]
descriptor=old.read_text().split('fn typ(')[0]
fixture=O/'fixture.rs';fixture.write_text(descriptor+'\nuse vir::ast_util::{bool_typ,str_unique_var};\n'+helpers+builder+tail.read_text())
libs=[];artifacts=[]
for n in ['vir','air']:
    matches=list(deps.glob('lib'+n+'-*.rlib'));require(len(matches)==1,'one fresh library '+n)
    artifacts+=matches;libs+=['--extern',n+'='+str(matches[0])]
inputs=[Path(__file__),tail,lower,old,fixture,*artifacts]
guard=ArtifactGuard(inputs);receipt=dict(commands=[],semantic_proof_credit=False,generated_fixture_sha256=digest(fixture),inputs={str(p):digest(p) for p in inputs})
env=dict(os.environ,RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN='1.99.1',LD_LIBRARY_PATH=str(TC/'lib'),TMPDIR=str(O),RUST_MIN_STACK='20971520')
with replay_receipt(O,receipt):
    run=LoggedCommands(O,ROOT/'source',receipt['commands'],timeout=120,guard=guard)
    require(run('build',[str(TC/'bin/rustc'),'--edition=2024','--test',str(fixture),'-L','dependency='+str(deps),*libs,'-o',str(O/'boundary-tests')],env)==0,'boundary compile')
    require(run('run',[str(O/'boundary-tests'),'--nocapture','--test-threads=1'],env)==0,'boundary controls')
    require('D2-R1 reproduced:' in (O/'run.stderr').read_text(),'recorded panic')
    guard.check();receipt.update(guard_passed=True,logs={p.name:digest(p) for p in O.glob('*.std*')})
print(json.dumps(dict(output=str(O),success=receipt['success'],receipt_sha256=digest(O/'receipt.json'))))
