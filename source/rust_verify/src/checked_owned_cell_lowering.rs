//! Inactive native body proposals. No registration, field authority or proof fact.
use super::*;
use std::sync::Arc;
use vir::{
    ast::*,
    ast_util::{bool_typ, str_unique_var, types_equal},
    def::Spanned,
};

pub(super) struct Proposal {
    pub datatype: Datatype,
    pub methods: Vec<(DefId, Function)>,
}
fn span(tcx: TyCtxt<'_>, id: DefId) -> vir::messages::Span {
    vir::messages::Span {
        raw_span: crate::spans::to_raw_span(tcx.def_span(id)),
        id: tcx.def_path_hash(id).local_hash().as_u64(),
        data: vec![],
        as_string: tcx.def_path_str(id),
    }
}
fn path(tcx: TyCtxt<'_>, id: DefId) -> Result<Path, Reject> {
    crate::rust_to_vir_base::def_id_to_vir_path_option(tcx, None, id)
        .ok_or("native path unavailable")
}
fn u64t() -> Typ {
    Arc::new(TypX::Int(IntRange::U(64)))
}
fn cellt(d: &Datatype) -> Typ {
    Arc::new(TypX::Datatype(d.x.name.clone(), Arc::new(vec![u64t()]), Arc::new(vec![])))
}
fn field(d: &Datatype) -> FieldOpr {
    FieldOpr {
        datatype: d.x.name.clone(),
        variant: Arc::new("UnsafeCell".into()),
        field: vir::def::field_ident_from_rust("value"),
        get_variant: false,
        check: VariantCheck::None,
    }
}
fn datatype(tcx: TyCtxt<'_>, cell: DefId) -> Result<Datatype, Reject> {
    if native_cell(tcx)? != cell {
        return Err("native cell DefId differs from freshly validated lang item");
    }
    let adt = tcx.adt_def(cell);
    let generic = &tcx.generics_of(cell).own_params[0];
    let t = Arc::new(TypX::TypParam(Arc::new(generic.name.to_string())));
    let size = adt
        .sizedness_constraint(tcx, ty::SizedTraitKind::Sized)
        .ok_or("native sizedness constraint absent")?
        .skip_binder();
    if !matches!(size.kind(),ty::Param(p) if p.index==0 && p.name==generic.name) {
        return Err("native sizedness constraint differs from T");
    }
    let p = path(tcx, cell)?;
    let owner = p.pop_segment();
    let mut d = DatatypeX {
        name: Dt::Path(p),
        proxy: None,
        owning_module: Some(owner.clone()),
        visibility: Visibility { restricted_to: None },
        transparency: DatatypeTransparency::Never,
        typ_params: Arc::new(vec![(
            Arc::new(generic.name.to_string()),
            AcceptRecursiveType::Reject,
        )]),
        typ_bounds: Arc::new(vec![]),
        variants: Arc::new(vec![Variant {
            name: Arc::new("UnsafeCell".into()),
            ctor_style: CtorPrintStyle::Braces,
            fields: Arc::new(vec![vir::ast_util::ident_binder(
                &vir::def::field_ident_from_rust("value"),
                &(t.clone(), Mode::Exec, Visibility { restricted_to: Some(owner) }),
            )]),
        }]),
        mode: Mode::Exec,
        ext_equal: false,
        user_defined_invariant_fn: None,
        sized_constraint: Some(t),
        destructor: false,
    };
    d.transparency = DatatypeTransparency::Internal(
        vir::internal_repr::InternalRepresentation::owned_unsafe_cell_u64(&d, (CORE, CELL, VALUE))?,
    );
    Ok(Spanned::new(span(tcx, cell), d))
}
fn build(tcx: TyCtxt<'_>, native: DefId, op: Operation, d: &Datatype) -> Function {
    let s = span(tcx, native);
    let c = cellt(d);
    let u = u64t();
    let rc = Arc::new(TypX::MutRef(c.clone()));
    let ru = Arc::new(TypX::MutRef(u.clone()));
    let (argty, retty) = match op {
        Operation::New => (u.clone(), c.clone()),
        Operation::IntoInner => (c.clone(), u.clone()),
        Operation::GetMut => (rc.clone(), ru.clone()),
    };
    let arg = str_unique_var("owned_cell_input", VarIdentDisambiguate::NoBodyParam);
    let ret = str_unique_var("owned_cell_result", VarIdentDisambiguate::NoBodyParam);
    let param = |name, typ| {
        Spanned::new(
            s.clone(),
            ParamX { name, typ, mode: Mode::Exec, user_mut: false, unwrapped_info: None },
        )
    };
    let e = |t: &Typ, x| SpannedTyped::new(&s, t, x);
    let p = |t: &Typ, x| SpannedTyped::new(&s, t, x);
    let root = p(&argty, PlaceX::Local(arg.clone()));
    let read = |p: Place, kind| {
        e(
            &p.typ.clone(),
            ExprX::ReadPlace(
                p,
                UnfinalizedReadKind {
                    preliminary_kind: kind,
                    id: match op {
                        Operation::New => 1,
                        Operation::IntoInner => 2,
                        Operation::GetMut => 3,
                    },
                },
            ),
        )
    };
    let select = |value: Expr| e(&u, ExprX::UnaryOpr(UnaryOpr::Field(field(d)), value));
    let av = e(&argty, ExprX::Var(arg.clone()));
    let rv = e(&retty, ExprX::Var(ret.clone()));
    let eq = |a, b| e(&bool_typ(), ExprX::Binary(BinaryOp::Eq(Mode::Spec), a, b));
    let current = |r: Expr, t: &Typ| e(t, ExprX::Unary(UnaryOp::MutRefCurrent, r));
    let future = |r: Expr, t: &Typ| {
        e(t, ExprX::Unary(UnaryOp::MutRefFuture(MutRefFutureSourceName::MutRefFuture), r))
    };
    let (body, ensures) = match op {
        Operation::New => (
            e(
                &c,
                ExprX::Ctor(
                    d.x.name.clone(),
                    Arc::new("UnsafeCell".into()),
                    Arc::new(vec![vir::ast_util::ident_binder(
                        &vir::def::field_ident_from_rust("value"),
                        &read(root, ReadKind::Copy),
                    )]),
                    None,
                ),
            ),
            vec![eq(select(rv), av)],
        ),
        Operation::IntoInner => {
            (read(p(&u, PlaceX::Field(field(d), root)), ReadKind::Move), vec![eq(rv, select(av))])
        }
        Operation::GetMut => {
            let inner = p(&c, PlaceX::DerefMut(root));
            let fieldp = p(&u, PlaceX::Field(field(d), inner));
            (
                e(&ru, ExprX::BorrowMut(fieldp)),
                vec![
                    eq(current(rv.clone(), &u), select(current(av.clone(), &c))),
                    eq(select(future(av, &c)), future(rv, &u)),
                ],
            )
        }
    };
    // This is the REAL local compiler helper scope, never a forged core owner.
    let root = Arc::new(PathX {
        krate: crate::rust_to_vir_base::mk_crate_id(tcx, rustc_hir::def_id::LOCAL_CRATE),
        segments: Arc::new(vec![]),
    });
    let name = root.push_segment(Arc::new(format!(
        "owned_cell_native_body_{}%",
        tcx.def_path_hash(native).local_hash().as_u64()
    )));
    let vis = Visibility { restricted_to: Some(root.clone()) };
    let ret = param(ret, retty);
    Spanned::new(
        s.clone(),
        FunctionX {
            name: Arc::new(FunX { path: name }),
            proxy: None,
            kind: FunctionKind::Static,
            visibility: vis.clone(),
            body_visibility: BodyVisibility::Visibility(vis),
            opaqueness: Opaqueness::Opaque,
            owning_module: Some(root),
            mode: Mode::Exec,
            typ_params: Arc::new(vec![]),
            typ_bounds: Arc::new(vec![]),
            params: Arc::new(vec![param(arg, argty)]),
            outer_ret: ret.clone(),
            inner_ret: ret,
            require: Arc::new(vec![]),
            ensure: (Arc::new(ensures), Arc::new(vec![])),
            returns: None,
            decrease: Arc::new(vec![]),
            decrease_when: None,
            decrease_by: None,
            fndef_axioms: None,
            mask_spec: None,
            atomic_update: None,
            unwind_spec: None,
            item_kind: ItemKind::Function,
            attrs: Arc::new(FunctionAttrsX { uses_ghost_blocks: true, ..Default::default() }),
            body: Some(body),
            return_opaque_type_eqs: Arc::new(vec![]),
            extra_dependencies: vec![],
            hidden: Arc::new(vec![]),
        },
    )
}

