#!/usr/bin/env python3
"""Read-only authentication of completed outputs; never follow historical repo roots."""
import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
sys.dont_write_bytecode = True
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
COMMIT = 'd639f378db2f701ffea26cef3c52950bada392bb'
GMP = '332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37'

def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def load(p):
    return json.loads(p.read_bytes())

def need(ok, message):
    if not ok:
        raise RuntimeError(message)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--checkout', type=Path, required=True)
    ap.add_argument('--fresh', type=Path)
    ap.add_argument('--output', type=Path, required=True)
    a = ap.parse_args()
    root = a.checkout.resolve()
    research = root/'source/tools/research'
    here = research/'model_mutable_writer'
    frozen = load(here/'trial016-freeze.json')
    # Only a prepared, committed checkout is read, never a receipt's old workspace.
    for name, expected in frozen['files'].items():
        need(name.startswith('source/tools/'), 'unexpected source name')
        need(sha(root/name) == expected, 'freeze mismatch: '+name)
    for module, expected in frozen['constants'].items():
        text = (here/'candidates'/(module+'.v')).read_text()
        actual = re.findall(r'^\s*(?:Definition|Fixpoint|Lemma|Theorem|Corollary)\s+(\w+)', text, re.M)
        need(actual == expected, 'source declarations: '+module)
        need(not re.search(r'\b(Axiom|Admitted|admit|Parameter|Abort|native_compute|vm_cast_no_check|Program|Obligation|Inductive|Record|Instance)\b', text),
             'unexpected declaration/unchecked construct: '+module)
    baseline_pin = load(research/'mutable_buffer_goal/foundation-baseline.json')
    ready_path = BASE/'rocq-toolchain/baseline-ready.json'
    need(sha(ready_path) == baseline_pin['readiness']['sha256'], 'readiness hash')
    ready = load(ready_path)
    baseline_path = BASE/'rocq-toolchain/durable-build-2/receipt.json'
    need(ready['receipt'] == str(baseline_path), 'durable receipt path')
    need(sha(baseline_path) == baseline_pin['receipt']['sha256'], 'baseline hash')
    baseline = load(baseline_path)
    need(ready['source_commit'] == '115a618dd35ff8337112cf2081a7f5391d3973d5', 'foundation revision')
    need(ready['baseline_kernel_checked'] is True and baseline['success'] is True, 'baseline completion')
    need({'prover-version', 'build-baseline', 'kernel-check-baseline'} <=
         {c['label'] for c in baseline['commands'] if c.get('returncode') == 0}, 'baseline stages')
    build = BASE/'verus-belt-build-115a618d'
    need(ready['build'] == str(build), 'build path')
    sources = baseline['copied_sources']
    need(len(sources) == 523, 'source count')
    for name, expected in sources.items():
        p = build/name
        need(p.resolve().is_relative_to(build), 'source outside baseline')
        need(sha(p) == expected, 'baseline source hash: '+name)
    binding = load(research/'native_semantic_bridge/baseline-bindings.json')
    objects = {str(p.relative_to(build)):sha(p) for p in sorted((build/'src').rglob('*.vo'))}
    map_hash = hashlib.sha256(json.dumps(objects, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    need(len(objects) == 284 and map_hash == binding['compiled_module_map_sha256'], 'baseline objects')
    manifest = BASE/'rocq-toolchain/gmp-private/manifest.json'
    need(sha(manifest) == GMP == baseline_pin['toolchain']['GMP_packages']['sha256'], 'GMP pin')
    need(ready['runner'] == str(BASE/'rocq-toolchain/run-rocq'), 'runner location')
    for name in ['runner', 'opam', 'coqc', 'coqchk']:
        p = Path(ready['switch'])/'_opam/bin'/name if name in ['coqc', 'coqchk'] else Path(ready[name])
        need(sha(p) == binding['tools'][name], 'tool hash: '+name)
    sys.path.insert(0, str(research/'native_semantic_bridge'))
    from proof_checks import parse_axioms, parse_kernel_summary

    def authenticate(receipt_path, metadata, modules):
        if metadata:
            need(str(receipt_path) == metadata['receipt']['path'], 'receipt path')
            need(sha(receipt_path) == metadata['receipt']['sha256'], 'receipt hash')
        d = load(receipt_path)
        need(d['success'] is True and d['model_proofs_kernel_checked'] is True, 'incomplete output')
        need(d['publication'] is False and d['source_entry_proved'] is False and d['native_effects_discharged'] == 0,
             'unexpected scope')
        expected_names = [n+'.'+c for n in modules for c in frozen['constants'][n]]
        need(list(d['assumptions']) == expected_names and d['constant_count'] == len(expected_names), 'constant inventory')
        expected_assumptions = load(here/'observation-results.json')['assumptions']
        need(d['assumptions'] == {n:expected_assumptions[n] for n in expected_names}, 'extra/different assumptions')
        if metadata:
            need(d['assumptions'] == metadata['assumptions'], 'metadata assumptions')
        # Historical source keys are identifiers, not paths to open.
        input_hashes = {}
        for key, facts in d['inputs'].items():
            marker = '/source/tools/'
            if marker in key:
                relative = 'source/tools/'+key.split(marker, 1)[1]
                need(relative not in input_hashes, 'ambiguous historical source')
                input_hashes[relative] = facts['sha256']
        historical_freeze = load(here/('trial014-freeze.json' if len(expected_names) == 6 else 'trial016-freeze.json'))
        for name, expected in historical_freeze['files'].items():
            need(input_hashes.get(name) == expected, 'receipt frozen input: '+name)
        for n in modules:
            name = 'source/tools/research/model_mutable_writer/candidates/'+n+'.v'
            need(input_hashes.get(name) == frozen['files'][name] == sha(root/name), 'source identity: '+n)
        # Completed local proof directory only; do not follow embedded paths.
        work = receipt_path.parent/'proofs'
        products = {}
        for key, facts in d['proof_artifacts'].items():
            name = Path(key).name
            need(name not in products and '/' not in name, 'ambiguous artifact')
            need(sha(work/name) == facts['sha256'], 'artifact hash: '+name)
            products[name] = facts['sha256']
        need(set(products) == {n+s for n in [*modules,'Assumptions'] for s in ['.v','.vo']}, 'artifact inventory')
        for n in modules:
            need(products[n+'.v'] == sha(here/'candidates'/(n+'.v')), 'copied source: '+n)
        if metadata:
            need({n:v for n,v in products.items() if n.endswith('.vo')} == metadata['proof_objects'], 'object metadata')
        stages = ['compile-'+n for n in modules]+['assumptions','kernel']
        need([c['label'] for c in d['commands']] == stages, 'stage inventory')
        need(all(c['exit_code'] == 0 for c in d['commands']), 'failed stage')
        for c in d['commands']:
            need(c['argv'][0] == ready['runner'], 'unapproved runner')
        kernel = d['commands'][-1]['argv']
        need(kernel[1] == str(Path(ready['switch'])/'_opam/bin/coqchk') and '-o' in kernel,
             'kernel checker')
        need(not any(x in kernel for x in ['-norec','-admit','-quick','-vos','-vok']), 'nonrecursive/quick checker')
        need(kernel[-len(modules)-1:] == ['ModelMutableWriter.'+n for n in [*modules,'Assumptions']], 'kernel targets')
        text = (receipt_path.parent/'assumptions.stdout').read_text()
        sections = re.findall(r'^BEGIN (\S+)\n(.*?)^END (\S+)\s*$', text, re.M|re.S)
        need([n for n,_,_ in sections] == expected_names == [n for _,_,n in sections], 'assumption markers')
        need({n:parse_axioms(body) for n,body,_ in sections} == d['assumptions'], 'assumption parser')
        need(parse_kernel_summary((receipt_path.parent/'kernel.stderr').read_text()) == d['kernel_axioms'], 'kernel parser')
        need(set(d['kernel_axioms']) == set(expected_assumptions['observed.raw_model_transform_observed']), 'kernel axioms')
        return {'path':str(receipt_path), 'sha256':sha(receipt_path), 'success':True,
                'stages':{c['label']:c['exit_code'] for c in d['commands']},
                'artifacts':products, 'assumptions':d['assumptions'], 'kernel_axioms':d['kernel_axioms'],
                'log_sha256':{p.name:sha(p) for p in sorted(receipt_path.parent.iterdir())
                              if p.suffix in ['.stdout','.stderr']}}


    library_specs = [
        ('ModelNestedBorrowContext', Path('/tmp/model-nested-borrow-context-trial-053'),
         research/'model_nested_borrow_context/results.json', 'receipt', 'model_nested_borrow_context/candidates'),
        ('NativeHeapInitialization', BASE/'parent-heap-init-3d799b38',
         research/'mutable_buffer_goal/parent-heap-initialization.json', 'parent_replay', 'native_heap_initialization/candidates'),
        ('NativeSemanticBridge', BASE/'semantic-guard-validation-1efbd004/proofs',
         research/'mutable_buffer_goal/parent-semantic-foundation.json', 'review_fix_positive_receipt', 'native_semantic_bridge/candidates/loan-v12'),
        ('ModelInputContext', BASE/'parent-model-input-94b87742/proofs',
         research/'mutable_buffer_goal/parent-model-input-context.json', 'proof_receipt', 'model_input_context/candidates'),
        ('ModelStateObservation', BASE/'model-observation-trial017',
         research/'model_state_observation/results.json', 'receipt', 'model_state_observation/candidates'),
        ('ModelInputObservation', BASE/'input-observation-trial005',
         research/'model_input_observation/results.json', 'receipt', 'model_input_observation/candidates'),
    ]
    libraries = {}
    for namespace, directory, pin_path, key, candidates in library_specs:
        expected = load(pin_path)[key]
        receipt_path = directory/'receipt.json'
        need(expected['path'] == str(receipt_path) and sha(receipt_path) == expected['sha256'], 'dependency receipt pin')
        d = load(receipt_path)
        need(d['success'] is True and d['model_proofs_kernel_checked'] is True, 'incomplete dependency')
        products = {}
        for name, facts in d['proof_artifacts'].items():
            name = Path(name).name
            need(name not in products, 'duplicate dependency artifact')
            p = directory/'proofs'/name
            need(sha(p) == facts['sha256'], 'dependency artifact: '+str(p))
            products[name] = facts['sha256']
            if p.suffix == '.v' and name not in ['Assumptions.v', 'AssumptionInventory.v']:
                need(sha(research/candidates/name) == facts['sha256'], 'prepared dependency source identity')
        libraries[namespace] = {'receipt':str(receipt_path),'sha256':sha(receipt_path),
                                'metadata':str(pin_path.relative_to(root)), 'artifacts':products}

    seven = authenticate(BASE/'mutable-writer-trial016/receipt.json', load(here/'observation-results.json'),
                         ['resource_protocol','child','initialized','transform','observed'])
    six = authenticate(BASE/'parent-mutable-transform-014/proofs/receipt.json', load(here/'transform-results.json'),
                       ['resource_protocol','child','initialized','transform'])
    for n in ['resource_protocol','child','initialized','transform']:
        need(seven['artifacts'][n+'.vo'] == six['artifacts'][n+'.vo'], 'inherited object difference')
    result = {'implementation':COMMIT, 'freeze_sha256':sha(here/'trial016-freeze.json'),
              'frozen_inputs':frozen['files'], 'declarations':frozen['constants'],
              'baseline':{'readiness_sha256':sha(ready_path), 'receipt_sha256':sha(baseline_path),
                          'source_commit':ready['source_commit'], 'sources':len(sources),'objects':len(objects),
                          'objects_map_sha256':map_hash,'GMP_manifest_sha256':GMP,'success':True},
              'dependencies':libraries, 'authenticated_seven':seven,'authenticated_six':six}
    if a.fresh:
        fresh = authenticate(a.fresh/'receipt.json', None, ['resource_protocol','child','initialized','transform','observed'])
        need(fresh['artifacts'] == seven['artifacts'], 'fresh source/object mismatch')
        result['fresh'] = fresh
    a.output.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({'authenticated':True, 'fresh':bool(a.fresh), 'output':str(a.output)}))

if __name__ == '__main__':
    main()
