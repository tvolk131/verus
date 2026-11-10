"""Private durable foreground command supervisor; Python -B; no proof assertions."""
import argparse, datetime, json, os, pathlib, subprocess, sys, hashlib
p=argparse.ArgumentParser();p.add_argument('name');p.add_argument('cwd');p.add_argument('command',nargs=argparse.REMAINDER);a=p.parse_args()
r=pathlib.Path('/tmp/tree-borrows-checked-foundation-20261110'); logs=r/'jobs';logs.mkdir(exist_ok=True)
meta={'name':a.name,'cwd':a.cwd,'argv':a.command,'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'supervisor_pid':os.getpid()}
path=logs/(a.name+'.json')
with (logs/(a.name+'.log')).open('wb') as f:
 c=subprocess.Popen(a.command,cwd=a.cwd,stdout=f,stderr=subprocess.STDOUT,start_new_session=True)
 meta.update(pid=c.pid,pgid=os.getpgid(c.pid),startticks=pathlib.Path('/proc/'+str(c.pid)+'/stat').read_text().split()[21],log=str(f.name));path.write_text(json.dumps(meta,indent=2)+'\n')
 rc=c.wait()
meta.update(exit_code=rc,finished_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),log_sha256=hashlib.sha256(pathlib.Path(meta['log']).read_bytes()).hexdigest());path.write_text(json.dumps(meta,indent=2)+'\n');sys.exit(rc)
