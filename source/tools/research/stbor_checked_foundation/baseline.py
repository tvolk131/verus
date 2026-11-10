#!/usr/bin/env python3
"""Normally build all upstream modules, then check their complete closure.

Readiness is not emitted by this command. Subsequent inventories and guards
must bind its successful durable receipts before a baseline can be ready.
"""
from pathlib import Path
import json
import os
import shlex
import subprocess
import sys

from evidence import ROOT, UPSTREAM, save, sha, upstream_guard
from run import run

prefix = ROOT/'opamroot/legacy'
coqlib = prefix/'lib/coq'
env_path = str(prefix/'bin') + ':/usr/bin:/bin'
os.environ['PATH'] = env_path
for variable in ('COQPATH', 'COQLIB', 'OCAMLPATH'):
    os.environ.pop(variable, None)
os.environ['TMPDIR'] = str(ROOT/'tmp')
os.environ['XDG_CACHE_HOME'] = str(ROOT/'cache')


def project():
    tokens = shlex.split((UPSTREAM/'_CoqProject').read_text(), comments=True)
    sources = [t for t in tokens if t.endswith('.v')]
    if len(sources) != 44 or len(set(sources)) != 44:
        raise SystemExit('unexpected upstream project coverage')
    return {('stbor.'+str(Path(s).relative_to('theories').with_suffix('')).replace('/', '.')): s
            for s in sources}


if sys.argv[1] == 'build':
    upstream_guard()
    sources = project()
    # Fresh .vo files are required; no copied inspection products.
    if list(UPSTREAM.rglob('*.vo')):
        raise SystemExit('refuse preexisting upstream proof products')
    code = run('upstream-build', str(UPSTREAM), ['/usr/bin/make', '-j2'])
    upstream_guard()
    if code:
        raise SystemExit(code)
    entries = {}
    for module, source in sources.items():
        obj = UPSTREAM/Path(source).with_suffix('.vo')
        if not obj.is_file():
            raise SystemExit('missing artifact module: '+module)
        entries[module] = {'source': str(UPSTREAM/source), 'source_sha256': sha(UPSTREAM/source),
                           'object': str(obj), 'object_sha256': sha(obj)}
    actual = {str(p.relative_to(UPSTREAM)) for p in UPSTREAM.rglob('*.vo')}
    if actual != {str(Path(s).with_suffix('.vo')) for s in sources.values()}:
        raise SystemExit('unexpected extra/missing artifact module')
    save(ROOT/'manifests/artifact-modules.json', entries)
elif sys.argv[1] == 'check':
    upstream_guard()
    sources = project()
    modules = dict(json.loads((ROOT/'manifests/artifact-modules.json').read_text()))
    for module, entry in modules.items():
        if sha(entry['object']) != entry['object_sha256']:
            raise SystemExit('object guard failed: '+module)
    # Check all installed third-party dependency modules too; imported Coq
    # libraries are recursively checked, with no -norec/-admit exclusions.
    dependencies = {}
    for p in sorted((coqlib/'user-contrib').rglob('*.vo')):
        relative = p.relative_to(coqlib/'user-contrib')
        module = '.'.join(relative.with_suffix('').parts)
        dependencies[module] = {'object': str(p), 'object_sha256': sha(p)}
    save(ROOT/'manifests/dependency-objects-before-check.json', dependencies)
    args = [str(prefix/'bin/coqchk'), '-coqlib', str(coqlib),
            '-Q', str(UPSTREAM/'theories'), 'stbor', '-o']
    args += sorted(set(modules)|set(dependencies))
    code = run('recursive-kernel-check', str(UPSTREAM), args)
    upstream_guard()
    for module, entry in {**modules, **dependencies}.items():
        if sha(entry['object']) != entry['object_sha256']:
            raise SystemExit('object guard failed after check: '+module)
    if code:
        raise SystemExit(code)
else:
    raise SystemExit('usage: baseline.py build|check')
