//! Named views of inferred contracts.
//!
//! `#[verus_infer]` declares two open spec functions next to an inferred
//! function `f`: `f_requires(params)` and `f_ensures(params, ret)`, each with
//! the placeholder body `true`. After inference installs `f`'s contract, the
//! placeholders are replaced by that contract, with `f`'s parameters (and
//! return binder) renamed to the view's. The views are ordinary definitions:
//! they add no assumption, and the installed contract of `f` is unchanged, so
//! callers that never mention a view behave exactly as before. Proofs can use
//! the names to state lemmas about a large inferred contract.
use super::*;

/// The inferred function a view belongs to: the view's path with its
/// `_requires` / `_ensures` suffix removed.
fn target(view: &Function) -> Option<Fun> {
    let requires = view.x.attrs.infer_view?;
    let suffix = if requires { "_requires" } else { "_ensures" };
    let path = &view.x.name.path;
    let last = path.last_segment();
    let base = last.as_str().strip_suffix(suffix)?;
    let mut segments = (*path.segments).clone();
    *segments.last_mut()? = Arc::new(base.to_string());
    Some(Arc::new(FunX {
        path: Arc::new(PathX { krate: path.krate.clone(), segments: Arc::new(segments) }),
    }))
}

/// Rename `from`'s variables to `to`'s, keeping types and structure. Field
/// projections become place reads, the form spec function bodies use.
fn rename(e: &Expr, names: &HashMap<VarIdent, VarIdent>) -> Result<Expr, VirErr> {
    crate::ast_visitor::map_expr_visitor_env(
        e,
        &mut crate::ast_visitor::VisitorScopeMap::new(),
        &mut (),
        &|_, _, e| {
            Ok(match &e.x {
                ExprX::Var(n) | ExprX::VarAt(n, _) if names.contains_key(n) => {
                    e.new_x(ExprX::Var(names[n].clone()))
                }
                ExprX::UnaryOpr(UnaryOpr::Field(opr), base) if !opr.get_variant => {
                    let base = match &base.x {
                        ExprX::ReadPlace(p, _) => p.clone(),
                        ExprX::Var(n) => {
                            SpannedTyped::new(&base.span, &base.typ, PlaceX::Local(n.clone()))
                        }
                        _ => SpannedTyped::new(
                            &base.span,
                            &base.typ,
                            PlaceX::Temporary(base.clone()),
                        ),
                    };
                    let field =
                        SpannedTyped::new(&e.span, &e.typ, PlaceX::Field(opr.clone(), base));
                    crate::ast_util::place_to_spec_expr(&field)
                }
                _ => e.clone(),
            })
        },
        &|_, _, st| Ok(vec![st.clone()]),
        &|_, t| Ok(t.clone()),
        &|_, _, p| {
            Ok(match &p.x {
                PlaceX::Local(n) if names.contains_key(n) => {
                    p.new_x(PlaceX::Local(names[n].clone()))
                }
                _ => p.clone(),
            })
        },
        &|_, _, p| Ok(p.clone()),
    )
}

/// The view's body, if `f` has an installed contract the view can mirror.
fn body(view: &Function, f: &Function) -> Option<Result<Expr, VirErr>> {
    let requires = view.x.attrs.infer_view?;
    let extra = usize::from(!requires);
    if f.x.attrs.infer_candidate_id.is_none()
        || view.x.params.len() != f.x.params.len() + extra
        || view.x.typ_params.len() != f.x.typ_params.len()
        || !f.x.ensure.1.is_empty()
    {
        return None;
    }
    let mut names: HashMap<VarIdent, VarIdent> =
        f.x.params
            .iter()
            .zip(view.x.params.iter())
            .map(|(from, to)| (from.x.name.clone(), to.x.name.clone()))
            .collect();
    if !requires {
        names.insert(f.x.inner_ret.x.name.clone(), view.x.params.last()?.x.name.clone());
    }
    let clauses = if requires { &f.x.require } else { &f.x.ensure.0 };
    Some(rename(
        &match &clauses[..] {
            [single] => single.clone(),
            _ => crate::ast_util::conjoin(&view.span, clauses),
        },
        &names,
    ))
}

