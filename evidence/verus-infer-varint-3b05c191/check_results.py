#!/usr/bin/env python3
"""Validate replay receipts and emit a compact report; BLOCKED stays BLOCKED."""
import argparse
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from replay import HERE, ROOT, digest, source_guard
from infer_experiment import require, require_exact


def check_manual_sources():
    baseline = (HERE / 'diagnostics/application-no-layout-ordinary.rs').read_text()
    loop = '''for b in it: $buf.iter_mut()
            invariant_except_break
                i == it.index(),
                n == $number >> (7 * (i as u64)),
            invariant
                i <= 9,
                it.seq().len() == 10,
            ensures
                n == 0,
        {
            proof { shift_step($number, n, i as u64); }'''
    helper = None
    for name in ['canonical-attempt', 'length-attempt', 'length-negative']:
        src = (HERE / 'manual' / (name + '.rs')).read_text()
        start = src.index('\nverus! {\n// Authored mechanical lemma')
        end = src.index('\nmacro_rules! encode', start)
        if helper is None:
            helper = src[start:end]
        require(src[start:end] == helper, 'mechanical lemma drift')
        src = src[:start] + src[end:]
        initial = '        proof { assert($number >> 0 == $number) by (bit_vector); }\n'
        require(src.count(initial) == 1 and src.count(loop) == 1, 'mechanical annotation drift')
        src = src.replace(initial, '').replace(loop, 'for b in $buf.iter_mut() {')
        if name != 'canonical-attempt':
            src = src.replace('ensures 1 <= encoded.len() <= 10', 'ensures canonical(number, encoded@)')
        if name == 'length-negative':
            require(src.count('&$buf[0..i]') == 1, 'negative drift')
            src = src.replace('&$buf[0..i]', '&$buf[0..=i]')
        require(src == baseline, 'selected executable/semantic source drift in ' + name)


