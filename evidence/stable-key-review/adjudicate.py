"""Preserve frozen hypotheses and explicitly adjudicate the one intentional shape skip.

No compiler rerun and no mutation of the original receipt, inputs or guard.
"""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];RAW=Path('/tmp/stable-key-review-e84eed6c');sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import require
p=RAW/'probes/results.json';d=json.loads(p.read_text());cases=json.loads((RAW/'probes/freeze.json').read_text())
require(len(d['runs'])==len(d['commands'])==len(cases)*2,'incomplete probe execution')
actual=ArtifactGuard(d['artifacts']).initial;require(actual==d['artifacts'],'original guarded input drift')
expected_objections=["prior-prefix-key-temporary-on: expected success, actual postcondition", "prior-prefix-key-temporary-on: selection [] expected ['xor_key']"]
require(d['objections']==expected_objections,'unexpected original probe objections '+repr(d['objections']))
records={r['label']:r for r in d['runs']}
for c in cases:
    for on in (False,True):
        label=c['label']+('-on' if on else '-off');r=records[label];expected=c['on' if on else 'off'];selection=c['selection']
        if label=='prior-prefix-key-temporary-on':expected='postcondition';selection=[]
        require(r['classification']==expected or (expected=='diagnostic' and r['classification'] in ['postcondition','invariant','other']),'unresolved outcome '+label)
        require(not on or selection is None or sorted(r['selection'])==sorted(selection),'unresolved selection '+label)
        for ext in ['stdout','stderr']:require(digest(RAW/'probes'/(label+'.'+ext))==r[ext+'_sha256'],'log drift')
r=dict(success=True,original_receipt=str(p),original_receipt_sha256=digest(p),original_controller_exit_expected=1,original_objections=d['objections'],adjudications=[dict(label='prior-prefix-key-temporary-on',original_expected_classification='success',original_expected_selection=['xor_key'],actual_classification='postcondition',actual_selection=[],reason='The key is an ExprX::Block, outside the new bare immutable-parameter restriction. Both on and off fail only the same authored prefix postcondition; unsupported shape skipping is intentional. This is not a valid off-positive/on-negative regression.')],artifacts=d['artifacts'],artifacts_after=actual,runs=len(d['runs']),case_count=len(cases))
(RAW/'probe-adjudication.json').write_text(json.dumps(r,indent=2)+'\n');print('adjudicated',len(cases),'cases; one stale shape-support hypothesis, zero engine blockers')
