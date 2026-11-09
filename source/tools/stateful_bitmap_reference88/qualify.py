"""Freeze independent scope expectations and check implementation evidence, not review."""

import argparse
import json
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
from audit import audit, tokens
from run import CAMPAIGN, OUTPUT, REPO, artifact, digest, full_source_manifest, identity, record, source_manifest

FIXTURES = Path(__file__).resolve().parent


def require(condition, message):
    if not condition:
        raise ValueError(message)


def freeze():
    path = OUTPUT / 'frozen-plan.json'
    require(not path.exists(), 'acceptance plan already frozen')
    fixtures = {name: artifact(FIXTURES / name) for name in
                ['reference.rs', 'bridge.rs', 'import.rs', 'expected.json', 'controls.py', 'audit.py']}
    native = json.loads((OUTPUT / 'native.json').read_text())
    projection = audit(Path(native['bitmap']['path']).read_text(), (FIXTURES / 'reference.rs').read_text())
    require(projection['all_nine_bodies'] and projection['exact_struct'], 'native projection mismatch')
    record(OUTPUT / 'native-erasure-final.json', projection)
    record(path, dict(fixtures=fixtures, native=native,
                      compiler_sources=source_manifest(REPO), full_sources=full_source_manifest(REPO),
                      runtime=json.loads((OUTPUT / 'runtime-manifest.json').read_text()),
                      expectations=json.loads((FIXTURES / 'expected.json').read_text()),
                      controls=json.loads((OUTPUT / 'controls-final/frozen-expectations.json').read_text()),
                      planned_positive_runs=['bridge_final', 'reference_final', 'import_final'],
                      independent_review='Not performed here; parent must request NEW review'))
    print('Frozen source/fixture/runtime/scope plan ' + digest(path))


def positive(name, crate, expected, final_plan):
    directory = OUTPUT / 'runs' / name
    result = json.loads((directory / 'result.json').read_text())
    invocation = json.loads((directory / 'invocation.json').read_text())
    report = json.loads((directory / 'stdout').read_text())
    verification = report['verification-results']
    require(result['returncode'] == 0 and not result['timeout'], 'positive failed: ' + name)
    require(verification['success'] and verification['errors'] == 0
            and not verification['encountered-error'] and not verification['encountered-vir-error']
            and verification['is-verifying-entire-crate'], 'not complete backend success: ' + name)
    require(verification['verified'] == expected['verified'], 'unexpected proof total: ' + name)
    scope = report['verus-infer']['verification_scope']
    require(scope['native_collection_completed'] and scope['complete_selected_scope_verified'], 'scope failed')
    rows = {row['function']: row for row in scope['native_functions']}
    executable = expected.get('exec', expected.get('native_exec', []) + expected.get('authored_exec', []))
    proof = expected.get('proof', [])
    definitions = expected.get('spec', [])
    wanted = {crate + '::' + function: 'checked_exec_body' for function in executable}
    wanted.update({crate + '::' + function: 'checked_proof_body' for function in proof})
    wanted.update({crate + '::' + function: 'specification_definition_or_declaration' for function in definitions})
    require(set(rows) == set(wanted), 'extra/missing body scope: ' + name)
    for function, status in wanted.items():
        require(rows[function]['selected'] and rows[function]['body_status'] == status
                and not rows[function]['inference_candidate'], 'body status: ' + function)
        contract = status != 'specification_definition_or_declaration' and function != 'bridge88::extremes'
        require(rows[function]['ordinary_contract'] == contract,
                'contract classification: ' + function)
    require(scope['checked_exec_bodies'] == len(executable) and scope['checked_proof_bodies'] == len(proof),
            'body totals mismatch')
    inference = report['verus-infer']
    require(inference['requested_functions'] == 0 and inference['generated_functions'] == 0
            and not inference['functions'] and not inference['trusted_models'], 'unexpected Infer credit/trust')
    require(invocation['compiler_source_sha256'] == identity(final_plan['compiler_sources']), 'producer source mismatch')
    require('--no-cheating' in invocation['argv'] and '--compile' in invocation['argv']
            and '--output-json' in invocation['argv'] and '--num-threads' in invocation['argv']
            and 'VSTD_KIND' not in invocation['environment'], 'application command mismatch')
    source = OUTPUT / 'applications' / name / 'input.rs'
    fixture = 'bridge.rs' if name.startswith('bridge') else 'reference.rs' if name.startswith('reference') else 'import.rs'
    require(digest(source) == final_plan['fixtures'][fixture]['sha256'], 'frozen input mismatch')
    published = json.loads((OUTPUT / 'applications' / name / 'published.json').read_text())
    require(all(artifact(value['path']) == value for value in published.values()), 'published artifact drift')
    return dict(verification=verification, bodies=rows, published=published,
                reachable_functions=list(report['func-details']),
                inference_credit=False, ordinary_contracts=True,
                invocation=artifact(directory / 'invocation.json'), result=artifact(directory / 'result.json'),
                stdout=artifact(directory / 'stdout'), stderr=artifact(directory / 'stderr'))