fn matches_type(native: Ty<'_>, emitted: &Typ, cell: DefId, d: &Datatype) -> bool {
    match (native.kind(), &**emitted) {
        (ty::Uint(ty::UintTy::U64), TypX::Int(IntRange::U(64))) => true,
        (ty::Adt(a, args), TypX::Datatype(name, types, impls)) => {
            a.did() == cell
                && args.len() == 1
                && is_u64(args.type_at(0))
                && *name == d.x.name
                && types.len() == 1
                && matches!(&*types[0], TypX::Int(IntRange::U(64)))
                && impls.is_empty()
        }
        (ty::Ref(_, t, hir::Mutability::Mut), TypX::MutRef(inner)) => {
            matches_type(*t, inner, cell, d)
        }
        _ => false,
    }
}
use rustc_hir as hir;
/// Independently read the original typed body against ACTUAL emitted places,
/// types and read modes. Never consumes the build operation descriptor or its
/// proposed postcondition as evidence of computation.
fn corresponds(
    tcx: TyCtxt<'_>,
    cell: DefId,
    d: &Datatype,
    mir: &mir::Body<'_>,
    f: &Function,
) -> bool {
    if mir.arg_count != 1
        || mir.local_decls.len() != 2
        || mir.basic_blocks.len() != 1
        || f.x.params.len() != 1
        || f.x.mode != Mode::Exec
        || f.x.attrs.is_external_body
        || !f.x.typ_params.is_empty()
        || !f.x.typ_bounds.is_empty()
        || !f.x.require.is_empty()
        || f.x.proxy.is_some()
        || f.x.fndef_axioms.is_some()
        || f.x.returns.is_some()
        || f.x.mask_spec.is_some()
        || f.x.atomic_update.is_some()
        || f.x.unwind_spec.is_some()
        || !f.x.decrease.is_empty()
        || f.x.decrease_by.is_some()
        || f.x.decrease_when.is_some()
        || !f.x.extra_dependencies.is_empty()
        || !f.x.return_opaque_type_eqs.is_empty()
        || !f.x.hidden.is_empty()
        || !matches!(f.x.kind, FunctionKind::Static)
        || f.x.params[0].x.mode != Mode::Exec
        || f.x.outer_ret.x.mode != Mode::Exec
        || f.x.inner_ret.x.mode != Mode::Exec
        || f.x.inner_ret.x.name != f.x.outer_ret.x.name
        || !types_equal(&f.x.inner_ret.x.typ, &f.x.outer_ret.x.typ)
    {
        return false;
    }
    let root = Arc::new(PathX {
        krate: crate::rust_to_vir_base::mk_crate_id(tcx, rustc_hir::def_id::LOCAL_CRATE),
        segments: Arc::new(vec![]),
    });
    let native = mir.source.def_id();
    let expected = root.push_segment(Arc::new(format!(
        "owned_cell_native_body_{}%",
        tcx.def_path_hash(native).local_hash().as_u64()
    )));
    if f.x.name.path != expected
        || f.x.owning_module != Some(root.clone())
        || f.x.visibility.restricted_to != Some(root.clone())
        || !matches!(&f.x.body_visibility,BodyVisibility::Visibility(v) if v.restricted_to==Some(root.clone()))
    {
        return false;
    }
    let a = &f.x.params[0].x;
    let bb = &mir.basic_blocks[mir::START_BLOCK];
    if a.user_mut
        || a.unwrapped_info.is_some()
        || f.x.inner_ret.x.user_mut
        || f.x.outer_ret.x.user_mut
        || f.x.inner_ret.x.unwrapped_info.is_some()
        || f.x.outer_ret.x.unwrapped_info.is_some()
        || a.name == f.x.inner_ret.x.name
        || bb.is_cleanup
        || !bb.terminator.as_ref().is_some_and(|t| matches!(t.kind, mir::TerminatorKind::Return))
        || !matches_type(mir.local_decls[mir::Local::from_usize(1)].ty, &a.typ, cell, d)
        || !matches_type(mir.local_decls[mir::RETURN_PLACE].ty, &f.x.inner_ret.x.typ, cell, d)
    {
        return false;
    }
    let Some(body) = &f.x.body else {
        return false;
    };
    if !matches_type(mir.local_decls[mir::RETURN_PLACE].ty, &body.typ, cell, d) {
        return false;
    }
    let [s] = bb.statements.as_slice() else {
        return false;
    };
    let mir::StatementKind::Assign(assign) = &s.kind else {
        return false;
    };
    if assign.0.local != mir::RETURN_PLACE || !assign.0.projection.is_empty() {
        return false;
    }
    fn is_root(p: &Place, a: &ParamX) -> bool {
        types_equal(&p.typ, &a.typ) && matches!(&p.x,PlaceX::Local(v) if *v==a.name)
    }
    fn private_field(op: &FieldOpr, d: &Datatype) -> bool {
        op.datatype == d.x.name
            && op.variant.as_str() == "UnsafeCell"
            && op.field == vir::def::field_ident_from_rust("value")
            && !op.get_variant
            && matches!(op.check, VariantCheck::None)
    }
    match (&assign.1, &body.x) {
        (mir::Rvalue::Aggregate(kind, operands), ExprX::Ctor(dt, variant, fields, None)) => {
            let mir::AggregateKind::Adt(native, idx, args, user, active) = &**kind else {
                return false;
            };
            let [b] = fields.as_slice() else {
                return false;
            };
            let ExprX::ReadPlace(p, read) = &b.a.x else {
                return false;
            };
            let Some(mir::Operand::Copy(m)) = operands.iter().next() else {
                return false;
            };
            *native == cell
                && idx.as_usize() == 0
                && args.len() == 1
                && is_u64(args.type_at(0))
                && user.is_none()
                && active.is_none()
                && operands.len() == 1
                && m.local.as_usize() == 1
                && m.projection.is_empty()
                && *dt == d.x.name
                && variant.as_str() == "UnsafeCell"
                && b.name == vir::def::field_ident_from_rust("value")
                && is_root(p, a)
                && matches!(read.preliminary_kind, ReadKind::Copy)
                && read.id != 0
                && types_equal(&b.a.typ, &a.typ)
        }
        (
            mir::Rvalue::Use(mir::Operand::Move(m), mir::WithRetag::Yes),
            ExprX::ReadPlace(p, read),
        ) => {
            let PlaceX::Field(op, inner) = &p.x else {
                return false;
            };
            m.local.as_usize() == 1
                && matches!(m.projection.as_ref(),[mir::ProjectionElem::Field(i,t)] if i.as_usize()==0 && is_u64(*t))
                && private_field(op, d)
                && is_root(inner, a)
                && matches!(&*p.typ, TypX::Int(IntRange::U(64)))
                && matches!(read.preliminary_kind, ReadKind::Move)
                && read.id != 0
        }
        (
            mir::Rvalue::Ref(region, mir::BorrowKind::Mut { kind: mir::MutBorrowKind::Default }, m),
            ExprX::BorrowMut(p),
        ) => {
            let PlaceX::Field(op, inner) = &p.x else {
                return false;
            };
            let PlaceX::DerefMut(outer) = &inner.x else {
                return false;
            };
            matches!(region.kind(), ty::ReErased)
                && m.local.as_usize() == 1
                && matches!(m.projection.as_ref(),[mir::ProjectionElem::Deref,mir::ProjectionElem::Field(i,t)] if i.as_usize()==0 && is_u64(*t))
                && private_field(op, d)
                && is_root(outer, a)
                && matches!(mir.local_decls[mir::Local::from_usize(1)].ty.kind(),ty::Ref(_,native_inner,hir::Mutability::Mut) if matches_type(*native_inner,&inner.typ,cell,d))
                && matches!(&*p.typ, TypX::Int(IntRange::U(64)))
        }
        _ => false,
    }
}

