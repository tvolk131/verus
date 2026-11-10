#!/usr/bin/env python3
"""Bounded platform/OCaml/Coq executable and dynamic-library identities."""
from pathlib import Path
import re
import shutil
import subprocess

from evidence import ROOT, save, sha

tools = {Path(shutil.which(name)).resolve() for name in
         ['gcc', 'ld', 'as', 'make', 'm4', 'patch', 'git', 'curl', 'tar', 'python3', 'bash']}
for name in ['cc1', 'collect2', 'libgcc.a', 'libgcc_s.so.1']:
    path = Path(subprocess.check_output(['gcc', '-print-file-name='+name], text=True).strip())
    if path.is_file():
        tools.add(path.resolve())
tools.update(p.resolve() for p in (ROOT/'opamroot/legacy/bin').iterdir() if p.is_file())
tools.add(ROOT/'downloads/opam-2.3.0-x86_64-linux')
files = {str(p): sha(p) for p in tools}
ldd_outputs = {}
for tool in sorted(tools):
    result = subprocess.run(['ldd', str(tool)], stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT, text=True)
    ldd_outputs[str(tool)] = {'exit_status': result.returncode, 'output': result.stdout}
    for value in re.findall(r'(?:=>\s+|^\s*)(/[^\s()]+)', result.stdout, re.M):
        p = Path(value).resolve()
        if p.is_file():
            files[str(p)] = sha(p)
versions = {}
for command in [['gcc', '--version'], ['ld', '--version'], ['make', '--version'],
                ['ldd', '--version'], ['git', '--version'], ['python3', '--version']]:
    result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    if result.returncode:
        raise SystemExit('tool version failure')
    versions[' '.join(command)] = result.stdout
save(ROOT/'manifests/platform-toolchain.json', {'files': files,
      'dynamic_library_queries': ldd_outputs, 'versions': versions})
print('Pinned', len(files), 'platform/toolchain executable and runtime files')
