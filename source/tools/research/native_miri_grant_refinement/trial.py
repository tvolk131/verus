"""Fresh frozen ordinary verification epochs; never overwrites an attempt."""
import argparse, json, os, pathlib, sys, tempfile
HERE = pathlib.Path(__file__).resolve().parent
sys.dont_write_bytecode=True
sys.path.insert(0, str(HERE.parents[1]))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import LoggedCommands, replay_receipt

def main():
    p=argparse.ArgumentParser(); p.add_argument('source',type=pathlib.Path); p.add_argument('--off',action='store_true'); a=p.parse_args()
    freeze=json.loads((HERE/'FREEZE.json').read_text()); private=pathlib.Path(freeze['private_output']); bundle=pathlib.Path(next(iter(freeze['tools']))).parent
    for name, entry in freeze['tools'].items():
        if digest(name)!=entry['sha256']: raise ValueError('bundle authentication failed: '+name)
    out=pathlib.Path(tempfile.mkdtemp(prefix=a.source.stem+'-',dir=private)); src=out/'component.rs'; src.write_bytes(a.source.read_bytes())
    controller=out/'trial.py'; controller.write_bytes(pathlib.Path(__file__).read_bytes())
    manifest={'source':str(a.source.resolve()),'sha256':digest(src),'controller_sha256':digest(controller),'assistance':not a.off,'ordinary_full_scope':True}
    (out/'input-freeze.json').write_text(json.dumps(manifest,indent=2)+'\n')
    rec={'input':manifest,'output':str(out),'commands':[]}
    with replay_receipt(out,rec):
        guard=ArtifactGuard(tool_artifacts(bundle/'verus',bundle/'z3',extra=[src,controller,HERE/'FREEZE.json',HERE/'selected.json']))
        rec['artifacts']=guard.initial
        run=LoggedCommands(out,out,rec['commands'],timeout=180,guard=guard,status_key='returncode')
        env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',VERUS_Z3_PATH=str(bundle/'z3'),VERUS_INFER_CACHE=str(out/'cache'))
        argv=[str(bundle/'verus'),str(src),'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','20']
        if not a.off: argv+=['--infer-goals']
        rc=run('verify',argv,env); rec['returncode']=rc
        rec['stdout_sha256']=digest(out/'verify.stdout'); rec['stderr_sha256']=digest(out/'verify.stderr')
        try: rec['verifier_json']=json.loads((out/'verify.stdout').read_text())
        except json.JSONDecodeError: rec['unparsed_stdout']=True
        guard.check(); rec['artifacts_after']=guard.snapshot()
    print(out)
    print((out/'verify.stderr').read_text()[-14000:])
    if 'verifier_json' in rec: print(json.dumps(rec['verifier_json'].get('verification-results')))
if __name__=='__main__': main()
