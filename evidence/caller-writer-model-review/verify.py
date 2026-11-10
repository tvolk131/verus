#!/usr/bin/env python3
"""Read-only independent receipt audit; writes only this evidence dir/private trial.

Historical worktree prefixes are labels, always rebased onto this checkout.
No implementation changes, library builds, filesystem substitutions, or delegates.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
IMPL = ROOT/'source/tools/research/model_caller_writer'
TARGET = '2c4795b54e760dbd6aca9c30d29a67bacd566213'
BASE = Path('/tmp/verus-infer-buffer-cycle-20261109')
FROZEN = 'f41315ad53cb37fc827057bd8ad2f27811079e96c17eb120f1fbf1fb6b0ee928'
ORIGINAL = '2ce85b9ca96256319f3eaf078d4161809cb0d8e1bb3610935347cb40fcea3ed6'
GMP = '332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37'
FE = 'Coq.Logic.FunctionalExtensionality.functional_extensionality_dep'
EQ = 'Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq'
ALIASES = {'functional_extensionality_dep': FE,
           'FunctionalExtensionality.functional_extensionality_dep': FE,
           'Eqdep.Eq_rect_eq.eq_rect_eq': EQ}
NAMES = ['interface.xor_bytes', 'interface.caller_xor_body',
         'interface.caller_xor_procedure', 'interface.caller_xor_credit',
         'interface.caller_xor_contract', 'guarded.byte_child_transform_guarded',
         'caller.xor_bytes_exact', 'caller.ordinary_alive_under_child',
         'caller.caller_xor_verified']
HELPERS = [f'{m}.inventory_{side}_{i}' for m in ['Assumptions','ProofTerms']
           for i in range(9) for side in ['begin','end']]

def need(ok, why):
    if not ok:
        raise ValueError(why)

def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for block in iter(lambda: f.read(1024*1024), b''):
            h.update(block)
    return h.hexdigest()

def load(path):
    return json.loads(Path(path).read_bytes())

def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2, sort_keys=True)+'\n')

def historical_label(label):
    # Never open another checkout. Only selected source labels are rebased.
    if '/source/' in label and not label.startswith('/tmp/'):
        return ROOT/('source/'+label.split('/source/',1)[1])
    need(label.startswith('/tmp/'), 'unexpected historical label '+label)
    need('fullmodel' not in label and 'fullwriter' not in label,
         'forbidden live proof label')
    return Path(label)

def check_artifacts(artifacts, historical=False, metadata=False):
    for label, expected in artifacts.items():
        path = historical_label(label) if historical else Path(label)
        need('fullmodel' not in str(path) and 'fullwriter' not in str(path), 'forbidden input')
        need(sha(path)==expected['sha256'], 'artifact content changed '+str(path))
        if metadata:
            s = path.stat()
            need((s.st_size,s.st_mtime_ns,s.st_ino)==
                 (expected['size'],expected['mtime_ns'],expected['inode']),
                 'artifact metadata changed '+str(path))

def marked(path, names):
    out = Path(path).read_text()
    sections = re.findall(r'^BEGIN (\S+)\n(.*?)^END (\S+)\s*$',out,re.M|re.S)
    need([a for a,_,_ in sections]==names==[b for _,_,b in sections], 'marker inventory')
    need(len(re.findall(r'^BEGIN ',out,re.M))==len(names)==
         len(re.findall(r'^END ',out,re.M)), 'extra markers')
    tail = out.split('END '+names[-1],1)[1].strip().splitlines()
    need(tail==['Closed under the global context']*18, 'helper assumption closure')
    return {a:body.strip() for a,body,_ in sections}, sections

def assumptions(body):
    if body=='Closed under the global context': return []
    need(body.startswith('Axioms:\n'), 'unknown assumption output')
    found = []
    for line in body.splitlines()[1:]:
        m = re.match(r'^\s*(\S+)\s*:',line)
        if m:
            name = ALIASES.get(m[1],m[1]); need(name in [FE,EQ], 'new axiom '+name)
            found.append(name)
        else: need(bool(found) and (not line.strip() or line[:1].isspace()), 'malformed axiom')
    need(bool(found) and len(found)==len(set(found)), 'bad assumption inventory')
    return found

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--trial', type=Path, required=True)
    ap.add_argument('--resolution', action='store_true', help='compile frozen name-resolution commands')
    a = ap.parse_args(); trial = a.trial.resolve()
    need(not trial.is_relative_to(ROOT), 'raw trial must be private')
    status = load(IMPL/'STATUS.json')
    files = subprocess.check_output(['git','ls-tree','-r','--name-only',TARGET,
                                    'source/tools/research/model_caller_writer'],cwd=ROOT,text=True).splitlines()
    for name in files:
        need((ROOT/name).read_bytes()==subprocess.check_output(['git','show',TARGET+':'+name],cwd=ROOT),
             'implementation differs from target '+name)
    changed = subprocess.check_output(['git','diff','--name-only','bbbbd0ad^','--'],cwd=ROOT,text=True).splitlines()
    need(all(n.startswith('evidence/caller-writer-model-review/') for n in changed),
         'review changed paths outside owned evidence')
    for key in ['receipt','freeze','readiness','durable_baseline_receipt']:
        p = Path(status[key]['path'])
        need(sha(p)==status[key]['sha256'], 'handoff authentication '+key)
    need(sha(IMPL/'final-freeze.json')==FROZEN==sha(HERE/'review-freeze.json'), 'freeze identity')
    need(sha(status['receipt']['path'])==ORIGINAL, 'trial009 authentication')
    old = load(status['receipt']['path']); fresh = load(trial/'receipt.json')
    need(old['success'] and fresh['success'] and old['model_proofs_kernel_checked'] and
         fresh['model_proofs_kernel_checked'], 'unsuccessful model receipts')
    check_artifacts(old['inputs'], historical=True)
    check_artifacts(old['proof_artifacts'])
    check_artifacts(fresh['inputs'], metadata=True)
    check_artifacts(fresh['proof_artifacts'], metadata=True)
    freeze = load(HERE/'review-freeze.json')
    for name,value in freeze['files'].items(): need(sha(ROOT/name)==value, 'frozen source '+name)
    need(freeze['before_first_trial'] is True, 'missing pretrial freeze')
    # The original path prefix is historical; exact selected bytes are in this checkout.
    ready = load(status['readiness']['path']); baseline = load(ready['receipt'])
    trusted = load(IMPL.parent/'mutable_buffer_goal/foundation-baseline.json')
    binding = load(IMPL.parent/'native_semantic_bridge/baseline-bindings.json')
    need(ready['baseline_kernel_checked'] and baseline['success'], 'baseline unavailable')
    need(ready['source_commit']==status['foundation_commit']==binding['source_commit']==
         '115a618dd35ff8337112cf2081a7f5391d3973d5', 'foundation commit')
    need(sha(status['readiness']['path'])==trusted['readiness']['sha256'], 'ready trust')
    need(sha(ready['receipt'])==trusted['receipt']['sha256'], 'baseline trust')
    need(len(baseline['copied_sources'])==523, 'source count')
    build = Path(ready['build'])
    for name,value in baseline['copied_sources'].items(): need(sha(build/name)==value, 'foundation source')
    objects = {str(p.relative_to(build)):sha(p) for p in (build/'src').rglob('*.vo')}
    need(len(objects)==284 and hashlib.sha256(json.dumps(objects,sort_keys=True,separators=(',',':')).encode()).hexdigest()
         ==binding['compiled_module_map_sha256'], 'foundation object map')
    need({'prover-version','build-baseline','kernel-check-baseline'} <=
         {x['label'] for x in baseline['commands'] if x['returncode']==0}, 'baseline stages')
    switchbin = Path(ready['switch'])/'_opam/bin'
    tools = {n:sha(switchbin/n if n in ['coqc','coqchk'] else ready[n]) for n in ['coqc','coqchk','runner','opam']}
    need(tools==status['tools']==binding['tools'], 'tool bindings')
    need(sha(BASE/'rocq-toolchain/gmp-private/manifest.json')==GMP, 'GMP manifest')
    dependencies = load(IMPL/'dependency-pins.json')
    for label,dep in dependencies.items():
        d = Path(label); need(sha(d/'receipt.json')==dep['receipt_sha256'], 'dependency receipt')
        r = load(d/'receipt.json'); need(r['success'] and r['model_proofs_kernel_checked'], 'dependency unchecked')
        if 'committed_pin' in dep:
            need(load(IMPL.parent/dep['committed_pin'])['receipt']['sha256']==dep['receipt_sha256'], 'committed pin')
        else:
            need(load(IMPL.parent/'model_shared_byte_read/dependency-pins.json')[label]['receipt_sha256']==dep['receipt_sha256'],
                 'transitive receipt pin')
        need({p.name for p in (d/'proofs').iterdir() if p.suffix in ['.v','.vo']}==set(dep['artifacts']), 'dependency inventory')
        for name,value in dep['artifacts'].items(): need(sha(d/'proofs'/name)==value, 'dependency artifact')
    # Complete inventory, independent parsing of actual fresh output.
    inv = load(IMPL/'TRUST_INVENTORY.json')
    keys = ['constant_count','declaration_inventory','constructor_inventory','program_obligations',
            'assumptions','kernel_axioms','proof_terms','kernel_declarations','inventory_helper_theorems',
            'all_kernel_declarations','helper_assumptions']
    for key in keys: need(old[key]==fresh[key]==inv[key], 'inventory mismatch '+key)
    need(fresh['constant_count']==9 and fresh['kernel_declarations']==NAMES, 'source inventory')
    need(len(fresh['all_kernel_declarations'])==45 and
         set(fresh['all_kernel_declarations'])==set(NAMES+HELPERS), 'complete kernel inventory')
    need(fresh['constructor_inventory']==fresh['program_obligations']==[], 'generated inventory')
    need(fresh['helper_assumptions']=={n:[] for n in HELPERS}, 'all 36 helpers closed')
    bodies,_ = marked(trial/'assumptions.stdout',NAMES)
    need({n:assumptions(b) for n,b in bodies.items()}==fresh['assumptions'], 'fresh assumption parsing')
    _,terms = marked(trial/'proof-terms.stdout',NAMES)
    need({n:hashlib.sha256(b.encode()).hexdigest() for n,b,_ in terms}==fresh['proof_terms'], 'fresh term identities')
    kernel = (trial/'kernel.stderr').read_text()
    need(re.findall(r'checking cst:ModelCallerWriter\.(\w+\.\w+)',kernel)==fresh['all_kernel_declarations'], 'kernel log inventory')
    need(kernel.count('CONTEXT SUMMARY')==1, 'full kernel summary')
    for line in ['Theory: Set is predicative', 'Theory: Rewrite rules are not allowed',
                 'Constants/Inductives relying on type-in-type: <none>',
                 'Constants/Inductives relying on unsafe (co)fixpoints: <none>',
                 'Inductives whose positivity is assumed: <none>']:
        need(line in kernel.split('CONTEXT SUMMARY')[1], 'unsafe kernel summary')
    commands = fresh['commands']
    need([c['label'] for c in commands]==['compile-interface','compile-guarded','compile-caller','assumptions','proof-terms','kernel']
         and all(c['exit_code']==0 for c in commands), 'command stages')
    need(all('-q' in c['argv'] and Path(c['argv'][1]).name=='coqc' for c in commands[:-1]), 'normal coqc')
    kcmd = commands[-1]['argv']
    need(Path(kcmd[1]).name=='coqchk' and kcmd[2]=='-o' and kcmd[-5:]==
         ['ModelCallerWriter.'+m for m in ['interface','guarded','caller','Assumptions','ProofTerms']], 'kernel command')
    need(not set(kcmd)&{'-admit','-norec','-unsafe','-silent'}, 'kernel exclusion')
    old_art = {Path(n).name:v['sha256'] for n,v in old['proof_artifacts'].items()}
    new_art = {Path(n).name:v['sha256'] for n,v in fresh['proof_artifacts'].items()}
    need({n:v for n,v in old_art.items() if n.endswith('.v')}==
         {n:v for n,v in new_art.items() if n.endswith('.v')}, 'source or generated helper identity')
    # No definitions/proofs here: only frozen Locate/Check/Print commands.
    audit = trial/'resolution'
    if a.resolution:
        audit.mkdir(exist_ok=False)
        need(not re.search(r'\b(Definition|Lemma|Theorem|Axiom|Parameter|Inductive|Record|Instance|Program|Obligation)\b',
                           (HERE/'Resolution.v').read_text()), 'resolution adds declarations')
        freeze_audit = {str(HERE/'Resolution.v'):sha(HERE/'Resolution.v'), str(HERE/'verify.py'):sha(HERE/'verify.py')}
        save(audit/'freeze.json',freeze_audit)
        shutil.copyfile(HERE/'Resolution.v',audit/'Resolution.v')
        argv = commands[0]['argv'][:-1]+['-Q',str(audit),'ModelCallerWriterReview',str(audit/'Resolution.v')]
        env = dict(os.environ,PATH=str(switchbin)+':'+os.environ['PATH'])
        for n in ['COQPATH','COQLIB','COQCORELIB','OCAMLPATH','CAML_LD_LIBRARY_PATH','LD_PRELOAD','LD_LIBRARY_PATH']:
            env.pop(n,None)
        with (audit/'stdout').open('w') as stdout,(audit/'stderr').open('w') as stderr:
            p = subprocess.run(argv,cwd=audit,env=env,stdout=stdout,stderr=stderr,timeout=180)
        save(audit/'receipt.json',dict(argv=argv,exit_code=p.returncode,freeze=freeze_audit))
        need(p.returncode==0, 'resolution compilation')
        for path,value in freeze_audit.items(): need(sha(path)==value, 'resolution frozen input')
        need(sha(audit/'Resolution.v')==sha(HERE/'Resolution.v'), 'resolution copy')
    need(load(audit/'receipt.json')['exit_code']==0, 'missing successful resolution receipt')
    resolved = (audit/'stdout').read_text()
    need('ModelSharedByteRead.shared_read.shared_key_view' in resolved, 'qualified shared view')
    check_artifacts(fresh['inputs'], metadata=True)
    check_artifacts(fresh['proof_artifacts'], metadata=True)
    check_artifacts(old['inputs'], historical=True)
    check_artifacts(old['proof_artifacts'])
    summary = dict(schema=1,review_target=TARGET,success=True,findings=[],
        scope_commit='bbbbd0ad',model_only=True,publication=False,goal_complete=False,
        source_entry_proved=False,native_correspondences_proved=0,minimal_native_operations_open=44,
        native_phases_open=3,native_effects_open=101,full_HMAC='1/1 open',
        original_receipt=dict(path=status['receipt']['path'],sha256=ORIGINAL,authenticated=True),
        fresh_receipt=dict(path=str(trial/'receipt.json'),sha256=sha(trial/'receipt.json')),
        pretrial_freeze=dict(path=str((HERE/'review-freeze.json').relative_to(ROOT)),sha256=FROZEN,
                            identical_to_completed_freeze=True),
        foundation=dict(commit=ready['source_commit'],source_count=523,object_count=284,
                        ready=status['readiness'],baseline=status['durable_baseline_receipt'],GMP_manifest_sha256=GMP,tools=tools),
        original_inputs_authenticated=len(old['inputs']),fresh_inputs_retained=len(fresh['inputs']),
        original_products_authenticated=len(old_art),fresh_products_retained=len(new_art),
        source_and_generated_helper_hashes_equal=True,object_hashes_equal=old_art==new_art,
        source_and_object_hashes=dict(original=old_art,fresh=new_art),
        dependency_libraries=len(dependencies),external_frozen_artifacts=freeze['external_count'],
        source_declarations=9,inventory_helpers=36,total_local_kernel_declarations=45,
        constructors=0,program_obligations=0,generated_instances=0,new_axioms=0,new_primitives=0,
        assumptions=fresh['assumptions'],helper_assumptions=fresh['helper_assumptions'],kernel_axioms=fresh['kernel_axioms'],
        commands=[dict(label=c['label'],exit_code=c['exit_code'],seconds=c['seconds']) for c in commands],
        private_raw_logs={p.name:sha(p) for p in sorted(trial.iterdir()) if p.suffix in ['.stdout','.stderr']},
        resolution=dict(qualified_shared_view='ModelSharedByteRead.shared_read.shared_key_view',
                        source_sha256=sha(HERE/'Resolution.v'),receipt_sha256=sha(audit/'receipt.json'),
                        stdout_sha256=sha(audit/'stdout'),additional_declarations=0),
        historical_worktree_paths='labels only, rebased onto current checkout before content verification',
        retained_inputs_and_products='all content hashes checked; fresh metadata also checked after probe')
    save(HERE/'RECEIPT.json',summary)
    save(HERE/'INVENTORY.json',{key:fresh[key] for key in keys})
    # Stable content-only inventories, not raw compiler output or historical metadata.
    save(HERE/'INPUT_HASHES.json',dict(
        original={str(historical_label(n)):v['sha256'] for n,v in old['inputs'].items()},
        fresh={n:v['sha256'] for n,v in fresh['inputs'].items()}))
    print(json.dumps(dict(success=True,source_declarations=9,helpers=36,retained_inputs=len(fresh['inputs']),
                          object_hashes_equal=old_art==new_art,qualified_view=summary['resolution']['qualified_shared_view'])))

if __name__=='__main__': main()
