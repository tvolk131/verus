"""Review-only freeze path projection. Never read the live parent checkout."""
from pathlib import Path
import hashlib, json, subprocess
ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
CRASH = ROOT/'source/tools/research/redb_crash_protocol'
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
records = []
for name, source in [('selection', CRASH/'selection_contract/VALIDATION_FREEZE_2.json'),
                     ('storage-positive', CRASH/'storage_boundary/FROZEN_4_positive.json'),
                     ('storage-negative', CRASH/'storage_boundary/FROZEN_4_negative.json')]:
    freeze = json.loads(source.read_text())
    remapped = {}
    mapping = []
    for old, expected in freeze['inputs'].items():
        prefix = '/home/dev/projects/verus/'
        if not old.startswith(prefix): raise ValueError(old)
        new = ROOT/old[len(prefix):]
        actual = sha(new)
        if actual != expected: raise ValueError((new, expected, actual))
        committed = subprocess.check_output(['git','-C',str(ROOT),'show','03fd44ed:'+str(new.relative_to(ROOT))])
        if hashlib.sha256(committed).hexdigest() != expected: raise ValueError('parent mismatch')
        remapped[str(new)] = expected
        mapping.append(dict(original=old, remapped=str(new), sha256=expected))
    freeze['inputs'] = remapped
    target = HERE/(name+'-freeze.json')
    target.write_text(json.dumps(freeze,indent=2)+'\n')
    records.append(dict(original=str(source.relative_to(ROOT)), original_sha256=sha(source),
                        remapped=str(target.relative_to(ROOT)), remapped_sha256=sha(target), paths=mapping))
rt = freeze['runtime']
bundle = Path(rt['bundle'])
if sha(bundle/'receipt.json') != rt['receipt_sha256']: raise ValueError('receipt')
for n,h in rt['tool_hashes'].items():
    if sha(bundle/n) != h: raise ValueError(n)
if not (bundle/'verus-root').is_file(): raise ValueError('verus-root')
(HERE/'PREPARATION.json').write_text(json.dumps(dict(parent='03fd44eddf5562645c79e5be89ab0e16f2706db1',
    remappings=records, runtime=rt, verus_root_sha256=sha(bundle/'verus-root'),
    source_tree_sha256=subprocess.check_output(['git','rev-parse','03fd44ed:source'],text=True).strip()),indent=2)+'\n')
print('All freeze input hashes match committed parent; all eight tools, receipt and verus-root authenticated.')
