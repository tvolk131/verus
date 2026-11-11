//! Diagnostic only: reads actual pinned compiler instances; installs no summary.
#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_hir;
use rustc_middle::{mir,ty::{self,TyCtxt}};
struct Query;
impl rustc_driver::Callbacks for Query {
    fn after_analysis<'tcx>(&mut self,_:&rustc_interface::interface::Compiler,tcx:TyCtxt<'tcx>)->rustc_driver::Compilation {
        println!("TARGET {:?}\nDATA_LAYOUT {:?}",tcx.sess.target,tcx.data_layout);
        let env=ty::TypingEnv::fully_monomorphized();
        let roots:Vec<_>=tcx.hir_body_owners().filter(|id|tcx.def_kind(id.to_def_id())==rustc_hir::def::DefKind::Fn && ["actual_rev","actual_inner"].contains(&tcx.item_name(id.to_def_id()).as_str())).collect();
        if roots.len()!=2 {panic!("expected two roots");}
        let mut todo=vec![];
        for root in roots {
            let def=root.to_def_id();
            println!("ROOT {} SIGNATURE {:?}",tcx.def_path_str(def),tcx.fn_sig(def));
            let body=tcx.optimized_mir(def);
            println!("ROOT_BODY {body:#?}");
            for bb in body.basic_blocks.iter() {
                if let mir::TerminatorKind::Call{func,..}=&bb.terminator().kind {
                    if let ty::FnDef(f,a)=*func.ty(&body.local_decls,tcx).kind() {
                        todo.push(ty::Instance::try_resolve(tcx,env,f,a.no_bound_vars().expect("bound args")).expect("resolve").expect("instance"));
                    }
                }
            }
        }
        let mut seen=vec![];
        while let Some(i)=todo.pop() {
            if seen.contains(&i) {continue;} seen.push(i);
            if seen.len()>4 {panic!("unexpected dependency expansion");}
            let def=i.def_id();
            if !matches!(i.def,ty::InstanceKind::Item(_)) || !tcx.is_mir_available(def) {panic!("not an ordinary available body");}
            println!("INSTANCE {i:?}\nPATH {}\nPATH_HASH {:?}\nCRATE {} STABLE_CRATE_ID {:?}\nSIGNATURE {:?}\nPREDICATES {:?}\nPARENT {}\nIMPL_TRAIT {:?}\nSOURCE {}",tcx.def_path_str(def),tcx.def_path_hash(def),tcx.crate_name(def.krate),tcx.stable_crate_id(def.krate),tcx.fn_sig(def).instantiate(tcx,i.args),tcx.param_env(def),tcx.def_path_str(tcx.parent(def)),tcx.trait_of_assoc(def),tcx.sess.source_map().span_to_diagnostic_string(tcx.def_span(def)));
            let body=i.instantiate_mir_and_normalize_erasing_regions(tcx,env,ty::EarlyBinder::bind(tcx,tcx.instance_mir(i.def).clone()));
            println!("GENERIC_BODY {:#?}\nBODY {body:#?}",tcx.instance_mir(i.def));
            // The loaded optimized Iterator::rev is already an aggregate, so
            // explicitly inspect its semantic owner's private new as well.
            if tcx.trait_of_assoc(def).is_some() {
                if let ty::Adt(adt,args)=body.local_decls[mir::RETURN_PLACE].ty.kind() {
                    for &imp in tcx.inherent_impls(adt.did()) {
                        for a in tcx.associated_items(imp).in_definition_order() {
                            if a.ident(tcx).as_str()=="new" && tcx.def_kind(a.def_id)==rustc_hir::def::DefKind::AssocFn {
                                todo.push(ty::Instance::try_resolve(tcx,env,a.def_id,args).expect("new resolve").expect("new instance"));
                            }
                        }
                    }
                }
            }
            for (l,d) in body.local_decls.iter_enumerated() {
                println!("LOCAL {l:?} TY {} NEEDS_DROP {} LAYOUT {:?}",d.ty,d.ty.needs_drop(tcx,env),tcx.layout_of(env.as_query_input(d.ty)));
                if let ty::Adt(adt,args)=d.ty.kind() {
                    println!("ADT {} PATH_HASH {:?} ARGS {:?} DESTRUCTOR {:?}",tcx.def_path_str(adt.did()),tcx.def_path_hash(adt.did()),args,adt.destructor(tcx));
                    for (v,variant) in adt.variants().iter_enumerated() { for (n,field) in variant.fields.iter_enumerated() {
                        println!("FIELD {v:?}/{n:?} {} ID {:?} HASH {:?} VIS {:?} TY {:?}",field.name,field.did,tcx.def_path_hash(field.did),field.vis,field.ty(tcx,args));
                    }}
                }
            }
            for (n,bb) in body.basic_blocks.iter_enumerated() {
                println!("BLOCK {n:?} CLEANUP {} STATEMENTS {:?} TERMINATOR {:?}",bb.is_cleanup,bb.statements,bb.terminator().kind);
                if let mir::TerminatorKind::Call{func,..}=&bb.terminator().kind {
                    if let ty::FnDef(f,a)=*func.ty(&body.local_decls,tcx).kind() {
                        todo.push(ty::Instance::try_resolve(tcx,env,f,a.no_bound_vars().expect("bound args")).expect("resolve").expect("instance"));
                    } else {panic!("indirect constructor dependency");}
                }
            }
        }
        println!("INSTANCE_COUNT {}\nDIAGNOSTIC_ONLY true\nBRIDGE_PROVED false",seen.len());
        rustc_driver::Compilation::Stop
    }
}
fn main(){let a:Vec<String>=std::env::args().collect();rustc_driver::run_compiler(&a,&mut Query);}
