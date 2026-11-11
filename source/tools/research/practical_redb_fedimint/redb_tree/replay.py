#!/usr/bin/env python3
"""Focused R2 baseline/proof/control replay; existing shared execution utilities only."""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[2]))
from infer_experiment import (LoggedCommands, replay_receipt, require, require_exact,
                              require_publication, require_failure_kind)
from infer_eval_support import ArtifactGuard, digest, support_artifacts

EXEC = ['BtreeBitmap::' + n for n in ('get_level', 'get_height', 'set', 'clear', 'get_level_mut', 'update_to_root')]
EXEC += ['U64GroupedBitmap::' + n for n in ('data_index_of', 'select_mask', 'len', 'get', 'set', 'clear')]
PROOF = ['word_bits', 'word_frame', 'layer_edit', 'leaf_edit', 'clear_not_full', 'finish_mutation', 'propagation', 'ancestor_next']
SPECS = ['words', 'member', 'physical_bit', 'layer_wf', 'shape', 'edge', 'tree_wf', 'ancestor',
         'storage_frame', 'path_frame', 'allocated', 'count', 'mutation', 'pending', 'word_change']


def block(text, start):
    begin = text.index('{', start)
    end, depth = begin + 1, 1
    while depth:
        depth += (text[end] == '{') - (text[end] == '}')
        end += 1
    return begin, end


def tokens(text):
    return re.findall(r'"(?:\\.|[^"\\])*"|//[^\n]*|\w+|[^\s]', text)


def projection(body):
    # Only the literal proof-only annotation forms used in this compact fixture.
    while match := re.search(r'\bproof\s*\{', body):
        _, end = block(body, match.start())
        body = body[:match.start()] + body[end:]
    body = re.sub(r'let ghost [^;]+;', '', body)
    body = re.sub(r'\bloop\s+(?:invariant_except_break|decreases)\b.*?\n        \{', 'loop {', body, flags=re.S)
    return tokens(body)


def fidelity(meta, fixture):
    original = (Path(meta['private_artifacts']) / 'source/bitmap.rs').read_text()
    manual = fixture.read_text()
    result = {}
    for name, item in meta['items'].items():
        extracted = original[item['start_byte']:item['end_byte']]
        require(hashlib.sha256(extracted.encode()).hexdigest() == item['sha256'], 'extraction: ' + name)
        if name[0].isupper():
            require(tokens(extracted[extracted.index('{'):]) == tokens(manual[block(manual, manual.index('struct ' + name))[0]:block(manual, manual.index('struct ' + name))[1]]), 'struct fields: ' + name)
            continue
        kind, method = name.split('_', 1)
        impl = manual.index('impl ' + ('BtreeBitmap' if kind == 'tree' else 'U64GroupedBitmap') + ' {')
        signature = manual.index('fn ' + method + '(', impl)
        begin = manual.index('\n{', signature) + 1
        _, end = block(manual, begin)
        require(projection(manual[begin:end]) == tokens(extracted[extracted.index('{'):]), 'exec body: ' + name)
        result[name] = item['body_sha256']
    corrected = (HERE / 'corrected.rs').read_text()
    for name in SPECS:
        a = corrected.index('spec fn ' + name + '('); _, e = block(corrected, a)
        b = manual.index('spec fn ' + name + '('); _, f = block(manual, b)
        require(tokens(corrected[a:e]) == tokens(manual[b:f].replace('#[trigger] ', '')), 'spec changed: ' + name)
    for owner in ('BtreeBitmap', 'U64GroupedBitmap'):
        for identity in [n for n in EXEC if n.startswith(owner + '::')]:
            method = identity.split('::')[-1]
            a = corrected.index('fn ' + method + '(', corrected.index('impl ' + owner + ' {'))
            b = manual.index('fn ' + method + '(', manual.index('impl ' + owner + ' {'))
            require(tokens(corrected[a:corrected.index('\n{', a)]) == tokens(manual[b:manual.index('\n{', b)]), 'contract changed: ' + identity)
    for token in ('assume(', 'admit(', 'external_body', 'assume_specification', 'external_fn_specification', 'verify=false'):
        require(token not in manual, 'forbidden trust: ' + token)
    return result