pub(super) fn propose(tcx: TyCtxt<'_>) -> Result<Proposal, Reject> {
    let cell = native_cell(tcx)?;
    let d = datatype(tcx, cell)?;
    let mut methods = vec![];
    for op in [Operation::New, Operation::IntoInner, Operation::GetMut] {
        let native = native_method(tcx, cell, op)?;
        let instance = signature(tcx, cell, native, op)?;
        let env = ty::TypingEnv::fully_monomorphized();
        let mir = instance.instantiate_mir_and_normalize_erasing_regions(
            tcx,
            env,
            ty::EarlyBinder::bind(tcx, tcx.instance_mir(instance.def).clone()),
        );
        inspect(tcx, cell, op, &mir)?;
        let f = build(tcx, native, op, &d);
        if !corresponds(tcx, cell, &d, &mir, &f) {
            return Err("native owned cell emitted-Function/place/type correspondence failed");
        }
        methods.push((native, f));
    }
    Ok(Proposal { datatype: d, methods })
}

#[cfg(test)]
pub(super) fn test_actual_proposals(tcx: TyCtxt<'_>) {
    use std::collections::HashMap;
    let proposal = propose(tcx).unwrap();
    let d = &proposal.datatype;
    let DatatypeTransparency::Internal(repr) = &d.x.transparency else {
        panic!("descriptor absent");
    };
    repr.validate(&d.x).unwrap();
    repr.validate_instantiation(&Arc::new(vec![u64t()])).unwrap();
    for bad in [
        vec![],
        vec![Arc::new(TypX::Bool)],
        vec![Arc::new(TypX::Int(IntRange::U(32)))],
        vec![Arc::new(TypX::TypParam(Arc::new("T".into())))],
        vec![cellt(d)],
        vec![u64t(), u64t()],
    ] {
        assert!(repr.validate_instantiation(&Arc::new(bad)).is_err());
    }
    assert!(serde_json::to_string(&d.x).is_err());
    assert!(serde_json::from_str::<DatatypeTransparency>(r#"{"Internal":{}}"#).is_err());
    let root = Arc::new(PathX {
        krate: crate::rust_to_vir_base::mk_crate_id(tcx, rustc_hir::def_id::LOCAL_CRATE),
        segments: Arc::new(vec![]),
    });
    assert!(!repr.source_visible_to(&root));
    for change in 0..8 {
        let mut bad = d.x.clone();
        match change {
            0 => bad.owning_module = Some(root.clone()),
            1 => bad.sized_constraint = None,
            2 => bad.ext_equal = true,
            3 => bad.destructor = true,
            4 => bad.mode = Mode::Spec,
            5 => Arc::make_mut(&mut bad.typ_params)[0].1 = AcceptRecursiveType::Accept,
            6 => {
                Arc::make_mut(&mut Arc::make_mut(&mut bad.variants)[0].fields)[0] =
                    vir::ast_util::ident_binder(
                        &vir::def::field_ident_from_rust("other"),
                        &(
                            Arc::new(TypX::TypParam(bad.typ_params[0].0.clone())),
                            Mode::Exec,
                            repr.source_visibility().clone(),
                        ),
                    )
            }
            7 => Arc::make_mut(&mut bad.variants)[0].name = Arc::new("Other".into()),
            _ => unreachable!(),
        }
        assert!(repr.validate(&bad).is_err(), "descriptor corruption {change}");
    }
    for (native, f) in &proposal.methods {
        let op = if tcx.item_name(*native).as_str() == "new" {
            Operation::New
        } else if tcx.item_name(*native).as_str() == "into_inner" {
            Operation::IntoInner
        } else {
            Operation::GetMut
        };
        let instance = signature(tcx, native_cell(tcx).unwrap(), *native, op).unwrap();
        let env = ty::TypingEnv::fully_monomorphized();
        let mir = instance.instantiate_mir_and_normalize_erasing_regions(
            tcx,
            env,
            ty::EarlyBinder::bind(tcx, tcx.instance_mir(instance.def).clone()),
        );
        assert!(corresponds(tcx, native_cell(tcx).unwrap(), d, &mir, f));
        for change in 0..12 {
            let mut x = f.x.clone();
            match change {
                0 => {
                    Arc::make_mut(&mut Arc::make_mut(&mut x.params)[0]).x.typ = Arc::new(TypX::Bool)
                }
                1 => Arc::make_mut(&mut x.inner_ret).x.typ = Arc::new(TypX::Int(IntRange::U(32))),
                2 => {
                    x.body = Some(
                        x.body.as_ref().unwrap().new_typ(&Arc::new(TypX::Int(IntRange::U(32)))),
                    )
                }
                3 => x.body = None,
                4 => x.owning_module = Some(d.x.owning_module.clone().unwrap()),
                5 => Arc::make_mut(&mut x.attrs).is_external_body = true,
                6 => {
                    x.require = Arc::new(vec![SpannedTyped::new(
                        &f.span,
                        &bool_typ(),
                        ExprX::Const(Constant::Bool(false)),
                    )])
                }
                7 => Arc::make_mut(&mut x.outer_ret).x.mode = Mode::Spec,
                8 => x.name = Arc::new(FunX { path: root.push_segment(Arc::new("forged".into())) }),
                9 => x.visibility = Visibility { restricted_to: None },
                10 => {
                    let e = x.body.as_ref().unwrap();
                    x.body = Some(e.new_x(ExprX::Block(Arc::new(vec![]), Some(e.clone()))));
                }
                11 => {
                    let e = x.body.as_ref().unwrap();
                    let mut changed = e.x.clone();
                    match &mut changed {
                        ExprX::Ctor(_, _, fields, _) => {
                            let b = &fields[0];
                            let mut b = (**b).clone();
                            b.name = vir::def::field_ident_from_rust("other");
                            Arc::make_mut(fields)[0] = Arc::new(b);
                        }
                        ExprX::ReadPlace(p, read) => {
                            read.preliminary_kind = ReadKind::Copy;
                            let PlaceX::Field(op, inner) = &p.x else { unreachable!() };
                            let mut op = op.clone();
                            op.field = vir::def::field_ident_from_rust("other");
                            *p = p.new_x(PlaceX::Field(op, inner.clone()));
                        }
                        ExprX::BorrowMut(p) => {
                            let PlaceX::Field(op, inner) = &p.x else { unreachable!() };
                            let mut op = op.clone();
                            op.field = vir::def::field_ident_from_rust("other");
                            *p = p.new_x(PlaceX::Field(op, inner.clone()));
                        }
                        _ => unreachable!(),
                    }
                    x.body = Some(e.new_x(changed));
                }
                _ => unreachable!(),
            }
            assert!(
                !corresponds(tcx, native_cell(tcx).unwrap(), d, &mir, &f.new_x(x)),
                "emitted Function corruption {op:?}/{change}"
            );
        }
        let mut wrong_source = mir.clone();
        let other = proposal.methods.iter().find(|(n, _)| n != native).unwrap().0;
        wrong_source.source = mir::MirSource::item(other);
        assert!(
            !corresponds(tcx, native_cell(tcx).unwrap(), d, &wrong_source, f),
            "wrong native origin must reject"
        );
        for corruption in 0..2 {
            let body = f.x.body.as_ref().unwrap();
            let mut x = f.x.clone();
            let mut b = body.x.clone();
            match &mut b {
                ExprX::Ctor(_, _, fields, _) => {
                    let binder = Arc::make_mut(&mut Arc::make_mut(fields)[0]);
                    let mut r = binder.a.x.clone();
                    let ExprX::ReadPlace(p, kind) = &mut r else { unreachable!() };
                    if corruption == 0 {
                        kind.preliminary_kind = ReadKind::Move;
                    } else {
                        *p = p.new_x(PlaceX::Local(f.x.inner_ret.x.name.clone()));
                    }
                    binder.a = binder.a.new_x(r);
                }
                ExprX::ReadPlace(p, kind) => {
                    if corruption == 0 {
                        kind.preliminary_kind = ReadKind::Copy;
                    } else {
                        let PlaceX::Field(op, inner) = &p.x else { unreachable!() };
                        *p = p.new_x(PlaceX::Field(
                            op.clone(),
                            inner.new_x(PlaceX::Local(f.x.inner_ret.x.name.clone())),
                        ));
                    }
                }
                ExprX::BorrowMut(p) => {
                    if corruption == 0 {
                        b = ExprX::ReadPlace(
                            p.clone(),
                            UnfinalizedReadKind { preliminary_kind: ReadKind::ImmutBor, id: 3 },
                        );
                    } else {
                        let PlaceX::Field(op, inner) = &p.x else { unreachable!() };
                        *p = p.new_x(PlaceX::Field(op.clone(), inner.new_typ(&u64t())));
                    }
                }
                _ => unreachable!(),
            }
            x.body = Some(body.new_x(b));
            assert!(
                !corresponds(tcx, native_cell(tcx).unwrap(), d, &mir, &f.new_x(x)),
                "independent read/target corruption {op:?}/{corruption}"
            );
        }
    }
    let modules = vec![
        Spanned::new(d.span.clone(), ModuleX { path: root.clone(), reveals: None }),
        Spanned::new(
            d.span.clone(),
            ModuleX { path: d.x.owning_module.clone().unwrap(), reveals: None },
        ),
    ];
    let k = Arc::new(KrateX {
        functions: proposal.methods.iter().map(|(_, f)| f.clone()).collect(),
        datatypes: vec![d.clone()],
        modules,
        reveal_groups: vec![],
        traits: vec![],
        trait_impls: vec![],
        assoc_type_impls: vec![],
        opaque_types: vec![],
        external_fns: vec![],
        external_types: vec![],
        path_as_rust_names: vec![],
        arch: Arch { word_bits: ArchWordBits::Exactly(64) },
    });
    vir::internal_repr::validate_krate_uses(&k).unwrap();
    let stubs: Arc<HashMap<_, _>> = Arc::new(
        k.functions
            .iter()
            .map(|f| (f.x.name.clone(), vir::ast_util::function_to_stub(f)))
            .collect(),
    );
    let traits = vir::traits::merge_external_traits(&k, &[]).unwrap();
    let (initial, errors, _) =
        vir::well_formed::check_crate_initial(&k, &[], stubs.clone(), traits, true);
    assert!(errors.is_empty(), "initial WF errors: {errors:?}");
    let initial = initial.unwrap_or_else(|_| panic!("native descriptor initial WF failed"));
    let demote = vir::traits::DemoteTraitLookup::new(&k, &[], &stubs);
    let warnings = vir::context::WarningCtx {
        fun_warn_configs: k.functions.iter().map(|f| (f.x.name.clone(), None)).collect(),
    };
    let first = &k.functions[0];
    let mut wrong = first.x.clone();
    let bad_cell = Arc::new(TypX::Datatype(
        d.x.name.clone(),
        Arc::new(vec![Arc::new(TypX::Int(IntRange::U(32)))]),
        Arc::new(vec![]),
    ));
    Arc::make_mut(&mut Arc::make_mut(&mut wrong.params)[0]).x.typ = bad_cell;
    let wrong = first.new_x(wrong);
    let mut bad_krate = (*k).clone();
    bad_krate.functions = vec![wrong.clone()];
    assert!(
        vir::internal_repr::validate_krate_uses(&Arc::new(bad_krate)).is_err(),
        "backend use hook must reject C<u32>"
    );
    let (bad_result, bad_errors, _) = vir::well_formed::check_function(
        &k,
        stubs.clone(),
        initial.lookup.clone(),
        &demote,
        &wrong,
        &warnings,
        true,
        false,
    );
    let mut bad_notes: Vec<String> = bad_errors.iter().map(|e| e.note.clone()).collect();
    if let Err(e) = bad_result {
        bad_notes.extend(e.errors.iter().map(|e| e.note.clone()));
    }
    assert!(
        bad_notes.iter().any(|n| n.contains("only the actual u64")),
        "WF must reject actual C<u32> use: {bad_notes:?}"
    );
    for f in &k.functions {
        let (result, errors, _) = vir::well_formed::check_function(
            &k,
            stubs.clone(),
            initial.lookup.clone(),
            &demote,
            f,
            &warnings,
            true,
            false,
        );
        let mut notes: Vec<String> = errors.iter().map(|e| e.note.clone()).collect();
        if let Err(e) = result {
            notes.extend(e.errors.iter().map(|e| e.note.clone()));
            notes.extend(e.boundary_errors.iter().map(|(_, e)| e.note.clone()));
        }
        eprintln!("D1 full Function WF {}: {:?}", f.x.name.path.segments.last().unwrap(), notes);
        assert!(
            notes.iter().any(|e| e.contains("opaque datatype")),
            "expected ordinary private-field WF refusal, got {notes:?}"
        );
    }
}
