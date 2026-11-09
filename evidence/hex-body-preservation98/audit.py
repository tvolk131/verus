#!/usr/bin/env python3
"""AUDIT/METADATA only: independently compare frozen source/scope identities."""

import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import tarfile
import time


INPUT = Path('/private/tmp/verus-hex130-privacy-repair-20261107-zm5hKz')
REVIEW = Path('/private/tmp/verus-hex130-independent-review-20261107-7xnzpY')
ARCHIVE = Path('/Users/tommyvolk/.t3/worktrees/verus/t3-60978068/evidence/verus-infer-hex-v3/native-source.tar.gz')
HISTORICAL_ROOT = '/Users/tommyvolk/Documents/GitHub/hex-conservative'
MANIFEST_SHA256 = '3ef7d0e40cf0150e40ee071bfd1629e8f440f5246b42633708ca80206f5c8d62'
ARCHIVE_SHA256 = 'e7f4b63eced039d437b54858fc600071b41614aa66b2a9fdc27caa94c9e0de5b'
REVIEW_PINS = {
    'REVIEW.md': '72e536659fb1cf25bb974a088885d29dec14d4eea8d362130e2cb4b244f6b6f8',
    'independent-evidence.json': 'eb253b6b0d15dd6ca3341b71436a88ef86df36bacc30fa831996b6505bb86ea3',
}
ADDED_IMPLEMENTATIONS = {
    '<Case as core::clone::Clone>::clone', 'Case::table',
    'encoding_spec::canonical_byte_encoding_goal', 'table::Table::byte_to_hex_chars',
}
OVERLOAD_PREFIX = '<error::DecodeVariableLengthBytesError as core::convert::From<error::'
OVERLOADS = {
    OVERLOAD_PREFIX + 'InvalidCharError>>::from': (150, 5, 150, 41),
    OVERLOAD_PREFIX + 'OddLengthStringError>>::from': (157, 5, 157, 45),
}
SPAN_PATTERN = re.compile(r'(.+):(\d+):(\d+): (\d+):(\d+)\Z')


class AuditError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise AuditError(message)


def digest(content):
    return hashlib.sha256(content).hexdigest()


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON key: ' + key)
        result[key] = value
    return result


def read_json(path):
    return json.loads(path.read_bytes(), object_pairs_hook=unique_object)


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def safe_relative(name):
    require(isinstance(name, str) and bool(name), 'missing relative file')
    relative = PurePosixPath(name)
    require(not relative.is_absolute() and '..' not in relative.parts
            and relative.as_posix() == name, 'noncanonical relative file: ' + name)
    return name


def regular_files(root):
    require(root.is_dir() and not root.is_symlink(), 'unsafe root: ' + str(root))
    found = {}
    for directory, directories, files in os.walk(root, followlinks=False):
        for name in directories:
            require(not (Path(directory) / name).is_symlink(), 'symlink directory')
        for name in files:
            path = Path(directory) / name
            require(stat.S_ISREG(path.lstat().st_mode), 'not a regular file: ' + str(path))
            found[path.relative_to(root).as_posix()] = path
    return found


def authenticate(input_root, review_root, archive_path, phase):
    manifest_path = input_root / 'output-manifest.json'
    manifest_bytes = manifest_path.read_bytes()
    require(digest(manifest_bytes) == MANIFEST_SHA256, 'manifest pin mismatch')
    manifest = json.loads(manifest_bytes, object_pairs_hook=unique_object)
    require(isinstance(manifest, dict) and len(manifest) == 143, 'manifest cardinality')
    found = regular_files(input_root)
    require(set(found) == set(manifest) | {'output-manifest.json'}, 'input path inventory mismatch')
    files = []
    for name, expected in sorted(manifest.items()):
        safe_relative(name)
        require(isinstance(expected, dict) and set(expected) == {'bytes', 'sha256'}, 'manifest record schema')
        content = found[name].read_bytes()
        actual = {'bytes': len(content), 'sha256': digest(content)}
        require(actual == expected, 'frozen file mismatch: ' + name)
        files.append({'path': name, **actual, 'match': True})
    external = []
    for path, expected in [(archive_path, ARCHIVE_SHA256)] + [
            (review_root / name, pin) for name, pin in REVIEW_PINS.items()]:
        require(stat.S_ISREG(path.lstat().st_mode), 'unsafe external file')
        content = path.read_bytes()
        require(digest(content) == expected, 'external pin mismatch: ' + str(path))
        external.append({'path': str(path), 'bytes': len(content), 'sha256': expected})
    return {
        'phase': phase, 'utc': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
        'manifest': {'path': str(manifest_path), 'bytes': len(manifest_bytes), 'sha256': MANIFEST_SHA256},
        'listed_files': len(files), 'regular_files_including_manifest': len(found),
        'unlisted_files': 0, 'symlinks_or_special_files': 0,
        'all_match': True, 'files': files, 'external_inputs': external,
    }


