#!/usr/bin/env python3
"""Prepare all controls and immutable prospective manifest BEFORE their trials."""
import hashlib
import json
from pathlib import Path
HERE=Path(__file__).resolve().parent
base=(HERE/'protocol.rs').read_text()

def append(body):
    i=base.rfind('\n}')
    return base[:i]+'\n'+body+base[i:]

cases={
 'early_ack': (base.replace('p.phase == Phase::SecondBarrier && q.phase == Phase::Acknowledged',
                           'p.phase == Phase::Preparing && q.phase == Phase::Acknowledged'),
               'postcondition','Ack guard weakened to Preparing; ack_is_after_second must reject'),
 'publication_before_barrier': (base.replace('p.phase == Phase::FirstBarrier && p.buffered.len() == 0',
                                           'p.phase == Phase::Preparing && p.buffered.len() == 0'),
                'postcondition','publication_requires_first must reject publication before barrier1'),
 'launder_corrupt_bytes': (base.replace('if s.detected_corrupt { s.raw } else { canonical }','canonical'),
                'postcondition','detected_corrupt_preserved must reject canonical reserialization'),
 'header_only_coherence': (append('''pub proof fn false_header_only_coherence(h: Header)
    ensures header_select(h) != Selection::Reject ==> selected_good(h,header_select(h)),
{}
'''), 'postcondition','Newer checksum-valid secondary need not have complete pages; prior=false witness retained'),
 'error_implies_rollback': (append('''pub proof fn false_error_implies_rollback(s: Storage, t: Storage, image: Seq<u8>)
    requires wf(s), storage_step(s,t,Event::SyncError(image)),
    ensures t.persistent == s.base,
{}
'''), 'postcondition','Failed sync may persist all pending bytes; full/zero progress witnesses retained'),
 'whole_write_atomic': (append('''pub proof fn false_whole_write_atomic(s: Storage, image: Seq<u8>)
    requires wf(s), s.pending.len() == 1, allowed(s,image),
    ensures image == s.base || image == s.volatile,
{}
'''), 'postcondition','Byte-level tearing is permitted; torn_header_witness retained'),
 'missing_write_coverage': (append('''pub proof fn false_sync_without_coverage(s: Storage, t: Storage, c: Candidate)
    requires wf(s), storage_step(s,t,Event::SyncSuccess),
    ensures matches(t.persistent,c),
{}
'''), 'postcondition','A successful barrier cannot cover bytes never submitted; no candidate-coverage premise'),
 'missing_retention': (append('''pub proof fn false_retention_without_frame(s: Storage, t: Storage, c: Candidate, w: Write)
    requires wf(s), retained(s,c), matches(s.volatile,c), storage_step(s,t,Event::Submit(w)),
    ensures retained(t,c), matches(t.volatile,c),
{}
'''), 'postcondition','An overwrite of an old-root byte invalidates retention; frame premise necessary'),
}
manifest=dict(model_sha256=hashlib.sha256(base.encode()).hexdigest(),prospective=True,cases={})
d=HERE/'controls'; d.mkdir(exist_ok=True)
for name,(source,expected,meaning) in cases.items():
    if name in ('early_ack','publication_before_barrier'):
        # This positive construction is invalid under the intentionally broken guard;
        # omit it so the negative has only the intended postcondition failure.
        source=source[:source.index('pub proof fn concrete_two_barrier_success')]+ '}\n'
        meaning += '; positive construction excluded from this mutant scope'
    assert source!=base
    p=d/(name+'.rs'); p.write_text(source)
    manifest['cases'][name]=dict(sha256=hashlib.sha256(source.encode()).hexdigest(),
        expected=expected,meaning=meaning,source='controls/'+p.name)
(HERE/'CONTROLS.json').write_text(json.dumps(manifest,indent=2)+'\n')
