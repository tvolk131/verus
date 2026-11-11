"""Source/Git + exact named sealed MIR checks; no historical audit execution."""
from build import *

def audit():
 inp=json.loads((ROOT/'evidence/nonzero-ord-factoring-design/INPUTS.json').read_text())
 rebindings={}
 for n,h in inp['own_files'].items():
  old=subprocess.check_output(['git','show',inp['base']+':'+n],cwd=ROOT)
  need(hashlib.sha256(old).hexdigest()==h,'design Git source '+n)
  rebindings[n]=dict(design=h,current=sha(ROOT/n),unchanged=sha(ROOT/n)==h)
 for n in inp['inherited_git_rebindings']:
  need((ROOT/n).read_bytes()==subprocess.check_output(['git','show','5d79f711:'+n],cwd=ROOT),'inherited unchanged '+n)
 seal=json.loads((ROOT/'source/tools/research/nonzero_ordering_goal/SEAL.json').read_text())
 allowed={'/tmp/nonzero-ordering-baseline-qoptrgkf','/tmp/nonzero-ordering-mir-x1sbsnfo'}
 need(set(seal['roots'])==allowed,'sealed roots changed')
 count=0
 for r,files in seal['roots'].items():
  for n,h in files.items():
   p=Path(n);need(not p.is_absolute() and '..' not in p.parts,'invalid sealed path')
   need(sha(Path(r)/p)==h,'sealed file '+n);count+=1
 need(count==168,'sealed count')
 mir=json.loads((ROOT/'source/tools/research/nonzero_ordering_goal/MIR.json').read_text())
 raw=Path('/tmp/nonzero-ordering-mir-x1sbsnfo/inspect/stdout').read_text()
 sections={s.splitlines()[0]:s for s in raw.split('BEGIN_INSTANCE ')[1:]}
 for row in mir['inventory']:
  need(hashlib.sha256(sections[row['instance']].encode()).hexdigest()==row['whole_mir_sha256'],'MIR whole section')
 # Exact selected pinned compiler/core files only; never follow checkout labels.
 frozen=json.loads(Path('/tmp/nonzero-ordering-mir-x1sbsnfo/inspect/freeze.json').read_text())['inputs']
 core={n:h for n,h in frozen.items() if 'libcore-' in n or n.endswith('/bin/rustc') or 'librustc_driver' in n}
 need(len(core)==3,'core input count')
 for n,h in core.items():need(sha(n)==h,'compiler/core identity')
 changed=subprocess.check_output(['git','diff','--name-only','a3b1e280','7fde950c','--','source','dependencies','tools','rust-toolchain.toml'],cwd=ROOT).decode().splitlines()
 need({n for n in changed if not n.startswith(('source/docs/', 'source/tools/research/'))} == {'source/rust_verify/src/import_export.rs','source/vir/src/infer/registry.rs'},'prepared production integration scope')
 need('const INFER_SCHEMA: u32 = 39;' in (ROOT/'source/rust_verify/src/import_export.rs').read_text(),'schema')
 need('candidate-bound-evidence-v73' in (ROOT/'source/rust_verify/src/import_export.rs').read_text(),'policy')
 need('verus-infer-v76:' in (ROOT/'source/vir/src/infer/registry.rs').read_text(),'cache epoch')
 return dict(source_base='7fde950cf87f67c2fe755c4f5f1e96f358436a37',design_rebindings=rebindings,prepared_changes=changed,sealed_files=count,whole_mir_instances=len(sections),compiler_core=core,versions=dict(schema=39,policy=73,cache=76),ordering_export_kinds=0,new_std_contracts=0,publication=False,goal_complete=False)
if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'use -B')
 j=audit()
 if '--write' in sys.argv:write(HERE/'SOURCE_AUTHENTICATION.json',j)
 else:need(j==json.loads((HERE/'SOURCE_AUTHENTICATION.json').read_text()),'record differs');print('source audit passed')
