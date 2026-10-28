//! Bounded modeling of Rust's builtin tuple Clone shim.
//! The public helpers are verified field-cloning implementations. The shim's
//! return relation permits either cloning or copying each field, via `cloned`.
use rustc_hir::def_id::DefId;
use rustc_middle::ty::{GenericArgsRef, TyCtxt, TyKind};
use rustc_span::Symbol;
use std::sync::Arc;
use vir::ast::*;

pub(crate) fn helper<'tcx>(
    tcx: TyCtxt<'tcx>,
    method: DefId,
    args: GenericArgsRef<'tcx>,
) -> Option<(DefId, GenericArgsRef<'tcx>)> {
    if crate::verus_items::get_rust_item(tcx, method)
        != Some(crate::verus_items::RustItem::CloneClone)
    {
        return None;
    }
    let TyKind::Tuple(fields) = args.first()?.as_type()?.kind() else { return None };
    if !(1..=4).contains(&fields.len()) {
        return None;
    }
    let id = tcx.get_diagnostic_item(Symbol::intern(&format!(
        "verus::vstd::std_specs::clone::tuple_clone_{}",
        fields.len()
    )))?;
    Some((id, tcx.mk_args_from_iter(fields.iter().map(rustc_middle::ty::GenericArg::from))))
}
fn arity(f: &FunctionStubX) -> Option<usize> {
    let p = &f.name.path;
    if p.krate != CrateId::Vstd
        || p.segments.len() != 3
        || p.segments[0].as_str() != "std_specs"
        || p.segments[1].as_str() != "clone"
    {
        return None;
    }
    (1..=4).find(|n| p.segments[2].as_str() == format!("tuple_clone_{n}"))
}
fn same(a: &Typ, b: &Typ) -> bool {
    vir::ast_util::types_equal(a, b)
}
pub(crate) fn adjust_stub(f: &mut FunctionStubX) -> Result<(), VirErr> {
    let Some(n) = arity(f) else { return Ok(()) };
    let bad = || vir::messages::error_bare("invalid builtin tuple Clone helper signature");
    if f.mode != Mode::Exec
        || f.attrs.is_external_body
        || f.attrs.is_unsafe
        || f.attrs.is_async
        || f.attrs.infer
        || f.params.len() != 1
        || f.typ_params.len() != n
        || f.params[0].x.mode != Mode::Exec
        || f.outer_ret.x.mode != Mode::Exec
    {
        return Err(bad());
    }
    let TypX::Decorate(TypDecoration::Ref, None, typ) = &*f.params[0].x.typ else {
        return Err(bad());
    };
    let TypX::Datatype(Dt::Tuple(m), ts, _) = &**typ else { return Err(bad()) };
    if *m != n
        || ts.len() != n
        || !same(typ, &f.outer_ret.x.typ)
        || !ts
            .iter()
            .zip(f.typ_params.iter())
            .all(|(t, n)| matches!(&**t,TypX::TypParam(p) if p==n))
    {
        return Err(bad());
    }
    let trait_path = Arc::new(PathX {
        krate: CrateId::Core,
        segments: Arc::new(vec![Arc::new("clone".into()), Arc::new("Clone".into())]),
    });
    f.kind = FunctionKind::TraitMethodImpl {
        method: Arc::new(FunX { path: trait_path.push_segment(Arc::new("clone".into())) }),
        trait_path,
        impl_path: f.name.path.push_segment(Arc::new("builtin_impl".into())),
        trait_typ_args: Arc::new(vec![typ.clone()]),
        inherit_body_from: None,
    };
    Ok(())
}
pub(crate) fn register(stubs: &[FunctionStub], krate: &mut KrateX) {
    for f in stubs.iter().filter(|f| arity(&f.x).is_some()) {
        let FunctionKind::TraitMethodImpl { trait_path, impl_path, trait_typ_args, .. } = &f.x.kind
        else {
            continue;
        };
        krate.trait_impls.push(vir::def::Spanned::new(
            f.span.clone(),
            TraitImplX {
                impl_path: impl_path.clone(),
                typ_params: f.x.typ_params.clone(),
                typ_bounds: f.x.typ_bounds.clone(),
                trait_path: trait_path.clone(),
                trait_typ_args: trait_typ_args.clone(),
                trait_typ_arg_impls: vir::def::Spanned::new(f.span.clone(), Arc::new(vec![])),
                owning_module: f.x.owning_module.clone(),
                auto_imported: false,
                external_trait_blanket: false,
            },
        ));
    }
}

