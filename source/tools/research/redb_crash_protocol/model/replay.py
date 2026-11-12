#!/usr/bin/env python3
"""Private accepted78 replay; immutable inputs, prospective freezes, exact scopes."""
import argparse
import hashlib
import importlib
import json
import os
from pathlib import Path
import re
import shutil
import sys
import tempfile
import time
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
PARENT = 'c008326bfed88eab958335412cdc5ec0e9c8130e'

def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--candidate', type=Path, default=HERE/'protocol.rs')
    ap.add_argument('--expected', choices=('pass','assertion','postcondition'), default='pass')
    ap.add_argument('--label', default='model')
    args = ap.parse_args()
    start = time.monotonic()
    cases = json.loads((HERE.parent/'CASES.json').read_text())
    # These are immutable fixed-parent files, not a live sibling or parent lane.
    for n, h in cases['shared_helpers'].items():
        assert sha(ROOT/n) == h, n
    for n, h in cases['source_sha256'].items():
        assert sha(Path(cases['redb_checkout'])/n) == h, n
    for n, h in cases['charter_sha256'].items():
        assert sha(HERE.parent/n) == h, n
    runtime = cases['accepted_runtime']
    original = Path(runtime['bundle'])
    assert sha(original/'receipt.json') == runtime['receipt_sha256']
    for n, h in runtime['tool_hashes'].items():
        assert sha(original/n) == h, n
    assert (original/'verus-root').is_file()
    private = Path(tempfile.mkdtemp(prefix='redb-model-accepted78-'))
    bundle = private/'bundle'
    shutil.copytree(original,bundle,symlinks=False)
    assert sorted(p.name for p in bundle.iterdir()) == sorted(p.name for p in original.iterdir())
    for n,h in runtime['tool_hashes'].items():
        assert sha(bundle/n) == h, n
    assert sha(bundle/'receipt.json') == runtime['receipt_sha256']
    helpers = private/'helpers'
    helpers.mkdir()
    for n,h in cases['shared_helpers'].items():
        shutil.copyfile(ROOT/n, helpers/Path(n).name)
        assert sha(helpers/Path(n).name) == h
    sys.path.insert(0,str(helpers))
    from infer_eval_support import ArtifactGuard, tool_artifacts
    from infer_experiment import LoggedCommands, replay_receipt, validate_goal_run, require
    out = HERE/'evidence'/private.name
    out.mkdir(parents=True)
    source = out/'candidate.rs'
    shutil.copyfile(args.candidate,source)
    text = source.read_text()
    # Every proof/exec selected. Spec functions are definitions, not body certifications.
    proofs = re.findall(r'pub proof fn (\w+)',text)
    execs = re.findall(r'pub fn (\w+)',text)
    specs = re.findall(r'pub open spec fn (\w+)',text)
    assert len(proofs) == len(set(proofs)) and proofs
    assert not re.search(r'\b(assume|admit|axiom)\s*[(!]|external_body|external_fn_specification|verifier::external',text)
    freeze = dict(parent=PARENT, candidate_sha256=sha(source), original_candidate=str(args.candidate),
        expected=args.expected, exec_bodies=execs, proof_bodies=proofs, full_body_identities=["model::"+n for n in execs+proofs],
        full_spec_identities=["model::"+n for n in specs], spec_definitions=specs,
        expected_infer_helpers=[], infer_helper_count=0, prospective=True,
        existing_infer_registry_sha256=sha(HERE/'ACCEPTED78_INFER_REGISTRY.json'),
        source_coupling_proved=False, model_only=True,
        all_trace_domain='arbitrary finite Seq; no bounded enumeration implied')
    (out/'freeze.json').write_text(json.dumps(freeze,indent=2)+'\n')
    extra = [source,out/'freeze.json',Path(__file__),HERE/'ACCEPTED78_INFER_REGISTRY.json',HERE.parent/'CASES.json',bundle/'receipt.json',bundle/'verus-root',
        *[Path(cases['redb_checkout'])/n for n in cases['source_sha256']],
        *[HERE.parent/n for n in cases['charter_sha256']],
        *[helpers/Path(n).name for n in cases['shared_helpers']]]
    toolchain = Path(cases['toolchain'])
    extra += [toolchain/'bin/rustc',toolchain/'bin/rustup'] if (toolchain/'bin/rustup').exists() else [toolchain/'bin/rustc']
    guard = ArtifactGuard(tool_artifacts(bundle/'verus',bundle/'z3',extra=extra))
    receipt = dict(freeze=freeze, commands=[], runs=[], artifacts=guard.initial,
        private_bundle=str(bundle), toolchain=str(toolchain), source_coupling_proved=False,
        setup_seconds=time.monotonic()-start)
    with replay_receipt(out,receipt):
        call = LoggedCommands(out,private,receipt['commands'],timeout=180,guard=guard,status_key='returncode')
        for infer in (False,True):
            label = args.label+('_infer' if infer else '_ordinary')
            cache = private/(label+'-cache')
            cache.mkdir()
            temp = private/(label+'-tmp')
            temp.mkdir()
            target = private/(label+'-target')
            target.mkdir()
            env = dict(os.environ,PATH=str(toolchain/'bin')+':/home/dev/.cargo/bin:'+os.environ['PATH'],
                RUSTUP_TOOLCHAIN=str(toolchain), RUST_MIN_STACK='33554432',
                VERUS_Z3_PATH=str(bundle/'z3'), VERUS_INFER_CACHE=str(cache),
                CARGO_TARGET_DIR=str(target), TMPDIR=str(temp))
            artifact = out/(label+'.vir')
            assert not artifact.exists()
            cmd = [str(bundle/'verus'),str(source),'--crate-type=lib','--crate-name=model',
                '--no-cheating','--output-json','--multiple-errors','20','--num-threads','1',
                ]
            if infer or args.expected=='pass': cmd += ['--export',str(artifact)]
            if infer: cmd.append('--infer-goals')
            rc = call(label,cmd,env)
            result = dict(returncode=rc,stdout=(out/(label+'.stdout')).read_text(),
                stderr=(out/(label+'.stderr')).read_text())
            data = validate_goal_run(result,args.expected=='pass',expected_functions=set(),
                exec_bodies=set(execs),proof_bodies=set(proofs),
                failure_kind={'pass':None,'assertion':'assertion failed','postcondition':'postcondition not satisfied'}[args.expected])
            native = data['verus-infer']['verification_scope']['native_functions']
            status = {'checked_exec_body','checked_proof_body'} if args.expected=='pass' else {'selected_body_not_certified_by_successful_full_run'}
            full = [b['function'] for b in native if b['body_status'] in status]
            require(len(full)==len(set(full)) and set(full)==set(freeze['full_body_identities']), 'qualified body inventory mismatch')
            require(artifact.exists() == (args.expected=='pass'), 'ordinary export presence disagrees')
            require(data['verus-infer']['trusted_models'] == (json.loads((HERE/'ACCEPTED78_INFER_REGISTRY.json').read_text()) if infer else []), 'accepted78 registry changed')
            require(not data['verus-infer']['imported_inference_dependencies'], 'unexpected inferred dependency')
            # interface_written reports *inferred* interface publication: no helpers here.
            require(data['verus-infer']['interface_written'] is (infer and args.expected=='pass'), 'inference export status mismatch')
            # No reuse of local exported interfaces; negatives must not publish any.
            receipt['runs'].append(dict(label=label,verification=data['verification-results'],
                scope=data['verus-infer']['verification_scope'],
                helpers=data['verus-infer'].get('functions',[]),
                stdout_sha256=sha(out/(label+'.stdout')),stderr_sha256=sha(out/(label+'.stderr')),
                export_sha256=sha(artifact) if artifact.exists() else None))
            print(label,data['verification-results'],flush=True)
        guard.check()
        receipt['total_seconds']=time.monotonic()-start
    print(out,flush=True)

if __name__ == '__main__':
    main()
