#!/usr/bin/env python3
"""Summarize a successful frozen replay; retain failed development identities."""
import argparse, hashlib, json, re, sys
from pathlib import Path
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import require

def main():
    p = argparse.ArgumentParser()
    p.add_argument('trial', type=Path)
    a = p.parse_args()
    trial = a.trial.resolve()
    receipt = trial/'receipt.json'
    raw = receipt.read_bytes()
    r = json.loads(raw)
    require(r['success'] is True and r['model_proofs_kernel_checked'] is True, 'unsuccessful replay')
    paths = [receipt, trial/'freeze.json', *map(Path,r['inputs']), *map(Path,r['proof_artifacts']),
             *trial.glob('*.stdout'), *trial.glob('*.stderr')]
    guard = ArtifactGuard(paths)
    def retained(path): return guard.initial[str(Path(path).resolve())]['sha256']
    require(retained(receipt) == hashlib.sha256(raw).hexdigest(), 'receipt changed during load')
    for name, info in {**r['inputs'], **r['proof_artifacts']}.items():
        require(retained(name) == info['sha256'], 'artifact no longer matches replay '+name)
    freeze = json.loads((trial/'freeze.json').read_text())
    source = (HERE/'candidates/reader_na.v').read_text()
    require(source.count('iApply (wp_read_na_guarded_cells') == 2, 'missing genuine Na primitive applications')
    require(not re.search(r'\b(?:ScOrd|wp_read_sc\w*|shared_key_read|key_reader_read)\b', source), 'SC WP used in Na reader')
    loop = (HERE/'candidates/xor_na.v').read_text()
    require('wp_write_na ⊤ p _ _ _ with "PadHead"' in loop, 'wrong store credit interface')
    require('WriteCredit' not in loop, 'extra input store credit')
    attempts = []
    for d in sorted(trial.parent.glob('model-ordinary-shared-xor-trial-*')):
        if not d.is_dir() or not (d/'receipt.json').exists(): continue
        attempt = json.loads((d/'receipt.json').read_text())
        attempts.append(dict(path=str(d),receipt_sha256=digest(d/'receipt.json'),
            freeze_sha256=digest(d/'freeze.json'),success=attempt['success'],
            semantic_evidence=attempt['success'] and attempt['model_proofs_kernel_checked'],
            failure=attempt.get('failure'),
            commands=[{k:c[k] for k in ['label','exit_code','seconds']} for c in attempt['commands']],
            logs={f.name:digest(f) for f in sorted(d.iterdir()) if f.suffix in ['.stdout','.stderr']}))
    result = dict(schema=1, scope='MODEL ordinary shared reader/session and same-algorithm XOR; fixed pair adapter',
        receipt=dict(path=str(receipt),sha256=retained(receipt),success=True),
        freeze=dict(path=str((HERE/'final-freeze.json').relative_to(ROOT)),sha256=retained(trial/'freeze.json')),
        normal_coqc=True,full_recursive_coqchk=True,kernel_exclusions=[],
        constant_count=r['constant_count'],declarations=r['declaration_inventory'],
        existing_instance_registrations=r['existing_instance_registrations'],
        constructor_inventory=r['constructor_inventory'],program_obligations=r['program_obligations'],
        assumptions=r['assumptions'],kernel_axioms=r['kernel_axioms'],
        inventory_helpers=r['inventory_helper_theorems'],helper_assumptions=r['helper_assumptions'],
        kernel_declarations=r['all_kernel_declarations'],
        proof_sources={Path(n).name:i['sha256'] for n,i in r['proof_artifacts'].items() if n.endswith('.v')},
        proof_objects={Path(n).name:i['sha256'] for n,i in r['proof_artifacts'].items() if n.endswith('.vo')},
        printed_term_section_hashes=r['proof_terms'],printed_terms_may_be_depth_limited=True,
        logs={f.name:retained(f) for f in sorted(trial.iterdir()) if f.suffix in ['.stdout','.stderr']},
        dependency_pins_sha256=digest(HERE/'dependency-pins.json'),
        algorithm_audit=r['algorithm_audit'],
        na_reader_from_actual_indexed_lifetime_and_physical_guards=True,sc_black_box_wp_not_used=True,
        ordinary_read_runtime_phases=2,arbitrary_concurrent_read_count_primitive=True,
        raw_session_prepare_credits='g+1',prepared_session_extra_prepare_credits=0,
        per_read_credits='3*(g+1)',loop_credits='count*3*(g+1)',pad_input_store_credits=0,
        same_key_base_advancing_start=True,generic_start_count=True,
        exact_transform_prefix_and_xor_min=True,unchanged_suffix_length_and_byte_range=True,
        same_Cap_R_Frame_and_R_G_restoration=True,separate_Cap_input_at_zero=True,
        zero_count_no_reader_no_lookup_no_memory_access=True,
        all_finite_pad_and_key_lengths=True,pad_128_empty_and_above_128_keys=True,
        exclusive_key_vector_required=False,shared_aliases_preserved=True,
        adapter_consumes_old_sc_R_G_then_opens_fresh_na_session=True,
        adapter_returns_old_sc_token=False,adapter_prepared_Cap_no_extra_credit=True,
        same_Outer_TypeG_Pad_Lpad_and_outside_Frame=True,
        adequacy_internal_setup_credits='2+k',new_client_credit_premises=False,
        initialized_writer_observer_composition_proved_here=False,
        baseline=dict(commit='115a618dd35ff8337112cf2081a7f5391d3973d5',
            sources=523,objects=284,
            GMP_manifest_sha256='332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37',
            readiness_and_successful_durable_receipt_authenticated=True),
        all_receipt_source_object_tool_identities_guarded_through_final_check=True,
        all_development_attempts_retained=True,new_axioms_primitives_trusted_summaries_unsafe_modes=0,
        original_Rust_and_frozen_SC_unchanged=True,domain_reduction=False,
        unrestricted_concurrent_SC_Na_equivalence_claimed=False,
        actual_native_correspondences=0,minimal_native_simulations_open=44,
        native_phase_obligations_open=3,native_writer_effects_open=101,
        canonical='6/0 inherited, not rerun',HMAC='1/1 inherited, not rerun',
        Rust_input_provenance_ancestor_history_retag_C_F_proved=False,
        total_termination_claimed=False,publication=False,goal_complete=False,
        independent_review='none; no subdelegates as instructed',
        commands=[{k:c[k] for k in ['label','exit_code','seconds']} for c in r['commands']],
        replay='python3 -B source/tools/research/model_ordinary_shared_xor/check.py --freeze source/tools/research/model_ordinary_shared_xor/final-freeze.json --output /tmp/model-ordinary-shared-xor-replay-UNUSED')
    guard.check()
    (HERE/'final-freeze.json').write_bytes((trial/'freeze.json').read_bytes())
    require(digest(HERE/'final-freeze.json')==result['freeze']['sha256'],'final freeze copy differs')
    (HERE/'results.json').write_text(json.dumps(result,indent=2)+'\n')
    (HERE/'attempts.json').write_text(json.dumps(dict(schema=1,attempts=attempts,
        failed_attempts_are_semantic_evidence=False,
        notes=['Initial git commit needed command-local author identity; no global config edit.',
               'Attempted to stop suspected slow trial008 adapter, but no matching compiler existed: all modules had already compiled; nothing was signaled or changed.']),indent=2)+'\n')
    guard.check()
    print(json.dumps(dict(receipt_sha256=result['receipt']['sha256'],constants=result['constant_count'],success=True)))
if __name__ == '__main__': main()
