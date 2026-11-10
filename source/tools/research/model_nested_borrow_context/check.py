#!/usr/bin/env python3
"""Frozen model-only proof replay; shared baseline and parsers stay read-only."""
import argparse, hashlib, json, os, re, shlex, shutil, sys
from pathlib import Path
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
BRIDGE = HERE.parent/'native_semantic_bridge'
sys.path[:0] = [str(ROOT/'source/tools'), str(BRIDGE)]
from infer_eval_support import ArtifactGuard, digest, support_artifacts
from infer_experiment import LoggedCommands, require, replay_receipt
from proof_checks import parse_axioms, parse_kernel_summary
from acceptance_inputs import require_pinned_environment
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
BUILD = BASE/'verus-belt-build-115a618d'
READY = BASE/'rocq-toolchain/baseline-ready.json'
MODULES = ['protocol', 'instances', 'controls']
HEAP = BASE/'gmp-pin-heap-positive'
SEM = BASE/'semantic-guard-validation-1efbd004/proofs'
TYPED = BASE/'parent-model-input-94b87742/proofs'
LIBS = [(HEAP, 'NativeHeapInitialization'), (SEM, 'NativeSemanticBridge'), (TYPED, 'ModelInputContext')]
PINS = {str(HEAP): 'a3aea2cd291724a26f76082739455cbd4cfa8b64b3d3dcb465a465ed5c6d739d',
        str(SEM): '790aaa8e2eadd1a668bef67cdce938dc3048318d85ee753057cfb805e7982e41',
        str(TYPED): '4c6783ade2f675dc4e53cdeb8d9f3b0f8fa2fa414df3913cb9f0554ca6c4119c'}

def declarations(path):
    return re.findall(r'^\s*(?:Definition|Fixpoint|Lemma|Theorem|Corollary)\s+(\w+)', path.read_text(), re.M)

