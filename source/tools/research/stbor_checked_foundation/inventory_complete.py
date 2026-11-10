#!/usr/bin/env python3
"""Complete compiler-discovered declaration/type/assumption and loaded-object maps.

Additive pre-inventory correction: include record types and all kernel mutual
inductive block names; inventory.py is retained unchanged as a frozen draft."""
from pathlib import Path
import json
import os
import re
import sys

from evidence import ROOT, UPSTREAM, save, sha, upstream_guard, guard
from run import run

prefix = ROOT/'opamroot/legacy'
coqlib = prefix/'lib/coq'
os.environ['PATH'] = str(prefix/'bin')+':/usr/bin:/bin'
os.environ.pop('COQPATH', None)
os.environ.pop('COQLIB', None)
os.environ.pop('OCAMLPATH', None)


def discover():
    receipt = json.loads((ROOT/'receipts/recursive-kernel-check.json').read_text())
    if receipt.get('exit_status') != 0 or receipt.get('state') != 'finished':
        raise SystemExit('no successful complete kernel-check receipt')
    log = Path(receipt['log'])
    if sha(log) != receipt['log_sha256']:
        raise SystemExit('kernel-check log guard failed')
    text = log.read_text()
    checked = set(re.findall(r'^Checking library: (\S+)\s*$', text, re.M))
    if 'Modules were successfully checked' not in text:
        raise SystemExit('kernel completion marker absent')
    objects = {}
    for m in re.finditer(r'\[intern (.*?) \.\.\.', text):
        path = Path(m.group(1))
        if not path.is_absolute():
            path = UPSTREAM/path
        source = path.with_suffix('.v')
        objects[str(path)] = {'object_sha256': sha(path), 'source': str(source),
                              'source_sha256': sha(source) if source.is_file() else None}
    save(ROOT/'manifests/kernel-loaded-objects.json', objects)
    artifact = json.loads((ROOT/'manifests/artifact-modules.json').read_text())
    dependencies = json.loads((ROOT/'manifests/dependency-objects-before-check.json').read_text())
    missing = set(artifact)|set(dependencies)
    missing -= checked
    if missing:
        raise SystemExit('unchecked requested modules: '+repr(sorted(missing)))
    declarations = {}
    for name in sorted(set(re.findall(r'^\s*checking cst:(stbor\.\S+)\s*$', text, re.M))):
        declarations[name] = {'kind': 'kernel_constant'}
    for name in sorted(set(re.findall(r'^\s*checking mutind block: (stbor\.\S+)\s*$', text, re.M))):
        declarations.setdefault(name, {'kind': 'kernel_mutual_inductive_block'})
    for module, entry in artifact.items():
        glob = Path(entry['source']).with_suffix('.glob')
        for line in glob.read_text().splitlines():
            match = re.match(r'^(ind|constr|rec|proj|def|prf|ax|inst|meth) (\d+):(\d+) (\S+) (\S+)$', line)
            if not match:
                continue
            kind, start, end, scope, base = match.groups()
            name = module + ('.'+scope if scope != '<>' else '') + '.' + base
            if kind not in ('ind', 'rec', 'constr') and name not in declarations:
                continue
            declarations.setdefault(name, {'kind': kind})
            declarations[name].update(glob_kind=kind, source=entry['source'],
                                      source_byte_start=int(start), source_byte_end=int(end))
    if not declarations:
        raise SystemExit('empty compiler declaration inventory')
    raw = ROOT/'declaration-output'
    raw.mkdir()
    commands = ['Require ' + ' '.join(sorted(artifact)) + '.',
                'Set Printing Width 1000000.', 'Set Printing All.', 'Set Printing Universes.']
    for i, (name, entry) in enumerate(sorted(declarations.items())):
        entry['index'] = i
        commands += [f'Redirect "{raw}/{i}-type" Check {name}.',
                     f'Redirect "{raw}/{i}-assumptions" Print Assumptions {name}.']
        if entry['kind'] == 'kernel_mutual_inductive_block' or entry.get('glob_kind') in ('ind', 'rec'):
            entry['print_full_declaration'] = True
            commands.append(f'Redirect "{raw}/{i}-declaration" Print {name}.')
    query = ROOT/'declaration-inventory.v'
    query.write_text('\n'.join(commands)+'\n')
    save(ROOT/'manifests/declarations-before-query.json', declarations)
    save(ROOT/'manifests/kernel-coverage.json', {'checked_modules': sorted(checked),
          'artifact_modules': sorted(artifact), 'requested_dependency_modules': sorted(dependencies),
          'constants_and_inductive_constructors': len(declarations),
          'query_sha256': sha(query), 'kernel_log_sha256': sha(log),
          'inventory_controller': {'path': str(Path(__file__).resolve()), 'sha256': sha(Path(__file__))},
          'supplemental_controller_freeze_sha256': sha(ROOT/'manifests/supplemental-controller-freeze.json')})


def query():
    supplemental = json.loads((ROOT/'manifests/supplemental-controller-freeze.json').read_text())
    guard(Path('/'), supplemental['files'])
    code = run('declaration-inventory', str(UPSTREAM),
               [str(prefix/'bin/coqtop'), '-quiet', '-batch', '-coqlib', str(coqlib),
                '-Q', str(UPSTREAM/'theories'), 'stbor',
                '-l', str(ROOT/'declaration-inventory.v')])
    if code:
        raise SystemExit(code)
    declarations = json.loads((ROOT/'manifests/declarations-before-query.json').read_text())
    for name, entry in declarations.items():
        i = entry['index']
        type_path = ROOT/f'declaration-output/{i}-type.out'
        assumption_path = ROOT/f'declaration-output/{i}-assumptions.out'
        tp, ap = type_path.read_text(), assumption_path.read_text()
        if not tp.strip() or not ap.strip() or 'Error:' in tp+ap:
            raise SystemExit('incomplete declaration output: '+name)
        if 'Closed under the global context' not in ap and 'Axioms:' not in ap:
            raise SystemExit('unrecognized assumption output: '+name)
        entry.update(type_text=tp, assumptions_text=ap, type_sha256=sha(type_path),
                     assumptions_sha256=sha(assumption_path))
        if entry.get('print_full_declaration'):
            full = ROOT/f'declaration-output/{i}-declaration.out'
            if not full.read_text().strip() or 'Error:' in full.read_text():
                raise SystemExit('incomplete inductive/record declaration: '+name)
            entry.update(declaration_text=full.read_text(), declaration_sha256=sha(full))
    save(ROOT/'manifests/declaration-inventory.json', declarations)
    upstream_guard()
    guard(Path('/'), supplemental['files'])
    print('Inventoried', len(declarations), 'compiler-discovered declarations including generated constants and constructors')


if sys.argv[1] == 'discover':
    discover()
elif sys.argv[1] == 'query':
    query()
else:
    raise SystemExit('usage: inventory.py discover|query')
