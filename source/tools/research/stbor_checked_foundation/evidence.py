#!/usr/bin/env python3
"""Content inventories and explicit guards for the private baseline.

This tool never declares native validity and never creates a readiness receipt.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path('/tmp/stbor-checked-foundation-20261110')
OWN = Path(__file__).resolve().parent
UPSTREAM = ROOT / 'src' / 'stacked-borrows'


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def save(p, obj):
    if p.exists():
        raise SystemExit('refuse overwrite: ' + str(p))
    p.write_text(json.dumps(obj, sort_keys=True, indent=2) + '\n')


def inventory(path):
    return {str(p.relative_to(path)): sha(p) for p in sorted(path.rglob('*'))
            if p.is_file()}


def guard(path, files):
    bad = [f for f, h in files.items() if not (path/f).is_file() or sha(path/f) != h]
    if bad:
        raise SystemExit('guard failed: ' + repr(bad))
    return {'matched': len(files), 'mismatches': []}


def upstream_guard():
    return guard(UPSTREAM, json.loads((OWN/'upstream.json').read_text())['tracked_source_sha256'])


def freeze(label, prefix):
    save(ROOT/'manifests'/label, {
        'upstream': upstream_guard(), 'controller': inventory(OWN),
        'installed_prefix': str(prefix), 'installed_files': inventory(prefix),
        'downloads': inventory(ROOT/'downloads'),
        'dependency_initial_sources': inventory(ROOT/'preflight-sources'),
        'dependency_build_sources': inventory(ROOT/'source-snapshots'),
        'opam_config_sha256': sha(ROOT/'opamroot'/'config'),
    })


if __name__ == '__main__':
    if sys.argv[1] == 'upstream-guard':
        print(json.dumps(upstream_guard()))
    elif sys.argv[1] == 'freeze':
        freeze(sys.argv[2], Path(sys.argv[3]))
    elif sys.argv[1] == 'guard':
        f = json.loads((ROOT/'manifests'/sys.argv[2]).read_text())
        result = {'upstream': upstream_guard(),
                  'installed': guard(Path(f['installed_prefix']), f['installed_files']),
                  'dependency_initial_sources': guard(ROOT/'preflight-sources', f['dependency_initial_sources']),
                  'dependency_build_sources': guard(ROOT/'source-snapshots', f['dependency_build_sources']),
                  'downloads': guard(ROOT/'downloads', f['downloads'])}
        print(json.dumps(result))
    else:
        raise SystemExit('unknown command')
