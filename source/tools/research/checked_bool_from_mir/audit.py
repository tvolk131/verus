"""Audit only this lane's named completed epochs; never rerun historical drivers."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from functools import lru_cache

import controller as c

sha = lru_cache(maxsize=None)(c.sha)
ORIGIN = c.HERE.parent / 'item_tag_constructor_composition'
sys.path.insert(0, str(ORIGIN))
from inventory import inventory, signature
from source import fidelity


def need(ok, message):
    c.need(ok, message)


def last(rows, label):
    found = [r for r in rows if r['label'] == label]
    need(bool(found), 'missing epoch ' + label)
    return Path(found[-1]['path'])


def classify(row, r, d):
    if not r['final_guards_passed']:
        return 'excluded_controller_input_drift'
    if row['label'] == 'build-vir':
        return 'diagnostic_library_build' if r['exit'] == 0 else 'setup_offline_resolution_failure'
    if row['label'] == 'compile-inspector':
        return 'diagnostic_inspector_build' if r['exit'] == 0 else 'diagnostic_compilation_failure'
    if row['label'] == 'typed-inspection':
        return 'structural_diagnostic_no_body_proof'
    if row['label'] == 'actual':
        return 'true_property_library_contract_gap'
    if row['label'] == 'full-help':
        return 'true_property_original_protector_proof_gap'
    if row['label'] in {'mirror', 'full-helpers'}:
        return 'independent_diagnostic_support_positive'
    return 'compiler_identity'


def ordinary(d, case):
    r = json.loads((d / 'receipt.json').read_text())
    p = r['plan']; v = inventory(d); vr = v['verification']
    need(r['final_guards_passed'] is True, 'ordinary guards')
    need(vr['is-verifying-entire-crate'] and not vr['encountered-vir-error'], 'ordinary full scope')
    need([vr['verified'], vr['errors']] == p['expected']['query_result'], 'ordinary query outcome')
    failed = case in {'actual', 'full-help'}
    need(r['exit'] == int(failed) and vr['success'] is not failed, 'ordinary exit/success')
    need(v['inference_interface_written'] is not failed, 'strict export outcome')
    need((d / 'component.vir').exists() is not failed, 'private export product')
    if case.startswith('full'):
        need(signature(d) == p['expected']['original_scope'], 'exact frozen body/helper/native inventory')
        need(fidelity((d / 'component.rs').read_text()) == p['expected']['fidelity'], 'original source fidelity')
        helpers = v['helpers']
        need(len(helpers) == 1 and helpers[0]['function'] == 'component::BorTag::eq', 'only original Eq helper')
        h = helpers[0]
        need(h['mode'] == 'exact' and h['contract']['requires'] == 'true'
             and h['required_checks_passed'] is True, 'exact healthy Eq')
        need(set(h['proofs']) == {'soundness', 'maximality', 'exactness'}, 'all mandatory Eq checks')
        need(all(q['required'] is True and q['status'] == 'proved' for q in h['proofs'].values()), 'Eq obligations proved')
        need(v['failed_functions'] == (['component::Item::new'] if failed else []), 'singleton constructor gap')
        if failed:
            err = (d / 'stderr').read_text()
            need('debug_assert!(new.protected() == protected)' in err, 'original protector gap')
            need('debug_assert!(new.tag() == tag)' not in err
                 and 'debug_assert!(new.perm() == perm)' not in err, 'other debug checks healthy')
        else:
            need(v['native_scope']['complete_selected_scope_verified'] is True, 'support selected scope')
        for n in ['component::BorTag::ctor%', 'component::Item::ctor%']:
            need(n in v['local_function_details'] and v['ordinary_original_functions'][n]['body_present'],
                 'ordinary generated constructor body present')
        v['source_adapters'] = p['expected']['fidelity']
        v['manual_support'] = dict(proof_functions=['packed_fields_raw'],
            proof_blocks=3 if failed else 2, bit_vector_assertions=7,
            quantified_packing_assertions=1 if failed else 0,
            local_packing_lemma_calls=1 if failed else 0,
            same_helper_source=True, no_bool_conversion_lemma=True)
    else:
        need(v['helpers'] == [], 'bool probes have no inferred helper')
        need(v['failed_functions'] == (['component::all_bool'] if failed else []), 'bool singleton')
        bodies = {n for n, f in v['ordinary_original_functions'].items() if f['mode'] == 'Exec' and f['body_present']}
        need(bodies == {'component::all_bool'}, 'exact bool ordinary body scope')
    # Preserve every selected/excluded native identity and ordinary generated
    # body identity, not only the number of queries or an empty helper set.
    v.update(path=str(d), receipt_sha256=sha(d / 'receipt.json'),
             source_sha256=sha(d / 'component.rs'), foreign_body_registered=False,
             checked_foreign_instance=False, publication=False)
    return v


def main():
    need(sys.flags.dont_write_bytecode, 'run with Python -B')
    baseline = c.authenticate()
    rows = [json.loads(s) for s in (c.HERE / 'attempts.jsonl').read_text().splitlines()]
    ledger = []
    rejected_guards = []
    for row in rows:
        d = Path(row['path'])
        need(sha(d / 'receipt.json') == row['receipt_sha256'], 'receipt hash ' + str(d))
        r = json.loads((d / 'receipt.json').read_text())
        need(row['guards'] == r['final_guards_passed'] and row['exit'] == r['exit'], 'ledger consistency')
        for p, h in r['products'].items():
            need(sha(d / p) == h, 'retained product drift ' + p)
        for p, h in r.get('external_build_products', {}).items():
            need(sha(p) == h, 'external owned build product drift')
        if r['final_guards_passed']:
            for p, h in r['inputs'].items():
                p = Path(p)
                snapshot = d / ('controller-' + p.name)
                checked = snapshot if p.parent == c.HERE and snapshot.exists() else p
                need(sha(checked) == h, 'frozen input drift ' + str(p))
        else:
            rejected_guards.append(str(d))
        ledger.append({**row, 'classification': classify(row, r, d),
                       'stdout_sha256': sha(d / 'stdout'), 'stderr_sha256': sha(d / 'stderr'),
                       'freeze_sha256': sha(d / 'freeze.json')})
    need(rejected_guards == ['/tmp/checked-bool-build-vir-481unrgi'], 'unexpected guard rejection')
    build = last(rows, 'build-vir')
    br = json.loads((build / 'receipt.json').read_text())
    need(br['exit'] == 0 and br['final_guards_passed'] and br.get('external_build_products'), 'accepted private VIR build')
    need(br['environment']['CARGO_BUILD_JOBS'] == '2'
         and br['environment']['CARGO_NET_OFFLINE'] == 'true', 'private offline two-job build')
    for key in ['CARGO_HOME', 'CARGO_TARGET_DIR', 'CARGO_BUILD_BUILD_DIR']:
        need(br['environment'][key].startswith(br['plan']['private_root'] + '/'), 'private build confinement')
    version = last(rows, 'compiler-version')
    need('58cf8f9c35b355199727627d07ae34a4e2846d67' in (version / 'stdout').read_text(), 'exact compiler commit')
    diag = last(rows, 'typed-inspection')
    dr = json.loads((diag / 'receipt.json').read_text()); out = (diag / 'stdout').read_text()
    need(dr['exit'] == 0 and dr['final_guards_passed'], 'typed probe success')
    roots = {}
    for part in out.split('ROOT ')[1:]:
        name = part.splitlines()[0]
        roots[name] = [s for s in part.splitlines()[1:] if s.startswith((
            'REJECT ', 'OBSERVATION ', 'TYPED_CONTROLS ', 'VIR_STRUCTURAL_CONTROLS ', 'BLOCKED '))]
    need(set(roots) == {'probe_actual', 'probe_other_output', 'probe_generic',
                       'probe_user_trait', 'probe_function_pointer'}, 'exact inspected root inventory')
    need('TYPED_CONTROLS 12' in roots['probe_actual']
         and 'VIR_STRUCTURAL_CONTROLS 8 diagnostic_only_not_proofs' in roots['probe_actual'], 'typed diagnostic controls')
    for name, reason in [('probe_other_output', 'wrong_trait_input_or_Self'),
                         ('probe_generic', 'unresolved_substitutions'),
                         ('probe_user_trait', 'callee_not_core_From_method'),
                         ('probe_function_pointer', 'not_direct_FnDef_call')]:
        need(roots[name] == ['REJECT ' + reason], 'identity/call rejection ' + name)
    need('CONTRACT_AVAILABLE false' in out and 'FOREIGN_BRIDGE_PROVED false' in out
         and 'PUBLICATION false' in out, 'no available contract')
    mir = out.split('RAW_FOREIGN_MIR ', 1)[1].split('\nOBSERVATION ', 1)[0]
    vir = out.split('RAW_ORDINARY_VIR ', 1)[1].split('\nBLOCKED ', 1)[0]
    proof = {k: ordinary(last(rows, k), k) for k in ['actual', 'mirror', 'full-help', 'full-helpers']}
    changed = subprocess.check_output(['git', 'diff', '--name-only', '098a8be6'], cwd=c.ROOT).decode().splitlines()
    changed += subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard'], cwd=c.ROOT).decode().splitlines()
    need(all(p.startswith('source/tools/research/checked_bool_from_mir/') for p in changed), 'owned research-only edits')
    result = dict(decision='INCOMPLETE_no_registered_checked_foreign_contract',
        early_interface_commit='ae3cd734', base='098a8be66e00734a010bf525ff832a81157621ea',
        engine='3aa7f49a', baseline_receipt_sha256=c.BUNDLE_RECEIPT, tools=baseline['tools'],
        parent_tag_receipt_sha256='ba75225dcf12c21a8f7c17fcfdef493d1083aa3ed2c865d74740b0a66de476aa',
        versions=dict(schema=38, policy=69, cache=72, baseline_vstd='2250/0', version_changes=False),
        diagnostic=dict(path=str(diag), receipt_sha256=sha(diag / 'receipt.json'),
            exact_roots=roots, actual_foreign_mir_debug_sha256=hashlib.sha256(mir.encode()).hexdigest(),
            ordinary_VIR_debug_sha256=hashlib.sha256(vir.encode()).hexdigest(),
            ordinary_VIR_constructed=True, ordinary_pipeline_run=False,
            identity=[s for s in out.splitlines() if s.startswith('IDENTITY ')],
            operation=['Copy bool local 1', 'IntToInt cast to u64', 'assign local 0', 'Return'],
            local_count=2, argument_count=1, block_count=1, statement_count=1,
            both_locals_need_drop=False, diagnostic_control_count=20,
            proof_token=False, contract_available=False),
        private_VIR_library_build=dict(path=str(build), receipt_sha256=sha(build / 'receipt.json'),
            modified_engine_built=False, vstd_rebuilt=False, external_product_hash_count=len(br['external_build_products'])),
        ordinary_trials=proof, attempt_ledger=ledger,
        all_accepted_evidence_guards_passed=True, excluded_guard_failures=rejected_guards,
        unreceipted_setup_failures=2, unreceipted_artifact_locator_failure=1,
        production_changes=False, new_std_models=False,
        cold_warm_checked_instance_tests='NOT_AVAILABLE', import_publication_tests='NOT_AVAILABLE',
        healthy_checked_instance_false_result_control='NOT_AVAILABLE',
        full_Item_new='36/1_UNPROVED_original_protector_assertion',
        publication=False, native_correspondences=0, goal_complete=False,
        whole_goal='PENDING', HMAC='1/1_UNPROVED')
    c.write(c.HERE / 'RESULTS.json', result)
    print(json.dumps(dict(attempts=len(rows), decision=result['decision'],
        typed_diagnostics=20, identity_rejections=4, actual='0/1', full='36/1', support='36/0',
        mirror='1/0', accepted_evidence_guards=True, excluded_guard_failures=len(rejected_guards))))


if __name__ == '__main__':
    main()
