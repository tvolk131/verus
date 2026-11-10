"""Bind commands/logs, exact selected inventories and the post-run source freeze."""
import json,hashlib,re,subprocess,os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).parent;RAW=Path('/tmp/observer-origin-review-d35c31fa')
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def load(p):return json.loads(Path(p).read_text())
freeze=load(HERE/'source-freeze.json')
drift=[p for p,h in freeze['files'].items() if sha(ROOT/p)!=h]
if drift:raise RuntimeError('tracked source drift: '+repr(drift))
runtime=load(RAW/'runtime.json')
if any(sha(p)!=h for p,h in runtime['files'].items()):raise RuntimeError('runtime drift')
bundle=load(RAW/'bundle/copy-manifest.json')
if any(sha(RAW/'bundle'/p)!=v['sha256'] for p,v in bundle['files'].items()):raise RuntimeError('bundle drift')
if sha(ROOT/'source/vstd/Cargo.lock')!=sha(HERE/'vstd-build.Cargo.lock'):raise RuntimeError('vstd lock drift')
checks=[]
for c in load(RAW/'validation.json'):
 t=Path(c['log']).read_text();m=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',t)
 checks.append(dict(**c,results=[dict(zip(['status','passed','failed','ignored','measured','filtered'],[x[0]]+list(map(int,x[1:])))) for x in m]))
targets={p.stem for p in (ROOT/'source/rust_verify_test/tests').glob('infer*.rs')}
labels=[c['label'] for c in checks]
if len(targets)!=32 or len(labels)!=len(set(labels)) or set(labels)!=targets|{'vir','vstd','vstd-lock'}:raise RuntimeError('incomplete validation inventory')
if any(c['returncode']!=0 for c in checks):raise RuntimeError('validation command failed')
for c in checks:
 if c['label'] not in targets|{'vir'}:continue
 if len(c['results'])!=1 or c['results'][0]['status']!='ok' or any(c['results'][0][k] for k in ['failed','ignored','measured','filtered']):raise RuntimeError('incomplete test target '+c['label'])
if 'verification results:: 2226 verified, 0 errors' not in (RAW/'vstd.log').read_text():raise RuntimeError('vstd incomplete')
probes=load(RAW/'probes/results.json');can=next((RAW/'canonical').glob('*/receipt.json'));obs=next((RAW/'observations').glob('*/receipt.json'))
receipts={}
for name,p in [('canonical',can),('observations',obs)]:
 d=load(p)
 receipts[name]=dict(path=str(p),sha256=sha(p),success=d['success'],commands=d['commands'],runs=[{k:v for k,v in r.items() if k in ['label','verification','expected_execs','expected_proofs','expected_helpers','expected_failure_kind','source_sha256','stdout_sha256','stderr_sha256','diagnostic_only','rejection','poisoned_helper']} for r in d['runs']])
receipts['probes']=dict(path=str(RAW/'probes/results.json'),sha256=sha(RAW/'probes/results.json'),success=probes['success'],commands=probes['commands'],objections=probes['objections'],runs=probes['runs'])
supplement=load(RAW/'supplement/results.json')
receipts['supplement']=dict(path=str(RAW/'supplement/results.json'),sha256=sha(RAW/'supplement/results.json'),success=supplement['success'],commands=supplement['commands'],objections=supplement['objections'],runs=supplement['runs'])
logs={str(p.relative_to(RAW)):dict(sha256=sha(p),size=p.stat().st_size) for p in RAW.rglob('*') if p.is_file() and p.suffix in ['.log','.stdout','.stderr','.json','.txt'] and 'cache' not in p.parts and 'compiler-cache' not in p.parts}
manifest=dict(reviewed_head=freeze['head'],raw=str(RAW),source_freeze_sha256=sha(HERE/'source-freeze.json'),source_file_count=len(freeze['files']),tracked_source_drift=drift,build=dict(argv=['cargo','build','--release','--locked','-p','rust_verify','-p','verus','-p','cargo-verus','-p','vstd_build'],cwd=str(ROOT/'source'),env=dict(PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],CARGO_BUILD_JOBS='2',VERUS_Z3_PATH=str(ROOT/'source/z3')),log_sha256=sha(RAW/'build.log')),validation=checks,session_completion=load(RAW/'session-completion.json'),bundle=load(RAW/'bundle/copy-manifest.json'),runtime=load(RAW/'runtime.json'),replays=receipts,audits=dict(canonical=load(RAW/'canonical-audit.json'),independent=load(RAW/'independent-audit.json')),raw_files=logs,evidence_files={str(p.relative_to(HERE)):sha(p) for p in HERE.rglob('*') if p.is_file() and p.name!='manifest.json'})
(HERE/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('frozen sources',len(freeze['files']),'validation entries',len(checks),'primary probes',len(probes['runs']),'supplement',len(supplement['runs']))
