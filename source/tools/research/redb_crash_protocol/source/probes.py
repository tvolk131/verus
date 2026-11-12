#!/usr/bin/env python3
"""Bounded field-preserving contexts. No replacement backend/checksum/Mutex summaries."""
import hashlib, json, re
from pathlib import Path
from freeze import HERE, span
O = HERE/'original'
def item(rel, kind, name, occurrence=0):
    text=(O/rel).read_text()
    matches=list(re.finditer(r'(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?'+kind+r'\s+'+name+r'\b',text))
    start=matches[occurrence].start(); end=span(text,start)
    if kind == 'struct' and ';' in text[start:text.index('{',start)]:
        end=text.index(';',start)+1
    return text[start:end]
def visibility(s): return s.replace('pub(super)', 'pub(crate)')
def annotate(s, spec):
    at=s.index('{')
    # Return binding changes signature only; complete executable body is identical.
    head=s[:at]
    if '-> &TransactionHeader' in head: head=head.replace('-> &TransactionHeader','-> (r: &TransactionHeader)')
    return visibility(head+'\n'+spec+'\n'+s[at:])
def common():
    pieces=[]
    for rel, name, derive in [
        ('src/tree_store/page_store/base.rs','PageNumber','Copy, Clone, Eq, PartialEq'),
        ('src/transaction_tracker.rs','TransactionId','Copy, Clone, Ord, PartialOrd, Eq, PartialEq'),
        ('src/tree_store/btree_base.rs','BtreeHeader','Copy, Clone, Eq, PartialEq'),
        ('src/tree_store/page_store/header.rs','TransactionHeader','Clone'),
        ('src/tree_store/page_store/header.rs','DatabaseHeader','Clone')]:
        pieces.append('#[derive('+derive+')]\n'+visibility(item(rel,'struct',name)))
    return '\n'.join(pieces)
H='src/tree_store/page_store/header.rs'
VALID='self.primary_slot < 2, (self.primary_slot ^ 1) < 2, self.primary_slot != (self.primary_slot ^ 1),'
OLDVALID=VALID.replace('self.', 'old(self).')
FRAME=['recovery_required','two_phase_commit','primary_unflushed','page_size','region_header_pages',
       'region_max_data_pages','full_regions','trailing_partial_region_pages']
def frame(): return '\n'.join('final(self).'+f+' == old(self).'+f+',' for f in FRAME)
def publication():
    defs=common()
    methods=[]
    for name,spec in [
        ('primary_slot',f'requires {VALID}\nensures *r == self.transaction_slots[self.primary_slot as int],'),
        ('secondary_slot',f'requires {VALID}\nensures *r == self.transaction_slots[(self.primary_slot ^ 1) as int],'),
        ('published_slot',f'''requires {VALID}
ensures *r == self.transaction_slots[(if self.primary_unflushed {{ self.primary_slot ^ 1 }} else {{ self.primary_slot }}) as int],'''),
        ('swap_primary_slot',f'''requires {OLDVALID}
ensures final(self).primary_slot == old(self).primary_slot ^ 1,
final(self).transaction_slots == old(self).transaction_slots,
{frame()}'''),
        ('write_secondary_slot',f'''requires {OLDVALID}
ensures final(self).primary_slot == old(self).primary_slot,
final(self).transaction_slots[old(self).primary_slot as int] == old(self).transaction_slots[old(self).primary_slot as int],
final(self).transaction_slots[(old(self).primary_slot ^ 1) as int].transaction_id == transaction_id,
final(self).transaction_slots[(old(self).primary_slot ^ 1) as int].user_root == user_root,
final(self).transaction_slots[(old(self).primary_slot ^ 1) as int].system_root == system_root,
final(self).transaction_slots[(old(self).primary_slot ^ 1) as int].corrupt_bytes == None,
final(self).transaction_slots[(old(self).primary_slot ^ 1) as int].version == old(self).transaction_slots[(old(self).primary_slot ^ 1) as int].version,
{frame()}''')]:
        methods.append(annotate(item(H,'fn',name),spec))
    return 'use vstd::prelude::*;\nverus! {\nconst TRANSACTION_SIZE: usize = 128;\ntype Checksum = u128;\n'+defs+'\nimpl DatabaseHeader {\n'+'\n'.join(methods)+'\n}\n}\n'
def selection():
    # Keep exact error variants and fields. std::io maps the default std context; no law supplied.
    text=publication()
    text=text.rsplit('}',1)[0]
    text+='\nuse std::io;\nuse core::panic;\n'+item('src/error.rs','enum','StorageError')+'\n'
    text+='pub type Result<T = (), E = StorageError> = core::result::Result<T, E>;\n'
    text+=visibility(item(H,'struct','UnrepairedDatabaseHeader'))+'\nimpl UnrepairedDatabaseHeader {\n'
    text+=visibility(item(H,'fn','select_primary_slot'))+'\n'+visibility(item(H,'fn','finalize_transaction_slots'))+'\n}\n}\n'
    return text

