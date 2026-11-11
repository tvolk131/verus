//! Diagnostic loaded-core test of the Stage A validator; never installs a type.
#![feature(rustc_private)]
extern crate rust_verify;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
extern crate vir;
use rustc_middle::{
    mir,
    ty::{self, TyCtxt},
};
use std::sync::Arc;
use vir::ast::*;
struct Inspect;
impl rustc_driver::Callbacks for Inspect {
    fn after_analysis<'tcx>(
        &mut self,
        _: &rustc_interface::interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> rustc_driver::Compilation {
        let root = tcx
            .hir_body_owners()
            .find(|r| {
                tcx.def_kind(r.to_def_id()) == rustc_hir::def::DefKind::Fn
                    && tcx.item_name(r.to_def_id()).as_str() == "actual_rev"
            })
            .expect("own Item root");
        let body = tcx.optimized_mir(root.to_def_id());
        let env = ty::TypingEnv::fully_monomorphized();
        let mut found = 0;
        for block in body.basic_blocks.iter() {
            let mir::TerminatorKind::Call { func, .. } = &block.terminator().kind else { continue };
            let ty::FnDef(def, args) = *func.ty(&body.local_decls, tcx).kind() else {
                panic!("indirect root")
            };
            let iterator = tcx
                .get_diagnostic_item(rustc_span::Symbol::intern("Iterator"))
                .expect("core trait");
            assert_eq!(tcx.trait_of_assoc(def), Some(iterator));
            let args = args.no_bound_vars().expect("args");
            let instance = ty::Instance::try_resolve(tcx, env, def, args).unwrap().unwrap();
            let b = instance.instantiate_mir_and_normalize_erasing_regions(
                tcx,
                env,
                ty::EarlyBinder::bind(tcx, tcx.instance_mir(instance.def).clone()),
            );
            let ty::Adt(adt, actual_args) = *b.local_decls[mir::RETURN_PLACE].ty.kind() else {
                panic!("actual Rev ADT")
            };
            let id = rust_verify::internal_repr::validate_core_rev_metadata(tcx, adt)
                .expect("actual loaded core Rev");
            let ty::Adt(wrong, _) = *actual_args.type_at(0).kind() else {
                panic!("actual Enumerate")
            };
            assert!(rust_verify::internal_repr::validate_core_rev_metadata(tcx, wrong).is_err());
            let path = |segments: &[&str]| {
                Arc::new(PathX {
                    krate: CrateId::Core,
                    segments: Arc::new(segments.iter().map(|s| Arc::new(s.to_string())).collect()),
                })
            };
            let owner = path(&["iter", "adapters", "rev"]);
            let param = Arc::new(tcx.generics_of(adt.did()).own_params[0].name.to_string());
            let mut d = DatatypeX {
                name: Dt::Path(path(&["iter", "adapters", "rev", "Rev"])),
                proxy: None,
                owning_module: Some(owner.clone()),
                visibility: Visibility { restricted_to: None },
                transparency: DatatypeTransparency::Never,
                typ_params: Arc::new(vec![(param.clone(), AcceptRecursiveType::Reject)]),
                typ_bounds: Arc::new(vec![]),
                variants: Arc::new(vec![Variant {
                    name: Arc::new("Rev".into()),
                    ctor_style: CtorPrintStyle::Braces,
                    fields: Arc::new(vec![vir::ast_util::ident_binder(
                        &vir::def::field_ident_from_rust("iter"),
                        &(
                            Arc::new(TypX::TypParam(param)),
                            Mode::Exec,
                            Visibility { restricted_to: Some(owner) },
                        ),
                    )]),
                }]),
                mode: Mode::Exec,
                ext_equal: false,
                user_defined_invariant_fn: None,
                sized_constraint: None,
                destructor: false,
            };
            let descriptor = rust_verify::internal_repr::core_rev_descriptor(tcx, adt, &d)
                .expect("descriptor from loaded actual metadata");
            d.transparency = DatatypeTransparency::Internal(descriptor.clone());
            descriptor.validate(&d).unwrap();
            d.destructor = true;
            assert!(descriptor.validate(&d).is_err());
            println!(
                "ACTUAL {:?} {:?} FIELD {:?} {:?} ARGS {:?} ITEM_SIGNATURE {:?}",
                adt.did(),
                tcx.def_path_hash(adt.did()),
                adt.non_enum_variant().fields.iter().next().unwrap().did,
                tcx.def_path_hash(adt.non_enum_variant().fields.iter().next().unwrap().did),
                actual_args,
                tcx.fn_sig(root.to_def_id())
            );
            println!(
                "TARGET {} IDENTITY {:?} METADATA_PASS true WRONG_ADT_REFUSED true MUTATED_DESCRIPTOR_REFUSED true INSTALLED false",
                tcx.sess.target.llvm_target, id
            );
            found += 1;
        }
        assert_eq!(found, 1);
        rustc_driver::Compilation::Stop
    }
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    rustc_driver::run_compiler(&args, &mut Inspect);
}
