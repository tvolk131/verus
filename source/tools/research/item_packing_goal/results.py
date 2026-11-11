"""Compact committed receipts/identities; raw evidence stays private."""
import collections,json,re,subprocess,sys
from pathlib import Path
import run as t
import check as audit
R=t.root()
def collect():
 paths=[p/'receipt.json' for p in sorted(R.iterdir()) if p.is_dir() and (p/'receipt.json').exists()]
 paths += [R/name/temp/'receipt.json' for name in ['stable-full','stable-reuse'] for temp in ['cold','warm']]
 receipts={}
 for p in paths:
  j=json.loads(p.read_text());d=p.parent;f=json.loads((d/'freeze.json').read_text())
  t.need(t.sha(d/'freeze.json')==j['freeze_sha256'] and t.sha(d/'stdout')==j['stdout_sha256'] and t.sha(d/'stderr')==j['stderr_sha256'],'receipt integrity '+str(p))
  receipts[str(d.relative_to(R))]=j|dict(receipt_sha256=t.sha(p),tools=f['tools'],fixture_sha256=f['fixtures'])
 inv=dict(full=audit.checked(R/'stable-full',True),reuse=audit.checked(R/'stable-reuse'))
 t.write(t.HERE/'INVENTORY.json',inv)
 # The only source mutation permitted between the actual source-control trials
 # and shipping proof is the owned arithmetic file. Original typed bodies match.
 for label in ['ablation-full','false-constant-full']:
  f=json.loads((R/label/'freeze.json').read_text());orig=json.loads((R/'stable-full/cold/freeze.json').read_text())
  t.need(set(f['source'])==set(orig['source']),'source set')
  changed=[p for p,h in f['source'].items() if orig['source'][p]!=h]
  t.need(changed==['source/rust_verify/src/checked_foreign_packing.rs'],'source-control scope')
  t.need(audit.source_bodies(R/label)==audit.source_bodies(R/'stable-full/cold'),'original source bodies unchanged')
 tools={n:t.sha(R/'bundle'/n) for n in t.TOOLS}
 for label in ['final-production-full','final-production-reuse','final-packing-tests']:
  f=json.loads((R/label/'freeze.json').read_text());t.need(f['tools']==tools,'final source-matching tools '+label)
 t.need((t.ROOT/'source/rust_verify/src/checked_foreign_packing.rs').read_bytes()==(R/'shipping.rs').read_bytes(),'shipping source restored')

 j=dict(base='d27f4dd94ff7d3f09abfeedf6cdb0aa7c3f44b9f',freeze_milestone='eea87441',accepted_engine='35bc7b12fff34551da06a9b81e65f1f20d530992',private_root=str(R),production_sha256=t.sha(t.ROOT/'source/rust_verify/src/checked_foreign_packing.rs'),tools=tools,receipts=receipts,toolchain=json.loads((R/'toolchain-identities.json').read_text()),build_inputs_manifest_sha256=t.sha(R/'index-git-input-identities.json'),parent_authentication_sha256=t.sha(R/'parent-authentication.json'),test_binary=json.loads((R/'final-test-binary.json').read_text()),final_production=json.loads((R/'final-production-comparison.json').read_text()),source_controls={name:t.sha(R/(name+'.rs')) for name in ['shipping','ablation','false-constant']},queries={'full':{'ordinary_root':27,'isolated_bv':116,'fresh_eq':3,'actual_AIR_files':118},'packet':{'ordinary_root':3,'isolated_bv':29,'fresh_eq':0,'actual_AIR_files':30}},validation=dict(compiler_tests=37,VIR_tests=149,matching_vstd=2250),exclusions=['sandbox startup loopback permission failure before command execution','initial commit identity missing; command-local identity used, no config writes','baseline-* 4 setup failures due to unbuilt VERUS_ROOT, corrected baseline2-*','extension1-full89/1 and extension1-reuse12/1 missing direct-return discovery; conservative failures','focused-build exit0 guardfalse during concurrent vstd outputs; setup-only, not accepted proof','focused-tests ENOENT inherited build-dir/profile layout; 13+12+3 passed, localforeign4/5; corrected fresh private profile localforeign5/5, packing3/3','first audit raw stable-source parent path mismatch; exact frozen source filename rebound, no typed/coordinate normalization'],status='implementation_complete_pending_independent_parent_acceptance',native=0,publication=False,goal_complete=False,unchanged=dict(canonical='parent6/0 inherited, not rerun',HMAC='1/1 UNPROVED unchanged, not rerun',open='all44+3/101'))
 t.write(t.HERE/'RESULTS.json',j)
 print('receipts',len(receipts),'tools',tools['rust_verify'])
if __name__=='__main__':
 t.need(sys.flags.dont_write_bytecode,'use -B');collect()
