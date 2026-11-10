#!/usr/bin/env python3
"""Compact evidence from finished private receipts; no acceptance by exit alone."""
import hashlib, json, re, os
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=Path(os.environ.get('STBOR_ENTRY_OUTPUT_ROOT', '/tmp/stbor-entry-handoff-20261110'))
BASE=Path('/tmp/stbor-checked-foundation-20261110')
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def load(p): return json.loads(p.read_text())
def save(p,x): p.write_text(json.dumps(x,sort_keys=True,indent=2)+'\n')
def ref(p): return dict(path=str(p),sha256=sha(p))
def require_check(condition, message):
    if not condition:
        raise RuntimeError(message)

trials=[ROOT/('trial%03d'%i) for i in range(1,12)] + [ROOT/'accepted013',ROOT/'accepted025']
accepted={}
for name,count in [('accepted013',13),('accepted025',25)]:
    d=ROOT/name; a=load(d/'acceptance.json')
    require_check(a['declarations']==count and a['all_individually_closed'], 'Required proof/artifact check failed at original line 16')
    require_check(load(d/'kernel.json')['exit_status']==0, 'Required proof/artifact check failed at original line 17')
    for file,h in load(d/'products.json').items(): require_check(sha(d/file)==h, 'Required proof/artifact check failed at original line 18')
    if name=='accepted025':
        require_check(load(d/'controller-exit.json')['exit_status']==0, 'Required proof/artifact check failed at original line 20')
        freeze=load(d/'pretrial-freeze.json')
        require_check(sha(d/'controller.py')==freeze['controller_sha256'], 'Required proof/artifact check failed at original line 22')
    accepted[name]=ref(d/'acceptance.json')
commands=[]; attempts=[]
for d in trials:
    receipts=[]
    for p in sorted(d.glob('*.json')):
        x=load(p)
        if 'argv' not in x: continue
        require_check(x['state']=='finished', 'Required proof/artifact check failed at original line 30')
        require_check(sha(x['log'])==x['log_sha256'], 'Required proof/artifact check failed at original line 31')
        receipts.append(p.name)
        commands.append(dict(trial=d.name,receipt=ref(p),argv=x['argv'],cwd=x['cwd'],
            environment=x['environment'],exit_status=x['exit_status'],
            started=x['started'],finished=x['finished'],
            pid=x['pid'],pgid=x['pgid'],startticks=x['startticks'],
            runner_sha256=x['runner_sha256'],log=ref(Path(x['log'])),sources=x['inputs']))
    entry=dict(trial=d.name,receipts=receipts)
    if (d/'pretrial-freeze.json').exists(): entry['pretrial_freeze']=ref(d/'pretrial-freeze.json')
    if (d/'controller.py').exists(): entry['controller_snapshot']=ref(d/'controller.py')
    if (d/'foundation-guard.json').exists(): entry['foundation_guard']=ref(d/'foundation-guard.json')
    if d.name=='trial007':
        entry.update(classification='setup_scan_aborted_before_formal_command',setup=ref(d/'setup-aborted.json'))
        require_check(not receipts, 'Required proof/artifact check failed at original line 44')
    else:
        failed=[x for x in commands if x['trial']==d.name and x['exit_status']!=0]
        entry['classification']='proof_elaboration_failure_not_semantic_negative' if failed else 'normal_success_not_independent_acceptance'
        if failed:
            text=Path(failed[-1]['log']['path']).read_text()
            entry['diagnostic']=text[text.find('File "'):].strip()
    entry['source_files']={p.name:ref(p) for p in d.glob('*.v')}
    attempts.append(entry)
final=ROOT/'accepted025'
inv=load(final/'declaration-inventory.json')
compact={}
for name,x in inv.items():
    require_check(x['assumptions_text'].strip()=='Closed under the global context', 'Required proof/artifact check failed at original line 57')
    compact[name]={k:v for k,v in x.items() if k not in ('type_text','assumptions_text')}
    compact[name]['assumptions']='Closed under the global context'
for f in ['candidate.v','composition.v']:
    require_check(sha(HERE/f)==sha(final/f), 'Required proof/artifact check failed at original line 61')
map=load(final/'object-dependency-map.json')
objects={name:{k:x[k] for k in ('object','object_sha256','source_sha256')}
         for name,x in map['modules'].items()}
a=load(final/'acceptance.json')
require_check(set(objects)==set(a['checked_modules']), 'Required proof/artifact check failed at original line 66')
for name,x in objects.items(): require_check(sha(x['object'])==x['object_sha256'], 'Required proof/artifact check failed at original line 67')
save(HERE/'COMMANDS.json',dict(commands=commands,attempts=attempts,
    command_count=len(commands),raw_logs_private=True,
    setup_note='Initial Git author setup failure exists in tool transcript only; command-local identity used thereafter.'))
save(HERE/'INVENTORY.json',dict(declarations=compact,complete_private_inventory=ref(final/'declaration-inventory.json'),
     definitions=3,proved_lemmas=22,own_declarations=25))
save(HERE/'OBJECTS.json',dict(modules=objects,complete_private_dependency_map=ref(final/'object-dependency-map.json'),
    complete_loaded_object_inventory=ref(final/'loaded-objects.json'),
    compiled_own_objects=a['compiled_objects'],kernel_receipt=ref(final/'kernel.json')))
context=load(BASE/'completed-build-receipt.json')['kernel_context_summary']
status=dict(schema=2,state='checked_conditional_historical_model_composition',
    own_declarations=25,definitions=3,proofs_accepted=22,all_own_individually_closed=True,
    original_13_normally_compiled_and_recursively_checked=True,
    generic_positive_n=True,instances=[10,128],zero_dangling_scope='historical_model_stuttering_only',
    model_kernel_checked=True,actual_source_instance=False,independent_combined_acceptance=False,
    native_correspondences=0,minimal_simulations_open=44,phase_simulations_open=3,writer_effects_open=101,
    inherited_varint='6/0',inherited_hmac='1/1 UNPROVED',publication=False,goal_complete=False,
    baseline_ready=ref(BASE/'baseline-ready.json'),baseline_completed=ref(BASE/'completed-build-receipt.json'),
    accepted_runs=accepted,private_root=str(ROOT),
    candidate004=dict(source_sha256='434ed59ef957c98daf4a0c6825d63d2203d56fc6dc666de003122e4d882b14e1',
      original_source_git='04b51409:source/tools/research/stbor_entry_handoff/candidate.v',
      chronological_freeze=ref(HERE/'candidate004-freeze.json'),chronological_status=ref(HERE/'candidate004-status.json')),
    final_sources={f:sha(HERE/f) for f in ('candidate.v','composition.v')},
    final_checker_controller=ref(final/'controller.py'),
    own_closure_modules=len(objects),full_initial_kernel_modules=len(load(ROOT/'accepted013/acceptance.json')['checked_modules']),
    own_kernel_context=a['kernel_context'],inherited_full_foundation_context=context,
    limitations='MODEL valid_block + WF + live call; no all-lawful Rust entry/operation/C-F bridge; source obligations in REPORT.md; no semantic negative controls')
save(HERE/'status.json',status)
print('Evidence summarized:',len(commands),'commands,',len(objects),'modules,',len(inv),'declarations')
