"""Independent ordinary runner: no imported delivered acceptance predicates."""
import hashlib, json, os, pathlib, re, subprocess, sys, time
HERE=pathlib.Path(__file__).resolve().parent
ROOT=HERE.parents[1]
BUNDLE=pathlib.Path('/tmp/verus-infer-buffer-cycle-20261109/parent-stable-key-bundle-a93b774a')
BASELINE=pathlib.Path('/tmp/verus-infer-buffer-cycle-20261109/stable-key-validation-a93b774a/canonical/canonical-zgtzh85h/receipt.json')
def need(c,m):
    if not c: raise RuntimeError(m)
def sha(p): return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def save(p,x): p.write_text(json.dumps(x,indent=2,sort_keys=True)+'\n')
def snap(p):
    p=pathlib.Path(p); s=p.stat()
    return dict(sha256=sha(p),bytes=s.st_size,mtime_ns=s.st_mtime_ns,inode=s.st_ino)
def validate(data,stderr,code,name,positive,enabled,file):
    v=data['verification-results']; inf=data['verus-infer']; scope=inf['verification_scope']
    need(code==(0 if positive else 1),name+': exit')
    need(v==dict(success=positive,verified=3 if positive else 2,errors=0 if positive else 1,
        **{'encountered-error':not positive,'encountered-vir-error':False,'is-verifying-entire-crate':True}),name+': summary')
    need(inf['crate_success'] is positive and inf['functions']==[] and inf['requested_functions']==0
         and inf['generated_functions']==0 and inf['imported_inference_dependencies']==[],name+': helper identities')
    need(inf['assistance_explanations']['enabled'] is enabled,name+': goal mode')
    need(scope['native_collection_completed'] is True and scope['complete_selected_scope_verified'] is positive,name+': full collection')
    need(scope['checked_proof_bodies']==0 and scope['checked_exec_bodies']==(3 if positive else 0),name+': checked body counts')
    rows=scope['native_functions']; by={r['rust_function']:r for r in rows}
    need(len(rows)==len(by)==14,name+': exact Rust scope size')
    execs={'Permission::grants','<Permission as std::clone::Clone>::clone','<AccessKind as std::clone::Clone>::clone'}
    aux={'permission_access_table':'specification_definition_or_declaration',
        '_::<impl vstd::prelude::Structural for Permission>::assert_receiver_is_structural':'selected_item_without_translated_body',
        '_::<impl vstd::prelude::Structural for AccessKind>::assert_receiver_is_structural':'selected_item_without_translated_body'}
    outside={f'<{ty} as std::{tr}>::{fn}' for ty in ['Permission','AccessKind'] for tr,fn in [
        ('cmp::Eq','assert_fields_are_eq'),('cmp::PartialEq','eq'),('fmt::Debug','fmt'),('hash::Hash','hash')]}
    need(set(by)==execs|set(aux)|outside,name+': all identities')
    for ident,r in by.items():
        need(r['selected'] is (ident not in outside) and r['inference_candidate'] is False,name+': selection '+ident)
        need(r['ordinary_contract'] is (ident in execs),name+': contract '+ident)
        expected=aux.get(ident,('outside_selected_scope' if ident in outside else
            ('checked_exec_body' if positive else 'selected_body_not_certified_by_successful_full_run')))
        need(r['body_status']==expected,name+': body status '+ident)
        need(r['source'].startswith(str(file)+':'),name+': body source '+ident)
    failed=inf['failed_functions']
    need(failed==([] if positive else [name.replace('-','_')+'::Permission::grants']),name+': exact failed function; Clone bodies must stay healthy')
    bad=['panicked at','resource limit','timed out','precondition not satisfied','assertion failed',
         'arithmetic underflow','arithmetic overflow','invariant not satisfied','recommendation not met','error[E']
    need(not any(x in stderr for x in bad),name+': mixed/frontend/resource diagnostic')
    errors=re.findall(r'^error: (.+)$',stderr,re.M)
    need(errors==([] if positive else ['postcondition not satisfied','aborting due to 1 previous error']),name+': exact error inventory')
    if not positive:
        need(str(file)+':33:17' in stderr and 'failed this postcondition' in stderr,name+': postcondition location')
    return dict(name=name,goals_enabled=enabled,returncode=code,verification=v,selected_items=[by[k] for k in sorted(execs|set(aux))],
                outside_selected_scope=[by[k] for k in sorted(outside)],failed_functions=failed,inferred_functions=inf['functions'],
                trusted_models=inf.get('trusted_models'),status=inf['status'])