def main():
    meta = json.loads((HERE / 'private.json').read_text())
    out = Path(sys.argv[1]).resolve()
    require(str(out).startswith('/tmp/redb-r2-'), 'private R2 output required')
    out.mkdir(mode=0o700, parents=True, exist_ok=False)
    receipt = dict(case='R2', application_property_proved=False, commands=[], outcomes=[])
    with replay_receipt(out, receipt):
        private = Path(meta['private_artifacts']); runtime = private / 'runtime'
        pins = {**meta['runtime']['tools'], 'receipt.json': meta['runtime']['receipt_sha256']}
        for name, sha in pins.items():
            require(digest(runtime / name) == sha, 'runtime pin: ' + name)
        for name, item in meta['source']['files'].items():
            p = private / 'source' / Path(name).name
            require(digest(p) == item['sha256'] and p.stat().st_size == item['bytes'], 'source pin: ' + name)
        require(digest(HERE / 'component.rs') == meta['fixture_sha256'], 'initial freeze changed')
        projected = fidelity(meta, HERE / 'manual.rs')
        env = dict(os.environ, RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN='1.99.1',
                   PATH='/home/dev/.cargo/bin:/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin:' + os.environ['PATH'],
                   VERUS_Z3_PATH=str(runtime / 'z3'), VERUS_INFER_CACHE=str(out / 'cache'))
        inputs = [HERE / n for n in ('component.rs','corrected.rs','baseline.rs','manual.rs','interface.rs','callers.rs','controls.rs','private.json','replay.py')]
        guard = ArtifactGuard([*[runtime / n for n in pins], *inputs, *support_artifacts(),
                               private / 'source/bitmap.rs', private / 'source/buddy_allocator.rs'])
        receipt.update(fidelity=projected, fixture_sha256={p.name: digest(p) for p in inputs})
        run = LoggedCommands(out, out, receipt['commands'], timeout=120, guard=guard)

        def check(label, path, expected, *, flags=(), execs=EXEC, proofs=PROOF, artifact=None, failure=None):
            case_guard = ArtifactGuard([path])
            code = run(label, [str(runtime / 'verus'), str(path), '--edition=2024', '--no-cheating',
                              '--output-json', '--multiple-errors', '20', *flags], env)
            case_guard.check()
            stdout = out / (label + '.stdout'); stderr = out / (label + '.stderr')
            data = json.loads(stdout.read_text()); error = stderr.read_text()
            v = data['verification-results']; inf = data['verus-infer']; scope = inf['verification_scope']
            helpers = inf.get('functions', []); require_exact(helpers)
            require(not helpers, 'unexpected inferred helper identities')
            resource = 'resource limit' in error.lower()
            require(not any(t in error.lower() for t in ('panicked at', 'timed out')), 'internal/timeout outcome')
            frontend = v.get('encountered-vir-error') is True or ('success' not in v and v.get('encountered-error') is True)
            success = code == 0 and v.get('success') is True
            require(success is expected, 'unexpected outcome: ' + label)
            require(scope['native_collection_completed'] is True, 'incomplete native inventory')
            if success:
                require(not resource and scope['complete_selected_scope_verified'] is True, 'uncertified positive')
                for kind, names in [('exec',execs), ('proof',proofs)]:
                    checked = [b['rust_function'] for b in scope['native_functions'] if b['body_status'] == 'checked_' + kind + '_body']
                    require(sorted(checked) == sorted(names) and scope['checked_' + kind + '_bodies'] == len(names), 'checked ' + kind + ' identities: ' + label)
            else:
                require(scope['checked_exec_bodies'] == scope['checked_proof_bodies'] == 0, 'negative certified bodies')
                selected = [b['rust_function'] for b in scope['native_functions'] if b['body_status'] == 'selected_body_not_certified_by_successful_full_run']
                require(sorted(selected) == sorted([*execs,*proofs]), 'negative body inventory: ' + label)
                if failure:
                    require(not frontend and not resource, 'not a semantic negative')
                    require_failure_kind(error, failure)
                    require(not any(t in error for t in ('possible arithmetic underflow/overflow','invariant not satisfied','recommendation not met','error[E')), 'mixed negative')
            if artifact is not None:
                require_publication(data, success, artifact)
            receipt['outcomes'].append(dict(label=label, success=success, frontend=frontend, resource=resource,
                verification=v, scope=scope, inferred_helpers=helpers, goal_assistance=inf.get('goal_assistance'),
                interface_written=inf['interface_written'], stderr_sha256=digest(stderr), stdout_sha256=digest(stdout)))
            return data

        eligible = out / 'eligible.rs'
        eligible.write_text((HERE / 'baseline.rs').read_text().replace('t[h].len == words(t[h+1].len)', '#[trigger] t[h].len == words(t[h+1].len)'))
        manual = out / 'manual.rs'; shutil.copyfile(HERE / 'manual.rs', manual)
        for name, flags in [('ordinary',()), ('infer',('--infer-goals',))]:
            check('baseline_' + name, eligible, False, flags=flags, proofs=[])
            check('manual_' + name, manual, True, flags=flags)
        check('manual_infer_warm', manual, True, flags=['--infer-goals'])
        # Healthy helper bodies are unchanged from the positive fixture.
        controls = out / 'controls.rs'; controls.write_text(manual.read_text() + (HERE / 'controls.rs').read_text())
        for kind in ('leaf','count','frame'):
            artifact = out / ('negative_' + kind + '.vir')
            check('negative_' + kind, controls, False,
                  flags=['--infer-goals','--cfg','negative_' + kind,'--export',str(artifact)],
                  artifact=artifact, failure='postcondition not satisfied',
                  execs=EXEC + [{'leaf':'wrong_set_membership','count':'wrong_repeat_count','frame':'wrong_clear_frame'}[kind]])
        api = out / 'api.rs'; api.write_text(manual.read_text() + (HERE / 'interface.rs').read_text())
        interface = out / 'api.vir'; library = out / 'libapi.rlib'
        check('provider', api, True, flags=['--crate-type=lib','--crate-name=r2_api','--compile',
              '--export',str(interface),'-o',str(library),'--infer-goals'], artifact=interface,
              execs=EXEC + ['CheckedBitmap::mark_allocated','CheckedBitmap::mark_free'])
        provider_guard = ArtifactGuard([interface,library])
        consumer = out / 'callers.rs'; shutil.copyfile(HERE / 'callers.rs',consumer)
        for label, flags in [('caller_ordinary',[]),('caller_infer',['--infer-goals'])]:
            check(label, consumer, True, flags=['--crate-type=lib','--extern','r2_api='+str(library),
                  '--import','r2_api='+str(interface),*flags],
                  execs=['record_alloc_membership','free_membership','repeat_alloc_membership'], proofs=[])
            provider_guard.check()
        receipt['application_property_proved'] = True
        guard.check()
    print(out)


if __name__ == '__main__':
    main()
