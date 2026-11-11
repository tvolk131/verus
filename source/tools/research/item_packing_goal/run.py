"""Isolated packing experiments. Run python3 -B (also checked under -O).
Only named immutable inputs; every completed trial has a frozen command receipt.
"""
import hashlib, importlib.util, json, os, re, shutil, subprocess, sys, tempfile, time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4]
HERE=Path(__file__).resolve().parent
W=Path('/tmp/checked-bool-build-klx7abqu')
REVIEW=Path('/tmp/local-checked-foreign-review-l3x_pgnw')
PARENT=Path('/tmp/verus-infer-buffer-cycle-20261109/parent-combined70-bundle-35bc7b12')
TC=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
TOOLS=['verus','rust_verify','z3','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','libvstd.rlib','vstd.vir']
def need(b,m):
 if not b: raise RuntimeError(m)
def sha(p): return hashlib.file_digest(Path(p).open('rb'),'sha256').hexdigest()
def write(p,j): Path(p).write_text(json.dumps(j,indent=2,sort_keys=True)+'\n')
def module(name,p):
 spec=importlib.util.spec_from_file_location(name,p);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
S=module('packing_source',HERE.parent/'item_tag_constructor_composition/source.py')
def fixture():
 # Rebind pure generator input to authenticated completed full-help body bytes.
 # Never follow its historical upstream path into another artifact/checkout.
 p=REVIEW/'stable-full-help/component.rs';f=json.loads((REVIEW/'stable-full-help/cold/freeze.json').read_text())
 need(sha(p)==f['fixtures'][str(p)],'completed fixture hash')
 text=p.read_text();helper=S.block(text,'proof fn packed_fields_raw(')
 clean=text.replace(helper+'\n','')
 while 'proof {' in clean:
  b=S.block(clean,'proof {');a=clean.index('proof {');clean=clean[:a-8]+clean[a+len(b)+1:]
 # Preserve exactly the generator's original body indentation/newlines.
 bor=re.search(r'#\[derive\(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord\)\]\npub struct BorTag\(NonZero<u64>\);',text).group()+'\n'
 it=re.search(r'#\[derive\(Copy, Clone, Hash, PartialEq, Eq\)\]\npub struct Item\(u64\);',text).group()+'\n'
 it+='\n'.join(re.findall(r'^const \w+: u64 = .*;',text,re.M))+'\n'+S.block(text,'pub enum Permission')+'\n'
 anchors=[('b','pub fn new(i:'),('b','pub fn get('),('i','pub fn tag('),('i','pub fn new(tag:'),('i','pub fn perm('),('i','pub fn protected('),('i','fn to_bits('),('i','fn from_bits(')]
 for owner,a in anchors:
  body=S.block(clean,a)
  if owner=='b':bor+='    '+body+'\n'
  else:it+='    '+body+'\n'
 it+='\n'.join(re.findall(r'^    const \w+: u64 = .*;',text,re.M))+'\n'
 S.load=lambda:(bor,it)
 full=S.generate('full');helped=S.generate('full-help')
 need(helped==text,'pure full-help regeneration byte mismatch')
 need(not re.search(r'\bproof\b|\bassert\s*\(',full),'local proof glue')
 return full,helped,S.fidelity(full)
