//! Read-only pinned rustc query. No foreign-body contract is installed.
#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_hir;
use rustc_middle::{mir,ty::{self,TyCtxt}};
struct Query;
impl rustc_driver::Callbacks for Query {
    fn after_analysis<'tcx>(&mut self,_:&rustc_interface::interface::Compiler,tcx:TyCtxt<'tcx>)->rustc_driver::Compilation {
        let env=ty::TypingEnv::fully_monomorphized();
        let roots:Vec<_>=tcx.hir_body_owners().filter(|id|tcx.def_kind(id.to_def_id())==rustc_hir::def::DefKind::Fn && tcx.item_name(id.to_def_id()).as_str()=="actual_from").collect();
        if roots.len()!=1 { panic!("one actual_from root required"); }
        let caller=tcx.optimized_mir(roots[0].to_def_id());
        println!("CALLER {caller:#?}");
        let mut found=Vec::new();
        for bb in caller.basic_blocks.iter() {
            if let mir::TerminatorKind::Call{func,..}=&bb.terminator().kind {
                if let ty::FnDef(def,args)=*func.ty(&caller.local_decls,tcx).kind() {
                    let inst=ty::Instance::try_resolve(tcx,env,def,args.no_bound_vars().expect("monomorphic args")).expect("resolution").expect("instance");
                    found.push(inst);
                }
            }
        }
        if found.len()!=1 { panic!("one direct call required, got {}",found.len()); }
        let inst=found[0];let def=inst.def_id();
        println!("RESOLVED_DEF_ID {def:?}\nRESOLVED_INSTANCE {inst:?}\nDEF_PATH {}\nDEF_PATH_HASH {:?}\nCRATE {}\nARGS {:?}\nSIGNATURE {:?}\nSOURCE {}",tcx.def_path_str(def),tcx.def_path_hash(def),tcx.crate_name(def.krate),inst.args,tcx.fn_sig(def).instantiate(tcx,inst.args),tcx.sess.source_map().span_to_diagnostic_string(tcx.def_span(def)));
        if !matches!(inst.def,ty::InstanceKind::Item(_)) || !tcx.is_mir_available(def) {panic!("unavailable ordinary MIR");}
        let body=inst.instantiate_mir_and_normalize_erasing_regions(tcx,env,ty::EarlyBinder::bind(tcx,tcx.instance_mir(inst.def).clone()));
        println!("FULL_INSTANTIATED_BODY {body:#?}");
        let mut pure=body.arg_count==1 && body.local_decls.len()==2 && body.basic_blocks.len()==1;
        for (local,decl) in body.local_decls.iter_enumerated() {
            println!("LOCAL {local:?} TYPE {} NEEDS_DROP {}",decl.ty,decl.ty.needs_drop(tcx,env));
            pure &= !decl.ty.needs_drop(tcx,env);
        }
        pure &= matches!(body.local_decls[mir::RETURN_PLACE].ty.kind(),ty::Uint(ty::UintTy::U64));
        pure &= matches!(body.local_decls[mir::Local::from_usize(1)].ty.kind(),ty::Bool);
        for (idx,bb) in body.basic_blocks.iter_enumerated() {
            println!("BLOCK {idx:?} CLEANUP {} STATEMENT_COUNT {}",bb.is_cleanup,bb.statements.len());
            pure &= !bb.is_cleanup && bb.statements.len()==1;
            for s in &bb.statements {
                println!("STATEMENT {:?} SOURCE {}",s.kind,tcx.sess.source_map().span_to_diagnostic_string(s.source_info.span));
                pure &= match &s.kind {
                    mir::StatementKind::Assign(a) => {
                        let (p,r)=&**a;
                        p.local==mir::RETURN_PLACE && p.projection.is_empty() && match r {
                            mir::Rvalue::Cast(mir::CastKind::IntToInt,mir::Operand::Copy(input),out) => input.local==mir::Local::from_usize(1) && input.projection.is_empty() && matches!(out.kind(),ty::Uint(ty::UintTy::U64)),
                            _=>false,
                        }
                    },_=>false,
                };
            }
            println!("TERMINATOR {:?} SOURCE {}",bb.terminator().kind,tcx.sess.source_map().span_to_diagnostic_string(bb.terminator().source_info.span));
            pure &= matches!(bb.terminator().kind,mir::TerminatorKind::Return);
        }
        println!("EXACT_PRIMITIVE_SHAPE {pure}\nFOREIGN_BRIDGE_PROVED false");
        if !pure {panic!("outside primitive whitelist; no summary accepted");}
        rustc_driver::Compilation::Stop
    }
}
fn main(){let args:Vec<String>=std::env::args().collect();rustc_driver::run_compiler(&args,&mut Query);}
