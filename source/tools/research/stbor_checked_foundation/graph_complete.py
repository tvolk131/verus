#!/usr/bin/env python3
"""Scan all loaded sources with package-specific original loadpath rules.

The earlier scan is retained as incomplete despite exit zero. -boot includes
standard-library dependencies; per-package roots preserve unqualified Require
resolution (notably Paco's upstream -R . Paco).
"""
from pathlib import Path
import json
import os
import subprocess

from evidence import ROOT, UPSTREAM, save, sha, guard

prefix = ROOT/'opamroot/legacy'
lib = prefix/'lib/coq'
os.environ['PATH'] = str(prefix/'bin')+':/usr/bin:/bin'
stage = ROOT/'manifests/dependency-controller-freeze.json'
frozen = json.loads(stage.read_text())
guard(Path('/'), frozen['files'])
loaded = json.loads((ROOT/'manifests/kernel-loaded-objects.json').read_text())
groups = {name: [] for name in ['Coq', 'stbor', 'stdpp', 'iris', 'Paco', 'Equations']}
for entry in loaded.values():
    source = Path(entry['source'])
    if not source.is_file():
        raise SystemExit('missing source')
    if source.is_relative_to(UPSTREAM):
        group = 'stbor'
    elif source.is_relative_to(lib/'user-contrib'):
        group = source.relative_to(lib/'user-contrib').parts[0]
    else:
        group = 'Coq'
    groups[group].append(str(source))
common = [str(prefix/'bin/coqdep'), '-boot', '-coqlib', str(lib),
          '-R', str(lib/'theories'), 'Coq', '-R', str(lib/'plugins'), 'Coq']
for directory in sorted((lib/'plugins').iterdir()):
    if directory.is_dir():
        common += ['-I', str(directory)]
recipes = {
    'Coq': [],
    'stdpp': ['-Q', str(lib/'user-contrib/stdpp'), 'stdpp'],
    'iris': ['-Q', str(lib/'user-contrib/stdpp'), 'stdpp',
             '-Q', str(lib/'user-contrib/iris'), 'iris'],
    'Paco': ['-R', str(lib/'user-contrib/Paco'), 'Paco'],
    'Equations': ['-R', str(lib/'user-contrib/Equations'), 'Equations',
                  '-I', str(lib/'user-contrib/Equations')],
    'stbor': ['-Q', str(lib/'user-contrib/stdpp'), 'stdpp',
              '-Q', str(lib/'user-contrib/iris'), 'iris',
              '-R', str(lib/'user-contrib/Paco'), 'Paco',
              '-R', str(lib/'user-contrib/Equations'), 'Equations',
              '-I', str(lib/'user-contrib/Equations'),
              '-Q', str(UPSTREAM/'theories'), 'stbor'],
}
commands, rules, raw = [], [], []
for group, sources in groups.items():
    command = common+recipes[group]+sorted(sources)
    result = subprocess.run(command, cwd=UPSTREAM, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, text=True)
    commands.append({'group': group, 'command': command,
                     'exit_status': result.returncode, 'stderr': result.stderr})
    print(group, 'sources', len(sources), 'exit', result.returncode, flush=True)
    if result.returncode or result.stderr.strip():
        print(result.stderr, flush=True)
        save(ROOT/'manifests/dependency-graph-complete-failed-commands.json', commands)
        raise SystemExit('dependency graph is incomplete: diagnostic output')
    raw.append(result.stdout)
    for line in result.stdout.replace('\\\n', ' ').splitlines():
        if not line.strip():
            continue
        if ': ' not in line:
            raise SystemExit('unrecognized dependency rule')
        targets, prerequisites = line.split(': ', 1)
        rules.append({'targets': targets.split(), 'prerequisites': prerequisites.split()})
out = ROOT/'manifests/recursive-source-dependencies-complete.d'
out.write_text(''.join(raw))
guard(Path('/'), frozen['files'])
save(ROOT/'manifests/dependency-graph-complete.json', {'commands': commands,
     'source_count': sum(map(len, groups.values())), 'compiler_dependency_rules': rules,
     'raw_sha256': sha(out), 'controller_sha256': sha(Path(__file__)),
     'controller_freeze_sha256': sha(stage), 'all_diagnostics_empty': True})
print('Complete package-scoped compiler dependency map recorded')