/// Whether a contract mentions something that is not usable everywhere in
/// `within`: a field or variant of a datatype that is opaque there, or a
/// function that is not visible there.
fn mentions_hidden(
    clauses: &[Expr],
    within: &Visibility,
    datatypes: &HashMap<Path, Datatype>,
    functions: &HashMap<Fun, Visibility>,
) -> bool {
    let hidden = |dt: &Dt| match dt {
        Dt::Tuple(_) => false,
        Dt::Path(p) => datatypes.get(p).is_some_and(|d| {
            !within.at_least_as_restrictive_as(&d.x.visibility)
                || crate::ast_util::source_datatype_visibility(&d.x.transparency)
                    .is_none_or(|v| !within.at_least_as_restrictive_as(v))
        }),
    };
    let mut found = false;
    for clause in clauses {
        crate::ast_visitor::expr_visitor_walk(clause, &mut |e| {
            found |= match &e.x {
                ExprX::UnaryOpr(UnaryOpr::Field(FieldOpr { datatype, .. }), _)
                | ExprX::UnaryOpr(UnaryOpr::IsVariant { datatype, .. }, _)
                | ExprX::Ctor(datatype, ..) => hidden(datatype),
                ExprX::Call { target: CallTarget::Fun(_, name, ..), .. } => {
                    functions.get(name).is_some_and(|v| !within.at_least_as_restrictive_as(v))
                }
                _ => false,
            };
            crate::ast_visitor::VisitorControlFlow::Recurse
        });
    }
    found
}

/// Replace every view's placeholder body with its function's installed contract.
pub fn fill(functions: &mut [Function], datatypes: &[Datatype]) -> Result<(), VirErr> {
    let datatypes: HashMap<Path, Datatype> = datatypes
        .iter()
        .filter_map(|d| match &d.x.name {
            Dt::Path(p) => Some((p.clone(), d.clone())),
            Dt::Tuple(_) => None,
        })
        .collect();
    let visibilities: HashMap<Fun, Visibility> =
        functions.iter().map(|f| (f.x.name.clone(), f.x.visibility.clone())).collect();
    let targets: HashMap<Fun, Function> = functions
        .iter()
        .filter(|f| f.x.attrs.infer)
        .map(|f| (f.x.name.clone(), f.clone()))
        .collect();
    // Contracts share subterms; mode checking needs each temporary place to
    // have its own identity, so view bodies get fresh ids from a reserved range.
    let mut id = u64::MAX - (1 << 40);
    // Filled views by (function, is the requires view), and those narrowed to
    // their module.
    let mut filled: HashMap<(Fun, bool), Function> = HashMap::new();
    let mut narrowed = std::collections::HashSet::new();
    for view in functions.iter_mut().filter(|f| f.x.attrs.infer_view.is_some()) {
        let Some(f) = target(view).and_then(|t| targets.get(&t)) else { continue };
        let Some(body) = body(view, f) else { continue };
        let requires = view.x.attrs.infer_view == Some(true);
        let mut x = view.x.clone();
        x.body = Some(super::assistance::fresh(&body?, &mut id)?);
        // A view's body is declared visible wherever the view is. If the
        // contract reads state that is hidden somewhere in that scope (a `pub`
        // getter over private fields, say), the body can only be visible inside
        // the module, like a closed spec function.
        let clauses = if requires { &f.x.require } else { &f.x.ensure.0 };
        if let (Some(module), BodyVisibility::Visibility(within)) =
            (&x.owning_module, &x.body_visibility)
        {
            if mentions_hidden(clauses, within, &datatypes, &visibilities) {
                let private = Visibility { restricted_to: Some(module.clone()) };
                x.body_visibility = BodyVisibility::Visibility(private.clone());
                x.opaqueness = Opaqueness::Revealed { visibility: private };
                narrowed.insert((f.x.name.clone(), requires));
            }
        }
        *view = view.new_x(x);
        filled.insert((f.x.name.clone(), requires), view.clone());
    }
    for f in functions.iter_mut().filter(|f| f.x.attrs.infer) {
        let mut x = f.x.clone();
        // A clause that reads state hidden from some of the function's callers
        // cannot be written in its contract. It is installed as a call to its
        // module-private view instead: callers inside the module see the same
        // formula, callers outside see an abstract predicate.
        //
        // A large precondition of a crate-private function is installed as a
        // call to its view too, so a caller's obligation is literally
        // `f_requires(args)`: a lemma (or broadcast lemma triggered on that
        // term) about the view then matches it. The view is a definition of the
        // same formula, so the contract is unchanged. Other public contracts
        // stay inline for cross-crate importers.
        if let Some(view) = filled.get(&(f.x.name.clone(), true)) {
            let hidden = narrowed.contains(&(f.x.name.clone(), true));
            let large = f.x.visibility.restricted_to.is_some()
                && format_contract_parts(f).0.chars().count() > LARGE_CLAUSE_CHARS;
            if !f.x.require.is_empty() && (hidden || large) {
                x.require = Arc::new(vec![view_call(f, view, None)]);
            }
        }
        if let Some(view) = filled.get(&(f.x.name.clone(), false)) {
            if narrowed.contains(&(f.x.name.clone(), false)) && !f.x.ensure.0.is_empty() {
                let ret = &f.x.inner_ret;
                let ret = SpannedTyped::new(&f.span, &ret.x.typ, ExprX::Var(ret.x.name.clone()));
                x.ensure = (Arc::new(vec![view_call(f, view, Some(ret))]), x.ensure.1.clone());
            }
        }
        *f = f.new_x(x);
    }
    Ok(())
}

