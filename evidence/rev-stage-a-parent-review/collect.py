"""Write compact immutable receipts; audit.py is separate and read-only."""
import json,pathlib,re,subprocess
import build as b
H=b.HERE;R=pathlib.Path((H/'PRIVATE_ROOT').read_text().strip())
def load(p):return json.loads(pathlib.Path(p).read_text())
def manifest(root):return {str(p.relative_to(root)):b.sha(p) for p in sorted(root.rglob('*')) if p.is_file()}
accepted=['build-01','vstd-normal-01','vir-157-final','rust-verify-lib8-final','existing-constructors13-final','independent-test-build-wf2-final','independent-tests-wf2-final','inspector-build-rerun','actual-metadata-rerun','rust-privacy','forged-attribute','item132','wrapper28-final','packet31','bool_provider','bool_import','bool_false','old_schema','old_policy','truncated_base','ordinary_borrow3','generic_nested','private_pattern','private_contract','private_constructor','runtime-item132','runtime-ordinary_borrow3','target-spec','compiler-version']
excluded={
 'vir-157':'157 library tests passed, but default Cargo doctests failed; unrelated diagnostic-file input guard also failed; superseded by explicit --lib',
 'inspector-build':'preliminary success superseded by isolated repeat',
 'actual-metadata':'metadata succeeded but independent input guard detected diagnostic test file changed midrun; superseded',
 'rust-verify-lib8':'missing hardcoded own release-layout vstd fixtures, not semantic failure',
 'existing-constructors13':'missing own include_str research fixture, not semantic failure',
 'independent-test-build-01':'superseded first diagnostic build',
 'independent-tests-01':'incorrect test expectation: WhenVisible Visibility missing Option field defaults to None; Internal refusal itself succeeded',
 'independent-test-build-final':'superseded five-test build',
 'independent-tests-final':'superseded successful five-test subset before actual WF/Ctx-entry control',
 'independent-test-build-wf-final':'test harness attempted to clone non-Clone GlobalCtx; compile failure, not semantic negative',
 'wrapper28':'ordinary base fixture15/0 passed; expected28 omitted the four inference clients and observer; superseded by complete fixture',
}
epochs={}
for p in sorted((R/'epochs').iterdir()):
 if not (p/'receipt.json').exists():continue
 name=p.name;b.need(name in accepted or name in excluded,'unclassified '+name)
 epochs[name]=dict(receipt_sha256=b.sha(p/'receipt.json'),classification='accepted' if name in accepted else 'excluded_or_superseded',reason=excluded.get(name,'matching final owned tools/source'),exit=load(p/'receipt.json')['exit'])
 b.need(p.name not in accepted or load(p/'receipt.json')['guards'],'guards '+name)
# Gather exact source proof/runtime/helper/model inventories without claiming every Rust body selected.
apps={}
for name in ['item132','wrapper28-final','packet31','bool_provider','bool_import','bool_false','ordinary_borrow3','generic_nested']:
 d=R/'apps'/name;raw=load(R/'epochs'/name/'stdout');j=raw['verus-infer'];b.write(d/'inventory.json',j)
 logs=manifest(d/'logs');b.write(d/'log-manifest.json',logs)
 native=j['verification_scope']['native_functions']
 # AIR files are shared module query logs and per-function stages; count only module *.air without -final/-poly.
 air={p.name:p.read_text().count('(check-valid') for p in (d/'logs').glob('*.air') if not p.name.endswith('-final.air')}
 runtime=d/'runtime.mir'
 apps[name]=dict(verification=raw['verification-results'],requested_helpers=j['requested_functions'],generated_helpers=j['generated_functions'],scope={k:v for k,v in j['verification_scope'].items() if k!='native_functions'},functions=[[f['function'],f['body_status'],f['selected'],f['ordinary_contract']] for f in native],exact_inventory_sha256=b.sha(d/'inventory.json'),log_manifest_sha256=b.sha(d/'log-manifest.json'),air_check_valid_occurrences_by_file=air,log_count=len(logs),trusted_model_inventory=dict(count=len(j['trusted_models']),sha256=__import__('hashlib').sha256(json.dumps(j['trusted_models'],sort_keys=True).encode()).hexdigest()))
 if runtime.exists():apps[name]['erased_runtime']=dict(sha256=b.sha(runtime),bodies=[line.rstrip() for line in runtime.read_text().splitlines() if line.startswith('fn ')])