def main():
    frozen=json.loads((HERE/'freeze-own.json').read_bytes()); initial=json.loads((HERE/'pretrial-inventory.json').read_bytes())
    runner_freeze=json.loads((HERE/'runner-freeze.json').read_bytes())
    for p,h in runner_freeze['inputs'].items(): need(sha(p)==h,'runner pretrial identity '+p)
    paths=set(initial['files'])|set(runner_freeze['inputs'])|{str(HERE/'runner-freeze.json')}
    before={p:snap(p) for p in paths}
    for p,x in initial['files'].items(): need(before[p]['sha256']==x['sha256'],'initial input mismatch '+p)
    def guard():
        for p,s in before.items(): need(snap(p)==s,'input final guard '+p)
    accepted=json.loads(BASELINE.read_bytes()); need(accepted['success'] is True,'canonical baseline success')
    remap={x['own_path']:x['old_label'] for x in frozen['review_remapping']['entries']}
    for p,h in frozen['inputs'].items():
        need(sha(p)==h,'own freeze '+p)
        if pathlib.Path(p).parent==BUNDLE or p in remap and p.endswith(('infer_eval_support.py','infer_experiment.py')):
            # Only the already authenticated same committed repository bytes may
            # use an original receipt key. Never read that original path.
            key=remap.get(p,p)
            need(accepted['artifacts'][key]['sha256']==h,'canonical artifact binding '+p)
    out=HERE/'replay002'; out.mkdir(exist_ok=False); (out/'tmp').mkdir()
    receipt=dict(success=False,commands=[],runs=[],input_snapshots=before,native_correspondences=0,publication=False,goal_complete=False)
    try:
        component=(ROOT/'source/tools/research/native_miri_permission_grants/component.rs').read_text()
        cases=[('enabled-cold',component,True,True),('enabled-warm',component,True,True),('disabled',component,False,True),
            ('disabled-guard-removed',component.replace('self != Permission::Disabled','true'),True,False),
            ('read-inverted',component.replace('access == AccessKind::Read','access == AccessKind::Write'),True,False),
            ('readonly-guard-removed',component.replace('self != Permission::SharedReadOnly','true'),True,False)]
        for name,text,enabled,positive in cases:
            file=out/(name+'.rs'); file.write_text(text)
            need(sha(file)==initial['control_source_sha256'][name],name+': frozen control bytes')
            file_before=snap(file); cache=out/('enabled-cache' if enabled and positive else name+'-cache')
            env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',RUSTUP_AUTO_INSTALL='0',
                VERUS_Z3_PATH=str(BUNDLE/'z3'),VERUS_INFER_CACHE=str(cache),TMPDIR=str(out/'tmp'),PYTHONDONTWRITEBYTECODE='1')
            argv=[str(BUNDLE/'verus'),str(file),'--crate-type=lib','--no-cheating','--output-json']+(['--infer-goals'] if enabled else [])
            guard(); start=time.time()
            run=subprocess.run(argv,cwd=out,env=env,capture_output=True,text=True,timeout=120)
            for suffix,raw in [('stdout',run.stdout),('stderr',run.stderr)]: (out/(name+'.'+suffix)).write_text(raw)
            receipt['commands'].append(dict(argv=argv,cwd=str(out),environment={k:env[k] for k in ['PATH','RUSTUP_TOOLCHAIN','RUSTUP_AUTO_INSTALL','VERUS_Z3_PATH','VERUS_INFER_CACHE','TMPDIR','PYTHONDONTWRITEBYTECODE']},returncode=run.returncode,started=start,finished=time.time()))
            guard(); need(snap(file)==file_before,name+': source final guard')
            row=validate(json.loads(run.stdout),run.stderr,run.returncode,name,positive,enabled,file)
            row.update(source_sha256=sha(file),stdout_sha256=sha(out/(name+'.stdout')),stderr_sha256=sha(out/(name+'.stderr')))
            receipt['runs'].append(row)
            print(name,row['verification']['verified'],row['verification']['errors'],flush=True)
        guard(); receipt.update(success=True,all_final_guards_passed=True,semantic_mutations_rejected=3,manual_proof_bodies=[],inferred_helpers=[],actual_source_bridge_proved=False)
    except BaseException as e:
        receipt['failure']=dict(type=type(e).__name__,message=str(e)); raise
    finally: save(out/'receipt.json',receipt)
if __name__=='__main__': main()
