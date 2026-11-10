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
from binding import check_source, controls
from proof_checks import parse_axioms, parse_kernel_summary
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
BUILD = BASE/'verus-belt-build-115a618d'
READY = BASE/'rocq-toolchain/baseline-ready.json'
MODULES = ['distance', 'branches']
MIR = Path('/tmp/native-mir-lrust-c6116d49-trial018')
LIBS = [(MIR, 'NativeMirLrust')]

def declarations(path):
    text = path.read_text()
    found = [(m.start(),m[1]) for m in re.finditer(r'^\s*(?:Definition|Fixpoint|Inductive|Lemma|Theorem|Corollary)\s+(\w+)',text,re.M)]
    for block in re.finditer(r'^Inductive\s+\w+.*?:=(.*?)\.\n',text,re.M|re.S):
        found += [(block.start(1)+m.start(),m[1]) for m in re.finditer(r'^\|\s+(\w+)',block[1],re.M)]
    names = [n for _,n in sorted(found)]
    for m in re.finditer(r'^Inductive\s+(\w+)(.*?):=',text,re.M|re.S):
        # Coq-generated elimination principles are real declarations too.
        names += [m[1]+'_'+suffix for suffix in
                  (['ind','sind'] if 'Prop' in m[2] else ['rect','ind','rec','sind'])]
    require(len(names)==len(set(names)), 'duplicate declaration inventory')
    return names

def inputs():
    return [HERE/'INTERFACE.md', HERE/'SEMANTICS.md', HERE/'check.py', HERE/'binding.py', HERE/'pins.json',
            HERE/'source-connected.json', ROOT/'evidence/native-functional-boundary-audit/COMPLETED_MIR.json', BRIDGE/'proof_checks.py', BRIDGE/'baseline-bindings.json',
            HERE.parent/'mutable_buffer_goal/foundation-pin.json',
            HERE.parent/'mutable_buffer_goal/foundation-baseline.json',
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
        dep_pin = load(HERE/'pins.json')
        library_receipts = {str(d):load(d/'receipt.json') for d,_ in LIBS}
        trusted = load(HERE.parent/'mutable_buffer_goal/foundation-baseline.json')
        binding = load(BRIDGE/'baseline-bindings.json')
        pin = load(HERE.parent/'mutable_buffer_goal/foundation-pin.json')
        switchbin = Path(ready['switch'])/'_opam/bin'
        toolpaths = {n: (switchbin/n if n in ['coqc', 'coqchk'] else Path(ready[n]))
                     for n in ['coqc', 'coqchk', 'runner', 'opam']}
        sources = baseline['copied_sources']
        paths = [*inputs(), a.freeze, READY, baseline_receipt, BUILD/'_CoqProject',
                 *toolpaths.values(), *[Path(p) for p in dep_pin['source_files']], BASE/'rocq-toolchain/gmp-private/manifest.json',
                 *[d/'receipt.json' for d,_ in LIBS],
                 *[x for d,_ in LIBS for x in (d/'proofs').glob('*') if x.suffix in ['.v','.vo']],
                 *[BUILD/n for n in sources], *list((BUILD/'src').rglob('*.vo'))]
        guard = ArtifactGuard(paths)
        def retained(path): return guard.initial[str(path.resolve())]['sha256']
        for path, data in loaded.items():
            require(retained(path) == hashlib.sha256(data).hexdigest(), 'JSON changed while loading '+str(path))
        require(retained(a.freeze) == hashlib.sha256(frozen_bytes).hexdigest(), 'freeze changed during load')
        for name, expected in freeze['files'].items():
            require(retained(ROOT/name) == expected, 'frozen snapshot mismatch '+name)
        require(retained(READY) == trusted['readiness']['sha256'], 'readiness snapshot mismatch')
        require(retained(baseline_receipt) == trusted['receipt']['sha256'], 'baseline receipt snapshot mismatch')
        require(retained(BASE/'rocq-toolchain/gmp-private/manifest.json') == trusted['toolchain']['GMP_packages']['sha256'], 'GMP manifest pin mismatch')
        require(retained(MIR/'receipt.json') == dep_pin['mir_receipt_sha256'], 'MIR prerequisite receipt mismatch')
        for d,_ in LIBS:
            lib = library_receipts[str(d)]
            require(lib['success'] is True and lib['model_proofs_kernel_checked'] is True, 'unchecked prerequisite')
            for name, facts in lib['proof_artifacts'].items():
                path=d/'proofs'/Path(name).name
                require(retained(path)==facts['sha256'], 'prerequisite artifact mismatch '+str(path))
        for p, sha in dep_pin['source_files'].items(): require(retained(Path(p)) == sha, 'source evidence mismatch '+p)
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
        receipt['inputs'] = guard.initial
        receipt['source_controls'] = controls()
        (a.output/'source-controls.json').write_text(json.dumps(receipt['source_controls'],indent=2)+'\n')
        require(check_source() == json.loads((HERE/'source-connected.json').read_text()), 'source binding changed')
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
        args += ['-Q',str(work),'NativePointerDistance']
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
            require(not re.search(r'\b(Axiom|Admitted|admit|Parameter|Abort|Program|Obligation|abstract|native_compute|vm_cast_no_check)\b',text),
                    'unchecked or incomplete construct')
            target = work/src.name
            require(run('compile-'+n,[str(toolpaths['runner']),str(toolpaths['coqc']),'-q',*args,str(target)],env)==0,
                    'proof-development/setup failure, not semantic evidence: '+n)
            require(target.with_suffix('.vo').is_file(), 'missing object')
            for g in held: g.check()
            hold(target.with_suffix('.vo'), digest(target.with_suffix('.vo')))
            names += [n+'.'+x for x in freeze['constants'][n]]
        inv = 'From NativePointerDistance Require Import '+' '.join(MODULES)+'.\n'
        for name in names:
            inv += f'Goal True. idtac "BEGIN {name}". exact I. Qed.\nPrint Assumptions NativePointerDistance.{name}.\nGoal True. idtac "END {name}". exact I. Qed.\n'
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
                *['NativePointerDistance.'+n for n in MODULES], 'NativePointerDistance.Assumptions'],env)==0,'recursive checker failed')
        kernel_text = (a.output/'kernel.stderr').read_text()
        # coqchk prints the heading and sentinel on one line for an empty TCB.
        # Normalize only that exact sentinel; preserve every safety heading.
        if '* Axioms: <none>\n' in kernel_text:
            require(kernel_text.count('* Axioms: <none>\n') == 1, 'duplicate empty axiom summary')
            kernel_text = kernel_text.replace('* Axioms: <none>\n', '* Axioms:\n')
        receipt['kernel_axioms'] = parse_kernel_summary(kernel_text)
        require(receipt['kernel_axioms'] == [] and all(v == [] for v in receipt['assumptions'].values()),
                'this arithmetic fragment must be entirely closed')
        for g in held: g.check()
        products.check()
        guard.check()
        receipt.update(model_proofs_kernel_checked=True, constant_count=len(names), program_obligations=0,
                       proof_artifacts=products.initial)
        print(json.dumps(dict(output=str(a.output), model_proofs_kernel_checked=True,publication=False)))
if __name__ == '__main__': main()
