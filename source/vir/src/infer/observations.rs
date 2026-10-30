//! Candidate-only abstraction through authored projection observations. Every
//! replacement is checked against the original executable body by Verus.
use super::*;

fn base(t: &Typ) -> &Typ {
    match &**t {
        TypX::Decorate(TypDecoration::Ref, None, t) => base(t),
        _ => t,
    }
}
fn read(e: &Expr) -> Expr {
    fn place(p: &Place) -> Expr {
        match &p.x {
            PlaceX::Local(n) => SpannedTyped::new(&p.span, &p.typ, ExprX::Var(n.clone())),
            PlaceX::Temporary(e) => read(e),
            PlaceX::Field(op, parent) => SpannedTyped::new(
                &p.span,
                &p.typ,
                ExprX::UnaryOpr(UnaryOpr::Field(op.clone()), place(parent)),
            ),
            _ => SpannedTyped::new(
                &p.span,
                &p.typ,
                ExprX::ReadPlace(
                    p.clone(),
                    UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: u64::MAX },
                ),
            ),
        }
    }
    match &e.x {
        ExprX::ReadPlace(p, _) => place(p),
        ExprX::UnaryOpr(UnaryOpr::Field(op), base) =>
            e.new_x(ExprX::UnaryOpr(UnaryOpr::Field(op.clone()), read(base))),
        ExprX::Block(ss, Some(t)) if ss.is_empty() => read(t),
        _ => e.clone(),
    }
}
pub(super) fn logical_value(e: &Expr) -> Expr {
    read(e)
}
fn target(e: &Expr) -> Option<(&Fun, &Expr)> {
    let ExprX::Call { target: CallTarget::Fun(kind, n, ..), args, post_args: None, body: None } =
        &e.x
    else {
        return None;
    };
    if args.len() != 1 {
        return None;
    }
    Some((
        if let CallTargetKind::DynamicResolved { resolved, .. } = kind { resolved } else { n },
        &args[0],
    ))
}

#[derive(Clone)]
enum Step {
    Field(FieldOpr),
    Call(Fun),
}
struct Projection {
    spec: Function,
    path: Vec<Step>,
    output: Vec<(FieldOpr, Typ)>,
}

