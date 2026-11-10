//! Closed native-for observations shared by checked annotation passes.
//! Recognition is bounded and is never proof evidence.
use super::*;
use crate::ast_visitor::{VisitorControlFlow, expr_visitor_walk};

pub(super) const NODE_BUDGET: usize = 16384;
pub(super) const CAPACITIES: &[usize] = &[10, 128];

#[derive(Clone)]
pub(super) struct NativeMutableFor {
    pub loop_expr: Expr,
    pub buffer: Param,
    pub capacity: usize,
    pub key: Option<Expr>,
    pub wrapper: Expr,
    pub iterator: Typ,
}

pub(super) fn byte(t: &Typ) -> bool {
    matches!(&**t, TypX::Int(IntRange::U(8)))
}
pub(super) fn ty(t: &Typ, k: CrateId, path: &[&str]) -> Option<Typs> {
    let TypX::Datatype(Dt::Path(p), ts, _) = &**t else { return None };
    models::path_is(p, k, path).then(|| ts.clone())
}
pub(super) fn native(t: &Typ, mutable: bool) -> bool {
    ty(t, CrateId::Core, &["slice", "iter", if mutable { "IterMut" } else { "Iter" }])
        .is_some_and(|ts| ts.len() == 1 && byte(&ts[0]))
}
pub(super) fn slice(t: &Typ) -> bool {
    matches!(&**t, TypX::Primitive(Primitive::Slice, ts) if ts.len() == 1 && byte(&ts[0]))
}
pub(super) fn array(t: &Typ) -> Option<usize> {
    let TypX::Primitive(Primitive::Array, ts) = &**t else { return None };
    if ts.len() != 2 || !byte(&ts[0]) {
        return None;
    }
    let TypX::ConstInt(n) = &*ts[1] else { return None };
    CAPACITIES.iter().copied().find(|v| *n == BigInt::from(*v))
}
pub(super) fn resolved(e: &Expr) -> Option<(&Fun, &Typs, &Exprs)> {
    let ExprX::Call { target: CallTarget::Fun(k, n, ts, _, _), args, post_args: None, body: None } =
        &e.x
    else {
        return None;
    };
    match k {
        CallTargetKind::Static | CallTargetKind::ExternalTraitDefault => Some((n, ts, args)),
        CallTargetKind::DynamicResolved { resolved, typs, .. } => Some((resolved, typs, args)),
        _ => None,
    }
}
fn generic(f: &Function, i: usize) -> Typ {
    Arc::new(TypX::TypParam(f.x.typ_params[i].clone()))
}
pub(super) fn same(a: &Typ, b: &Typ) -> bool {
    crate::ast_util::types_equal(a, b)
}
fn generic_slice(t: &Typ, g: &Typ) -> bool {
    matches!(&**t, TypX::Primitive(Primitive::Slice, ts) if ts.len() == 1 && same(&ts[0],g))
}
/// Closed expected generic bounds; compare a multiset so source ordering is
/// irrelevant, while extra/duplicate/strengthened bounds reject the catalog.
pub(super) fn bounds(f: &Function, kind: &str) -> bool {
    let g = |i| generic(f, i);
    let sized = |t, s| Arc::new(GenericBoundX::Trait(TraitId::Sizedness(s), Arc::new(vec![t])));
    let trait_bound = |t, path: &[&str]| {
        Arc::new(GenericBoundX::Trait(
            TraitId::Path(Arc::new(PathX {
                krate: CrateId::Core,
                segments: Arc::new(path.iter().map(|s| Arc::new((*s).into())).collect()),
            })),
            Arc::new(vec![t]),
        ))
    };
    let iter = &["iter", "traits", "iterator", "Iterator"];
    let expected = match kind {
        "slice" if f.x.typ_params.len() == 1 => vec![sized(g(0), Sizedness::Sized)],
        "array" if f.x.typ_params.len() == 2 => vec![
            sized(g(0), Sizedness::Sized),
            Arc::new(GenericBoundX::ConstTyp(g(1), Arc::new(TypX::Int(IntRange::USize)))),
        ],
        "wrapper" if f.x.typ_params.len() == 1 => {
            vec![sized(g(0), Sizedness::Sized), trait_bound(g(0), iter)]
        }
        "zip" if f.x.typ_params.len() == 2 => vec![
            sized(g(0), Sizedness::MetaSized),
            trait_bound(g(0), iter),
            sized(g(1), Sizedness::Sized),
            sized(g(0), Sizedness::Sized),
            trait_bound(g(1), &["iter", "traits", "collect", "IntoIterator"]),
        ],
        _ => return false,
    };
    let encode = |v: &[GenericBound]| {
        let mut v: Vec<_> = v.iter().map(|v| serde_json::to_vec(v).expect("VIR bound")).collect();
        v.sort();
        v
    };
    encode(&expected) == encode(&f.x.typ_bounds)
}
fn iter_mut(f: &Function) -> bool {
    if !models::path_is(&f.x.name.path, CrateId::Core, &["slice", "impl&%0", "iter_mut"])
        || !f
            .x
            .proxy
            .as_ref()
            .is_some_and(|p| models::spec_proxy(&p.x, &["std_specs", "slice"], "iter__mut"))
        || f.x.mode != Mode::Exec
        || !matches!(f.x.kind, FunctionKind::Static)
        || f.x.typ_params.len() != 1
        || f.x.params.len() != 1
        || f.x.params[0].x.mode != Mode::Exec
        || f.x.params[0].x.user_mut
        || f.x.inner_ret.x.mode != Mode::Exec
        || !bounds(f, "slice")
    {
        return false;
    }
    let g = generic(f, 0);
    matches!(&*f.x.params[0].x.typ, TypX::MutRef(t) if generic_slice(t,&g))
        && ty(&f.x.inner_ret.x.typ, CrateId::Core, &["slice", "iter", "IterMut"])
            .is_some_and(|ts| ts.len() == 1 && same(&ts[0], &g))
}

