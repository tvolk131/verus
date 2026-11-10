#!/usr/bin/env python3
"""Audit private raw receipts and emit a compact source-connected manifest."""
import argparse
import json
from pathlib import Path
import sys

from experiment import (HERE, ROOT, BASE, BUNDLE, NAMES, OBSERVER, check_freeze, digest,
    diagnostic, git, require, require_exact, require_function_coverage)
from infer_experiment import require_publication, validate_goal_run


def main():
    p=argparse.ArgumentParser()
    p.add_argument('--raw',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args()
    f=check_freeze()
    summary=dict(schema=1, base=BASE, freeze_sha256=digest(HERE/'freeze.json'),
        controller_sha256=digest(HERE/'experiment.py'), auditor_sha256=digest(Path(__file__)),
        goal_complete=False, shared_work_on_both_full_properties=False,
        publication=False, native_correspondences=0, native_effect_obligations_open=101,
        raw_root=str(a.raw), baseline_artifacts=f['baseline_artifacts'],
        baseline_manifest_sha256=f['baseline_manifest_sha256'],
        variants={}, trials=[], diagnostic_profiles={}, commands=[],
        fresh_parent_and_independent_acceptance_required=True)
    setup=json.loads((a.raw/'setup/receipt.json').read_text())
    require(setup['success'], 'setup did not pass')
    summary['setup_receipt_sha256']=digest(a.raw/'setup/receipt.json')
    rejected=a.raw/'observer-off-build-setup-rejected'
    if rejected.exists():
        r=json.loads((rejected/'receipt.json').read_text())
        require(not r['success'] and r['production_restored'], 'bad retained setup receipt')
        summary['retained_setup_failure']=dict(receipt=str(rejected/'receipt.json'),
            receipt_sha256=digest(rejected/'receipt.json'), failure=r['failure'],
            classification='controller vstd output-format gate; not application proof evidence',
            commands=r['commands'], logs={p.name:digest(p) for p in rejected.iterdir()
                if p.suffix in ('.stdout','.stderr')})
    for variant in ('observer-off','capacity-off'):
        path=a.raw/(variant+'-build')
        b=json.loads((path/'receipt.json').read_text())
        require(b['success'] and b['production_restored'],'invalid build')
        changed=[p for p,h in b['source_before_sha256'].items() if b['source_built_sha256'][p]!=h]
        require(changed==[OBSERVER], 'unexpected build-source differences')
        require(b['source_after_sha256']==b['source_before_sha256'], 'not restored')
        summary['variants'][variant]=dict(receipt=str(path/'receipt.json'),
            receipt_sha256=digest(path/'receipt.json'),
            full_source_inventory_files=len(b['source_before_sha256']),
            altered_files={p:b['source_built_sha256'][p] for p in changed},
            actual_artifacts={n:h['sha256'] for n,h in b['bundle']['files'].items()},
            version=(path/'built-version.stdout').read_text(),
            logs=b['log_sha256'],matching_vstd=[2226,0],production_restored=True)
        summary['commands'].extend(b['commands'])
    for variant in ('baseline','observer-off','capacity-off'):
        path=a.raw/(variant+'-trials')
        receipt=json.loads((path/'receipt.json').read_text())
        require(receipt['success'] and len(receipt['runs'])==15,'trial receipt incomplete')
        summary.setdefault('trial_receipts',{})[variant]=dict(path=str(path/'receipt.json'),
            sha256=digest(path/'receipt.json'))
        summary['commands'].extend(receipt['commands'])
        for run in receipt['runs']:
            label=run['label']
            stdout=path/(label+'.stdout')
            stderr=path/(label+'.stderr')
            require(digest(stdout)==run['stdout_sha256'] and digest(stderr)==run['stderr_sha256'],
                    'raw logs changed')
            data=json.loads(stdout.read_text())
            inf=data['verus-infer']
            require(inf.get('generation_failure') is None and
                inf['status'] in ('no_inference','caller_proof_failed','verification_failed'),
                'unhealthy mandatory/preparation status')
            require_function_coverage(inf['functions'],set())
            require_exact(inf['functions'])
            success=run['verification']['success']
            result=dict(returncode=run['returncode'],stdout=stdout.read_text(),stderr=stderr.read_text())
            if run['case'] in ('plain','wrong_value'):
                validate_goal_run(result,run['case']=='plain',expected_functions=set(),
                    exec_bodies=set(run['exec_bodies']),proof_bodies=set(),
                    failure_kind=None if run['case']=='plain' else 'postcondition not satisfied')
            else:
                diagnostic(result,success,run['exec_bodies'])
            require_publication(data,success,path/(label+'.vir'))
            key=variant+'/'+run['case']+'/'+('on' if run['goals'] else 'off')
            # Exact diagnostic text, source locations and assertion snippets, once
            # per cold/warm pair. Original paths remain in hash-bound raw logs.
            text=stderr.read_text().replace(str(path),'<trial-dir>')
            if key in summary['diagnostic_profiles']:
                require(summary['diagnostic_profiles'][key]['verification']==run['verification'],
                        'cold/warm verification outcomes differ')
                if summary['diagnostic_profiles'][key]['stderr']!=text:
                    key += '/' + run['phase']
            if key not in summary['diagnostic_profiles']:
                summary['diagnostic_profiles'][key]=dict(stderr=text,
                    verification=run['verification'],error_headers=[line for line in text.splitlines()
                        if line.startswith('error: ')])
            selected=[b for b in inf['verification_scope']['native_functions'] if b['selected']]
            summary['trials'].append(dict(runtime=variant,label=label,source_sha256=run['source_sha256'],
                verification=run['verification'],diagnostic_profile=key,
                error_diagnostic_counts=run['diagnostic_counts'],status=inf['status'],
                numeric_applied=run['numeric_applied'],observation_report=run['observation_report'],
                exact_local_inventory=dict(exec=run['exec_bodies'],proof=[],helpers=[]),
                selected_scope=selected,
                complete_selected_scope_verified=inf['verification_scope']['complete_selected_scope_verified'],
                interface_written=inf['interface_written'],
                stdout_sha256=run['stdout_sha256'],stderr_sha256=run['stderr_sha256'],
                cache_files_before=run['cache_files_before'],cache_files_after=run['cache_files_after']))
    for case in ('canonical','prefix','full'):
        disabled=[r['verification'] for r in summary['trials']
            if r['label'].startswith(case+'_off_')]
        require(len(disabled)==6 and all(v==disabled[0] for v in disabled),
                'disabled runtimes differ: '+case)
    summary['disabled_outcomes_identical_across_runtimes']=True
    outcomes={(r['runtime'],r['label']):r['verification']['success'] for r in summary['trials']}
    summary['causal_findings']={case:dict(
        baseline_proves=outcomes['baseline',case+'_on_cold'],
        removing_observer_breaks_proof=(outcomes['baseline',case+'_on_cold'] and
            not outcomes['observer-off',case+'_on_cold']),
        removing_only_capacity_breaks_proof=(outcomes['baseline',case+'_on_cold'] and
            not outcomes['capacity-off',case+'_on_cold']))
        for case in ('canonical','prefix','full')}
    changed=git('diff',BASE,'--name-only').decode().splitlines()
    require(all(p.startswith('source/tools/research/shared_observation_ablation/') for p in changed),
        'production differs from baseline')
    summary['production_identical_to_base']=True
    summary['changed_paths_vs_base']=changed
    a.output.write_text(json.dumps(summary,indent=2)+'\n')
    print(a.output)

if __name__=='__main__': main()
