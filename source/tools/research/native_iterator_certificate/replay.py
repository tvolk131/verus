#!/usr/bin/env python3
"""Guarded real-compiler extraction. No successful native-proof publication exists."""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import sys
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, support_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require
from freeze import freeze
from check import inspect
from controls import run_controls
from vc import generate, require_unsat

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--sysroot', type=Path, default=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu'))
    p.add_argument('--solver', type=Path, default=Path('/home/dev/projects/verus/source/z3'))
    a = p.parse_args()
    a.output = a.output.resolve()
    a.output.mkdir(parents=True, exist_ok=False)
    receipt = dict(schema=1, native_bridge_proved=False, commands=[])
    with replay_receipt(a.output, receipt):
        frozen = freeze(a.output / 'inputs.json')
        source_map_path = HERE.parent / 'native_iterator_refinement/source-map.json'
        source_map = json.loads(source_map_path.read_text())
        native_root = a.sysroot / 'lib/rustlib/src/rust'
        for name, expected in source_map['native_files'].items():
            require(digest(native_root / name) == expected, f'pinned Rust source mismatch: {name}')
        selected = HERE.parent / 'buffer_transfer_goal/original-selected.rs'
        transfer_manifest = HERE.parent / 'buffer_transfer_goal/freeze.json'
        target_manifest = HERE.parent / 'mutable_buffer_goal/transfer-freeze.json'
        bundle_manifest = HERE.parent / 'native_iterator_refinement/bundle-identity.json'
        require(digest(selected) == json.loads(transfer_manifest.read_text())['files']['original-selected.rs'], 'upstream selection binding')
        selected_loop = '\n'.join(line.strip() for line in selected.read_text().splitlines())
        original = HERE / 'fixtures/original.rs'
        require(selected_loop in '\n'.join(line.strip() for line in original.read_text().splitlines()), 'original loop changed')
        tools = [a.sysroot / 'bin/rustc', a.sysroot / 'bin/cargo',
                 *a.sysroot.glob('lib/librustc_driver-*.so'),
                 *a.sysroot.glob('lib/rustlib/x86_64-unknown-linux-gnu/lib/libcore-*.rlib'),
                 *a.sysroot.glob('lib/rustlib/x86_64-unknown-linux-gnu/lib/librustc_middle-*.rmeta')]
        inputs = [HERE / name for name in frozen['files']]
        inputs += [source_map_path, selected, transfer_manifest, target_manifest, bundle_manifest, a.output / 'inputs.json',
                   *[native_root / name for name in source_map['native_files']]]
        a.solver = a.solver.resolve()
        require(digest(a.solver) == json.loads(bundle_manifest.read_text())['z3']['sha256'], 'accepted solver mismatch')
        inputs.append(a.solver)
        guard = ArtifactGuard([*tools, *inputs, *support_artifacts()])
        receipt['artifacts'] = guard.initial
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
                   RUSTUP_TOOLCHAIN='1.99.1', RUSTC_BOOTSTRAP='1', CARGO_BUILD_JOBS='2',
                   CARGO_TARGET_DIR=str(a.output / 'target'),
                   LD_LIBRARY_PATH=str(a.sysroot / 'lib'))
        run = LoggedCommands(a.output, ROOT / 'source', receipt['commands'], timeout=240, guard=guard)
        require(run('rustc-version', [str(a.sysroot / 'bin/rustc'), '-vV'], env) == 0, 'rustc identity')
        require('release: 1.99.1' in (a.output / 'rustc-version.stdout').read_text(), 'wrong compiler version')
        require(run('build', [str(a.sysroot / 'bin/cargo'), 'build', '--offline', '--locked',
            '--manifest-path', str(HERE / 'Cargo.toml'), '--config',
            f'build.build-dir="{a.output / "build"}"'], env) == 0, 'extractor build failed')
        binary = a.output / 'target/debug/native-iterator-extract'
        binary_guard = ArtifactGuard([binary])
        receipt['extractor'] = binary_guard.initial
        def extract(name, source, instances=256, operations=100000):
            evidence = a.output / (name + '.json')
            binary_guard.check()
            require(run('extract-' + name, [str(binary), str(evidence), str(instances), str(operations), str(source),
                '--crate-name', 'native_writer', '--edition', '2024', '--sysroot', str(a.sysroot),
                '-Zmir-opt-level=0', '-Coverflow-checks=on'], env) == 0, 'extraction failed')
            binary_guard.check()
            data = json.loads(evidence.read_text())
            require(data['native_bridge_proved'] is False, 'extractor must not certify')
            require(data['input_sha256'] == digest(source), 'tool/source binding mismatch')
            return data
        data = extract('original', original)
        evidence = a.output / 'original.json'
        require(not data['pending_instances'], 'call graph incomplete')
        require(not any(b['kind'].endswith('_budget') for b in data['extraction_blockers']), 'extraction exhausted')
        inspection = inspect(data, digest(original))
        require(not inspection['writer_diagnostics'], 'original writer shape')
        require(len(inspection['conditional_native_value_summaries']) == 2, 'normal native step unsupported')
        require(len(inspection['conditional_constructor_value_summaries']) == 3, 'normal native constructor unsupported')
        receipt['controls'] = run_controls(data, digest(original))
        receipt['source_controls'] = []
        for name, reason in [('changed-write','writer_xor_operation'), ('missing-write','writer_store_inventory'),
                             ('tail-write','writer_store_inventory'), ('helper-effect','writer_call_inventory'),
                             ('drop-effect','drop_effect_requires_proof'), ('wrong-origin','native_value_summary'),
                             ('callback-effect','indirect_call'), ('wrong-rhs','writer_pair_origin')]:
            source = HERE / 'fixtures' / (name + '.rs')
            variant = extract(name, source)
            require(variant['input_sha256'] != data['input_sha256'], 'source variant must honestly rebind')
            result = inspect(variant, digest(source))
            require(reason in result['writer_diagnostics'] or any(b['kind'] == reason for b in result['blockers']), f'{name}: intended effect not rejected')
            receipt['source_controls'].append(dict(case=name, source_sha256=digest(source), evidence_sha256=digest(a.output/(name+'.json')),
                semantic_diagnostic=reason, native_bridge_proved=False))
        repeated = extract('original-repeated', original)
        require(repeated == data, 'non-deterministic extraction')
        for name, ib, ob, reason in [('instance-budget',1,100000,'instance_budget'),('operation-budget',256,1,'operation_budget')]:
            limited = extract(name, original, ib, ob)
            require(any(b['kind'] == reason for b in limited['extraction_blockers']), 'resource exhaustion not explicit')
            receipt['controls'].append(dict(control=name, rejected=True, reason=reason, semantic_negative=False))
        # Human source correspondence is bound separately from compiled core MIR.
        paths = set()
        for instance in data['instances']:
            for text in [instance.get('definition_source',''), *[s['source'] for b in instance.get('blocks',[]) for s in b['statements']]]:
                paths.update(re.findall(r'library/[^:\s]+\.rs', text))
        native_guard = ArtifactGuard([native_root / p for p in paths])
        receipt['native_source_bindings'] = {p:digest(native_root/p) for p in sorted(paths)}
        (a.output / 'inspection.json').write_text(json.dumps(inspection, indent=2) + '\n')
        # Isolated byte-for-byte solver copy; no parent artifact is mutated.
        solver = a.output / 'z3'
        shutil.copy2(a.solver, solver)
        require(digest(solver) == digest(a.solver) and solver.stat().st_ino != a.solver.stat().st_ino, 'private solver copy')
        solver_guard = ArtifactGuard([solver])
        receipt['solver'] = solver_guard.initial
        require(run('solver-version', [str(solver), '--version'], env) == 0, 'solver identity')
        require('4.16.0' in (a.output/'solver-version.stdout').read_text(), 'wrong solver version')
        receipt['backend_vcs'] = []
        for name, content in generate(inspection).items():
            vc = a.output / (name+'.smt2')
            vc.write_text(content)
            vc_guard = ArtifactGuard([vc])
            code = run('vc-'+name, [str(solver), str(vc)], env)
            require_unsat(code, (a.output/('vc-'+name+'.stdout')).read_text(), (a.output/('vc-'+name+'.stderr')).read_text())
            vc_guard.check()
            receipt['backend_vcs'].append(dict(name=name, status='conditional_scalar_only_unsat', sha256=digest(vc), native_refinement=False))
        # False scalar VC must remain satisfiable; no proof status is inferred from exit 0.
        false_vc = a.output/'false-scalar.smt2'
        false_vc.write_text('(set-logic QF_LIA)\n(declare-const x Int)\n(assert (= x 0))\n(assert (not (> x 0)))\n(check-sat)\n')
        require(run('false-scalar', [str(solver),str(false_vc)],env) == 0, 'false scalar control tool failure')
        require((a.output/'false-scalar.stdout').read_text().strip() == 'sat', 'false scalar control unexpectedly proved')
        try:
            require_unsat(0, (a.output/'false-scalar.stdout').read_text(), '')
        except ValueError:
            pass
        else:
            raise ValueError('satisfiable backend proof accepted')
        receipt['controls'].append(dict(control='false-backend-scalar', rejected=True, reason='sat'))
        resource_vc = a.output/'resource-scalar.smt2'
        resource_vc.write_text(next(iter(generate(inspection).values())).replace('(set-logic QF_LIA)', '(set-logic QF_LIA)\n(set-option :rlimit 1)'))
        code = run('resource-scalar', [str(solver), str(resource_vc)], env)
        stdout = (a.output/'resource-scalar.stdout').read_text()
        require(code == 0 and stdout.strip() == 'unknown', 'resource control did not exhaust the solver')
        try:
            require_unsat(code, stdout, (a.output/'resource-scalar.stderr').read_text())
        except ValueError:
            pass
        else:
            raise ValueError('resource-limited backend proof accepted')
        receipt['controls'].append(dict(control='resource-backend-scalar', rejected=True, reason='unknown', semantic_negative=False))
        solver_guard.check()
        native_guard.check()
        summary = dict(native_bridge_proved=False, root=data['root'], target=data['target'],
            pointer_bits=data['pointer_bits'], instances=len(data['instances']), operations=data['operations'],
            extraction_blockers=data['extraction_blockers'],
            calls=[dict(display=i['display'], kind=i['kind'], blocks=len(i.get('blocks', []))) for i in data['instances']],
            evidence_sha256=digest(evidence))
        summary.update(conditional_native_value_summaries=inspection['conditional_native_value_summaries'],
            conditional_constructor_value_summaries=inspection['conditional_constructor_value_summaries'],
            typed_operation_counts=inspection['operation_counts'],
            unproved_generated_obligations=len(inspection['generated_obligations']),
            remaining_unproved=inspection['missing_refinement'],
            backend_vcs=receipt['backend_vcs'], false_certificate_controls=len(receipt['controls']),
            honestly_rebound_source_controls=receipt['source_controls'], ordinary_Verus_proofs=[])
        receipt['summary'] = summary
        (a.output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
        guard.check()

if __name__ == '__main__':
    main()
