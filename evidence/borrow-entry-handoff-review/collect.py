"""Collect independent finished receipts and exact imported-trust differences."""
import importlib.util, pathlib, json, sys, subprocess, re
sys.dont_write_bytecode=True
P=pathlib.Path
spec=importlib.util.spec_from_file_location('review',P(__file__).with_name('replay.py'))
r=importlib.util.module_from_spec(spec); spec.loader.exec_module(r)
e=r.E; out=r.OUT
st=r.load(out/'stbor/fresh/acceptance.json'); sti=r.load(e/'stbor-INVENTORY.json'); tbi=r.load(e/'tree-INVENTORY.json')
oldst=r.load(r.ST/'acceptance.json'); oldti=r.load(r.SRC/'tree_borrows_entry_handoff/INVENTORY.json')
r.need(st['checked_modules']==oldst['checked_modules']==sti['checked_libraries'],'Stacked library coverage')
r.need(tbi['checked_libraries']==oldti['recursive_checked_libraries'],'Tree library coverage')
r.need(tbi['kernel_constant_count']==51983,'Tree constant coverage')
stmap=r.load(out/'stbor/fresh/object-dependency-map.json')['modules']; oldmap=r.load(r.ST/'object-dependency-map.json')['modules']
for n,x in stmap.items():
 r.need(x['object_sha256']==oldmap[n]['object_sha256'],'Stacked object mismatch '+n)
 r.need(x['recorded_compiled_dependencies']==oldmap[n]['recorded_compiled_dependencies'],'Stacked dependency digest mismatch '+n)
 r.need(x['recorded_imports']==oldmap[n]['recorded_imports'],'Stacked import mismatch '+n)
# Normalize just the physical root labels for the newly compiled Tree object.
expected=r.load(r.SRC/'tree_borrows_entry_handoff/OBJECTS.json')['all_actual_recursive_loaded_objects']; actual=r.load(out/'tree/source/independent-loaded-objects.json')
normalized={}
for label,x in actual.items():
 p=P(label)
 key='own:final-01-source/Entry.vo' if p==out/'tree/source/Entry.vo' else 'baseline:'+str(p.relative_to(r.BT))
 normalized[key]=x['sha256']
r.need(set(normalized)==set(expected),'Tree loaded set')
for k,h in normalized.items():
 if k.startswith('baseline:'): r.need(h==expected[k],'Tree upstream object mismatch '+k)
# Original compiler products are compared as data; fresh failures classified separately.
native=r.load(out/'compiler/receipt.json'); r.need(native['success'] and native['all_final_guards_passed'],'compiler acceptance')
r.need(len(native['commands'])==9,'compiler count')
for c in native['commands']:
 r.need(c['exit_code']==(1 if c['label'] in ['pinned-unpin','pinned-unsafe-unpin','cell-freeze'] else 0),'compiler exit '+c['label'])
commands=[]
for p in sorted(out.rglob('*.json')):
 x=r.load(p)
 if isinstance(x,dict) and 'argv' in x and ('exit_code' in x or 'exit_status' in x):
  log=p.with_suffix('.log')
  if 'log_sha256' in x and log.exists(): r.verify(log,x['log_sha256'])
  commands.append({'receipt':str(p),'sha256':r.sha(p),'argv':x['argv'],'exit':x.get('exit_code',x.get('exit_status')),'log':str(log) if log.exists() else x.get('log'),'log_sha256':x.get('log_sha256')})
for c in native['commands']:
 commands.append(dict(c,receipt=str(out/'compiler/receipt.json'),stdout_sha256=r.sha(out/'compiler'/(c['label']+'.stdout')),stderr_sha256=r.sha(out/'compiler'/(c['label']+'.stderr'))))
r.save(e/'COMMANDS.json',{'reviewer_commands':commands,'replay':['python3 -B evidence/borrow-entry-handoff-review/replay.py prepare','python3 -B evidence/borrow-entry-handoff-review/replay.py stbor','python3 -B evidence/borrow-entry-handoff-review/replay.py tree','python3 -B evidence/borrow-entry-handoff-review/replay.py compiler','python3 -B evidence/borrow-entry-handoff-review/compare.py','python3 -B evidence/borrow-entry-handoff-review/collect.py'],'root_is_fresh_and_must_not_be_reused':str(out),'collect_controller_sha256':r.sha(__file__)})
# All fresh private files: sources, objects, tools summaries, queries, logs and receipts.
files={str(p.relative_to(out)):r.sha(p) for p in sorted(out.rglob('*')) if p.is_file()}
r.save(e/'PRIVATE_ARTIFACTS.json',{'root':str(out),'files':files,'all_completed':True})
# Record exact proof source equality against immutable Git objects, never historical paths.
history={}
for commit,directory in [('89494dce','stbor_entry_handoff'),('15e3c268','stbor_entry_handoff'),('995809af','tree_borrows_entry_handoff'),('5f4a04c5','tree_borrows_entry_handoff'),('7f415a57','native_entry_type_binding')]:
 names=[p for p in (r.SRC/directory).iterdir() if p.is_file()]
 checks={}
 for p in names:
  rel=str(p.relative_to(r.REPO)); data=subprocess.check_output(['git','show',commit+':'+rel],cwd=r.REPO)
  checks[rel]=r.hashlib.sha256(data).hexdigest(); r.need(checks[rel]==r.sha(p),'committed source mismatch '+rel)
 history[commit]={'full_commit':subprocess.check_output(['git','rev-parse',commit],cwd=r.REPO,text=True).strip(),'files':checks}
