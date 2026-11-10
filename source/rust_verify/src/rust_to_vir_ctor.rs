use crate::unsupported_err;
use rustc_hir::def::{CtorKind, CtorOf, DefKind, Res};
use rustc_middle::ty::{Ty, TyCtxt, TyKind, VariantDef};
use rustc_span::Span;
use rustc_span::def_id::DefId;
use vir::ast::VirErr;

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum AdtKind {
    Struct,
    Enum,
    Union,
}

#[derive(Debug)]
pub(crate) struct Ctor<'tcx> {
    pub adt_def_id: DefId,
    pub variant_def: &'tcx VariantDef,
    pub kind: AdtKind,
}

impl<'tcx> Ctor<'tcx> {
    pub(crate) fn variant_name(
        &self,
        tcx: TyCtxt<'tcx>,
        fields: &'tcx [rustc_hir::ExprField<'tcx>],
    ) -> vir::ast::Ident {
        match &self.kind {
            AdtKind::Union => {
                // For a union, rustc has one "variant" with all the fields, while our
                // VIR representation has one variant per field.
                // Use the name of the VIR variant (same as the field name).
                assert!(fields.len() == 1);
                air::ast_util::str_ident(fields[0].ident.as_str())
            }
            AdtKind::Struct | AdtKind::Enum => {
                air::ast_util::str_ident(&self.variant_def.ident(tcx).as_str())
            }
        }
    }
}

// Some notes about rustc's representation:
//
//  * Every ADT has some number of "variants". Structs and unions always 1 "variant".
//
//  * Every variant optionally has a "constructor kind". The ctor_kind() is one of:
//        None, Some(CtorKind::Fn) or Some(CtorKind::Const)
//
//  * For variants that have a CtorKind, there is a separate DefId for the constructor.
//
//  * Res object can resolve to either Constructors, Variants, or Types.
//
//  * When a constructor is used as an expression (ExprKind::Path) it will resolve to the
//    the DefId of the Constructor (the Res will be of DefKind::Ctor or Res::SelfCtor, i.e.,
//    something with Ctor in its name). This can only happen for any variant which has
//    a CtorKind and a Constructor DefId.
//
//  * However, for a *braces-style* constructor expression (ExprKind::Struct),
//    the Res resolves to a Variant (for enums) or to a Type (for structs and unions).
//    This can happen for _any_ variant, not just those that have a CtorKind.

/// Const and tuple-style (i.e., when the path appears in an expression position)
///
/// Returns struct/enum only, never union. Returns None if this is not a constructor.
///
/// Caller should check the CtorKind is correct for its context.
pub(crate) fn resolve_ctor<'tcx>(tcx: TyCtxt<'tcx>, res: Res) -> Option<(Ctor<'tcx>, CtorKind)> {
    match res {
        Res::Def(DefKind::Ctor(CtorOf::Struct, ctor_kind), ctor_did) => {
            let struct_did = tcx.parent(ctor_did);
            let adt_def = tcx.adt_def(struct_did);
            assert!(adt_def.is_struct());
            let variant_def = adt_def.non_enum_variant();
            assert!(variant_def.ctor_kind() == Some(ctor_kind));
            Some((Ctor { adt_def_id: struct_did, variant_def, kind: AdtKind::Struct }, ctor_kind))
        }
        Res::Def(DefKind::Ctor(CtorOf::Variant, ctor_kind), variant_ctor_did) => {
            let variant_did = tcx.parent(variant_ctor_did);
            let enum_did = tcx.parent(variant_did);
            let adt_def = tcx.adt_def(enum_did);
            assert!(adt_def.is_enum());
            let variant_def = adt_def.variant_with_ctor_id(variant_ctor_did);
            assert!(variant_def.ctor_kind() == Some(ctor_kind));
            Some((Ctor { adt_def_id: adt_def.did(), variant_def, kind: AdtKind::Enum }, ctor_kind))
        }
        Res::SelfCtor(impl_id) => {
            let self_ty = tcx.type_of(impl_id).skip_binder();
            let struct_did = match self_ty.kind() {
                TyKind::Adt(adt_def, _args) => adt_def.did(),
                _ => {
                    panic!("got unexpected Self type trying to resolve constructor")
                }
            };

            let adt_def = tcx.adt_def(struct_did);
            assert!(adt_def.is_struct());

            let variant_def = adt_def.non_enum_variant();

            assert!(variant_def.ctor_kind().is_some());

            Some((
                Ctor { adt_def_id: struct_did, variant_def, kind: AdtKind::Struct },
                variant_def.ctor_kind().unwrap(),
            ))
        }
        _ => None,
    }
}

