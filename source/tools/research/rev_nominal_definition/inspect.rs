//! Diagnostic only. Loaded rustc objects are printed; no VIR/model rule installed.
#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_hir;
use rustc_middle::{mir, ty::{self, TyCtxt}};
struct Inspect;
fn dump<'tcx>(tcx: TyCtxt<'tcx>, i: ty::Instance<'tcx>) {
    let env = ty::TypingEnv::fully_monomorphized();
    let def = i.def_id();
    println!("INSTANCE {i:?}\nPATH {}\nPATH_HASH {:?}\nCRATE {} STABLE_CRATE_ID {:?}\nSIGNATURE {:?}\nPREDICATES {:?}\nPARENT {}\nIMPL_TRAIT {:?}\nSOURCE {}", tcx.def_path_str(def), tcx.def_path_hash(def), tcx.crate_name(def.krate), tcx.stable_crate_id(def.krate), tcx.fn_sig(def).instantiate(tcx,i.args), tcx.param_env(def), tcx.def_path_str(tcx.parent(def)), tcx.trait_of_assoc(def), tcx.sess.source_map().span_to_diagnostic_string(tcx.def_span(def)));
    let generic = tcx.instance_mir(i.def);
    let body = i.instantiate_mir_and_normalize_erasing_regions(tcx, env, ty::EarlyBinder::bind(tcx, generic.clone()));
    println!("GENERIC_BODY {generic:#?}\nBODY {body:#?}");
    for (l,d) in body.local_decls.iter_enumerated() {
        println!("LOCAL {l:?} TY {} NEEDS_DROP {} LAYOUT {:?}",d.ty,d.ty.needs_drop(tcx,env),tcx.layout_of(env.as_query_input(d.ty)));
        let t = match d.ty.kind() {ty::Ref(_, t, _) => *t, _ => d.ty};
        if let ty::Adt(adt,args) = t.kind() {
            println!("ADT {:?} PATH_HASH {:?} ARGS {:?} DESTRUCTOR {:?}",adt.did(),tcx.def_path_hash(adt.did()),args,adt.destructor(tcx));
            for (v,variant) in adt.variants().iter_enumerated() { for (n,field) in variant.fields.iter_enumerated() {
                println!("FIELD {v:?}/{n:?} {} ID {:?} HASH {:?} VIS {:?} TY {:?}",field.name,field.did,tcx.def_path_hash(field.did),field.vis,field.ty(tcx,args));
            }}
        }
    }
    for (n,bb) in body.basic_blocks.iter_enumerated() {
        println!("BLOCK {n:?} CLEANUP {} STATEMENTS {:?} TERMINATOR {:?}",bb.is_cleanup,bb.statements,bb.terminator().kind);
    }
}
impl rustc_driver::Callbacks for Inspect {
    fn after_analysis<'tcx>(&mut self,_:&rustc_interface::interface::Compiler,tcx:TyCtxt<'tcx>)->rustc_driver::Compilation {
        println!("TARGET {:?}\nDATA_LAYOUT {:?}",tcx.sess.target,tcx.data_layout);
        for item in tcx.hir_crate_items(()).free_items() {
            let id=item.owner_id.to_def_id();
            if tcx.def_kind(id)==rustc_hir::def::DefKind::Struct {
                let adt=tcx.adt_def(id);
                println!("LOCAL_ADT {:?} HASH {:?} TYPE {:?} SOURCE {}", id, tcx.def_path_hash(id), tcx.type_of(id),tcx.sess.source_map().span_to_diagnostic_string(tcx.def_span(id)));
                for field in &adt.non_enum_variant().fields {
                    println!("LOCAL_FIELD {:?} HASH {:?} VIS {:?} TYPE {:?}",field.did,tcx.def_path_hash(field.did),field.vis,tcx.type_of(field.did));
                }
            }
        }
        let env=ty::TypingEnv::fully_monomorphized();
        let roots:Vec<_>=tcx.hir_body_owners().filter(|id| tcx.def_kind(id.to_def_id())==rustc_hir::def::DefKind::Fn && ["actual_rev","actual_next","actual_back"].contains(&tcx.item_name(id.to_def_id()).as_str())).collect();
        if roots.len()!=3 { panic!("expected three diagnostic roots"); }
        // Root names choose diagnostics only. They confer no semantic authority.
        let mut todo=vec![];
        for root in roots {
            let def=root.to_def_id();
            println!("ROOT {} HASH {:?} SIGNATURE {:?}",tcx.def_path_str(def),tcx.def_path_hash(def),tcx.fn_sig(def));
            let body=tcx.optimized_mir(def);
            println!("ROOT_BODY {body:#?}");
            for bb in body.basic_blocks.iter() {
                if let mir::TerminatorKind::Call {func,..}=&bb.terminator().kind {
                    if let ty::FnDef(f,a)=*func.ty(&body.local_decls,tcx).kind() {
                        todo.push(ty::Instance::try_resolve(tcx,env,f,a.no_bound_vars().expect("bound args")).expect("resolve").expect("instance"));
                    }
                }
            }
        }
        // Inspect new independently from the actual return ADT of resolved rev.
        for i in todo.clone() {
            let sig=tcx.fn_sig(i.def_id()).instantiate(tcx,i.args).skip_binder();
            if let ty::Adt(adt,args)=sig.output().kind() {
                for &imp in tcx.inherent_impls(adt.did()) {
                    for a in tcx.associated_items(imp).in_definition_order() {
                        if a.ident(tcx).as_str()=="new" && tcx.def_kind(a.def_id)==rustc_hir::def::DefKind::AssocFn {
                            todo.push(ty::Instance::try_resolve(tcx,env,a.def_id,args).expect("new resolve").expect("new instance"));
                        }
                    }
                }
            }
        }
        let mut seen=vec![];
        for i in todo {
            if seen.contains(&i) {continue;} seen.push(i);
            dump(tcx,i);
        }
        println!("INSTANCE_COUNT {}\nDIAGNOSTIC_ONLY true\nNATIVE_BINDING false",seen.len());
        rustc_driver::Compilation::Stop
    }
}
fn main() {let a:Vec<String>=std::env::args().collect();rustc_driver::run_compiler(&a,&mut Inspect);}
