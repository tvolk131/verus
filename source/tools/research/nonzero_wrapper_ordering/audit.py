"""Read-only prerequisite audit; exception gates work with Python -O -B."""
import ast, hashlib, json, re, subprocess, sys
from pathlib import Path
import build as b
import fixtures as f
H=b.HERE;R=b.root()
def read(p): return json.loads(p.read_text())
def check(ok,msg):
    if not ok: raise RuntimeError(msg)
def audit_receipt(label):
    d=R/label;j=read(d/'receipt.json');z=read(d/'freeze.json')
    check(j['guards'],'last guards '+label)
    for name in ['freeze.json','stdout','stderr']:
        key={'freeze.json':'freeze_sha256','stdout':'stdout_sha256','stderr':'stderr_sha256'}[name]
        check(b.sha(d/name)==j[key],'receipt artifact '+label+'/'+name)
    for p,h in z.get('source',{}).items():check(b.sha(b.ROOT/p)==h,'production source '+p)
    for n,h in z.get('controllers',{}).items():check(b.sha(H/n)==h,'controller '+n)
    for n,h in z.get('tools',{}).items():check(b.sha(R/'bundle'/n)==h,'tool '+n)
    for p,h in z.get('inputs',{}).items():check(b.sha(p)==h,'diagnostic input '+p)
    for p,h in z.get('fixtures',{}).items():check(b.sha(p)==h,'fixture '+p)
    for p,h in z.get('toolchain',{}).get('files',{}).items():check(b.sha(p)==h,'toolchain '+p)
    return j
check(sys.flags.dont_write_bytecode,'-B required')
S=read(H/'SEAL.json')
for p,h in S.items():check(b.sha(H/p)==h,'sealed research file '+p)
for p in H.glob('*.py'):
    check(not any(isinstance(n,ast.Assert) for n in ast.walk(ast.parse(p.read_text()))),'assert-based gate '+p.name)
check(b.manifest()==read(R/'setup.json')['source'],'full production source unchanged')
changed=subprocess.check_output(['git','diff','--name-only','913044c4'],cwd=b.ROOT).decode().splitlines()
check(all(p.startswith('source/tools/research/nonzero_wrapper_ordering/') for p in changed),'production diff')
for p,h in read(R/'toolchain.json')['files'].items():check(b.sha(p)==h,'compiler/core input '+p)
J=read(H/'RESULTS.json');check(J['status']=='blocked_before_production' and not J['publication'] and not J['goal_complete'],'scope')
check(J['source_proofs']==0 and J['native_correspondences']==0,'no proof claim')
check(audit_receipt('build')['exit']==0,'compiler build')
check(audit_receipt('vstd-prerequisite')['exit']!=0,'missing solver must refuse')
check('authorized solver missing; vstd/proof trials forbidden' in (R/'vstd-prerequisite/stderr').read_text(),'precise prerequisite refusal')
check(not (R/'eight-tools.json').exists(),'no eight-tool freeze claimed')
check(not any((R/'bundle'/p).exists() for p in ['z3','vstd.vir','libvstd.rlib']),'missing inputs remain absent')
D=read(R/'diagnostics.json');check(D==J['diagnostics'],'diagnostic binding')
for mode,z in D['fixtures'].items():
    check(b.sha(z['path'])==z['sha256'],'source fixture '+mode)
    check(hashlib.sha256(f.source(mode).encode()).hexdigest()==z['sha256'],'source generation '+mode)
check(D['declaration_sha256']==f.DECL_SHA,'declaration identity')
check(b.sha(R/'native.rs')==D['native_source_sha256'],'native source')
check(b.sha(R/'diagnostic-final.mir')==D['mir_sha256'],'native mir')
mir=(R/'diagnostic-final.mir').read_text();starts=list(re.finditer(r'^fn (.*?)\n',mir,re.M))
check(len(starts)==15 and len(D['native_MIR_sections'])==15,'complete section inventory')
for i,m in enumerate(starts):
    section=mir[m.start():starts[i+1].start() if i+1<len(starts) else len(mir)]
    row=D['native_MIR_sections'][i]
    check(row==dict(signature=m[1],section_sha256=hashlib.sha256(section.encode()).hexdigest(),verified=False),'MIR section '+str(i))
for label in ['expanded','mir']:
    check(audit_receipt('diagnostic-final-'+label)['exit']==0,'native compiler '+label)
for mode in ['original','request','authored']:
    for setting in ['off','on']:
        check(J['baselines'][mode][setting]['classification']=='not_run_infrastructure_prerequisite','no false semantic classification')
print(json.dumps(dict(audit='passed',optimized=sys.flags.optimize>0,source_proofs=0,native_correspondences=0,publication=False,goal_complete=False),sort_keys=True))
