#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;

use rustc_middle::{mir, ty::{self, TyCtxt}};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::{BTreeSet, VecDeque}, path::PathBuf};

fn hash(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }
fn identity(tcx: TyCtxt<'_>, instance: ty::Instance<'_>) -> String {
    format!("{}::{:?}::{:?}", tcx.crate_name(instance.def_id().krate), tcx.def_path_hash(instance.def_id()), instance)
}

struct Extract { output: PathBuf, budget: usize }
impl rustc_driver::Callbacks for Extract {
    fn after_analysis<'tcx>(&mut self, _: &rustc_interface::interface::Compiler, tcx: TyCtxt<'tcx>) -> rustc_driver::Compilation {
        let env = ty::TypingEnv::fully_monomorphized();
        let roots: Vec<_> = tcx.hir_body_owners().filter(|id| tcx.def_kind(id.to_def_id()) == rustc_hir::def::DefKind::Fn
            && tcx.item_name(id.to_def_id()).as_str() == "xor_key").collect();
        if roots.len() != 1 { panic!("expected exactly one xor_key body"); }
        let root = ty::Instance::mono(tcx, roots[0].to_def_id());
        let mut queue = VecDeque::from([root]);
        let mut seen = BTreeSet::new();
        let mut instances = Vec::new();
        let mut blockers = Vec::new();
        let mut operations = 0usize;
        while let Some(instance) = queue.pop_front() {
            let id = identity(tcx, instance);
            if !seen.insert(id.clone()) { continue; }
            if seen.len() > self.budget { blockers.push(json!({"kind":"instance_budget", "instance":id})); break; }
            if tcx.intrinsic(instance.def_id()).is_some() {
                blockers.push(json!({"kind":"intrinsic_semantics", "instance":id}));
                instances.push(json!({"id":id, "display":format!("{instance:?}"), "kind":"intrinsic"}));
                continue;
            }
            if matches!(instance.def, ty::InstanceKind::Item(_)) && !tcx.is_mir_available(instance.def_id()) {
                blockers.push(json!({"kind":"unavailable_mir", "instance":id}));
                instances.push(json!({"id":id, "display":format!("{instance:?}"), "kind":"unavailable"}));
                continue;
            }
            let body = instance.instantiate_mir_and_normalize_erasing_regions(tcx, env,
                ty::EarlyBinder::bind(tcx, tcx.instance_mir(instance.def).clone()));
            let mut locals = Vec::new();
            for (local, decl) in body.local_decls.iter_enumerated() {
                let layout = tcx.layout_of(env.as_query_input(decl.ty));
                let layout = match layout {
                    Ok(l) => json!({"size":l.size.bytes(),"align":l.align.abi.bytes(),"fields":format!("{:?}",l.fields),"variants":format!("{:?}",l.variants)}),
                    Err(e) => { blockers.push(json!({"kind":"layout", "instance":id,"local":local.as_usize()})); json!({"error":format!("{e:?}")}) }
                };
                locals.push(json!({"local":local.as_usize(), "ty":format!("{}",decl.ty), "layout":layout,
                    "needs_drop":decl.ty.needs_drop(tcx, env)}));
            }
            let mut blocks = Vec::new();
            for (bb, block) in body.basic_blocks.iter_enumerated() {
                let mut statements = Vec::new();
                for statement in &block.statements {
                    operations += 1;
                    statements.push(json!({"debug":format!("{:?}",statement.kind), "source":tcx.sess.source_map().span_to_diagnostic_string(statement.source_info.span)}));
                }
                operations += 1;
                let term = block.terminator();
                let mut call = Value::Null;
                if let mir::TerminatorKind::Call { func, .. } = &term.kind {
                    let ty = func.ty(&body.local_decls, tcx);
                    if let ty::FnDef(def_id,args) = *ty.kind() {
                        match ty::Instance::try_resolve(tcx, env, def_id,args.no_bound_vars().expect("monomorphic function arguments")) {
                            Ok(Some(callee)) => { call=json!({"resolved":identity(tcx,callee),"display":format!("{callee:?}")}); queue.push_back(callee); }
                            _ => blockers.push(json!({"kind":"unresolved_call","instance":id,"bb":bb.as_usize()})),
                        }
                    } else { blockers.push(json!({"kind":"indirect_call","instance":id,"bb":bb.as_usize()})); }
                }
                if let mir::TerminatorKind::Drop { place, .. } = &term.kind {
                    let drop_ty = place.ty(&body.local_decls,tcx).ty;
                    let callee = ty::Instance::resolve_drop_glue(tcx,drop_ty);
                    call=json!({"resolved":identity(tcx,callee),"display":format!("{callee:?}"),"drop":true});
                    queue.push_back(callee);
                }
                blocks.push(json!({"bb":bb.as_usize(),"cleanup":block.is_cleanup,"statements":statements,
                    "terminator":format!("{:?}",term.kind),"call":call,
                    "successors":term.successors().map(|b|b.as_usize()).collect::<Vec<_>>() }));
            }
            instances.push(json!({"id":id,"display":format!("{instance:?}"),"kind":"mir","args":format!("{:?}",instance.args),
                "definition_source":tcx.sess.source_map().span_to_diagnostic_string(tcx.def_span(instance.def_id())),
                "locals":locals,"blocks":blocks,"mir_sha256":hash(format!("{body:#?}").as_bytes())}));
        }
        let output = json!({"schema":1,"native_bridge_proved":false,"root":identity(tcx,root),
            "target":tcx.sess.opts.target_triple.to_string(),"pointer_bits":tcx.data_layout.pointer_size().bits(),
            "instances":instances,"operations":operations,"extraction_blockers":blockers,
            "pending_instances":queue.into_iter().map(|i|identity(tcx,i)).collect::<Vec<_>>()});
        std::fs::write(&self.output, serde_json::to_vec_pretty(&output).unwrap()).unwrap();
        rustc_driver::Compilation::Stop
    }
}
fn main() {
    let mut args = std::env::args().skip(1);
    let output = PathBuf::from(args.next().expect("output.json"));
    let budget = args.next().expect("instance budget").parse().expect("positive integer budget");
    let mut rustc_args = vec!["native-iterator-extract".to_owned()];
    rustc_args.extend(args);
    rustc_driver::run_compiler(&rustc_args, &mut Extract {output,budget});
}
