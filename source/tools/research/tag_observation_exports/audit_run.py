"""Durably capture a bounded owned posthoc audit; never rerun verifier epochs."""
import json,os,subprocess,sys,tempfile,time
from pathlib import Path
import replay as e

def main():
    bundle=e.authenticate();d=Path(tempfile.mkdtemp(prefix='tag-observation-audit-',dir='/tmp'));d.chmod(0o700)
    paths=[*e.HERE.glob('*.py'),e.HERE/'attempts.jsonl',e.HERE/'identities.json',e.HERE/'BASELINE.json',e.B/'receipt.json',*[e.B/n for n in e.NAMES]]
    for p in paths:
        if p.parent==e.HERE:(d/('controller-'+p.name)).write_bytes(p.read_bytes())
    frozen={str(p):e.sha(p) for p in paths}
    cmd=[sys.executable,'-B','-O',str(e.HERE/'audit.py')]
    r=dict(argv=cmd,cwd=str(e.ROOT),inputs=frozen,tools=bundle['tools'])
    e.write(d/'freeze.json',r);fh=e.sha(d/'freeze.json');start=time.monotonic()
    with (d/'stdout').open('wb') as out,(d/'stderr').open('wb') as err:
        rc=subprocess.run(cmd,cwd=e.ROOT,env={**os.environ,'PYTHONDONTWRITEBYTECODE':'1'},stdout=out,stderr=err,timeout=180).returncode
    r.update(exit=rc,seconds=time.monotonic()-start,final_guards_passed=all(e.sha(p)==h for p,h in frozen.items()) and e.sha(d/'freeze.json')==fh)
    e.authenticate()
    r['products']={str(p.relative_to(d)):e.sha(p) for p in d.rglob('*') if p.is_file()}
    if (e.HERE/'RESULTS.json').exists():r['results_sha256']=e.sha(e.HERE/'RESULTS.json')
    e.write(d/'receipt.json',r)
    row=dict(path=str(d),receipt_sha256=e.sha(d/'receipt.json'),exit=rc,guards=r['final_guards_passed'])
    with (e.HERE/'audits.jsonl').open('a') as out:out.write(json.dumps(row)+'\n')
    print(json.dumps(row));print((d/'stdout').read_text(),end='')
    if rc:print((d/'stderr').read_text(),end='')
    e.need(rc==0 and r['final_guards_passed'],'posthoc audit failed')
if __name__=='__main__':main()
