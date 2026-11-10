#!/usr/bin/env python3
"""Compact the fixed private review runs and verify their additional log pins."""
import hashlib
import json
from pathlib import Path
import shutil

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
RUN = Path('/tmp/source-freeze-completeness-review-0912d2a7-final')
VALIDATION = Path('/tmp/source-freeze-completeness-review-0912d2a7-validation-final')


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def require(value, message):
    if not value:
        raise ValueError(message)


def write(name, value):
    (HERE/name).write_text(json.dumps(value, indent=2)+'\n')


validation = read(VALIDATION/'validation.json')
require(validation.pop('artifacts_after') == validation['artifacts_before'], 'changed input')
validation['raw_validation'] = dict(path=str(VALIDATION/'validation.json'), sha256=digest(VALIDATION/'validation.json'))
results = read(RUN/'results.json')
validation['raw_adversarial_results'] = dict(path=str(RUN/'results.json'), sha256=digest(RUN/'results.json'))
validation['cases'] = []
races = []
for item in results['cases']:
    receipt_path = Path(item['argv'][-1])/'receipt.json'
    require(digest(receipt_path) == item['receipt_sha256'], 'changed case receipt')
    receipt = item['receipt']
    compact = {k:v for k,v in item.items() if k != 'receipt'}
    compact['receipt_path'] = str(receipt_path)
    compact['success'] = receipt['success']
    compact['failure'] = receipt.get('failure')
    compact['publication'] = receipt['publication']
    compact['native_bridge_proved'] = receipt['native_bridge_proved']
    validation['cases'].append(compact)
    if item.get('stage') == 'between_validation_and_guard':
        private = receipt_path.parents[1]
        frozen = read(private/'freeze.json')
        require(digest(private/'freeze.json') == item['freeze_sha256'], 'changed repro freeze')
        pin = private/'source/tools/research/native_semantic_bridge/source-bindings.json'
        races.append(dict(name=item['name'], frozen_binding_sha256=frozen['files'][str(pin.relative_to(private))],
            actual_binding_sha256=digest(pin), binding=read(pin),
            actual_receipt_sha256=digest(private/'extraction/receipt.json'),
            extraction_receipt=read(private/'extraction/receipt.json'), accepted_receipt=receipt,
            freeze=frozen, controller_receipt_sha256=digest(receipt_path)))
committed = read(ROOT/'source/tools/research/native_semantic_bridge/source-obligations.json')
instances = committed.pop('instances')
for operation in committed['native_operations']:
    operation['instance'] = instances[operation['instance']]
for mode in ('normal', 'optimized'):
    actual = read(VALIDATION/(mode+'-source_map.py')/'source-obligations.json')
    require(actual == committed, 'map changed')
validation['fresh_maps_match_committed_after_instance_expansion'] = True
log_pins = {}
parent = read('/tmp/verus-infer-buffer-cycle-20261109/semantic-guard-validation-1efbd004/receipt.json')
for command in parent['commands']:
    require(command['returncode'] == 0 and digest(command['log']) == command['log_sha256'], 'parent log pin')
    log_pins[command['log']] = command['log_sha256']
entry = read(ROOT/'source/tools/research/native_source_entry/results.json')
for item in entry['logs'].values():
    require(digest(item['path']) == item['sha256'], 'entry log pin')
    log_pins[item['path']] = item['sha256']
validation['additional_authenticated_historical_logs'] = log_pins
for mode, index in [('normal', 1), ('optimized', 5)]:
    shutil.copyfile(VALIDATION/f'{index:02d}.stderr', HERE/f'unit-{mode}.log')
write('race-receipts.json', races)
write('validation.json', validation)
print(json.dumps(dict(validation_sha256=digest(HERE/'validation.json'),
    race_receipts_sha256=digest(HERE/'race-receipts.json'), cases=len(validation['cases']),
    extra_historical_logs=len(log_pins))))