def classify(row, stderr):
    fixture = row['fixture']
    name = Path(fixture).stem
    if fixture.startswith('fixtures/'):
        if not name.startswith('adapted-'):
            return 'frontend_macro', '`for` loops produced by a macro expansion'
        if name == 'adapted-ordinary':
            return 'layout_policy_and_safety', 'external_body/assume_specification not allowed with --no-cheating'
        return 'inference_iter_mut', 'Verus Infer unsupported: calls require an acyclic same-crate inferred callee'
    if name == 'application-no-layout-inferred':
        return 'inference_iter_mut', 'Verus Infer unsupported: calls require an acyclic same-crate inferred callee'
    if name == 'application-no-layout-ordinary':
        return 'canonical_and_safety_unproved', 'postcondition not satisfied'
    if (name.startswith('semantic-negative-') or name == 'macro-adapter-negative'
            or name in ['canonical-attempt', 'length-negative']):
        return 'authored_postcondition_failure', 'postcondition not satisfied'
    require(name in ['engine-positive', 'macro-adapter-positive', 'semantic-positive', 'length-attempt'],
            'unclassified fixture ' + fixture)
    return 'proved_control_or_partial', None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--raw-root', type=Path, required=True)
    parser.add_argument('--compact', type=Path)
    args = parser.parse_args()
    source_guard()
    check_manual_sources()
    receipts = []
    for path in args.raw_root.glob('varint-*/receipt.json'):
        data = json.loads(path.read_text())
        require(data.get('completed') is True and data.get('artifacts_unchanged') is True, 'incomplete run ' + str(path))
        require(data['application_proofs_proved'] is False, 'application proof flag drift')
        receipts.append((data['host']['started_utc'], path, data))
    receipts.sort(key=lambda item: (item[0], str(item[1])))
    require(receipts, 'no receipts')
    selected = {}
    native = None
    history = []
    tool_hashes = {}
    for _, path, data in receipts:
        history.append(dict(receipt=str(path), sha256=digest(path), stage=data['stage'], cases=len(data['cases'])))
        for filename, identity in data['artifacts'].items():
            if (filename.endswith(('/rust_verify', '/verus', '/z3', '/vstd.vir', '/libvstd.rlib', '/libverus_builtin.rlib'))
                    or 'librustc_driver' in filename or 'libverus_builtin_macros.' in filename
                    or 'libverus_state_machines_macros.' in filename):
                previous = tool_hashes.setdefault(filename, identity['sha256'])
                require(previous == identity['sha256'] == digest(filename), 'tool drift ' + filename)
        for row in data['cases']:
            selected[(row['name'], row['phase'])] = (path.parent, row)
        if 'native_supporting_oracles' in data:
            native = (path.parent, data)
    compact = []
    for key, (raw, row) in sorted(selected.items()):
        fixture = HERE / row['fixture']
        require(digest(fixture) == row['source_sha256'], 'selected result is stale: ' + str(key))
        stderr_path = raw / row['name'] / (row['phase'] + '.stderr')
        stdout_path = stderr_path.with_suffix('.stdout')
        require(digest(stderr_path) == row['stderr_sha256'] and digest(stdout_path) == row['stdout_sha256'], 'log drift')
        stderr = stderr_path.read_text()
        require(not any(s in stderr.lower() for s in ['panicked at', 'resource limit', 'timed out']), 'nonproof failure')
        require('--no-cheating' in row['command'], 'missing trust policy')
        require(not any(flag in row['command'] for flag in ['--no-verify', '--no-lifetime', '--no-erasure-check', '--verify-function', '--verify-module']), 'verification bypass')
        require(json.loads(stdout_path.read_text()) == row['verifier_json'], 'receipt/stdout mismatch')
        outcome, diagnostic = classify(row, stderr)
        data = row['verifier_json']
        require(isinstance(data, dict), 'missing verifier JSON')
        vr = data['verification-results']
        inference = data.get('verus-infer', {})
        positive = outcome == 'proved_control_or_partial'
        require((row['returncode'] == 0) == positive, 'unexpected process result ' + str(key))
        # Baseline Verus eagerly exports ordinary (non-inference-aware) VIR.
        # Record that behavior explicitly; it is never evidence of a proof.
        legacy_negative_export = (not positive and '--infer-goals' not in row['command']
            and outcome in ['layout_policy_and_safety', 'canonical_and_safety_unproved',
                            'authored_postcondition_failure'])
        require(row['export_exists'] == (positive or legacy_negative_export), 'unexpected publication behavior ' + str(key))
        if positive:
            require(vr.get('success') is True and vr.get('errors') == 0 and vr.get('is-verifying-entire-crate') is True, 'incomplete positive proof')
        else:
            require(diagnostic in stderr and vr.get('success') is not True, 'wrong negative diagnostic')
            require(inference.get('interface_written', False) is False, 'negative interface publication')
        if outcome == 'inference_iter_mut':
            require('"iter_mut"' in stderr, 'wrong inferred call blocker')
        if outcome in ['authored_postcondition_failure', 'canonical_and_safety_unproved']:
            require(vr.get('encountered-vir-error') is False and vr.get('is-verifying-entire-crate') is True, 'frontend posed as semantic failure')
            if outcome == 'authored_postcondition_failure':
                require(vr.get('errors') == 1, 'mixed semantic failure')
        if Path(row['fixture']).stem == 'engine-positive':
            require_exact(inference['functions'])
            require(len(inference['functions']) == 1 and inference['interface_written'] is True, 'missing exact sanity publication')
        compact.append(dict(name=row['name'], phase=row['phase'], fixture=row['fixture'],
                            source_sha256=row['source_sha256'], outcome=outcome, returncode=row['returncode'],
                            wall_micros=row['wall_micros'], export_exists=row['export_exists'],
                            legacy_ordinary_negative_export=legacy_negative_export,
                            verification_results=vr, raw=str(raw / row['name'])))
    require(native is not None, 'missing native supporting checks')
    raw, data = native
    require(digest(HERE / 'native.rs') == data['artifacts'][str(HERE / 'native.rs')]['sha256'], 'native source drift')
    require(len(data['native_supporting_oracles']) == 4 and all(row['returncode'] == 0 for row in data['native_supporting_oracles']), 'native failure')
    for mode in ['debug', 'release']:
        require('3 passed; 0 failed' in (raw / f'native-{mode}-run.stdout').read_text(), 'native result drift')
    require(len(compact) == 66, 'missing or extra selected cases')
    report = dict(schema=1, application_status='blocked', application_proofs_proved=False,
                  partial_property='Every u64 and initialized exclusive arbitrary [u8;10]: returned length 1..=10, original safety/debug assertions; authored invariants and bit-vector lemma required.',
                  selected_cases=compact, attempts=history, tool_sha256=tool_hashes,
                  native=dict(raw=str(raw), cases_per_build=173503, builds=['debug', 'release'],
                              oracle_only=True, commands=data['native_supporting_oracles']),
                  validation='passed; full canonical property remains unproved')
    report['legacy_ordinary_negative_exports'] = sum(row['legacy_ordinary_negative_export'] for row in compact)
    report['publication_limit'] = 'Baseline ordinary --export without inference/--infer-goals eagerly writes VIR even on proof failure. Such files are not proof certificates. Inference-aware seeded gates passed; legacy import behavior was not tested.'
    if args.compact:
        args.compact.write_text(json.dumps(report, indent=2) + '\n')
    print(f'validated {len(compact)} selected verifier cases across {len(receipts)} retained receipts; full canonical property BLOCKED')


if __name__ == '__main__':
    main()