fn path(
    e: &Expr,
    root: &VarIdent,
    all: &HashMap<Fun, Function>,
    depth: usize,
) -> Option<Vec<Step>> {
    if depth == 0 {
        return None;
    }
    let e = read(e);
    match &e.x {
        ExprX::Var(n) if n == root => Some(vec![]),
        ExprX::UnaryOpr(UnaryOpr::Field(op), a)
            if op.check == VariantCheck::None && !op.get_variant =>
        {
            let mut p = path(a, root, all, depth - 1)?;
            p.push(Step::Field(op.clone()));
            Some(p)
        }
        _ => {
            let (n, a) = target(&e)?;
            let f = all.get(n)?;
            // Expand only ordinary unary definitions whose bodies are themselves
            // projections. Canonical/certified sequence views remain a call step.
            if f.x.attrs.infer_sequence_view
                || collections::model(f) == Some(collections::Model::View)
            {
                let mut p = path(a, root, all, depth - 1)?;
                p.push(Step::Call(n.clone()));
                Some(p)
            } else if is_pure_observation(f) && f.x.params.len() == 1 {
                let mut p = path(a, root, all, depth - 1)?;
                p.extend(path(f.x.body.as_ref()?, &f.x.params[0].x.name, all, depth - 1)?);
                Some(p)
            } else {
                None
            }
        }
    }
}
fn projections(all: &HashMap<Fun, Function>) -> Vec<Projection> {
    fn collect(
        f: &Function,
        e: &Expr,
        output: Vec<(FieldOpr, Typ)>,
        all: &HashMap<Fun, Function>,
        out: &mut Vec<Projection>,
        depth: usize,
    ) {
        if depth == 0 || out.len() >= 128 {
            return;
        }
        let e = read(e);
        if let ExprX::Ctor(dt @ Dt::Tuple(_), variant, fields, None) = &e.x {
            for field in fields.iter() {
                let mut output = output.clone();
                output.push((
                    FieldOpr {
                        datatype: dt.clone(),
                        variant: variant.clone(),
                        field: field.name.clone(),
                        get_variant: false,
                        check: VariantCheck::None,
                    },
                    field.a.typ.clone(),
                ));
                collect(f, &field.a, output, all, out, depth - 1);
            }
        } else if let Some(path) = path(&e, &f.x.params[0].x.name, all, 8).filter(|p| !p.is_empty())
        {
            out.push(Projection { spec: f.clone(), path, output });
        }
    }
    let mut fs: Vec<_> = all
        .values()
        .filter(|f| {
            f.x.name.path.krate != CrateId::Vstd
                && is_pure_observation(f)
                && !owned::borrowed::is_value(f)
                && !f.x.attrs.is_external_body
                && f.x.params.len() == 1
                && f.x.visibility.is_public()
                && f.x.body.is_some()
                && matches!(&**base(&f.x.params[0].x.typ), TypX::Datatype(..))
                && (matches!(f.x.kind, FunctionKind::Static)
                    || matches!(&f.x.kind,FunctionKind::TraitMethodImpl{method,..}
                if models::path_is(&method.path,CrateId::Vstd,&["view","View","view"])))
        })
        .collect();
    fs.sort_by_key(|f| f.x.name.clone());
    let mut out = vec![];
    for f in fs {
        collect(f, f.x.body.as_ref().unwrap(), vec![], all, &mut out, 4);
    }
    // Prefer the most enclosing abstraction, e.g. packet@.round over a
    // coordinate getter that would leave packet's private coord field exposed.
    out.sort_by_key(|p| std::cmp::Reverse(p.path.len()));
    out
}
fn projection_call(e: &Expr, all: &HashMap<Fun, Function>, borrowed_only: bool) -> Option<Expr> {
    let (name, arg) = target(e)?;
    let f = all.get(name).filter(|f| {
        f.x.params.len() == 1
            && is_pure_observation(f)
            && if borrowed_only {
                owned::borrowed::is_value(f)
            } else {
                f.x.name.path.krate != CrateId::Vstd
                    && f.x
                        .body
                        .as_ref()
                        .is_some_and(|body| path(body, &f.x.params[0].x.name, all, 8).is_some())
            }
    })?;
    let ExprX::Call { target: CallTarget::Fun(CallTargetKind::Static, _, types, ..), .. } = &e.x
    else {
        return None;
    };
    if types.len() != f.x.typ_params.len() {
        return None;
    }
    let ts: HashMap<_, _> = f.x.typ_params.iter().cloned().zip(types.iter().cloned()).collect();
    crate::ast_visitor::map_expr_visitor_env(
        &read(f.x.body.as_ref()?),
        &mut crate::ast_visitor::VisitorScopeMap::new(),
        &mut (),
        &|_, _, e| {
            Ok(if matches!(&e.x,ExprX::Var(n) if n==&f.x.params[0].x.name) {
                read(arg)
            } else {
                e.clone()
            })
        },
        &|_, _, s| Ok(vec![s.clone()]),
        &|_, t| {
            Ok(match &**t {
                TypX::TypParam(n) => ts.get(n).cloned().unwrap_or_else(|| t.clone()),
                _ => t.clone(),
            })
        },
        &|_, _, p| {
            Ok(if matches!(&p.x,PlaceX::Local(v) if v==&f.x.params[0].x.name) {
                p.new_x(PlaceX::Temporary(read(arg)))
            } else {
                p.clone()
            })
        },
        &|_, _, p| Ok(p.clone()),
    )
    .ok()
}
pub(super) fn expand_projections(e: &Expr, all: &HashMap<Fun, Function>) -> Result<Expr, VirErr> {
    let mut out = e.clone();
    let mut budget = FORMULA_BUDGET;
    for _ in 0..8 {
        out = crate::ast_visitor::map_expr_visitor_env(
            &out,
            &mut crate::ast_visitor::VisitorScopeMap::new(),
            &mut budget,
            &|budget, _, e| {
                *budget = budget.checked_sub(1).ok_or_else(|| {
                    unsupported(&e.span, "projection definition expansion budget exceeded")
                })?;
                Ok(projection_call(e, all, false).unwrap_or_else(|| e.clone()))
            },
            &|_, _, s| Ok(vec![s.clone()]),
            &|_, t| Ok(t.clone()),
            &|_, _, p| Ok(p.clone()),
            &|_, _, p| Ok(p.clone()),
        )?;
    }
    Ok(out)
}
// Normalize a tuple component only for matching an enclosing authored
// observation. For example, inner@.1 can match outer's scalar View field.
// The expanded private path is never retained when no public observation fits.
fn component_projection(e: &Expr, all: &HashMap<Fun, Function>) -> Option<Expr> {
    let mut receiver = read(e);
    let mut steps = Vec::new();
    while let ExprX::UnaryOpr(UnaryOpr::Field(op), base) = &receiver.x {
        if !matches!(op.datatype, Dt::Tuple(_)) || steps.len() == 8 {
            return None;
        }
        steps.push(op.clone());
        receiver = read(base);
    }
    if steps.is_empty() {
        return None;
    }
    let (name, arg) = target(&receiver)?;
    let f = all.get(name).filter(|f| {
        is_pure_observation(f)
            && f.x.visibility.is_public()
            && !f.x.attrs.is_external_body
            && f.x.proxy.is_none()
            && f.x.params.len() == 1
            && f.x.body.is_some()
            && f.x.name.path.krate != CrateId::Vstd
    })?;
    let ExprX::Call { target: CallTarget::Fun(kind, _, types, ..), .. } = &receiver.x else {
        return None;
    };
    let types = match kind {
        CallTargetKind::DynamicResolved { typs, .. } => typs,
        _ => types,
    };
    if types.len() != f.x.typ_params.len() {
        return None;
    }
    let mut component = read(f.x.body.as_ref()?);
    for step in steps.iter().rev() {
        let ExprX::Ctor(dt, variant, fields, None) = &component.x else {
            return None;
        };
        if dt != &step.datatype || variant != &step.variant {
            return None;
        }
        component = read(&fields.iter().find(|field| field.name == step.field)?.a);
    }
    path(&component, &f.x.params[0].x.name, all, 8)?;
    let ts: HashMap<_, _> = f.x.typ_params.iter().cloned().zip(types.iter().cloned()).collect();
    let out = crate::ast_visitor::map_expr_visitor_env(
        &component,
        &mut crate::ast_visitor::VisitorScopeMap::new(),
        &mut (),
        &|_, _, e| {
            Ok(if matches!(&e.x,ExprX::Var(v) if v==&f.x.params[0].x.name) {
                read(arg)
            } else {
                e.clone()
            })
        },
        &|_, _, s| Ok(vec![s.clone()]),
        &|_, t| {
            Ok(if let TypX::TypParam(n) = &**t {
                ts.get(n).cloned().unwrap_or_else(|| t.clone())
            } else {
                t.clone()
            })
        },
        &|_, _, p| {
            Ok(if matches!(&p.x,PlaceX::Local(v) if v==&f.x.params[0].x.name) {
                p.new_x(PlaceX::Temporary(read(arg)))
            } else {
                p.clone()
            })
        },
        &|_, _, p| Ok(p.clone()),
    )
    .ok()?;
    crate::ast_util::types_equal(&out.typ, &e.typ).then_some(out)
}

