#!/usr/bin/env python3
"""Independent compact acceptance audit of a complete 26-case raw replay."""
import argparse
import json
from pathlib import Path
import sys
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import digest
from infer_experiment import (require, require_exact, require_failure_kind,
    require_function_coverage, validate_goal_run, validate_run)
from fidelity import fidelity

POSITIVES = {'ordinary_canonical': ({'u64'}, {'radix_positive','quotient_step','byte_step'}, set()),
    'canonical_cold': ({'u64'}, set(), set()), 'canonical_warm': ({'u64'}, set(), set()),
    'safety_length': ({'safety_length'}, set(), set()),
    'healthy_exact_helper': ({'u64','identity'}, set(), {'identity'}),
    'provider_cold': ({'u64','identity'}, set(), {'identity'}),
    'provider_warm': ({'u64','identity'}, set(), {'identity'}),
    'consumer': ({'caller'}, set(), set())}
SEMANTIC = {'false_return_length','false_payload','false_continuation','false_minimal_length','seeded_failed_export'}
DIAGNOSTIC = {'disabled','wrong_shift','wrong_break','wrong_counter','wrong_payload',
              'wrong_continuation','step8','capacity9','width32','generic_radix'}
LABELS = set(POSITIVES) | SEMANTIC | DIAGNOSTIC | {'wrong_return_range','poisoned_helper','metadata_poison'}


def main():
    p = argparse.ArgumentParser()
    p.add_argument('receipt', type=Path)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    receipt = json.loads(a.receipt.read_text())
    out = a.receipt.parent
    require(receipt['success'] is True, 'controller did not complete')
    labels = [r['label'] for r in receipt['runs']]
    require(len(labels)==len(set(labels)) and set(labels)==LABELS, 'case inventory mismatch')
    require(receipt['artifacts']==receipt['artifacts_after'], 'guarded artifact drift')
    for name, sha in receipt['sources'].items():
        require(digest(ROOT/name)==sha, 'source changed after replay: '+name)
    historical = json.loads((ROOT/'source/tools/research/mutable_buffer_goal/freeze.json').read_text())
    for name, sha in historical['files'].items():
        require(digest(ROOT/name)==sha, 'historical frozen evidence changed')
    summaries=[]
    for r in receipt['runs']:
        label=r['label']
        command=next(c for c in receipt['commands'] if c['label']==label)
        source=Path(command['argv'][1])
        require(digest(source)==r['source_sha256'], 'frozen input drift: '+label)
        require('--no-cheating' in command['argv'] and not any(f in command['argv'] for f in
            ('--no-lifetime','--no-verify','--verify-root','--verify-function','--verify-only-module')), 'checking bypass')
        stdout=(out/(label+'.stdout')).read_text()
        stderr=(out/(label+'.stderr')).read_text()
        require(digest(out/(label+'.stdout'))==r['stdout_sha256'] and
                digest(out/(label+'.stderr'))==r['stderr_sha256'], 'raw log drift')
        run=dict(returncode=command['returncode'],stdout=stdout,stderr=stderr)
        data=json.loads(stdout)
        info=data.get('verus-infer',{})
        functions=info.get('functions',[])
        helpers=({'identity'} if label in SEMANTIC or label in ('wrong_return_range','poisoned_helper')
                 else POSITIVES[label][2] if label in POSITIVES else set())
        # Identity and mandatory proof checks precede the requested goal's result.
        require_function_coverage(functions,helpers)
        if label=='poisoned_helper':
            f=functions[0]
            require(f['mode']=='exact' and f['required_checks_passed'] is False, 'poisoned mode/acceptance')
            for name,status in [('soundness','not_proved'),('maximality','proved'),('exactness','not_proved')]:
                require(f['proofs'][name]['required'] is True and f['proofs'][name]['status']==status,
                    'poisoned helper mandatory proof mismatch: '+name)
            require(f['metrics']['cache']=='hit', 'poisoned cache not consumed')
            validate_run(run,False)
            require(info['crate_success'] is False and info['interface_written'] is False,'poisoned export')
            category='intentional cached-helper poison; all three required statuses bound'
        else:
            require_exact(functions)
            if label in POSITIVES:
                execs,proofs,_=POSITIVES[label]
                validate_goal_run(run,True,expected_functions=helpers,exec_bodies=execs,proof_bodies=proofs)
                category='positive'
            elif label in SEMANTIC:
                validate_goal_run(run,False,expected_functions=helpers,
                    exec_bodies={'u64','identity','false_goal'},proof_bodies=set(),failure_kind='postcondition not satisfied')
                category='strict semantic negative with healthy exact helper'
            elif label=='metadata_poison':
                validate_run(run,False,allow_frontend=True)
                require('rebuild the library' in stderr, 'missing import rejection')
                require(not any(k in stderr for k in ('assertion failed','precondition not satisfied','postcondition not satisfied')), 'mixed metadata rejection')
                category='intentional metadata poison; not semantic evidence'
            else:
                validate_run(run,False)
                v=data['verification-results']
                require(v.get('is-verifying-entire-crate') is True and v.get('errors',0)>0
                    and 'error[E' not in stderr, 'frontend or incomplete diagnostic')
                selected=[f for f in info['verification_scope']['native_functions']
                    if f['body_status']=='selected_body_not_certified_by_successful_full_run']
                require_function_coverage(selected,{'u64','identity'} if label=='wrong_return_range' else {'u64'})
                if label=='wrong_return_range':
                    require_failure_kind(stderr,'postcondition not satisfied')
                    require(v['errors']==1 and 'invariant not satisfied' not in stderr and
                            'possible arithmetic underflow/overflow' not in stderr, 'wrong range mixed failure')
                    category='authored postcondition rejection; inherited cast recommendation note excludes strict gate'
                else:
                    require(not any(d['profile']=='mutable_buffer_u64_radix128' and d['status']=='applied'
                        for d in info['assistance_explanations']['decisions']), 'unsupported shape selected')
                    category='unsupported/disabled shape: ordinary invariant/safety/goal rejection, not semantic evidence'
        if label in ('ordinary_canonical','canonical_cold','canonical_warm'):
            fidelity(source)
            expected=HERE/('candidate-004.rs' if label=='ordinary_canonical' else 'candidate-008.rs')
            require(source.read_bytes()==expected.read_bytes(),'canonical input is not exact frozen fixture')
        if label=='consumer':
            require(info['imported_inference_dependencies'],'missing certified import identity')
        if label=='seeded_failed_export':
            require(not (out/'failed.vir').exists() and info['interface_written'] is False,'stale failed export remains')
        summaries.append(dict(label=label,category=category,source_sha256=r['source_sha256'],
            verification=data['verification-results'],helpers=functions,
            ordinary_execs=r['expected_execs'],ordinary_proofs=r['expected_proofs'],
            assistance=r['assistance'],stdout_sha256=r['stdout_sha256'],stderr_sha256=r['stderr_sha256']))
    result=dict(schema=1,success=True,case_count=len(labels),raw_receipt=str(a.receipt.resolve()),
        raw_receipt_sha256=digest(a.receipt),audit_sha256=digest(Path(__file__)),
        source_hashes=receipt['sources'],commands=receipt['commands'],artifacts=receipt['artifacts'],runs=summaries)
    a.output.write_text(json.dumps(result,indent=2)+'\n')
    print('accepted',len(labels),'cases:',a.output)

if __name__=='__main__':
    main()