def inputs():
    return [HERE/'INTERFACE.md', HERE/'FROZEN.md', HERE/'check.py', BRIDGE/'proof_checks.py', BRIDGE/'acceptance_inputs.py',
            BRIDGE/'baseline-bindings.json',
            HERE.parent/'mutable_buffer_goal/foundation-pin.json',
            HERE.parent/'mutable_buffer_goal/foundation-baseline.json',
            HERE.parent/'mutable_buffer_goal/parent-heap-initialization.json',
            HERE.parent/'mutable_buffer_goal/parent-semantic-foundation.json',
            *support_artifacts(), *[HERE/'candidates'/(n+'.v') for n in MODULES]]

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--freeze', type=Path, required=True)
    p.add_argument('--output', type=Path)
    p.add_argument('--create-freeze', action='store_true')
    a = p.parse_args()
    if a.create_freeze:
        require(not a.freeze.exists(), 'freeze already exists')
        a.freeze.write_text(json.dumps(dict(before_first_trial=True,
            files={str(x.relative_to(ROOT)):digest(x) for x in inputs()},
            constants={n:declarations(HERE/'candidates'/(n+'.v')) for n in MODULES}), indent=2)+'\n')
        return
    require(a.output is not None, 'output required')
    a.output = a.output.resolve()
    a.output.mkdir(parents=True, exist_ok=False)
    receipt = dict(schema=1, commands=[], publication=False, model_proofs_kernel_checked=False,
                   source_entry_proved=False, native_effects_discharged=0)
    with replay_receipt(a.output, receipt):
        frozen_bytes = a.freeze.read_bytes()
        freeze = json.loads(frozen_bytes)
        require(freeze['before_first_trial'] is True, 'missing pretrial freeze')
        require(set(freeze['files']) == {str(x.relative_to(ROOT)) for x in inputs()}, 'freeze inputs differ')
        loaded = {}
        def load(path):
            loaded[path] = path.read_bytes()
            return json.loads(loaded[path])
        ready = load(READY)
        baseline_receipt = Path(ready['receipt'])
        baseline = load(baseline_receipt)
        library_receipts = {str(d):load(d/'receipt.json') for d,_ in LIBS}
        trusted = load(HERE.parent/'mutable_buffer_goal/foundation-baseline.json')
        binding = load(BRIDGE/'baseline-bindings.json')
        pin = load(HERE.parent/'mutable_buffer_goal/foundation-pin.json')
        switchbin = Path(ready['switch'])/'_opam/bin'
        toolpaths = {n: (switchbin/n if n in ['coqc', 'coqchk'] else Path(ready[n]))
                     for n in ['coqc', 'coqchk', 'runner', 'opam']}
        sources = baseline['copied_sources']
        paths = [*inputs(), a.freeze, READY, baseline_receipt, BUILD/'_CoqProject',
                 *toolpaths.values(), BASE/'rocq-toolchain/gmp-private/manifest.json',
                 *[d/'receipt.json' for d,_ in LIBS],
                 *[x for d,_ in LIBS for x in (d/'proofs').glob('*') if x.suffix in ['.v','.vo']],
                 *[BUILD/n for n in sources], *list((BUILD/'src').rglob('*.vo'))]
        guard = ArtifactGuard(paths)
        require_pinned_environment(guard.initial, BASE/'rocq-toolchain/gmp-private/manifest.json', trusted)
        def retained(path): return guard.initial[str(path.resolve())]['sha256']
        for path, data in loaded.items():
            require(retained(path) == hashlib.sha256(data).hexdigest(), 'JSON changed while loading '+str(path))
        require(retained(a.freeze) == hashlib.sha256(frozen_bytes).hexdigest(), 'freeze changed during load')
        for name, expected in freeze['files'].items():
            require(retained(ROOT/name) == expected, 'frozen snapshot mismatch '+name)
        require(retained(READY) == trusted['readiness']['sha256'], 'readiness snapshot mismatch')
        require(retained(baseline_receipt) == trusted['receipt']['sha256'], 'baseline receipt snapshot mismatch')
        require(retained(BASE/'rocq-toolchain/gmp-private/manifest.json') == trusted['toolchain']['GMP_packages']['sha256'], 'GMP manifest pin mismatch')
        for d,_ in LIBS:
            require(retained(d/'receipt.json') == PINS[str(d)], 'prerequisite receipt pin mismatch '+str(d))
            lib = library_receipts[str(d)]
            require(lib['success'] is True and lib['model_proofs_kernel_checked'] is True, 'unchecked prerequisite')
            for name, facts in lib['proof_artifacts'].items():
                path=d/'proofs'/Path(name).name
                require(retained(path)==facts['sha256'], 'prerequisite artifact mismatch '+str(path))
        # Parse only authenticated bytes; persistent substitutions before snapshot reject.
        guard.check()
        require(ready['source_commit'] == pin['commit'] == binding['source_commit'] and
                ready['baseline_kernel_checked'] is True and baseline['success'] is True,
                'foundation not checked')
        require(Path(ready['build']).resolve() == BUILD, 'unexpected build')
        require(len(sources) == trusted['copied_source_files'] == 523, 'source inventory incomplete')
        require({'prover-version','build-baseline','kernel-check-baseline'} <=
                {c['label'] for c in baseline['commands'] if c['returncode'] == 0}, 'missing baseline stages')
        for n, expected in sources.items():
            require(retained(BUILD/n) == expected, 'foundation source changed '+n)
        require(retained(BUILD/'_CoqProject') == pin['files']['_CoqProject'], 'loadpaths changed')
        objects = {str(x.relative_to(BUILD)):retained(x) for x in paths
                   if x.suffix == '.vo' and x.is_relative_to(BUILD)}
        require(len(objects) == binding['compiled_module_count'] == 284 and
                hashlib.sha256(json.dumps(objects,sort_keys=True,separators=(',',':')).encode()).hexdigest()
                == binding['compiled_module_map_sha256'], 'baseline objects changed')
        for n, path in toolpaths.items(): require(retained(path) == binding['tools'][n], 'tool changed '+n)
        require(library_receipts[str(TYPED)]['constant_count'] == 26 and
                len([x for x in library_receipts[str(TYPED)]['proof_artifacts'] if x.endswith('.vo')]) == 6,
                'typed prerequisite inventory mismatch')
        receipt['inputs'] = guard.initial
        (a.output/'freeze.json').write_bytes(frozen_bytes)
        run = LoggedCommands(a.output, ROOT, receipt['commands'], timeout=1800, guard=guard)
        env = dict(os.environ, PATH=str(switchbin)+':'+os.environ['PATH'])
        args = []
        for line in (BUILD/'_CoqProject').read_text().splitlines():
            words = shlex.split(line, comments=True)
            if words and words[0] in ['-Q','-R']:
                require(len(words)==3, 'unexpected loadpath')
                args += [words[0], str(BUILD/words[1]), words[2]]
        work = a.output/'proofs'
        work.mkdir()
        # Rocq/lia auxiliary artifacts stay in the private trial directory.
        run.cwd = work
        for d,namespace in LIBS: args += ['-Q',str(d/'proofs'),namespace]
        args += ['-Q',str(work),'ModelNestedBorrowContext']
        held = []
        def hold(path, expected):
            g = ArtifactGuard([path])
            require(g.initial[str(path.resolve())]['sha256']==expected, 'copied source identity mismatch')
            held.append(g)
        names = []
        require({x.stem for x in (HERE/'candidates').glob('*.v')} == set(MODULES), 'extra module')
        for n in MODULES:
            shutil.copyfile(HERE/'candidates'/(n+'.v'),work/(n+'.v'))
            hold(work/(n+'.v'),freeze['files'][str((HERE/'candidates'/(n+'.v')).relative_to(ROOT))])
        for n in MODULES:
            src = HERE/'candidates'/(n+'.v')
            require(declarations(src)==freeze['constants'][n], 'constant inventory changed')
            text = src.read_text()
            require(not re.search(r'\b(Axiom|Admitted|admit|Parameter|Abort|native_compute|vm_cast_no_check|Program|Obligation)\b',text),
                    'unchecked or incomplete construct')
            target = work/src.name
            require(run('compile-'+n,[str(toolpaths['runner']),str(toolpaths['coqc']),'-q',*args,str(target)],env)==0,
                    'proof-development/setup failure, not semantic evidence: '+n)
            require(target.with_suffix('.vo').is_file(), 'missing object')
            names += [n+'.'+x for x in freeze['constants'][n]]
        inv = 'From ModelNestedBorrowContext Require Import '+' '.join(MODULES)+'.\n'
        for name in names:
            inv += f'Goal True. idtac "BEGIN {name}". exact I. Qed.\nPrint Assumptions ModelNestedBorrowContext.{name}.\nGoal True. idtac "END {name}". exact I. Qed.\n'
        inventory = work/'Assumptions.v'
        inventory.write_text(inv)
        hold(inventory,hashlib.sha256(inv.encode()).hexdigest())
        require(run('assumptions',[str(toolpaths['runner']),str(toolpaths['coqc']),'-q',*args,str(inventory)],env)==0,'assumptions command failed')
        out = (a.output/'assumptions.stdout').read_text()
        sections = re.findall(r'^BEGIN (\S+)\n(.*?)^END (\S+)\s*$',out,re.M|re.S)
        require([x for x,_,_ in sections]==names==[x for _,_,x in sections], 'incomplete assumption inventory')
        require(len(re.findall(r'^BEGIN ',out,re.M))==len(names)==len(re.findall(r'^END ',out,re.M)), 'extra markers')
        receipt['assumptions'] = {n:parse_axioms(body) for n,body,_ in sections}
        products = ArtifactGuard([x for x in work.iterdir() if x.suffix in ['.v','.vo']])
        require(run('kernel',[str(toolpaths['runner']),str(toolpaths['coqchk']),'-silent','-o',*args,
                *['ModelNestedBorrowContext.'+n for n in MODULES], 'ModelNestedBorrowContext.Assumptions'],env)==0,'recursive checker failed')
        receipt['kernel_axioms'] = parse_kernel_summary((a.output/'kernel.stderr').read_text())
        for g in held: g.check()
        products.check()
        guard.check()
        receipt.update(model_proofs_kernel_checked=True, constant_count=len(names),
                       proof_artifacts=products.initial)
        print(json.dumps(dict(output=str(a.output), model_proofs_kernel_checked=True,publication=False)))
if __name__ == '__main__': main()