/// Exact native wrapper member, with a closed generic receiver and mode.
pub(super) fn wrapper_member(f: &Function, name: &str) -> bool {
    let p = &f.x.name.path;
    p.krate == CrateId::Vstd && p.segments.len() == 4
        && p.segments[0].as_str()=="std_specs" && p.segments[1].as_str()=="iter"
        && p.segments[2].starts_with("impl&%") && p.segments[3].as_str()==name
        && f.x.proxy.is_none() && matches!(f.x.kind,FunctionKind::Static)
        && f.x.mode==Mode::Spec && f.x.typ_params.len()==1 && f.x.params.len()==1
        && bounds(f,"wrapper") && f.x.inner_ret.x.mode==Mode::Spec
        && f.x.params[0].x.mode==Mode::Spec && !f.x.params[0].x.user_mut
        && ty(&f.x.params[0].x.typ, CrateId::Vstd, &["std_specs","iter","VerusForLoopWrapper"])
            .is_some_and(|ts| ts.len()==1 && same(&ts[0],&generic(f,0)))
        && match name {
            "wf" => same(&f.x.inner_ret.x.typ,&bool_typ()),
            "index" => same(&f.x.inner_ret.x.typ,&int_typ()),
            "seq" => iterators::sequence(&f.x.inner_ret.x.typ).is_some_and(|t|
                matches!(&**t,TypX::Projection { trait_typ_args, trait_path, name }
                    if trait_typ_args.len()==1 && same(&trait_typ_args[0],&generic(f,0))
                    && models::path_is(trait_path,CrateId::Core,&["iter","traits","iterator","Iterator"])
                    && name.as_str()=="Item")),
            _ => false,
        }
}

