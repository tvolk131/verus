//! Immutable shared slices, expressed with Verus's existing length/index IR.
//! No enumeration or length cap is needed: each read remains a symbolic term.
use super::*;

/// A local immutable view. This is candidate-generation data, never a VIR
/// expression, cached formula, executable rewrite, or trusted slice axiom.
#[derive(Clone)]
pub(super) struct View {
    pub base: Term,
    pub start: Term,
    pub end: Term,
}

pub(super) fn element(typ: &Typ) -> Option<Typ> {
    match &**arrays::readonly_base(typ) {
        TypX::Primitive(Primitive::Slice, args) if args.len() == 1 && scalar(&args[0]) => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

pub(super) fn input_type(typ: &Typ) -> bool {
    matches!(&**typ, TypX::Decorate(TypDecoration::Ref, None, inner)
        if matches!(&**inner, TypX::Primitive(Primitive::Slice, _)) && element(inner).is_some())
}

pub(super) fn compatible(a: &Typ, b: &Typ) -> bool {
    match (element(a), element(b)) {
        (Some(a), Some(b)) => crate::ast_util::types_equal(&a, &b),
        _ => false,
    }
}

/// Array-to-slice coercions can keep the array's exact symbolic value during
/// substitution. Length and indexing then specialize to the original array.
/// The reverse direction is never valid.
pub(super) fn substitutes(actual: &Typ, expected: &Typ) -> bool {
    match (arrays::shape(actual), element(expected)) {
        (Some((a, _)), Some(b)) => crate::ast_util::types_equal(&a, &b),
        _ => false,
    }
}

pub(super) fn length(span: &Span, base: Term) -> Result<Term, VirErr> {
    if let Some(view) = base.view {
        return summary::binary(
            span,
            BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)),
            view.end,
            view.start,
        );
    }
    if let Some((_, n)) = arrays::shape(&base.expr.typ) {
        return term(span, &int_typ(), ExprX::Const(Constant::Int(n.into())), 1);
    }
    if element(&base.expr.typ).is_none() {
        return Err(unsupported(span, "length requires an immutable scalar slice or array"));
    }
    term(
        span,
        &int_typ(),
        ExprX::Unary(UnaryOp::Length(ArrayKind::Slice), base.expr),
        1 + base.size,
    )
}

pub(super) fn index(span: &Span, base: Term, index: Term) -> Result<Term, VirErr> {
    if let Some(view) = base.view {
        let index = summary::binary(
            span,
            BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Allow)),
            view.start,
            index,
        )?;
        return self::index(span, view.base, index);
    }
    if arrays::shape(&base.expr.typ).is_some() {
        return arrays::index(span, base, index);
    }
    let typ = element(&base.expr.typ)
        .ok_or_else(|| unsupported(span, "indexing requires an immutable scalar slice"))?;
    if !eligibility::integer(&index.expr.typ) {
        return Err(unsupported(span, "noninteger slice index"));
    }
    term(
        span,
        &typ,
        ExprX::Binary(BinaryOp::Index(ArrayKind::Slice, BoundsCheck::Allow), base.expr, index.expr),
        1 + base.size + index.size,
    )
}

pub(super) fn bound(span: &Span, base: Term, index: Term) -> Result<Term, VirErr> {
    let zero = term(span, &int_typ(), ExprX::Const(Constant::Int(0.into())), 1)?;
    let lower = summary::binary(span, BinaryOp::Inequality(InequalityOp::Le), zero, index.clone())?;
    let upper =
        summary::binary(span, BinaryOp::Inequality(InequalityOp::Lt), index, length(span, base)?)?;
    logical(span, LogicalOp::And, lower, upper)
}

use super::models::reference;
fn core_path(path: &Path, parts: &[&str]) -> bool {
    models::path_is(path, CrateId::Core, parts)
}

