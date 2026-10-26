//! Closed recognition of Option/Result propagation. Internal ControlFlow and
//! Infallible values are inspected here, never admitted as general source or
//! summary values. The executable lowering remains unchanged.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Carrier {
    Option,
    Result,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Model {
    Branch(Carrier),
    Residual(Carrier),
}

fn path_is(path: &Path, krate: CrateId, parts: &[&str]) -> bool {
    path.krate == krate
        && path.segments.len() == parts.len()
        && path.segments.iter().zip(parts).all(|(a, b)| a.as_str() == *b)
}

fn datatype(parts: &[&str], args: Vec<Typ>) -> Typ {
    Arc::new(TypX::Datatype(
        Dt::Path(Arc::new(PathX {
            krate: CrateId::Core,
            segments: Arc::new(parts.iter().map(|s| Arc::new((*s).into())).collect()),
        })),
        Arc::new(args),
        Arc::new(vec![]),
    ))
}

fn carrier(kind: Carrier, args: Vec<Typ>) -> Typ {
    datatype(
        match kind {
            Carrier::Option => &["option", "Option"],
            Carrier::Result => &["result", "Result"],
        },
        args,
    )
}

fn residual(kind: Carrier, error: Option<Typ>) -> Typ {
    let mut args = vec![datatype(&["convert", "Infallible"], vec![])];
    args.extend(error);
    carrier(kind, args)
}

fn control_flow(residual: Typ, success: Typ) -> Typ {
    datatype(&["ops", "control_flow", "ControlFlow"], vec![residual, success])
}

/// Canonical declarations, including their complete generic signature/bounds.
pub(super) fn model(function: &Function) -> Option<Model> {
    let f = &function.x;
    let p = &f.name.path;
    let proxy = &f.proxy.as_ref()?.x;
    if f.mode != Mode::Exec
        || !matches!(f.item_kind, ItemKind::Function)
        || f.body.is_some()
        || !f.require.is_empty()
        || f.params.len() != 1
        || f.params[0].x.mode != Mode::Exec
        || f.params[0].x.user_mut
        || f.inner_ret.x.mode != Mode::Exec
        || f.attrs.is_unsafe
        || f.attrs.is_async
        || f.attrs.infer
        || f.attrs.is_external_body
        || f.attrs.autospec.is_some()
        || p.krate != CrateId::Core
        || p.segments.len() != 3
        || !p.segments[1].starts_with("impl")
        || proxy.krate != CrateId::Vstd
        || proxy.segments.len() != 3
        || proxy.segments[0].as_str() != "std_specs"
        || proxy.segments[1].as_str() != "control_flow"
        || !proxy.segments[2].starts_with("_verus_external_fn_specification_")
    {
        return None;
    }
    let kind = match p.segments[0].as_str() {
        "option" => Carrier::Option,
        "result" => Carrier::Result,
        _ => return None,
    };
    let operation = match p.segments[2].as_str() {
        "branch" if proxy.segments[2].ends_with("_branch") => Model::Branch(kind),
        "from_residual" if proxy.segments[2].ends_with("_from__residual") => Model::Residual(kind),
        _ => return None,
    };
    let count = match operation {
        Model::Branch(Carrier::Result) => 2,
        Model::Residual(Carrier::Result) => 3,
        _ => 1,
    };
    if f.typ_params.len() != count {
        return None;
    }
    let ts: Vec<_> = f.typ_params.iter().map(|t| Arc::new(TypX::TypParam(t.clone()))).collect();
    let mut sized = vec![0; count];
    let mut from = 0;
    for bound in f.typ_bounds.iter() {
        match &**bound {
            GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), args) if args.len() == 1 => {
                let i = ts.iter().position(|t| crate::ast_util::types_equal(t, &args[0]))?;
                sized[i] += 1;
            }
            GenericBoundX::Trait(TraitId::Path(path), args)
                if operation == Model::Residual(Carrier::Result)
                    && path_is(path, CrateId::Core, &["convert", "From"])
                    && args.len() == 2
                    && crate::ast_util::types_equal(&args[0], &ts[2])
                    && crate::ast_util::types_equal(&args[1], &ts[1]) =>
            {
                from += 1;
            }
            _ => return None,
        }
    }
    if sized.iter().any(|n| *n != 1)
        || from != usize::from(operation == Model::Residual(Carrier::Result))
    {
        return None;
    }
    let (input, output) = match operation {
        Model::Branch(kind) => (
            carrier(kind, ts.clone()),
            control_flow(residual(kind, ts.get(1).cloned()), ts[0].clone()),
        ),
        Model::Residual(Carrier::Option) => (residual(kind, None), carrier(kind, ts.clone())),
        Model::Residual(Carrier::Result) => {
            (residual(kind, Some(ts[1].clone())), carrier(kind, vec![ts[0].clone(), ts[2].clone()]))
        }
    };
    if !crate::ast_util::types_equal(&input, &f.params[0].x.typ)
        || !crate::ast_util::types_equal(&output, &f.inner_ret.x.typ)
        || !crate::ast_util::types_equal(&output, &f.outer_ret.x.typ)
    {
        return None;
    }
    match &f.kind {
        FunctionKind::Static => {} // After ordinary trait demotion.
        FunctionKind::ForeignTraitMethodImpl { method, impl_path, trait_path, trait_typ_args } => {
            let (tr, op, args) = match operation {
                Model::Branch(_) => ("Try", "branch", vec![input.clone()]),
                Model::Residual(_) => {
                    ("FromResidual", "from_residual", vec![output.clone(), input.clone()])
                }
            };
            if !path_is(trait_path, CrateId::Core, &["ops", "try_trait", tr])
                || !path_is(&method.path, CrateId::Core, &["ops", "try_trait", tr, op])
                || impl_path.krate != CrateId::Core
                || impl_path.segments.as_ref() != &p.segments[..2]
                || trait_typ_args.len() != args.len()
                || !trait_typ_args
                    .iter()
                    .zip(&args)
                    .all(|(a, b)| crate::ast_util::types_equal(a, b))
            {
                return None;
            }
        }
        _ => return None,
    }
    match operation {
        // The existing generic Result model permits arbitrary From conversions
        // and has no no_unwind clause. We do not strengthen it. Recognition of
        // a concrete use below restricts it to Rust's blanket identity conversion.
        Model::Residual(Carrier::Result) if f.unwind_spec.is_none() => {}
        Model::Residual(Carrier::Result) => return None,
        _ if matches!(f.unwind_spec, Some(UnwindSpec::NoUnwind)) => {}
        _ => return None,
    }
    Some(operation)
}

