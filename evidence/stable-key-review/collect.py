"""Require complete scoped commands, bind immutable sources/runtime and raw evidence."""
import json,hashlib,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent;RAW=Path('/tmp/stable-key-review-e84eed6c')
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def load(p):return json.loads(Path(p).read_text())
def require(ok,msg):
    if not ok:raise RuntimeError(msg)
freeze=load(HERE/'source-freeze.json');drift=[p for p,h in freeze['files'].items() if sha(ROOT/p)!=h];require(not drift,'tracked source drift '+str(drift))
runtime=load(RAW/'runtime.json');require(all(sha(p)==h for p,h in runtime['files'].items()),'runtime drift')
bundle=load(RAW/'bundle/copy-manifest.json');require(all(sha(RAW/'bundle'/p)==v['sha256'] for p,v in bundle['files'].items()),'bundle drift')
require(sha(ROOT/'source/vstd/Cargo.lock')==sha(HERE/'vstd-build.Cargo.lock'),'vstd lock drift')
checks=[]
for c in load(RAW/'validation.json'):
    require(sha(c['log'])==c['sha256'],'command log drift')
    matches=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',Path(c['log']).read_text())
    checks.append(dict(**c,results=[dict(zip(['status','passed','failed','ignored','measured','filtered'],[x[0]]+list(map(int,x[1:])))) for x in matches]))
expected={'compiler-build','vstd-lock','vstd','vir','infer_buffer_observation_goals','infer_mutable_buffer_goals','infer'}
require(len(checks)==len(expected) and {c['label'] for c in checks}==expected,'validation scope incomplete/duplicated')
require(all(c['returncode']==0 for c in checks),'validation command failed')
for c in checks:
    if c['label'] in {'vir','infer_buffer_observation_goals','infer_mutable_buffer_goals','infer'}:
        require(len(c['results'])==1 and c['results'][0]['status']=='ok' and not any(c['results'][0][k] for k in ['failed','ignored','measured','filtered']),'incomplete tests '+c['label'])
require('verification results:: 2226 verified, 0 errors' in (RAW/'vstd.log').read_text(),'vstd incomplete')
receipts={}
for name in ['canonical','observations']:
    p=next((RAW/name).glob('*/receipt.json'));d=load(p);require(d['success'],'replay failure')
    receipts[name]=dict(path=str(p),sha256=sha(p),success=True,commands=d['commands'],runs=[{k:v for k,v in r.items() if k in ['label','verification','expected_execs','expected_proofs','expected_helpers','expected_failure_kind','source_sha256','stdout_sha256','stderr_sha256','diagnostic_only','rejection','poisoned_helper']} for r in d['runs']])
probes=load(RAW/'probes/results.json');adjudication=load(RAW/'probe-adjudication.json');require(adjudication['success'] and sha(RAW/'probes/results.json')==adjudication['original_receipt_sha256'],'probes incomplete')
receipts['probes']=dict(path=str(RAW/'probes/results.json'),sha256=sha(RAW/'probes/results.json'),adjudication=adjudication,**{k:probes[k] for k in ['commands','objections','runs']})
audits={name:load(RAW/(name+'-audit.json')) for name in ['canonical','independent']};require(all(a['success'] for a in audits.values()),'audit failure')
logs={str(p.relative_to(RAW)):dict(sha256=sha(p),size=p.stat().st_size) for p in RAW.rglob('*') if p.is_file() and p.suffix in ['.log','.stdout','.stderr','.json','.txt'] and not any('cache' in part for part in p.relative_to(RAW).parts)}
manifest=dict(reviewed_head=freeze['head'],reviewed_scope=freeze['scope'],production_base=freeze['base'],raw=str(RAW),source_scope=load(RAW/'source-scope.json'),reference_comparison=load(RAW/'reference-comparison.json'),source_freeze_sha256=sha(HERE/'source-freeze.json'),source_file_count=len(freeze['files']),tracked_source_drift=drift,validation=checks,compiler_suite_scope=dict(targets=['infer_buffer_observation_goals','infer_mutable_buffer_goals','infer'],full_32_target_suite_run=False,other_29_targets_not_run=True),bundle=bundle,runtime=runtime,replays=receipts,audits=audits,raw_files=logs,evidence_files={str(p.relative_to(HERE)):sha(p) for p in HERE.rglob('*') if p.is_file() and p.name!='manifest.json'})
(HERE/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('bound',len(freeze['files']),'sources;',len(checks),'commands;',len(probes['runs']),'probe invocations')
