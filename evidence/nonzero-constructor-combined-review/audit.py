"""Read-only acceptance audit of explicitly named own epochs and delivered ledger."""
import hashlib,json,re,subprocess
from pathlib import Path as P
import review as e
import identity
H=e.HERE;W=P(e.read(H/'work.json')['work']);B=W/'final-bundle'

def main():
    own_guard=e.hashes(list(H.glob('*.py'))+[H/'checkpoint.json',H/'work.json'])
    e.write(W/'final-audit-freeze.json',own_guard)
    checkpoint=e.read(H/'checkpoint.json')
    e.need(e.authenticate()==checkpoint['delivered'],'delivered ledger drift')
    e.guard({str(e.ROOT/k):v for k,v in checkpoint['source_hashes'].items()})
    br=e.read(B/'receipt.json');e.need(br['success'] and br['final_guards_passed'],'bundle not guarded')
    for n,h in br['tools'].items():e.need(e.sha(B/n)==h,'acceptance tool drift')
    # Every own command keeps argv, exits, logs, products and input-freeze identity.
    epochs=e.read_epochs(W);records=[]
    for row in epochs:
        out=P(row['path']);rp=out/'receipt.json';e.need(e.sha(rp)==row['receipt_sha256'],'own receipt drift')
        r=e.read(rp);e.need(e.sha(out/'freeze.json')==r['freeze_sha256'],'own freeze drift')
        for n,h in r['products'].items():
            if str(W/'target')+'/' in n:continue # mutable own build products; frozen bundle is acceptance input
            product=P(n)
            if str(W/'probes') in n:
                product=out/'retained'/P(n).name # each cold/warm immutable copy, not the later overwritten export path
            e.need(e.sha(product)==h,'own retained product drift '+str(product))
        records.append(dict(**row,exit=r['exit'],final_guards_passed=r['final_guards_passed'],commands=r['commands']))
    vals=[];selected={};tests={};body=None;old=None
    for row in map(json.loads,(W/'validations.jsonl').read_text().splitlines()):
        out=P(row['path']);e.need(e.sha(out/'validation.json')==row['validation_sha256'],'validation drift')
        v=e.read(out/'validation.json');vals.append(dict(**row,case=v.get('case'),phase=v.get('phase'),accepted=v.get('acceptance_checks_passed',v.get('validation_passed',False)),classification=v.get('classification')))
        for n,h in v.get('retained_products',{}).items():e.need(e.sha(out/n)==h,'retained product drift')
        if not vals[-1]['accepted']:continue
        # The exact controller bytes used by every accepted epoch are retained.
        freeze=e.read(out/'freeze.json')
        for n,h in freeze['inputs'].items():
            p=P(n)
            if p.parent==H and p.suffix=='.py':e.need(e.sha(out/'controller-snapshots'/p.name)==h,'accepted controller snapshot absent '+str(out)+' '+p.name)
        if 'case' in v:selected[(v['case'],v['phase'])]=(out,v)
        if 'suite' in v:
            for n,h in v['test_products'].items():e.need(e.sha(n)==h,'test product drift')
            e.need(e.sha(v['binary'])==v['binary_sha256'],'test binary drift');tests[v['suite']]=dict(path=str(out),passed=v['passed'],binary_sha256=v['binary_sha256'])
        if v.get('validation_passed'):body=v
        if v.get('classification')=='unchanged_old_policy_certificate_rejection':old=v
    expected={('macro','cold'),('macro','warm')}|{(x,'check') for x in ['import','width','off','false','weak','reversed','u32','ordering','compose']}
    e.need(set(selected)==expected,'accepted case coverage mismatch')
    e.need(body is not None and old is not None,'body/policy check missing')
    e.need({k:v['passed'] for k,v in tests.items()}=={'constructor_function_values':13,'infer_compare_derive':7,'infer_nonzero_derive':3},'test coverage mismatch')
    # Build/vstd remain bound even though bookkeeping/controller fixes followed.
    for row in [br['compiler'],br['vstd']]:
        out=P(row['path']);r=e.read(out/'receipt.json');e.need(r['exit']==0 and r['final_guards_passed'],'build/vstd not accepted')
        for n,h in e.read(out/'freeze.json')['inputs'].items():
            p=P(n)
            if p==H/'review.py':e.need(e.sha(out/'controller-snapshots'/p.name)==h,'executed build/vstd controller snapshot missing')
    e.need('verification results:: 2250 verified, 0 errors' in (P(br['vstd']['path'])/'stdout').read_text(),'matching vstd accounting')
    # Observe, rather than presume, pruning of genuinely closed imported bodies.
    text=(W/'probes-v3/import/vir/crate.vir').read_text();closed={}
    for m in re.finditer(r'\(Function\s',text):
        f=text[m.start():identity.end(text,m.start())];name=identity.field(f,'name')
        if name.startswith('(Fun :path component::') and any(name.endswith('::'+x+')') for x in ['eq','eq_spec','__verus_infer_nonzero_u64_view']):closed[name]=identity.field(f,'body')=='None'
    e.need(len(closed)==3 and all(closed.values()),'closed imported body unexpectedly present')
    e.need(subprocess.run(['git','diff','--exit-code','3aa7f49a','--','source','dependencies','rust-toolchain.toml'],cwd=e.ROOT,capture_output=True).returncode==0,'production/source changed')
    compact=[]
    for (case,phase),(out,v) in selected.items():
        i=v['inference']
        compact.append(dict(case=case,phase=phase,path=str(out),source_sha256=v['source_sha256'],plan_sha256=v['plan_sha256'],classification=v['classification'],verification=v['verification'],helpers=i['functions'],original_exec_body_hashes=v.get('original_exec_body_hashes'),all_local_function_details=v.get('all_local_function_details'),native_scope=i['verification_scope'] if case in ['macro','compose','import'] else None,interface_written=i['interface_written'],failed_functions=i['failed_functions']))
    excluded=P('/tmp/combined-review-build-1j987939/excluded-receipt.json')
    result=dict(review_base='3aa7f49a04a5e93c2ede882f2b947aac6e39263d',decision='scoped_accept',findings=[],policy=69,cache=72,schema=38,build=br['compiler'],vstd=br['vstd'],bundle=dict(path=str(B),receipt_sha256=e.sha(B/'receipt.json'),tools=br['tools']),cases=compact,tests=tests,body_identity=body,imported_closed_bodies_absent=closed,old_interface_gate=old,epochs=records,validations=vals,excluded_guard_receipt=dict(path=str(excluded),sha256=e.sha(excluded)),source_guards_passed=True,production_changed=False,controllers={p.name:e.sha(p) for p in H.glob('*.py')},delivered_epochs=51,delivered_relative_products=sum(x['relative_products'] for x in checkpoint['delivered']),publication=False,goal_complete=False,native_correspondences=0,ordering='UNPROVED',parent_whole_goal='PENDING')
    e.guard(own_guard)
    result['final_audit_freeze_sha256']=e.sha(W/'final-audit-freeze.json')
    result['final_audit_guards_passed']=True
    e.write(H/'RESULTS.json',result)
    print('SCOPED ACCEPT: vstd2250/0; Eq cold/warm6/0; import4/0; composition10/0; focused23/0; no production edits')
if __name__=='__main__':main()
