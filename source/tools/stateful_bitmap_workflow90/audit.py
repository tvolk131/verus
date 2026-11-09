"""Frozen source-bound projections; not a general Rust equivalence proof."""

import hashlib
import importlib.util
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
from check_redb_stateful import NATIVE_SHA256
spec = importlib.util.spec_from_file_location('reference88_audit', HERE.parent / 'stateful_bitmap_reference88/audit.py')
reference = importlib.util.module_from_spec(spec)
spec.loader.exec_module(reference)

HASHES = {
    'native_manual': 'b5d9378b8444263a34abd5b24a180eb134494d9e85328491a95552d8d222a98e',
    'native_ablated': '61d0c463c03f7cec55177028e45d07430c28dd7fa5e0931a959309b4221e2634',
    'tree_manual': '16d217b972c5437fbbbf75bcd3bebca767e77817f7577fe859b1214b65b347cc',
    'tree_ablated': '02765d6fb299fcd47d1e85b1d8d974db14fb6c097df870b116b9024c6b46d7ef',
}

def require(condition, message):
    if not condition:
        raise ValueError(message)

def erase_tree(body):
    result, erased = [], []
    position = 0
    while position < len(body):
        if body[position:position + 2] == ['proof', '{']:
            end = reference.block_end(body, position + 1) + 1
        elif body[position:position + 2] == ['let', 'ghost']:
            end = body.index(';', position) + 1
        elif body[position:position + 2] == ['loop', 'invariant_except_break']:
            boundaries = [offset for offset in range(position, len(body))
                          if body[offset:offset + 5] == ['{', 'let', 'ghost', 'before', '=']]
            require(len(boundaries) == 1, 'ambiguous tree loop boundary')
            end = boundaries[0]
            result.append('loop')
        else:
            result.append(body[position])
            position += 1
            continue
        erased.append({'start': position, 'end': end, 'tokens': body[position:end]})
        position = end
    return result, erased

def audit(native_path, fixtures=HERE / 'fixtures'):
    for name, expected in HASHES.items():
        path = Path(fixtures) / (name + '.rs')
        require(hashlib.sha256(path.read_bytes()).hexdigest() == expected, 'frozen fixture drift: ' + name)
    require(hashlib.sha256(Path(native_path).read_bytes()).hexdigest() == NATIVE_SHA256,
            'wrong pinned native source')
    native = Path(native_path).read_text()
    native_tokens = reference.tokens(native)
    result = {}
    for name, expected in HASHES.items():
        path = Path(fixtures) / (name + '.rs')
        require(hashlib.sha256(path.read_bytes()).hexdigest() == expected, 'frozen fixture drift: ' + name)
        text = path.read_text()
        if name.startswith('native_'):
            row = reference.audit(native, text)
            require(row['all_nine_bodies'] and row['exact_struct'], 'native A projection mismatch')
        else:
            items = reference.tokens(text)
            rows = {}
            for owner, routine in [('BtreeBitmap', 'new'), ('U64GroupedBitmap', 'required_words'), ('U64GroupedBitmap', 'new_full')]:
                original_impl = reference.extract(native_tokens, ['impl', owner])
                candidate_impl = reference.extract(items, ['impl', owner])
                original = reference.extract(original_impl, ['fn', routine, '('])
                candidate = reference.extract(candidate_impl, ['fn', routine, '('])
                projected, erased = erase_tree(candidate)
                require(original == projected, 'tree native projection mismatch: ' + routine)
                rows[routine] = dict(native_tokens=original, projected_tokens=projected, erased=erased)
            for owner in ('BtreeBitmap', 'U64GroupedBitmap'):
                require(reference.extract(native_tokens, ['struct', owner]) == reference.extract(items, ['struct', owner]), 'tree struct mismatch: ' + owner)
            row = dict(bodies=rows, exact_structs=True)
        result[name] = dict(sha256=expected, projection=row)
    return result

if __name__ == '__main__':
    print(json.dumps(audit(Path(sys.argv[1])), sort_keys=True, indent=2))
