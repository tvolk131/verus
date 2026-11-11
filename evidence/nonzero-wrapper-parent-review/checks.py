"""Focused matching-runtime Rust checks; all raw outputs private."""
from resume import *
SUITES=['certified_foreign_bool','local_checked_foreign','local_checked_packing','infer_goals','infer_nonzero_derive','constructor_function_values','infer_compare_derive','infer_nonzero_ordering']
def compile_tests():
    j=run('final-test-build-v4',[TC/'bin/cargo','test','--offline','--locked','-j','2','-p','rust_verify_test','--no-run','--message-format=json',*sum((['--test',s] for s in SUITES),[]) ]);need(j['exit']==0,'focused test build')
    j=run('final-unit-build-v2',[TC/'bin/cargo','test','--offline','--locked','-j','2','-p','vir','-p','rust_verify','--lib','--no-run','--message-format=json']);need(j['exit']==0,'unit test build')
def execute():
    artifacts={}
    for label in ['final-test-build-v4','final-unit-build-v2']:
        for line in (R/label/'stdout').read_text().splitlines():
            try:j=json.loads(line)
            except ValueError:continue
            if j.get('reason')=='compiler-artifact' and j.get('executable') and j['profile']['test']:
                n=j['target']['name']+('-unit' if j['target']['name']=='rust_verify' and 'lib' in j['target']['kind'] else '');artifacts[n]=j['executable']
    for p in [R/'target-verus/debug',ROOT/'source/target-verus/release']:
        p.parent.mkdir(exist_ok=True,parents=True);need(not p.exists() and not p.is_symlink(),'fresh harness alias');p.symlink_to(B,target_is_directory=True)
    z=ROOT/'source/z3';need(not z.exists() and not z.is_symlink(),'own solver harness path fresh');z.symlink_to(B/'z3')
    for profile in [R/'target/debug',R/'build/debug']:
        profile.mkdir(exist_ok=True,parents=True)
        for n in b.TOOLS:
            dest=profile/n
            if dest.exists() and sha(dest)==sha(B/n):continue
            if dest.exists():dest.unlink()
            shutil.copy2(B/n,dest)
    for name in ['vir','rust_verify-unit',*SUITES]:
        j=run('final-test-'+name,[artifacts[name],'--test-threads=1'],dependencies=[Path(artifacts[name])]);need(j['exit']==0,'test failure '+name+': '+(R/('final-test-'+name)/'stdout').read_text()[-5000:])
if __name__=='__main__':
    need(EPOCH=='proofs','matching final epoch')
    {'build':compile_tests,'run':execute}[sys.argv[1]]()