/// Braces-style (i.e., when the path appears as an argument to ExprKind::Struct or PatKind::Struct)
pub(crate) fn resolve_braces_ctor<'tcx>(
    tcx: TyCtxt<'tcx>,
    res: Res,
    ty: Ty<'tcx>,
    allow_union: bool,
    span: Span,
) -> Result<Ctor<'tcx>, VirErr> {
    match ty.kind() {
        TyKind::Adt(adt_def, _args) => {
            if adt_def.is_struct() || adt_def.is_union() {
                if !allow_union && adt_def.is_union() {
                    unsupported_err!(span, "using a union here")
                }
                let kind = if adt_def.is_struct() { AdtKind::Struct } else { AdtKind::Union };
                return Ok(Ctor {
                    adt_def_id: adt_def.did(),
                    variant_def: adt_def.non_enum_variant(),
                    kind,
                });
            }
        }
        _ => {
            panic!("get_adt_res expected Adt type");
        }
    }

    // Must be an enum, so the only possibility is DefKind::Variant
    // (DefKind::Ctor only shows for resolve_ctor, don't have to handle it here)

    match res {
        Res::Def(DefKind::Variant, did) => {
            let enum_did = tcx.parent(did);
            let variant_def = tcx.adt_def(enum_did).variant_with_id(did);
            return Ok(Ctor { adt_def_id: enum_did, variant_def, kind: AdtKind::Enum });
        }
        _ => {
            crate::internal_err!(span, "unexpected Res trying to resolve struct constructor", res)
        }
    }
}