def qualify():
    plan = json.loads((OUTPUT / 'frozen-plan.json').read_text())
    for name, binding in plan['fixtures'].items():
        require(artifact(FIXTURES / name) == binding, 'frozen fixture changed: ' + name)
    require(source_manifest(REPO) == plan['compiler_sources'], 'compiler source changed')
    frozen_sources = dict(plan['full_sources'])
    amendment_path = OUTPUT / 'qualification-metadata-correction.json'
    if amendment_path.exists():
        amendment = json.loads(amendment_path.read_text())
        own_path = 'source/tools/stateful_bitmap_reference88/qualify.py'
        require(amendment['path'] == own_path and amendment['old_sha256'] == frozen_sources[own_path]
                and amendment['new_sha256'] == digest(REPO / own_path), 'invalid tooling correction')
        frozen_sources[own_path] = amendment['new_sha256']
    require(full_source_manifest(REPO) == frozen_sources, 'full source changed after freeze')
    for name, binding in plan['runtime'].items():
        require(artifact(OUTPUT / 'runtime' / name) == binding, 'runtime changed')
    positives = {name: positive(name, crate, plan['expectations'][kind], plan)
                 for name, crate, kind in [('bridge_final', 'bridge88', 'bridge'),
                                           ('reference_final', 'reference88', 'reference'),
                                           ('import_final', 'client88', 'import')]}
    import_invocation = json.loads((OUTPUT / 'runs/import_final/invocation.json').read_text())
    dependency = positives['reference_final']['published']
    for flag, kind in [('--import', 'interface'), ('--extern', 'library')]:
        wanted = 'reference88=' + dependency[kind]['path']
        require(import_invocation['argv'].count(flag) == 1
                and import_invocation['argv'][import_invocation['argv'].index(flag) + 1] == wanted,
                'actual imported artifact mismatch')
    controls = {}
    for name, expected in plan['controls'].items():
        run_name = 'control_final_' + name
        directory = OUTPUT / 'runs' / run_name
        if not directory.exists():
            continue
        result = json.loads((directory / 'result.json').read_text())
        stderr = (directory / 'stderr').read_text()
        text = (directory / 'stdout').read_text()
        report = json.loads(text) if text.strip() else {}
        verification = report.get('verification-results', {})
        intended = expected['diagnostic'] in stderr
        failed = result['returncode'] != 0 and not result['timeout']
        frontend = 'not supported' in expected['diagnostic']
        semantic = bool(failed and intended and not frontend and verification.get('errors', 0) > 0
                        and not verification.get('encountered-vir-error', True))
        source = OUTPUT / 'applications' / run_name / 'input.rs'
        require(digest(source) == expected['sha256'], 'control input mismatch')
        controls[name] = dict(expected=expected, intended_diagnostic=intended, failed=failed,
                              semantic_kill_credit=semantic, frontend_boundary=frontend,
                              verification=verification, result=artifact(directory / 'result.json'),
                              stderr=artifact(directory / 'stderr'))
    required_controls = set(plan['controls']) - {'wrong_method'}
    require(required_controls <= set(controls), 'missing controls')
    require(all(controls[name]['failed'] and controls[name]['intended_diagnostic']
                for name in required_controls), 'a discriminating control failed its expectation')
    result = dict(verdict='Implementation reference success, conditional on parent qualification and NEW independent review',
                  positives=positives, controls=controls, new_trust=plan['expectations']['trust'],
                  acceptance_rows=dict(B1='implementation pass', P1='implementation pass', P2='implementation pass',
                                       P3='implementation pass; ordinary real import, no inference certification',
                                       N1='implementation pass; frontend boundaries get zero semantic credit',
                                       A1='authored proof only; no new automation claim', R1='implementation pass; see compatibility disclosures',
                                       R2='OPEN; parent and new independent reviewer own this'),
                  budget=json.loads((OUTPUT / 'launch-counts.json').read_text()),
                  frozen_plan=artifact(OUTPUT / 'frozen-plan.json'),
                  inference_success=False, overall_goal_complete=False)
    record(OUTPUT / 'implementation-qualification.json', result)
    print(result['verdict'])


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('operation', choices=['freeze', 'qualify'])
    if parser.parse_args().operation == 'freeze':
        freeze()
    else:
        qualify()
