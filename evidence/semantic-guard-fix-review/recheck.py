#!/usr/bin/env python3
"""Evidence-only independent recheck. No baseline writes or native proof claims."""
import argparse, copy, hashlib, json, os, shutil, subprocess, sys
from pathlib import Path
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
BRIDGE = ROOT/'source/tools/research/native_semantic_bridge'
RAW = Path('/tmp/native-semantic-bridge-e4a8dcab/source-v1')
READY = Path('/tmp/verus-infer-buffer-cycle-20261109/rocq-toolchain/baseline-ready.json')
FREEZE = BRIDGE/'review-fix-freeze.json'

def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()

def need(ok, why):
    if not ok: raise RuntimeError(why)

def save(p, obj):
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(obj, indent=2)+'\n')

def mirror(out):
    dest = out/'private-controller-tree'
    f = json.loads(FREEZE.read_text())
    paths = set(f['files']) | {str(FREEZE.relative_to(ROOT)),
        'source/tools/infer_experiment.py', 'source/tools/infer_eval_support.py',
        'source/tools/research/native_heap_lowering/fixtures/drop.rs',
        str((BRIDGE/'test_proof_checks.py').relative_to(ROOT))}
    for n in sorted(paths):
        target = dest/n
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT/n, target)
        need(sha(target) == sha(ROOT/n), 'mirror byte mismatch '+n)
    save(out/'mirror.json', {n:sha(dest/n) for n in sorted(paths)})
    return dest

def run(out, label, cmd, cwd=ROOT):
    out.mkdir(parents=True, exist_ok=True)
    r = subprocess.run(list(map(str,cmd)), cwd=cwd, capture_output=True, text=True)
    (out/(label+'.stdout')).write_text(r.stdout)
    (out/(label+'.stderr')).write_text(r.stderr)
    return dict(label=label, argv=list(map(str,cmd)), cwd=str(cwd), returncode=r.returncode,
                stdout_sha256=sha(out/(label+'.stdout')), stderr_sha256=sha(out/(label+'.stderr')))

def omission(out):
    dest = mirror(out)
    bridge = dest/BRIDGE.relative_to(ROOT)
    raw = out/'synthetic-extraction'
    raw.mkdir()
    for n in ('drop-native.json', 'drop-borrowck.json'):
        shutil.copyfile(RAW/n, raw/n)
    save(raw/'receipt.json', {'success':True})
    pin = bridge/'source-bindings.json'
    original = json.loads(pin.read_text())
    full = json.loads(FREEZE.read_text())
    omitted = copy.deepcopy(full)
    del omitted['files'][str(pin.relative_to(dest))]
    # Freeze exists BEFORE replacing the omitted binding with arbitrary metadata.
    save(out/'omitted-binding-freeze.json', omitted)
    save(out/'full-freeze.json', full)
    before = sha(pin)
    original['receipt_sha256'] = sha(raw/'receipt.json')
    save(pin, original)
    results = []
    for tool in ('source_map.py','audit.py'):
        for case, f, expected in [('full',out/'full-freeze.json',False),
                                  ('omitted',out/'omitted-binding-freeze.json',True)]:
            result = out/(tool+'-'+case)
            cmd = [sys.executable,'-B','-O',bridge/tool,'--freeze',f,
                   '--extraction',raw,'--output',result]
            record = run(out,tool+'-'+case,cmd,cwd=dest)
            receipt = json.loads((result/'receipt.json').read_text())
            need(receipt['success'] is expected, 'unexpected omission result')
            need(receipt['publication'] is False and receipt['native_bridge_proved'] is False,
                 'accidental native claim')
            record.update(receipt_sha256=sha(result/'receipt.json'), success=receipt['success'],
                          failure=receipt.get('failure'),
                          binding_in_guard=str(pin) in receipt.get('artifacts',{}))
            results.append(record)
    save(out/'omission-results.json',dict(binding_before=before,binding_after=sha(pin),
        fake_receipt_sha256=sha(raw/'receipt.json'),cases=results,semantic_negatives=0))
    print(json.dumps(results,indent=2))

if __name__ == '__main__':
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['mirror','omission']);p.add_argument('--output',type=Path,required=True)
    a=p.parse_args();a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
    if a.mode=='mirror': print(mirror(a.output))
    else: omission(a.output)