oldcandidate=subprocess.check_output(['git','show','04b51409:source/tools/research/stbor_entry_handoff/candidate.v'],cwd=r.REPO)
r.need(r.hashlib.sha256(oldcandidate).hexdigest()=='434ed59ef957c98daf4a0c6825d63d2203d56fc6dc666de003122e4d882b14e1','historical candidate hash')
r.save(e/'GIT_AUTHENTICATION.json',{'deliveries':history,'candidate004_at_04b51409_sha256':r.hashlib.sha256(oldcandidate).hexdigest(),'review_base':r.load(e/'scope-freeze.json')['review_base']})
fullst=r.load(r.SB/'completed-build-receipt.json'); fulltb=r.load(r.BT/'baseline-ready.json')
r.verify('/tmp/stbor-entry-handoff-20261110/accepted013/acceptance.json','fd30af654d8de1cc44f2a5b4ae24d1edc674d04f97dcf94d6ce0d8983b70b4bf')
allst=set(r.load('/tmp/stbor-entry-handoff-20261110/accepted013/acceptance.json')['checked_modules']); closure=set(sti['checked_libraries'])
loadedtb=set(tbi['checked_libraries']); baseline_tb=set(oldti['recursive_checked_libraries'])
# Exact wider Tree set comes from the completed baseline kernel log bound by ready.
bk=r.load(fulltb['receipts']['model-kernel']['receipt']) if 'model-kernel' in fulltb['receipts'] else None
if bk is None:
 candidates=[x for n,x in fulltb['receipts'].items() if 'kernel' in n]
 r.need(len(candidates)==1,'baseline kernel receipt selector'); bk=r.load(candidates[0]['receipt'])
blog=P(bk['log']).read_text(); alltb=set(re.findall(r'Checking library: (\S+)',blog))
trust={'stbor':{'own_all_25':'Closed under the global context','actual_282_module_context':sti['kernel_context'],'full_518_foundation_context':fullst['kernel_context_summary'],'actual_closure_added_relative_to_foundation':['handoff.candidate','handoff.composition'],'actual_282_minus_historical_519':sorted(closure-allst),'historical_519_minus_actual_282':sorted(allst-closure),'full_foundation_not_rebuilt':True},'tree':{'own_all_17':'Closed under the global context','actual_458_module_context':tbi['kernel_context'],'full_507_foundation_context':fulltb['kernel_context_verbatim'],'actual_458_minus_full_507':sorted(loadedtb-alltb),'full_507_minus_actual_458':sorted(alltb-loadedtb),'full_foundation_not_rebuilt':True}}
r.save(e/'ASSUMPTIONS.json',trust)
receipt_paths={'stacked_acceptance':out/'stbor/fresh/acceptance.json','stacked_driver':out/'stbor/driver-accept.json','tree_compile':out/'tree/source/compile.json','tree_kernel':out/'tree/source/kernel.json','tree_inventory':out/'tree/source/independent-inventory.json','tree_auth_before':out/'tree/authentication-before.json','tree_auth_after':out/'tree/authentication-after.json','compiler':out/'compiler/receipt.json'}
r.save(e/'results.json',{'classification':'INDEPENDENT_CONDITIONAL_MODEL_AND_COMPILER_CHECKPOINTS_ONLY','review_base':r.load(e/'scope-freeze.json')['review_base'],'scope_commit':'eeca1cdc','pretrial_commit':'69249063','own_sources_normally_compiled':True,'full_recursive_kernels_without_exclusions':True,'stacked':{'own_closed':25,'definitions':3,'lemmas':22,'libraries':282,'own_objects_byte_identical':True,'all_loaded_object_hashes_and_dependency_maps_match':True},'tree':{'own_closed':17,'kernel_constants_including_record_projections':15,'record_and_constructor':2,'libraries':458,'recursive_constants':51983,'all_upstream_loaded_object_hashes_match':True,'all_17_printed_bodies_identical_to_delivery':True,'object_byte_identical':False,'physical_source_path_embedded':True},'compiler':{'commands':9,'positive_exit_zero':6,'ordinary_E0277_exit_one':3,'query_data_identical_to_delivery':True,'all_final_guards_passed':True,'no_assert_as_acceptance':True},'receipts':{n:{'path':str(p),'sha256':r.sha(p)} for n,p in receipt_paths.items()},'findings':[{'severity':'P3','location':'source/tools/research/tree_borrows_entry_handoff/MILESTONE.json:6','finding':'successful_compilation_receipts includes interface-02 exit_code 1 at line 23; this is elaboration history, not successful evidence; final-01/kernel-final-01/inventory-final-01 and fresh checks pass.','mathematical_acceptance_blocked':False}],'fresh_setup_or_elaboration_failures':[],'semantic_negatives':[],'compiler_rejections_are_trait_controls_not_application_semantic_negatives':True,'inherited_varint':'6/0','inherited_hmac':'1/1 UNPROVED','native_correspondences':0,'open_obligations':{'minimal':44,'phase':3,'writer_effects':101},'publication':False,'goal_complete':False,'fresh_whole_goal_acceptance_performed':False,'baseline_rebuilt':False,'production_changed':False})
print('Collected independent acceptance, exact trust differences, command/log/object maps')
