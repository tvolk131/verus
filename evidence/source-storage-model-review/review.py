#!/usr/bin/env python3
"""Read-only source/policy review and private MODEL replay, never native acceptance."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
MODEL = ROOT/'source/tools/research/native_source_storage'
GOAL = MODEL.parent/'mutable_buffer_goal'
sys.path[:0] = [str(ROOT/'source/tools'), str(GOAL)]
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import require, replay_receipt, LoggedCommands
from goal_scope import goal_scope_sha256, check_goal_snapshot, GOAL_REQUIREMENTS_SHA256
spec = importlib.util.spec_from_file_location('storage_review_controller', MODEL/'check.py')
controller = importlib.util.module_from_spec(spec)
spec.loader.exec_module(controller)
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
WORKER_SHA = '184b009ecbfa6a343e5183dce5e2df638bf906592c25b979bab5bd877a222b64'
PARENT_SHA = '47f3ebf95acf6e15d3a6b17deabb0103902d48ac62d05a5a7e2fa8079ff8cd1a'
MANAGER_SHA = '03f7e40a44dbebf5f66f7a38e144118902e579686485dab77ee63b62fe438f58'


def load(p):
    return json.loads(Path(p).read_text())


def write(p, d):
    Path(p).write_text(json.dumps(d, indent=2)+'\n')


def map_hash(d):
    return hashlib.sha256(json.dumps(d, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def math_products(receipt):
    # Historical filenames are labels only. Do not read their checkout paths.
    names = [m+s for m in controller.MODULES+controller.PARENTS for s in ['.v', '.vo']]
    out = {}
    for name in names:
        values = [v['sha256'] for k, v in receipt['proof_artifacts'].items()
                  if Path(k).name == name]
        require(len(values) == 1, 'ambiguous/missing math product '+name)
        out[name] = values[0]
    return out


def pure_policy_tests(snapshot):
    passed = []
    def case(name, fn, reject=False):
        try:
            fn()
        except ValueError:
            require(reject, 'unexpected rejection '+name)
        else:
            require(not reject, 'unexpected acceptance '+name)
        passed.append(name)
    h = hashlib.sha256(snapshot).hexdigest()
    case('current_scope', lambda: check_goal_snapshot(snapshot, h, GOAL_REQUIREMENTS_SHA256))
    doc = b'Documentary progress only\n'+snapshot
    case('documentary_progress_allowed', lambda: check_goal_snapshot(
        doc, hashlib.sha256(doc).hexdigest(), GOAL_REQUIREMENTS_SHA256))
    case('requirements_edit_rejected', lambda: goal_scope_sha256(
        snapshot.replace(b'every\nu64', b'some\nu64')), True)
    case('completion_edit_rejected', lambda: goal_scope_sha256(
        snapshot.replace(b'5. Fresh independent review', b'5. Optional independent review')), True)
    case('missing_scope_marker_rejected', lambda: goal_scope_sha256(snapshot.replace(
        b'## Required property and domain\n', b'')), True)
    case('duplicate_scope_marker_rejected', lambda: goal_scope_sha256(
        snapshot+b'## Required property and domain\n'), True)
    case('inverted_scope_markers_rejected', lambda: goal_scope_sha256(
        b'## Coordination\n## Required property and domain\n'), True)
    case('wrong_freeze_pin_rejected', lambda: check_goal_snapshot(snapshot, h, '0'*64), True)
    case('wrong_retained_snapshot_rejected', lambda: check_goal_snapshot(
        snapshot, '0'*64, GOAL_REQUIREMENTS_SHA256), True)
    summary = '''CONTEXT SUMMARY
===============
* Theory: Set is predicative
* Theory: Rewrite rules are not allowed
* Axioms: <none>
* Constants/Inductives relying on type-in-type: <none>
* Constants/Inductives relying on unsafe (co)fixpoints: <none>
* Inductives whose positivity is assumed: <none>
'''
    case('closed_kernel_summary', lambda: controller.kernel_summary_no_axioms(summary))
    case('kernel_axiom_rejected', lambda: controller.kernel_summary_no_axioms(
        summary.replace('* Axioms: <none>', '* Axioms: Bad')), True)
    case('unsafe_kernel_rejected', lambda: controller.kernel_summary_no_axioms(
        summary.replace('unsafe (co)fixpoints: <none>', 'unsafe (co)fixpoints: Bad')), True)
    case('missing_kernel_summary_rejected', lambda: controller.kernel_summary_no_axioms(''), True)
    case('duplicate_kernel_summary_rejected', lambda: controller.kernel_summary_no_axioms(summary+summary), True)
    return passed


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--output', required=True, type=Path)
    a = ap.parse_args()
    out = a.output.resolve()
    require(out.is_relative_to(Path('/tmp')), 'raw output must be private /tmp')
    out.mkdir(exist_ok=False)
    receipt = dict(schema=1, classification='independent_MODEL_review', commands=[],
        publication=False, goal_complete=False, native_correspondences=0,
        actual_source_instance=False, source_certificate_proved=False,
        source_simulations_open=44, phase_obligations_open=3, writer_effects_open=101)
    with replay_receipt(out, receipt):
        freeze_path = MODEL/'integration001-freeze.json'
        freeze = load(freeze_path)
        integ = load(MODEL/'integration-results.json')
        reported = load(MODEL/'results.json')
        historical = [integ[n] for n in ['delivered_receipt', 'integration_receipt', 'manager_receipt']]
        for info, expected in zip(historical, [WORKER_SHA, PARENT_SHA, MANAGER_SHA]):
            require(info['sha256'] == expected == digest(info['path']), 'historical receipt mismatch')
        worker, parent, manager = [load(x['path']) for x in historical]
        require(manager['success'] is True and manager['receipt_sha256'] == PARENT_SHA and
                manager['receipt'] == historical[1]['path'], 'manager binding')
        require(reported['receipt'] == historical[0], 'worker report binding')
        for r in [worker, parent]:
            require(r['success'] is True and r['model_proofs_kernel_checked'] is True and
                    r['constant_count'] == 142 and r['kernel_axioms'] == [] and
                    r['all_final_guards_before_success'] is True, 'unchecked prior replay')
        require(math_products(worker) == math_products(parent), 'prior math mismatch')
        # Authenticate documentary/model files to their delivered Git objects.
        auth = {}
        commits = {'REPORT.md':'ade18e67', 'results.json':'ade18e67', 'pins.json':'ade18e67',
            'README.md':'58a779e2', 'integration-results.json':'58a779e2',
            'integration001-freeze.json':'4aefbe9d', 'check.py':'4aefbe9d',
            'INTERFACE.md':'e77da206', 'source.py':'ade18e67',
            **{'candidates/'+m+'.v':'ade18e67' for m in controller.MODULES}}
        for name, commit in commits.items():
            path = MODEL/name
            blob = subprocess.check_output(['git', 'show', commit+':'+str(path.relative_to(ROOT))], cwd=ROOT)
            require(hashlib.sha256(blob).hexdigest() == digest(path), 'Git identity mismatch '+name)
            auth[str(path.relative_to(ROOT))] = dict(commit=commit, sha256=digest(path))
        ready_path = BASE/'rocq-toolchain/baseline-ready.json'
        ready = load(ready_path)
        baseline_path = BASE/'rocq-toolchain/durable-build-2/receipt.json'
        base = load(baseline_path)
        trust = load(GOAL/'foundation-baseline.json')
        bindings = load(MODEL.parent/'native_semantic_bridge/baseline-bindings.json')
        pins = load(MODEL/'pins.json')
        require(ready['receipt'] == str(baseline_path) and base['success'] is True and
                ready['baseline_kernel_checked'] is True, 'baseline not ready')
        require(digest(ready_path) == trust['readiness']['sha256'] and
                digest(baseline_path) == trust['receipt']['sha256'], 'readiness authenticity')
        require({'prover-version','build-baseline','kernel-check-baseline'} <=
            {c['label'] for c in base['commands'] if c['returncode'] == 0}, 'durable build commands')
        require(ready['source_commit'] == bindings['source_commit'] ==
            '115a618dd35ff8337112cf2081a7f5391d3973d5', 'foundation commit')
        require(Path(ready['build']) == controller.BUILD and len(base['copied_sources']) == 523, 'foundation shape')
        build = controller.BUILD
        for n, h in base['copied_sources'].items():
            require(digest(build/n) == h, 'foundation source mismatch '+n)
        object_map = {str(p.relative_to(build)):digest(p) for p in (build/'src').rglob('*.vo')}
        require(len(object_map) == 284 and map_hash(object_map) == bindings['compiled_module_map_sha256'], 'foundation objects')
        tools = {n:Path(ready['switch'])/'_opam/bin'/n for n in ['coqc','coqchk']}
        tools.update({n:Path(ready[n]) for n in ['runner','opam']})
        for n, p in tools.items():
            require(digest(p) == bindings['tools'][n], 'tool pin '+n)
        gmp = BASE/'rocq-toolchain/gmp-private/manifest.json'
        require(digest(gmp) == trust['toolchain']['GMP_packages']['sha256'] ==
            '332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37', 'GMP manifest')
        gmp_data = load(gmp)
        for x in gmp_data['packages']:
            require(digest(x['archive']) == x['sha256'], 'GMP archive')
        lib = Path(pins['coq_library_dependency_pin']['root'])
        dependencies = sorted([*(lib/'coq/theories').rglob('*.v'), *(lib/'coq/theories').rglob('*.vo'),
            *(lib/'coq-core/plugins').rglob('*.cmxs')])
        dep_map = {str(p.relative_to(lib)):digest(p) for p in dependencies}
        require(len(dep_map) == 1121 == pins['coq_library_dependency_pin']['count'] and
            map_hash(dep_map) == pins['coq_library_dependency_pin']['map_sha256'], 'Coq dependencies')
        require(pins['compiler_commit'] == '58cf8f9c35b355199727627d07ae34a4e2846d67', 'rustc commit')
        for x in pins['rust_sources']:
            require(digest(x['path']) == x['sha256'] and x['url'] ==
                'https://raw.githubusercontent.com/rust-lang/rust/'+pins['compiler_commit']+'/'+x['upstream'], 'Rust primary source')
        require(freeze['before_first_trial'] is True and freeze['goal_requirements_sha256'] ==
            GOAL_REQUIREMENTS_SHA256 and set(freeze['files']) ==
            {str(p.relative_to(ROOT)) for p in controller.inputs()}, 'integration freeze shape')
        for p, h in freeze['files'].items():
            require(digest(ROOT/p) == h, 'integration freeze hash '+p)
        paths = [Path(__file__), HERE/'INTERFACE.md', *controller.inputs(), freeze_path,
            GOAL/'TARGET.md', *[MODEL/n for n in commits], MODEL/'TRUST_INVENTORY.json',
            *[Path(x['path']) for x in historical], ready_path, baseline_path, gmp,
            *tools.values(), build/'_CoqProject', *[build/n for n in base['copied_sources']],
            *[build/n for n in object_map], *dependencies, *[Path(x['archive']) for x in gmp_data['packages']],
            *[Path(x['path']) for x in pins['rust_sources']]]
        guard = ArtifactGuard(paths)
        snapshot = (GOAL/'TARGET.md').read_bytes()
        policy_tests = pure_policy_tests(snapshot)
        preflight = dict(before_proof_trial=True, authenticated_files=auth,
            prior_receipts=historical, inputs=guard.initial, dependency_map_sha256=map_hash(dep_map),
            dependency_count=len(dep_map), baseline_source_count=523, baseline_object_count=284,
            policy_tests=policy_tests, goal_requirements_sha256=goal_scope_sha256(snapshot))
        write(out/'preflight.json', preflight)
        # Only this command compiles; unchanged controller retains all its original guards.
        run = LoggedCommands(out, ROOT, receipt['commands'], timeout=1800, guard=guard)
        replay = out/'replay'
        require(run('frozen-model-replay', [sys.executable, '-B', '-O', str(MODEL/'check.py'),
            '--freeze', str(freeze_path), '--output', str(replay)], dict(os.environ)) == 0, 'MODEL replay failed')
        fresh = load(replay/'receipt.json')
        require(fresh['success'] is True and fresh['model_proofs_kernel_checked'] is True and
            fresh['all_final_guards_before_success'] is True and fresh['program_obligations'] == 0 and
            fresh['kernel_axioms'] == [] and fresh['constant_count'] == 142, 'incomplete fresh checks')
        require(len(fresh['commands']) == 8 and all(c['exit_code'] == 0 for c in fresh['commands']), 'failed/missing Coq commands')
        require(math_products(fresh) == math_products(worker) == math_products(parent), 'math V/VO mismatch')
        require(fresh['assumptions'] == worker['assumptions'] == parent['assumptions'] ==
            reported['assumptions'] == load(MODEL/'TRUST_INVENTORY.json')['assumptions'] and
            len(fresh['assumptions']) == 142 and all(v == [] for v in fresh['assumptions'].values()), 'assumptions mismatch')
        names = {m+'.'+n for m in controller.MODULES for n in freeze['constants'][m]}
        require(names == set(fresh['assumptions']), 'inventory names mismatch')
        for m in controller.MODULES:
            printed = controller.printed_declarations((replay/'proofs/inventory'/(m+'.module.out')).read_text())
            require(set(printed) == set(freeze['constants'][m]), 'Print Module coverage '+m)
        inventory = {}
        for n in sorted(names):
            p = replay/'proofs/inventory'/(n+'.assumptions.out')
            statement = replay/'proofs/inventory'/(n+'.statement.out')
            require(p.read_text().strip() == 'Closed under the global context', 'not closed '+n)
            inventory[n] = dict(assumptions=[], assumptions_sha256=digest(p),
                statement_sha256=digest(statement))
        write(out/'assumptions.json', inventory)
        products = math_products(fresh)
        controller.kernel_summary_no_axioms((replay/'kernel.stderr').read_text())
        guard.check()
        receipt.update(verdict='accepted_MODEL_only', model_proofs_kernel_checked=True,
            all_final_guards_before_success=True, declaration_count=142, kernel_axioms=[],
            math_V_VO_match_worker_and_parent=True, math_products=products,
            assumptions_inventory=dict(path=str(out/'assumptions.json'), sha256=digest(out/'assumptions.json')),
            replay_receipt=dict(path=str(replay/'receipt.json'), sha256=digest(replay/'receipt.json')),
            preflight=dict(path=str(out/'preflight.json'), sha256=digest(out/'preflight.json')),
            policy_tests=policy_tests, goal_requirements_sha256=GOAL_REQUIREMENTS_SHA256)
    print(json.dumps(dict(success=True, verdict=receipt['verdict'], receipt=str(out/'receipt.json'))))


if __name__ == '__main__':
    main()
