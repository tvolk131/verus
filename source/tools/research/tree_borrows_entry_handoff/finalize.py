"""Collect compact receipts, full own inventories and explicit unchanged limits."""
import hashlib, json, pathlib, re

OWN = pathlib.Path(__file__).resolve().parent
BASE = OWN.parent / 'tree_borrows_checked_foundation'
OUT = pathlib.Path('/tmp/tree-borrows-entry-handoff-20261110')
DELIVERED = pathlib.Path('/tmp/tree-borrows-checked-foundation-20261110')

def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def write(n, x):
    (OWN / n).write_text(json.dumps(x, indent=2) + '\n')

if sha(OWN / 'Entry.v') != sha(OUT / 'final-01-source/Entry.v'):
    raise RuntimeError('compiled source no longer matches delivered source')
inventory = json.loads((OWN / 'INVENTORY.json').read_text())
if not inventory['all_individually_closed'] or len(inventory['all_own_declarations']) != 17:
    raise RuntimeError('incomplete own declaration audit')

commands = []
trials = []
for p in sorted(OUT.glob('*/receipt.json')):
    m = json.loads(p.read_text()); pre = p.parent / 'pretrial.json'
    if sha(pre) != m['pretrial_sha256'] or sha(p.parent / 'stdout.log') != m['log_sha256']:
        raise RuntimeError('receipt binding failure: ' + str(p))
    m.update(name=p.parent.name, receipt_path=str(p), receipt_sha256=sha(p), pretrial=json.loads(pre.read_text()))
    commands.append(m)
    failure = (p.parent / 'stdout.log').read_text() if m['exit_code'] else None
    trials.append({'name': p.parent.name, 'exit_code': m['exit_code'],
                   'classification': 'NORMAL_CHECK_SUCCESS' if not m['exit_code'] else 'PROOF_DEVELOPMENT_ELABORATION_FAILURE',
                   'failure_excerpt': failure[:2500] if failure else None,
                   'semantic_counterexample': False, 'receipt_sha256': sha(p), 'log_sha256': m['log_sha256']})
for required in ['final-01', 'kernel-final-01', 'inventory-final-01', 'authentication-final-01']:
    if not any(m['name'] == required and m['exit_code'] == 0 for m in commands):
        raise RuntimeError('missing final successful receipt: ' + required)
write('COMMANDS.json', {'private_root': str(OUT), 'supervised_commands': commands,
      'reproduction': ['python3 -B check.py auth auth-fresh-name.json',
                       'python3 -B check.py compile fresh-trial-name',
                       'python3 -B check.py kernel fresh-trial-name fresh-kernel-name'],
      'inventory_collection': ['python3 -B inventory.py prepare',
                               'python3 -B check.py query final-01 inventory-final-01 inventory.v',
                               'python3 -B inventory.py collect'],
      'receipt_policy': 'Fresh names only; no completed baseline prepare/build; raw output is private.'})
write('TRIALS.json', {'trials': trials, 'setup_events': [{
      'classification': 'GIT_AUTHOR_IDENTITY_SETUP_FAILURE',
      'argv': ['git', 'commit', '-m', 'research: freeze structural Tree Borrows entry handoff interface'],
      'exit_code': 128, 'message': 'Author identity unknown; unable to auto-detect email address',
      'provenance': 'Retrospective transcription of initial tool output; no pretrial claimed for this Git setup event.',
      'resolution': 'Per-command -c user.name=Codex -c user.email=codex@openai.com; no global/shared config mutation.'}],
      'interpretation': 'Elaboration/setup failures are not semantic evidence. Only completed normal checking, assumption queries and recursive kernel checking support the stated result.'})

auth = json.loads((OUT / 'auth-final.json').read_text())
write('AUTHENTICATION.json', {'initial': json.loads((OUT / 'auth.json').read_text()), 'final': auth,
      'final_authentication_receipt_sha256': sha(OUT / 'authentication-final-01/receipt.json'),
      'historical_driver_executed': False, 'completed_baseline_rebuilt_or_modified': False})
write('SOURCES.json', {'own_final_sources': {n: sha(OWN / n) for n in
      ['Entry.v', 'check.py', 'inventory.py', 'finalize.py', 'PLAN.md', 'REPORT.md']},
      'final_compiled_source_sha256': sha(OUT / 'final-01-source/Entry.v'),
      'own_object_sha256': sha(OUT / 'final-01-source/Entry.vo'),
      'baseline_ready_sha256': sha(DELIVERED / 'baseline-ready.json'),
      'immutable_baseline_commit': '04b51409',
      'baseline_manifests': {n: sha(BASE / n) for n in
      ['TOOLS.json', 'upstream-source.json', 'model-pretrial.json', 'ASSUMPTIONS.json',
       'INVENTORY.json', 'PUBLIC_TYPES.json', 'SOURCE_AUDIT.json']},
      'delivered_maps': {n: sha(DELIVERED / n) for n in
      ['installed-pretrial.json', 'model-objects.json', 'loaded-objects.json', 'model-pretrial.json']},
      'source_path_policy': 'Own checkout and named immutable delivered artifacts only. Historical checkout paths in receipt text are labels.'})

kernel = (OUT / 'kernel-final-01/stdout.log').read_text()
write('results.json', {
      'classification': 'CLOSED_STRUCTURAL_ACTUAL_CORE_ENTRY_PROGRESS_AND_FRAMES',
      'normal_compilation': True, 'full_recursive_kernel_check': True,
      'own_declarations': 17, 'all_own_declarations_individually_closed': True,
      'recursive_libraries': len(inventory['recursive_checked_libraries']),
      'recursive_kernel_constants': inventory['recursive_kernel_constant_count'],
      'actual_loaded_objects': json.loads((OWN / 'OBJECTS.json').read_text())['count'],
      'kernel_context_verbatim': kernel[kernel.index('CONTEXT SUMMARY'):],
      'full_delivered_baseline_context': ['functional_extensionality_dep', 'classic', 'constructive_indefinite_description'],
      'public_entry_premises': ['state_wf s', 'strs s !! blk = Some tr', 'tree_contains parent tr',
                                'cid ∈ scs s', '(0 < sz)%nat', 'entry_eligible tr (scs s) parent (off, sz)'],
      'public_entry_conclusions': ['derived fresh child creation', 'derived READ after creation', 'actual RetagIS bor_step',
                                   'final state_wf', 'exact outside-block frame'],
      'additional_derived_frame': 'Original nodes retain protector/default permission and lazy permission at all offsets outside the interval, within the same block.',
      'positive_sizes': 'all positive nat, including 10 and 128',
      'native_source_validity': False, 'lawful_Rust_structural_eligibility_coverage': False,
      'FnEntry_setup_argument_installation_source_mapping': False,
      'empty_dangling_and_current_Miri_correspondence': False,
      'source_safety_for_original_Rust': False, 'canonical': '6/0 unchanged; not rerun',
      'HMAC': '1/1 UNPROVED; no domain change, including key lengths 0 and >128; prefix and suffix remain required',
      'native_correspondences': 0, 'open_obligations': {'minimal': 44, 'phase': 3, 'writer_effects': 101},
      'publication': False, 'goal_complete': False,
      'independent_acceptance': 'Fresh combined independent acceptance remains required by integration lead',
      'evidence_files': {p.name: sha(p) for p in sorted(OWN.iterdir()) if p.is_file() and p.name != 'results.json'}})
print('compact final evidence collected')
