"""Supporting native macro-erasure comparison, not a formal acceptance gate."""
import json,os,pathlib,sys,tempfile
HERE=pathlib.Path(__file__).resolve().parent
sys.dont_write_bytecode=True
sys.path.insert(0,str(HERE.parents[1]))
from infer_eval_support import ArtifactGuard,digest,tool_artifacts
from infer_experiment import LoggedCommands,replay_receipt
from fidelity import strict_check as check
from replay_support import authenticate, require, environment
HARNESS=r'''
fn main() {
    let mut checksum = 0u64;
    let mut cases = 0u64;
    let extras = [0u64, 1, 0x1fffffffffffffff, 0x8000000000000000, 0x9fffffffffffffff];
    for n in 1..=6usize {
        for shape in 0..4u64.pow(n as u32) {
            for extra in extras {
                let borrows = (0..n).map(|j| Item(extra | (((shape >> (2*j)) & 3) << 61))).collect();
                let s = Stack { borrows };
                for g in 0..n {
                    let p = s.get(g).unwrap().perm();
                    if p == Permission::Unique || p == Permission::SharedReadWrite {
                        let r = s.find_first_write_incompatible(g);
                        checksum = checksum.wrapping_mul(31).wrapping_add(r as u64);
                        cases += 1;
                    }
                }
            }
        }
    }
    for word in [0u64,1,u64::MAX,0x8000000000000000,0x1fffffffffffffff,0x6000000000000000] {
        for p in [Permission::Unique,Permission::SharedReadWrite,Permission::SharedReadOnly,Permission::Disabled] {
            let mut i = Item(word);i.set_permission(p);
            checksum=checksum.wrapping_mul(31).wrapping_add(i.0).wrapping_add(i.protected() as u64);
            cases += 1;
        }
    }
    println!("cases={} checksum={}",cases,checksum);
}
'''
def main():
    _,_,authenticated=authenticate();check(HERE/'component.rs');freeze=json.loads((HERE/'FREEZE.json').read_text());private=pathlib.Path(json.loads((HERE/'RESUMPTION.json').read_text())['private_output']);bundle=pathlib.Path(next(iter(freeze['tools']))).parent
    rustc=pathlib.Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin/rustc')
    out=pathlib.Path(tempfile.mkdtemp(prefix='native-',dir=private)); orig=(HERE/'literal.rs').read_text().replace('use vstd::prelude::*;\n','').replace('verus! {\n','',1).rstrip()[:-1]
    (out/'literal.rs').write_text(orig+'\n'+HARNESS);(out/'annotated.rs').write_text((HERE/'component.rs').read_text()+'\n'+HARNESS);(out/'native_compare.py').write_bytes(pathlib.Path(__file__).read_bytes())
    rec={'output':str(out),'classification':'finite_native_support_only','commands':[]}
    (out/'input-freeze.json').write_text(json.dumps({'sources':{p.name:digest(p) for p in out.iterdir() if p.suffix in ['.py','.rs']},'bundle':freeze['tools']},indent=2)+'\n')
    with replay_receipt(out,rec):
        guard=ArtifactGuard(tool_artifacts(bundle/'verus',bundle/'z3',extra=authenticated+[rustc, pathlib.Path('/home/dev/.cargo/bin/rustc').resolve(), HERE/'fidelity.py']+list(out.glob('*.rs'))+list(out.glob('*.py'))));rec['artifacts']=guard.initial
        run=LoggedCommands(out,out,rec['commands'],timeout=180,guard=guard,status_key='returncode')
        env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',VERUS_Z3_PATH=str(bundle/'z3'),VERUS_INFER_CACHE=str(out/'cache'))
        require(run('version',[str(rustc),'-vV'],env)==0, 'version failed')
        rc1=run('literal-build',[str(rustc),str(out/'literal.rs'),'--edition=2021','-o',str(out/'literal')],env)
        rc2=run('annotated-build',[str(bundle/'verus'),str(out/'annotated.rs'),'--edition=2021','--no-cheating','--output-json','--compile','-o',str(out/'annotated')],env)
        rec['build_exits']=[rc1,rc2]
        if rc1==rc2==0:
            rec['run_exits']=[run(n,[str(out/n)],env) for n in ['literal','annotated']]
            rec['outputs_equal']=(out/'literal.stdout').read_bytes()==(out/'annotated.stdout').read_bytes()
            rec['observation']=(out/'literal.stdout').read_text()
            rec['binary_hashes']={n:digest(out/n) for n in ['literal','annotated']}
        require(rec['build_exits']==[0,0] and rec.get('run_exits')==[0,0] and rec.get('outputs_equal') is True, 'native comparison failed')
        for log in out.glob('*.std*'): rec.setdefault('logs_sha256',{})[log.name]=digest(log)
        guard.check();rec['artifacts_after']=guard.snapshot()
    print(out);print({k:rec[k] for k in ['build_exits','run_exits','outputs_equal','observation'] if k in rec});print((out/'annotated-build.stderr').read_text()[-2000:])
if __name__=='__main__':main()
