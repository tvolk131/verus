"""Narrow native-body token checks, not a general Rust equivalence theorem."""

import argparse
import hashlib
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
from stateful_bitmap_reference88.audit import block_end, extract, tokens


def require(condition, message):
    if not condition:
        raise ValueError(message)


def erase_checked_blocks(body):
    result, erased = [], []
    position = 0
    while position < len(body):
        if body[position:position + 2] == ['proof', '{']:
            end = block_end(body, position + 1) + 1
            erased.append(body[position:end])
            position = end
        else:
            result.append(body[position])
            position += 1
    return result, erased


def normalize_field_visibility(items):
    result, position = [], 0
    while position < len(items):
        if items[position:position + 4] == ['pub', '(', 'crate', ')']:
            result.append('pub')
            position += 4
        else:
            result.append(items[position])
            position += 1
    return result


def audit(native):
    native = Path(native)
    bindings = [
        ('src/tree_store/page_store/header.rs', 'native_codec_probe.rs', None, ['get_u32', 'get_u64']),
        ('src/tree_store/page_store/base.rs', 'native_page_codec.rs', 'PageNumber',
         ['serialized_size', 'to_le_bytes', 'from_le_bytes']),
        ('src/tree_store/btree_base.rs', 'native_btree_header_codec.rs', 'BtreeHeader',
         ['serialized_size', 'to_le_bytes', 'from_le_bytes']),
    ]
    rows = []
    for relative, fixture, owner, methods in bindings:
        original_path = native / relative
        fixture_path = HERE / fixture
        original = tokens(original_path.read_text())
        candidate = tokens(fixture_path.read_text())
        original_scope = extract(original, ['impl', owner]) if owner else original
        candidate_scope = extract(candidate, ['impl', owner]) if owner else candidate
        bodies = {}
        for name in methods:
            expected = extract(original_scope, ['fn', name, '('])
            actual = extract(candidate_scope, ['fn', name, '('])
            projection, erased = erase_checked_blocks(actual)
            require(expected == projection, 'native body differs: ' + str(owner) + '::' + name)
            bodies[name] = dict(native_tokens=expected, projected_tokens=projection,
                                erased_checked_proof_blocks=erased)
        struct = None
        if owner:
            struct = normalize_field_visibility(extract(original, ['struct', owner]))
            require(struct == extract(candidate, ['struct', owner]), 'native fields differ: ' + owner)
        rows.append(dict(native_source=relative, fixture=fixture,
            native_sha256=hashlib.sha256(original_path.read_bytes()).hexdigest(),
            fixture_sha256=hashlib.sha256(fixture_path.read_bytes()).hexdigest(),
            fields=struct, bodies=bodies))
    return dict(rows=rows, native_bodies=8, checked_native_crash_correctness=False,
                limitations=['body tokens only; contracts and type/call linkage require separate review',
                             'pub(crate) field visibility projected to pub for ordinary proof imports',
                             'BtreeHeader Debug derive excluded with zero formatting/destructor credit'])


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('native', type=Path)
    arguments = parser.parse_args()
    print(json.dumps(audit(arguments.native), sort_keys=True, indent=2))


if __name__ == '__main__':
    main()