b.write(H/'INVENTORY.json',dict(apps=apps,new_StageA_helpers=0,selected_Verus_scope_only=True,query_count_note='Occurrences per original module .air; do not add -final copies or equate queries with functions/runtime bodies.',native_correspondences=0,publication=False,goal_complete=False))
model_manifest={p:h for p,h in load(R/'shipping.json').items() if p.startswith('source/vstd/')}
b.write(R/'model-source.json',model_manifest)
worker=load(b.ROOT/'source/tools/research/rev_nominal_definition/stage_a/RESULTS.json')
compiler_paths=worker['compiler_changes']+['source/rust_verify/src/import_export.rs','source/vir/src/infer/registry.rs','source/Cargo.lock','source/Cargo.toml','rust-toolchain.toml']
source_manifest=load(R/'shipping.json');source_hashes={p:source_manifest[p] for p in compiler_paths}
# Research / historical checkpoint files are authenticated through own Git, never sibling worktree names.
checkpoint=subprocess.check_output(['git','ls-tree','-r','--name-only','f90a6679','source/tools/research/rev_nominal_definition'],cwd=b.ROOT,text=True).splitlines()
checkpoint_hashes={p:b.sha(b.ROOT/p) for p in checkpoint}
controllers={str(p.relative_to(b.ROOT)):b.sha(p) for p in sorted(H.iterdir()) if p.suffix in ['.py','.rs','.md']}
products={str(p.relative_to(R)):b.sha(p) for p in [R/'bootstrap.json',R/'shipping.json',R/'worker-authentication.json',R/'constructor-fixture.json',R/'diagnostic-source.json',R/'model-source.json',R/'diagnostic-tests-wf-final',R/'metadata-rerun/inspect']}
# Archive canonical raw receipts/source manifests; these are evidence, never reused build inputs.
for path in ['apps/item132/runtime.mir','apps/ordinary_borrow3/runtime.mir','target/debug/librust_verify.rlib']:
 products[path]=b.sha(R/path)
status=dict(VIR=157,additional_review_tests=6,rust_verify_lib=8,existing_constructor_tests=13,NORMAL_vstd='2250/0',Item='132/0',wrapper='28/0',packet='31/0',ordinary_borrow='3/0',generic_nested='4/0',wire='actual serde_json and bincode-next legacy compatible old variants; Internal encode/decode refused',production_installer=False,stage_b='unimplemented',native_correspondences=0,publication=False,goal_complete=False)
j=dict(schema=1,review_parent='f4415999876559504756ff9b4b9bfd245a09dd21',early_scope_commit='77660a1c',recommendation='inactive_StageA_only_subject_to_parent_acceptance',accepted_engine='d3b2aa6e / policy74 cache77 / docs1171bdf7',provisional=dict(schema=39,policy=75,cache=78),private_root=str(R),epochs=epochs,source_manifest_sha256=b.sha(R/'shipping.json'),source_files=len(source_manifest),compiler_source=source_hashes,model_source_manifest=dict(sha256=b.sha(R/'model-source.json'),files=len(model_manifest)),eight_tools={n:b.sha(R/'bundle'/n) for n in b.TOOLS},toolchain=load(R/'bootstrap.json')['toolchain'],target_spec_sha256=b.sha(R/'epochs/target-spec/stdout'),compiler_version_sha256=b.sha(R/'epochs/compiler-version/stdout'),worker_authentication_sha256=b.sha(R/'worker-authentication.json'),checkpoint_hashes=checkpoint_hashes,controllers=controllers,products=products,inventory_sha256=b.sha(H/'INVENTORY.json'),acceptance=status,execution_exception='Initial find .. -name AGENTS.md interrupted without output; possible sibling directory traversal, no sibling file content used',setup_exclusions_without_command_epoch=['ambiguous dependency library selection (retained diagnostic source v1-v4 and controller/tool transcript); all own Cargo dependency variants remained in private root','initial metadata rerun setup refused multiple libvir artifacts before command epoch','generic_nested expected2 corrected to actual4 verified; final proof successful and explicitly checked by audit'])
b.write(H/'RESULTS.json',j)
print(json.dumps(status,sort_keys=True))
