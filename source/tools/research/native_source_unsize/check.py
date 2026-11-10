#!/usr/bin/env python3
"""Normal kernel replay of carrier MODEL laws. Never a native certificate."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
STORAGE = HERE.parent/'native_source_storage'
GOAL = HERE.parent/'mutable_buffer_goal'
BRIDGE = HERE.parent/'native_semantic_bridge'
sys.path[:0] = [str(STORAGE), str(ROOT/'source/tools'), str(BRIDGE), str(GOAL)]
from infer_eval_support import ArtifactGuard, digest, support_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require
from goal_scope import check_goal_snapshot, goal_scope_sha256
spec = importlib.util.spec_from_file_location('storage_model_checker', STORAGE/'check.py')
storage_check = importlib.util.module_from_spec(spec)
spec.loader.exec_module(storage_check)
sys.path.insert(0, str(HERE))
from source_binding import GRAPH, source_text

BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
READY = BASE/'rocq-toolchain/baseline-ready.json'
MODULES = ['Carrier', 'Root']


def inputs():
    return [HERE/n for n in ['check.py', 'source_binding.py', 'pins.json', 'INTERFACE.md', 'README.md']] + [
        HERE/'candidates'/(n+'.v') for n in MODULES] + [
        STORAGE/'check.py', STORAGE/'source.py', STORAGE/'pins.json',
        STORAGE/'integration-results.json', STORAGE/'candidates/Storage.v',
        GRAPH, HERE.parent/'native_constructor_erasure/candidates/Interface.v',
        HERE.parent/'native_constructor_erasure/candidates/Graph.v',
        GOAL/'goal_scope.py', GOAL/'foundation-baseline.json',
        BRIDGE/'baseline-bindings.json', BRIDGE/'source-bindings.json',
        HERE/'test_source_binding.py', HERE/'source-controls-freeze.json', *support_artifacts()]


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--freeze', type=Path, required=True)
    p.add_argument('--create-freeze', action='store_true')
    p.add_argument('--output', type=Path)
    a = p.parse_args()
    if a.create_freeze:
        require(not a.freeze.exists(), 'freeze exists')
        a.freeze.write_text(json.dumps(dict(before_first_trial=True,
            goal_requirements_sha256=goal_scope_sha256((GOAL/'TARGET.md').read_bytes()),
            files={str(f.relative_to(ROOT)):digest(f) for f in inputs()},
            declarations={n:storage_check.declarations(HERE/'candidates'/(n+'.v')) for n in MODULES}), indent=2)+'\n')
        return
    require(a.output is not None, 'output required')
    a.output = a.output.resolve(); a.output.mkdir(parents=True, exist_ok=False)
    receipt = dict(schema=1, commands=[], publication=False, goal_complete=False,
        native_correspondences=0, actual_source_instance=False, source_certificate_proved=False,
        model_proofs_kernel_checked=False, source_simulations_open=44,
        phase_obligations_open=3, writer_effects_open=101)
    with replay_receipt(a.output, receipt):
        loaded = {}
        def load(path):
            loaded[path] = path.read_bytes(); return json.loads(loaded[path])
        freeze=load(a.freeze); pins=load(HERE/'pins.json'); ready=load(READY)
        trust=load(GOAL/'foundation-baseline.json'); binding=load(BRIDGE/'baseline-bindings.json')
        baseline_path=Path(ready['receipt']); baseline=load(baseline_path)
        parent_record=load(STORAGE/'integration-results.json')
        parent_path=Path(parent_record['integration_receipt']['path']); parent=load(parent_path)
        gmp=BASE/'rocq-toolchain/gmp-private/manifest.json'; gmp_data=load(gmp)
        storage_pins=load(STORAGE/'pins.json')
        source_bindings=load(BRIDGE/'source-bindings.json')
        native_path=Path('/tmp/native-semantic-bridge-e4a8dcab/source-v1/drop-native.json')
        native=load(native_path)
        lib=Path(storage_pins['coq_library_dependency_pin']['root'])
        libraries=sorted([*(lib/'coq/theories').rglob('*.v'), *(lib/'coq/theories').rglob('*.vo'),
                          *(lib/'coq-core/plugins').rglob('*.cmxs')])
        tools={n:Path(ready['switch'])/'_opam/bin'/n for n in ['coqc','coqchk']}
        tools.update({n:Path(ready[n]) for n in ['runner','opam']})
        parent_files=[Path(f) for f in parent['proof_artifacts'] if Path(f).name in
            ['Interface.v','Interface.vo','Graph.v','Graph.vo','Storage.v','Storage.vo']]
        require(len(parent_files)==6, 'ambiguous parent proof artifacts')
        target=GOAL/'TARGET.md'; target_snapshot=target.read_bytes()
        paths=[*inputs(), a.freeze, READY, baseline_path, parent_path, gmp, target,
               *tools.values(), *libraries, *parent_files,
               *[Path(x['path']) for x in pins['rust_sources']],
               *[Path(x) for x in pins['completed_extraction']],
               *[Path(x['archive']) for x in gmp_data['packages']]]
        guard=ArtifactGuard(paths)
        def retained(path): return guard.initial[str(path.resolve())]['sha256']
        receipt['goal_requirements_sha256']=check_goal_snapshot(target_snapshot,retained(target),
            freeze.get('goal_requirements_sha256'))
        for path,data in loaded.items():
            require(retained(path)==hashlib.sha256(data).hexdigest(), 'input changed during load '+str(path))
        require(freeze['before_first_trial'] is True, 'missing preregistration')
        require(set(freeze['files'])=={str(f.relative_to(ROOT)) for f in inputs()}, 'freeze incomplete')
        for f,h in freeze['files'].items(): require(retained(ROOT/f)==h, 'frozen source changed '+f)
        require(retained(READY)==trust['readiness']['sha256'] and
            retained(baseline_path)==trust['receipt']['sha256'], 'baseline receipt identity')
        require(baseline['success'] is True and ready['baseline_kernel_checked'] is True and
            ready['source_commit']==binding['source_commit']=='115a618dd35ff8337112cf2081a7f5391d3973d5' and
            baseline_path==BASE/'rocq-toolchain/durable-build-2/receipt.json', 'baseline not ready')
        require(retained(gmp)==trust['toolchain']['GMP_packages']['sha256']==
            '332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37', 'GMP pin')
        for x in gmp_data['packages']: require(retained(Path(x['archive']))==x['sha256'], 'GMP archive')
        for n,path in tools.items(): require(retained(path)==binding['tools'][n], 'tool changed '+n)
        library_map={str(f.relative_to(lib)):retained(f) for f in libraries}
        require(len(library_map)==1121==storage_pins['coq_library_dependency_pin']['count'] and
            hashlib.sha256(json.dumps(library_map,sort_keys=True,separators=(',',':')).encode()).hexdigest()==
            storage_pins['coq_library_dependency_pin']['map_sha256'], 'Coq library pin')
        require(retained(parent_path)==parent_record['integration_receipt']['sha256']==pins['storage_receipt_sha256'] and
            parent['success'] is True and parent['model_proofs_kernel_checked'] is True,
            'unauthenticated storage model')
        for f in parent_files:
            require(retained(f)==parent['proof_artifacts'][str(f)]['sha256'], 'parent object changed')
        for x in pins['rust_sources']:
            require(retained(Path(x['path']))==x['sha256'] and x['url']==
                'https://raw.githubusercontent.com/rust-lang/rust/58cf8f9c35b355199727627d07ae34a4e2846d67/'+x['upstream'],
                'pinned Rust implementation guidance changed')
        require(pins['compiler_commit']=='58cf8f9c35b355199727627d07ae34a4e2846d67', 'compiler identity')
        for path,h in pins['completed_extraction'].items():
            require(retained(Path(path))==h, 'completed extraction changed '+path)
        extraction=native_path.parent
        require(retained(extraction/'receipt.json')==source_bindings['receipt_sha256'], 'source receipt identity')
        for name,h in source_bindings['outputs'].items():
            require(retained(extraction/name)==h, 'source extraction identity '+name)
        require(native['pointer_bits']==64 and native['target']=='x86_64-unknown-linux-gnu' and
            native['native_bridge_proved'] is False and len(native['instances'])==6 and len(native['operations'])==44,
            'unsupported source layout/inventory')
        from source_binding import EXPECTED
        root=native['instances'][0]
        require(root['id']==native['root'] and root['blocks'][0]['bb']==0 and
            root['blocks'][0]['statements'][4]['ir']==EXPECTED, 'original typed source operation differs')
        require(digest(GRAPH)==pins['graph_sha256'] and
            source_text(json.loads(GRAPH.read_text()))==(HERE/'candidates/Root.v').read_text(), 'source binding differs')
        receipt['inputs']=guard.initial
        work=a.output/'proofs'; work.mkdir(); (work/'inventory').mkdir()
        ctor=a.output/'constructor'; ctor.mkdir()
        storage=a.output/'storage'; storage.mkdir()
        args=['-Q',str(ctor),'NativeConstructorErasure','-Q',str(storage),'NativeSourceStorage',
              '-Q',str(work),'NativeSourceUnsize']
        run=LoggedCommands(a.output,work,receipt['commands'],timeout=1800,guard=guard)
        held=[]; parent_sources=[]
        for name,folder in [('Interface',ctor),('Graph',ctor),('Storage',storage)]:
            src=next(f for f in parent_files if f.name==name+'.v')
            dst=folder/src.name; shutil.copyfile(src,dst)
            require(digest(dst)==retained(src), 'parent copy changed')
            held.append(ArtifactGuard([dst])); parent_sources.append(dst)
        for n in MODULES:
            src=HERE/'candidates'/(n+'.v')
            require(storage_check.declarations(src)==freeze['declarations'][n], 'inventory changed '+n)
            require(not re.search(r'\b(Axiom|Admitted|admit|Parameter|Abort|Program|Obligation|abstract|native_compute|vm_cast_no_check)\b',src.read_text()),
                    'unchecked proof construct')
            dst=work/src.name; shutil.copyfile(src,dst)
            require(digest(dst)==retained(src), 'candidate copy changed')
            held.append(ArtifactGuard([dst]))
        for src in parent_sources+[work/(n+'.v') for n in MODULES]:
            require(run('compile-'+src.stem,[str(tools['runner']),str(tools['coqc']),'-q',*args,str(src)],dict(os.environ))==0,
                    'proof-development/setup failure, not semantic evidence: '+src.stem)
            held.append(ArtifactGuard([src.with_suffix('.vo')]))
        names=[m+'.'+n for m in MODULES for n in freeze['declarations'][m]]
        inventory='From NativeSourceUnsize Require Import '+' '.join(MODULES)+'.\n'
        for name in names:
            inventory+=f'Redirect "inventory/{name}.assumptions" Print Assumptions NativeSourceUnsize.{name}.\n'
            inventory+=f'Redirect "inventory/{name}.statement" Check NativeSourceUnsize.{name}.\n'
        for m in MODULES: inventory+=f'Redirect "inventory/{m}.module" Print Module NativeSourceUnsize.{m}.\n'
        inv=work/'Assumptions.v'; inv.write_text(inventory); held.append(ArtifactGuard([inv]))
        require(run('inventory',[str(tools['runner']),str(tools['coqc']),'-q',*args,str(inv)],dict(os.environ))==0,
                'inventory compilation failure')
        for name in names:
            require((work/'inventory'/(name+'.assumptions.out')).read_text().strip()=='Closed under the global context',
                    'nonclosed or unclassified assumptions '+name)
        for m in MODULES:
            require(set(storage_check.printed_declarations((work/'inventory'/(m+'.module.out')).read_text()))==
                set(freeze['declarations'][m]), 'compiler-discovered inventory mismatch '+m)
        for src in parent_sources:
            prior=next(f for f in parent_files if f.name==src.with_suffix('.vo').name)
            require(digest(src.with_suffix('.vo'))==retained(prior), 'parent mathematical object differs')
        products=ArtifactGuard([f for tree in [work,ctor,storage] for f in tree.rglob('*')
                               if f.is_file() and f.suffix in ['.v','.vo','.out']])
        require(run('kernel',[str(tools['runner']),str(tools['coqchk']),'-silent','-o',*args,
            *['NativeSourceUnsize.'+m for m in MODULES],'NativeSourceUnsize.Assumptions'],dict(os.environ))==0,
            'recursive kernel check failed')
        receipt['kernel_axioms']=storage_check.kernel_summary_no_axioms((a.output/'kernel.stderr').read_text())
        for g in held: g.check()
        products.check(); guard.check()
        receipt.update(model_proofs_kernel_checked=True, source_declarations=len(names),
            assumptions={name:[] for name in names}, program_obligations=0,
            proof_artifacts=products.initial, all_final_guards_before_success=True,
            classification='unvalidated_array_slice_carrier_MODEL_not_complete_Rust_operation')
        print(json.dumps(dict(output=str(a.output),closed_declarations=len(names),native_correspondences=0)))


if __name__=='__main__': main()