/// The frontend replacement must never inherit a stronger/different helper
/// contract than the builtin shim's fieldwise reflexive-clone guarantee.
pub(crate) fn validate_full(f: &Function) -> Result<(), VirErr> {
    let Some(n) = ({
        let p = &f.x.name.path;
        (p.krate == CrateId::Vstd
            && p.segments.len() == 3
            && p.segments[0].as_str() == "std_specs"
            && p.segments[1].as_str() == "clone")
            .then(|| (1..=4).find(|n| p.segments[2].as_str() == format!("tuple_clone_{n}")))
            .flatten()
    }) else {
        return Ok(());
    };
    let x = &f.x;
    let bad = || vir::messages::error(&f.span, "invalid builtin tuple Clone helper contract");
    if !x.require.is_empty()
        || !x.ensure.1.is_empty()
        || x.ensure.0.len() != n
        || x.returns.is_some()
        || x.body.is_none()
        || x.attrs.is_external_body
        || x.attrs.is_unsafe
        || x.attrs.is_async
        || x.attrs.exec_assume_termination
        || x.attrs.exec_allows_no_decreases_clause
    {
        return Err(bad());
    }
    let mut sizes = vec![0; n];
    let mut clones = vec![0; n];
    for bound in x.typ_bounds.iter() {
        let GenericBoundX::Trait(trait_id, types) = &**bound else { return Err(bad()) };
        if types.len() != 1 {
            return Err(bad());
        }
        let TypX::TypParam(param) = &*types[0] else { return Err(bad()) };
        let i = x.typ_params.iter().position(|p| p == param).ok_or_else(bad)?;
        match trait_id {
            TraitId::Sizedness(Sizedness::Sized) => sizes[i] += 1,
            TraitId::Path(p)
                if p.krate == CrateId::Core
                    && p.segments.iter().map(|s| s.as_str()).eq(["clone", "Clone"]) =>
            {
                clones[i] += 1
            }
            _ => return Err(bad()),
        }
    }
    if sizes.iter().any(|n| *n != 1) || clones.iter().any(|n| *n != 1) {
        return Err(bad());
    }
    fn projection(e: &Expr, root: &VarIdent, index: usize, n: usize) -> bool {
        let op = |o: &FieldOpr| {
            o.datatype == Dt::Tuple(n)
                && o.field == vir::def::positional_field_ident(index)
                && o.variant == vir::def::prefix_tuple_variant(n)
                && !o.get_variant
                && o.check == VariantCheck::None
        };
        let local = |e: &Expr| {
            matches!(&e.x,ExprX::Var(v) if v==root)
                || matches!(&e.x,ExprX::ReadPlace(p,_) if matches!(&p.x,PlaceX::Local(v) if v==root))
        };
        match &e.x {
            ExprX::UnaryOpr(UnaryOpr::Field(o), b) => op(o) && local(b),
            ExprX::ReadPlace(p, _) => {
                matches!(&p.x,PlaceX::Field(o,b) if op(o)&&matches!(&b.x,PlaceX::Local(v) if v==root))
            }
            _ => false,
        }
    }
    for (i, clause) in x.ensure.0.iter().enumerate() {
        let ExprX::Call {
            target: CallTarget::Fun(CallTargetKind::Static, name, types, impls, _),
            args,
            post_args: None,
            body: None,
        } = &clause.x
        else {
            return Err(bad());
        };
        if name.path.krate != CrateId::Vstd
            || !name.path.segments.iter().map(|s| s.as_str()).eq(["pervasive", "cloned"])
            || types.len() != 1
            || !matches!(&*types[0],TypX::TypParam(p) if p==&x.typ_params[i])
            || args.len() != 2
            || !projection(&args[0], &x.params[0].x.name, i, n)
            || !projection(&args[1], &x.inner_ret.x.name, i, n)
        {
            return Err(bad());
        }
        // Impl paths come from Rust's checked Clone bounds; no extra clauses occur.
        let _ = impls;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuple_shim_mapping_rejects_stronger_contracts_and_signature_changes() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("target-verus/release/vstd.vir");
        let mut file = std::io::BufReader::new(std::fs::File::open(path).unwrap());
        #[derive(serde::Deserialize)]
        struct Library {
            krate: Krate,
            #[allow(dead_code)]
            metadata: crate::import_export::CrateMetadata,
        }
        let library: Library =
            bincode_next::serde::decode_from_std_read(&mut file, bincode_next::config::legacy())
                .unwrap();
        for n in 1..=4 {
            let original = library
                .krate
                .functions
                .iter()
                .find(|f| f.x.name.path.last_segment().as_str() == format!("tuple_clone_{n}"))
                .unwrap();
            // Imported exec bodies are erased. This sentinel tests only the
            // mapping gate; vstd's actual body receives ordinary verification.
            let mut x = original.x.clone();
            x.body = Some(SpannedTyped::new(
                &original.span,
                &vir::ast_util::unit_typ(),
                ExprX::Ctor(
                    Dt::Tuple(0),
                    vir::def::prefix_tuple_variant(0),
                    Arc::new(vec![]),
                    None,
                ),
            ));
            let valid = original.new_x(x);
            validate_full(&valid).unwrap();
            let mut stub = vir::ast_util::function_to_stub(&valid).x.clone();
            adjust_stub(&mut stub).unwrap();
            for mutation in 0..6 {
                let mut x = valid.x.clone();
                match mutation {
                    0 => x.require = Arc::new(vec![valid.x.ensure.0[0].clone()]),
                    1 => Arc::make_mut(&mut x.ensure.0).push(valid.x.ensure.0[0].clone()),
                    2 => {
                        Arc::make_mut(&mut x.ensure.0)[0] = SpannedTyped::new(
                            &valid.span,
                            &vir::ast_util::bool_typ(),
                            ExprX::Const(Constant::Bool(true)),
                        )
                    }
                    3 => Arc::make_mut(&mut x.attrs).is_external_body = true,
                    4 => Arc::make_mut(&mut x.attrs).exec_assume_termination = true,
                    5 => Arc::make_mut(&mut x.typ_bounds).push(valid.x.typ_bounds[0].clone()),
                    _ => unreachable!(),
                }
                assert!(
                    validate_full(&valid.new_x(x)).is_err(),
                    "accepted mutation {} for arity {}",
                    mutation,
                    n
                );
            }
            stub.outer_ret = stub
                .outer_ret
                .new_x(ParamX { typ: vir::ast_util::bool_typ(), ..stub.outer_ret.x.clone() });
            assert!(adjust_stub(&mut stub).is_err());
        }
    }
}