pub(super) struct Propagation {
    pub operand: Expr,
    pub branch: Expr,
    pub residual: Expr,
    kind: Carrier,
    return_type: Typ,
}

fn static_call(e: &Expr) -> Option<(&Fun, &Typs, &ImplPaths, &Exprs)> {
    let ExprX::Call {
        target: CallTarget::Fun(CallTargetKind::Static, name, typs, impls, attrs),
        args,
        post_args: None,
        body: None,
    } = &e.x
    else {
        return None;
    };
    if attrs.const_var
        || attrs.assume_external_allowed
        || !matches!(attrs.autospec, AutospecUsage::Final)
    {
        return None;
    }
    Some((name, typs, impls, args))
}

fn empty_blocks(mut e: &Expr) -> Option<&Expr> {
    for _ in 0..8 {
        match &e.x {
            ExprX::Block(stmts, Some(tail))
                if stmts.is_empty() && crate::ast_util::types_equal(&e.typ, &tail.typ) =>
            {
                e = tail
            }
            _ => return Some(e),
        }
    }
    None
}

fn binding(pattern: &Pattern, cf: &Typ, variant: &str, payload: &Typ) -> Option<VarIdent> {
    let TypX::Datatype(dt, ..) = &**cf else { return None };
    let PatternX::Constructor(actual, v, fields) = &pattern.x else { return None };
    if actual != dt
        || v.as_str() != variant
        || fields.len() != 1
        || fields[0].name != crate::def::positional_field_ident(0)
        || !crate::ast_util::types_equal(&pattern.typ, cf)
        || !crate::ast_util::types_equal(&fields[0].a.typ, payload)
    {
        return None;
    }
    let PatternX::Var(b) = &fields[0].a.x else { return None };
    if !matches!(b.by_ref, ByRef::No)
        || b.user_mut == Some(true)
        || !crate::ast_util::types_equal(&b.typ, payload)
    {
        return None;
    }
    Some(b.name.clone())
}

