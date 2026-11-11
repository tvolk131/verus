from trials import *
TARGETS={'certified_foreign_bool':3,'local_checked_foreign':5,'local_checked_packing':4,'infer_goals':12,'infer_nonzero_derive':3,'constructor_function_values':13,'fndef_types':68}
if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'use -B')
 r=R
 if sys.argv[1]=='build':
  args=[TC/'bin/cargo','test','--offline','--locked','-j','2','--no-run','--message-format=json','-p','rust_verify_test']
  for t in TARGETS:args+=['--test',t]
  need(run('test-build',args)['exit']==0,'test build')
  need(run('unit-build',[TC/'bin/cargo','test','--offline','--locked','-j','2','--no-run','--message-format=json','-p','vir','-p','rust_verify'])['exit']==0,'unit build')
 elif sys.argv[1]=='run':
  p=R/'target-verus/debug';p.parent.mkdir(exist_ok=True);
  if not p.exists():p.symlink_to(R/'bundle',target_is_directory=True)
  # Normal test harnesses choose both cargo output profiles; compiler bytes stay identical.
  for profile in [R/'target/debug',R/'build/debug']:
   for n in TOOLS:
    dst=profile/n
    if dst.exists() and sha(dst)==sha(R/'bundle'/n):continue
    if dst.exists():dst.unlink()
    shutil.copy2(R/'bundle'/n,dst)
  if not (R/'src/source/z3').exists():(R/'src/source/z3').symlink_to(R/'bundle/z3')
  artifacts={}
  for label in ['test-build','unit-build']:
   for line in (R/label/'stdout').read_text().splitlines():
    try:j=json.loads(line)
    except ValueError:continue
    if j.get('reason')=='compiler-artifact' and j.get('executable') and j['profile']['test']:artifacts[j['target']['name']+('-unit' if j['target']['name']=='rust_verify' and 'lib' in j['target']['kind'] else '')]=j['executable']
  write(R/'test-artifacts.json',{n:dict(path=p,sha256=sha(p)) for n,p in artifacts.items()})
  for t,expected in {'vir':149,'rust_verify-unit':2,**TARGETS}.items():
   args=[artifacts[t]]+(['import_export::infer_arch_tests'] if t=='rust_verify-unit' else [])+['--test-threads=1']
   rec=json.loads((R/('test-'+t)/'receipt.json').read_text()) if (R/('test-'+t)/'receipt.json').exists() else run('test-'+t,args);need(rec['exit']==0,'failed tests '+t)
   out=(R/('test-'+t)/'stdout').read_text();m=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',out);need(m and int(m[1])==expected and int(m[2])==0,'test counts '+t+out[-400:])
   need(int(m[3])==(1 if t=='fndef_types' else 0),'only existing ignored exec_calling_no_vstd')
