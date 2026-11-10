#!/usr/bin/env python3
"""Independent, read-only authentication of two exact completed MODEL libraries.

Run prepare before either replay; finish compares fresh normal-controller results.
All controller compilation and logs belong in the supplied fresh private /tmp root.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
RESEARCH = ROOT / 'source/tools/research'
SPECS = {
    'pair': ('model_pair_input', '27c0422f',
             '/tmp/verus-infer-buffer-cycle-20261109/pair-input-final023',
             'f57ba4ea66e1722586aafb1b92c4e9a6946183ae00fe974176baae620600ed40', 18),
    'shared': ('model_shared_byte_read', 'bef470ad',
               '/tmp/shared-byte-read-trial-022',
               'd434bc73bacd2335313b73001436696cf8a305930fa9fa30eb1c79b014311b69', 19),
}
ALLOWED = {'Coq.Logic.FunctionalExtensionality.functional_extensionality_dep',
           'Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq'}

def need(ok, why):
    if not ok:
        raise ValueError(why)

def sha(data):
    return hashlib.sha256(data).hexdigest()

def digest(path):
    return sha(path.read_bytes())

def canonical(obj):
    return sha(json.dumps(obj, sort_keys=True, separators=(',', ':')).encode())

def read(path):
    return json.loads(path.read_bytes())

def write(name, obj):
    (HERE / name).write_text(json.dumps(obj, indent=2, ensure_ascii=False) + '\n')

def git_bytes(revision, path):
    return subprocess.check_output(['git', 'show', revision + ':' + str(path.relative_to(ROOT))], cwd=ROOT)

def strip_comments(text):
    # Nested Coq comments, maintaining offsets and line numbers for statement extraction.
    out, depth, i = [], 0, 0
    while i < len(text):
        pair = text[i:i+2]
        if pair == '(*':
            depth += 1
            out.extend('  ')
            i += 2
        elif pair == '*)':
            need(depth > 0, 'unbalanced comment')
            depth -= 1
            out.extend('  ')
            i += 2
        else:
            out.append(text[i] if depth == 0 or text[i] == '\n' else ' ')
            i += 1
    need(depth == 0, 'unclosed comment')
    return ''.join(out)

def declarations(path, module):
    original = path.read_text()
    text = strip_comments(original)
    # Include every declaration class, even those absent in these particular sources.
    kinds = ('Definition|Fixpoint|CoFixpoint|Lemma|Theorem|Corollary|Proposition|Fact|Remark|'
             'Example|Instance|Inductive|CoInductive|Record|Class|Let|Axiom|Axioms|Parameter|Parameters')
    pattern = r'^\s*(?:(?:Local|Global|Polymorphic|Monomorphic|Program)\s+)*(' + kinds + r')\s+(\w+)'
    matches = list(re.finditer(pattern, text, re.M))
    need(not re.search(r'\b(Program|Obligation|Next Obligation|Axiom|Axioms|Parameter|Parameters|Admitted|admit|Abort|native_compute|vm_cast_no_check)\b|Unset\s+(Guard|Positivity|Universe)\s+Checking', text), 'unreviewed construct: ' + str(path))
    need(not re.search(r'\b(Inductive|CoInductive|Record|Class)\b', text), 'constructors/projections require explicit enumeration')
    result = []
    for j, match in enumerate(matches):
        start = match.start(1)
        end = matches[j+1].start() if j+1 < len(matches) else len(text)
        chunk = text[start:end]
        if match[1] in {'Definition', 'Fixpoint', 'CoFixpoint', 'Let'}:
            end = start + chunk.index('.\n') + 1
        else:
            proof = re.search(r'\bProof\b', chunk)
            need(proof is not None, 'missing proof: ' + match[2])
            end = start + proof.start()
        result.append(dict(name=module + '.' + match[2], kind=match[1],
                           line=original[:start].count('\n') + 1,
                           statement=original[start:end].strip()))
    return result

def artifacts(receipt, out):
    expected = {Path(k).name: v['sha256'] for k, v in receipt['proof_artifacts'].items()}
    actual = {p.name: digest(p) for p in (out/'proofs').iterdir() if p.suffix in {'.v', '.vo'}}
    need(actual == expected, 'proof source/object identities differ: ' + str(out))
    return actual

def audit_receipt(out, expected_assumptions, count):
    receipt = read(out / 'receipt.json')
    need(receipt['success'] is True and receipt['model_proofs_kernel_checked'] is True, 'unsuccessful receipt')
    need(receipt['constant_count'] == count and receipt['program_obligations'] == 0, 'inventory mismatch')
    need(receipt['assumptions'] == expected_assumptions, 'per-constant assumptions mismatch')
    need(set(receipt['kernel_axioms']) <= ALLOWED, 'unexpected kernel axioms')
    for axioms in receipt['assumptions'].values():
        need(set(axioms) <= ALLOWED, 'unexpected individual assumption')
    commands = receipt['commands']
    need(all(c['exit_code'] == 0 for c in commands), 'failed command')
    need(commands[-1]['label'] == 'kernel', 'kernel not final command')
    # These receipts do not pin log bytes. Parse their contents against the pinned
    # individual results, then seal raw-log hashes in our own pretrial manifest.
    sys.path.insert(0, str(RESEARCH/'native_semantic_bridge'))
    from proof_checks import parse_axioms, parse_kernel_summary
    text = (out/'assumptions.stdout').read_text()
    sections = re.findall(r'^BEGIN (\S+)\n(.*?)^END (\S+)\s*$', text, re.M | re.S)
    names = list(expected_assumptions)
    need([a for a, _, _ in sections] == names == [b for _, _, b in sections], 'log inventory coverage')
    need({a: parse_axioms(body) for a, body, _ in sections} == expected_assumptions, 'assumptions log mismatch')
    need(parse_kernel_summary((out/'kernel.stderr').read_text()) == receipt['kernel_axioms'], 'kernel log mismatch')
    kernel = commands[-1]['argv']
    need('-silent' in kernel and '-o' in kernel, 'missing recursive checker summary')
    need(not set(kernel) & {'-admit', '-norec', '-quick', '-vio'}, 'excluded/admitted check')
    return receipt, artifacts(receipt, out)

def prepare():
    need(not (HERE/'pretrial.json').exists(), 'prepare is write-once')
    frozen, libraries, inventory = {}, {}, {}
    for key, (directory, revision, completed, receipt_sha, count) in SPECS.items():
        scope = RESEARCH / directory
        results = read(scope/'results.json')
        freeze = read(scope/'final-freeze.json')
        # Every tracked scope file is identical to the exact requested implementation.
        tracked = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', revision,
                                          '--', str(scope.relative_to(ROOT))], cwd=ROOT, text=True).splitlines()
        for name in tracked:
            path = ROOT/name
            need(path.read_bytes() == git_bytes(revision, path), 'revision mismatch: ' + name)
            frozen[name] = digest(path)
        for name, expected in freeze['files'].items():
            need(digest(ROOT/name) == expected, 'freeze mismatch: ' + name)
            frozen[name] = expected
        need(results['receipt']['sha256'] == receipt_sha, 'committed receipt pin mismatch')
        out = Path(completed)
        need(digest(out/'receipt.json') == receipt_sha, 'completed receipt mismatch')
        need((out/'freeze.json').read_bytes() == (scope/'final-freeze.json').read_bytes(), 'completed freeze mismatch')
        receipt, objects = audit_receipt(out, results['assumptions'], count)
        expected_objects = results.get('proof_artifacts', {}) or {**results['proof_sources'], **results['proof_objects']}
        need(objects == expected_objects, 'committed artifact pins mismatch')
        declarations_by_module = {}
        records = []
        for module, names in freeze['constants'].items():
            src = scope/'candidates'/(module+'.v')
            ds = declarations(src, module)
            need([d['name'].split('.')[1] for d in ds] == names, 'independent declaration mismatch')
            need(objects[src.name] == digest(src), 'returned source mismatch')
            declarations_by_module[module] = names
            records.extend(ds)
        need(declarations_by_module == results['declarations'] and len(records) == count, 'statement coverage')
        need(set(results['assumptions']) == {d['name'] for d in records}, 'assumptions coverage')
        inventory[key] = dict(declarations=records, count=count,
                              constructors=[], projections=[], generated_obligations=[],
                              assumptions=results['assumptions'])
        libraries[key] = dict(directory=directory, implementation=revision,
                              completed=completed, receipt_sha256=receipt_sha,
                              freeze_sha256=digest(scope/'final-freeze.json'),
                              artifacts=objects, assumptions_sha256=canonical(results['assumptions']),
                              raw_logs={p.name: digest(p) for p in out.iterdir()
                                        if p.suffix in {'.stdout', '.stderr'}},
                              kernel_axioms=receipt['kernel_axioms'])
    write('statement-inventory.json', inventory)
    frozen[str((HERE/'review.py').relative_to(ROOT))] = digest(HERE/'review.py')
    frozen[str((HERE/'statement-inventory.json').relative_to(ROOT))] = digest(HERE/'statement-inventory.json')
    write('pretrial.json', dict(before_first_review_trial=True, libraries=libraries,
                                files=frozen, scope='two MODEL libraries only',
                                no_fixed_g_or_combined_budget_claim=True))
    print('Authenticated completed receipts, all returned artifacts, exact implementations and all 37 declarations.')

def finish(private):
    frozen = read(HERE/'pretrial.json')
    for name, expected in frozen['files'].items():
        need(digest(ROOT/name) == expected, 'pretrial input changed: ' + name)
    comparisons = {}
    for key, (directory, revision, completed, receipt_sha, count) in SPECS.items():
        scope = RESEARCH/directory
        results = read(scope/'results.json')
        original, old_artifacts = audit_receipt(Path(completed), results['assumptions'], count)
        need(digest(Path(completed)/'receipt.json') == receipt_sha, 'completed receipt changed')
        for name, expected in frozen['libraries'][key]['raw_logs'].items():
            need(digest(Path(completed)/name) == expected, 'completed log changed')
        fresh, fresh_artifacts = audit_receipt(private/key, results['assumptions'], count)
        need((private/key/'freeze.json').read_bytes() == (scope/'final-freeze.json').read_bytes(), 'replay freeze changed')
        need(fresh_artifacts == old_artifacts, 'fresh source/object bytes differ')
        need(fresh['kernel_axioms'] == original['kernel_axioms'], 'kernel assumptions differ')
        comparisons[key] = dict(output=str(private/key), receipt_sha256=digest(private/key/'receipt.json'),
                                 all_sources_objects_byte_identical=True, artifacts=fresh_artifacts,
                                 individual_assumptions_match=True, assumptions=fresh['assumptions'],
                                 assumptions_sha256=canonical(fresh['assumptions']),
                                 assumptions_stdout_sha256=digest(private/key/'assumptions.stdout'),
                                 kernel_stderr_sha256=digest(private/key/'kernel.stderr'),
                                 kernel_axioms=fresh['kernel_axioms'], constant_count=count,
                                 program_obligations=0,
                                 commands=[dict(label=c['label'], argv=c['argv'], exit_code=c['exit_code'])
                                           for c in fresh['commands']],
                                 input_artifact_count=len(fresh['inputs']))
    write('results.json', dict(accepted_model_libraries=True, replays=comparisons,
                               pretrial_sha256=digest(HERE/'pretrial.json'),
                               statement_inventory_sha256=digest(HERE/'statement-inventory.json'),
                               new_axioms=0, actual_native_correspondences=0,
                               minimal_simulations_open=44, phase_obligations_open=3,
                               writer_effects_open=101, full_hmac='inherited 1/1; not rerun',
                               combined_loop_or_fixed_budget_proved=False,
                               publication=False, goal_complete=False))
    print('Fresh compile, full recursive kernel check, all artifact bytes and individual assumptions match.')

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=['prepare', 'finish'])
    parser.add_argument('--private', type=Path)
    args = parser.parse_args()
    if args.action == 'prepare':
        prepare()
    else:
        need(args.private is not None, '--private required')
        finish(args.private.resolve())
