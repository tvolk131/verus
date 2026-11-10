#!/usr/bin/env python3
"""Retain compact review evidence; raw receipts and objects stay private."""
import argparse
import hashlib
import json
import sys
from pathlib import Path
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import require

def read(p):
    return json.loads(p.read_text())

def main():
    p = argparse.ArgumentParser()
    p.add_argument('success',type=Path)
    p.add_argument('--attempt',type=Path,action='append',default=[])
    a = p.parse_args()
    trial = a.success.resolve()
    r = read(trial/'receipt.json')
    require(r['success'] and r['final_identity_guards_passed'], 'incomplete review')
    fresh = read(trial/'replay/receipt.json')
    paths = [trial/'receipt.json',trial/'review-freeze.json',trial/'replay/receipt.json',
        trial/'replay/freeze.json', *map(Path,r['inputs']),
        *map(Path,fresh['proof_artifacts']), *[trial/'replay'/n for n in r['logs']]]
    guard = ArtifactGuard(paths)
    for label,info in {**r['inputs'],**fresh['proof_artifacts']}.items():
        require(guard.initial[str(Path(label).resolve())]['sha256'] == info['sha256'], 'changed review artifact '+label)
    require(digest(trial/'replay/receipt.json') == r['fresh_receipt_sha256'], 'fresh receipt pin')
    require(digest(trial/'review-freeze.json') == r['review_freeze_sha256'], 'review freeze pin')
    for n,h in r['logs'].items(): require(digest(trial/'replay'/n) == h, 'fresh log pin')
    subject = ROOT/'source/tools/research/model_ordinary_shared_xor'
    trusted = read(ROOT/'source/tools/research/mutable_buffer_goal/foundation-baseline.json')
    bindings = read(ROOT/'source/tools/research/native_semantic_bridge/baseline-bindings.json')
    ready_path = Path('/tmp/verus-infer-buffer-cycle-20261109/rocq-toolchain/baseline-ready.json')
    ready = read(ready_path)
    attempts = []
    for path in [*a.attempt,trial]:
        ar = read(path/'receipt.json')
        cr = read(path/'replay/receipt.json')
        attempts.append(dict(path=str(path),receipt_sha256=digest(path/'receipt.json'),
            pretrial_freeze_sha256=digest(path/'review-freeze.json'),
            success=ar['success'],failure=ar.get('failure'),
            component_receipt_sha256=digest(path/'replay/receipt.json'),
            component_success=cr['success'],component_kernel_checked=cr['model_proofs_kernel_checked'],
            semantic_negative_evidence=False))
    def pinned(path): return fresh['inputs'][str(path.resolve())]['sha256']
    out = dict(schema=1, finding='No correctness defect found in exact reviewed MODEL component',
        subject_commit=r['subject_commit'],early_interface_commit='990ad032',corrected_controller_commit='271f97c8',
        manual_statement_and_proof_audit_completed=True,
        review_receipt=dict(path=str(trial/'receipt.json'),sha256=digest(trial/'receipt.json'),success=True),
        fresh_receipt=dict(path=str(trial/'replay/receipt.json'),sha256=r['fresh_receipt_sha256'],success=True),
        pretrial_review_freeze_sha256=r['review_freeze_sha256'],
        unchanged_final_freeze_sha256=r['unchanged_final_freeze_sha256'],
        worker_authentication=r['worker_authentication'],
        normal_coqc=True,full_recursive_coqchk=True,kernel_exclusions=[],
        constant_count=21,declarations=r['declaration_inventory'],
        assumptions=r['assumptions'],kernel_axioms=r['kernel_axioms'],
        inventory_helpers=fresh['inventory_helper_theorems'],helper_assumptions=r['helper_assumptions'],
        all_105_kernel_declarations=r['kernel_declarations'],
        constructor_inventory=[],program_obligations=[],existing_instance_registrations=r['existing_instance_registrations'],
        fresh_sources={n:h for n,h in r['fresh_products'].items() if n.endswith('.v')},
        fresh_objects={n:h for n,h in r['fresh_products'].items() if n.endswith('.vo')},
        object_hashes_match_worker=r['object_hashes_match_worker'],
        logs=r['logs'],guard_count=r['original_guard_count'],fresh_input_count=len(fresh['inputs']),
        final_identity_guards_passed=True,
        baseline=dict(commit=ready['source_commit'],sources=523,objects=284,
            readiness_sha256=pinned(ready_path),durable_receipt_sha256=pinned(Path(ready['receipt'])),
            authenticated_foundation_pins_sha256=pinned(ROOT/'source/tools/research/mutable_buffer_goal/foundation-baseline.json'),
            compiled_object_map_sha256=bindings['compiled_module_map_sha256'],
            tools=bindings['tools'],GMP_manifest_sha256=pinned(Path('/tmp/verus-infer-buffer-cycle-20261109/rocq-toolchain/gmp-private/manifest.json'))),
        source_audit=read(HERE/'source-audit.json'),
        evidence_hashes={n:digest(HERE/n) for n in ['INTERFACE.md','review.py','audit.py','source-audit.json','summarize.py','REPORT.md']},
        fresh_commands=[dict(label=c['label'],exit_code=c['exit_code'],seconds=c['seconds'],
            argv_sha256=hashlib.sha256(json.dumps(c['argv'],separators=(',',':')).encode()).hexdigest()) for c in fresh['commands']],
        review_attempts=attempts,
        actual_native_correspondences=0,minimal_native_simulations_open=44,native_phase_obligations_open=3,native_writer_effects_open=101,
        canonical='6/0 inherited, not rerun',HMAC='1/1 inherited, not rerun',publication=False,goal_complete=False,
        native_provenance_retag_C_F_claim=False,termination_claim=False,concurrent_program_equivalence_claim=False,
        initialized_writer_final_observer_composition_claim=False,
        replay='python3 -B evidence/ordinary-shared-xor-review/review.py --output /tmp/ordinary-shared-xor-review-NEW')
    require(out['baseline']['readiness_sha256'] == trusted['readiness']['sha256'], 'readiness authentication')
    require(out['baseline']['durable_receipt_sha256'] == trusted['receipt']['sha256'], 'durable authentication')
    require(out['baseline']['GMP_manifest_sha256'] == '332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37','GMP pin')
    guard.check()
    (HERE/'attempts.json').write_text(json.dumps(dict(attempts=attempts,failures_are_semantic_negatives=False),indent=2)+'\n')
    (HERE/'results.json').write_text(json.dumps(out,indent=2)+'\n')
    guard.check()
    print(json.dumps(dict(receipt_sha256=out['review_receipt']['sha256'],guard_count=out['guard_count'],objects_match=out['object_hashes_match_worker'])))
if __name__ == '__main__': main()
