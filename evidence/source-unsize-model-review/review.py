#!/usr/bin/env python3
"""Independent receipt/product audit and documented replay; MODEL scope only."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
LANE = ROOT/'source/tools/research/native_source_unsize'
GOAL = ROOT/'source/tools/research/mutable_buffer_goal'
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
sys.path[:0] = [str(ROOT/'source/tools'), str(GOAL)]
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt, require
from goal_scope import check_goal_snapshot


def read(path):
    return json.loads(Path(path).read_text())


def canonical_hash(data):
    return hashlib.sha256(json.dumps(data, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def proof_map(receipt):
    pairs = [(Path(p).name, h['sha256']) for p, h in receipt['proof_artifacts'].items()
             if Path(p).suffix in ('.v', '.vo')]
    require(len(pairs) == len(dict(pairs)), 'ambiguous proof product basename')
    return dict(pairs)


def collect():
    """Only this checkout, explicit delivered completed products, shared pinned tools."""
    freeze = read(LANE/'trial004-freeze.json')
    pins = read(LANE/'pins.json')
    results = read(LANE/'results.json')
    ready_path = BASE/'rocq-toolchain/baseline-ready.json'
    ready = read(ready_path)
    storage_pins = read(ROOT/'source/tools/research/native_source_storage/pins.json')
    gmp_path = BASE/'rocq-toolchain/gmp-private/manifest.json'
    gmp = read(gmp_path)
    paths = [ROOT/p for p in freeze['files']]
    paths += [LANE/p for p in ('results.json', 'REPORT.md', 'trial001-freeze.json',
                              'trial002-freeze.json', 'trial003-freeze.json', 'trial004-freeze.json')]
    paths += [HERE/p for p in ('review.py', 'INTERFACE.md', 'SCOPE.json')]
    paths += [GOAL/'TARGET.md', GOAL/'test_goal_scope.py', ready_path, Path(ready['receipt']), gmp_path]
    paths += [Path(ready['switch'])/'_opam/bin'/n for n in ('coqc', 'coqchk')]
    paths += [Path(ready[n]) for n in ('runner', 'opam')]
    paths += [Path(x['archive']) for x in gmp['packages']]
    paths += [Path(x['path']) for x in pins['rust_sources'] + storage_pins['rust_sources']]
    paths += [Path(x) for x in pins['completed_extraction']]
    parent_path = Path(pins['storage_receipt_path'])
    require(parent_path == BASE/'parent-storage_scope_fix-001/proofs/receipt.json', 'unexpected parent receipt')
    paths += [parent_path]
    parent = read(parent_path)
    for p in parent['proof_artifacts']:
        if Path(p).name in ('Interface.v', 'Interface.vo', 'Graph.v', 'Graph.vo', 'Storage.v', 'Storage.vo'):
            require(Path(p).is_relative_to(parent_path.parent), 'parent artifact outside delivered output')
            paths.append(Path(p))
    # Never follow historical checkout paths in receipt['inputs'].
    for item in results['all_trials'].values():
        p = Path(item['path'])
        require(p.parent.parent == Path('/tmp/native-source-unsize-20261110'), 'unexpected trial path')
        paths.append(p)
        for q in read(p).get('proof_artifacts', {}):
            require(Path(q).is_relative_to(p.parent), 'trial artifact outside delivered output')
            paths.append(Path(q))
    lib = Path(storage_pins['coq_library_dependency_pin']['root'])
    libraries = sorted([*(lib/'coq/theories').rglob('*.v'), *(lib/'coq/theories').rglob('*.vo'),
                        *(lib/'coq-core/plugins').rglob('*.cmxs')])
    paths += libraries
    return paths, libraries, lib


def prepare(output):
    output.mkdir(parents=True, exist_ok=False)
    paths, libraries, lib = collect()
    snapshot = ArtifactGuard(paths).initial
    (output/'inputs.json').write_text(json.dumps(snapshot, indent=2)+'\n')
    library_map = {str(f.relative_to(lib)):snapshot[str(f.resolve())]['sha256'] for f in libraries}
    freeze = dict(schema=1, before_first_review_trial=True, inputs_path=str(output/'inputs.json'),
                  inputs_sha256=digest(output/'inputs.json'), input_count=len(snapshot),
                  source_controller_support={str(p.relative_to(ROOT)):digest(p) for p in paths if p.is_relative_to(ROOT)},
                  tools={n:digest(Path(read(BASE/'rocq-toolchain/baseline-ready.json')['switch'])/'_opam/bin'/n)
                         for n in ('coqc','coqchk')},
                  stdlib_count=len(library_map), stdlib_map_sha256=canonical_hash(library_map),
                  goal_requirements_sha256=read(LANE/'trial004-freeze.json')['goal_requirements_sha256'])
    require(not (HERE/'review-freeze.json').exists(), 'review freeze exists')
    (HERE/'review-freeze.json').write_text(json.dumps(freeze, indent=2)+'\n')
    print(json.dumps(dict(prepared=str(output), inputs=len(snapshot), stdlib=len(library_map))))


def run(output):
    freeze = read(HERE/'review-freeze.json')
    require(Path(freeze['inputs_path']) == output/'inputs.json', 'output differs from preregistration')
    require(digest(output/'inputs.json') == freeze['inputs_sha256'], 'input snapshot changed')
    initial = read(output/'inputs.json')
    guard = ArtifactGuard([*initial, HERE/'review-freeze.json', output/'inputs.json'])
    require(all(guard.initial[p] == h for p,h in initial.items()), 'preregistered inputs changed')
    receipt = dict(schema=1, commands=[], scope='unvalidated representation MODEL only',
                   native_correspondences=0, publication=False, goal_complete=False)
    with replay_receipt(output, receipt):
        target = (GOAL/'TARGET.md').read_bytes()
        receipt['goal_requirements_sha256'] = check_goal_snapshot(target, digest(GOAL/'TARGET.md'),
                                                                  freeze['goal_requirements_sha256'])
        receipt['initial_full_target_sha256'] = digest(GOAL/'TARGET.md')
        expected = read(LANE/'trial004-freeze.json')
        results = read(LANE/'results.json')
        pins = read(LANE/'pins.json')
        for f,h in expected['files'].items():
            require(digest(ROOT/f) == h, 'checkpoint source changed '+f)
        for key in ('receipt', 'freeze'):
            item = results[key]
            p = ROOT/item['path'] if key == 'freeze' else Path(item['path'])
            require(digest(p) == item['sha256'], 'results identity '+key)
        trials = {}
        for i,item in results['all_trials'].items():
            p = Path(item['path']); require(digest(p) == item['sha256'], 'trial receipt identity '+i)
            d = read(p); trials[i] = d
            require(d['success'] is item['success'] and len(d['commands']) == item['commands'], 'trial status '+i)
            for q,h in d.get('proof_artifacts',{}).items():
                require(digest(q) == h['sha256'], 'delivered product changed '+q)
        failed = trials['2']
        require(failed['success'] is False and failed['model_proofs_kernel_checked'] is False and
                failed['commands'] == [] and failed['failure'] == dict(type='TypeError', message="object of type 'int' has no len()"),
                'trial002 is not the documented setup failure')
        successful = [trials[i] for i in ('1','3','4')]
        require(all(d['success'] is True and d['model_proofs_kernel_checked'] is True and
                    d['source_declarations'] == 33 and d['kernel_axioms'] == [] for d in successful), 'successful trials')
        require(proof_map(successful[0]) == proof_map(successful[1]) == proof_map(successful[2]), 'successful math differs')
        parent_path = Path(pins['storage_receipt_path']); parent = read(parent_path)
        require(digest(parent_path) == pins['storage_receipt_sha256'] and parent['success'] is True and
                parent['model_proofs_kernel_checked'] is True, 'direct completed parent identity')
        parent_math = {k:v for k,v in proof_map(parent).items()
                       if k in ('Interface.v','Interface.vo','Graph.v','Graph.vo','Storage.v','Storage.vo')}
        require(len(parent_math) == 6, 'parent math count')
        for k,h in parent_math.items():
            require(proof_map(trials['4'])[k] == h, 'parent object identity '+k)
        storage_pins = read(ROOT/'source/tools/research/native_source_storage/pins.json')
        for x in pins['rust_sources'] + storage_pins['rust_sources']:
            require(digest(x['path']) == x['sha256'], 'Rust guidance pin '+x['path'])
        paths, libraries, lib = collect()
        library_map = {str(f.relative_to(lib)):digest(f) for f in libraries}
        require(len(library_map) == freeze['stdlib_count'] == 1121 and canonical_hash(library_map) ==
                freeze['stdlib_map_sha256'] == storage_pins['coq_library_dependency_pin']['map_sha256'], 'stdlib pin')
        # Readiness, durable build, GMP/tool/archive/source pins also gate the README controller before coqc.
        ready = read(BASE/'rocq-toolchain/baseline-ready.json')
        require(ready['baseline_kernel_checked'] is True and read(ready['receipt'])['success'] is True, 'baseline readiness')
        require(digest(BASE/'rocq-toolchain/gmp-private/manifest.json') ==
                '332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37', 'GMP manifest')
        command = LoggedCommands(output, ROOT, receipt['commands'], timeout=1800, guard=guard)
        for label,mode,file in [('source-normal',[],LANE/'test_source_binding.py'),
                                ('source-optimized',['-O'],LANE/'test_source_binding.py'),
                                ('goal-normal',[],GOAL/'test_goal_scope.py'),
                                ('goal-optimized',['-O'],GOAL/'test_goal_scope.py')]:
            require(command(label,['python3','-B',*mode,str(file),'-v'],dict(os.environ)) == 0, 'pure controls failed '+label)
        require(command('readme-replay',['python3','-B','-O',str(LANE/'check.py'),'--freeze',
                    str(LANE/'trial004-freeze.json'),'--output',str(output/'replay')],dict(os.environ)) == 0, 'README replay failed')
        replay = read(output/'replay/receipt.json')
        require(replay['success'] is True and replay['model_proofs_kernel_checked'] is True and
                replay['all_final_guards_before_success'] is True, 'fresh replay status')
        require(proof_map(replay) == proof_map(trials['4']), 'fresh mathematical sources/objects differ')
        names = [m+'.'+n for m,ns in expected['declarations'].items() for n in ns]
        require(len(names) == 33 and replay['assumptions'] == {n:[] for n in names} and replay['kernel_axioms'] == [], 'fresh closure')
        inspection = (output/'replay/proofs/Assumptions.v').read_text().splitlines()
        expected_inspection = ['From NativeSourceUnsize Require Import Carrier Root.']
        for n in names:
            expected_inspection += [f'Redirect "inventory/{n}.assumptions" Print Assumptions NativeSourceUnsize.{n}.',
                                    f'Redirect "inventory/{n}.statement" Check NativeSourceUnsize.{n}.']
        expected_inspection += [f'Redirect "inventory/{m}.module" Print Module NativeSourceUnsize.{m}.' for m in expected['declarations']]
        require(inspection == expected_inspection, 'inspection module contains unexpected commands/declarations')
        inventory = {}
        for n in names:
            stem = output/'replay/proofs/inventory'/n
            assumptions = Path(str(stem)+'.assumptions.out')
            statement = Path(str(stem)+'.statement.out')
            require(assumptions.read_text().strip() == 'Closed under the global context', 'individual assumptions '+n)
            inventory[n] = dict(axioms=[], assumptions_sha256=digest(assumptions),
                                statement=statement.read_text(), statement_sha256=digest(statement))
        # An independent, deliberately narrow inventory for these definition/lemma-only modules.
        for m,ns in expected['declarations'].items():
            text = re.sub(r'\(\*.*?\*\)', '', (LANE/'candidates'/(m+'.v')).read_text(), flags=re.S)
            require(re.findall(r'^(?:Definition|Lemma)\s+(\w+)',text,re.M) == ns, 'source inventory differs')
            require(not re.search(r'\b(?:Inductive|Record|Fixpoint|CoFixpoint|Instance|Axiom|Parameter|Admitted|admit|Program|Obligation|Abstract|Module|Section)\b',text), 'unexpected declaration kind')
            module = (output/'replay/proofs/inventory'/(m+'.module.out')).read_text()
            require(re.findall(r'^     (?:Definition|Parameter)\s+(\w+)',module,re.M) == ns, 'compiler inventory differs')
        (output/'complete-assumptions.json').write_text(json.dumps(inventory,indent=2)+'\n')
        guard.check()
        receipt.update(decision='accept_exact_MODEL_scope', assumptions={n:[] for n in names},
            source_declarations=33, generated_helpers=0, inspection_declarations=0, kernel_axioms=[],
            all_successful_math_products_identical=True, parent_math_objects_identical=True,
            source_controls=dict(normal=9,optimized=9), goal_controls=dict(normal=7,optimized=7),
            direct_parent_receipt=dict(path=str(parent_path),sha256=digest(parent_path)),
            replay_receipt=dict(path=str(output/'replay/receipt.json'),sha256=digest(output/'replay/receipt.json')),
            assumptions_inventory=dict(path=str(output/'complete-assumptions.json'),sha256=digest(output/'complete-assumptions.json')),
            mathematical_products=proof_map(replay), final_full_target_sha256=digest(GOAL/'TARGET.md'),
            all_final_guards_before_success=True)
        require(receipt['final_full_target_sha256'] == receipt['initial_full_target_sha256'], 'full TARGET changed')
    print(json.dumps(dict(receipt=str(output/'receipt.json'),sha256=digest(output/'receipt.json'),decision=receipt['decision'])))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--prepare',action='store_true')
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    (prepare if args.prepare else run)(args.output.resolve())
