#!/usr/bin/env python3
"""Authenticate bounded delivered evidence and replay inexpensive source checks."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
BRIDGE = ROOT/'source/tools/research/native_semantic_bridge'
ENTRY = ROOT/'source/tools/research/native_source_entry'
GOAL = ROOT/'source/tools/research/mutable_buffer_goal'
sys.path[:0] = [str(BRIDGE), str(ROOT/'source/tools')]
from infer_eval_support import ArtifactGuard, digest
from proof_checks import check_inventory, parse_axioms, parse_kernel_summary
from source_map import source_audit_inputs, validate_source_freeze, authenticated_extraction


def require(condition, message):
    if not condition:
        raise ValueError(message)


def read(path):
    return json.loads(Path(path).read_text())


def write(path, value):
    path.write_text(json.dumps(value, indent=2)+'\n')


def reference(ref):
    path = Path(ref.get('path', ref.get('receipt', '')))
    require(digest(path) == ref['sha256'], 'changed evidence '+str(path))
    return read(path)


def historical(name, commit):
    return subprocess.check_output(['git', 'show', commit+':'+str(name)], cwd=ROOT)


def main():
    output = Path(sys.argv[1]).resolve()
    output.mkdir(parents=True, exist_ok=False)
    validation = dict(base=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        commands=[], publication=False, native_correspondences_proved=0,
        semantic_proofs_run=0, coq_compilation_run=False)
    raw = Path('/tmp/native-semantic-bridge-e4a8dcab/source-v1')
    frozen = read(BRIDGE/'source-freeze-fix.json')
    paths = [ROOT/n for n in frozen['files']]
    paths += [raw/n for n in ('receipt.json', 'drop-native.json', 'drop-borrowck.json')]
    validate_source_freeze(frozen)
    authenticated_extraction(raw)
    require(len(source_audit_inputs()) == 7, 'mandatory set size')
    check_inventory(ROOT/frozen['candidate_directory'], frozen['modules'])
    proof_required = [BRIDGE/'check.py', BRIDGE/'proof_checks.py', BRIDGE/'source-bindings.json',
        BRIDGE/'baseline-bindings.json', GOAL/'foundation-baseline.json',
        *[(ROOT/frozen['candidate_directory'])/(m+'.v') for m in frozen['modules']]]
    require({str(p.relative_to(ROOT)) for p in proof_required} <= frozen['files'].keys(), 'proof compatibility')
    for name in ('check.py', 'proof_checks.py'):
        require((BRIDGE/name).read_bytes() == historical((BRIDGE/name).relative_to(ROOT), '1f727448'), 'changed proof checker')
    for p in (ROOT/frozen['candidate_directory']).glob('*.v'):
        require(p.read_bytes() == historical(p.relative_to(ROOT), '8935534e'), 'changed model source')
    validation['active_freeze'] = dict(path=str(BRIDGE/'source-freeze-fix.json'),
        sha256=digest(BRIDGE/'source-freeze-fix.json'), files=len(frozen['files']),
        required=[str(p.relative_to(ROOT)) for p in source_audit_inputs()], proof_controller_compatible=True)

    parent = read(GOAL/'parent-semantic-foundation.json')
    replay = reference(parent['review_fix_validation'])
    require(replay['success'] is True and len(replay['commands']) == 7, 'parent replay')
    proof = read(replay['proof_receipt'])
    require(digest(replay['proof_receipt']) == replay['proof_receipt_sha256'], 'parent proof receipt')
    require(proof['success'] is True and proof['model_proofs_kernel_checked'] is True, 'parent proof status')
    object_identities = {}
    for module, item in parent['parent_object_and_source_hash_audit'].items():
        original = Path(item['path'])
        source = ROOT/frozen['candidate_directory']/(module+'.v')
        current = Path(replay['proof_receipt']).parent/'proofs'/(module+'.vo')
        require(digest(original) == item['sha256'] == digest(current) == proof['proof_terms'][module], 'object identity')
        require(digest(source) == item['source_sha256'], 'source identity')
        object_identities[module] = dict(source_sha256=digest(source), object_sha256=digest(current))
        paths += [original, current]
    require(len(proof['assumption_inventory']) == 25, 'proof assumption inventory')
    validation['inherited_math'] = dict(replay_receipt_sha256=digest(parent['review_fix_validation']['receipt']),
        proof_receipt_sha256=digest(replay['proof_receipt']), object_identities=object_identities,
        assumptions=25, fresh_compile=False)

    handoff = read(GOAL/'parent-source-entry.json')
    inspection = reference(handoff['receipts']['inspection'])
    controls = reference(handoff['receipts']['source_controls'])
    require(inspection['success'] is True and controls['success'] is True, 'entry delivered status')
    candidate = ENTRY/'candidates/entry-v1/EntryInventory.v'
    text = candidate.read_text()
    require(not re.search(r'\b(Lemma|Theorem|Goal|Definition|Axiom|Admitted|Parameter|Obligation|Instance|Fixpoint|Inductive)\b', text),
        'new declaration in inspection')
    require(all(line.startswith(('From ', 'Set Printing Width ', 'Redirect ', 'Fail Check '))
        for line in text.splitlines()), 'unexpected inspection command')
    entry_freeze = read(ENTRY/'entry-v1-freeze.json')
    historical_ids = {}
    for name, sha in entry_freeze['files'].items():
        require(hashlib.sha256(historical(name, 'df15a254')).hexdigest() == sha, 'historical freeze mismatch '+name)
        historical_ids[name] = dict(sha256=sha, matches_current=digest(ROOT/name) == sha)
    for name, sha in entry_freeze['private_inputs'].items():
        require(digest(name) == sha, 'entry private input '+name)
        paths.append(Path(name))
    inventory = read(ENTRY/'checked-rule-inventory.json')['constants']
    require(len(inventory) == len(inspection['statements']) == len(inspection['assumptions']) == 42, '42 inventory')
    for item in inventory:
        name = item['constant']
        ref = inspection['statements'][name]
        require(digest(ref['path']) == item['sha256'] == ref['sha256'], 'statement hash '+name)
        require(Path(ref['path']).read_text() == item['statement'], 'statement bytes '+name)
        require(item['assumptions'] == inspection['assumptions'][name], 'assumption mismatch')
    for name, sha in inspection['outputs'].items():
        require(digest(name) == sha, 'inspection output '+name)
        paths.append(Path(name))
    for index, item in enumerate(inventory):
        path = Path(handoff['receipts']['inspection']['path']).parent/'inspection'/f'{index:02d}-assumptions.out'
        require(parse_axioms(path.read_text()) == item['assumptions'], 'parsed assumptions '+item['constant'])
    inspection_dir = Path(handoff['receipts']['inspection']['path']).parent
    require((inspection_dir/'inspection/EntryInventory.v').read_bytes() == candidate.read_bytes(), 'inspection source')
    require(digest(inspection_dir/'inspection/EntryInventory.vo') == inspection['inspection_object_sha256'], 'inspection object')
    require(parse_kernel_summary((inspection_dir/'recursive-kernel.stderr').read_text()) == inspection['kernel_axioms'], 'entry kernel policy')
    require(len([x for x in inventory if not x['assumptions']]) == 30, 'closed count')
    require(inspection['new_theorems'] == [] and inspection['native_input_proved'] is False
        and inspection['native_effects_discharged'] == 0 and inspection['publication'] is False, 'entry claim')
    validation['source_entry'] = dict(receipt_sha256=digest(handoff['receipts']['inspection']['path']),
        object_sha256=inspection['inspection_object_sha256'], statements=42, closed=30, dependent_FE=12,
        new_declarations=0, source_entry_proved=False, historical_freeze=historical_ids,
        replayed=False, historical_freeze_rewritten=False)

    provenance = read(GOAL/'canonical-fact-provenance.json')
    fact_receipt = reference(provenance['receipt'])
    fact_freeze = reference(provenance['pretrial_freeze'])
    build_receipt = reference(provenance['historical_build_receipt'])
    require(build_receipt['success'] is True and fact_receipt['success'] is True, 'fact receipt status')
    require(fact_receipt['verification']['verified'] == 6 and fact_receipt['verification']['errors'] == 0,
        'canonical diagnostic result')
    require(fact_receipt['checked_exec'] == ['u64'] and fact_receipt['checked_proof'] == []
        and fact_receipt['helpers'] == [], 'canonical checked inventory')
    for item in provenance['source_inputs']+[provenance['fixture']]:
        require(digest(ROOT/item['path']) == item['sha256'], 'fact source identity')
    for item in provenance['log_files']:
        require(digest(item['path']) == item['sha256'], 'fact log identity')
        paths.append(Path(item['path']))
    logdir = Path(provenance['log_files'][0]['path']).parent
    for item in provenance['log_anchors']:
        require(item['contains'] in (logdir/item['file']).read_text().splitlines()[item['line']-1], 'fact anchor')
    assume = (logdir/'root.air').read_text().splitlines()[15499]
    require((logdir/'root.air').read_text().splitlines()[15498].strip() == '(assume'
        and 'ens%core!slice.impl&%0.iter_mut.' in assume, 'constructor assumption anchor')
    require(provenance['indispensability_proved'] is False and provenance['source_native_correspondence_proved'] is False
        and provenance['common_checked_step_confirmed'] is False and provenance['publication'] is False, 'fact claim boundary')
    validation['fact_provenance'] = dict(receipt_sha256=digest(provenance['receipt']['path']),
        freeze_sha256=digest(provenance['pretrial_freeze']['path']), source_inputs=4, log_files=4, anchors=7,
        canonical='6/0', checked_exec=['u64'], checked_proof=[], helpers=[],
        causal_or_native_proof=False, rerun=False)

    guard = ArtifactGuard(paths)
    validation['artifacts_before'] = guard.initial
    serial = 0
    def run(argv):
        nonlocal serial
        serial += 1
        result = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True, timeout=120)
        out = output/f'{serial:02d}.stdout'
        err = output/f'{serial:02d}.stderr'
        out.write_text(result.stdout); err.write_text(result.stderr)
        require(result.returncode == 0, result.stderr)
        validation['commands'].append(dict(argv=argv, returncode=result.returncode,
            stdout=dict(path=str(out), sha256=digest(out)), stderr=dict(path=str(err), sha256=digest(err))))
    for optimized in (False, True):
        mode = 'optimized' if optimized else 'normal'
        py = [sys.executable, '-B', *(['-O'] if optimized else [])]
        run([*py, '-m', 'unittest', 'discover', '-s', str(BRIDGE), '-p', 'test_*.py', '-v'])
        for tool in ('source_map.py', 'audit.py'):
            run([*py, str(BRIDGE/tool), '--freeze', str(BRIDGE/'source-freeze-fix.json'),
                '--extraction', str(raw), '--output', str(output/(mode+'-'+tool))])
            receipt = read(output/(mode+'-'+tool)/'receipt.json')
            require(receipt['success'] is True and receipt['publication'] is False, 'healthy source status')
            require(all(str(p.resolve()) in receipt['artifacts'] for p in source_audit_inputs()), 'guard completeness')
            if tool == 'audit.py':
                require(len(receipt['cases']) == 12 and receipt['semantic_negative_count'] == 0, 'audit classification')
        generator_output = output/(mode+'-generated-freeze.json')
        run([*py, str(BRIDGE/'freeze.py'), '--candidate', str(ROOT/frozen['candidate_directory']),
            '--output', str(generator_output)])
        require(generator_output.read_bytes() == (BRIDGE/'source-freeze-fix.json').read_bytes(), 'generator equality')
    validation['generator_byte_identical_in_both_modes'] = True
    validation['normal_and_optimized_unit_tests'] = 11
    validation['normal_and_optimized_source_controls'] = 12
    guard.check()
    validation['artifacts_after'] = guard.snapshot()
    validation['success'] = True
    write(output/'validation.json', validation)
    print(json.dumps(dict(validation=str(output/'validation.json'), sha256=digest(output/'validation.json'))))


if __name__ == '__main__':
    main()