def source_trees(archive_path, native_root):
    original = {}
    with tarfile.open(archive_path, 'r:gz') as archive:
        total = 0
        for member in archive.getmembers():
            safe_relative(member.name)
            require(member.isdir() or member.isfile(), 'unsupported archive member')
            if member.isfile():
                require(member.name not in original, 'duplicate archive member')
                total += member.size
                require(total < 16 * 1024 * 1024, 'archive exceeds bounded read budget')
                original[member.name] = archive.extractfile(member).read()
    final = {name: path.read_bytes() for name, path in regular_files(native_root).items()}
    require(len(original) == 53, 'unexpected archive file count')
    validate_sources(original, final)
    return original, final


def validate_sources(original, final):
    require(set(final) == set(original) | {'src/encoding_spec.rs'}, 'unexpected added/removed source file')
    for name, content in original.items():
        if name != 'src/lib.rs':
            require(final[name] == content, 'archived file modified: ' + name)
    return LineMap(original['src/lib.rs'], final['src/lib.rs'])


class LineMap:
    def __init__(self, original, final):
        self.original = original.splitlines(keepends=True)
        self.final = final.splitlines(keepends=True)
        self.earliest = []
        cursor = 0
        for line in self.original:
            while cursor < len(self.final) and self.final[cursor] != line:
                cursor += 1
            require(cursor < len(self.final), 'unsupported edit: original is not an insertion-only subsequence')
            self.earliest.append(cursor + 1)
            cursor += 1
        self.latest = []
        cursor = len(self.final) - 1
        for line in reversed(self.original):
            while cursor >= 0 and self.final[cursor] != line:
                cursor -= 1
            require(cursor >= 0, 'mapping failure: no reverse embedding')
            self.latest.append(cursor + 1)
            cursor -= 1
        self.latest.reverse()

    def map_span(self, span):
        validate_span(self.original, span)
        start, column, end, end_column = span
        indices = range(start - 1, end)
        require(all(self.earliest[index] == self.latest[index] for index in indices),
                'ambiguous span identity: multiple insertion embeddings')
        mapped = self.earliest[start - 1:end]
        require(mapped == list(range(mapped[0], mapped[0] + len(mapped))),
                'unmappable span: insertion inside source identity')
        return mapped[0], column, mapped[-1], end_column

    def receipt(self):
        return {
            'algorithm': 'exact byte-line insertion subsequence; earliest/latest embeddings must agree on every span line',
            'original_lines': len(self.original), 'final_lines': len(self.final),
            'inserted_lines': len(self.final) - len(self.original),
            'line_map': [
                {'original': index + 1, 'earliest_current': early, 'latest_current': late, 'unique': early == late}
                for index, (early, late) in enumerate(zip(self.earliest, self.latest))
            ],
        }


def validate_span(lines, span):
    start, column, end, end_column = span
    require(1 <= start <= end <= len(lines), 'span line out of bounds')
    require(1 <= column <= len(lines[start - 1].rstrip(b'\r\n')) + 1, 'span start column out of bounds')
    require(1 <= end_column <= len(lines[end - 1].rstrip(b'\r\n')) + 1, 'span end column out of bounds')
    require(start != end or column < end_column, 'empty or reversed span')


def parse_source(source, expected_root, sources):
    require(isinstance(source, str), 'missing source span')
    match = SPAN_PATTERN.fullmatch(source)
    require(match is not None, 'malformed source span')
    filename = match.group(1)
    prefix = expected_root + '/'
    require(filename.startswith(prefix), 'source root mismatch: ' + filename)
    relative = safe_relative(filename[len(prefix):])
    require(relative.startswith('src/') and relative in sources, 'unknown source file: ' + relative)
    span = tuple(int(value) for value in match.groups()[1:])
    validate_span(sources[relative].splitlines(keepends=True), span)
    return relative, span


