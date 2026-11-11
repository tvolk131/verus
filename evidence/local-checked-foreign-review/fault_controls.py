"""Recorded semantic compiler controls in a private Git-byte source fixture only."""
import difflib,hashlib,json,shutil,subprocess
import review as t
from proofs import source
R=t.R;F=R/'contract-fixture';P=F/'source/rust_verify/src/checked_foreign.rs';Q=F/'source/rust_verify/src/checked_foreign_packing.rs'
# Freeze the completed extra-test module before resetting that private source copy.
contract=F/'source/vir/src/infer/contract.rs';identity=json.loads((F/'identity.json').read_text());t.need(t.sha(contract)==identity['original_contract_sha256'],'reset fixture changed');t.write(R/'extra-contract-identity.json',identity)
contract.write_bytes((t.ROOT/'source/vir/src/infer/contract.rs').read_bytes())
a=(t.ROOT/'source/rust_verify/src/checked_foreign.rs').read_text();q=(t.ROOT/'source/rust_verify/src/checked_foreign_packing.rs').read_text()
def replace(s,old,new):t.need(s.count(old)==1,'control patch unique');return s.replace(old,new)
cases={
 'false-proposal':(P,replace(a,'    let root = crate::rust_to_vir::get_root_module_path(ctxt);','    let ensure = e(&bool_typ(), ExprX::Const(Constant::Bool(false)));\n    let root = crate::rust_to_vir::get_root_module_path(ctxt);'),'actual','postcondition not satisfied'),
 'packing-proposal':(Q,replace(q,'ExpX::Const(Constant::Int(value))','ExpX::Const(Constant::Int(value + 1))'),'full-help','bitvector assertion not satisfied'),
 'generated-body':(P,replace(a,'    // Independent ordinary contract proposal, NOT inference-summary synthesis.','    let body = if let ExprX::If(c,a,Some(b)) = &body.x {body.new_x(ExprX::If(c.clone(),b.clone(),Some(a.clone())))}else{unreachable!()};\n    // Independent ordinary contract proposal, NOT inference-summary synthesis.'),'actual','correspondence failed'),
 'generated-type':(P,replace(a,'let u64t = Arc::new(TypX::Int(IntRange::U(64)));','let u64t = Arc::new(TypX::Int(IntRange::U(32)));'),'actual','correspondence failed'),
 'mir-local-type':(P,replace(a,'        let primitive = inspect(ctxt.tcx, env, &body).map_err(|e| {','        let mut body=body;body.local_decls[mir::RETURN_PLACE].ty=ctxt.tcx.types.u32;\n        let primitive = inspect(ctxt.tcx, env, &body).map_err(|e| {'),'actual','checked foreign MIR unsupported'),
 'mir-extra-block':(P,replace(a,'        let primitive = inspect(ctxt.tcx, env, &body).map_err(|e| {','        let mut body=body;let extra=body.basic_blocks[mir::START_BLOCK].clone();body.basic_blocks.as_mut().push(extra);\n        let primitive = inspect(ctxt.tcx, env, &body).map_err(|e| {'),'actual','checked foreign MIR unsupported'),
}
rows=[]
gitdir=subprocess.check_output(['git','rev-parse','--absolute-git-dir'],cwd=t.ROOT).decode().strip()
def fmanifest():
 return {str(p.relative_to(F)):t.sha(p) for top in ['source','dependencies','tools'] for p in sorted((F/top).rglob('*')) if p.is_file()}
for n,(p,code,kind,message) in cases.items():
 P.write_text(a);Q.write_text(q);p.write_text(code)
 d=R/('fault-'+n+'-complete');d.mkdir(exist_ok=True);(d/'mutation.patch').write_text(''.join(difflib.unified_diff((a if p==P else q).splitlines(True),code.splitlines(True),fromfile=str(p.relative_to(F)),tofile=str(p.relative_to(F)))))
 frozen=fmanifest();t.write(d/'fixture-source.json',frozen)
 r=json.loads((R/('fault-build-'+n+'-complete')/'receipt.json').read_text()) if (R/('fault-build-'+n+'-complete')/'receipt.json').exists() else t.run('fault-build-'+n+'-complete',[t.TC/'bin/cargo','build','--offline','--locked','-j','2','-p','rust_verify'],{'GIT_DIR':gitdir,'GIT_WORK_TREE':str(F),'CARGO_TARGET_DIR':str(R/'contract-target'),'CARGO_BUILD_BUILD_DIR':str(R/'contract-build')},cwd=F/'source')
 t.need(r['exit']==0,'control compilation '+n);t.need(fmanifest()==frozen,'control source drift')
 cb=d/'bundle';
 if not cb.exists():shutil.copytree(R/'bundle',cb,ignore=shutil.ignore_patterns('vstd'))
 if not (d/'actual-tools.json').exists():shutil.copy2(R/'contract-target/debug/rust_verify',cb/'rust_verify')
 tools={x:t.sha(cb/x) for x in t.TOOLS};t.write(d/'actual-tools.json',tools)
 src=d/'component.rs';src.write_text(source(kind))
 r=json.loads((R/('fault-proof-'+n+'-complete')/'receipt.json').read_text()) if (R/('fault-proof-'+n+'-complete')/'receipt.json').exists() else t.run('fault-proof-'+n+'-complete',[cb/'verus',src,'--crate-name=component','--crate-type=lib','--infer-goals','--no-cheating','--output-json','--multiple-errors','20','--compile','-o',d/'libcomponent.rlib'],{'VERUS_ROOT':str(cb)})
 t.need(r['exit']!=0 and message in (R/('fault-proof-'+n+'-complete')/'stderr').read_text(),'fault did not fail correctly '+n)
 t.need({x:t.sha(cb/x) for x in t.TOOLS}==tools and fmanifest()==frozen,'actual control tool/source drift')
 rows.append(dict(control=n,source_sha256=t.sha(d/'fixture-source.json'),patch_sha256=t.sha(d/'mutation.patch'),actual_tools=tools,verification=r.get('verification'),failed_functions=r.get('failed_functions')))
 t.write(R/'fault-controls.json',rows)
P.write_text(a);Q.write_text(q)
