"""Independent embedded-witness-only clones at real registration and post-WF sites.
Diagnostic compiler variant only; all positive proofs use the unchanged frozen build.
"""
from build import *
r=root()
def prepare():
 v=r/'structural-src';need(not v.exists(),'immutable variant')
 shutil.copytree(r/'src',v,ignore=shutil.ignore_patterns('.git','target-verus','z3'))
 p=v/'source/rust_verify/src/checked_nonzero_ord.rs';s=p.read_text()
 old='''        check_ordering_function(
            ctxt, caller, e.span, &ids, &carrier, &get, &primitive, &models, f,
        )?;'''
 new=old+'''
        review_witness_clones(ctxt, caller, e.span, &ids, &carrier, &get, &primitive, &models, f, "registration")?;'''
 need(s.count(old)==1,'registration anchor');s=s.replace(old,new)
 old='''        &models,
        f,
    )
}'''
 new='''        &models,
        f,
    )?;
    review_witness_clones(ctxt, c.caller, c.source.span, &ids, &carrier, &get, &primitive, &models, f, "post-WF-autospec-modes")
}'''
 need(s.count(old)==1,'post-boundary anchor');s=s.replace(old,new)
 addition=r'''
// REVIEW ONLY: this code calls the unchanged actual validator on independent
// clones. It never returns a mutated Function to translation or proof.
fn review_type(t: &Typ, edit: usize) -> Option<Typ> {
    match t.as_ref() {
        TypX::Decorate(k, a, inner) => Some(Arc::new(TypX::Decorate(k.clone(), a.clone(), review_type(inner, edit)?))),
        TypX::Datatype(n, args, paths) if paths.len() > 0 => {
            let mut ps = paths.as_ref().clone();
            if edit == 0 { ps.truncate(0); }
            else if edit == 1 {
                ps[0] = match &ps[0] {
                    ImplPath::TraitImplPath(p) => ImplPath::TraitImplPath(p.push_segment(Arc::new("independent_review_witness".into()))),
                    ImplPath::FnDefImplPath(f) => ImplPath::FnDefImplPath(Arc::new(FunX { path: f.path.push_segment(Arc::new("independent_review_witness".into())) })),
                };
            } else { let first = ps[0].clone(); ps.insert(0, first); }
            Some(Arc::new(TypX::Datatype(n.clone(), args.clone(), Arc::new(ps))))
        }
        _ => None,
    }
}
fn review_witness_clones<'tcx>(
    ctxt: &Context<'tcx>, caller: DefId, origin: rustc_span::Span,
    ids: &ConcreteOrderingInstances<'tcx>, carrier: &mir::Body<'tcx>,
    get: &mir::Body<'tcx>, primitive: &mir::Body<'tcx>,
    models: &OrderingModelBindings, f: &Function, boundary: &str,
) -> Result<(), VirErr> {
    let dest = std::env::var("REVIEW_WITNESS_RECORDS").expect("private review destination");
    let dir = std::path::Path::new(&dest).join(boundary);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("original.json"), serde_json::to_vec(f).unwrap()).unwrap();
    for edit in 0..3 {
        for site in 0..4 {
            let mut x = f.x.clone();
            if site < 2 {
                let old = x.params[site].clone();
                let mut px = old.x.clone();
                px.typ = review_type(&px.typ, edit).expect("actual nonempty embedded witness");
                Arc::make_mut(&mut x.params)[site] = old.new_x(px);
            } else {
                let edits = std::cell::Cell::new(0);
                let visitor = |e: &Expr| -> Result<Expr, VirErr> {
                    if let Some(t) = review_type(&e.typ, edit) {
                        edits.set(edits.get()+1);
                        Ok(SpannedTyped::new(&e.span, &t, e.x.clone()))
                    } else { Ok(e.clone()) }
                };
                if site == 2 { x.body = Some(vir::ast_visitor::map_expr_visitor(x.body.as_ref().unwrap(), &visitor)?); }
                else { Arc::make_mut(&mut x.ensure.0)[0] = vir::ast_visitor::map_expr_visitor(&x.ensure.0[0], &visitor)?; }
                if edits.get() == 0 { panic!("no nested witness changed"); }
            }
            let changed = f.new_x(x);
            std::fs::write(dir.join(format!("variant-{edit}-{site}.json")), serde_json::to_vec(&changed).unwrap()).unwrap();
            if check_ordering_function(ctxt, caller, origin, ids, carrier, get, primitive, models, &changed).is_ok() {
                return crate::util::err_span(origin, "independent embedded witness clone accepted");
            }
            eprintln!("REVIEW_WITNESS_REJECT {boundary} {edit} {site}");
        }
    }
    Ok(())
}
'''
 p.write_text(s+addition)
 before=manifest(r/'src');after=manifest(v)
 need({n for n in before if before[n]!=after[n]}=={'source/rust_verify/src/checked_nonzero_ord.rs'},'single diagnostic module changed')
 write(r/'structural-inputs.json',dict(base=before,variant=after,injection_sha256=hashlib.sha256(addition.encode()).hexdigest()))
 j=run('structural-build',[TC/'bin/cargo','build','--offline','--locked','-j','2','--manifest-path',v/'source/Cargo.toml','-p','rust_verify'])
 need(j['exit']==0,'diagnostic compiler build')
 shutil.copy2(r/'target/debug/rust_verify',r/'structural-rust_verify')
