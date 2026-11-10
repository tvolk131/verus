//! Local borrow-checker evidence, never a physical ownership certificate.
#![feature(rustc_private)]
extern crate rustc_borrowck;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
use rustc_borrowck::consumers::{get_bodies_with_borrowck_facts, ConsumerOptions};
use rustc_middle::{mir, ty::{self, TyCtxt}};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
#[path = "../../native_iterator_certificate/src/ir.rs"]
mod ir;
fn hash(b: &[u8]) -> String { format!("{:x}", Sha256::digest(b)) }
fn location(l: mir::Location) -> Value { json!({"bb":l.block.as_usize(),"statement":l.statement_index}) }
fn entry_statement<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>, s: &mir::StatementKind<'tcx>) -> Value {
    // The inherited enum encoder deliberately rejects unsizing. Record that
    // particular operation explicitly here; it remains an unproved heap step.
    if let mir::StatementKind::Assign(a) = s {
        if let mir::Rvalue::Cast(mir::CastKind::PointerCoercion(ty::adjustment::PointerCoercion::Unsize, _), operand, to) = &a.1 {
            return json!({"op":"assign","destination":ir::place(tcx,body,a.0),
                "value":{"op":"unsize","operand":ir::operand(tcx,body,operand),"to":to.to_string()}});
        }
    }
    ir::statement(tcx, body, s)
}
fn pointee<'tcx>(tcx: TyCtxt<'tcx>, t: ty::Ty<'tcx>) -> Value {
    match t.kind() {
        ty::Ref(_,p,m) => match p.kind() {
            ty::Array(element,n) => {
                let layout=tcx.layout_of(ty::TypingEnv::fully_monomorphized().as_query_input(*p)).expect("array layout");
                json!({"kind":"array_borrow","exclusive":m.is_mut(),"element":element.to_string(),"length":n.try_to_target_usize(tcx),"size":layout.size.bytes(),"align":layout.align.abi.bytes()})
            }
            ty::Slice(element) => json!({"kind":"slice_borrow","exclusive":m.is_mut(),"element":element.to_string()}),
            _ => json!({"kind":"unsupported_input"}),
        },
        _ => json!({"kind":"unsupported_input"}),
    }
}
thread_local! { static EVIDENCE: std::cell::RefCell<Option<Value>> = const { std::cell::RefCell::new(None) }; }
fn capture<'tcx>(tcx: TyCtxt<'tcx>, root: rustc_hir::def_id::LocalDefId) -> Value {
        let mut bodies = get_bodies_with_borrowck_facts(tcx, root, ConsumerOptions::PoloniusInputFacts);
        let facts=bodies.remove(&root).expect("root borrowck facts");
        let body=&facts.body;
        if body.basic_blocks.len()>128 || body.local_decls.len()>256 { panic!("entry evidence resource budget"); }
        let loans: Vec<_>=facts.borrow_set.iter_enumerated().map(|(i,b)|json!({
            "loan":i.as_usize(),"reserve":location(b.reserve_location()),
            "activation":format!("{:?}",b.activation_location()),"kind":match b.kind(){mir::BorrowKind::Shared=>"shared",mir::BorrowKind::Mut{..}=>"exclusive",_=>"fake"},
            "region":b.region().as_usize(),"borrowed":ir::place(tcx,body,b.borrowed_place()),"assigned":ir::place(tcx,body,b.assigned_place())})).collect();
        let locals:Vec<_>=body.local_decls.iter_enumerated().map(|(l,d)|json!({"local":l.as_usize(),"ty":d.ty.to_string(),"shape":ir::type_shape(tcx,d.ty)})).collect();
        let blocks:Vec<_>=body.basic_blocks.iter_enumerated().map(|(bb,b)|{
            let term=b.terminator();
            let call=if let mir::TerminatorKind::Call{func,..}=&term.kind {
                if let ty::FnDef(id,args)=*func.ty(&body.local_decls,tcx).kind() {json!({"definition":tcx.def_path_str(id),"identity":format!("{:?}",tcx.def_path_hash(id)),"arguments":format!("{args:?}")})}else{json!({"unsupported":"indirect_call"})}
            }else{Value::Null};
            json!({"bb":bb.as_usize(),"cleanup":b.is_cleanup,"statements":b.statements.iter().enumerate().map(|(n,s)|json!({"location":{"bb":bb.as_usize(),"statement":n},"ir":entry_statement(tcx,body,&s.kind),"source":tcx.sess.source_map().span_to_diagnostic_string(s.source_info.span)})).collect::<Vec<_>>(),"terminator":ir::terminator(tcx,body,&term.kind),"call":call})
        }).collect();
        let input=facts.input_facts.as_ref().expect("polonius input");
        let mut issued:Vec<_>=input.loan_issued_at.iter().map(|(r,l,p)|json!([r.as_usize(),l.as_usize(),p.as_usize()])).collect();
        issued.sort_by_key(|v|v.to_string());
        json!({"schema":1,"root_identity":format!("{:?}",tcx.def_path_hash(root.to_def_id())),
            "arg_count":body.arg_count,"input_shapes":body.args_iter().map(|l|pointee(tcx,body.local_decls[l].ty)).collect::<Vec<_>>(),"locals":locals,"blocks":blocks,"loans":loans,
            "borrowck_facts":{"loan_issued_at":issued,"cfg_edge_count":input.cfg_edge.len(),"subset_base_count":input.subset_base.len(),"path_assigned_at_base_count":input.path_assigned_at_base.len(),"loan_invalidated_at_count":input.loan_invalidated_at.len()},
            "body_sha256":hash(format!("{body:#?}").as_bytes()),"native_bridge_proved":false,"publication":false})
}
struct Extract { input: PathBuf, output: PathBuf }
impl rustc_driver::Callbacks for Extract {
    fn config(&mut self, config: &mut rustc_interface::interface::Config) {
        config.override_queries = Some(|_, providers| {
            providers.queries.mir_borrowck = |tcx, def| {
                if tcx.item_name(def.to_def_id()).as_str() == "xor_key" {
                    let evidence = capture(tcx,def);
                    EVIDENCE.with(|slot| { if slot.borrow_mut().replace(evidence).is_some() { panic!("duplicate root"); } });
                }
                // Run the ORIGINAL query provider; no lifetime/trait/body check
                // is skipped or replaced by the consumer's serialized result.
                let mut original = rustc_middle::query::Providers::default();
                rustc_borrowck::provide(&mut original);
                (original.mir_borrowck)(tcx,def)
            };
        });
    }
    fn after_analysis<'tcx>(&mut self, _: &rustc_interface::interface::Compiler, _: TyCtxt<'tcx>) -> rustc_driver::Compilation {
        let mut output=EVIDENCE.with(|slot|slot.borrow_mut().take()).expect("local root facts");
        output["source_sha256"]=json!(hash(&std::fs::read(&self.input).unwrap()));
        output["normal_analysis_completed"]=json!(true);
        std::fs::write(&self.output,serde_json::to_vec_pretty(&output).unwrap()).unwrap();
        rustc_driver::Compilation::Stop
    }
}
fn main(){
    let mut args=std::env::args().skip(1);
    let output=PathBuf::from(args.next().expect("output"));
    let input=PathBuf::from(args.next().expect("input"));
    let mut rustc_args=vec!["entry-extract".to_owned(),input.to_str().unwrap().to_owned()];rustc_args.extend(args);
    rustc_driver::run_compiler(&rustc_args,&mut Extract{input,output});
}
