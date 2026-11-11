"""Authenticate only named immutable historical inputs and own source freezes."""
import hashlib,json,subprocess,sys,shutil,importlib.util,difflib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]; E=Path(__file__).resolve().parent
OUT=Path('/tmp/practical-word-combined-review-auth');OUT.mkdir(exist_ok=False)
H=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
C=ROOT/'source/tools/research/practical_redb_fedimint'; S=C/'common_set'
sys.path.insert(0,str(ROOT/'source/tools'))
sel=json.loads((C/'SELECTION_R2_F2.json').read_text()); r={'parent':'5ecea5fe81c2bc2ef0c9050cdb29cea0c7ef21c4','sources':{},'historical':{}}
for project,d in sel['sources'].items():
 for rel,v in d['files'].items():
  p=Path(d['root'])/rel;assert H(p)==v['sha256'] and p.stat().st_size==v['bytes']
  dst=OUT/project/rel;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,dst)
  r['sources'][project+'/'+rel]=dict(v,private_copy=str(dst))
for name,h in json.loads((S/'COMBINED_FIXTURE_FREEZE.json').read_text()).items():assert H(S/name)==h
s2=(C/'word_frame/s2-reduced.rs').read_text();combined=(S/'r2-combined.rs').read_text()
removals=['    broadcast use vstd::set_lib::range_set_properties;\n','    assert(allocated(b) =~= (if value { allocated(a).insert(i as int) } else { allocated(a).remove(i as int) }));\n']
for line in removals:assert s2.count(line)==1;s2=s2.replace(line,'')
assert s2==combined
meta=json.loads((C/'redb_tree/private.json').read_text());meta['private_artifacts']=str(OUT/'r2')
(OUT/'r2/source').mkdir(parents=True);shutil.copy2(OUT/'redb/src/tree_store/page_store/bitmap.rs',OUT/'r2/source/bitmap.rs')
spec=importlib.util.spec_from_file_location('r2',C/'redb_tree/replay.py');r2=importlib.util.module_from_spec(spec);spec.loader.exec_module(r2)
r['r2_fidelity']=r2.fidelity(meta,S/'r2-combined.rs')
f=(C/'fedimint_quorum/component.rs').read_text()
for k,v in json.loads((C/'fedimint_quorum/manual.json').read_text()).items():
 f=f.replace('// MANUAL_'+k+': unsolved baseline slot, never claimed as a proof.','')
f+='\n'+(C/'fedimint_quorum/adapter.rs').read_text()
# Frozen S1 blank-slot whitespace is retained: compare exact tokens.
assert r2.tokens(f)==r2.tokens((S/'f2-reduced.rs').read_text())
r['r2_annotation_diff']=''.join(difflib.unified_diff((C/'redb_tree/manual.rs').read_text().splitlines(True),combined.splitlines(True)))
p=Path('/tmp/practical-s2-final-validation3/receipt.json');assert H(p)=='7b697e8ef25acf0ec075c268ec29e0e6f09bfbc8c68f2bccb84cca3f2d5fed57'
d=json.loads(p.read_text());assert len(d['outcomes'])==13
r['historical']['s2_receipt']=H(p);r['historical']['s2_runtime']={}
for name,h in d['runtime_sha256'].items():
 p=Path('/tmp/practical-s2-at99-build/runtime-tail')/name;assert H(p)==h;r['historical']['s2_runtime'][name]=h
p=Path('/tmp/practical-s2-at99-build/test-paths.json');assert H(p)=='7c3fe9b9669c3479266b5d816495c967d3c55f9ee5c53ec5a600cf04b4e88631'
r['historical']['s2_test_paths']=H(p)
# Track every own source/research input. Later source restoration must match.
files=subprocess.check_output(['git','ls-files','source','tools','dependencies'],cwd=ROOT,text=True).splitlines()
r['own_source']={p:H(ROOT/p) for p in files if (ROOT/p).is_file()}
(E/'AUTHENTICATION.json').write_text(json.dumps(r,indent=2)+'\n')
print('Exact combined source/body/domain freezes and historical S2 authenticated.')