def generate(out):
    out=Path(out)
    texts={'publication':publication(), 'selection':selection(), 'supported':supported()}
    # Semantic/safety controls preserve executable bodies; only one asserted contract changes.
    texts['publication_wrong']=texts['publication'].replace(
        'if self.primary_unflushed { self.primary_slot ^ 1 } else { self.primary_slot }',
        'if self.primary_unflushed { self.primary_slot } else { self.primary_slot ^ 1 }')
    texts['publication_unsafe']=texts['publication'].replace('requires '+VALID, 'requires true,')
    # Full original serialization/checksum context; no checksum callback summary.
    t=publication().rsplit('}',1)[0]
    original=(O/H).read_text()
    constants=original[original.index('pub(super) const MAGICNUMBER'):original.index('pub(crate) const PAGE_SIZE:')]
    # TRANSACTION_SIZE is already in publication; include all other exact constants.
    constants=constants.replace('const TRANSACTION_SIZE: usize = 128;', '')
    t+='\nuse core::mem::size_of;\n'+visibility(constants)+'\n'
    t+='const FILE_FORMAT_VERSION3: u8 = 3;\n'
    t+='impl TransactionId {\n'+item('src/transaction_tracker.rs','fn','raw_id')+'\n}\n'
    t+='impl PageNumber {\n'+item('src/tree_store/page_store/base.rs','fn','serialized_size')+'\n'+item('src/tree_store/page_store/base.rs','fn','to_le_bytes')+'\n}\n'
    t+='impl BtreeHeader {\n'+item('src/tree_store/btree_base.rs','fn','serialized_size')+'\n'+item('src/tree_store/btree_base.rs','fn','to_le_bytes')+'\n}\n'
    t+='impl TransactionHeader {\n'+visibility(item(H,'fn','to_bytes',1))+'\n}\n'
    t+='impl DatabaseHeader {\n'+visibility(item(H,'fn','to_bytes',0))+'\n}\n'
    t+=item('src/tree_store/page_store/page_manager.rs','fn','xxh3_checksum')+'\n}\n'
    # Complete bundled native checksum, rather than a callback law.
    xxh3=(O/'src/tree_store/page_store/xxh3.rs').read_text()
    t='mod xxh3 { use vstd::prelude::*; verus! {\n'+xxh3+'\n} }\nuse xxh3::hash128_with_seed;\n'+t
    texts['serialization']=t
    backend='use vstd::prelude::*;\nverus! {\nuse std::io; use core::panic;\nuse core::ops::Bound; use core::fmt::Debug;\nuse core::sync::atomic::{AtomicBool, Ordering};\n'
    backend+=item('src/error.rs','enum','BackendError')+'\n'+item('src/error.rs','enum','StorageError')+'\n'
    backend+='pub type Result<T = (), E = StorageError> = core::result::Result<T, E>;\n'
    backend+=item('src/db.rs','trait','StorageBackend')+'\n'
    backend+=item('src/tree_store/page_store/cached_file.rs','struct','CheckedBackend')+'\n'
    errors=(O/'src/error.rs').read_text();start=errors.index('impl From<io::Error> for StorageError')
    backend+=errors[start:span(errors,start)]+'\nimpl CheckedBackend {\n'
    for name in ['check_failure','sync_data']:
        backend+=item('src/tree_store/page_store/cached_file.rs','fn',name)+'\n'
    backend+='}\n}\n'
    texts['backend_sync']=backend
    inventory={}
    for name,text in texts.items():
        p=out/(name+'.rs');p.write_text(text)
        inventory[name]=dict(sha256=hashlib.sha256(text.encode()).hexdigest(),bytes=len(text.encode()))
    return inventory

# The harness adds proof/witness obligations; it does not replace an original algorithm.
DOMAIN_AND_WITNESS = r'''
verus! {
pub proof fn slot_domain(p: usize)
    requires p < 2,
    ensures (p ^ 1) < 2, p != (p ^ 1),
{
    assert((p ^ 1) < 2 && p != (p ^ 1)) by(bit_vector)
        requires p < 2;
}
pub fn publication_witness(prior_mode: bool, bytes: [u8; 128]) -> (r: DatabaseHeader)
    ensures r.primary_slot == 1, r.two_phase_commit == prior_mode,
        r.transaction_slots[0].corrupt_bytes == Some(bytes),
        r.transaction_slots[1].corrupt_bytes == None,
{
    let old_slot = TransactionHeader { version: 3, user_root: None, system_root: None,
        transaction_id: TransactionId(0), corrupt_bytes: Some(bytes) };
    let spare_slot = TransactionHeader { version: 3, user_root: None, system_root: None,
        transaction_id: TransactionId(0), corrupt_bytes: None };
    let mut header = DatabaseHeader { primary_slot: 0, recovery_required: true,
        two_phase_commit: prior_mode, primary_unflushed: false, page_size: 4096,
        region_header_pages: 0, region_max_data_pages: 1024, full_regions: 0,
        trailing_partial_region_pages: 1, transaction_slots: [old_slot, spare_slot] };
    proof { slot_domain(0); }
    header.write_secondary_slot(TransactionId(1), None, None);
    header.swap_primary_slot();
    proof { slot_domain(1); }
    let published = header.published_slot();
    assert(published.transaction_id == TransactionId(1));
    header.primary_unflushed = true;
    let published_old = header.published_slot();
    assert(published_old.corrupt_bytes == Some(bytes));
    header
}
}
'''
def supported():
    text=publication()+DOMAIN_AND_WITNESS
    # API-only visibility projection, for checked import controls. No body/type/field changes.
    text=text.replace('pub(crate)', 'pub')
    text=re.sub(r'(?m)^(    )(\w+): (.+),$', r'\1pub \2: \3,',text)
    return text

if __name__=='__main__':
    import sys
    print(json.dumps(generate(Path(sys.argv[1])),indent=2))
