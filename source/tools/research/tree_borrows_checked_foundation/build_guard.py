"""Opam build wrapper: bind actual extracted inputs to the frozen source tree."""
import datetime,hashlib,json,os,pathlib,subprocess,sys,time,fcntl,re,difflib
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
def guarded_sources():
    exact=dict(files);metadata=[]
    for name,expected in files.items():
        actual=cwd/name
        if name.endswith('.opam') and actual.is_file() and digest(actual)!=expected:
            frozen=(r/'frozen-sources'/pkg/name).read_text(); generated=actual.read_text()
            # opam source inserts a version header; dune may place that SAME
            # header after opam-version and normalize only the final newline.
            def parts(text):
                lines=text.splitlines();versions=[x for x in lines if x.startswith('version: ')]
                return sorted(versions), '\n'.join(x for x in lines if not x.startswith('version: ')).rstrip()
            if parts(frozen)!=parts(generated):raise RuntimeError('substantive opam metadata change '+name)
            metadata.append({'file':name,'initial_sha256':expected,'generated_sha256':digest(actual),
                'diff':''.join(difflib.unified_diff(frozen.splitlines(True),generated.splitlines(True),fromfile='frozen/'+name,tofile='generated/'+name))})
            exact.pop(name)
    result=check(cwd,exact);result['metadata_version_relocation_only']=metadata
    return result
log=r/'build-guards';log.mkdir(exist_ok=True);target=log/(pkg+'-'+str(time.time_ns())+'.json')
meta={'package':pkg,'cwd':str(cwd),'argv':sys.argv[1:],'started':datetime.datetime.now(datetime.timezone.utc).isoformat(),'synthetic_metadata_absent_in_build':synthetic}
try:
 meta['initial']=guarded_sources();target.write_text(json.dumps(meta,indent=2)+'\n')
 rc=subprocess.run(sys.argv[1:]).returncode;meta['exit_code']=rc
 meta['final']=guarded_sources()
except Exception as e:
 meta['guard_error']=str(e);rc=97
meta['finished']=datetime.datetime.now(datetime.timezone.utc).isoformat();target.write_text(json.dumps(meta,indent=2)+'\n');sys.exit(rc)