fn local(e: &Expr, name: &VarIdent, typ: &Typ) -> bool {
    let Some(e) = empty_blocks(e) else { return false };
    if !crate::ast_util::types_equal(&e.typ, typ) {
        return false;
    }
    match &e.x {
        ExprX::Var(v) => v == name,
        ExprX::ReadPlace(p, _) => {
            matches!(&p.x, PlaceX::Local(v) if v == name)
                && crate::ast_util::types_equal(&p.typ, typ)
        }
        _ => false,
    }
}

fn is_never(t: &Typ) -> bool {
    matches!(&**t, TypX::Decorate(TypDecoration::Never, None, inner) if enums::unit(inner))
}

fn returned_call(body: &Expr, success: &Typ) -> Option<Expr> {
    let mut e = empty_blocks(body)?;
    if let ExprX::NeverToAny(inner) = &e.x {
        if !crate::ast_util::types_equal(&e.typ, success) || !is_never(&inner.typ) {
            return None;
        }
        e = empty_blocks(inner)?;
    }
    if let ExprX::ReadPlace(p, _) = &e.x {
        let PlaceX::Temporary(inner) = &p.x else { return None };
        if !is_never(&p.typ) || !crate::ast_util::types_equal(&p.typ, &inner.typ) {
            return None;
        }
        e = empty_blocks(inner)?;
    }
    if !is_never(&e.typ) {
        return None;
    }
    let ExprX::Return(Some(call)) = &e.x else { return None };
    Some(empty_blocks(call)?.clone())
}

/// Validate the complete lowered operation, including both arms and bindings.
/// No proof statements, extra calls, guards or assignments may hide in it.
pub(super) fn recognize(
    e: &Expr,
    functions: &HashMap<Fun, Function>,
    result: &Typ,
) -> Result<Option<Propagation>, VirErr> {
    let ExprX::Match(place, arms, false) = &e.x else { return Ok(None) };
    let PlaceX::Temporary(branch) = &place.x else { return Ok(None) };
    let Some((name, typs, impls, args)) = static_call(branch) else { return Ok(None) };
    let Some(Model::Branch(kind)) = functions.get(name).and_then(model) else { return Ok(None) };
    let invalid = || unsupported(&e.span, "unsupported Option/Result ? lowering");
    if arms.len() != 2
        || !impls.is_empty()
        || args.len() != 1
        || typs.len() != if kind == Carrier::Option { 1 } else { 2 }
        || !enums::supported(&args[0].typ)
        || !enums::supported(result)
        || !crate::ast_util::types_equal(&args[0].typ, &carrier(kind, typs.as_ref().clone()))
        || !crate::ast_util::types_equal(&e.typ, &typs[0])
    {
        return Err(invalid());
    }
    let residual_type = residual(kind, typs.get(1).cloned());
    let cf = control_flow(residual_type.clone(), typs[0].clone());
    if !crate::ast_util::types_equal(&place.typ, &cf)
        || !crate::ast_util::types_equal(&branch.typ, &cf)
        || arms.iter().any(|a| !matches!(a.x.guard.x, ExprX::Const(Constant::Bool(true))))
    {
        return Err(invalid());
    }
    let residual_binding =
        binding(&arms[0].x.pattern, &cf, "Break", &residual_type).ok_or_else(invalid)?;
    let value_binding =
        binding(&arms[1].x.pattern, &cf, "Continue", &typs[0]).ok_or_else(invalid)?;
    if residual_binding == value_binding || !local(&arms[1].x.body, &value_binding, &typs[0]) {
        return Err(invalid());
    }
    let conversion = returned_call(&arms[0].x.body, &typs[0]).ok_or_else(invalid)?;
    let (name, ctyps, cimpls, cargs) = static_call(&conversion).ok_or_else(invalid)?;
    if functions.get(name).and_then(model) != Some(Model::Residual(kind))
        || cargs.len() != 1
        || !local(&cargs[0], &residual_binding, &residual_type)
        || !crate::ast_util::types_equal(&conversion.typ, result)
    {
        return Err(invalid());
    }
    match kind {
        Carrier::Option
            if ctyps.len() == 1
                && cimpls.is_empty()
                && crate::ast_util::types_equal(result, &carrier(kind, ctyps.as_ref().clone())) => {
        }
        Carrier::Result if ctyps.len() == 3 => {
            if !crate::ast_util::types_equal(&ctyps[1], &typs[1])
                || !crate::ast_util::types_equal(&ctyps[1], &ctyps[2])
            {
                return Err(unsupported(
                    &e.span,
                    "? currently requires identical Result error types",
                ));
            }
            if cimpls.len() != 1
                || !matches!(&cimpls[0], ImplPath::TraitImplPath(p)
                if p.krate == CrateId::Core && p.segments.len() == 2 && p.segments[0].as_str() == "convert" && p.segments[1].starts_with("impl"))
                || !crate::ast_util::types_equal(
                    result,
                    &carrier(kind, vec![ctyps[0].clone(), ctyps[2].clone()]),
                )
            {
                return Err(invalid());
            }
        }
        _ => return Err(invalid()),
    }
    Ok(Some(Propagation {
        operand: args[0].clone(),
        branch: branch.clone(),
        residual: conversion,
        kind,
        return_type: result.clone(),
    }))
}