fn replace(
    e: &Expr,
    p: &Projection,
    function: &Function,
    scope: &crate::ast_visitor::VisitorScopeMap,
    all: &HashMap<Fun, Function>,
    datatypes: &[Datatype],
) -> Option<Expr> {
    // A generated borrow definition may hide a simple physical projection.
    // Expand it only for matching an authored observation. If none matches,
    // retain the original checked definition and its abstraction boundary.
    let mut receiver = component_projection(e, all)
        .or_else(|| projection_call(e, all, true))
        .unwrap_or_else(|| read(e));
    for step in p.path.iter().rev() {
        receiver = match (step, &receiver.x) {
            (Step::Field(a), ExprX::UnaryOpr(UnaryOpr::Field(b), x))
                if records::shape::same_path(&[a.clone()], &[b.clone()]) =>
            {
                read(x)
            }
            (Step::Call(n), _) => {
                let (actual, arg) = target(&receiver)?;
                if n != actual {
                    return None;
                }
                read(arg)
            }
            _ => return None,
        };
    }
    fn input(e: &Expr, params: &Params, scope: Option<&crate::ast_visitor::VisitorScopeMap>) -> bool {
        let e = read(e);
        match &e.x {
            ExprX::Var(n) => params.iter().any(|p| p.x.name == *n)
                || scope.is_some_and(|scope| scope.get(n).is_some()),
            ExprX::UnaryOpr(UnaryOpr::Field(_), a) => input(a, params, scope),
            _ => false,
        }
    }
    let TypX::Datatype(dt, args, _) = &**base(&receiver.typ) else { return None };
    if !input(&receiver, &function.x.params, None) {
        // Lift bound predicates when publishing requires an abstraction. Keep
        // the original lambda vocabulary where the representation is legal.
        if !input(&receiver, &function.x.params, Some(scope)) {
            return None;
        }
        let datatype = datatypes.iter().find(|d| &d.x.name == dt)?;
        match &datatype.x.transparency {
            DatatypeTransparency::WhenVisible(vis)
                if !function.x.visibility.at_least_as_restrictive_as(vis) => {}
            _ => return None,
        }
    }
    let TypX::Datatype(expected, params, _) = &**base(&p.spec.x.params[0].x.typ) else {
        return None;
    };
    if dt != expected
        || params.len() != args.len()
        || params.len() != p.spec.x.typ_params.len()
        || !params
            .iter()
            .zip(p.spec.x.typ_params.iter())
            .all(|(t, n)| matches!(&**t,TypX::TypParam(p) if p==n))
    {
        return None;
    }
    let ts: HashMap<_, _> = p.spec.x.typ_params.iter().cloned().zip(args.iter().cloned()).collect();
    let arg = records::coerce(
        &e.span,
        records::atom(receiver.clone()).ok()?,
        &crate::sst_util::subst_typ(&ts, &p.spec.x.params[0].x.typ),
    )
    .ok()?
    .expr;
    let attrs = CallTargetAttrs {
        autospec: AutospecUsage::Final,
        const_var: false,
        assume_external_allowed: false,
    };
    let target = match &p.spec.x.kind {
        FunctionKind::Static => CallTarget::Fun(
            CallTargetKind::Static,
            p.spec.x.name.clone(),
            args.clone(),
            Arc::new(vec![]),
            attrs,
        ),
        FunctionKind::TraitMethodImpl { method, impl_path, trait_typ_args, .. } => CallTarget::Fun(
            CallTargetKind::DynamicResolved {
                resolved: p.spec.x.name.clone(),
                typs: args.clone(),
                impl_paths: Arc::new(vec![]),
                is_trait_default: false,
            },
            method.clone(),
            Arc::new(trait_typ_args.iter().map(|t| crate::sst_util::subst_typ(&ts, t)).collect()),
            Arc::new(vec![ImplPath::TraitImplPath(impl_path.clone())]),
            attrs,
        ),
        _ => return None,
    };
    let mut out = SpannedTyped::new(
        &e.span,
        &crate::sst_util::subst_typ(&ts, &p.spec.x.inner_ret.x.typ),
        ExprX::Call { target, args: Arc::new(vec![arg]), post_args: None, body: None },
    );
    for (op, t) in &p.output {
        out = SpannedTyped::new(
            &e.span,
            &crate::sst_util::subst_typ(&ts, t),
            ExprX::UnaryOpr(UnaryOpr::Field(op.clone()), out),
        );
    }
    crate::ast_util::types_equal(&out.typ, &e.typ).then_some(out)
}

