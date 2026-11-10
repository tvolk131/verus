#!/usr/bin/env python3
"""Authenticate pinned package-feature source facts; no native theorem."""
import argparse
import hashlib
import json
from pathlib import Path
import tomllib


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    own = Path(__file__).resolve().parent
    p = argparse.ArgumentParser()
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=False)
    result = dict(success=False, classification='primary_source_feature_inspection_only',
                  native_correspondences=0, publication=False, goal_complete=False)
    try:
        pin_bytes = (own / 'pins.json').read_bytes()
        pins = json.loads(pin_bytes)
        frozen = json.loads((own / 'freeze.json').read_bytes())
        for n, digest in frozen['files'].items():
            require(sha(own / n) == digest, 'frozen inspector input differs: ' + n)
        require(hashlib.sha256(pin_bytes).hexdigest() == frozen['files']['pins.json'],
                'pin changed during load')
        original = {}
        for n, item in pins['sources'].items():
            path = Path(item['path'])
            require(sha(path) == item['sha256'], 'source identity differs: ' + n)
            body = path.read_bytes()
            blob = hashlib.sha1(b'blob ' + str(len(body)).encode() + b'\0' + body).hexdigest()
            require(blob == item['git_blob'], 'Git blob differs: ' + n)
            original[str(path)] = item['sha256']
        cargo = tomllib.loads(Path(pins['sources']['miri_cargo']['path']).read_text())
        require(cargo['package']['name'] == 'miri', 'wrong package')
        features = cargo['features']
        require(features['default'] == ['stack-cache', 'native-lib'], 'default features differ')
        require(features['stack-cache'] == [], 'stack-cache declaration differs')
        require(features['expensive-consistency-checks'] == ['stack-cache'],
                'consistency-check dependency differs')
        fx = Path(pins['sources']['fx_reexport']['path']).read_text()
        require('pub use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet, FxHasher};' in fx,
                'FxHashSet re-export differs')
        for path, digest in original.items():
            require(sha(path) == digest, 'inspected source changed')
        for n, digest in frozen['files'].items():
            require(sha(own / n) == digest, 'inspector changed')
        result.update(success=True, source_hashes=original,
                      package_features=features,
                      cache_enabled_by_package_default=True,
                      actual_build_flags_observed=False,
                      fx_underlying_hashset_or_hasher_semantics_proved=False,
                      runtime_or_native_correspondence_proved=False)
    except BaseException as error:
        result['failure'] = dict(type=type(error).__name__, message=str(error))
        raise
    finally:
        (a.output / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')


if __name__ == '__main__':
    main()