/// Canonical `[T] as Index<I>` declaration. Only a complete concrete range call
/// is admitted below; this does not permit arbitrary Index implementations.
pub(super) fn range_model(function: &Function) -> bool {
    let f = &function.x;
    let p = &f.name.path;
    let Some(proxy) = &f.proxy else { return false };
    if p.krate != CrateId::Core
        || p.segments.len() != 4
        || p.segments[0].as_str() != "slice"
        || p.segments[1].as_str() != "index"
        || !p.segments[2].starts_with("impl")
        || p.segments[3].as_str() != "index"
        || !models::spec_proxy(&proxy.x, &["std_specs", "slice"], "_index")
        || f.mode != Mode::Exec
        || !matches!(f.item_kind, ItemKind::Function)
        || f.body.is_some()
        || f.attrs.infer
        || f.attrs.is_unsafe
        || f.attrs.is_async
        || f.attrs.is_external_body
        || f.typ_params.len() != 2
        || f.params.len() != 2
        || f.params.iter().any(|p| p.x.user_mut || p.x.mode != Mode::Exec)
    {
        return false;
    }
    let ts: Vec<_> = f.typ_params.iter().map(|n| Arc::new(TypX::TypParam(n.clone()))).collect();
    let slice = Arc::new(TypX::Primitive(Primitive::Slice, Arc::new(vec![ts[0].clone()])));
    let mut sized = [0; 2];
    let mut index = 0;
    for bound in f.typ_bounds.iter() {
        match &**bound {
            GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), args) if args.len() == 1 => {
                let Some(i) = ts.iter().position(|t| crate::ast_util::types_equal(t, &args[0]))
                else {
                    return false;
                };
                sized[i] += 1;
            }
            GenericBoundX::Trait(TraitId::Path(p), args)
                if core_path(p, &["slice", "index", "SliceIndex"])
                    && args.len() == 2
                    && crate::ast_util::types_equal(&args[0], &ts[1])
                    && crate::ast_util::types_equal(&args[1], &slice) =>
            {
                index += 1;
            }
            _ => return false,
        }
    }
    let ret = reference(Arc::new(TypX::Projection {
        trait_typ_args: Arc::new(vec![ts[1].clone(), slice.clone()]),
        trait_path: Arc::new(PathX {
            krate: CrateId::Core,
            segments: Arc::new(
                ["slice", "index", "SliceIndex"].into_iter().map(|s| Arc::new(s.into())).collect(),
            ),
        }),
        name: Arc::new("Output".into()),
    }));
    sized == [1, 1]
        && index == 1
        && crate::ast_util::types_equal(&f.params[0].x.typ, &reference(slice))
        && crate::ast_util::types_equal(&f.params[1].x.typ, &ts[1])
        && crate::ast_util::types_equal(&f.inner_ret.x.typ, &ret)
        && crate::ast_util::types_equal(&f.outer_ret.x.typ, &ret)
}