def checked_rows(scope, expected_exec, expected_labels, expected_proof):
    require(isinstance(scope, dict), 'missing verification scope')
    require(scope.get('native_collection_completed') is True
            and scope.get('complete_selected_scope_verified') is True, 'incomplete selected scope')
    require(type(scope.get('checked_exec_bodies')) is int
            and scope['checked_exec_bodies'] == expected_exec, 'wrong declared exec count')
    require(type(scope.get('checked_proof_bodies')) is int
            and scope['checked_proof_bodies'] == expected_proof, 'wrong declared proof count')
    rows = scope.get('native_functions')
    require(isinstance(rows, list), 'missing native_functions')
    for row in rows:
        require(isinstance(row, dict), 'invalid scope row')
        for field in ('function', 'rust_function', 'source', 'body_status'):
            require(isinstance(row.get(field), str) and bool(row[field].strip()), 'missing field: ' + field)
        require(type(row.get('selected')) is bool, 'missing selected field')
    checked = [(index, row) for index, row in enumerate(rows) if row['body_status'] == 'checked_exec_body']
    proofs = [row for row in rows if row['body_status'] == 'checked_proof_body']
    require(len(checked) == expected_exec and len(proofs) == expected_proof, 'wrong actual scope counts')
    require(all(row['selected'] for _, row in checked) and all(row['selected'] for row in proofs), 'unchecked selection flag')
    require(len({row['function'] for _, row in checked}) == expected_labels, 'wrong function-label diagnostic count')
    return checked


def identities(rows, root, sources, mapping=None):
    identified = []
    for index, row in rows:
        filename, span = parse_source(row['source'], root, sources)
        normalized = mapping.map_span(span) if mapping is not None and filename == 'src/lib.rs' else span
        key = (row['rust_function'], filename, normalized)
        identified.append((key, index, row, span))
    counts = Counter(item[0] for item in identified)
    require(all(count == 1 for count in counts.values()), 'duplicate identity/multiplicity')
    require(all(count == 1 for count in Counter(item[0][0] for item in identified).values()),
            'duplicate implementation/multiplicity')
    return identified, counts


def compare_scopes(historical, current, original, final, current_root):
    mapping = validate_sources(original, final)
    old_rows = checked_rows(historical, 32, 31, 0)
    new_rows = checked_rows(current, 36, 35, 1)
    old, old_counts = identities(old_rows, HISTORICAL_ROOT, original, mapping)
    new, new_counts = identities(new_rows, current_root, final)
    missing = old_counts - new_counts
    require(not missing, 'missing historical identity/multiplicity: ' + repr(list(missing)))
    extras = new_counts - old_counts
    require(sum(extras.values()) == 4 and {key[0] for key in extras} == ADDED_IMPLEMENTATIONS,
            'unexpected added implementation inventory')
    by_key = {item[0]: item for item in new}
    matches = []
    for key, old_index, old_row, old_span in old:
        _, new_index, new_row, new_span = by_key[key]
        require(old_row['function'] == new_row['function'], 'matched identity changed function label')
        matches.append({
            'rust_function': key[0], 'source_relative_file': key[1],
            'original_span': list(old_span), 'normalized_current_span': list(key[2]),
            'actual_current_span': list(new_span), 'historical_row_index': old_index,
            'current_row_index': new_index, 'function_label_diagnostic_only': old_row['function'],
            'historical_source': old_row['source'], 'current_source': new_row['source'],
            'historical_multiplicity': old_counts[key], 'current_multiplicity': new_counts[key], 'matched': True,
        })
    overloads = [item for item in matches if item['rust_function'] in OVERLOADS]
    require(len(overloads) == 2, 'both From overloads must be individually retained')
    for item in overloads:
        require(item['source_relative_file'] == 'src/error.rs'
                and tuple(item['original_span']) == OVERLOADS[item['rust_function']], 'overload span binding')
    return {
        'classification': 'AUDIT/METADATA', 'historical_exec_rows': len(old), 'historical_function_labels': 31,
        'current_exec_rows': len(new), 'current_function_labels': 35, 'matched_original_rows': len(matches),
        'missing_multiset': [], 'unexpected_duplicate_multiplicity': [],
        'matches': matches, 'both_overloads': overloads,
        'added_implementations': sorted(key[0] for key in extras),
    }


def budget(output, started):
    free = shutil.disk_usage('/private/tmp').free
    size = sum(path.stat().st_size for path in regular_files(output).values())
    elapsed = time.monotonic() - started
    require(free > 16 * 1024**3, 'free space must remain above 16 GiB')
    require(size < 32 * 1024**2, 'scratch must remain below 32 MiB')
    require(elapsed <= 720, 'audit walltime exceeds 12 minutes')
    return {'free_bytes': free, 'scratch_bytes': size, 'audit_elapsed_seconds': elapsed}