/// Give eligible Rust constructor items their own checked FnDef implementation.
/// This is not a closure conversion: both the VIR singleton and its type use the
/// rustc constructor DefId. FnDef's existing checked-contract translation supplies
/// the Fn-family interface, just as for an ordinary exec function.
pub(crate) fn register_function_items<'tcx>(
    ctxt: &crate::context::Context<'tcx>,
    krate: &vir::ast::KrateX,
    state: &mut crate::rust_to_vir::State,
    infos: &mut Vec<crate::rust_to_vir_func::FunctionOrConstInfo<'tcx>>,
) -> Result<(), VirErr> {
    use crate::rust_to_vir_func::FunctionOrConstInfo;
    use std::sync::Arc;
    use vir::ast::*;
    use vir::ast_util::{bool_typ, ident_binder, str_unique_var};

    for item in crate::util::iter_crate_free_items(ctxt.tcx) {
        let id = item.owner_id.to_def_id();
        if !matches!(ctxt.tcx.def_kind(id), DefKind::Struct | DefKind::Enum) {
            continue;
        }
        let path = ctxt.def_id_to_vir_path(id);
        let Some(datatype) = krate.datatypes.iter().find(|d| d.x.name == Dt::Path(path.clone()))
        else {
            continue;
        };
        let d = &datatype.x;
        // Do not expose an opaque representation, erase a ghost field, or promise
        // an unconditional constructor when a user invariant must be established.
        if d.proxy.is_some()
            || d.mode != Mode::Exec
            || matches!(d.transparency, DatatypeTransparency::Never)
            || d.user_defined_invariant_fn.is_some()
        {
            continue;
        }
        let adt = ctxt.tcx.adt_def(id);
        let ret_typ =
            ctxt.mid_ty_to_vir(id, item.span, &ctxt.tcx.type_of(id).skip_binder(), None)?;
        for variant in adt.variants() {
            if variant.ctor_kind() != Some(CtorKind::Fn) {
                continue;
            }
            let variant_name = air::ast_util::str_ident(variant.ident(ctxt.tcx).as_str());
            let fields = &d.get_variant(&variant_name).fields;
            if fields.len() != variant.fields.len() {
                return crate::util::err_span(item.span, "constructor field signature mismatch");
            }
            if fields.iter().any(|f| f.a.1 != Mode::Exec) {
                continue;
            }
            let ctor_id = variant.ctor_def_id().unwrap();
            let name = Arc::new(FunX { path: ctxt.def_id_to_vir_path(ctor_id) });
            // These nodes have no HIR counterparts. Keep diagnostic source data
            // but use fresh IDs and an internal raw span, so erasure does not try
            // to map generated binders/reads to unrelated source nodes.
            let span = || {
                let mut s = ctxt.spans.to_air_span(item.span);
                s.raw_span = Arc::new(());
                s
            };
            let params: Vec<_> = fields
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    vir::def::Spanned::new(
                        span(),
                        ParamX {
                            name: str_unique_var(
                                &format!("constructor_arg_{i}"),
                                VarIdentDisambiguate::NoBodyParam,
                            ),
                            typ: f.a.0.clone(),
                            mode: Mode::Exec,
                            user_mut: false,
                            unwrapped_info: None,
                        },
                    )
                })
                .collect();
            let ret = vir::def::Spanned::new(
                span(),
                ParamX {
                    name: str_unique_var("constructor_result", VarIdentDisambiguate::NoBodyParam),
                    typ: ret_typ.clone(),
                    mode: Mode::Exec,
                    user_mut: false,
                    unwrapped_info: None,
                },
            );
            let ctor = |spec: bool| {
                let fields = fields
                    .iter()
                    .zip(params.iter())
                    .zip(variant.fields.iter())
                    .map(|((f, p), rust_field)| {
                        let x = if spec {
                            ExprX::Var(p.x.name.clone())
                        } else {
                            let place = SpannedTyped::new(
                                &span(),
                                &p.x.typ,
                                PlaceX::Local(p.x.name.clone()),
                            );
                            let field_ty = ctxt.tcx.type_of(rust_field.did).skip_binder();
                            let env = rustc_middle::ty::TypingEnv::new(
                                ctxt.tcx.param_env(id),
                                rustc_middle::ty::TypingMode::non_body_analysis(),
                            );
                            let kind = if ctxt.tcx.type_is_copy_modulo_regions(env, field_ty) {
                                ReadKind::Copy
                            } else {
                                ReadKind::Move
                            };
                            ExprX::ReadPlace(
                                place,
                                UnfinalizedReadKind {
                                    preliminary_kind: kind,
                                    id: ctxt.unique_read_kind_id(),
                                },
                            )
                        };
                        ident_binder(&f.name, &SpannedTyped::new(&span(), &p.x.typ, x))
                    })
                    .collect();
                SpannedTyped::new(
                    &span(),
                    &ret_typ,
                    ExprX::Ctor(
                        Dt::Path(path.clone()),
                        variant_name.clone(),
                        Arc::new(fields),
                        None,
                    ),
                )
            };
            let result = SpannedTyped::new(&span(), &ret_typ, ExprX::Var(ret.x.name.clone()));
            let ensure = SpannedTyped::new(
                &span(),
                &bool_typ(),
                ExprX::Binary(BinaryOp::Eq(Mode::Spec), result, ctor(true)),
            );
            let visibility = crate::rust_to_vir_base::mk_visibility(ctxt, ctor_id);
            let body = ctor(false);
            let full = vir::def::Spanned::new(
                span(),
                FunctionX {
                    name,
                    proxy: None,
                    kind: FunctionKind::Static,
                    visibility: visibility.clone(),
                    body_visibility: BodyVisibility::Visibility(visibility.clone()),
                    opaqueness: Opaqueness::Opaque,
                    owning_module: d.owning_module.clone(),
                    mode: Mode::Exec,
                    typ_params: Arc::new(d.typ_params.iter().map(|(n, _)| n.clone()).collect()),
                    typ_bounds: d.typ_bounds.clone(),
                    params: Arc::new(params),
                    outer_ret: ret.clone(),
                    inner_ret: ret,
                    require: Arc::new(vec![]),
                    ensure: (Arc::new(vec![ensure]), Arc::new(vec![])),
                    returns: None,
                    decrease: Arc::new(vec![]),
                    decrease_when: None,
                    decrease_by: None,
                    fndef_axioms: None,
                    mask_spec: None,
                    atomic_update: None,
                    unwind_spec: None,
                    item_kind: ItemKind::Function,
                    attrs: Arc::new(FunctionAttrsX {
                        uses_ghost_blocks: true,
                        ..FunctionAttrsX::default()
                    }),
                    body: Some(body),
                    return_opaque_type_eqs: Arc::new(vec![]),
                    extra_dependencies: vec![],
                    hidden: Arc::new(vec![]),
                },
            );
            state.insert_fun_warn_config(ctxt, &full.x.name, ctor_id);
            infos.push(FunctionOrConstInfo::Constructor {
                stub: vir::ast_util::function_to_stub(&full),
                full,
            });
            ctxt.constructor_functions.borrow_mut().insert(ctor_id);
        }
    }
    Ok(())
}
