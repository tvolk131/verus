#!/usr/bin/env python3
"""Normal checking of an authored partial MODEL. Never native acceptance."""
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
GRAPH = HERE.parent/"native_constructor_erasure/graph.json"
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
READY = BASE/'rocq-toolchain/baseline-ready.json'
BUILD = BASE/'verus-belt-build-115a618d'
MODULES = ['Model']
PARENTS = ['Interface', 'Graph', 'Composition']

def declarations(path):
    text = re.sub(r'\(\*.*?\*\)', '', path.read_text(), flags=re.S)
    ns = re.findall(r'^\s*(?:Definition|Lemma|Theorem|Inductive|Record)\s+(\w+)',text,re.M)
    for match in re.finditer(r'^Inductive\s+(\w+)(.*?)\.(?:\n|$)',text,re.M|re.S):
        ns += re.findall(r'(?:^\s*|\|\s*)(\w+)',match[2].split(':=',1)[1])
        ns += [match[1]+'_'+suffix for suffix in
               (['ind','sind'] if re.search(r'\bProp\s*:=',match[2]) else ['rect','ind','rec','sind'])]
    for match in re.finditer(r'^Record\s+(\w+)(?:\s+\([^\n]*?\))?\s*:=\s*(\w+)\s*\{(.*?)\}\.',text,re.M|re.S):
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
    paths = [HERE/'check.py', HERE/'INTERFACE.md', HERE/'pins.json',
        *[HERE/'candidates'/(n+'.v') for n in MODULES],
        GOAL_DIR/'goal_scope.py', GRAPH, BRIDGE/'baseline-bindings.json',
        BRIDGE/'source-bindings.json', GOAL_DIR/'foundation-baseline.json',
        GOAL_DIR/'foundation-pin.json', *support_artifacts()]
    for lane in ['native_source_storage','native_constructor_erasure','native_original_reference_exit']:
        paths += [HERE.parent/lane/'integration-results.json', HERE.parent/lane/'REPORT.md']
    paths += [ROOT/'evidence/native-functional-boundary-audit/REPORT.md',
        HERE.parent/'native_borrow_foundation_inspection/REPORT.md',
        HERE.parent/'native_borrow_foundation_inspection/pins.json']
    return paths


