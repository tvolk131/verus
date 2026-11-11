"""Authenticate completed owned MIR diagnostics and seal compact evidence. No proofs run."""
from audit import audit
from run import *

def main():
    baseline=Path(sys.argv[1]);mir=Path(sys.argv[2])
    version=subprocess.check_output([TC/'bin/rustc','-vV'],env=environment(mir)).decode()
    need('commit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67' in version,'compiler/Miri source mismatch')
    expected=json.loads((HERE/'BASELINE.json').read_text())
    need(audit(baseline)==expected,'baseline no longer authentic')
    receipt=json.loads((mir/'receipt.json').read_text())
    for label,row in receipt['results'].items():
        d=mir/label
        need(row['exit']==0 and row['input_guards_passed'],'diagnostic failure')
        need(sha(d/'freeze.json')==row['freeze_sha256'],'diagnostic freeze changed')
        frozen=json.loads((d/'freeze.json').read_text())
        for p,h in frozen['inputs'].items():need(sha(p)==h,'diagnostic input drift '+p)
        for n in ['stdout','stderr']:need(sha(d/n)==row[n+'_sha256'],'diagnostic output drift')
    need(sha(mir/'inventory.json')==receipt['inventory_sha256'],'inventory drift')
    rows=json.loads((mir/'inventory.json').read_text())
    need(len(rows)==7 and not any(x['unavailable'] for x in rows),'incomplete diagnostic call closure')
    need(all('UNRESOLVED_CALL' not in x and 'NON_FNDEF_CALL' not in x for x in (mir/'inspect/stdout').read_text().splitlines()),'unresolved call')
    nonzero=next(x for x in rows if x['path']=='<std::num::NonZero<T> as std::cmp::Ord>::cmp')
    get=next(x for x in rows if x['path']=='std::num::NonZero::<T>::get')
    primitive=next(x for x in rows if 'for u64' in x['path'])
    need(len(nonzero['all_blocks'])==2 and len(nonzero['calls'])==1,'NonZero Ord shape')
    need(sum(x.count('(Transmute)') for x in nonzero['all_blocks'])==2,'get inlining shape')
    need(sum(x.count('(Transmute)') for x in get['all_blocks'])==1 and not get['calls'],'get shape')
    need(len(primitive['all_blocks'])==1 and not primitive['calls'] and 'Cmp(' in primitive['all_blocks'][0],'u64 Cmp shape')
    compact=[{k:v for k,v in x.items() if k!='all_blocks'}|dict(block_count=len(x['all_blocks'])) for x in rows]
    write(HERE/'MIR.json',dict(schema=1,private_root=str(mir),receipt_sha256=sha(mir/'receipt.json'),
       baseline_receipt_sha256=sha(baseline/'receipt.json'),
       diagnostic_only=True,compiler_matches_Miri_pin='58cf8f9c35b355199727627d07ae34a4e2846d67',rustc_version=version,
       inventory=compact,whole_mir_fingerprint='rustc whole Body StableHash with spans excluded; diagnostic, not a foreign certificate',
       earliest_required_correspondence='actual instantiated/inlined NonZero<u64>::get transmute fragment must refine its existing get boundary with result=n@; not established by recognizing a cast or hashing MIR',
       required_primitive='actual u64 Ord Cmp rvalue must correspond to ordinary typed numeric Ordering body and prove result contract',
       foreign_certificates_produced=0,ordering_accepted=False,publication=False,goal_complete=False))
    # Seal only own named products; no traversal of any external target or cache.
    roots=[baseline,mir]
    seal={str(root):{str(p.relative_to(root)):sha(p) for p in sorted(root.rglob('*')) if p.is_file()} for root in roots}
    write(HERE/'SEAL.json',dict(roots=seal,controller_inputs={str(p.relative_to(ROOT)):sha(p) for p in [HERE/'run.py',HERE/'audit.py',HERE/'mir_inventory.py',Path(__file__)]},
       immutable_bundle_receipt_sha256=BUNDLE_SHA,publication=False,goal_complete=False))
    print('Authenticated original baseline and seven-instance whole-MIR diagnostic; ordering remains unproved.')

if __name__=='__main__':main()