/// After trait lowering, a canonical Index requirement is attached to the
/// abstract trait call. Admit it only when the resolved declaration and concrete
/// types identify the same closed shared-range operation recognized at source.
pub(super) fn maximality_model(function: &Function, resolved: &Function, typs: &Typs) -> bool {
    let f = &function.x;
    if !core_path(&f.name.path, &["ops", "index", "Index", "index"])
        || f.mode != Mode::Exec
        || f.body.is_some()
        || f.params.len() != 2
        || f.typ_params.len() != 2
        || f.attrs.infer
        || f.attrs.is_unsafe
        || f.attrs.is_async
        || !range_model(resolved)
        || typs.len() != 2
    {
        return false;
    }
    let Some(proxy) = &f.proxy else { return false };
    if proxy.x.krate != CrateId::Vstd
        || proxy.x.segments.iter().map(|s| s.as_str()).collect::<Vec<_>>()
            != ["std_specs", "core", "ExIndex", "index"]
    {
        return false;
    }
    let ts: Vec<_> = f.typ_params.iter().map(|n| Arc::new(TypX::TypParam(n.clone()))).collect();
    let trait_path = Arc::new(PathX {
        krate: CrateId::Core,
        segments: Arc::new(
            ["ops", "index", "Index"].into_iter().map(|s| Arc::new(s.into())).collect(),
        ),
    });
    let ret = reference(Arc::new(TypX::Projection {
        trait_typ_args: Arc::new(ts.clone()),
        trait_path: trait_path.clone(),
        name: Arc::new("Output".into()),
    }));
    if !matches!(&f.kind,FunctionKind::TraitMethodDecl{trait_path:p,has_default:false} if p==&trait_path)
        || !crate::ast_util::types_equal(&f.params[0].x.typ, &reference(ts[0].clone()))
        || !crate::ast_util::types_equal(&f.params[1].x.typ, &ts[1])
        || !crate::ast_util::types_equal(&f.inner_ret.x.typ, &ret)
        || !crate::ast_util::types_equal(&f.outer_ret.x.typ, &ret)
        || f.params.iter().any(|p| p.x.mode != Mode::Exec || p.x.user_mut)
    {
        return false;
    }
    let mut meta = [0; 2];
    let mut sized = 0;
    let mut index = 0;
    for bound in f.typ_bounds.iter() {
        match &**bound {
            GenericBoundX::Trait(TraitId::Sizedness(Sizedness::MetaSized), args)
                if args.len() == 1 =>
            {
                let Some(i) = ts.iter().position(|t| crate::ast_util::types_equal(t, &args[0]))
                else {
                    return false;
                };
                meta[i] += 1;
            }
            GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), args)
                if args.len() == 1 && crate::ast_util::types_equal(&args[0], &ts[1]) =>
            {
                sized += 1
            }
            GenericBoundX::Trait(TraitId::Path(p), args)
                if p == &trait_path
                    && args.len() == 2
                    && args.iter().zip(&ts).all(|(a, b)| crate::ast_util::types_equal(a, b)) =>
            {
                index += 1
            }
            _ => return false,
        }
    }
    if meta != [1, 1] || sized != 1 || index != 1 {
        return false;
    }
    let TypX::Primitive(Primitive::Slice, elements) = &*typs[0] else { return false };
    let TypX::Datatype(Dt::Path(path), ts, impls) = &*typs[1] else { return false };
    let kind = path.segments.last().map(|s| s.as_str()).unwrap_or("");
    elements.len() == 1
        && scalar(&elements[0])
        && impls.is_empty()
        && matches!(kind, "Range" | "RangeFrom" | "RangeTo" | "RangeFull")
        && core_path(path, &["ops", "range", kind])
        && ts.len() == usize::from(kind != "RangeFull")
        && ts.iter().all(|t| matches!(&**t, TypX::Int(IntRange::USize)))
}

pub(super) struct RangeCall {
    pub helper: Expr,
    pub base: Expr,
    pub fields: Vec<(String, Expr)>,
}

