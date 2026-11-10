#!/usr/bin/env python3
"""Resolve and freeze all proof dependency payloads before compilation."""
from pathlib import Path
import hashlib
import json
import subprocess

root = Path('/tmp/stbor-checked-foundation-20261110')
opam = root/'downloads/opam-2.3.0-x86_64-linux'
packages = ['camlp5.7.08', 'num.1.3', 'ocamlfind.1.8.1', 'coq.8.9.1',
            'coq-stdpp.dev.2019-11-22.2.16eb222f',
            'coq-iris.dev.2019-12-20.0.62798412', 'coq-paco.4.0.0',
            'coq-equations.1.2.1+8.9']
for package in packages:
    dest = root/'preflight-sources'/package
    if not dest.exists():
        subprocess.run([str(opam), 'source', '--dir='+str(dest), package], check=True)
    metadata = subprocess.check_output([str(opam), 'show', '--raw', package])
    (root/'manifests'/('package-'+package+'.opam')).write_bytes(metadata)
    files = {str(p.relative_to(dest)): hashlib.sha256(p.read_bytes()).hexdigest()
             for p in sorted(dest.rglob('*')) if p.is_file() and '.git' not in p.parts}
    (root/'manifests'/('preflight-'+package+'.json')).write_text(
        json.dumps({'package': package, 'files': files}, indent=2)+'\n')
print('All dependency sources extracted and frozen; no Coq compilation requested')