/// A single native mutable for, directly borrowing a bounded byte-array input.
/// Dynamic slices are supported on the immutable Zip side, without a key bound.
/// Multiple loops, mutable slice capacity proofs, adapters and custom iterators skip.
pub(super) fn describe(f: &Function, all: &HashMap<Fun, Function>) -> Option<NativeMutableFor> {
    if f.x.mode != Mode::Exec
        || f.x.name.path.krate == CrateId::Vstd
        || f.x.attrs.infer
        || f.x.attrs.is_external_body
        || f.x.attrs.is_unsafe
        || f.x.attrs.is_async
        || f.x.attrs.bit_vector
        || f.x.attrs.exec_assume_termination
        || f.x.attrs.exec_allows_no_decreases_clause
    {
        return None;
    }
    let body = f.x.body.as_ref()?;
    let mut loops = vec![];
    let mut mut_calls = vec![];
    let mut zip_calls = vec![];
    let mut budget = NODE_BUDGET;
    expr_visitor_walk(body, &mut |e| {
        if budget == 0 {
            return VisitorControlFlow::Stop(());
        }
        budget -= 1;
        if matches!(&e.x, ExprX::Loop { .. }) {
            loops.push(e.clone());
        }
        if let Some((n, _, _)) = resolved(e) {
            if all.get(n).is_some_and(iter_mut) {
                mut_calls.push(e.clone());
            }
            if models::path_is(
                &n.path,
                CrateId::Core,
                &["iter", "traits", "iterator", "Iterator", "zip"],
            ) {
                zip_calls.push(e.clone());
            }
        }
        VisitorControlFlow::Recurse
    });
    if budget == 0 || loops.len() != 1 || mut_calls.len() != 1 || zip_calls.len() > 1 {
        return None;
    }
    let loop_expr = loops.pop()?;
    let ExprX::Loop {
        is_for_loop: true,
        assume_termination: false,
        atomic_call: false,
        cond: None,
        invs,
        ..
    } = &loop_expr.x
    else {
        return None;
    };
    let (_, mts, ma) = resolved(&mut_calls[0])?;
    if mts.len() != 1 || !byte(&mts[0]) || ma.len() != 1 || !native(&mut_calls[0].typ, true) {
        return None;
    }
    let (n, ts, args) = resolved(&ma[0])?;
    let coercion = all.get(n)?;
    if !models::path_is(&n.path, CrateId::Vstd, &["array", "ref_mut_array_unsizing_coercion"])
        || coercion.x.mode != Mode::Exec
        || coercion.x.typ_params.len() != 2
        || !bounds(coercion, "array")
        || coercion.x.params.len() != 1
        || coercion.x.params[0].x.mode != Mode::Exec
        || !matches!(coercion.x.kind, FunctionKind::Static)
        || ts.len() != 2
        || !byte(&ts[0])
        || args.len() != 1
    {
        return None;
    }
    let cg = generic(coercion, 0);
    let ca = Arc::new(TypX::Primitive(
        Primitive::Array,
        Arc::new(vec![cg.clone(), generic(coercion, 1)]),
    ));
    if coercion.x.params[0].x.user_mut
        || coercion.x.inner_ret.x.mode != Mode::Exec
        || !same(&coercion.x.params[0].x.typ, &Arc::new(TypX::MutRef(ca)))
        || !matches!(&*coercion.x.inner_ret.x.typ, TypX::MutRef(t) if generic_slice(t,&cg))
    {
        return None;
    }
    let (ExprX::BorrowMut(p) | ExprX::TwoPhaseBorrowMut(p)) = &args[0].x else { return None };
    let PlaceX::DerefMut(p) = &p.x else { return None };
    let PlaceX::Local(v) = &p.x else { return None };
    let buffer = f.x.params.iter().find(|p| p.x.name == *v)?.clone();
    if buffer.x.mode != Mode::Exec || buffer.x.user_mut {
        return None;
    }
    let TypX::MutRef(t) = &*buffer.x.typ else { return None };
    let capacity = array(t)?;
    if !same(&p.typ, &buffer.x.typ)
        || !same(&ts[1], &Arc::new(TypX::ConstInt(BigInt::from(capacity))))
    {
        return None;
    }
    let mut iterator = mut_calls[0].typ.clone();
    let key = if let Some(zip) = zip_calls.first() {
        let (n, ts, a) = resolved(zip)?;
        let z = all.get(n)?;
        if z.x.mode != Mode::Exec
            || z.x.typ_params.len() != 2
            || !bounds(z, "zip")
            || z.x.params.len() != 2
            || !z.x.params.iter().all(|p| p.x.mode == Mode::Exec && !p.x.user_mut)
            || !same(&z.x.params[0].x.typ, &generic(z, 0))
            || !same(&z.x.params[1].x.typ, &generic(z, 1))
            || !z.x.proxy.as_ref().is_some_and(|p| {
                models::path_is(&p.x, CrateId::Vstd, &["std_specs", "iter", "ExIterator", "zip"])
            })
            || ts.len() != 2
            || !same(&ts[0], &iterator)
            || a.len() != 2
            || serde_json::to_vec(&a[0]).ok()? != serde_json::to_vec(&mut_calls[0]).ok()?
        {
            return None;
        }
        let signature =
            ty(&z.x.inner_ret.x.typ, CrateId::Core, &["iter", "adapters", "zip", "Zip"])?;
        if !matches!(&z.x.kind, FunctionKind::TraitMethodDecl {trait_path,has_default:false}
            if models::path_is(trait_path,CrateId::Core,&["iter","traits","iterator","Iterator"]))
            || z.x.inner_ret.x.mode != Mode::Exec
            || signature.len() != 2
            || !same(&signature[0], &generic(z, 0))
            || !matches!(&*signature[1], TypX::Projection {trait_typ_args,trait_path,name}
                if trait_typ_args.len()==1 && same(&trait_typ_args[0],&generic(z,1))
                && models::path_is(trait_path,CrateId::Core,&["iter","traits","collect","IntoIterator"])
                && name.as_str()=="IntoIter")
        {
            return None;
        }
        let TypX::Decorate(TypDecoration::Ref, None, kt) = &*a[1].typ else { return None };
        if !slice(kt) || !same(&ts[1], &a[1].typ) {
            return None;
        }
        let zts = ty(&zip.typ, CrateId::Core, &["iter", "adapters", "zip", "Zip"])?;
        if zts.len() != 2 || !same(&zts[0], &iterator) || !native(&zts[1], false) {
            return None;
        }
        iterator = zip.typ.clone();
        Some(a[1].clone())
    } else {
        None
    };
    let mut wrappers = vec![];
    for inv in invs.iter() {
        expr_visitor_walk(&inv.inv, &mut |e| {
            if let Some((n, ts, args)) = resolved(e) {
                if all.get(n).is_some_and(|f| wrapper_member(f, "wf"))
                    && ts.len() == 1
                    && same(&ts[0], &iterator)
                    && args.len() == 1
                {
                    wrappers.push(args[0].clone());
                }
            }
            VisitorControlFlow::<()>::Recurse
        });
    }
    if wrappers.len() != 1 {
        return None;
    }
    Some(NativeMutableFor { loop_expr, buffer, capacity, key, wrapper: wrappers.pop()?, iterator })
}