pub(super) fn range_call(
    expr: &Expr,
    functions: &HashMap<Fun, Function>,
) -> Result<Option<RangeCall>, VirErr> {
    let ExprX::Call {
        target: CallTarget::Fun(kind, original_name, original_typs, original_impls, attrs),
        args,
        post_args: None,
        body: None,
    } = &expr.x
    else {
        return Ok(None);
    };
    let (name, typs, impls) = match kind {
        CallTargetKind::Static => (original_name, original_typs, original_impls),
        CallTargetKind::DynamicResolved { resolved, typs, impl_paths, is_trait_default: false } => {
            (resolved, typs, impl_paths)
        }
        _ => return Ok(None),
    };
    if !functions.get(name).is_some_and(range_model) {
        return Ok(None);
    }
    let invalid = || unsupported(&expr.span, "unsupported shared subslice lowering");
    if typs.len() != 2
        || !scalar(&typs[0])
        || args.len() != 2
        || impls.len() != 1
        || attrs.const_var
        || attrs.assume_external_allowed
        || !matches!(attrs.autospec, AutospecUsage::Final)
        || !matches!(&impls[0],ImplPath::TraitImplPath(p) if p.krate==CrateId::Core && p.segments.len()==3 && p.segments[0].as_str()=="slice" && p.segments[1].as_str()=="index" && p.segments[2].starts_with("impl"))
    {
        return Err(invalid());
    }
    if matches!(kind, CallTargetKind::DynamicResolved { .. }) {
        let coherent = core_path(&original_name.path, &["ops", "index", "Index", "index"])
            && original_typs.len() == 2
            && original_impls.len() == 2
            && matches!(&*original_typs[0],TypX::Primitive(Primitive::Slice,ts) if ts.len()==1 && crate::ast_util::types_equal(&ts[0],&typs[0]))
            && crate::ast_util::types_equal(&original_typs[1], &typs[1])
            && original_impls[1] == impls[0]
            && matches!(&original_impls[0],ImplPath::TraitImplPath(p) if p.krate==CrateId::Core && p.segments.as_ref()==&name.path.segments[..3]);
        if !coherent {
            return Err(invalid());
        }
    }
    if !element(&args[0].typ).is_some_and(|t| crate::ast_util::types_equal(&t, &typs[0]))
        || !input_type(&expr.typ)
        || !element(&expr.typ).is_some_and(|t| crate::ast_util::types_equal(&t, &typs[0]))
    {
        return Err(invalid());
    }
    let ExprX::Ctor(Dt::Path(path), variant, fields, None) = &args[1].x else {
        return Err(invalid());
    };
    let kind = path.segments.last().map(|s| s.as_str()).unwrap_or("");
    if !core_path(path, &["ops", "range", kind])
        || variant.as_str() != kind
        || !crate::ast_util::types_equal(&args[1].typ, &typs[1])
    {
        return Err(invalid());
    }
    let expected: &[&str] = match kind {
        "Range" => &["start", "end"],
        "RangeFrom" => &["start"],
        "RangeTo" => &["end"],
        "RangeFull" => &[],
        _ => return Err(invalid()),
    };
    let TypX::Datatype(Dt::Path(dt), ts, _) = &*args[1].typ else { return Err(invalid()) };
    if dt != path
        || ts.len() != usize::from(kind != "RangeFull")
        || ts.iter().any(|t| !matches!(&**t, TypX::Int(IntRange::USize)))
        || fields.len() != expected.len()
        || expected.iter().any(|n| fields.iter().filter(|f| f.name.as_str() == *n).count() != 1)
        || fields.iter().any(|f| !matches!(&*f.a.typ, TypX::Int(IntRange::USize)))
    {
        return Err(invalid());
    }
    Ok(Some(RangeCall {
        helper: expr.new_x(ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::Static,
                name.clone(),
                typs.clone(),
                impls.clone(),
                attrs.clone(),
            ),
            args: args.clone(),
            post_args: None,
            body: None,
        }),
        base: args[0].clone(),
        fields: fields.iter().map(|f| (f.name.to_string(), f.a.clone())).collect(),
    }))
}

impl Engine<'_> {
    pub(super) fn subslice(&mut self, expr: &Expr, range: RangeCall) -> Result<Summary, VirErr> {
        let base = self.expr(&range.base)?;
        let mut safe = base.safe;
        let base =
            value(&Summary { safe: boolean(&expr.span, true), value: base.value }, &expr.span)?;
        let mut start = term(&expr.span, &int_typ(), ExprX::Const(Constant::Int(0.into())), 1)?;
        let mut end = length(&expr.span, base.clone())?;
        for (name, e) in range.fields {
            let s = self.expr(&e)?;
            safe = logical(&expr.span, LogicalOp::And, safe, s.safe.clone())?;
            if name == "start" {
                start = value(&s, &expr.span)?;
            } else {
                end = value(&s, &expr.span)?;
            }
        }
        let lower = summary::binary(
            &expr.span,
            BinaryOp::Inequality(InequalityOp::Le),
            start.clone(),
            end.clone(),
        )?;
        let upper = summary::binary(
            &expr.span,
            BinaryOp::Inequality(InequalityOp::Le),
            end.clone(),
            length(&expr.span, base.clone())?,
        )?;
        let bounds = logical(&expr.span, LogicalOp::And, lower, upper)?;
        safe = logical(&expr.span, LogicalOp::And, safe, self.require_active(&expr.span, bounds)?)?;
        let size = 1 + base.size + start.size + end.size;
        if size > FORMULA_BUDGET {
            return Err(error(&expr.span, "Verus Infer incomplete: expanded view budget exceeded"));
        }
        Ok(Summary {
            safe,
            value: Some(Term {
                expr: SpannedTyped::new(&expr.span, &expr.typ, base.expr.x.clone()),
                size,
                view: Some(Box::new(View { base, start, end })),
            }),
        })
    }
}

