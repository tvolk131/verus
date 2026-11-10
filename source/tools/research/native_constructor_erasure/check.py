#!/usr/bin/env python3
"""Check conditional composition + exact pinned-foundation diagnostic. No Rust acceptance."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import sys
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
BRIDGE = HERE.parent/'native_semantic_bridge'
GOAL_DIR = HERE.parent/'mutable_buffer_goal'
TARGET = GOAL_DIR/'TARGET.md'
sys.path[:0] = [str(HERE), str(ROOT/'source/tools'), str(BRIDGE)]
from infer_eval_support import ArtifactGuard, digest, support_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require
sys.path.insert(0, str(GOAL_DIR))
from goal_scope import goal_scope_sha256, check_goal_snapshot
from proof_checks import parse_axioms
from source import load_graph, selected, coq_graph, controls, EXTRACTION
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
READY = BASE/'rocq-toolchain/baseline-ready.json'
BUILD = BASE/'verus-belt-build-115a618d'
PREREQ = Path('/tmp/native-mir-lrust-c6116d49-trial018')
MODULES = ['Interface', 'Graph', 'Composition', 'Boundary']


def declarations(path):
    text = re.sub(r'\(\*.*?\*\)', '', path.read_text(), flags=re.S)
    ns = re.findall(r'^\s*(?:Definition|Lemma|Theorem|Inductive|Record)\s+(\w+)',text,re.M)
    for match in re.finditer(r'^Inductive\s+(\w+)(.*?)\.(?:\n|$)',text,re.M|re.S):
        ns += re.findall(r'(?:^\s*|\|\s*)(\w+)',match[2].split(':=',1)[1])
        ns += [match[1]+'_'+suffix for suffix in
               (['ind','sind'] if re.search(r'\bProp\s*:=',match[2]) else ['rect','ind','rec','sind'])]
    for match in re.finditer(r'^Record\s+(\w+)\s*:=\s*(\w+)\s*\{(.*?)\}\.',text,re.M|re.S):
        ns += [match[2]] + re.findall(r'(?:^|;)\s*(\w+)\s*:',match[3])
    require(len(ns) == len(set(ns)), 'duplicate declaration inventory')
    return ns


def kernel_summary_no_axioms(text):
    require(text.count('CONTEXT SUMMARY') == 1, 'missing/duplicate kernel summary')
    summary = text.split('CONTEXT SUMMARY',1)[1]
    require({x.strip() for x in summary.splitlines() if x.strip()} == {
        '===============', '* Theory: Set is predicative',
        '* Theory: Rewrite rules are not allowed', '* Axioms: <none>',
        '* Constants/Inductives relying on type-in-type: <none>',
        '* Constants/Inductives relying on unsafe (co)fixpoints: <none>',
        '* Inductives whose positivity is assumed: <none>'},
        'unsafe/unrecognized/nonempty-axiom kernel summary')
    return []


def printed_declarations(text):
    ns = re.findall(r'^     (?:Definition|Parameter|Record|Inductive)\s+(\w+)',text,re.M)
    for m in re.finditer(r'^     Inductive\s+\w+(.*?)(?=^     Definition|^     Inductive|^     Record|^   End)',text,re.M|re.S):
        ns += re.findall(r'(?:^\s*|\|\s*)(\w+)\s*:',m[1].split(':=',1)[1])
    ns += re.findall(r'^     Record\s+\w+.*?:=\s*(\w+)',text,re.M)
    require(len(ns)==len(set(ns)), 'duplicate printed declaration')
    return ns


def inputs():
    return [HERE/'check.py', HERE/'source.py', HERE/'INTERFACE.md', HERE/'graph.json',
        *[HERE/'candidates'/(n+'.v') for n in MODULES],
        HERE.parent/'native_mir_lrust_refinement/dependency-pins.json',
        HERE.parent/'native_mir_lrust_refinement/source-coverage.json',
        HERE.parent/'native_mir_lrust_refinement/results.json',
        HERE.parent/'native_mir_lrust_refinement/candidates/fragment.v',
        HERE.parent/'native_semantic_bridge/source-bindings.json',
        BRIDGE/'baseline-bindings.json', BRIDGE/'proof_checks.py',
        HERE.parent/'mutable_buffer_goal/foundation-baseline.json',
        HERE.parent/'mutable_buffer_goal/foundation-pin.json',
        GOAL_DIR/'goal_scope.py',
        HERE.parent/'mutable_buffer_goal/NATIVE_CERTIFICATE.md',
        HERE.parent/'mutable_buffer_goal/FACT_PROVENANCE.md',
        HERE.parent/'native_source_entry/INTERFACE.md',
        HERE.parent/'native_source_entry/MISSING_LAYER.md',
        ROOT/'evidence/native-functional-boundary-audit/REPORT.md',
        ROOT/'evidence/native-functional-boundary-audit/COMPLETED_MIR.json',
        ROOT/'evidence/native-functional-boundary-audit/SOURCE_INVENTORY.json',
        *support_artifacts()]


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--freeze',type=Path,required=True)
    p.add_argument('--create-freeze',action='store_true')
    p.add_argument('--output',type=Path)
    a = p.parse_args()
    if a.create_freeze:
        require(not a.freeze.exists(), 'freeze already exists')
        a.freeze.write_text(json.dumps(dict(before_first_trial=True,
            classification='conditional_composition_and_foundation_diagnostic_not_source_certificate',
            goal_requirements_sha256=goal_scope_sha256(TARGET.read_bytes()),
            files={str(f.relative_to(ROOT)):digest(f) for f in inputs()},
            constants={n:declarations(HERE/'candidates'/(n+'.v')) for n in MODULES}),indent=2)+'\n')
        return
    require(a.output is not None, 'output required')
    a.output = a.output.resolve()
    a.output.mkdir(parents=True,exist_ok=False)
    receipt = dict(schema=1, commands=[], publication=False, source_certificate_proved=False,
                   actual_source_instance=False, native_correspondences=0,
                   conditional_proofs_kernel_checked=False, source_simulations_open=44,
                   phase_obligations_open=3, writer_effects_open=101)
    with replay_receipt(a.output,receipt):
        loaded = {}
        def load(path):
            loaded[path] = path.read_bytes()
            return json.loads(loaded[path])
        freeze = load(a.freeze)
        require(freeze['before_first_trial'] is True, 'missing pretrial freeze')
        require(set(freeze['files']) == {str(f.relative_to(ROOT)) for f in inputs()},'freeze set mismatch')
        ready = load(READY)
        baseline_receipt = Path(ready['receipt'])
        base = load(baseline_receipt)
        trust = load(HERE.parent/'mutable_buffer_goal/foundation-baseline.json')
        bindings = load(BRIDGE/'baseline-bindings.json')
        pin = load(HERE.parent/'mutable_buffer_goal/foundation-pin.json')
        result = load(HERE.parent/'native_mir_lrust_refinement/results.json')
        prereq = load(PREREQ/'receipt.json')
        tools = {n:Path(ready['switch'])/'_opam/bin'/n for n in ['coqc','coqchk']}
        tools.update({n:Path(ready[n]) for n in ['runner','opam']})
        gmp = BASE/'rocq-toolchain/gmp-private/manifest.json'
        target_snapshot = TARGET.read_bytes()
        paths = [TARGET,*inputs(),a.freeze,READY,baseline_receipt,PREREQ/'receipt.json',gmp,
                 *tools.values(), BUILD/'_CoqProject',
                 *[BUILD/n for n in base['copied_sources']],
                 *list((BUILD/'src').rglob('*.vo')),
                 *[Path(n) for n in prereq['proof_artifacts']],
                 *[EXTRACTION/n for n in ['receipt.json','drop-native.json','drop-borrowck.json']]]
        guard = ArtifactGuard(paths)
        def retained(path): return guard.initial[str(path.resolve())]['sha256']
        receipt['goal_requirements_sha256'] = check_goal_snapshot(
            target_snapshot, retained(TARGET), freeze.get('goal_requirements_sha256'))
        for path, data in loaded.items():
            require(retained(path)==hashlib.sha256(data).hexdigest(),'input changed while loading '+str(path))
        for n, value in freeze['files'].items():
            require(retained(ROOT/n)==value,'freeze mismatch '+n)
        require(retained(READY)==trust['readiness']['sha256'] and
                retained(baseline_receipt)==trust['receipt']['sha256'], 'untrusted kernel readiness/receipt')
        require(retained(gmp)==trust['toolchain']['GMP_packages']['sha256']==
                '332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37','GMP manifest mismatch')
        require(ready['source_commit']==bindings['source_commit']==pin['commit']==
                '115a618dd35ff8337112cf2081a7f5391d3973d5' and
                ready['baseline_kernel_checked'] is True and base['success'] is True,
                'unchecked foundation')
        require(Path(ready['build'])==BUILD and len(base['copied_sources'])==523,'wrong baseline')
        require({'prover-version','build-baseline','kernel-check-baseline'} <=
                {c['label'] for c in base['commands'] if c['returncode']==0},'incomplete durable build')
        for n, value in base['copied_sources'].items():
            require(retained(BUILD/n)==value,'baseline source changed '+n)
        require(retained(BUILD/'_CoqProject')==pin['files']['_CoqProject'],'loadpaths changed')
        objects = {str(p.relative_to(BUILD)):retained(p) for p in paths
                   if p.suffix=='.vo' and p.is_relative_to(BUILD)}
        require(len(objects)==284==bindings['compiled_module_count'] and
                hashlib.sha256(json.dumps(objects,sort_keys=True,separators=(',',':')).encode()).hexdigest()
                ==bindings['compiled_module_map_sha256'],'baseline object map mismatch')
        for n,path in tools.items(): require(retained(path)==bindings['tools'][n],'tool identity changed '+n)
        require(retained(PREREQ/'receipt.json')==result['receipt']['sha256']==
                'e98d875a3062053a87e0030d9ee9ec71536a58a942f5ddf182a1cbd2162f3998' and
                prereq['success'] is True and prereq['model_proofs_kernel_checked'] is True,
                'unchecked completed descriptor library')
        for n, facts in prereq['proof_artifacts'].items():
            require(retained(Path(n))==facts['sha256'],'completed product/source changed '+n)
        require(retained(PREREQ/'proofs/fragment.v')==retained(
            HERE.parent/'native_mir_lrust_refinement/candidates/fragment.v'),'descriptor source mismatch')
        data = load_graph()
        graph = selected(*data)
        require(graph==json.loads((HERE/'graph.json').read_text()),'graph inventory mismatch')
        require(coq_graph(graph)==(HERE/'candidates/Graph.v').read_text(),'generated Coq graph mismatch')
        receipt['shape_controls'] = controls(data)
        receipt['inputs'] = guard.initial
        work = a.output/'proofs'
        work.mkdir()
        (work/'inventory').mkdir()
        args=[]
        for line in (BUILD/'_CoqProject').read_text().splitlines():
            words=shlex.split(line,comments=True)
            if words and words[0] in ['-Q','-R']:
                require(len(words)==3,'unexpected loadpath')
                args += [words[0],str(BUILD/words[1]),words[2]]
        args += ['-Q',str(PREREQ/'proofs'),'NativeMirLrust','-Q',str(work),'NativeConstructorErasure']
        env=dict(os.environ)
        run=LoggedCommands(a.output,work,receipt['commands'],timeout=1800,guard=guard)
        held=[]
        def hold(path,value):
            g=ArtifactGuard([path]); require(g.initial[str(path.resolve())]['sha256']==value,'copy identity mismatch')
            held.append(g)
        names=[]
        for n in MODULES:
            src=HERE/'candidates'/(n+'.v')
            require(declarations(src)==freeze['constants'][n],'declaration set changed')
            require(not re.search(r'\b(Axiom|Admitted|admit|Parameter|Abort|Program|Obligation|abstract|native_compute|vm_cast_no_check)\b',src.read_text()),'unproved/unchecked construct')
            target=work/src.name
            shutil.copyfile(src,target)
            hold(target,freeze['files'][str(src.relative_to(ROOT))])
        for n in MODULES:
            require(run('compile-'+n,[str(tools['runner']),str(tools['coqc']),'-q',*args,str(work/(n+'.v'))],env)==0,
                    'proof-development/setup failure (not semantic evidence): '+n)
            product=work/(n+'.vo')
            require(product.is_file(),'missing compiled module')
            held.append(ArtifactGuard([product]))
            names += [n+'.'+x for x in freeze['constants'][n]]
        inventory='From NativeConstructorErasure Require Import '+' '.join(MODULES)+'.\n'
        for n in names:
            inventory += f'Redirect "inventory/{n}.assumptions" Print Assumptions NativeConstructorErasure.{n}.\n'
            inventory += f'Redirect "inventory/{n}.statement" Check NativeConstructorErasure.{n}.\n'
        for n in MODULES:
            inventory += f'Redirect "inventory/{n}.module" Print Module NativeConstructorErasure.{n}.\n'
        target=work/'Assumptions.v'
        target.write_text(inventory)
        hold(target,digest(target))
        require(run('assumptions',[str(tools['runner']),str(tools['coqc']),'-q','-output-directory',str(work),*args,str(target)],env)==0,
                'assumptions/declaration inspection failed')
        receipt['assumptions']={n:parse_axioms((work/'inventory'/(n+'.assumptions.out')).read_text()) for n in names}
        require(all(not v for v in receipt['assumptions'].values()),'new conditional/diagnostic theorem has axiom')
        receipt['module_prints']={n:dict(path=str(work/'inventory'/(n+'.module.out')),
                                         sha256=digest(work/'inventory'/(n+'.module.out'))) for n in MODULES}
        for n in MODULES:
            printed=printed_declarations((work/'inventory'/(n+'.module.out')).read_text())
            require(set(printed)==set(freeze['constants'][n]),'incomplete compiler-discovered declaration inventory: '+n)
        products=ArtifactGuard([p for p in work.rglob('*') if p.is_file() and p.suffix in ['.v','.vo','.out']])
        require(run('kernel',[str(tools['runner']),str(tools['coqchk']),'-silent','-o',*args,
            *['NativeConstructorErasure.'+n for n in MODULES],'NativeConstructorErasure.Assumptions'],env)==0,
            'recursive kernel check failed')
        receipt['kernel_axioms']=kernel_summary_no_axioms((a.output/'kernel.stderr').read_text())
        for g in held: g.check()
        products.check()
        guard.check()
        receipt.update(conditional_proofs_kernel_checked=True,constant_count=len(names),
                       program_obligations=0, proof_artifacts=products.initial,
                       all_final_guards_before_success=True,
                       classification='conditional_composition_and_exact_foundation_blocker_not_Rust_certificate')
        print(json.dumps(dict(output=str(a.output),conditional_proofs_kernel_checked=True,
                              source_certificate_proved=False,publication=False)))

if __name__=='__main__': main()