pub(super) fn lift_value(
    e: &Expr,
    all: &HashMap<Fun, Function>,
    function: &Function,
    datatypes: &[Datatype],
) -> Result<Expr, VirErr> {
    let ps = projections(all);
    crate::ast_visitor::map_expr_visitor_env(
        e,
        &mut crate::ast_visitor::VisitorScopeMap::new(),
        &mut FORMULA_BUDGET.clone(),
        &|budget, scope, e| {
            *budget = budget
                .checked_sub(1)
                .ok_or_else(|| unsupported(&e.span, "observation lifting budget exceeded"))?;
            Ok(ps.iter().find_map(|p| replace(e, p, function, scope, all, datatypes)).unwrap_or_else(|| e.clone()))
        },
        &|_, _, s| Ok(vec![s.clone()]),
        &|_, t| Ok(t.clone()),
        &|_, _, p| Ok(p.clone()),
        &|_, _, p| Ok(p.clone()),
    )
}

pub(super) fn lift_contract(
    f: &Function,
    c: Contract,
    all: &HashMap<Fun, Function>,
    datatypes: &[Datatype],
) -> Result<Contract, VirErr> {
    fn post(e: &Expr, all: &HashMap<Fun, Function>, function: &Function, datatypes: &[Datatype]) -> Result<Expr, VirErr> {
        Ok(match &e.x {
            ExprX::Binary(BinaryOp::Eq(Mode::Spec), a, b) => e.new_x(ExprX::Binary(
                BinaryOp::Eq(Mode::Spec),
                a.clone(),
                lift_value(b, all, function, datatypes)?,
            )),
            ExprX::Logical(op, a, b) => e.new_x(ExprX::Logical(
                *op,
                if *op == LogicalOp::Implies {
                    lift_value(a, all, function, datatypes)?
                } else {
                    post(a, all, function, datatypes)?
                },
                post(b, all, function, datatypes)?,
            )),
            _ => e.clone(),
        })
    }
    // Preserve the independently selected output-observation schema. This
    // profile only changes the input expressions used to explain those outputs.
    let c = Contract::new(lift_value(&c.pre, all, f, datatypes)?, post(&c.post, all, f, datatypes)?)?;
    // Module pruning removes hidden representations. Diagnose an unavailable
    // abstraction before a proposed formula reaches that boundary.
    let visible = |op: &FieldOpr| {
        datatypes
            .iter()
            .find(|d| d.x.name == op.datatype)
            .and_then(|d| d.x.variants.iter().find(|v| v.name == op.variant))
            .and_then(|v| v.fields.iter().find(|p| p.name == op.field))
            .is_none_or(|p| crate::ast_util::is_visible_to_opt(&p.a.2, &f.x.owning_module))
    };
    for root in [&c.pre, &c.post] {
        crate::ast_visitor::expr_visitor_check(root, &mut |_, e| {
            let valid = match &e.x {
                ExprX::UnaryOpr(UnaryOpr::Field(op), _) => visible(op),
                ExprX::ReadPlace(p, _) => {
                    let mut p = p;
                    let mut valid = true;
                    while let PlaceX::Field(op, parent) = &p.x {
                        valid &= visible(op);
                        p = parent;
                    }
                    valid
                }
                _ => true,
            };
            if valid {
                Ok(())
            } else {
                Err(unsupported(
                    &e.span,
                    "private record formulas require an authored projection observation",
                ))
            }
        })?;
    }
    Ok(c)
}
