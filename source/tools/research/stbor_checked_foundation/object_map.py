#!/usr/bin/env python3
"""Normally compile a metadata reader, then map actual checked .vo summaries."""
from pathlib import Path
import json
import os
import shutil
import subprocess

from evidence import ROOT, OWN, sha, save, guard
from run import run

prefix = ROOT/'opamroot/legacy'
os.environ['PATH'] = str(prefix/'bin')+':/usr/bin:/bin'
freeze = ROOT/'manifests/object-controller-freeze.json'
frozen = json.loads(freeze.read_text())
guard(Path('/'), frozen['files'])
directory = ROOT/'object-reader-v3'
directory.mkdir()
source = directory/'vo_summary.ml'
shutil.copyfile(OWN/'vo_summary.ml', source)
if sha(source) != frozen['files'][str(OWN/'vo_summary.ml')]:
    raise SystemExit('metadata-reader source copy mismatch')
command = [str(prefix/'bin/ocamlfind'), 'ocamlopt', '-rectypes', '-thread',
           '-package', 'coq.kernel', '-linkpkg',
           str(source),
           '-o', str(directory/'vo-summary')]
code = run('object-reader-compile-v3', str(directory), command)
if code:
    raise SystemExit(code)
reader_sha = sha(directory/'vo-summary')
loaded = json.loads((ROOT/'manifests/kernel-loaded-objects.json').read_text())
for p, entry in loaded.items():
    if sha(p) != entry['object_sha256']:
        raise SystemExit('checked object guard failed')
command = [str(directory/'vo-summary'), *sorted(loaded)]
code = run('object-reader-query-v3', str(directory), command)
if code:
    raise SystemExit(code)
rows = (ROOT/'logs/object-reader-query-v3.log').read_text().splitlines()
modules = {}
for row in rows:
    name, file, dependencies, imports = row.split('\t')
    if name in modules or file not in loaded:
        raise SystemExit('duplicate/unknown compiled module')
    modules[name] = {**loaded[file], 'object': file,
                     'recorded_compiled_dependencies': dict(x.split('=') for x in dependencies.split(';') if x),
                     'recorded_imports': list(filter(None, imports.split(';')))}
checked = set(json.loads((ROOT/'manifests/kernel-coverage.json').read_text())['checked_modules'])
if set(modules) != checked or len(modules) != len(loaded):
    raise SystemExit('compiled module map differs from kernel coverage')
for name, entry in modules.items():
    if not set(entry['recorded_compiled_dependencies']).issubset(modules):
        raise SystemExit('compiled dependency omitted from checked closure: '+name)
    if sha(entry['object']) != entry['object_sha256']:
        raise SystemExit('checked object changed after metadata query')
guard(Path('/'), frozen['files'])
if sha(directory/'vo-summary') != reader_sha:
    raise SystemExit('metadata reader product guard failed')
save(ROOT/'manifests/object-dependency-map.json', {
    'modules': modules, 'module_count': len(modules),
    'reader': str(directory/'vo-summary'), 'reader_sha256': reader_sha,
    'reader_source_sha256': sha(source), 'controller_freeze_sha256': sha(freeze),
    'scope': 'serialized compiled-library dependency summaries from already fully kernel-checked objects; inspection only'})
print('Mapped actual compiled dependencies for', len(modules), 'kernel-checked source/object pairs')
