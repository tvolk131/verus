"""Fresh ordinary full-body epochs with explicit semantic outcome gates."""
import argparse, json, pathlib, tempfile
from replay_support import *
def main():
    p=argparse.ArgumentParser();p.add_argument('source',type=pathlib.Path);p.add_argument('--off',action='store_true');p.add_argument('--kind',choices=['positive','frontend','no_manual','negative'],default='positive');p.add_argument('--extra');a=p.parse_args()
    freeze,bundle,files=authenticate()
    out=pathlib.Path(tempfile.mkdtemp(prefix=a.source.stem+'-',dir=private_root()));src=out/'component.rs';src.write_bytes(a.source.read_bytes());controller=out/'trial.py';controller.write_bytes(pathlib.Path(__file__).read_bytes())
    manifest={'source':str(a.source.resolve()),'sha256':digest(src),'controller_sha256':digest(controller),'assistance':not a.off,'ordinary_full_scope':True,'kind':a.kind,'extra':a.extra}
    (out/'input-freeze.json').write_text(json.dumps(manifest,indent=2)+'\n');rec={'input':manifest,'output':str(out),'commands':[]}
    with replay_receipt(out,rec):
        guard=ArtifactGuard(files+[src,controller]);rec['artifacts']=guard.initial
        run=LoggedCommands(out,out,rec['commands'],timeout=180,guard=guard,status_key='returncode')
        argv=[str(bundle/'verus'),str(src),'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','20']
        if not a.off:argv+=['--infer-goals']
        rc=run('verify',argv,environment(bundle,out));rec['returncode']=rc
        rec['stdout_sha256']=digest(out/'verify.stdout');rec['stderr_sha256']=digest(out/'verify.stderr')
        data=json.loads((out/'verify.stdout').read_text());stderr=(out/'verify.stderr').read_text()
        line=validate(data,rc,stderr,a.kind=='positive',extra=a.extra,frontend=a.kind=='frontend',diagnostic=a.kind=='no_manual')
        if a.kind=='negative':
            start=src.read_text()[:src.read_text().index('fn '+a.extra)].count('\n')+1
            require(line>=start,'negative diagnostic escaped control')
        rec['verification_results']=data['verification-results'];inf=data['verus-infer'];rec['scope']=inf['verification_scope']
        rec['assistance_summary']={k:inf[k] for k in ['status','requested_functions','generated_functions','goal_assistance','interface_written']}
        rec['assistance_summary']['explanations_enabled']=inf['assistance_explanations']['enabled']
        rec['assistance_summary']['decision_statuses']=sorted(set(d['status'] for d in inf['assistance_explanations']['decisions']))
        guard.check();rec['artifacts_after']=guard.snapshot()
    print(out);print(json.dumps(rec['verification_results']))
if __name__=='__main__':main()
