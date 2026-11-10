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
MODULES = ['interface', 'guarded', 'caller']
DEPS = json.loads((HERE/'dependency-pins.json').read_text())
LIBS = [(Path(p), d['namespace']) for p,d in DEPS.items()]

def external_inputs():
    lib = BASE/'rocq-toolchain/switch/_opam/lib/coq'
    core = BASE/'rocq-toolchain/switch/_opam/lib/coq-core'
    stubs = BASE/'rocq-toolchain/switch/_opam/lib/stublibs'
    gmp = BASE/'rocq-toolchain/gmp-private'
    return [*[d/'receipt.json' for d,_ in LIBS],
            *[d/'proofs'/n for d,_ in LIBS for n in DEPS[str(d)]['artifacts']],
            *[p for p in lib.rglob('*') if p.is_file() and p.suffix in ['.vo','.cmxs','.cma','.cmi']],
            *[p for p in core.rglob('*') if p.is_file() and p.suffix in ['.cmxs','.cma','.cmi','.so']],
            *[p for p in stubs.rglob('*') if p.is_file()],
            *[p for p in gmp.rglob('*') if p.is_file()]]

def content_map(paths):
    return {str(p.resolve()):digest(p) for p in sorted(set(paths))}

def map_hash(m):
    return hashlib.sha256(json.dumps(m,sort_keys=True,separators=(',',':')).encode()).hexdigest()

def declarations(path):
    return re.findall(r'^\s*(?:Definition|Fixpoint|Lemma|Theorem|Corollary)\s+(\w+)', path.read_text(), re.M)