def trial():
 # Reuse the original cold command and full normal matching macro/vstd inputs;
 # only the diagnostic executable differs. No certified export selected.
 f=json.loads((r/'bundle-exact-cold/freeze.json').read_text());argv=f['argv']
 argv=[str(r/'structural-rust_verify'),*argv[1:]]
 argv[argv.index('--log-dir')+1]=str(r/'structural-boundaries/vir')
 argv[argv.index('-o')+1]=str(r/'structural-boundaries/liblocal_order.rlib')
 binary_hash=sha(r/'structural-rust_verify')
 write(r/'structural-boundary-inputs.json',dict(binary_sha256=binary_hash,variant_source=manifest(r/'structural-src'),production_tools={n:sha(bundle()/n) for n in TOOLS},argv=argv))
 j=run('structural-boundaries',argv,{'REVIEW_WITNESS_RECORDS':str(r/'witness-records')})
 need(j['exit']==0 and j['verification']['verified']==2 and j['verification']['errors']==0,'diagnostic source positive')
 need(sha(r/'structural-rust_verify')==binary_hash,'diagnostic binary drift')
 write(r/'structural-result.json',audit())
def mask(v):
 if isinstance(v,list):return [mask(x) for x in v]
 if isinstance(v,dict):return {k:([mask(x) for x in a[:2]]+["WITNESS"]) if k=='Datatype' else mask(a) for k,a in v.items()}
 return v
def audit():
 inputs=json.loads((r/'structural-inputs.json').read_text())
 boundary_inputs=json.loads((r/'structural-boundary-inputs.json').read_text())
 need(boundary_inputs['binary_sha256']==sha(r/'structural-rust_verify') and boundary_inputs['variant_source']==manifest(r/'structural-src'),'diagnostic pre-proof guards')
 need(boundary_inputs['production_tools']=={n:sha(bundle()/n) for n in TOOLS},'diagnostic uses same final production tools')
 need(manifest(r/'structural-src')==inputs['variant'] and manifest(r/'src')==inputs['base'],'variant guards')
 records={}
 for boundary in ['registration','post-WF-autospec-modes']:
  d=r/'witness-records'/boundary;orig=json.loads((d/'original.json').read_text());record={}
  for edit in range(3):
   for site in range(4):
    p=d/f'variant-{edit}-{site}.json';v=json.loads(p.read_text())
    need(v!=orig and mask(v)==mask(orig),'edit outside embedded datatype paths')
    record[p.name]=sha(p)
  records[boundary]=dict(original_sha256=sha(d/'original.json'),rejected_clones=record)
 err=(r/'structural-boundaries/stderr').read_text()
 need(err.count('REVIEW_WITNESS_REJECT')==24,'both actual boundaries exercised')
 return dict(records=records,originals_identical_across_boundaries=records['registration']['original_sha256']==records['post-WF-autospec-modes']['original_sha256'],only_embedded_witnesses_changed=True,semantic_false_success_claimed=False,diagnostic_binary_sha256=sha(r/'structural-rust_verify'),inputs_sha256=sha(r/'structural-inputs.json'))
if __name__=='__main__':
 if sys.argv[1]=='prepare':prepare()
 elif sys.argv[1]=='trial':trial()
 else:need(audit()==json.loads((r/'structural-result.json').read_text()),'structural record drift')