/// Enumerate genuine source calls and the two validated standard helpers of
/// each propagation. Generated scaffolding is never a general generic-call gate.
pub(super) fn visit_calls(
    body: &Expr,
    result: &Typ,
    functions: &HashMap<Fun, Function>,
    callback: &mut impl FnMut(&Expr, bool) -> Result<(), VirErr>,
) -> Result<(), VirErr> {
    let walked = crate::ast_visitor::expr_visitor_dfs(
        body,
        &mut crate::ast_visitor::VisitorScopeMap::new(),
        &mut |_, e| {
            let checked = (|| {
                if let Some(p) = recognize(e, functions, result)? {
                    callback(&p.branch, true)?;
                    callback(&p.residual, true)?;
                    visit_calls(&p.operand, result, functions, callback)?;
                    return Ok(true);
                }
                if matches!(e.x, ExprX::Call { .. }) {
                    callback(e, false)?;
                }
                Ok::<_, VirErr>(false)
            })();
            match checked {
                Ok(true) => crate::ast_visitor::VisitorControlFlow::Return,
                Ok(false) => crate::ast_visitor::VisitorControlFlow::Recurse,
                Err(err) => crate::ast_visitor::VisitorControlFlow::Stop(err),
            }
        },
    );
    match walked {
        crate::ast_visitor::VisitorControlFlow::Stop(err) => Err(err),
        _ => Ok(()),
    }
}

