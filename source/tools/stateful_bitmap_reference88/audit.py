"""Independent token comparison after a narrow, enumerated proof erasure."""

import argparse
import hashlib
import json
from pathlib import Path
import re

TOKEN = re.compile(r'//[^\n]*|/\*.*?\*/|"(?:\\.|[^"\\])*"|[A-Za-z_][A-Za-z_0-9]*|[0-9]+|[^\s]', re.S)
METHODS = ['required_words', 'data_index_of', 'select_mask', 'len', 'resize', 'get', 'set', 'clear']


def tokens(text):
    return [match.group() for match in TOKEN.finditer(text)
            if not match.group().startswith(('//', '/*'))]


def block_end(items, start):
    depth = 1
    for position in range(start + 1, len(items)):
        if items[position] == '{':
            depth += 1
        elif items[position] == '}':
            depth -= 1
            if depth == 0:
                return position
    raise ValueError('unterminated block')


def extract(items, prefix):
    positions = [position for position in range(len(items))
                 if items[position:position + len(prefix)] == prefix]
    if len(positions) != 1:
        raise ValueError('ambiguous declaration: ' + str(prefix))
    start = items.index('{', positions[0] + len(prefix))
    return items[start + 1:block_end(items, start)]


def erase(body):
    result, erased = [], []
    position = 0
    while position < len(body):
        if body[position:position + 2] == ['proof', '{']:
            end = block_end(body, position + 1) + 1
            erased.append(dict(kind='checked proof block', tokens=body[position:end]))
        elif body[position:position + 2] == ['let', 'ghost']:
            end = body.index(';', position) + 1
            erased.append(dict(kind='ghost snapshot', tokens=body[position:end]))
        elif body[position:position + 5] == ['for', 'word', 'in', 'cursor', ':']:
            header = ['for', 'word', 'in', 'start_word', '.', '.', '=', 'end_word']
            invariant = body.index('invariant', position)
            if body[position + 5:invariant] != header[3:]:
                raise ValueError('changed native iterator')
            candidates = [offset for offset in range(invariant, len(body))
                          if body[offset:offset + 4] == ['{', 'let', 'ghost', 'before']]
            if len(candidates) != 1:
                raise ValueError('ambiguous loop-body boundary')
            end = candidates[0]
            erased.append(dict(kind='checked loop invariants and cursor binding',
                               tokens=body[position:end]))
            result.extend(header)
        else:
            result.append(body[position])
            position += 1
            continue
        position = end
    return result, erased


def audit(native, reference):
    native_items, reference_items = tokens(native), tokens(reference)
    original_impl = extract(native_items, ['impl', 'U64GroupedBitmap'])
    candidate_impl = extract(reference_items, ['impl', 'U64GroupedBitmap'])
    rows = {}
    for name in ['bits_in_range'] + METHODS:
        original = extract(native_items if name == 'bits_in_range' else original_impl, ['fn', name, '('])
        candidate = extract(reference_items if name == 'bits_in_range' else candidate_impl, ['fn', name, '('])
        projection, erased = erase(candidate)
        rows[name] = dict(equal=projection == original, native_tokens=original,
                          projected_tokens=projection, erased=erased,
                          native_sha256=hashlib.sha256(json.dumps(original).encode()).hexdigest())
    original_struct = extract(native_items, ['struct', 'U64GroupedBitmap'])
    candidate_struct = extract(reference_items, ['struct', 'U64GroupedBitmap'])
    return dict(all_nine_bodies=all(row['equal'] for row in rows.values()),
                exact_struct=original_struct == candidate_struct,
                struct_tokens=candidate_struct, bodies=rows,
                limitations='Source-level audited erasure, not a general Rust erasure tool.')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('native', type=Path)
    parser.add_argument('reference', type=Path)
    parser.add_argument('output', type=Path)
    arguments = parser.parse_args()
    report = audit(arguments.native.read_text(), arguments.reference.read_text())
    arguments.output.write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps(dict(all_nine_bodies=report['all_nine_bodies'], exact_struct=report['exact_struct'])))
    if not report['all_nine_bodies'] or not report['exact_struct']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
