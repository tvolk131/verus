"""Read-only exact body, field, controller and receipt audit of this review's delta."""
from pathlib import Path
import ast, hashlib, importlib.util, json, re, subprocess, sys
ROOT=Path(__file__).resolve().parents[2]; HERE=Path(__file__).resolve().parent
C=ROOT/'source/tools/research/redb_crash_protocol'; O=C/'source/original'
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_experiment import require
sys.path.insert(0,str(C/'source'))
from probes import item, span

def h(b): return hashlib.sha256(b).hexdigest()
def sha(p): return h(p.read_bytes())
def git(rev,p): return subprocess.check_output(['git','-C',str(ROOT),'show',rev+':'+str(p.relative_to(ROOT))])
def load(n,p):
 s=importlib.util.spec_from_file_location(n,p); m=importlib.util.module_from_spec(s); s.loader.exec_module(m); return m

def shapes(s): return re.sub(r'\bpub(?:\([^)]*\))?\s*','',s).strip()
sel=(C/'selection_contract/checked_derives.rs').read_text()
original=(C/'selection_contract/original_contract.rs').read_text()
f=load('review_fixtures',C/'eligibility/fixtures.py')
require(original==f.contracted(),'full original-contract projection')
decl='#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]\npub struct TransactionId(pub u64);'
expected=original.replace(decl,'')
expected=expected.replace('use vstd::prelude::*;','use vstd::prelude::*;\n#[verus_verify(infer_derive(PartialEq,PartialOrd,Ord))]\n'+decl,1)
require(sel==expected,'derive delta contains more than opt-in and declaration relocation')
H='src/tree_store/page_store/header.rs'
bodies=[]
for name in ['primary_slot','secondary_slot','published_slot','swap_primary_slot','write_secondary_slot',
             'select_primary_slot','finalize_transaction_slots']:
 raw=item(H,'fn',name); body=raw[raw.index('{'):]
 require(body in sel,'complete original body '+name)
 for variant in ['provider','wrong','unsafe']:
  require(body in Path('/tmp/redb-new-review-03fd44ed-selection',variant+'.rs').read_text(),'control body changed '+name)
 bodies.append(dict(name=name,original_file=H,body_sha256=h(body.encode()),body_bytes=len(body.encode())))
structs=[]
for rel,name in [('src/tree_store/page_store/base.rs','PageNumber'),('src/transaction_tracker.rs','TransactionId'),
 ('src/tree_store/btree_base.rs','BtreeHeader'),(H,'TransactionHeader'),(H,'DatabaseHeader'),(H,'UnrepairedDatabaseHeader')]:
 raw=item(rel,'struct',name)
 match=re.search(r'(?:pub )?struct '+name+r'\b',sel); start=match.start()
 if name=='TransactionId': finish=sel.index(';',start)+1
 else: finish=span(sel,start)
 projected=sel[start:finish]
 require(shapes(raw)==shapes(projected),'field/type drift '+name)
 structs.append(dict(name=name,file=rel,original_sha256=h(raw.encode()),visibility_erased_equal=True))
backend=Path('/tmp/redb-new-review-03fd44ed-storage-positive/backend_nominal.rs').read_text()
trait=item('src/db.rs','trait','StorageBackend')
require(trait in backend,'full original trait/default body')
checked=item('src/tree_store/page_store/cached_file.rs','struct','CheckedBackend')
require(checked in backend,'original checked fields')
errors=(O/'src/error.rs').read_text(); start=errors.index('impl From<io::Error> for StorageError'); conv=errors[start:span(errors,start)]
require(conv in backend,'original From body')
for n in ['check_failure','sync_data']:
 raw=item('src/tree_store/page_store/cached_file.rs','fn',n)
 require(raw in backend,'original backend body '+n)
 bodies.append(dict(name='CheckedBackend::'+n,original_file='src/tree_store/page_store/cached_file.rs',body_sha256=h(raw[raw.index('{'):].encode())))
