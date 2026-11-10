#!/usr/bin/env python3
"""Frozen read-only review, normal replay and complete segment kernel inventory."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
RESEARCH = ROOT/'source/tools/research'
sys.path[:0] = [str(ROOT/'source/tools'), str(RESEARCH/'native_semantic_bridge'),
                str(RESEARCH/'native_byte_xor')]
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, require, replay_receipt
from proof_checks import parse_kernel_summary

BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
BUILD = BASE/'verus-belt-build-115a618d'
READY = BASE/'rocq-toolchain/baseline-ready.json'
PRIM = BASE/'parent-native-primitives-a0d77659/proofs'


def load(p):
    return json.loads(Path(p).read_bytes())


def canonical_hash(data):
    return hashlib.sha256(json.dumps(data, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def controller(name):
    p = RESEARCH/name/'check.py'
    spec = importlib.util.spec_from_file_location(name+'_controller', p)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def paths(reg, byte):
    local = set(reg.inputs()+byte.inputs()+[HERE/'INTERFACE.md', HERE/'review.py'])
    for mod, names in {
        'native_register_segment': ['REPORT.md', 'results.json', 'final-freeze.json'],
        'native_byte_xor': ['REPORT.md', 'TRUST_INVENTORY.json', 'REPRO.md', 'EVIDENCE.json', 'trial008-freeze.json'],
        'native_mir_lrust_refinement': ['INTERFACE.md', 'candidates/fragment.v', 'candidates/simulation.v', 'candidates/reified.v', 'candidates/physical.v']
    }.items():
        local.update(RESEARCH/mod/n for n in names)
    ready = load(READY)
    baseline = load(ready['receipt'])
    external = set(byte.external_inputs()+[READY, Path(ready['receipt']), BUILD/'_CoqProject', PRIM/'receipt.json'])
    external.update(BUILD/n for n in baseline['copied_sources'])
    external.update((BUILD/'src').rglob('*.vo'))
    external.update(p for p in (PRIM/'proofs').iterdir() if p.suffix in ['.v', '.vo'])
    external.update(Path(ready[n]) for n in ['runner', 'opam'])
    external.update(Path(ready['switch'])/'_opam/bin'/n for n in ['coqc', 'coqchk'])
    for pin in [load(RESEARCH/'native_register_segment/results.json'), load(RESEARCH/'native_byte_xor/EVIDENCE.json')]:
        receipt = Path(pin['receipt']['path'])
        # All historical checkout names remain metadata; only explicit /tmp evidence is accessed.
        require(str(receipt).startswith('/tmp/'), 'worker receipt outside /tmp')
        external.update([receipt, receipt.parent/'freeze.json'])
        external.update(p for p in (receipt.parent/'proofs').iterdir() if p.suffix in ['.v', '.vo'])
        external.update(p for p in receipt.parent.iterdir() if p.suffix in ['.stdout', '.stderr'])
        snapshots = receipt.parent/'retained-json'
        if snapshots.exists():
            external.update(snapshots.glob('*.json'))
    return sorted(local), sorted(external)


def historical_input(path):
    """Resolve only /tmp or repository-relative labels, never another checkout."""
    p = Path(path)
    if str(p).startswith('/tmp/'):
        return p
    for prefix in ['source/', 'evidence/']:
        token = '/'+prefix
        if token in str(p):
            return ROOT/(prefix+str(p).split(token, 1)[1])
    raise ValueError('unmapped historical label '+str(p))


def authenticate_worker(module, pin, freeze_name):
    rp = Path(pin['receipt']['path'])
    require(digest(rp) == pin['receipt']['sha256'], 'worker receipt identity '+module)
    r = load(rp)
    freeze = RESEARCH/module/freeze_name
    require(digest(rp.parent/'freeze.json') == digest(freeze), 'worker retained freeze '+module)
    frozen = load(freeze)
    for name, h in frozen['files'].items():
        require(digest(ROOT/name) == h, 'worker frozen input '+name)
    require(r['success'] is True and r['model_proofs_kernel_checked'] is True, 'worker success '+module)
    names = next(iter(frozen['constants'].values()))
    stem = 'segment' if module == 'native_register_segment' else 'native_xor'
    require(list(r['assumptions']) == [stem+'.'+n for n in names], 'worker assumptions names')
    require(all(v == [] for v in r['assumptions'].values()), 'worker nonclosed assumptions')
    for label, info in r['proof_artifacts'].items():
        p = rp.parent/'proofs'/Path(label).name
        require(digest(p) == info['sha256'], 'worker artifact '+str(p))
    # Receipt inputs are labels; rebase repository labels into this checkout.
    for label, info in r['inputs'].items():
        require(digest(historical_input(label)) == info['sha256'], 'worker input identity '+label)
    for label, info in r.get('retained_json', {}).items():
        retained = Path(info['path'])
        require(retained.is_relative_to(rp.parent), 'retained snapshot outside worker output')
        require(digest(retained) == info['sha256'] == digest(historical_input(label)), 'worker retained JSON '+label)
    for log, h in pin.get('logs', {}).items():
        require(digest(rp.parent/log) == h, 'worker log '+log)
    text = (rp.parent/'assumptions.stdout').read_text()
    sections = re.findall(r'^BEGIN (\S+)\n(.*?)^END (\S+)\s*$', text, re.M|re.S)
    require([a for a,_,_ in sections] == list(r['assumptions']) == [b for _,_,b in sections], 'worker raw assumption names')
    require(all(body.strip() == 'Closed under the global context' for _,body,_ in sections), 'worker raw assumptions')
    require(len(names) == r['constant_count'] == (30 if stem == 'segment' else 26), 'worker count')
    if stem == 'segment':
        for name, h in pin['proof_objects'].items():
            require(digest(rp.parent/'proofs'/name) == h, 'segment result object pin')
    else:
        require(digest(rp.parent/'proofs/native_xor.vo') == pin['object_sha256'], 'byte object pin')
        require(digest(rp.parent/'proofs/native_xor.v') == pin['source_sha256'], 'byte source pin')
        require(digest(freeze) == pin['freeze_sha256'], 'byte freeze pin')
    require(digest(rp.parent/'proofs'/(stem+'.v')) == digest(RESEARCH/module/'candidates'/(stem+'.v')), 'worker candidate source')
    return r, rp


def clean_environment():
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
    for key in ['COQPATH', 'COQLIB', 'COQCORELIB', 'OCAMLPATH', 'CAML_LD_LIBRARY_PATH', 'LD_PRELOAD', 'LD_LIBRARY_PATH']:
        env.pop(key, None)
    return env


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--freeze', type=Path, required=True)
    parser.add_argument('--create-freeze', action='store_true')
    parser.add_argument('--output', type=Path)
    a = parser.parse_args()
    reg, byte = controller('native_register_segment'), controller('native_byte_xor')
    local, external = paths(reg, byte)
    files = {str(p.relative_to(ROOT)): digest(p) for p in local}
    emap = {str(p): digest(p) for p in external}
    frozen = dict(schema=1, review_base='91eee431526e9006e2a58c46c8c73fd73a238f60', before_trials=True,
                  files=files, external_count=len(emap), external_inventory_sha256=canonical_hash(emap),
                  worker_freezes={'register': digest(RESEARCH/'native_register_segment/final-freeze.json'),
                                  'byte': digest(RESEARCH/'native_byte_xor/trial008-freeze.json')})
    if a.create_freeze:
        require(not a.freeze.exists(), 'freeze exists')
        a.freeze.write_text(json.dumps(frozen, indent=2)+'\n')
        return
    require(load(a.freeze) == frozen, 'review freeze mismatch')
    require(a.output is not None and not a.output.resolve().is_relative_to(ROOT), 'private output required')
    a.output.mkdir(parents=True, exist_ok=False)
    result = dict(schema=1, commands=[], native_correspondences=0, publication=False,
                  goal_complete=False, review_base=frozen['review_base'])
    with replay_receipt(a.output, result):
        guard = ArtifactGuard([*local, *external, a.freeze])
        result['inputs'] = guard.initial
        (a.output/'freeze.json').write_bytes(a.freeze.read_bytes())
        (a.output/'external-inventory.json').write_text(json.dumps(emap, indent=2)+'\n')
        snapshots = ArtifactGuard([a.output/'freeze.json', a.output/'external-inventory.json'])
        run = LoggedCommands(a.output, ROOT, result['commands'], timeout=1800, guard=guard)
        env = clean_environment()
        result['modules'] = {}
        for module, freeze_name in [('native_register_segment', 'final-freeze.json'), ('native_byte_xor', 'trial008-freeze.json')]:
            pin = load(RESEARCH/module/('results.json' if module == 'native_register_segment' else 'EVIDENCE.json'))
            worker, wp = authenticate_worker(module, pin, freeze_name)
            output = a.output/module
            argv = [sys.executable, '-B', str(RESEARCH/module/'check.py'), '--freeze', str(RESEARCH/module/freeze_name), '--output', str(output)]
            require(run('replay-'+module, argv, env) == 0, 'development/setup failure in '+module)
            fresh = load(output/'receipt.json')
            require(fresh['success'] and fresh['model_proofs_kernel_checked'], 'fresh replay unsuccessful')
            require(fresh['assumptions'] == worker['assumptions'] and all(v == [] for v in fresh['assumptions'].values()), 'fresh assumptions differ')
            objects, sources = {}, {}
            for label, info in worker['proof_artifacts'].items():
                name = Path(label).name
                require(digest(output/'proofs'/name) == info['sha256'], 'fresh artifact differs '+name)
                (objects if name.endswith('.vo') else sources)[name] = info['sha256']
            require((output/'assumptions.stdout').read_bytes() == (wp.parent/'assumptions.stdout').read_bytes(), 'raw assumptions differ')
            require(set(fresh['kernel_axioms']) == set(worker['kernel_axioms']), 'kernel context differs')
            result['modules'][module] = dict(worker_receipt=pin['receipt'], fresh_receipt=dict(path=str(output/'receipt.json'), sha256=digest(output/'receipt.json')),
                objects=objects, sources=sources, assumptions=fresh['assumptions'], kernel_axioms=fresh['kernel_axioms'],
                commands=fresh['commands'], proof_terms_match=fresh.get('proof_terms') == worker.get('proof_terms'))
        # The upstream register controller is silent. Add independent printed types,
        # bodies and generated eliminators, then a verbose full recursive check.
        output = a.output/'native_register_segment'
        fresh = load(output/'receipt.json')
        compile_cmd = fresh['commands'][0]['argv']
        args = compile_cmd[3:-1]
        ready = load(READY)
        runner = ready['runner']
        coqc = str(Path(ready['switch'])/'_opam/bin/coqc')
        coqchk = str(Path(ready['switch'])/'_opam/bin/coqchk')
        names = load(RESEARCH/'native_register_segment/final-freeze.json')['constants']['segment']
        inv = 'From NativeRegisterSegment Require Import segment.\nSet Printing All.\n'
        for n in names:
            inv += f'Goal True. idtac "BEGIN segment.{n}". exact I. Qed.\nPrint NativeRegisterSegment.segment.{n}.\nGoal True. idtac "END segment.{n}". exact I. Qed.\n'
        inv += 'Print Module NativeRegisterSegment.segment.\n'
        target = output/'proofs/ReviewTerms.v'
        target.write_text(inv)
        product_guard = ArtifactGuard([p for p in (output/'proofs').iterdir() if p.suffix in ['.v', '.vo']])
        run.cwd = output/'proofs'
        require(run('register-review-terms', [runner, coqc, '-q', *args, str(target)], env) == 0, 'review terms setup failure')
        printed = (a.output/'register-review-terms.stdout').read_text()
        sections = re.findall(r'^BEGIN (\S+)\n(.*?)^END (\S+)\s*$', printed, re.M|re.S)
        require([n for n,_,_ in sections] == ['segment.'+n for n in names] == [n for _,_,n in sections], 'printed register inventory')
        checked_products = ArtifactGuard([p for p in (output/'proofs').iterdir() if p.suffix in ['.v', '.vo']])
        require(run('register-verbose-kernel', [runner, coqchk, '-o', *args, 'NativeRegisterSegment.segment', 'NativeRegisterSegment.Assumptions', 'NativeRegisterSegment.ReviewTerms'], env) == 0, 'verbose kernel failure')
        kernel = (a.output/'register-verbose-kernel.stderr').read_text()
        axioms = parse_kernel_summary(kernel)
        csts = re.findall(r'checking cst:NativeRegisterSegment\.segment\.(\w+)', kernel)
        inds = re.findall(r'checking ind:NativeRegisterSegment\.segment\.(\w+)', kernel)
        datatype = ['instruction', 'register_step', 'segment_run']
        constructors = ['AssignPure', 'RegisterCompute', 'SegmentNil', 'SegmentCons']
        require(set(csts) == set(names)-set(datatype)-set(constructors) and len(csts) == 23, 'segment kernel constants differ '+repr(csts))
        require(set(inds) == set(datatype) and len(inds) == 3, 'segment datatype inventory '+repr(inds))
        helpers = ['Unnamed_thm']+['Unnamed_thm'+str(i) for i in range(59)]
        for mod in ['Assumptions', 'ReviewTerms']:
            require(re.findall(r'checking cst:NativeRegisterSegment\.'+mod+r'\.(\w+)', kernel) == helpers, 'segment helper inventory '+mod)
        require(len(re.findall(r'checking cst:NativeRegisterSegment\.', kernel)) == 143, 'extra segment namespace constant')
        result['register_inventory'] = dict(declarations=names, constructors=constructors, inductives=inds,
            kernel_constants=csts, generated_eliminators=names[-8:], program_obligations=[],
            kernel_axioms=axioms, helpers_per_inventory=60,
            printed_declaration_sha256={n: hashlib.sha256(body.encode()).hexdigest() for n,body,_ in sections},
            products=checked_products.initial)
        product_guard.check()
        checked_products.check()
        snapshots.check()
        guard.check()
        result['findings'] = []
        result['logs'] = {p.name:digest(p) for p in sorted(a.output.iterdir()) if p.suffix in ['.stdout', '.stderr']}
        print(json.dumps(dict(output=str(a.output), success=True, native_correspondences=0)))

if __name__ == '__main__':
    main()
