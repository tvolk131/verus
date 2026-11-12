#!/usr/bin/env python3
"""Package immutable receipts; quarantine exports, which are never proof inputs."""
import hashlib
import json
from pathlib import Path
HERE=Path(__file__).resolve().parent

def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()

records=[]
locations=json.loads((HERE/'ARTIFACT_LOCATIONS.json').read_text()) if (HERE/'ARTIFACT_LOCATIONS.json').exists() else {}
for value in locations.values():
    assert sha(Path(value['private_path'])) == value['sha256']
for path in sorted((HERE/'evidence').glob('*/receipt.json')):
    r=json.loads(path.read_text())
    d=path.parent
    assert sha(d/'candidate.rs') == r['freeze']['candidate_sha256']
    for run in r['runs']:
        for ext in ('stdout','stderr'):
            assert sha(d/(run['label']+'.'+ext)) == run[ext+'_sha256']
    for artifact in sorted(d.glob('*.vir')):
        private=Path(r['private_bundle']).parent/'quarantined-exports'
        private.mkdir(exist_ok=True)
        dest=private/artifact.name
        assert not dest.exists()
        locations[str(artifact.relative_to(HERE))]=dict(sha256=sha(artifact),private_path=str(dest),
            note='relocated after checks; never imported; export presence alone is not proof evidence')
        artifact.rename(dest)
    records.append(dict(path=str(path.relative_to(HERE)),sha256=sha(path),
        candidate_sha256=r['freeze']['candidate_sha256'],expected=r['freeze']['expected'],
        success=r['success'],commands=len(r['commands']),
        compiler_seconds=sum(c['seconds'] for c in r['commands']),setup_seconds=r.get('setup_seconds',0),
        labels=[c['label'] for c in r['commands']],failure=r.get('failure')))
(HERE/'ARTIFACT_LOCATIONS.json').write_text(json.dumps(locations,indent=2)+'\n')
final=next(r for r in records if 'header_and_pages_ordinary' in r['labels'])
assert final['success'] and final['candidate_sha256']==sha(HERE/'protocol.rs')
controls=json.loads((HERE/'CONTROLS.json').read_text())
assert controls['model_sha256']==sha(HERE/'protocol.rs')
accepted_controls={}
for name,c in controls['cases'].items():
    assert sha(HERE/c['source'])==c['sha256']
    matches=[r for r in records if r['success'] and r['candidate_sha256']==c['sha256']
        and name+'_ordinary' in r['labels'] and r['expected']==c['expected']]
    assert len(matches)==1
    accepted_controls[name]=matches[0]['path']
report=dict(parent='c008326bfed88eab958335412cdc5ec0e9c8130e',interface_commit='4f376735',
    model_commit='a5a3dca7',model_sha256=sha(HERE/'protocol.rs'),model_theorem=True,
    source_coupling_proved=False,application_goal_achieved=False,native_fault_baseline=False,
    final_positive=final['path'],controls=accepted_controls,all_trials=records,
    total_compiler_seconds=sum(r['compiler_seconds'] for r in records),
    total_setup_seconds=sum(r['setup_seconds'] for r in records),
    total_invocations=sum(r['commands'] for r in records),
    final_invocations=18,positive_verified_obligations_per_mode=26,
    selected_proof_bodies=21,selected_exec_bodies=0,infer_helpers=0,
    existing_accepted78_infer_registry_entries=359,registry_sha256=sha(HERE/'ACCEPTED78_INFER_REGISTRY.json'),
    no_new_trusted_models=True,no_engineering_changes=True,infer_annotation_reduction=0,
    domain='arbitrary finite byte sequences and finite traces with explicit frame/entry premises; concrete witnesses are not exhaustive enumeration',
    HMAC_native='DEFERRED AND UNACHIEVED; native0/44+3/101 untouched')
(HERE/'RESULTS.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:report[k] for k in ('total_compiler_seconds','total_setup_seconds','total_invocations','final_invocations')},indent=2))