/// `view(f's type parameters)(f's parameters, ret?)`, typed as a boolean.
fn view_call(f: &Function, view: &Function, ret: Option<Expr>) -> Expr {
    let span = &f.span;
    let typs: Vec<Typ> =
        f.x.typ_params.iter().map(|t| Arc::new(TypX::TypParam(t.clone()))).collect();
    let args: Vec<Expr> =
        f.x.params
            .iter()
            .map(|p| SpannedTyped::new(span, &p.x.typ, ExprX::Var(p.x.name.clone())))
            .chain(ret)
            .collect();
    SpannedTyped::new(
        span,
        &Arc::new(TypX::Bool),
        ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::Static,
                view.x.name.clone(),
                Arc::new(typs),
                Arc::new(vec![]),
                CallTargetAttrs {
                    autospec: AutospecUsage::Final,
                    const_var: false,
                    assume_external_allowed: false,
                },
            ),
            args: Arc::new(args),
            post_args: None,
            body: None,
        },
    )
}

pub fn is_view(f: &Function) -> bool {
    f.x.attrs.infer_view.is_some()
}

/// Whether a clause of `f`'s contract was installed as a call to one of its
/// named views (`is_view` says which functions are views). Such a contract is
/// what callers rely on, but it is not a summary another crate's inference
/// can use: a hidden clause must stay abstract there.
pub fn installed_through_view(f: &Function, is_view: impl Fn(&Fun) -> bool) -> bool {
    f.x.require.iter().chain(f.x.ensure.0.iter()).any(|clause| {
        matches!(&clause.x, ExprX::Call { target: CallTarget::Fun(_, name, ..), .. } if is_view(name))
    })
}

/// Above this many characters, a formatted clause is reported as large.
pub const LARGE_CLAUSE_CHARS: usize = 2000;

/// Sizes of an installed contract's formatted clauses, and whether it was
/// checked with nonlinear arithmetic.
pub fn contract_size(f: &Function) -> (usize, usize, bool) {
    let (pre, post) = format_contract_parts(f);
    (pre.chars().count(), post.chars().count(), f.x.attrs.nonlinear)
}

/// The names of `f`'s views present among `functions`, as Rust paths.
pub fn view_names(f: &Function, functions: &[Function]) -> Vec<String> {
    let mut names: Vec<_> = functions
        .iter()
        .filter(|v| is_view(v) && target(v).as_ref() == Some(&f.x.name))
        .map(|v| {
            (!v.x.attrs.infer_view.unwrap(), crate::ast_util::fun_as_friendly_rust_name(&v.x.name))
        })
        .collect();
    names.sort();
    names.into_iter().map(|(_, name)| name).collect()
}

/// A note for an inferred contract large enough to be hard for callers to
/// use directly; it points to the named views and to a hand-written fallback.
/// `f` is the generated candidate (its contract before any view-call install).
pub fn large_contract_note(f: &Function, functions: &[Function]) -> Option<String> {
    let (pre, post, nonlinear) = contract_size(f);
    if pre <= LARGE_CLAUSE_CHARS && post <= LARGE_CLAUSE_CHARS {
        return None;
    }
    let name = crate::ast_util::fun_as_friendly_rust_name(&f.x.name);
    let views = view_names(f, functions);
    let refer = if views.is_empty() {
        String::new()
    } else {
        format!(
            " Proofs can refer to it as {}, for example in a lemma that establishes the precondition from a compact domain.",
            views.iter().map(|v| format!("`{v}`")).collect::<Vec<_>>().join(" and ")
        )
    };
    Some(format!(
        "Verus Infer: the inferred contract of `{name}` is large (requires: {pre} characters, ensures: {post} characters{}).{refer} If callers find it hard to use, a hand-written contract stating a compact invariant may be easier; inferred callees can stay inferred.",
        if nonlinear { ", nonlinear arithmetic" } else { "" },
    ))
}
