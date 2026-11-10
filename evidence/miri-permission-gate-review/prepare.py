import hashlib, json, pathlib, subprocess, sys
HERE=pathlib.Path(__file__).resolve().parent
ROOT=HERE.parents[1]
def need(c,m):
    if not c: raise RuntimeError(m)
def sha(p): return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def save(p,x): p.write_text(json.dumps(x,indent=2,sort_keys=True)+'\n')
parent=ROOT/'source/tools/research/native_miri_permission_grants/freeze003.json'
f=json.loads(parent.read_bytes()); inputs={}; remap=[]
for p,h in f['inputs'].items():
    if p.startswith('/home/dev/projects/verus/'):
        rel=p.removeprefix('/home/dev/projects/verus/')
        q=ROOT/rel
        committed=subprocess.check_output(['git','show','8aa9fa72:'+rel],cwd=ROOT)
        need(hashlib.sha256(committed).hexdigest()==h==sha(q), 'own committed input differs: '+rel)
        remap.append(dict(old_label=p,own_path=str(q),sha256=h))
    else: q=pathlib.Path(p)
    need(sha(q)==h,'input mismatch: '+str(q)); inputs[str(q)]=h
f['inputs']=inputs
f['review_remapping']={'source_freeze_sha256':sha(parent),'own_head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'entries':remap,'external_inputs_unchanged':True}
save(HERE/'freeze-own.json',f)
paths=set(inputs)
for d in ['native_miri_permission_grants','stbor_entry_handoff','tree_borrows_entry_handoff']:
    paths.update(str(p) for p in (ROOT/'source/tools/research'/d).iterdir() if p.is_file())
paths.add(str(ROOT/'source/tools/research/mutable_buffer_goal/parent-entry-replay.json'))
paths.update(str(p) for p in HERE.iterdir() if p.is_file() and p.name!='pretrial-inventory.json')
# Fixed named completed evidence: do not follow checkout labels from receipts.
paths.update(['/tmp/native-miri-permission-grants-parent003/receipt.json',
'/tmp/native-miri-permission-grants-parent001/receipt.json',
'/tmp/native-miri-permission-grants-parent002/receipt.json',
'/tmp/verus-infer-buffer-cycle-20261109/stbor-controller-gate-transformation.json',
'/tmp/verus-infer-buffer-cycle-20261109/parent-entry-replay-eb7778b5/guard-bytecode-audit.json'])
b=pathlib.Path('/tmp/parent-stbor-entry-handoff-eb7778b5/accepted025')
paths.update(str(p) for p in b.rglob('*') if p.is_file())
component=(ROOT/'source/tools/research/native_miri_permission_grants/component.rs').read_text()
controls={name:hashlib.sha256(text.encode()).hexdigest() for name,text in [
('enabled-cold',component),('enabled-warm',component),('disabled',component),
('disabled-guard-removed',component.replace('self != Permission::Disabled','true')),
('read-inverted',component.replace('access == AccessKind::Read','access == AccessKind::Write')),
('readonly-guard-removed',component.replace('self != Permission::SharedReadOnly','true'))]}
save(HERE/'pretrial-inventory.json',dict(before_first_trial=True,files={p:{'sha256':sha(p),'bytes':pathlib.Path(p).stat().st_size} for p in sorted(paths)},control_source_sha256=controls,python=sys.version,executable=sys.executable,native_correspondences=0,publication=False,goal_complete=False))
print('Own freeze and pretrial inventory written; no verifier trial run.')
