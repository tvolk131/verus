#!/usr/bin/env python3
"""Emit a receipt-bound EXTERNAL MODEL gate only after real checked evidence."""
from pathlib import Path
import json
import re
import subprocess

from evidence import ROOT, UPSTREAM, OWN, guard, save, sha, upstream_guard


def successful(name):
    path = ROOT/'receipts'/(name+'.json')
    value = json.loads(path.read_text())
    if value.get('state') != 'finished' or value.get('exit_status') != 0:
        raise SystemExit('unsuccessful required trial: '+name)
    if sha(value['log']) != value['log_sha256']:
        raise SystemExit('log guard failed: '+name)
    return {'path': str(path), 'sha256': sha(path), 'command': value['command'],
            'exit_status': value['exit_status'], 'log_sha256': value['log_sha256'],
            'runner_sha256': value['runner_sha256']}


receipts = {name: successful(name) for name in
            ['switch-create-4071', 'release-authentication',
             'dependency-downloads-authenticated', 'dependency-preflight-after-refresh',
             'install-frozen-dependencies', 'upstream-build',
             'recursive-kernel-check', 'declaration-inventory', 'dependency-graph',
             'toolchain-query']}
kernel = receipts['recursive-kernel-check']['command']
if any(flag in kernel for flag in ['-admit', '-norec', '-silent']):
    raise SystemExit('kernel-check exclusion or suppressed coverage')
if '-o' not in kernel:
    raise SystemExit('missing kernel context inventory')

frozen = json.loads((ROOT/'manifests/upstream-tool-freeze.json').read_text())
guards = {'upstream': upstream_guard(),
          'installed_toolchain': guard(Path(frozen['installed_prefix']), frozen['installed_files']),
          'initial_dependency_payloads': guard(ROOT/'preflight-sources', frozen['dependency_initial_sources']),
          'dependency_build_entry_snapshots': guard(ROOT/'source-snapshots', frozen['dependency_build_sources']),
          'downloads': guard(ROOT/'downloads', frozen['downloads'])}
# Controllers are guarded through the trials; reports/results are written later.
guards['trial_controller'] = guard(OWN, frozen['controller'])
platform = json.loads((ROOT/'manifests/platform-toolchain.json').read_text())
guards['platform_tools_and_runtime'] = guard(Path('/'), platform['files'])

artifact = json.loads((ROOT/'manifests/artifact-modules.json').read_text())
dependencies = json.loads((ROOT/'manifests/dependency-objects-before-check.json').read_text())
coverage = json.loads((ROOT/'manifests/kernel-coverage.json').read_text())
loaded = json.loads((ROOT/'manifests/kernel-loaded-objects.json').read_text())
declarations = json.loads((ROOT/'manifests/declaration-inventory.json').read_text())
expected = set(artifact)|set(dependencies)
if len(artifact) != 44 or not expected.issubset(coverage['checked_modules']):
    raise SystemExit('incomplete kernel-check module coverage')
for entry in list(artifact.values())+list(dependencies.values()):
    if sha(entry['object']) != entry['object_sha256']:
        raise SystemExit('checked object changed')
for path, entry in loaded.items():
    if sha(path) != entry['object_sha256'] or not entry['source_sha256']:
        raise SystemExit('incomplete/changed loaded source/object pair: '+path)
    if sha(entry['source']) != entry['source_sha256']:
        raise SystemExit('loaded source changed: '+entry['source'])
known_sources = set(json.loads((OWN/'upstream.json').read_text())['tracked_source_sha256'].values())
for path in (ROOT/'manifests').glob('preflight-*.json'):
    known_sources.update(json.loads(path.read_text())['files'].values())
for entry in loaded.values():
    if entry['source_sha256'] not in known_sources:
        raise SystemExit('loaded source not bound to initial source freeze: '+entry['source'])
if len(declarations) != coverage['constants_and_inductive_constructors']:
    raise SystemExit('incomplete per-declaration assumption inventory')
for name, entry in declarations.items():
    for suffix in ['type', 'assumptions']:
        out = ROOT/f'declaration-output/{entry["index"]}-{suffix}.out'
        if sha(out) != entry[suffix+'_sha256']:
            raise SystemExit('declaration output guard failed: '+name)
    if not entry['type_text'].strip() or not entry['assumptions_text'].strip():
        raise SystemExit('empty declaration inventory: '+name)

hooks = {}
for path in sorted((ROOT/'receipts').glob('hook-*.json')):
    value = json.loads(path.read_text())
    if value.get('state') != 'finished' or value.get('exit_status') != 0 or value.get('formal_source_guard') != 'matched':
        raise SystemExit('unsuccessful package build hook: '+str(path))
    if 'log' in value and sha(value['log']) != value['log_sha256']:
        raise SystemExit('changed package build log')
    hooks[str(path)] = sha(path)

maps = {}
for name in ['pretrial-dependencies.json', 'release-authentication.json',
             'upstream-tool-freeze.json', 'artifact-modules.json',
             'dependency-objects-before-check.json', 'kernel-loaded-objects.json',
             'kernel-coverage.json', 'declaration-inventory.json',
             'dependency-graph.json', 'toolchain-query.json', 'platform-toolchain.json']:
    path = ROOT/'manifests'/name
    maps[name] = {'path': str(path), 'sha256': sha(path)}
tools = {name: {'path': str(ROOT/'opamroot/legacy/bin'/name),
                'sha256': sha(ROOT/'opamroot/legacy/bin'/name)}
         for name in ['coqc', 'coqchk', 'coqtop', 'coqdep', 'coq_makefile', 'ocamlc', 'ocamlopt']}
kernel_log = Path(json.loads((ROOT/'receipts/recursive-kernel-check.json').read_text())['log']).read_text()
context = kernel_log.split('CONTEXT SUMMARY', 1)[-1]
receipt = {'schema': 1, 'state': 'completed_checked_external_model',
           'upstream_commit': 'd1471b10fe4cb4f506371b2e3acdd927d0a0c076',
           'build_and_check_receipts': receipts, 'package_build_receipt_sha256': hooks,
           'tools': tools, 'maps': maps, 'final_guards': guards,
           'artifact_module_count': len(artifact),
           'checked_module_count': len(coverage['checked_modules']),
           'declaration_count': len(declarations),
           'kernel_context_summary': context,
           'scope': 'external historical model, relative to inventoried upstream axioms',
           'native_correspondences': 0, 'publication': False, 'goal_complete': False}
receipt_path = ROOT/'completed-build-receipt.json'
save(receipt_path, receipt)
# Receipt exists durably and has been re-read before readiness is emitted.
receipt_digest = sha(receipt_path)
if json.loads(receipt_path.read_text()) != receipt:
    raise SystemExit('completed receipt reload mismatch')
save(ROOT/'baseline-ready.json', {
    'schema': 1, 'state': 'checked_external_model', 'fully_ready': True,
    'completed_receipt': str(receipt_path), 'completed_receipt_sha256': receipt_digest,
    'upstream_commit': receipt['upstream_commit'], 'tools': tools, 'maps': maps,
    'loadpaths_and_versions': maps['toolchain-query.json'],
    'scope': receipt['scope'], 'normal_compilation_succeeded': True,
    'recursive_kernel_check_succeeded': True, 'native_correspondences': 0,
    'publication': False, 'goal_complete': False})
print('Receipt-bound checked external-model baseline ready:', receipt_digest)
