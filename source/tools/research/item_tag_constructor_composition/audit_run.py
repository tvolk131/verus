"""Retain validation argv, identities, logs and failures separately from proof epochs."""
import json,subprocess,sys,tempfile,time
from pathlib import Path
import epoch as e

def main():
    d=Path(tempfile.mkdtemp(prefix='item-tag-audit-',dir='/tmp'));d.chmod(0o700)
    frozen={str(p):e.sha(p) for p in e.HERE.iterdir() if p.is_file() and p.name not in ['RESULTS.json','AUDITS.json']}
    argv=[sys.executable,'-B','-O',str(e.HERE/'audit.py')]
    record={'argv':argv,'cwd':str(e.ROOT),'inputs':frozen};e.write(d/'freeze.json',record)
    for p in e.HERE.iterdir():
        if str(p) in frozen:(d/p.name).write_bytes(p.read_bytes())
    start=time.monotonic()
    with (d/'stdout').open('wb') as out,(d/'stderr').open('wb') as err:r=subprocess.run(argv,cwd=e.ROOT,stdout=out,stderr=err)
    record.update(exit=r.returncode,seconds=time.monotonic()-start,final_guards_passed=all(e.sha(p)==h for p,h in frozen.items()),stdout_sha256=e.sha(d/'stdout'),stderr_sha256=e.sha(d/'stderr'))
    if r.returncode==0:record['RESULTS_sha256']=e.sha(e.HERE/'RESULTS.json');(d/'RESULTS.json').write_bytes((e.HERE/'RESULTS.json').read_bytes())
    e.write(d/'receipt.json',record)
    dest=e.HERE/'AUDITS.json';old=json.loads(dest.read_text()) if dest.exists() else []
    old.append({'path':str(d),'receipt_sha256':e.sha(d/'receipt.json'),'exit':r.returncode,'guards':record['final_guards_passed']});e.write(dest,old)
    print(json.dumps(old[-1]));print((d/'stdout').read_text());print((d/'stderr').read_text())
    e.need(record['final_guards_passed'],'audit guard');e.need(r.returncode==0,'audit rejection')
if __name__=='__main__':main()