def main():
    p=argparse.ArgumentParser()
    p.add_argument('--freeze',type=Path,required=True)
    p.add_argument('--create-freeze',action='store_true')
    p.add_argument('--output',type=Path)
    a=p.parse_args()
    if a.create_freeze:
        require(not a.freeze.exists(),'freeze already exists')
        a.freeze.write_text(json.dumps(dict(before_first_trial=True,
            classification='MODEL_conservative_reborrow_not_Rust_certificate',
            goal_requirements_sha256=goal_scope_sha256(TARGET.read_bytes()),
            files={str(f.relative_to(ROOT)):digest(f) for f in inputs()},
            constants={n:declarations(HERE/'candidates'/(n+'.v')) for n in MODULES}),indent=2)+'\n')
        return
    require(a.output is not None,'output required')
    a.output=a.output.resolve(); a.output.mkdir(parents=True,exist_ok=False)
    receipt=dict(schema=1,commands=[],publication=False,native_correspondences=0,
        source_certificate_proved=False,actual_source_instance=False,goal_complete=False,
        model_proofs_kernel_checked=False,source_simulations_open=44,
        phase_obligations_open=3,writer_effects_open=101)
    with replay_receipt(a.output,receipt):
        loaded={}
        def load(path):
            loaded[path]=path.read_bytes(); return json.loads(loaded[path])
        freeze=load(a.freeze); ready=load(READY)
        baseline_receipt=Path(ready['receipt']); base=load(baseline_receipt)
        trust=load(HERE.parent/'mutable_buffer_goal/foundation-baseline.json')
        bindings=load(BRIDGE/'baseline-bindings.json')
        pin=load(HERE.parent/'mutable_buffer_goal/foundation-pin.json')
        pins=load(HERE/'pins.json')
        completed={}
        for lane,x in pins['integration_receipts'].items():
            path=Path(x['path']); data=load(path)
            require(digest(path)==x['sha256'] and data['success'] is True,'unchecked dependency '+lane)
            completed[lane]=(path,data)
        completed_receipt,prior=completed['native_source_storage']
        require(freeze['before_first_trial'] is True,'missing pretrial freeze')
        require(set(freeze['files'])=={str(f.relative_to(ROOT)) for f in inputs()},'freeze set mismatch')
        tools={n:Path(ready['switch'])/'_opam/bin'/n for n in ['coqc','coqchk']}
        tools.update({n:Path(ready[n]) for n in ['runner','opam']})
        gmp=BASE/'rocq-toolchain/gmp-private/manifest.json'
        gmp_data=load(gmp)
        lib=Path(pins['coq_library_dependency_pin']['root'])
        dependencies=sorted([*list((lib/'coq/theories').rglob('*.v')),*list((lib/'coq/theories').rglob('*.vo')),*list((lib/'coq-core/plugins').rglob('*.cmxs'))])
        objects=list((BUILD/'src').rglob('*.vo'))
        target_snapshot=TARGET.read_bytes()
        paths=[TARGET,*inputs(),a.freeze,READY,baseline_receipt,completed_receipt,gmp,
            *tools.values(),BUILD/'_CoqProject', *[BUILD/n for n in base['copied_sources']],
            *objects,*dependencies,*[Path(x['archive']) for x in gmp_data['packages']],*[Path(x['path']) for x in pins['rust_sources']+pins['research_sources']],
            *[Path(x) for x in pins['completed_extraction']],
            *[path for path,data in completed.values()], *[Path(x) for path,data in completed.values() for x in data['proof_artifacts']]]
        guard=ArtifactGuard(paths)
        def retained(path):return guard.initial[str(path.resolve())]['sha256']
        receipt['goal_requirements_sha256']=check_goal_snapshot(
            target_snapshot,retained(TARGET),freeze.get('goal_requirements_sha256'))
        for path,data in loaded.items():
            require(retained(path)==hashlib.sha256(data).hexdigest(),'input changed while loading '+str(path))
        for n,h in freeze['files'].items():require(retained(ROOT/n)==h,'freeze mismatch '+n)
        for x in gmp_data['packages']:require(retained(Path(x['archive']))==x['sha256'],'GMP archive changed')
        dependency_map={str(f.relative_to(lib)):retained(f) for f in dependencies}
        require(len(dependency_map)==pins['coq_library_dependency_pin']['count'] and
            hashlib.sha256(json.dumps(dependency_map,sort_keys=True,separators=(',',':')).encode()).hexdigest()==
            pins['coq_library_dependency_pin']['map_sha256'],'Coq dependency map changed')
        require(pins['compiler_commit']==
            '58cf8f9c35b355199727627d07ae34a4e2846d67','compiler commit changed')
        for x in pins['rust_sources']:require(retained(Path(x['path']))==x['sha256'] and x['url']==
            'https://raw.githubusercontent.com/rust-lang/rust/'+pins['compiler_commit']+'/'+x['upstream'],'rust source changed')
        source_bindings=load(BRIDGE/'source-bindings.json')
        extraction=Path('/tmp/native-semantic-bridge-e4a8dcab/source-v1')
        require(retained(extraction/'receipt.json')==source_bindings['receipt_sha256'],'unauthenticated source receipt')
        for n,h in source_bindings['outputs'].items():require(retained(extraction/n)==h,'unauthenticated completed source '+n)
        for n,h in pins['completed_extraction'].items():require(retained(Path(n))==h,'completed extraction changed')
        require(retained(READY)==trust['readiness']['sha256'] and retained(baseline_receipt)==trust['receipt']['sha256'],
                'untrusted readiness/receipt')
        require(retained(gmp)==trust['toolchain']['GMP_packages']['sha256']==
            '332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37','GMP manifest mismatch')
        require(ready['source_commit']==bindings['source_commit']==pin['commit']==
            '115a618dd35ff8337112cf2081a7f5391d3973d5' and ready['baseline_kernel_checked'] is True and base['success'] is True,
            'unchecked foundation')
        require(baseline_receipt==BASE/'rocq-toolchain/durable-build-2/receipt.json' and
            Path(ready['build'])==BUILD and len(base['copied_sources'])==523,'wrong baseline')
        require({'prover-version','build-baseline','kernel-check-baseline'} <=
            {c['label'] for c in base['commands'] if c['returncode']==0},'incomplete durable build')
        for n,h in base['copied_sources'].items():require(retained(BUILD/n)==h,'baseline source changed '+n)
        require(retained(BUILD/'_CoqProject')==pin['files']['_CoqProject'],'loadpaths changed')
        object_map={str(x.relative_to(BUILD)):retained(x) for x in objects}
        require(len(object_map)==284==bindings['compiled_module_count'] and
            hashlib.sha256(json.dumps(object_map,sort_keys=True,separators=(',',':')).encode()).hexdigest()==
            bindings['compiled_module_map_sha256'],'baseline objects changed')
        for n,path in tools.items():require(retained(path)==bindings['tools'][n],'tool changed '+n)
        for lane,(path,data) in completed.items():
            require(retained(path)==pins['integration_receipts'][lane]['sha256'],'dependency receipt changed')
            for n,x in data['proof_artifacts'].items():
                require(retained(Path(n))==x['sha256'],'completed product changed '+n)
        for x in pins['research_sources']:
            require(retained(Path(x['path']))==x['sha256'],'research source changed')
        require(prior['constant_count']==142 and prior['model_proofs_kernel_checked'] is True,
            'storage interface identity changed')
        require(len(completed['native_constructor_erasure'][1]['assumptions'])==96,
            'constructor identity changed')
        receipt['inputs']=guard.initial
        work=a.output/'proofs'; work.mkdir(); (work/'inventory').mkdir()
        parent=a.output/'parent'; parent.mkdir()
        storage=a.output/'storage'; storage.mkdir()
        args=[]
        for line in (BUILD/'_CoqProject').read_text().splitlines():
            words=shlex.split(line,comments=True)
            if words and words[0] in ['-Q','-R']:
                require(len(words)==3,'unexpected loadpath');args += [words[0],str(BUILD/words[1]),words[2]]
        args += ['-Q',str(parent),'NativeConstructorErasure','-Q',str(storage),'NativeSourceStorage','-Q',str(work),'NativeSourceReborrow']
        env=dict(os.environ);run=LoggedCommands(a.output,work,receipt['commands'],timeout=1800,guard=guard)
        held=[]
        def hold(path,h):
            g=ArtifactGuard([path]);require(g.initial[str(path.resolve())]['sha256']==h,'copy mismatch');held.append(g)
        for n in PARENTS+['Storage','Sites','Prefix']:
            dest=parent if n in PARENTS else storage
            for ext in ['.v','.vo']:
                matches=[Path(x) for x in prior['proof_artifacts'] if Path(x).name==n+ext]
                require(len(matches)==1,'ambiguous pinned dependency '+n+ext)
                src=matches[0]; target=dest/src.name
                shutil.copyfile(src,target);hold(target,retained(src))
        for n in MODULES:
            src=HERE/'candidates'/(n+'.v');target=work/src.name
            require(declarations(src)==freeze['constants'][n],'inventory changed')
            require(not re.search(r'\b(Axiom|Admitted|admit|Parameter|Abort|Program|Obligation|abstract|native_compute|vm_cast_no_check)\b',src.read_text()),'unproved/unchecked construct')
            shutil.copyfile(src,target);hold(target,retained(src))
        for n in MODULES:
            src=work/(n+'.v')
            require(run('compile-'+n,[str(tools['runner']),str(tools['coqc']),'-q',*args,str(src)],env)==0,
                'proof-development/setup failure (not semantic evidence): '+n)
            held.append(ArtifactGuard([src.with_suffix('.vo')]))
        names=[n+'.'+x for n in MODULES for x in freeze['constants'][n]]
        inventory='From NativeSourceReborrow Require Import '+' '.join(MODULES)+'.\n'
        for n in names:
            inventory += f'Redirect "inventory/{n}.assumptions" Print Assumptions NativeSourceReborrow.{n}.\n'
            inventory += f'Redirect "inventory/{n}.statement" Check NativeSourceReborrow.{n}.\n'
        for n in MODULES:inventory += f'Redirect "inventory/{n}.module" Print Module NativeSourceReborrow.{n}.\n'
        target=work/'Assumptions.v';target.write_text(inventory);hold(target,digest(target))
        require(run('assumptions',[str(tools['runner']),str(tools['coqc']),'-q',*args,str(target)],env)==0,'assumptions inspection failed')
        receipt['assumptions']={}
        for n in names:
            require((work/'inventory'/(n+'.assumptions.out')).read_text().strip()=='Closed under the global context',
                'nonempty/unclassified assumptions '+n)
            receipt['assumptions'][n]=[]
        for n in MODULES:
            printed=printed_declarations((work/'inventory'/(n+'.module.out')).read_text())
            require(set(printed)==set(freeze['constants'][n]),'incomplete Coq-discovered inventory '+n+
                ': '+str(set(printed)^set(freeze['constants'][n])))
        products=ArtifactGuard([x for tree in [work,parent,storage] for x in tree.rglob('*') if x.is_file() and x.suffix in ['.v','.vo','.out']])
        require(run('kernel',[str(tools['runner']),str(tools['coqchk']),'-silent','-o',*args,
            *['NativeSourceReborrow.'+n for n in MODULES],'NativeSourceReborrow.Assumptions'],env)==0,'recursive kernel check failed')
        receipt['kernel_axioms']=kernel_summary_no_axioms((a.output/'kernel.stderr').read_text())
        for g in held:g.check()
        products.check();guard.check()
        receipt.update(model_proofs_kernel_checked=True,constant_count=len(names),program_obligations=0,
            proof_artifacts=products.initial,all_final_guards_before_success=True,
            classification='MODEL_conservative_reborrow_not_Rust_certificate')
        print(json.dumps(dict(output=str(a.output),model_proofs_kernel_checked=True,native_correspondences=0)))

if __name__=='__main__':main()
