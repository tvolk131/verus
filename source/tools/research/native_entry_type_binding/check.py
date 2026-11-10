#!/usr/bin/env python3
"""Compiler typing/query receipts only; never publish a native proof."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, support_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--freeze', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output = a.output.resolve()
    a.output.mkdir(parents=True, exist_ok=False)
    receipt = dict(schema=1, classification='compiler_typing_and_queries_only',
                   native_correspondences=0, publication=False, goal_complete=False,
                   commands=[])
    with replay_receipt(a.output, receipt):
        freeze_bytes = a.freeze.read_bytes()
        frozen = json.loads(freeze_bytes)
        required = {str(x.resolve()) for x in [HERE / n for n in
                    ['check.py', 'query.rs', 'probes.rs', 'INTERFACE.md']]}
        required.update(str(x.resolve()) for x in support_artifacts())
        canonical = ROOT / 'source/tools/research/varint_canonical_goal/candidate-008.rs'
        writer = ROOT / 'source/tools/research/native_iterator_certificate/fixtures/original.rs'
        required.update([str(canonical), str(writer)])
        require(required <= set(frozen['inputs']), 'incomplete frozen source/controller set')
        require(frozen['before_first_trial'] is True, 'missing pretrial declaration')
        guard = ArtifactGuard([Path(x) for x in frozen['inputs']] + [a.freeze])
        for path, expected in frozen['inputs'].items():
            require(guard.initial[str(Path(path).resolve())]['sha256'] == expected,
                    'frozen input differs: ' + path)
        require(guard.initial[str(a.freeze.resolve())]['sha256'] ==
                hashlib.sha256(freeze_bytes).hexdigest(), 'freeze load changed')
        receipt['inputs'] = guard.initial
        sysroot = Path(frozen['sysroot'])
        compiler = sysroot / 'bin/rustc'
        env = dict(os.environ, RUSTC_BOOTSTRAP='1', RUSTUP_TOOLCHAIN='1.99.1',
                   LD_LIBRARY_PATH=str(sysroot / 'lib'),
                   TMPDIR=str(a.output / 'tmp'))
        Path(env['TMPDIR']).mkdir()
        for name in ['RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC_WRAPPER',
                     'RUSTC_WORKSPACE_WRAPPER']:
            env.pop(name, None)
        run = LoggedCommands(a.output, a.output, receipt['commands'], timeout=180, guard=guard)
        require(run('version', [str(compiler), '-vV'], env) == 0, 'compiler identity query')
        version = (a.output / 'version.stdout').read_text()
        require('commit-hash: 58cf8f9c35b355199727627d07ae34a4e2846d67' in version
                and 'release: 1.99.1' in version, 'wrong compiler revision')
        binary = a.output / 'entry-type-query'
        require(run('build-query', [str(compiler), str(HERE / 'query.rs'), '--edition=2024',
                '--crate-name', 'entry_type_query', '--sysroot', str(sysroot),
                '-Cprefer-dynamic', '-o', str(binary)], env) == 0, 'query driver build')
        binary_guard = ArtifactGuard([binary])
        receipt['query_binary'] = binary_guard.initial
        base_args = ['--edition=2024', '--crate-type=lib', '--sysroot', str(sysroot),
                     '-Coverflow-checks=on']
        require(run('ordinary-probes', [str(compiler), str(HERE / 'probes.rs'), *base_args,
                '--emit=metadata', '-o', str(a.output / 'probes.rmeta')], env) == 0,
                'universal trait requirements and constant layouts did not typecheck')
        queries = {}
        for name, source in [('probes', HERE / 'probes.rs'), ('writer', writer),
                             ('canonical', canonical)]:
            output = a.output / (name + '.json')
            args = [str(binary), str(output), str(source), *base_args]
            if name == 'canonical':
                bundle = Path(frozen['verus_bundle'])
                args += ['--extern', 'vstd=' + str(bundle / 'libvstd.rlib'),
                         '-L', 'dependency=' + str(bundle)]
            binary_guard.check()
            require(run('query-' + name, args, env) == 0, 'type query failed: ' + name)
            binary_guard.check()
            data = json.loads(output.read_text())
            require(data['classification'] == 'compiler_type_queries_only'
                    and data['native_correspondences'] == 0
                    and data['publication'] is False, 'query scope')
            queries[name] = data
        records = {name: {(v['function'], v['parameter']): v for v in data['records']}
                   for name, data in queries.items()}
        require(len(records['probes']) == 7 and len(records['writer']) == 2
                and len(records['canonical']) == 2, 'unexpected parameter inventory')
        for table in records.values():
            for v in table.values():
                if v['subject_type'] in ['u8', '[u8; 0]', '[u8; 10]', '[u8; 128]', '[u8]']:
                    require(all(v[x] is True for x in ['unpin', 'unsafe_unpin', 'freeze']),
                            'byte-type trait classification')
        require(records['writer'][('xor_key', 0)]['subject_type'] == '[u8; 128]'
                and records['writer'][('xor_key', 0)]['reference_kind'] == 'Mut'
                and records['writer'][('xor_key', 1)]['subject_type'] == '[u8]',
                'unchanged writer input types')
        require(records['canonical'][('u64', 1)]['subject_type'] == '[u8; 10]'
                and records['canonical'][('u64', 1)]['reference_kind'] == 'Mut',
                'unchanged canonical buffer input')
        for table, key, size in [('writer', ('xor_key', 0), 128),
                                 ('canonical', ('u64', 1), 10)]:
            v = records[table][key]
            require(v['sized'] is True and v['static_size'] == size and v['align'] == 1,
                    'actual byte-array layout')
        for table, key in [('writer', ('xor_key', 1)), ('probes', ('probe_slice', 0))]:
            v = records[table][key]
            require(v['sized'] is False and v['static_size'] is None and v['align'] == 1,
                    'slice minimum layout mistaken for static size')
        require(records['probes'][('probe_pinned', 0)]['unpin'] is False
                and records['probes'][('probe_pinned', 0)]['unsafe_unpin'] is False
                and records['probes'][('probe_cell', 0)]['freeze'] is False,
                'negative compiler-query discriminators')
        receipt['queries'] = queries
        receipt['ordinary_rejections'] = []
        for name, trait, ty in [('pinned-unpin', 'Unpin', 'std::marker::PhantomPinned'),
                                ('pinned-unsafe-unpin', 'UnsafeUnpin', 'std::marker::PhantomPinned'),
                                ('cell-freeze', 'Freeze', 'std::cell::UnsafeCell<u8>')]:
            source = a.output / (name + '.rs')
            source.write_text('#![feature(freeze, unsafe_unpin)]\n'
                              'use std::marker::{Unpin, UnsafeUnpin, Freeze};\n'
                              f'fn need<T: {trait}>() {{}}\n'
                              f'pub fn check() {{ need::<{ty}>(); }}\n')
            rc = run(name, [str(compiler), str(source), *base_args, '--crate-name',
                          'negative_trait', '--emit=metadata', '-o', str(a.output / (name + '.rmeta'))], env)
            err = (a.output / (name + '.stderr')).read_text()
            require(rc == 1 and 'error[E0277]' in err and trait in err,
                    'expected ordinary trait rejection: ' + name)
            receipt['ordinary_rejections'].append(dict(name=name, trait=trait, diagnostic='E0277'))
        binary_guard.check()
        guard.check()
        receipt.update(all_final_guards_passed=True,
                       universal_arrays_and_unsized_slice_typechecked=True,
                       runtime_permissions_derived=False, actual_source_bridge_proved=False)


if __name__ == '__main__':
    main()
