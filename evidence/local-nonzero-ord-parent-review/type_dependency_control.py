"""Narrow synthetic checker control; own private cfg(test)-only snapshot."""
from build import *
r=root();v=r/'type-dependency-control-src';need(not v.exists(),'immutable variant')
shutil.copytree(r/'src',v,ignore=shutil.ignore_patterns('.git','target-verus','z3'))
p=v/'source/rust_verify/src/checked_nonzero_ord_tests.rs';original=p.read_text()
anchor='        // Additional model/type/operand/constructor changes in the actual\n'
need(original.count(anchor)==1,'single actual Function control anchor')
addition='''        // Independent review control: actual semantic type kept; remove only
        // its embedded ZeroablePrimitive implementation dependency witness.
        let mut changed = original.x.clone();
        let old_param = changed.params[0].clone();
        let mut param = old_param.x.clone();
        let TypX::Decorate(decoration, allocator, inner) = &*param.typ else { unreachable!() };
        let TypX::Datatype(path, arguments, dependencies) = &**inner else { unreachable!() };
        assert!(!dependencies.is_empty(), "actual compiler type has dependency witness");
        param.typ = Arc::new(TypX::Decorate(decoration.clone(), allocator.clone(),
            Arc::new(TypX::Datatype(path.clone(), arguments.clone(), Arc::new(vec![])))));
        Arc::make_mut(&mut changed.params)[0] = old_param.new_x(param);
        let accepted = check_ordering_function(
            &ctxt, caller, e.span, &ids, &carrier, &get, &primitive, &models,
            &original.new_x(changed)).is_ok();
        println!("REVIEW_EMBEDDED_TYPE_DEPENDENCY_CONTROL accepted={accepted}");
'''
p.write_text(original.replace(anchor,addition+anchor))
current=manifest(v);base=manifest(r/'src');changed={n for n in base if current[n]!=base[n]}
need(changed=={'source/rust_verify/src/checked_nonzero_ord_tests.rs'},'cfg(test)-only variant')
(v/'source/target-verus').mkdir();(v/'source/target-verus/release').symlink_to(bundle(),target_is_directory=True)
write(r/'type-dependency-control-inputs.json',dict(base_source=base,variant_source=current,only_cfg_test_changed=True,test_patch_sha256=hashlib.sha256(addition.encode()).hexdigest(),tools={n:sha(bundle()/n) for n in TOOLS}))
j=run('review-type-dependency-test-build',[TC/'bin/cargo','test','--offline','--locked','-j','2','--manifest-path',v/'source/Cargo.toml','-p','rust_verify','--lib','--no-run','--message-format=json'])
need(j['exit']==0,'structural variant build setup')
a=[]
for line in (r/'review-type-dependency-test-build/stdout').read_text().splitlines():
 try:d=json.loads(line)
 except ValueError:continue
 if d.get('reason')=='compiler-artifact' and d.get('executable') and d['target']['name']=='rust_verify' and 'lib' in d['target']['kind']:a.append(d['executable'])
need(len(a)==1,'one actual loaded compiler unit')
j=run('review-type-dependency-actual-function',[a[0],'checked_nonzero_ord::tests::local_ordering_whole_mir_and_actual_function_structural_controls','--exact','--nocapture','--test-threads=1'])
need(j['exit']==0,'structural setup failure, excluded')
need(manifest(v)==current and manifest(r/'src')==base,'variant or positive source drift')
need('REVIEW_EMBEDDED_TYPE_DEPENDENCY_CONTROL accepted=true' in (r/'review-type-dependency-actual-function/stdout').read_text(),'coverage gap not reproduced')
write(r/'type-dependency-control-result.json',dict(accepted_cleared_param_embedded_impl_paths=True,classification='synthetic actual compiler-loaded MIR/actual Function clone; no semantic false-success claim',only_cfg_test_changed=True,base_source_sha256=hashlib.sha256(json.dumps(base,sort_keys=True).encode()).hexdigest(),variant_source_sha256=hashlib.sha256(json.dumps(current,sort_keys=True).encode()).hexdigest(),build_receipt_sha256=sha(r/'review-type-dependency-test-build/receipt.json'),run_receipt_sha256=sha(r/'review-type-dependency-actual-function/receipt.json'),production_tools_unchanged=all(sha(bundle()/n)==h for n,h in json.loads((r/'bundle-final-tools.json').read_text())['tools'].items())))