def root(): return Path((HERE/'PRIVATE_ROOT').read_text().strip())
def env(r):
 return dict(HOME='/home/dev',PATH='/home/dev/.cargo/bin:/usr/bin:/bin',RUSTUP_TOOLCHAIN='1.99.1',RUSTUP_HOME='/home/dev/.rustup',RUSTC_BOOTSTRAP='1',LD_LIBRARY_PATH=str(TC/'lib'),CARGO_HOME=str(r/'cargo-home'),CARGO_TARGET_DIR=str(r/'target'),CARGO_BUILD_BUILD_DIR=str(r/'cargo-build'),CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true',XDG_CACHE_HOME=str(r/'cache'),TMPDIR=str(r/'tmp'),VERUS_ROOT=str(r/'bundle'),VERUS_Z3_PATH=str(r/'bundle/z3'),VERUS_INFER_CACHE=str(r/'candidate-cache'),RUST_MIN_STACK='20971520',RUSTFLAGS='--cfg proc_macro_span --cfg verus_keep_ghost --cfg span_locations --cfg procmacro2_semver_exempt',VERUS_IN_VARGO='1',VERUS_TEST_LOG_DIR=str(r/'test-logs'))
def manifest():
 paths=subprocess.check_output(['git','ls-files','-z','source','dependencies'],cwd=ROOT).decode().split('\0')
 return {p:sha(ROOT/p) for p in paths if p and '/research/item_packing_goal/' not in p}
def run(label,argv,extra=None,bundle=None):
 r=root();d=r/label;d.mkdir(parents=True,exist_ok=True);need(not (d/'receipt.json').exists(),'immutable '+label)
 b=bundle or r/'bundle';f=dict(argv=list(map(str,argv)),cwd=str(ROOT/'source'),environment=env(r)|dict(VERUS_ROOT=str(b),VERUS_Z3_PATH=str(b/'z3'))|dict(extra or {}),source=manifest(),tools={n:sha(b/n) for n in TOOLS if (b/n).exists()},fixtures={str(p):sha(p) for p in argv if str(p).endswith('.rs') and Path(p).is_file()})
 write(d/'freeze.json',f);start=time.monotonic()
 with (d/'stdout').open('wb') as out,(d/'stderr').open('wb') as err:p=subprocess.run(f['argv'],cwd=f['cwd'],env=f['environment'],stdout=out,stderr=err)
 guards=f['source']==manifest() and all(sha(p)==h for p,h in f['fixtures'].items()) and all(sha(b/n)==h for n,h in f['tools'].items())
 j=dict(exit=p.returncode,guards=guards,seconds=time.monotonic()-start,freeze_sha256=sha(d/'freeze.json'),stdout_sha256=sha(d/'stdout'),stderr_sha256=sha(d/'stderr'))
 try:j['verification']=json.loads((d/'stdout').read_text())['verification-results']
 except (ValueError,KeyError):pass
 write(d/'receipt.json',j);print(label,json.dumps(j),flush=True);need(guards,'trial drift');return j

def trial(label,src,b=None,on=True,extra=()):
 r=root();b=b or r/'bundle';d=r/label;d.mkdir(parents=True,exist_ok=True)
 p=d/'component.rs';p.write_text(src)
 return run(label,[b/'verus',p,'--crate-name=component','--crate-type=lib','--no-cheating','--compile','--output-json','--multiple-errors','20','--log-all','--log-dir',d/'vir','-o',d/'libcomponent.rlib',*(['--infer-goals'] if on else []),*extra],bundle=b)
def setup():
 r=Path(tempfile.mkdtemp(prefix='item-packing-'));(HERE/'PRIVATE_ROOT').write_text(str(r)+'\n')
 need(sha(PARENT/'receipt.json')=='f24584b8ccfde3ac92e84c146d87317b04231b2b165ef759411a0c53ea3199e3','parent receipt')
 j=json.loads((PARENT/'receipt.json').read_text());need(j['success'] and j['matching_vstd_verified']==2250,'parent status')
 for n,h in j['tools'].items():need(sha(PARENT/n)==h,'parent tool '+n)
 need(not subprocess.check_output(['git','diff','35bc7b12','--','source/rust_verify','source/vir','source/vstd','source/builtin','source/builtin_macros','source/state_machines_macros'],cwd=ROOT),'baseline source drift')
 write(r/'parent-authentication.json',j)
 for d in ['tmp','bundle','cargo-home/registry','cargo-home/git']:(r/d).mkdir(parents=True,exist_ok=True)
 for part in ['cache','index']:shutil.copytree(REVIEW/'cargo-home/registry'/part,r/'cargo-home/registry'/part)
 shutil.copytree(REVIEW/'cargo-home/git/db',r/'cargo-home/git/db')
 j=json.loads((REVIEW/'cache-manifest.json').read_text())
 for p,h in j['validated'].items():need(sha(r/p)==h,'archive '+p)
 shutil.copy2(PARENT/'z3',r/'bundle/z3')
 full,helped,fidelity=fixture();(r/'full.rs').write_text(full);(r/'full-help.rs').write_text(helped)
 write(HERE/'FREEZE.json',dict(full_sha256=sha(r/'full.rs'),full_help_sha256=sha(r/'full-help.rs'),fidelity=fidelity,manual=dict(proof_functions=1,proof_blocks=3,lemma_calls=1,quantified_bridges=1,bitvector_assertions=7),parent_receipt_sha256=sha(PARENT/'receipt.json')))
 print(r)
def build(label):
 r=root();j=run(label,[TC/'bin/cargo','build','--offline','--locked','-j','2',*sum((['-p',p] for p in ['rust_verify','verus','verus_builtin','verus_builtin_macros','verus_state_machines_macros','vstd_build']),[])])
 need(j['exit']==0,'build')
 for n in TOOLS:
  if n not in ['z3','vstd.vir','libvstd.rlib']:shutil.copy2(r/'target/debug'/n,r/'bundle'/n)
 (r/'bundle/verus-root').touch()
 if not (r/'bundle/vstd').exists():shutil.copytree(ROOT/'source/vstd',r/'bundle/vstd')
if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'run with -B')
 if sys.argv[1]=='setup':setup()
 elif sys.argv[1]=='baseline':
  for name in ['full','full-help']:
   for on in [True,False]:trial('baseline2-'+name+('-on' if on else '-off'),(root()/(name+'.rs')).read_text(),PARENT,on)
 elif sys.argv[1]=='build':build(sys.argv[2])
 elif sys.argv[1]=='vstd':run(sys.argv[2],[root()/'target/debug/vstd_build',root()/'bundle'])
 elif sys.argv[1]=='full':trial(sys.argv[2],(root()/'full.rs').read_text())
