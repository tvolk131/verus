"""Cold/warm runs with byte-identical argv, environment, sources and tool bundle.
Archive each completed run before reusing its exact output paths.
"""
from pathlib import Path
import json
import shutil
import sys
import trials as t
r=Path(sys.argv[1]);b=r/'bundle'
for kind,expected in [('actual',2),('full-help',52)]:
    label='stable-'+kind
    previous=None
    for temperature in ['cold','warm']:
        # A separate cache makes the first run cold without removing any prior evidence.
        case_root=r/label;case_root.mkdir(exist_ok=True)
        d,record=t.trial(case_root,b,'run',kind)
        t.need(record['exit']==0 and record['verification']['verified']==expected,'stable proof')
        archive=case_root/temperature
        t.need(not archive.exists(),'archive must be fresh')
        shutil.copytree(d,archive)
        current=json.loads((archive/'freeze.json').read_text())
        if previous:
            t.need(current==previous,'cold/warm frozen input identity changed')
            a=json.loads((archive/'stdout').read_text())
            old=json.loads((case_root/'cold/stdout').read_text())
            if kind=='full-help':
                f=a['verus-infer']['functions'][0];g=old['verus-infer']['functions'][0]
                # Parent owns cache73: cold generated and cached decoded formulas
                # currently carry different diagnostic proposal spans. Preserve
                # and report this exact identity drift; never call the IDs equal.
                t.write(case_root/'cache-reconciliation.json',dict(cold_candidate=g['candidate_id'],warm_candidate=f['candidate_id'],same_candidate=(f['candidate_id']==g['candidate_id']),parent_owned=True))
                t.need(f['metrics']['cache']=='hit','warm cache was not reused')
                for proof in f['proofs'].values():t.need(proof['required'] and proof['status']=='proved','mandatory proof missing')
        previous=current
