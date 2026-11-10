"""Opam build wrapper: bind actual extracted inputs to the frozen source tree."""
import datetime,hashlib,json,os,pathlib,subprocess,sys,time,fcntl
from guard import check,digest
r=pathlib.Path('/tmp/tree-borrows-checked-foundation-20261110')
# Serialize package build commands; each upstream make/dune still has at most -j2.
# Opam may schedule two packages, but only one compiles at a time.
lock=(r/'package-build.lock').open('w');fcntl.flock(lock,fcntl.LOCK_EX)
cwd=pathlib.Path.cwd();pkg=cwd.name
manifest=r/'source-manifests'/(pkg+'.json');files=json.loads(manifest.read_text()) if manifest.exists() else {}
# `opam source` supplies a synthetic root opam only for packages without one.
# It is package metadata (independently pinned), never a semantic source file.
synthetic=[]
if 'opam' in files and not (cwd/'opam').exists():
 synthetic=['opam'];files.pop('opam')
log=r/'build-guards';log.mkdir(exist_ok=True);target=log/(pkg+'-'+str(time.time_ns())+'.json')
meta={'package':pkg,'cwd':str(cwd),'argv':sys.argv[1:],'started':datetime.datetime.now(datetime.timezone.utc).isoformat(),'synthetic_metadata_absent_in_build':synthetic}
try:
 meta['initial']=check(cwd,files);target.write_text(json.dumps(meta,indent=2)+'\n')
 rc=subprocess.run(sys.argv[1:]).returncode;meta['exit_code']=rc
 meta['final']=check(cwd,files)
except Exception as e:
 meta['guard_error']=str(e);rc=97
meta['finished']=datetime.datetime.now(datetime.timezone.utc).isoformat();target.write_text(json.dumps(meta,indent=2)+'\n');sys.exit(rc)
