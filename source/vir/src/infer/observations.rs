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
fn borrowed_projection(e: &Expr, all: &HashMap<Fun, Function>) -> Option<Expr> {
    let (name, arg) = target(e)?;
    let f = all.get(name).filter(|f| owned::borrowed::is_value(f) && f.x.params.len() == 1)?;
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
        &|_, _, p| Ok(p.clone()),
        &|_, _, p| Ok(p.clone()),
    )
    .ok()
}
fn replace(
    e: &Expr,
    p: &Projection,
    params: &Params,
    all: &HashMap<Fun, Function>,
) -> Option<Expr> {
    // A generated borrow definition may hide a simple physical projection.
    // Expand it only for matching an authored observation. If none matches,
    // retain the original checked definition and its abstraction boundary.
    let mut receiver = borrowed_projection(e, all).unwrap_or_else(|| read(e));
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
    fn input(e: &Expr, params: &Params) -> bool {
        let e = read(e);
        match &e.x {
            ExprX::Var(n) => params.iter().any(|p| p.x.name == *n),
            ExprX::UnaryOpr(UnaryOpr::Field(_), a) => input(a, params),
            _ => false,
        }
    }
    // Bound callback variables keep their original predicates. Changing a
    // lambda's vocabulary can require a separate extensional closure proof.
    if !input(&receiver, params) {
        return None;
    }
    let TypX::Datatype(dt, args, _) = &**base(&receiver.typ) else { return None };
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
    params: &Params,
) -> Result<Expr, VirErr> {
    let ps = projections(all);
    crate::ast_visitor::map_expr_visitor_env(
        e,
        &mut crate::ast_visitor::VisitorScopeMap::new(),
        &mut FORMULA_BUDGET.clone(),
        &|budget, _, e| {
            *budget = budget
                .checked_sub(1)
                .ok_or_else(|| unsupported(&e.span, "observation lifting budget exceeded"))?;
            Ok(ps.iter().find_map(|p| replace(e, p, params, all)).unwrap_or_else(|| e.clone()))
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
    fn post(e: &Expr, all: &HashMap<Fun, Function>, params: &Params) -> Result<Expr, VirErr> {
        Ok(match &e.x {
            ExprX::Binary(BinaryOp::Eq(Mode::Spec), a, b) => e.new_x(ExprX::Binary(
                BinaryOp::Eq(Mode::Spec),
                a.clone(),
                lift_value(b, all, params)?,
            )),
            ExprX::Logical(op, a, b) => e.new_x(ExprX::Logical(
                *op,
                if *op == LogicalOp::Implies {
                    lift_value(a, all, params)?
                } else {
                    post(a, all, params)?
                },
                post(b, all, params)?,
            )),
            _ => e.clone(),
        })
    }
    // Preserve the independently selected output-observation schema. This
    // profile only changes the input expressions used to explain those outputs.
    let c = Contract::new(lift_value(&c.pre, all, &f.x.params)?, post(&c.post, all, &f.x.params)?)?;
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