require(not re.search(r'\b(requires|ensures|assume|admit)\b',trait),'unexpected backend semantic contract')
interfaces=re.findall(r'(?m)^    fn (\w+)\([^;{]+;',trait)
require(set(interfaces)=={'len','read','set_len','sync_data','write'} and len(interfaces)==5,'five abstract methods')
wrong=Path('/tmp/redb-new-review-03fd44ed-storage-negative/backend_nominal.rs').read_text()
require(wrong==backend.replace('Self { StorageError::Io(error) }','Self { StorageError::Unsupported }'),'wrong adapter changes runtime')
unchanged=[]
for p in [C/'CASES.json', C/'model/protocol.rs', C/'model/CONTROLS.json', C/'model/controls.py',
          C/'model/ACCEPTED78_INFER_REGISTRY.json', *sorted((C/'model/controls').glob('*.rs')),
          ROOT/'source/tools/infer_experiment.py',ROOT/'source/tools/infer_eval_support.py']:
 b=p.read_bytes(); require(b==git('1ca1cae5',p),'changed accepted input '+str(p))
 unchanged.append(dict(path=str(p.relative_to(ROOT)),sha256=h(b),unchanged_from='1ca1cae5'))
control=json.loads((C/'model/CONTROLS.json').read_text())
require(sha(C/'model/protocol.rs')==control['model_sha256'],'model freeze')
require(sha(C/'model/controls/error_implies_rollback.rs')==control['cases']['error_implies_rollback']['sha256'],'control freeze')
replay=(C/'model/replay.py').read_text(); require(not any(isinstance(n,ast.Assert) for n in ast.walk(ast.parse(replay))),'assert remains')
require(not subprocess.check_output(['git','diff','--name-only','1ca1cae5','03fd44ed','--','source/rust_verify','source/vstd','source/builtin','source/builtin_macros','source/state_machines_macros']).strip(),'foundational source changed')
cases=json.loads((C/'CASES.json').read_text())
charter=[]
for n,expected_hash in cases['charter_sha256'].items():
 p=C/n; historic=git('c008326b',p)
 require(h(historic)==expected_hash,'historical charter')
 charter.append(dict(path=n,registered_sha256=expected_hash,current_sha256=sha(p),current_differs=sha(p)!=expected_hash))
require((C/'CASES.json').read_bytes()==git('c008326b',C/'CASES.json'),'immutable CASES')
# Authenticate the selected original source snapshots against their frozen source manifest.
srcfreeze=json.loads((C/'source/FROZEN.json').read_text())
srcs=[]
for rel in sorted({b['original_file'] for b in bodies}|{s['file'] for s in structs}|{'src/db.rs','src/error.rs'}):
 p=O/rel; require(sha(p)==srcfreeze['source_sha256'][rel],'original snapshot hash')
 pinned=Path(cases['redb_checkout'])/rel
 require(p.read_bytes()==pinned.read_bytes(),'pinned source mismatch')
 srcs.append(dict(file=rel,sha256=sha(p)))
result=dict(selection_body_fidelity=bodies[:7],selection_fields=structs,
            selection_projection='visibility widening, return bindings/contracts, module flattening, unchanged derives plus one stock infer_derive opt-in/declaration relocation',
            storage_bodies=bodies[7:],storage_from_impl_sha256=h(conv.encode()),storage_trait_sha256=h(trait.encode()),
            storage_abstract_methods=interfaces,storage_persistence_or_value_laws_added=False,
            unchanged_inputs=unchanged,charter=charter,selected_original_sources=srcs,
            replay_assert_count=0,source_coupling_proved=False)
(HERE/'SOURCE_AUDIT.json').write_text(json.dumps(result,indent=2)+'\n')
print('Exact selected original bodies/fields, five abstract interfaces and unchanged model/control/trusted inputs authenticated.')
