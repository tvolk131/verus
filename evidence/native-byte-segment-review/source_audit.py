#!/usr/bin/env python3
"""Source-only identity audit; no native semantic certification or source writes."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
R = ROOT/'source/tools/research'
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
PRIM = BASE/'parent-native-primitives-a0d77659'
SOURCE = Path('/tmp/native-semantic-bridge-e4a8dcab/source-v1')


def digest(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def load(p):
    return json.loads(Path(p).read_bytes())


def require(ok, why):
    if not ok:
        raise ValueError(why)


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--freeze', type=Path, required=True)
    p.add_argument('--create-freeze', action='store_true')
    p.add_argument('--output', type=Path)
    a = p.parse_args()
    local = [HERE/'source_audit.py', HERE/'INTERFACE.md', R/'native_semantic_bridge/source-bindings.json',
             R/'native_mir_lrust_refinement/reify.py', R/'native_mir_lrust_refinement/source-coverage.json',
             R/'native_mir_lrust_refinement/dependency-pins.json', R/'native_mir_lrust_refinement/check.py',
             R/'mutable_buffer_goal/parent-native-primitive-refinement.json',
             *[R/'native_mir_lrust_refinement/candidates'/n for n in ['fragment.v', 'simulation.v', 'physical.v', 'reified.v']]]
    external = [SOURCE/n for n in ['receipt.json', 'drop-native.json', 'drop-borrowck.json']]
    external += [PRIM/'receipt.json', PRIM/'proofs/receipt.json']
    external += [f for f in (PRIM/'proofs/proofs').iterdir() if f.suffix in ['.v', '.vo']]
    freeze = dict(before_trial=True, files={str(f.relative_to(ROOT)):digest(f) for f in local},
                  external={str(f):digest(f) for f in external})
    if a.create_freeze:
        require(not a.freeze.exists(), 'freeze exists')
        a.freeze.write_text(json.dumps(freeze, indent=2)+'\n')
        return
    require(load(a.freeze) == freeze, 'source freeze mismatch')
    require(a.output is not None and not a.output.resolve().is_relative_to(ROOT), 'private output required')
    a.output.mkdir(parents=True, exist_ok=False)
    result = dict(success=False, source_identity_only=True, native_correspondences=0, publication=False)
    try:
        pin = load(R/'mutable_buffer_goal/parent-native-primitive-refinement.json')
        require(digest(PRIM/'receipt.json') == pin['receipt']['sha256'], 'primitive outer receipt')
        require(digest(PRIM/'proofs/receipt.json') == pin['proof_receipt']['sha256'], 'primitive proof receipt')
        outer, proof = load(PRIM/'receipt.json'), load(PRIM/'proofs/receipt.json')
        require(outer['success'] is True and proof['success'] is True and proof['model_proofs_kernel_checked'] is True, 'primitive success')
        for n, h in pin['proof_objects'].items():
            require(digest(PRIM/'proofs/proofs'/n) == h, 'primitive object '+n)
        for label, info in proof['proof_artifacts'].items():
            require(digest(PRIM/'proofs/proofs'/Path(label).name) == info['sha256'], 'primitive artifact '+label)
        for name in ['fragment', 'simulation', 'physical', 'reified']:
            require(digest(PRIM/'proofs/proofs'/(name+'.v')) == digest(R/'native_mir_lrust_refinement/candidates'/(name+'.v')), 'primitive source '+name)
        # No path from the source receipt is followed: only the three explicit pins.
        source_pin = load(R/'native_semantic_bridge/source-bindings.json')
        deps = load(R/'native_mir_lrust_refinement/dependency-pins.json')
        for n,h in {'receipt.json':source_pin['receipt_sha256'], **source_pin['outputs']}.items():
            require(digest(SOURCE/n) == h == deps['source_files'][str(SOURCE/n)], 'source identity '+n)
        require(load(SOURCE/'receipt.json')['success'] is True, 'source receipt unsuccessful')
        spec = importlib.util.spec_from_file_location('source_review_reifier', R/'native_mir_lrust_refinement/reify.py')
        reifier = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(reifier)
        selected = reifier.run_reifier()
        require(selected == load(R/'native_mir_lrust_refinement/source-coverage.json'), 'source coverage mismatch')
        require(reifier.proof_source(selected).encode() == (R/'native_mir_lrust_refinement/candidates/reified.v').read_bytes(), 'reified source mismatch')
        native = load(SOURCE/'drop-native.json')
        inst = next(i for i in native['instances'] if i['id'] == selected['source_constructor'])
        block = next(b for b in inst['blocks'] if b['bb'] == 2)
        edges = [dict(bb=2, statement=i, ir=block['statements'][i]['ir']) for i in [1,2]]
        controls = reifier.controls()  # deepcopy/in-memory only; never invokes writing CLI.
        result.update(success=True, primitive_receipt=pin['receipt'], primitive_proof_receipt=pin['proof_receipt'],
            primitive_objects=pin['proof_objects'], source_files=deps['source_files'],
            selected_edges=edges, source_counts=selected['counts'], controls=controls,
            source_constructor=selected['source_constructor'], freeze_sha256=digest(a.freeze))
        # Frozen artifacts remain immutable; no changing-input or substitution test.
        require(load(a.freeze) == dict(before_trial=True,
            files={str(f.relative_to(ROOT)):digest(f) for f in local}, external={str(f):digest(f) for f in external}), 'source inputs changed')
        (a.output/'freeze.json').write_bytes(a.freeze.read_bytes())
    except BaseException as e:
        result.update(success=False, failure=dict(type=type(e).__name__, message=str(e)), classification='review setup/development failure; not semantic rejection')
        raise
    finally:
        (a.output/'receipt.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(dict(output=str(a.output), success=True, native_correspondences=0)))

if __name__ == '__main__':
    main()
