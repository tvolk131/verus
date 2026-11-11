"""Capture reproducible normal/optimized audit outcomes; logs stay private."""
from review import *
need(sys.flags.dont_write_bytecode,'python -B')
phase=sys.argv[1];need(phase in ['worker','final'],'phase')
script=HERE/('authenticate.py' if phase=='worker' else 'audit.py');record=HERE/('AUDIT_'+phase.upper()+'.json')
need(not record.exists(),'immutable audit record')
rows={}
for name,flags in [('normal',['-B']),('optimized',['-O','-B'])]:
 d=root()/'audit-runs'/(phase+'-'+name);d.mkdir(parents=True,exist_ok=False)
 args=[sys.executable,*flags,str(script)];start=time.monotonic()
 with (d/'stdout').open('wb') as out,(d/'stderr').open('wb') as err:p=subprocess.run(args,cwd=ROOT,stdout=out,stderr=err)
 rows[name]=dict(argv=args,exit=p.returncode,seconds=time.monotonic()-start,stdout_sha256=sha(d/'stdout'),stderr_sha256=sha(d/'stderr'))
 need(p.returncode==0,'audit failure '+str(d))
write(record,dict(auditor_sha256=sha(script),runs=rows,normal_optimized_identical=rows['normal']['stdout_sha256']==rows['optimized']['stdout_sha256']))
print('PASS',phase,'normal + optimized')
