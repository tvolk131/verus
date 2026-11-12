#!/usr/bin/env python3
"""Read-only reuse of committed source projection; no executable body edits."""
import sys
sys.dont_write_bytecode = True
import importlib.util
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent/'source'))
spec = importlib.util.spec_from_file_location('committed_source_probes', HERE.parent/'source/probes.py')
p = importlib.util.module_from_spec(spec)
spec.loader.exec_module(p)
NOMINAL = '''
// Opaque nominal identities only. No method specifications, views or field assumptions.
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExIoError(std::io::Error);
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPanicLocation<'a>(core::panic::Location<'a>);
'''
def baseline(nominal=True):
    text = p.selection()
    if nominal:
        text = text.replace('verus! {', 'verus! {\n'+NOMINAL, 1)
    return text

SPECS = r'''
pub open spec fn selection_errors(h: UnrepairedDatabaseHeader) -> bool {
    h.primary_corrupted && (h.inner.two_phase_commit || h.secondary_corrupted)
}
pub open spec fn selection_switches(h: UnrepairedDatabaseHeader) -> bool {
    !h.inner.two_phase_commit && !h.secondary_corrupted &&
        (h.primary_corrupted ||
        h.inner.transaction_slots[(h.inner.primary_slot ^ 1) as int].transaction_id.0 >
            h.inner.transaction_slots[h.inner.primary_slot as int].transaction_id.0)
}
'''
DOMAIN = 'old(self).inner.primary_slot < 2, (old(self).inner.primary_slot ^ 1) < 2, old(self).inner.primary_slot != (old(self).inner.primary_slot ^ 1),'
def contracted():
    text=baseline().rsplit('}',1)[0]+'\n'+SPECS+'\n}\n'
    raw=p.visibility(p.item(p.H,'fn','select_primary_slot'))
    signature, body=raw.split('{',1)
    signature=signature.replace('-> Result<bool>', '-> (r: Result<bool>)')
    spec='\nrequires '+DOMAIN+'\nensures\n'
    spec+='    r.is_Err() == selection_errors(*old(self)),\n'
    spec+='    match r {\n        Ok(kept) => kept == !selection_switches(*old(self)),\n'
    spec+='        Err(StorageError::Corrupted(msg)) => msg@ == (if old(self).inner.two_phase_commit {\n'
    spec+='            "Primary is corrupted despite 2-phase commit"@\n        } else { "Both commit slots are corrupted"@ }),\n'
    spec+='        _ => false,\n    },\n'
    spec+='    final(self).inner.primary_slot == (if selection_switches(*old(self)) { old(self).inner.primary_slot ^ 1 } else { old(self).inner.primary_slot }),\n'
    spec+='    final(self).inner.primary_slot < 2,\n'
    for name in ['transaction_slots',*p.FRAME]:
        spec+='    final(self).inner.'+name+' == old(self).inner.'+name+',\n'
    for name in ['primary_corrupted','secondary_corrupted']:
        spec+='    final(self).'+name+' == old(self).'+name+',\n'
    text=text.replace(raw,signature+spec+'{'+body)
    raw=p.visibility(p.item(p.H,'fn','finalize_transaction_slots'))
    signature,body=raw.split('{',1)
    signature=signature.replace('-> Result<DatabaseHeader>', '-> (r: Result<DatabaseHeader>)')
    spec=r'''
ensures r.is_Err() == self.primary_corrupted,
    match r {
        Ok(header) => header == self.inner,
        Err(StorageError::Corrupted(msg)) => msg@ == "Primary commit slot is corrupted"@,
        _ => false,
    },
'''
    return publicize(text.replace(raw,signature+spec+'{'+body))


def publicize(text):
    import re
    text=text.replace('pub(crate)', 'pub')
    text=text.replace('fn select_primary_slot', 'pub fn select_primary_slot')
    text=text.replace('struct TransactionId(u64);', 'struct TransactionId(pub u64);')
    return re.sub(r'(?m)^(    )(\w+): (.+),$', r'\1pub \2: \3,',text)

def eligible():
    # Closeout only: slot-domain eligibility plus complete independent finalize contract.
    # The rejected complete ordering contract remains available as contracted().
    text=contracted()
    start=text.index('    pub fn select_primary_slot')
    body=text.index('{\n        // If the primary',start)
    text=text[:start]+'''    pub fn select_primary_slot(&mut self) -> Result<bool>
requires '''+DOMAIN+'\n'+text[body:]
    return text.replace('\n'+SPECS+'\n','\n')

def closeout_cases():
    text=eligible()
    return {'eligible':text,
        'finalize_wrong':text.replace('r.is_Err() == self.primary_corrupted','r.is_Err() == !self.primary_corrupted'),
        'selection_unsafe':text.replace('requires '+DOMAIN,'requires true,')}
