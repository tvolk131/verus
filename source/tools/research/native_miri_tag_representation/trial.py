"""Private, hash-frozen ordinary epochs. A met frontend expectation is not a proof."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1]))
from infer_eval_support import ArtifactGuard, digest, support_artifacts, tool_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require

BUNDLE = Path('/tmp/verus-infer-buffer-cycle-20261109/parent-stable-key-bundle-a93b774a')
ACCEPTED = Path('/tmp/verus-infer-buffer-cycle-20261109/stable-key-validation-a93b774a/canonical/canonical-zgtzh85h/receipt.json')
ACCEPTED_SHA = '2dda5d93603528fe2623f4e37bc7cbaa48a537e3184c653a1abe1a4028f4ed3a'
TOOL_NAMES = {'verus','rust_verify','vstd.vir','libvstd.rlib','libverus_builtin.rlib',
              'libverus_builtin_macros.so','libverus_state_machines_macros.so','z3'}
RUSTC = Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin/rustc')

def authenticate():
    require(digest(ACCEPTED) == ACCEPTED_SHA, 'accepted receipt drift')
    accepted = json.loads(ACCEPTED.read_bytes())
    require(accepted['success'] is True, 'canonical run did not succeed')
    files = tool_artifacts(BUNDLE/'verus', BUNDLE/'z3')
    require({p.name for p in files if p.parent == BUNDLE} == TOOL_NAMES, 'not all eight tools')
    mappings = {}
    for p in files:
        if p.parent == BUNDLE:
            key = str(p)
        else:
            keys = [k for k in accepted['artifacts'] if k.endswith('/source/tools/'+p.name)]
            require(len(keys) == 1, 'ambiguous historical support key')
            key = keys[0]  # key ONLY: never open historical checkout paths
        require(digest(p) == accepted['artifacts'][key]['sha256'], 'runtime/support drift: '+str(p))
        mappings[str(p)] = key
    return files, mappings

def check_data(data, rc, stderr, expected, names, negative):
    vr = data['verification-results']
    inf = data['verus-infer']
    require(inf['interface_written'] is False, 'unexpected publication')
    require(inf['functions'] == [] and inf['requested_functions'] == 0
            and inf['generated_functions'] == 0, 'unexpected inferred helper')
    require(all(not v for k,v in inf.items() if k.endswith('_assistance') and isinstance(v,list)), 'unexpected assistance')
    if expected == 'frontend':
        require(rc != 0 and (vr.get('encountered-error') is True or vr.get('encountered-vir-error') is True), 'not a frontend rejection')
        require(vr.get('success',False) is False, 'frontend reported success')
        return
    require(vr.get('is-verifying-entire-crate') is True, 'partial crate')
    scope = inf['verification_scope']
    require(scope['native_collection_completed'] is True, 'incomplete collection')
    bodies = scope['native_functions']
    selected = [b for b in bodies if b['selected'] and b['body_status'] not in
                ['specification_definition_or_declaration','selected_item_without_translated_body']]
    require({b['rust_function'] for b in selected} == set(names)
            and len(selected) == len(names), 'selected body inventory mismatch')
    require(not any(b['body_status'] in ['external_specification','unverified_body_with_contract','interface_without_checked_body'] for b in selected), 'unchecked local body')
    if expected == 'positive':
        require(rc == 0 and vr['success'] is True and vr['errors'] == 0, 'ordinary proof failed')
        require(inf['crate_success'] is True and scope['complete_selected_scope_verified'] is True, 'incomplete certified scope')
        require(all(b['body_status'] in ['checked_exec_body','checked_proof_body'] for b in selected), 'uncertified body')
        require(inf.get('failed_functions',[]) == [], 'unexpected failed query')
    elif expected == 'semantic':
        require(rc != 0 and vr['success'] is False and vr['errors'] == 1, 'not a single semantic rejection')
        require('postcondition not satisfied' in stderr or 'assertion failed' in stderr, 'missing semantic diagnostic')
        require(not any(s in stderr for s in ['precondition not satisfied','possible arithmetic','invariant not satisfied','recommendation not met','error[E']), 'unhealthy helper/control')
        failed = [b['function'] for b in selected if b['rust_function'] == negative]
        require(len(failed) == 1 and inf['failed_functions'] == failed, 'failed-function singleton differs')
    else:
        require(rc != 0 and vr.get('success') is False and vr.get('errors',0) > 0, 'not an ordinary proof failure')

def main():
    p=argparse.ArgumentParser()
    p.add_argument('source',type=Path); p.add_argument('--name',required=True)
    p.add_argument('--expect',choices=['frontend','positive','proof','semantic','native'],required=True)
    p.add_argument('--compile',action='store_true'); p.add_argument('--native-run',action='store_true')
    p.add_argument('--contains'); p.add_argument('--off',action='store_true')
    p.add_argument('--inventory',type=Path); p.add_argument('--negative-function')
    a=p.parse_args()
    out=Path(tempfile.mkdtemp(prefix='native-miri-tag-'+a.name+'-',dir='/tmp'))
    out.chmod(0o700)
    rec=dict(schema=1,name=a.name,expected=a.expect,output=str(out),commands=[],
             native_correspondences=0,publication=False,goal_complete=False,
             expectation_met_is_not_source_acceptance=True)
    with replay_receipt(out,rec):
        tools,mapping=authenticate();rec['canonical_support_key_mapping']=mapping
        src=out/'component.rs';src.write_bytes(a.source.read_bytes())
        names=json.loads(a.inventory.read_bytes()) if a.inventory else []
        for f in HERE.glob('*.py'):
            (out/f.name).write_bytes(f.read_bytes())
        pins=HERE.parent/'native_miri_policy_pin/pins.json'
        selection=HERE/'selection.json'
        frozen=json.loads(selection.read_bytes())
        originals=[Path(e['path']) for e in frozen['sources'].values()]
        for e in frozen['sources'].values():
            require(digest(e['path'])==e['sha256'], 'primary-source identity drift')
        inputs=tools+[ACCEPTED,RUSTC,Path('/home/dev/.cargo/bin/rustc').resolve(),
                      Path(sys.executable).resolve(),pins,selection,a.source,src]
        inputs+=list(HERE.glob('*.py'))+list(HERE.glob('*.rs'))+list(HERE.glob('*.md'))+originals
        inputs+=list(out.glob('*.py'))
        if a.inventory:inputs.append(a.inventory)
        guard=ArtifactGuard(inputs)
        env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',
                 VERUS_Z3_PATH=str(BUNDLE/'z3'),VERUS_INFER_CACHE=str(out/'cache'),
                 TMPDIR=str(out),XDG_CACHE_HOME=str(out/'cache'),CARGO_TARGET_DIR=str(out/'target'))
        if a.expect=='native':
            argv=[str(RUSTC),str(src),'--edition=2021']
            argv+=(['-o',str(out/'native-test')] if a.native_run else ['--crate-type=lib','-o',str(out/'literal.rlib')])
        else:
            argv=[str(BUNDLE/'verus'),str(src),'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','20']
            if not a.off:argv+=['--infer-goals']
            if a.compile:argv+=['--compile','-o',str(out/'checked.rlib')]
        manifest=dict(inputs=guard.initial,argv=argv,environment={k:env[k] for k in ['RUSTUP_TOOLCHAIN','VERUS_Z3_PATH','VERUS_INFER_CACHE','TMPDIR','XDG_CACHE_HOME','CARGO_TARGET_DIR']},
                      source_sha256=digest(src),before_first_trial=True,expected=a.expect,assistance=not a.off,
                      body_inventory=names,negative_function=a.negative_function)
        (out/'input-freeze.json').write_text(json.dumps(manifest,indent=2)+'\n')
        rec['freeze_sha256']=digest(out/'input-freeze.json');rec['source_sha256']=digest(src)
        freeze_guard=ArtifactGuard([out/'input-freeze.json'])
        rec['artifacts']=guard.initial
        run=LoggedCommands(out,out,rec['commands'],timeout=120,guard=guard)
        try:
            rc=run('check',argv,env);rec['terminal_exit']=rc
            stdout=(out/'check.stdout').read_text();stderr=(out/'check.stderr').read_text()
            rec['logs']={n:digest(out/n) for n in ['check.stdout','check.stderr']}
            low=stderr.lower()
            require(not any(s in low for s in ['panicked at','resource limit','timed out','internal compiler error']), 'resource/compiler failure is not semantic evidence')
            if a.contains:require(a.contains in stderr,'missing expected diagnostic')
            if a.expect=='native':
                require(rc==0 and (out/('native-test' if a.native_run else 'literal.rlib')).is_file(),'native typecheck failed')
                if a.native_run:
                    product_guard=ArtifactGuard([out/'native-test'])
                    rec['native_product_before_run']=product_guard.initial
                    code=run('native',[str(out/'native-test')],env)
                    require(code==0,'native comparison failed')
                    product_guard.check()
                rec['classification']='native_compilation_supporting_only'
            else:
                data=json.loads(stdout)
                rec['verification']=data['verification-results'];rec['inference']=data['verus-infer']
                vr=data['verification-results']
                rec['observed_classification']=('frontend_failure' if vr.get('encountered-vir-error') or not vr.get('is-verifying-entire-crate') else 'ordinary_success' if vr.get('success') else 'ordinary_proof_failure')
                check_data(data,rc,stderr,a.expect,names,a.negative_function)
                rec['classification']= {'frontend':'frontend_blocker_no_semantic_evidence','positive':'ordinary_component_proof','semantic':'semantic_control','proof':'ordinary_proof_failure'}[a.expect]
        finally:
            guard.check();freeze_guard.check()
            rec['all_final_guards_passed']=True
            rec['products']={str(f.relative_to(out)):dict(sha256=digest(f),bytes=f.stat().st_size)
                             for f in out.rglob('*') if f.is_file() and f.name not in ['receipt.json','receipt.json.tmp']}
    print(json.dumps(dict(output=str(out),receipt_sha256=digest(out/'receipt.json'),
                         classification=rec['classification'],verification=rec.get('verification'))))

if __name__=='__main__':main()
