"""Selected native body/field correspondence; not whole-crate equivalence."""

import argparse
import hashlib
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from audit_codecs import block_end, erase_checked_blocks, extract, require, tokens


def without_visibility(items):
    result = []
    position = 0
    while position < len(items):
        if items[position] == 'pub':
            position += 1
            if position < len(items) and items[position] == '(':
                while items[position] != ')':
                    position += 1
                position += 1
        else:
            result.append(items[position])
            position += 1
    return result


def tuple_field(items, name):
    start = next(index for index in range(len(items) - 2)
                 if items[index:index + 3] == ['struct', name, '(']) + 3
    end = items.index(')', start)
    return without_visibility(items[start:end])


def method_body(items, name):
    prefix = ['fn', name, '(']
    positions = [index for index in range(len(items)) if items[index:index + 3] == prefix]
    require(len(positions) == 1, 'ambiguous method')
    stack = []
    matching = {')': '(', ']': '[', '}': '{'}
    for position in range(positions[0] + 2, len(items)):
        item = items[position]
        if item == '{' and not stack:
            return items[position + 1:block_end(items, position)]
        if item in ('(', '[', '{'):
            stack.append(item)
        elif item in matching:
            require(stack and stack.pop() == matching[item], 'unbalanced method declaration')
    raise ValueError('method body missing')


def audit(native):
    fixture_path = HERE / 'native_header_slots.rs'
    candidate = tokens(fixture_path.read_text())
    rows = []
    for relative, owner, methods in [
        ('src/transaction_tracker.rs', 'TransactionId', ['new', 'raw_id', 'next', 'increment']),
        ('src/tree_store/page_store/header.rs', 'TransactionHeader', ['new']),
        ('src/tree_store/page_store/header.rs', 'DatabaseHeader',
         ['primary_slot', 'secondary_slot', 'published_slot', 'write_secondary_slot', 'swap_primary_slot']),
    ]:
        native_path = native / relative
        original = tokens(native_path.read_text())
        native_scope = extract(original, ['impl', owner])
        candidate_scope = extract(candidate, ['impl', owner])
        bodies = {}
        for name in methods:
            expected = method_body(native_scope, name)
            actual, erased = erase_checked_blocks(method_body(candidate_scope, name))
            require(actual == expected, owner + '::' + name + ' body mismatch')
            bodies[name] = dict(tokens=expected, erased_checked_blocks=erased)
        if owner == 'TransactionId':
            fields = tuple_field(original, owner)
            require(fields == tuple_field(candidate, owner), 'transaction identifier representation mismatch')
        else:
            fields = without_visibility(extract(original, ['struct', owner]))
            require(fields == without_visibility(extract(candidate, ['struct', owner])), owner + ' fields mismatch')
        rows.append(dict(native_source=relative, owner=owner, fields=fields, bodies=bodies,
                         native_sha256=hashlib.sha256(native_path.read_bytes()).hexdigest()))
    return dict(rows=rows, selected_native_bodies=10,
        fixture_sha256=hashlib.sha256(fixture_path.read_bytes()).hexdigest(),
        limits=['body tokens and field types; signatures and call/type identity require separate review',
                'field and method visibility widened for ordinary proof imports',
                'three generated Clone bodies separately reported; non-Copy Clone has no semantic contract',
                'TransactionId ordering/hash/debug derives outside selected scope',
                'identifier exhaustion and valid primary-slot bounds are unproved caller premises',
                'no header serialization, native I/O, publication or recovery acceptance'])


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('native', type=Path)
    arguments = parser.parse_args()
    print(json.dumps(audit(arguments.native), sort_keys=True, indent=2))


if __name__ == '__main__':
    main()