/// Canonical spec declarations used implicitly by ordinary slice bounds/read
/// lowering. Their full imported artifact (including axioms) is fingerprinted.
pub(super) fn model(f: &Function) -> bool {
    let f = &f.x;
    let len = f.name == crate::def::fn_slice_len();
    let index = f.name == crate::def::fn_slice_index();
    if !(len || index)
        || f.mode != Mode::Spec
        || !matches!(f.kind, FunctionKind::Static)
        || !matches!(f.item_kind, ItemKind::Function)
        || f.typ_params.len() != 1
        || f.body.is_some()
        || f.proxy.is_some()
        || f.attrs.infer
        || f.params.len() != if len { 1 } else { 2 }
        || f.params.iter().any(|p| p.x.mode != Mode::Spec || p.x.user_mut)
        || !f.require.is_empty()
        || !f.ensure.0.is_empty()
        || !f.ensure.1.is_empty()
    {
        return false;
    }
    let t = Arc::new(TypX::TypParam(f.typ_params[0].clone()));
    let slice = Arc::new(TypX::Decorate(
        TypDecoration::Ref,
        None,
        Arc::new(TypX::Primitive(Primitive::Slice, Arc::new(vec![t.clone()]))),
    ));
    let output = if len { Arc::new(TypX::Int(IntRange::USize)) } else { t.clone() };
    f.typ_bounds.len() == 1
        && matches!(&*f.typ_bounds[0], GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), args)
            if args.len()==1 && crate::ast_util::types_equal(&args[0], &t))
        && crate::ast_util::types_equal(&f.params[0].x.typ, &slice)
        && (len || matches!(&*f.params[1].x.typ, TypX::Int(IntRange::Int)))
        && f.inner_ret.x.mode == Mode::Spec
        && f.outer_ret.x.mode == Mode::Spec
        && crate::ast_util::types_equal(&f.inner_ret.x.typ, &output)
        && crate::ast_util::types_equal(&f.outer_ret.x.typ, &output)
}

pub(super) fn dependencies(e: &Expr, functions: &HashMap<Fun, Function>) -> Vec<Fun> {
    fn indexed(p: &Place) -> bool {
        match &p.x {
            PlaceX::Index(_, _, ArrayKind::Slice, _) => true,
            PlaceX::Index(base, _, _, _) | PlaceX::Field(_, base) => indexed(base),
            _ => false,
        }
    }
    let index = matches!(&e.x, ExprX::ReadPlace(p, _) if indexed(p))
        || matches!(e.x, ExprX::Binary(BinaryOp::Index(ArrayKind::Slice, _), ..));
    if index {
        vec![crate::def::fn_slice_len(), crate::def::fn_slice_index()]
    } else if matches!(e.x, ExprX::Unary(UnaryOp::Length(ArrayKind::Slice), _))
        || matches!(&e.x, ExprX::Call { target: CallTarget::Fun(_, name, ..), args, .. }
            if functions.get(name).and_then(models::array_model) == Some(models::ArrayModel::Len)
                && args.len()==1 && models::array_call(&args[0], functions).ok().flatten() != Some(models::ArrayModel::AsSlice))
    {
        vec![crate::def::fn_slice_len()]
    } else {
        vec![]
    }
}

#[cfg(test)]
pub(super) mod tests;
