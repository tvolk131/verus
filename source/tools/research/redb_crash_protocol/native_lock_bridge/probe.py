#!/usr/bin/env python3
"""One frozen, bounded feasibility experiment; never installs a native fact."""
import argparse
import json
import os
from pathlib import Path
import shutil
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import (LoggedCommands, replay_receipt, require, validate_run,
                             validate_goal_run, require_function_coverage)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args(); out = args.output.resolve(); out.mkdir(parents=True, exist_ok=False)
    receipt = dict(kind='AUTHORIZED_SINGLE_NATIVE_LOCK_BRIDGE_FEASIBILITY', commands=[], outcomes=[],
                   native_bridge_proved=False, production_facts_added=0, source_coupling_proved=False)
    with replay_receipt(out, receipt):
        freeze = json.loads((HERE / 'FROZEN.json').read_text())
        for path, value in freeze['inputs'].items(): require(digest(Path(path)) == value, 'input '+path)
        tc = Path(freeze['toolchain']); runtime = freeze['runtime']; src = Path(runtime['bundle'])
        require(digest(src / 'receipt.json') == runtime['receipt_sha256'], 'accepted runtime receipt')
        bundle = out / 'bundle'; bundle.mkdir()
        for name, value in runtime['tool_hashes'].items():
            require(digest(src / name) == value, 'accepted tool '+name)
            shutil.copy2(src / name, bundle / name); require(digest(bundle / name) == value, 'private tool')
        shutil.copy2(src / 'verus-root', bundle / 'verus-root')
        negative = (HERE / 'permission_control.rs').read_text().replace('ensures result == value,', 'ensures result != value,')
        wrong = out / 'wrong_permission_value.rs'; wrong.write_text(negative)
        require(digest(wrong) == freeze['wrong_permission_value_sha256'], 'frozen false control')
        guard = ArtifactGuard([HERE / 'FROZEN.json', wrong] + [Path(p) for p in freeze['inputs']] +
                              [bundle / name for name in runtime['tool_hashes']])
        run = LoggedCommands(out, out, receipt['commands'], timeout=120, guard=guard)
        env = {k:v for k,v in os.environ.items() if k not in
               ['RUSTFLAGS', 'RUSTDOCFLAGS', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'RUSTC']}
        env.update(PATH='/home/dev/.cargo/bin:'+str(tc/'bin')+':/usr/bin:/bin',
                   RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN=str(tc),
                   LD_LIBRARY_PATH=str(tc/'lib'), VERUS_Z3_PATH=str(bundle/'z3'))
        require(run('compiler_identity', [str(tc/'bin/rustc'), '-vV'], env) == 0, 'compiler identity')
        require('release: 1.99.1' in (out/'compiler_identity.stdout').read_text(), 'pinned compiler')
        native = out / 'native'; native.mkdir()
        require(run('native_mir', [str(tc/'bin/rustc'), str(HERE/'native.rs'), '--crate-type=lib',
            '--crate-name=lock_bridge_probe', '--edition=2024', '--emit=mir,obj', '-Copt-level=2',
            '-Coverflow-checks=on', '-Zmir-opt-level=3', '-Zprint-type-sizes', '--out-dir', str(native)],
            dict(env,RUSTC_BOOTSTRAP='1')) == 0, 'ordinary native type/borrow/MIR compilation')
        mir = native/'lock_bridge_probe.mir'
        require(mir.is_file(), 'actual native MIR output')
        receipt['native_products'] = {str(p):digest(p) for p in native.iterdir() if p.is_file()}
        text = mir.read_text()
        receipt['native_mir_markers'] = {name: name in text for name in
            ['native_handoff', 'native_cell_pointer', 'UnsafeCell', 'MutexGuard', 'drop', 'DerefMut']}
        # Markers are observations, never completeness/native-refinement claims.
        cases = [('permission_control',HERE/'permission_control.rs',True),
                 ('wrong_permission_value',wrong,False),
                 ('native_lock',HERE/'checked.rs',None),
                 ('native_cell_dependency',HERE/'cell_pointer.rs',None)]
        for name, source, expected in cases:
            for infer in [False,True]:
                label = name + ('_infer' if infer else '_ordinary')
                argv = [str(bundle/'verus'),str(source),'--crate-type=lib','--edition=2024',
                        '--no-cheating','--output-json','--time']
                if infer: argv.append('--infer-goals')
                code = run(label,argv,dict(env,VERUS_INFER_CACHE=str(out/('cache_'+label))))
                stderr = (out/(label+'.stderr')).read_text()
                evidence = dict(returncode=code,stdout=(out/(label+'.stdout')).read_text(),stderr=stderr)
                if expected is not None:
                    data = validate_goal_run(evidence,expected,expected_functions=set(),
                        exec_bodies={'existing_permission_control'},proof_bodies=set(),
                        failure_kind=None if expected else 'postcondition not satisfied')
                    require(data['verification-results']['errors'] == (0 if expected else 1), 'control scope')
                else:
                    data = validate_run(evidence,False,allow_frontend=True)
                    require(data['verification-results'].get('encountered-vir-error') is True,
                            'frontier must be classified as frontend, not semantic failure')
                    term = 'Mutex' if name=='native_lock' else 'UnsafeCell'
                    require(term in stderr and 'is not supported' in stderr, 'exact native dependency')
                    require_function_coverage(data.get('verus-infer',{}).get('functions',[]),[])
                receipt['outcomes'].append(dict(label=label,data=data,stderr_sha256=digest(out/(label+'.stderr'))))
                print(label,data['verification-results'],flush=True)
        guard.check(); receipt['final_guard_passed']=True
        receipt['stop_reason']='Native UnsafeCell access/permission interpretation is not supplied by existing nominal/source-body facilities; no such foundation implemented.'

if __name__=='__main__': main()