def inputs():
    return [HERE/'dependency-pins.json', HERE/'INTERFACE.md', HERE/'check.py',
            *[HERE.parent/v['committed_pin'] for v in DEPS.values() if 'committed_pin' in v],
            HERE.parent/'model_shared_byte_read/dependency-pins.json',
            HERE.parent/'model_byte_xor/STATUS.json',
            HERE.parent/'model_buffer_transform/STATUS.json',
            BRIDGE/'proof_checks.py', BRIDGE/'acceptance_inputs.py', BRIDGE/'baseline-bindings.json',
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
        external = content_map(external_inputs())
        a.freeze.write_text(json.dumps(dict(before_first_trial=True,
            files={str(x.relative_to(ROOT)):digest(x) for x in inputs()},
            external_inventory_sha256=map_hash(external),
            external_count=len(external),
            constants={n:declarations(HERE/'candidates'/(n+'.v')) for n in MODULES}), indent=2)+'\n')
        return
    require(a.output is not None, 'output required')
    require(not a.output.resolve().is_relative_to(ROOT), 'outputs must be outside repository')
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
        deps = load(HERE/'dependency-pins.json')
        require(deps == DEPS, 'dependency pins changed during startup')
        library_receipts = {str(d):load(d/'receipt.json') for d,_ in LIBS}
        shared_pins = load(HERE.parent/'model_shared_byte_read/dependency-pins.json')
        direct_pins = {r:load(HERE.parent/r) for r in {v['committed_pin'] for v in deps.values() if 'committed_pin' in v}}
        ready = load(READY)
        baseline_receipt = Path(ready['receipt'])
        baseline = load(baseline_receipt)
        trusted = load(HERE.parent/'mutable_buffer_goal/foundation-baseline.json')
        binding = load(BRIDGE/'baseline-bindings.json')
        pin = load(HERE.parent/'mutable_buffer_goal/foundation-pin.json')
        switchbin = Path(ready['switch'])/'_opam/bin'
        toolpaths = {n: (switchbin/n if n in ['coqc', 'coqchk'] else Path(ready[n]))
                     for n in ['coqc', 'coqchk', 'runner', 'opam']}
        sources = baseline['copied_sources']
        external = external_inputs()
        paths = [*external, *inputs(), a.freeze, READY, baseline_receipt, BUILD/'_CoqProject',
                 *toolpaths.values(), BASE/'rocq-toolchain/gmp-private/manifest.json',
                 *[BUILD/n for n in sources], *list((BUILD/'src').rglob('*.vo'))]
        guard = ArtifactGuard(paths)
        require_pinned_environment(guard.initial,
            BASE/'rocq-toolchain/gmp-private/manifest.json', trusted)
        def retained(path): return guard.initial[str(path.resolve())]['sha256']
        for path, data in loaded.items():
            require(retained(path) == hashlib.sha256(data).hexdigest(), 'JSON changed while loading '+str(path))
        require(retained(a.freeze) == hashlib.sha256(frozen_bytes).hexdigest(), 'freeze changed during load')
        observed_external = {str(p.resolve()):retained(p) for p in external}
        require(len(observed_external) == freeze['external_count'] and map_hash(observed_external) == freeze['external_inventory_sha256'], 'external dependency set changed')
        for d,namespace in LIBS:
            dep = deps[str(d)]
            expected = (direct_pins[dep['committed_pin']]['receipt']['sha256']
                        if 'committed_pin' in dep else shared_pins[str(d)]['receipt_sha256'])
            require(retained(d/'receipt.json') == expected == dep['receipt_sha256'], 'dependency receipt authentication')
            lib = library_receipts[str(d)]
            require(lib['success'] is True and lib['model_proofs_kernel_checked'] is True, 'unchecked library')
            require({p.name for p in (d/'proofs').iterdir() if p.suffix in ['.v','.vo']} == set(dep['artifacts']), 'library file inventory')
            require({Path(n).name:f['sha256'] for n,f in lib['proof_artifacts'].items()} == dep['artifacts'], 'receipt artifact inventory')
            if 'committed_pin' not in dep:
                require(dep['artifacts'] == shared_pins[str(d)]['artifacts'], 'transitive committed artifact pins')
            for name, expected in dep['artifacts'].items():
                require(retained(d/'proofs'/name) == expected, 'dependency artifact '+name)
        for name, expected in freeze['files'].items():
            require(retained(ROOT/name) == expected, 'frozen snapshot mismatch '+name)
        require(retained(READY) == trusted['readiness']['sha256'], 'readiness snapshot mismatch')
        require(retained(baseline_receipt) == trusted['receipt']['sha256'], 'baseline receipt snapshot mismatch')
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
        (a.output/'freeze.json').write_bytes(frozen_bytes)
        run = LoggedCommands(a.output, ROOT, receipt['commands'], timeout=1800, guard=guard)
        env = dict(os.environ, PATH=str(switchbin)+':'+os.environ['PATH'])
        for key in ['COQPATH','COQLIB','COQCORELIB','OCAMLPATH','CAML_LD_LIBRARY_PATH','LD_PRELOAD','LD_LIBRARY_PATH']:
            env.pop(key,None)
        receipt['load_environment'] = 'explicit pinned -Q loadpaths; inherited Coq/OCaml/library overrides removed; pinned runner adds GMP and opam switch'
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
        args += ['-Q',str(work),'ModelCallerWriter']
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
            require(not re.search(r'\b(Axiom|Admitted|admit|Parameter|Abort|native_compute|vm_cast_no_check)\b',text),
                    'unchecked or incomplete construct')
            require(not re.search(r'\b(Inductive|CoInductive|Record|Class|Instance|Program|Obligation|Obligations|Equations)\b',text), 'undeclared generated inventory')
            target = work/src.name
            require(run('compile-'+n,[str(toolpaths['runner']),str(toolpaths['coqc']),'-q',*args,str(target)],env)==0,
                    'proof-development/setup failure, not semantic evidence: '+n)
            require(target.with_suffix('.vo').is_file(), 'missing object')
            names += [n+'.'+x for x in freeze['constants'][n]]
        inv = 'From ModelCallerWriter Require Import '+' '.join(MODULES)+'.\n'
        helper_names = {m: [f'{m}.inventory_{side}_{i}' for i in range(len(names))
                           for side in ['begin','end']] for m in ['Assumptions','ProofTerms']}
        for i,name in enumerate(names):
            inv += f'Lemma inventory_begin_{i} : True. idtac "BEGIN {name}". exact I. Qed.\nPrint Assumptions ModelCallerWriter.{name}.\nLemma inventory_end_{i} : True. idtac "END {name}". exact I. Qed.\n'
        for name in helper_names['Assumptions']:
            inv += f'Print Assumptions ModelCallerWriter.{name}.\n' 
        inventory = work/'Assumptions.v'
        inventory.write_text(inv)
        hold(inventory,hashlib.sha256(inv.encode()).hexdigest())
        require(run('assumptions',[str(toolpaths['runner']),str(toolpaths['coqc']),'-q',*args,str(inventory)],env)==0,'assumptions command failed')
        out = (a.output/'assumptions.stdout').read_text()
        sections = re.findall(r'^BEGIN (\S+)\n(.*?)^END (\S+)\s*$',out,re.M|re.S)
        require([x for x,_,_ in sections]==names==[x for _,_,x in sections], 'incomplete assumption inventory')
        require(len(re.findall(r'^BEGIN ',out,re.M))==len(names)==len(re.findall(r'^END ',out,re.M)), 'extra markers')
        receipt['assumptions'] = {n:parse_axioms(body) for n,body,_ in sections}
        require(out.split('END '+names[-1],1)[1].strip().splitlines() ==
                ['Closed under the global context'] * len(helper_names['Assumptions']), 'helper assumptions not closed')
        terms = 'From ModelCallerWriter Require Import '+ ' '.join(MODULES)+'.\nSet Printing All.\n'
        for i,name in enumerate(names):
            terms += f'Lemma inventory_begin_{i} : True. idtac "BEGIN {name}". exact I. Qed.\nPrint ModelCallerWriter.{name}.\nLemma inventory_end_{i} : True. idtac "END {name}". exact I. Qed.\n'
        for name in helper_names['ProofTerms']:
            terms += f'Print Assumptions ModelCallerWriter.{name}.\n' 
        termfile = work/'ProofTerms.v'
        termfile.write_text(terms)
        hold(termfile,hashlib.sha256(terms.encode()).hexdigest())
        require(run('proof-terms',[str(toolpaths['runner']),str(toolpaths['coqc']),'-q',*args,str(termfile)],env)==0, 'proof-term printing failed')
        printed = re.findall(r'^BEGIN (\S+)\n(.*?)^END (\S+)\s*$', (a.output/'proof-terms.stdout').read_text(), re.M|re.S)
        require([n for n,_,_ in printed] == names == [n for _,_,n in printed], 'proof-term inventory incomplete')
        require((a.output/'proof-terms.stdout').read_text().split('END '+names[-1],1)[1].strip().splitlines() ==
                ['Closed under the global context'] * len(helper_names['ProofTerms']), 'term helper assumptions not closed')
        term_hashes = {n:hashlib.sha256(body.encode()).hexdigest() for n,body,_ in printed}
        products = ArtifactGuard([x for x in work.iterdir() if x.suffix in ['.v','.vo']])
        require(run('kernel',[str(toolpaths['runner']),str(toolpaths['coqchk']),'-o',*args,
                *['ModelCallerWriter.'+n for n in MODULES], 'ModelCallerWriter.Assumptions', 'ModelCallerWriter.ProofTerms'],env)==0,'recursive checker failed')
        kernel_output = (a.output/'kernel.stderr').read_text()
        receipt['kernel_axioms'] = parse_kernel_summary(kernel_output)
        checked = re.findall(r'checking cst:ModelCallerWriter\.((?:'+'|'.join(MODULES)+r')\.\w+)', kernel_output)
        require(checked == names, 'kernel declaration inventory differs: '+repr(checked))
        all_checked = re.findall(r'checking cst:ModelCallerWriter\.(\w+\.\w+)', kernel_output)
        expected_all = names + helper_names['Assumptions'] + helper_names['ProofTerms']
        require(len(all_checked) == len(expected_all) and set(all_checked) == set(expected_all), 'full kernel inventory differs')
        for module,ordered in helper_names.items():
            require([n for n in all_checked if n.startswith(module+'.')] == ordered, 'helper declaration order differs')
        for g in held: g.check()
        products.check()
        guard.check()
        require(all(set(v) <= {'Coq.Logic.FunctionalExtensionality.functional_extensionality_dep', 'Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq'} for v in receipt['assumptions'].values()), 'new logical assumptions in candidate')
        require(set(receipt['kernel_axioms']) <= {'Coq.Logic.FunctionalExtensionality.functional_extensionality_dep', 'Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq'}, 'new kernel logical principles')
        receipt.update(model_proofs_kernel_checked=True, constant_count=len(names), declaration_inventory=freeze['constants'],
                       constructor_inventory=[], program_obligations=[], proof_terms=term_hashes, kernel_declarations=checked,
                       inventory_helper_theorems=helper_names, all_kernel_declarations=all_checked,
                       helper_assumptions={n:[] for values in helper_names.values() for n in values},
                       proof_artifacts=products.initial)
        print(json.dumps(dict(output=str(a.output), model_proofs_kernel_checked=True,publication=False)))
if __name__ == '__main__': main()