def run(input_root, review_root, archive_path, output):
    started = time.monotonic()
    require(output.resolve() == Path(__file__).resolve().parent
            and output.resolve().parent == Path('/private/tmp'), 'output must be this separate /private/tmp scratch')
    initial_budget = budget(output, started)
    entry = authenticate(input_root, review_root, archive_path, 'entry')
    write_json(output / 'audit-authentication-entry.json', entry)
    results = {'classification': 'AUDIT/METADATA', 'status': 'FAIL', 'native_execution': False,
               'strict_persisted_warm_cache_gate': 'UNMET', 'redb_tasks_open': 18,
               'trust_admission': 'UNCHANGED', 'final_acceptance': False, 'parent_integration': False}
    try:
        review_text = (review_root / 'REVIEW.md').read_text()
        review_evidence = read_json(review_root / 'independent-evidence.json')
        results['review_read_only_context'] = {
            'report_characters': len(review_text), 'evidence_top_level_keys': sorted(review_evidence),
            'derived_matches_used_as_oracle': False,
        }
        original, final = source_trees(archive_path, input_root / 'native')
        mapping = validate_sources(original, final)
        write_json(output / 'line-mapping.json', mapping.receipt())
        source_records = [
            {'file': name, 'archive_sha256': digest(content), 'final_sha256': digest(final[name]),
             'unchanged': content == final[name]}
            for name, content in sorted(original.items())
        ]
        write_json(output / 'source-comparison.json', {
            'archive_files': len(original), 'final_files': len(final), 'archived_files': source_records,
            'only_modified_archive_file': 'src/lib.rs', 'only_added_file': 'src/encoding_spec.rs',
            'added_file_sha256': digest(final['src/encoding_spec.rs']), 'lib_insertion_only': True,
        })
        historical = read_json(input_root / 'historical-v3-baseline.json')['inference']['verification_scope']
        all_matches = {}
        for number in ('03', '05', '06'):
            stdout_path = input_root / ('invocation-' + number) / 'stdout'
            data = read_json(stdout_path)
            recorded = read_json(stdout_path.parent / 'result.json')
            require(recorded['returncode'] == 0 and recorded['timeout'] is False, 'recorded positive not successful')
            require(recorded['stdout_sha256'] == digest(stdout_path.read_bytes()), 'stdout result binding mismatch')
            require(recorded['source_hashes'] == {name: digest(content) for name, content in final.items()},
                    'recorded positive source map differs from final tree')
            for filename in ('lib.rs', 'encoding_spec.rs'):
                require((stdout_path.parent / 'source' / filename).read_bytes() == final['src/' + filename],
                        'positive source snapshot mismatch')
            inference = data['verus-infer']
            require(inference['crate_success'] is True and inference['interface_written'] is True,
                    'recorded positive metadata incomplete')
            all_matches[number] = compare_scopes(historical, inference['verification_scope'], original, final,
                                                  str(input_root / 'native'))
        write_json(output / 'matches.json', all_matches)
        results.update(status='PASS', positive_invocations=['03', '05', '06'],
                       matched_original_rows_per_positive=32, total_independent_row_matches=96,
                       both_overloads_individually_matched_per_positive=True,
                       normalization={'historical_root': HISTORICAL_ROOT, 'current_root': str(input_root / 'native'),
                                      'span_coordinates': 'original lib.rs forwarded into final-source coordinates; columns unchanged'},
                       limits_entry=initial_budget)
    except Exception as error:
        results['error'] = type(error).__name__ + ': ' + str(error)
    finally:
        try:
            exit_receipt = authenticate(input_root, review_root, archive_path, 'exit')
            write_json(output / 'audit-authentication-exit.json', exit_receipt)
            require(entry['files'] == exit_receipt['files'] and entry['external_inputs'] == exit_receipt['external_inputs'],
                    'entry/exit input mutation')
            results['entry_exit_authenticated'] = True
            results['limits_exit'] = budget(output, started)
        except Exception as error:
            results.update(status='FAIL', exit_error=type(error).__name__ + ': ' + str(error))
        write_json(output / 'results.json', results)
    print(json.dumps({key: results[key] for key in ('classification', 'status', 'strict_persisted_warm_cache_gate')}))
    if results['status'] != 'PASS':
        print(results.get('error', results.get('exit_error', 'unknown error')))
    return 0 if results['status'] == 'PASS' else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, default=INPUT)
    parser.add_argument('--review', type=Path, default=REVIEW)
    parser.add_argument('--archive', type=Path, default=ARCHIVE)
    parser.add_argument('--out', type=Path, default=Path(__file__).resolve().parent)
    options = parser.parse_args()
    try:
        return run(options.input, options.review, options.archive, options.out)
    except (AuditError, KeyError, OSError, TypeError, json.JSONDecodeError) as error:
        print('AUDIT/METADATA FAIL: ' + str(error))
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
