#!/usr/bin/env python3
"""Check a source-faithful permission component; never certify native entry."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, support_artifacts, tool_artifacts
from infer_experiment import (LoggedCommands, replay_receipt, require, validate_run,
                              require_exact, require_failure_kind, require_function_coverage)

BUNDLE = Path('/tmp/verus-infer-buffer-cycle-20261109/parent-stable-key-bundle-a93b774a')
ACCEPTED = Path('/tmp/verus-infer-buffer-cycle-20261109/stable-key-validation-a93b774a/canonical/canonical-zgtzh85h/receipt.json')
ACCEPTED_SHA = '2dda5d93603528fe2623f4e37bc7cbaa48a537e3184c653a1abe1a4028f4ed3a'


def block(text, marker):
    require(text.count(marker) == 1, 'ambiguous source marker ' + marker)
    start = text.index('{', text.index(marker))
    depth = 0
    for index in range(start, len(text)):
        if text[index] == '{':
            depth += 1
        elif text[index] == '}':
            depth -= 1
            if depth == 0:
                return text[start:index + 1]
    raise ValueError('unclosed source block')


def tokens(text):
    return re.findall(r'\w+|[^\s]', re.sub(r'//[^\n]*', '', text))


def validate_component(observation, expected):
    # Shared basename coverage cannot distinguish the two original Clone derives.
    # Retain and check both by their complete rust_function identities instead.
    data = validate_run(observation, expected)
    summary = data['verification-results']
    require(summary.get('is-verifying-entire-crate') is True, 'partial crate checking')
    require(summary.get('verified') == (3 if expected else 2)
            and summary.get('errors') == (0 if expected else 1), 'wrong query inventory')
    inf = data['verus-infer']
    require(inf['crate_success'] is expected, 'crate status mismatch')
    require_function_coverage(inf['functions'], set())
    require_exact(inf['functions'])
    scope = inf['verification_scope']
    require(scope['native_collection_completed'] is True, 'incomplete native collection')
    require(scope['complete_selected_scope_verified'] is expected, 'scope certification mismatch')
    all_selected = [x for x in scope['native_functions'] if x['selected'] is True]
    auxiliary = {
        'permission_access_table': 'specification_definition_or_declaration',
        '_::<impl vstd::prelude::Structural for AccessKind>::assert_receiver_is_structural':
            'selected_item_without_translated_body',
        '_::<impl vstd::prelude::Structural for Permission>::assert_receiver_is_structural':
            'selected_item_without_translated_body',
    }
    selected = [x for x in all_selected if x['rust_function'] not in auxiliary]
    aux = [x for x in all_selected if x['rust_function'] in auxiliary]
    require(len(all_selected) == 6 and len(aux) == 3
            and {x['rust_function'] for x in aux} == set(auxiliary), 'auxiliary scope differs')
    require(all(x['ordinary_contract'] is False
                and x['body_status'] == auxiliary[x['rust_function']] for x in aux),
            'unexpected auxiliary contract/body status')
    names = [x['rust_function'] for x in selected]
    require(len(names) == 3 and set(names) == {
        '<AccessKind as std::clone::Clone>::clone',
        '<Permission as std::clone::Clone>::clone', 'Permission::grants'}, 'body identities differ')
    status = 'checked_exec_body' if expected else 'selected_body_not_certified_by_successful_full_run'
    require(all(x['body_status'] == status for x in selected), 'unverified selected contract')
    require(scope['checked_proof_bodies'] == 0, 'unexpected proof bodies')
    if expected:
        require(scope['checked_exec_bodies'] == 3, 'checked body count mismatch')
    else:
        require_failure_kind(observation['stderr'], 'postcondition not satisfied')
        require(not any(s in observation['stderr'] for s in [
            'possible arithmetic underflow/overflow', 'invariant not satisfied',
            'recommendation not met', 'error[E']), 'mixed rejection kind')
    return data


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--freeze', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output = a.output.resolve()
    a.output.mkdir(parents=True, exist_ok=False)
    receipt = dict(schema=1, classification='ordinary_source_component_only',
                   native_correspondences=0, publication=False, goal_complete=False,
                   commands=[], runs=[])
    with replay_receipt(a.output, receipt):
        freeze_bytes = a.freeze.read_bytes()
        frozen = json.loads(freeze_bytes)
        pin = ROOT / 'source/tools/research/native_miri_policy_pin/pins.json'
        required = [HERE / n for n in ['component.rs', 'replay.py', 'INTERFACE.md']]
        required += [pin, ACCEPTED, *support_artifacts()]
        require(set(map(lambda p: str(p.resolve()), required)) <= set(frozen['inputs']),
                'incomplete local/controller input freeze')
        guard = ArtifactGuard([Path(n) for n in frozen['inputs']] + [a.freeze])
        for n, expected in frozen['inputs'].items():
            require(guard.initial[str(Path(n).resolve())]['sha256'] == expected,
                    'frozen input mismatch: ' + n)
        require(guard.initial[str(a.freeze.resolve())]['sha256'] == hashlib.sha256(freeze_bytes).hexdigest(),
                'freeze changed during load')
        require(frozen['before_first_trial'] is True and digest(ACCEPTED) == ACCEPTED_SHA,
                'missing pretrial or accepted-runtime identity')
        accepted = json.loads(ACCEPTED.read_bytes())
        require(accepted['success'] is True, 'runtime baseline failed')
        for p in tool_artifacts(BUNDLE / 'verus', BUNDLE / 'z3'):
            require(str(p.resolve()) in frozen['inputs'], 'unfrozen verifier artifact')
            require(digest(p) == accepted['artifacts'][str(p)]['sha256'], 'accepted runtime changed')
        pins = json.loads(pin.read_bytes())
        source_names = ['src/tools/miri/src/borrow_tracker/stacked_borrows/mod.rs',
                        'src/tools/miri/src/borrow_tracker/stacked_borrows/item.rs',
                        'src/tools/miri/src/borrow_tracker/mod.rs']
        originals = []
        for n in source_names:
            q = pins['sources'][n]
            require(frozen['inputs'][q['path']] == q['sha256'], 'wrong primary-source identity')
            originals.append(Path(q['path']).read_text())
        component = (HERE / 'component.rs').read_text()
        require(tokens(block(originals[0], 'fn grants(')) == tokens(block(component, 'pub fn grants(')),
                'selected executable body differs')
        require(tokens(block(originals[1], 'pub enum Permission')) == tokens(block(component, 'pub enum Permission')),
                'permission variants differ')
        require(tokens(block(originals[2], 'pub enum AccessKind')) == tokens(block(component, 'pub enum AccessKind')),
                'access variants differ')
        receipt['artifacts'] = guard.initial
        receipt['fidelity'] = dict(executable_tokens_identical=True, enum_variants_identical=True,
                                  changed_representation=False,
                                  adapter='verus! namespace, Structural derives, public method and return binder/ensures only')
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
                   RUSTUP_TOOLCHAIN='1.99.1', VERUS_Z3_PATH=str(BUNDLE / 'z3'))
        run = LoggedCommands(a.output, a.output, receipt['commands'], timeout=120, guard=guard)
        cache = a.output / 'enabled-cache'
        cases = [('enabled-cold', component, True, True),
                 ('enabled-warm', component, True, True),
                 ('disabled', component, False, True),
                 ('disabled-guard-removed', component.replace('self != Permission::Disabled', 'true'), True, False),
                 ('read-inverted', component.replace('access == AccessKind::Read', 'access == AccessKind::Write'), True, False),
                 ('readonly-guard-removed', component.replace('self != Permission::SharedReadOnly', 'true'), True, False)]
        for name, source, enabled, expected in cases:
            file = a.output / (name + '.rs')
            file.write_text(source)
            source_guard = ArtifactGuard([file])
            env['VERUS_INFER_CACHE'] = str(cache if enabled and expected else a.output / (name + '-cache'))
            args = [str(BUNDLE / 'verus'), str(file), '--crate-type=lib', '--no-cheating', '--output-json']
            if enabled:
                args.append('--infer-goals')
            code = run(name, args, env)
            observation = dict(returncode=code, stdout=(a.output / (name + '.stdout')).read_text(),
                               stderr=(a.output / (name + '.stderr')).read_text())
            data = validate_component(observation, expected)
            source_guard.check()
            summary = data['verification-results']
            require(summary['errors'] == (0 if expected else 1), 'unexpected semantic error inventory')
            receipt['runs'].append(dict(name=name, enabled=enabled, expected=expected,
                                       source_sha256=digest(file), source_guard=source_guard.initial,
                                       verification=summary, inferred_functions=data['verus-infer']['functions']))
        guard.check()
        receipt.update(all_final_guards_passed=True, semantic_mutations_rejected=3,
                       inferred_helpers=[], manual_proof_bodies=[], actual_source_bridge_proved=False,
                       causal_assistance='redundant: exact ordinary component proves with goals enabled and disabled')


if __name__ == '__main__':
    main()