impl Engine<'_> {
    pub(super) fn propagate(&mut self, e: &Expr, p: Propagation) -> Result<Summary, VirErr> {
        let arg = self.expr(&p.operand)?;
        let input = value(&arg, &e.span)?;
        let success = Arc::new(
            match p.kind {
                Carrier::Option => "Some",
                Carrier::Result => "Ok",
            }
            .into(),
        );
        let test = enums::is_variant(&e.span, input.clone(), &success)?;
        let active = self
            .exits
            .as_ref()
            .ok_or_else(|| unsupported(&e.span, "? requires an enclosing function exit"))?
            .active
            .clone();
        let failed =
            logical(&e.span, LogicalOp::And, active.clone(), exits::not(&e.span, test.clone())?)?;
        let (variant, payload) = match p.kind {
            Carrier::Option => ("None", None),
            Carrier::Result => {
                ("Err", Some(enums::field(&e.span, input.clone(), &Arc::new("Err".into()))?))
            }
        };
        let output = enums::ctor(&e.span, &p.return_type, &Arc::new(variant.into()), payload)?;
        self.capture_exit(&e.span, failed, output)?;
        self.exits.as_mut().unwrap().active = logical(&e.span, LogicalOp::And, active, test)?;
        Ok(Summary { safe: arg.safe, value: Some(enums::field(&e.span, input, &success)?) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{Value, evaluate, function, input, span};

    fn path(krate: CrateId, parts: &[&str]) -> Path {
        Arc::new(PathX {
            krate,
            segments: Arc::new(parts.iter().map(|s| Arc::new((*s).into())).collect()),
        })
    }

    fn declaration(operation: Model) -> Function {
        let base = function("model", input("x"));
        let mut f = base.x.clone();
        let (kind, method, count) = match operation {
            Model::Branch(k) => (k, "branch", if k == Carrier::Option { 1 } else { 2 }),
            Model::Residual(k) => (k, "from_residual", if k == Carrier::Option { 1 } else { 3 }),
        };
        f.name = Arc::new(FunX {
            path: path(
                CrateId::Core,
                &[if kind == Carrier::Option { "option" } else { "result" }, "impl_test", method],
            ),
        });
        f.proxy = Some(crate::def::Spanned {
            span: span(),
            x: path(
                CrateId::Vstd,
                &[
                    "std_specs",
                    "control_flow",
                    if method == "branch" {
                        "_verus_external_fn_specification_test_branch"
                    } else {
                        "_verus_external_fn_specification_test_from__residual"
                    },
                ],
            ),
        });
        f.typ_params =
            Arc::new(["T", "E", "F"][..count].iter().map(|s| Arc::new((*s).into())).collect());
        let ts: Vec<_> = f.typ_params.iter().map(|t| Arc::new(TypX::TypParam(t.clone()))).collect();
        let mut bounds: Vec<_> = ts
            .iter()
            .map(|t| {
                Arc::new(GenericBoundX::Trait(
                    TraitId::Sizedness(Sizedness::Sized),
                    Arc::new(vec![t.clone()]),
                ))
            })
            .collect();
        if count == 3 {
            bounds.push(Arc::new(GenericBoundX::Trait(
                TraitId::Path(path(CrateId::Core, &["convert", "From"])),
                Arc::new(vec![ts[2].clone(), ts[1].clone()]),
            )));
        }
        f.typ_bounds = Arc::new(bounds);
        let (arg, ret) = match operation {
            Model::Branch(k) => (
                carrier(k, ts.clone()),
                control_flow(residual(k, ts.get(1).cloned()), ts[0].clone()),
            ),
            Model::Residual(Carrier::Option) => (residual(kind, None), carrier(kind, ts)),
            Model::Residual(Carrier::Result) => (
                residual(kind, Some(ts[1].clone())),
                carrier(kind, vec![ts[0].clone(), ts[2].clone()]),
            ),
        };
        let mut param = f.params[0].x.clone();
        param.typ = arg;
        f.params = Arc::new(vec![f.params[0].new_x(param)]);
        let mut output = f.inner_ret.x.clone();
        output.typ = ret;
        f.inner_ret = f.inner_ret.new_x(output);
        f.outer_ret = f.inner_ret.clone();
        f.body = None;
        f.attrs = Arc::new(FunctionAttrsX::default());
        f.unwind_spec = if count == 3 { None } else { Some(UnwindSpec::NoUnwind) };
        base.new_x(f)
    }

    fn call(f: &Function, typs: Vec<Typ>, impls: Vec<ImplPath>, arg: Expr, output: Typ) -> Expr {
        SpannedTyped::new(
            &span(),
            &output,
            ExprX::Call {
                target: CallTarget::Fun(
                    CallTargetKind::Static,
                    f.x.name.clone(),
                    Arc::new(typs),
                    Arc::new(impls),
                    CallTargetAttrs {
                        autospec: AutospecUsage::Final,
                        const_var: false,
                        assume_external_allowed: false,
                    },
                ),
                args: Arc::new(vec![arg]),
                post_args: None,
                body: None,
            },
        )
    }

    fn pattern(cf: &Typ, variant: &str, name: &str, payload: &Typ) -> Pattern {
        let name = VarIdent(Arc::new(name.into()), VarIdentDisambiguate::AirLocal);
        let binding = SpannedTyped::new(
            &span(),
            payload,
            PatternX::Var(PatternBinding {
                name,
                by_ref: ByRef::No,
                typ: payload.clone(),
                user_mut: Some(false),
                copy: true,
            }),
        );
        let TypX::Datatype(dt, ..) = &**cf else { unreachable!() };
        SpannedTyped::new(
            &span(),
            cf,
            PatternX::Constructor(
                dt.clone(),
                Arc::new(variant.into()),
                Arc::new(vec![Arc::new(air::ast::BinderX {
                    name: crate::def::positional_field_ident(0),
                    a: binding,
                })]),
            ),
        )
    }

    fn variable(pattern: &Pattern) -> Expr {
        let PatternX::Constructor(_, _, fields) = &pattern.x else { unreachable!() };
        let PatternX::Var(binding) = &fields[0].a.x else { unreachable!() };
        SpannedTyped::new(&span(), &binding.typ, ExprX::Var(binding.name.clone()))
    }

    fn lowered(kind: Carrier, operand: Expr, result: &Typ) -> (Expr, Vec<Function>) {
        let branch = declaration(Model::Branch(kind));
        let conversion = declaration(Model::Residual(kind));
        let scalar = input("x").typ.clone();
        let ts =
            if kind == Carrier::Option { vec![scalar.clone()] } else { vec![scalar.clone(); 2] };
        let res = residual(kind, if kind == Carrier::Option { None } else { Some(scalar.clone()) });
        let cf = control_flow(res.clone(), scalar.clone());
        let place = SpannedTyped::new(
            &span(),
            &cf,
            PlaceX::Temporary(call(&branch, ts, vec![], operand, cf.clone())),
        );
        let bad = pattern(&cf, "Break", "residual", &res);
        let good = pattern(&cf, "Continue", "value", &scalar);
        let ts =
            if kind == Carrier::Option { vec![scalar.clone()] } else { vec![scalar.clone(); 3] };
        let impls = if kind == Carrier::Option {
            vec![]
        } else {
            vec![ImplPath::TraitImplPath(path(CrateId::Core, &["convert", "impl_identity"]))]
        };
        let conversion_call = call(&conversion, ts, impls, variable(&bad), result.clone());
        let never =
            Arc::new(TypX::Decorate(TypDecoration::Never, None, crate::ast_util::unit_typ()));
        let ret = SpannedTyped::new(&span(), &never, ExprX::Return(Some(conversion_call)));
        let arm = |pattern, body| {
            crate::def::Spanned::new(
                span(),
                ArmX { pattern, guard: boolean(&span(), true).expr, body },
            )
        };
        (
            SpannedTyped::new(
                &span(),
                &scalar,
                ExprX::Match(
                    place,
                    Arc::new(vec![
                        arm(bad, SpannedTyped::new(&span(), &scalar, ExprX::NeverToAny(ret))),
                        arm(good.clone(), variable(&good)),
                    ]),
                    false,
                ),
            ),
            vec![branch, conversion],
        )
    }

    #[test]
    fn propagation_models_and_lowerings_are_closed() {
        for kind in [Carrier::Option, Carrier::Result] {
            for operation in [Model::Branch(kind), Model::Residual(kind)] {
                let original = declaration(operation);
                assert_eq!(model(&original), Some(operation));
                let mut f = original.x.clone();
                f.proxy = None;
                assert!(model(&original.new_x(f)).is_none());
                let mut f = original.x.clone();
                f.typ_bounds = Arc::new(vec![]);
                assert!(model(&original.new_x(f)).is_none());
                let mut f = original.x.clone();
                f.require = Arc::new(vec![boolean(&span(), true).expr]);
                assert!(model(&original.new_x(f)).is_none());
                let mut f = original.x.clone();
                f.params = Arc::new(vec![]);
                assert!(model(&original.new_x(f)).is_none());
            }
            let result = carrier(
                kind,
                if kind == Carrier::Option {
                    vec![input("x").typ.clone()]
                } else {
                    vec![input("x").typ.clone(); 2]
                },
            );
            let operand = enums::ctor(
                &span(),
                &result,
                &Arc::new(if kind == Carrier::Option { "Some" } else { "Ok" }.into()),
                Some(Term { expr: input("x"), size: 1 }),
            )
            .unwrap()
            .expr;
            let (original, declarations) = lowered(kind, operand, &result);
            let functions = declarations.iter().map(|f| (f.x.name.clone(), f.clone())).collect();
            assert!(recognize(&original, &functions, &result).unwrap().is_some());
            let ExprX::Match(place, arms, _) = &original.x else { unreachable!() };
            for mutation in 0..4 {
                let mut arms = (**arms).clone();
                let mut arm = ArmX {
                    pattern: arms[0].x.pattern.clone(),
                    guard: arms[0].x.guard.clone(),
                    body: arms[0].x.body.clone(),
                };
                match mutation {
                    0 => arm.guard = boolean(&span(), false).expr,
                    1 => arm.body = arms[1].x.body.clone(),
                    2 => {
                        arm.body = SpannedTyped::new(
                            &span(),
                            &arm.body.typ,
                            ExprX::Block(
                                Arc::new(vec![crate::def::Spanned::new(
                                    span(),
                                    StmtX::Expr(boolean(&span(), true).expr),
                                )]),
                                Some(arm.body.clone()),
                            ),
                        )
                    }
                    _ => arms.swap(0, 1),
                }
                if mutation != 3 {
                    arms[0] = arms[0].new_x(arm);
                }
                let changed = original.new_x(ExprX::Match(place.clone(), Arc::new(arms), false));
                assert!(recognize(&changed, &functions, &result).is_err(), "mutation {mutation}");
            }
        }
    }

    #[test]
    fn exhaustive_u8_propagation_preserves_domain_outputs_and_original_body() {
        for kind in [Carrier::Option, Carrier::Result] {
            let scalar = input("x").typ.clone();
            let result = carrier(
                kind,
                if kind == Carrier::Option {
                    vec![scalar.clone()]
                } else {
                    vec![scalar.clone(); 2]
                },
            );
            let success = Arc::new(if kind == Carrier::Option { "Some" } else { "Ok" }.into());
            let failure = Arc::new(if kind == Carrier::Option { "None" } else { "Err" }.into());
            let binary = |op, a: Expr, b: Expr| {
                SpannedTyped::new(
                    &span(),
                    &if matches!(op, BinaryOp::Arith(_)) { scalar.clone() } else { bool_typ() },
                    ExprX::Binary(op, a, b),
                )
            };
            let add = BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::U(8))));
            let sub = BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Error(IntRange::U(8))));
            let failed = enums::ctor(
                &span(),
                &result,
                &failure,
                if kind == Carrier::Option {
                    None
                } else {
                    Some(Term { expr: input("y"), size: 1 })
                },
            )
            .unwrap()
            .expr;
            let passed = enums::ctor(
                &span(),
                &result,
                &success,
                Some(Term { expr: binary(sub, input("x"), input("y")), size: 1 }),
            )
            .unwrap()
            .expr;
            let operand = SpannedTyped::new(
                &span(),
                &result,
                ExprX::If(
                    binary(BinaryOp::Inequality(InequalityOp::Lt), input("x"), input("y")),
                    failed,
                    Some(passed),
                ),
            );
            for prefix in [false, true] {
                let (propagate, declarations) = lowered(kind, operand.clone(), &result);
                let other = if prefix { binary(add, input("x"), input("y")) } else { input("x") };
                let sum = if prefix {
                    binary(add, other, propagate)
                } else {
                    binary(add, propagate, other)
                };
                let body =
                    enums::ctor(&span(), &result, &success, Some(Term { expr: sum, size: 1 }))
                        .unwrap()
                        .expr;
                let original = function("propagation", body);
                let mut f = original.x.clone();
                let mut ret = f.inner_ret.x.clone();
                ret.typ = result.clone();
                f.inner_ret = f.inner_ret.new_x(ret);
                f.outer_ret = f.inner_ret.clone();
                let original = original.new_x(f);
                let mut registry = registry::Registry::new(None);
                let fresh =
                    registry.infer_with_imports(&[original.clone()], &declarations).unwrap();
                let cached =
                    registry.infer_with_imports(&[original.clone()], &declarations).unwrap();
                assert_eq!((registry.hits, registry.misses), (1, 1));
                assert_eq!(format_contract(&fresh[0]), format_contract(&cached[0]));
                assert!(Arc::ptr_eq(
                    original.x.body.as_ref().unwrap(),
                    fresh[0].x.body.as_ref().unwrap()
                ));
                let ExprX::Binary(BinaryOp::Eq(_), _, output) = &fresh[0].x.ensure.0[0].x else {
                    panic!("contract")
                };
                for x in 0..=255u8 {
                    for y in 0..=255u8 {
                        let other = if prefix { x.checked_add(y) } else { Some(x) };
                        let expected = other.and_then(|other| match x.checked_sub(y) {
                            None => Some(Value::Enum(
                                failure.to_string(),
                                if kind == Carrier::Option {
                                    None
                                } else {
                                    Some(Box::new(Value::Int(y.into())))
                                },
                            )),
                            Some(value) => value.checked_add(other).map(|v| {
                                Value::Enum(
                                    success.to_string(),
                                    Some(Box::new(Value::Int(v.into()))),
                                )
                            }),
                        });
                        assert_eq!(
                            evaluate(&fresh[0].x.require[0], x, y),
                            Value::Bool(expected.is_some()),
                            "{kind:?} prefix={prefix} ({x},{y})"
                        );
                        if let Some(expected) = expected {
                            assert_eq!(
                                evaluate(output, x, y),
                                expected,
                                "{kind:?} prefix={prefix} ({x},{y})"
                            );
                        }
                    }
                }
            }
        }
    }
}
