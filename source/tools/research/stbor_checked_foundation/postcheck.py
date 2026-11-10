#!/usr/bin/env python3
"""Record concrete tool configuration/loadpaths and compiler dependency graph."""
from pathlib import Path
import json
import os
import subprocess
import sys

from evidence import ROOT, UPSTREAM, save, sha

prefix = ROOT/'opamroot/legacy'
os.environ['PATH'] = str(prefix/'bin')+':/usr/bin:/bin'
for name in ['COQLIB', 'COQPATH', 'OCAMLPATH']:
    os.environ.pop(name, None)


if sys.argv[1] == 'tools':
    commands = [[str(prefix/'bin'/name), *args] for name, args in [
        ('coqc', ['--version']), ('coqchk', ['-v']), ('coqtop', ['--version']),
        ('coqc', ['-config']), ('ocamlc', ['-version']), ('ocamlc', ['-config']),
        ('ocamlfind', ['printconf']), ('ocamlfind', ['list'])]]
    commands += [[str(ROOT/'downloads/opam-2.3.0-x86_64-linux'), 'list',
                  '--installed', '--columns=name,version', '--color=never']]
    artifact = json.loads((ROOT/'manifests/artifact-modules.json').read_text())
    query = ROOT/'loadpath-inventory.v'
    query.write_text('Require '+' '.join(sorted(artifact))+'.\nPrint LoadPath.\nPrint Libraries.\n')
    commands += [[str(prefix/'bin/coqtop'), '-quiet', '-batch', '-Q',
                  str(UPSTREAM/'theories'), 'stbor', '-l', str(query)]]
    outputs = []
    for command in commands:
        result = subprocess.run(command, cwd=UPSTREAM, stdout=subprocess.PIPE,
                                stderr=subprocess.STDOUT, text=True)
        print(json.dumps({'command': command, 'exit_status': result.returncode}))
        print(result.stdout)
        outputs.append({'command': command, 'exit_status': result.returncode,
                        'output': result.stdout})
        if result.returncode:
            raise SystemExit(result.returncode)
    save(ROOT/'manifests/toolchain-query.json', {'commands': outputs,
          'loadpath_query_sha256': sha(query), 'cwd': str(UPSTREAM),
          'environment': {k: os.environ.get(k) for k in
                          ['PATH', 'COQPATH', 'COQLIB', 'OCAMLPATH', 'OPAMROOT', 'OPAMSWITCH']}})
elif sys.argv[1] == 'graph':
    loaded = json.loads((ROOT/'manifests/kernel-loaded-objects.json').read_text())
    sources = sorted({entry['source'] for entry in loaded.values()})
    if any(not Path(source).is_file() for source in sources):
        raise SystemExit('missing source for compiler dependency graph')
    command = [str(prefix/'bin/coqdep'), '-Q', str(UPSTREAM/'theories'), 'stbor', *sources]
    result = subprocess.run(command, cwd=UPSTREAM, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, text=True)
    print(result.stderr)
    if result.returncode:
        raise SystemExit(result.returncode)
    out = ROOT/'manifests/recursive-source-dependencies.d'
    out.write_text(result.stdout)
    # Keep every compiler-produced rule; both normal .vo and optional .vio
    # targets in coqdep output are syntax, not claims that .vio was built.
    rules = []
    for line in result.stdout.replace('\\\n', ' ').splitlines():
        if not line.strip():
            continue
        if ': ' not in line:
            raise SystemExit('unrecognized compiler dependency rule')
        targets, prerequisites = line.split(': ', 1)
        rules.append({'targets': targets.split(), 'prerequisites': prerequisites.split()})
    if not rules:
        raise SystemExit('empty compiler dependency graph')
    save(ROOT/'manifests/dependency-graph.json', {'command': command,
          'exit_status': result.returncode, 'stderr': result.stderr,
          'compiler_dependency_rules': rules, 'raw_sha256': sha(out),
          'source_count': len(sources)})
    print('Recorded compiler dependency rules for', len(sources), 'loaded sources')
else:
    raise SystemExit('usage: postcheck.py tools|graph')
