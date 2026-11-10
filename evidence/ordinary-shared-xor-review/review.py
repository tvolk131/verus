#!/usr/bin/env python3
"""Authenticate a pinned completed trial, then replay unchanged in private output."""
import argparse
import hashlib
import importlib.util
import json
import re
import subprocess
import sys
from pathlib import Path
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
SUBJECT = ROOT/'source/tools/research/model_ordinary_shared_xor'
TARGET = '212044b1551d3e14d9010c063bcdd73af78c71cf'
WORKER = Path('/tmp/model-ordinary-shared-xor-trial-008')
WORKER_SHA = '79b5452cb7e3dee9cf69ea6582eda68e6fc5ec3c31ab1e37dc6ac38c563b75ab'
sys.path.insert(0, str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, require, replay_receipt

def load(p):
    return json.loads(p.read_text())

def git_bytes(relative):
    return subprocess.check_output(['git','show',TARGET+':'+str(relative)],cwd=ROOT)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--output', type=Path, required=True)
    args = ap.parse_args()
    output = args.output.resolve()
    require(not output.is_relative_to(ROOT), 'private output required')
    output.mkdir(parents=True, exist_ok=False)
    receipt = dict(schema=1, subject_commit=TARGET, commands=[], publication=False,
                   goal_complete=False, findings_pending_manual_audit=True)
    with replay_receipt(output, receipt):
        # Import only the exact pinned, read-only controller in this checkout.
        require((SUBJECT/'check.py').read_bytes() == git_bytes((SUBJECT/'check.py').relative_to(ROOT)), 'controller differs from target')
        spec = importlib.util.spec_from_file_location('subject_check', SUBJECT/'check.py')
        c = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(c)
        subject_files = subprocess.check_output(['git','ls-tree','-r','--name-only',TARGET,'--',str(SUBJECT.relative_to(ROOT))],cwd=ROOT,text=True).splitlines()
        source_pins = {}
        for relative in subject_files:
            raw = git_bytes(relative)
            require((ROOT/relative).read_bytes() == raw, 'subject differs from target '+relative)
            source_pins[relative] = hashlib.sha256(raw).hexdigest()
        freeze = load(SUBJECT/'final-freeze.json')
        for relative, sha in freeze['files'].items():
            require(digest(ROOT/relative) == sha, 'frozen input mismatch '+relative)
            require(hashlib.sha256(git_bytes(relative)).hexdigest() == sha, 'frozen input not at target '+relative)
        result = load(SUBJECT/'results.json')
        require(digest(WORKER/'receipt.json') == WORKER_SHA == result['receipt']['sha256'], 'worker receipt identity')
        worker = load(WORKER/'receipt.json')
        require(worker['success'] and worker['model_proofs_kernel_checked'], 'worker not checked')
        require((WORKER/'freeze.json').read_bytes() == (SUBJECT/'final-freeze.json').read_bytes(), 'worker freeze differs')
        require(digest(SUBJECT/'final-freeze.json') == result['freeze']['sha256'], 'final freeze pin')
        # Build allowlist from explicit pinned controller inputs, never traverse
        # or open a historical checkout path in a receipt.
        ready = load(c.READY)
        baseline_receipt = Path(ready['receipt'])
        baseline = load(baseline_receipt)
        switchbin = Path(ready['switch'])/'_opam/bin'
        tools = {n:(switchbin/n if n in ['coqc','coqchk'] else Path(ready[n])) for n in ['coqc','coqchk','runner','opam']}
        allowed_external = {p.resolve() for p in [*c.external_inputs(), c.READY,
            baseline_receipt, c.BUILD/'_CoqProject', *tools.values(),
            c.BASE/'rocq-toolchain/gmp-private/manifest.json',
            *[c.BUILD/n for n in baseline['copied_sources']], *list((c.BUILD/'src').rglob('*.vo'))]}
        mapped_inputs = {}
        for label, info in worker['inputs'].items():
            if '/source/' in label and label.rsplit('/source/',1)[1] in {x.removeprefix('source/') for x in freeze['files']}:
                path = ROOT/'source'/label.rsplit('/source/',1)[1]
            elif label == '/tmp/model-ordinary-shared-xor-trial-008-freeze.json':
                path = SUBJECT/'final-freeze.json'
            else:
                # Membership is lexical before resolving: unknown paths cannot
                # cause accesses to live checkout metadata.
                require(label in {str(x) for x in allowed_external}, 'unallowed recorded input '+label)
                path = Path(label)
            require(digest(path) == info['sha256'], 'worker input identity '+label)
            mapped_inputs[label] = str(path)
        products = {}
        expected_products = {**result['proof_sources'], **result['proof_objects']}
        require({Path(n).name for n in worker['proof_artifacts']} == set(expected_products), 'worker product inventory')
        for label, info in worker['proof_artifacts'].items():
            name = Path(label).name
            path = WORKER/'proofs'/name
            require(str(path) == label, 'unexpected product location')
            require(digest(path) == info['sha256'] == expected_products[name], 'worker product pin '+name)
            products[name] = info['sha256']
        for name, sha in result['logs'].items():
            require(digest(WORKER/name) == sha, 'worker log pin '+name)
        expected_names = [m+'.'+n for m, ns in freeze['constants'].items() for n in ns]
        require(len(expected_names) == 21, 'target inventory')
        require(worker['assumptions'] == result['assumptions'], 'worker assumption inventory')
        helpers = worker['inventory_helper_theorems']
        require(sum(map(len, helpers.values())) == 84 and len(worker['helper_assumptions']) == 84, 'helper inventory')
        require(all(v == [] for v in worker['helper_assumptions'].values()), 'helper assumptions')
        paths = [*map(Path,mapped_inputs.values()), *[ROOT/x for x in source_pins],
                 HERE/'review.py', HERE/'INTERFACE.md', WORKER/'receipt.json', WORKER/'freeze.json',
                 *[WORKER/'proofs'/n for n in products], *[WORKER/n for n in result['logs']]]
        guard = ArtifactGuard(paths)
        require(digest(WORKER/'receipt.json') == WORKER_SHA, 'worker changed during authentication')
        for label, info in worker['inputs'].items():
            require(guard.initial[str(Path(mapped_inputs[label]).resolve())]['sha256'] == info['sha256'], 'worker input changed before snapshot '+label)
        for name, expected in products.items():
            require(guard.initial[str((WORKER/'proofs'/name).resolve())]['sha256'] == expected, 'worker product changed before snapshot '+name)
        for name, expected in result['logs'].items():
            require(guard.initial[str((WORKER/name).resolve())]['sha256'] == expected, 'worker log changed before snapshot '+name)
        for relative, expected in source_pins.items():
            require(guard.initial[str((ROOT/relative).resolve())]['sha256'] == expected, 'subject changed before snapshot '+relative)
        receipt['inputs'] = guard.initial
        pretrial = dict(before_first_trial=True, subject_commit=TARGET,
                        final_freeze_sha256=digest(SUBJECT/'final-freeze.json'),
                        files={str(p):v['sha256'] for p,v in guard.initial.items()})
        (output/'review-freeze.json').write_text(json.dumps(pretrial,indent=2)+'\n')
        freeze_guard = ArtifactGuard([output/'review-freeze.json'])
        receipt['worker_authentication'] = dict(receipt_sha256=WORKER_SHA,
            recorded_inputs=len(mapped_inputs), source_objects=products,
            historical_paths_opened=False, subject_files=source_pins)
        # No implementation/controller edits. Unchanged final freeze is passed
        # directly, and all Coq products go to a fresh child directory.
        run = LoggedCommands(output, ROOT, receipt['commands'], timeout=1800, guard=guard)
        import os
        require(run('fresh-replay',[sys.executable,'-B',str(SUBJECT/'check.py'),
            '--freeze',str(SUBJECT/'final-freeze.json'),'--output',str(output/'replay')],dict(os.environ)) == 0,
            'setup/development/replay failure; not semantic negative evidence')
        fresh = load(output/'replay/receipt.json')
        require(fresh['success'] and fresh['model_proofs_kernel_checked'], 'fresh replay incomplete')
        for field in ['assumptions','kernel_axioms','declaration_inventory','constructor_inventory',
                      'program_obligations','existing_instance_registrations',
                      'inventory_helper_theorems','helper_assumptions','all_kernel_declarations','algorithm_audit']:
            require(fresh[field] == worker[field], 'fresh inventory differs '+field)
        fresh_products = {Path(n).name:info['sha256'] for n,info in fresh['proof_artifacts'].items()}
        require(set(fresh_products) == set(products), 'fresh products incomplete')
        require(all(fresh_products[n] == sha for n,sha in result['proof_sources'].items()), 'fresh sources differ')
        # .vo files may encode physical compilation paths. Record actual object
        # hashes and equality; successful recursive checking is authoritative.
        logs = {p.name:digest(p) for p in (output/'replay').iterdir() if p.suffix in ['.stdout','.stderr']}
        kernel = (output/'replay/kernel.stderr').read_text()
        checked = re.findall(r'checking cst:ModelOrdinarySharedXor\.(\w+\.\w+)',kernel)
        require(len(checked) == 105 and set(checked) == set(fresh['all_kernel_declarations']), 'independent kernel inventory')
        require('Modules were successfully checked' in kernel, 'missing full kernel success')
        for cmd in fresh['commands']:
            require(cmd['exit_code'] == 0, 'nonzero check')
            require(not any(x in cmd['argv'] for x in ['-admit','-norec','-exclude','-unsafe','-vos','-vok','-vio']), 'excluded/unsafe check')
        fresh_guard = ArtifactGuard([output/'replay/receipt.json',output/'replay/freeze.json',
            *map(Path,fresh['proof_artifacts']), *[output/'replay'/n for n in logs]])
        for label, info in fresh['proof_artifacts'].items():
            require(fresh_guard.initial[str(Path(label).resolve())]['sha256'] == info['sha256'], 'fresh product changed')
        # Finish all guards before setting successful review receipt fields.
        guard.check()
        freeze_guard.check()
        fresh_guard.check()
        receipt.update(fresh_receipt_sha256=digest(output/'replay/receipt.json'),
            review_freeze_sha256=digest(output/'review-freeze.json'),
            unchanged_final_freeze_sha256=digest(SUBJECT/'final-freeze.json'),
            fresh_commands=fresh['commands'], declaration_inventory=fresh['declaration_inventory'],
            assumptions=fresh['assumptions'], kernel_axioms=fresh['kernel_axioms'],
            helper_assumptions=fresh['helper_assumptions'],
            kernel_declarations=checked, constructor_inventory=[], program_obligations=[],
            existing_instance_registrations=fresh['existing_instance_registrations'],
            fresh_products=fresh_products, object_hashes_match_worker={n:fresh_products[n]==sha for n,sha in result['proof_objects'].items()},
            logs=logs, algorithm_audit=fresh['algorithm_audit'],
            original_guard_count=len(guard.initial), final_identity_guards_passed=True,
            normal_coqc=True, full_recursive_coqchk=True, kernel_exclusions=[])
        print(json.dumps(dict(output=str(output),fresh_replay_checked=True,manual_audit_pending=True)))
if __name__ == '__main__':
    main()
