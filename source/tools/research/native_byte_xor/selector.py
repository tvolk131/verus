"""Fail-closed typed payload selection. This is NOT a proof/importer/native rule."""
from copy import deepcopy
import hashlib
import json
from pathlib import Path
from infer_experiment import require

OUTER = 'f8aaaa6617dd7f28fa8dbdfe4479e5afc4cfec6f1f84ed5a9684562d09d4a727'
INNER = '878b4925d3d044d4a153437b0246d4a921b7a4819a867adc9a3225df75b18450'
RAW = 'bf1f279f9292fe0fe1e062989d5082293e3cace0cfd251017f3f024637cd436b'
CANONICAL = '71a94b7844e377ce134ca562905b6e46bb7486fb1a2094359999f271dc8e1505'

def canonical(x):
    return hashlib.sha256(json.dumps(x,sort_keys=True,separators=(',',':')).encode()).hexdigest()

def select(pin, ir, local_types):
    # Full dictionary equality rejects extra/unknown payload fields as well.
    require(pin['pointer_bits'] == 64, 'unsupported pointer width')
    expected = {'op':'assign',
        'destination':{'local':17,'projection':[{'op':'deref'}],'ty':'u8'},
        'value':{'op':'binary','kind':'BitXor',
          'left':{'op':'copy','place':{'local':17,'projection':[{'op':'deref'}],'ty':'u8'}},
          'right':{'op':'move','place':{'local':19,'projection':[],'ty':'u8'}}}}
    require(pin['expected_ir'] == expected and ir == expected, 'unknown instruction payload')
    require(local_types == {'17':{'ty':'&mut u8','shape':{'kind':'ref','mutable':True,'pointee':'u8'}},
                            '19':{'ty':'u8','shape':{'kind':'scalar_or_function','ty':'u8'}}},
            'unknown local type/width')
    return True

def authenticate_source(pin, results, outer, inner, retained):
    require(pin['source_receipt']['sha256'] == results['receipt']['sha256'] == OUTER and
            retained(Path(pin['source_receipt']['path'])) == OUTER, 'outer source receipt')
    require(pin['extraction_receipt']['sha256'] == results['inherited_receipt']['sha256'] == INNER and
            retained(Path(pin['extraction_receipt']['path'])) == INNER, 'inherited extraction receipt')
    require(outer['success'] is True and inner['success'] is True and
            outer['native_bridge_proved'] is False and inner['native_bridge_proved'] is False,
            'source receipt success/status')
    require(pin['native_json']['sha256'] == RAW and retained(Path(pin['native_json']['path'])) == RAW,
            'native raw JSON identity')
    require(pin['native_json']['canonical_json_sha256'] == results['summary']['evidence_sha256'] ==
            outer['summary']['evidence_sha256'] == CANONICAL, 'canonical source binding')
    require(results['summary'] == outer['summary'], 'source results/outer summary mismatch')
    require(pin['actual_native_correspondences'] == 0 and pin['publication'] is False and
            results['checked_source_correspondences'] == [] and results['native_bridge_proved'] is False,
            'source identity cannot establish correspondence')
    data = Path(pin['native_json']['path']).read_bytes()
    require(hashlib.sha256(data).hexdigest() == RAW, 'native JSON changed during load')
    evidence = json.loads(data)
    require(canonical(evidence) == CANONICAL, 'canonical JSON identity')
    require(evidence['root'] == pin['root'] == results['summary']['root'] == inner['summary']['root'] and
            evidence['pointer_bits'] == pin['pointer_bits'] == inner['summary']['pointer_bits'] and
            evidence['target'] == results['summary']['target'] == inner['summary']['target'] == 'x86_64-unknown-linux-gnu',
            'root/target binding')
    require(pin['block'] == 7 and pin['statement'] == 6, 'instruction location')
    roots = [i for i in evidence['instances'] if i['id'] == pin['root']]
    require(len(roots) == 1 and roots[0]['kind'] == 'mir', 'unique MIR root')
    blocks = [b for b in roots[0]['blocks'] if b['bb'] == 7]
    require(len(blocks) == 1, 'unique block')
    ir = blocks[0]['statements'][6]['ir']
    locals_ = [l for l in roots[0]['locals'] if l['local'] in [17,19]]
    require(sorted(l['local'] for l in locals_) == [17,19], 'unique locals')
    types = {str(l['local']):{'ty':l['ty'],'shape':l['shape']} for l in locals_}
    select(pin, ir, types)
    return dict(root=pin['root'],block=7,statement=6,ir=ir,local_types=types,
                raw_sha256=RAW,canonical_sha256=CANONICAL, source_identity_only=True,
                trusted_semantic_rule=False,native_correspondences=0)

def selector_controls(pin, ir, local_types):
    cases = []
    for name in ['BitOr','wrong-left-place','wrong-right-place','wrong-destination','wrong-width',
                 'Copy-instead-of-Move','Move-instead-of-Copy','unknown-field','pointer-width']:
        p, x, t = deepcopy(pin), deepcopy(ir), deepcopy(local_types)
        if name == 'BitOr': x['value']['kind'] = 'BitOr'
        elif name == 'wrong-left-place': x['value']['left']['place']['local'] = 18
        elif name == 'wrong-right-place': x['value']['right']['place']['local'] = 18
        elif name == 'wrong-destination': x['destination']['local'] = 18
        elif name == 'wrong-width': x['value']['left']['place']['ty'] = 'u16'
        elif name == 'Copy-instead-of-Move': x['value']['right']['op'] = 'copy'
        elif name == 'Move-instead-of-Copy': x['value']['left']['op'] = 'move'
        elif name == 'unknown-field': x['value']['unknown'] = True
        elif name == 'pointer-width': p['pointer_bits'] = 32
        try: select(p,x,t)
        except ValueError as error:
            cases.append(dict(case=name,rejected=True,reason=str(error),classification='in-memory selector control; not native semantic evidence'))
        else: raise ValueError('selector falsely accepted '+name)
    return cases
