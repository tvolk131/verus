//! Proposed constructor observations through an author's tuple-valued View.
//! Completeness is not assumed: SST exactness still observes the original type.
use super::*;

pub(super) struct RecordView {
    pub function: Function,
    types: Typs,
}

pub(super) fn observation(f: &Function) -> bool {
    is_pure_observation(f)
        && !f.x.attrs.is_external_body
        && f.x.proxy.is_none()
        && f.x.params.len() == 1
        && f.x.visibility.is_public()
        && f.x.typ_params.len() <= 4
        && matches!(&f.x.kind,FunctionKind::TraitMethodImpl{method,..}
            if models::path_is(&method.path,CrateId::Vstd,&["view","View","view"]))
}

pub(super) fn find(t: &Typ, all: &HashMap<Fun, Function>, ds: &[Datatype]) -> Option<RecordView> {
    let TypX::Datatype(dt, args, _) = &**base(t) else {
        return None;
    };
    let d = ds.iter().find(|d| &d.x.name == dt)?;
    let fields = records::shape::fields(base(t), ds)?;
    // This new profile handles complete opaque components. Ordinary records and
    // certified sequence wrappers retain their established output schemas.
    // Use the datatype's owner, so imported and local selection is identical.
    if !fields.iter().any(|(_, t)| {
        records::shape::opaque_value(t, ds, &d.x.owning_module)
            && sequence_views::find(t, all, ds).is_none()
    }) {
        return None;
    }
    let params: Vec<_> = d.x.typ_params.iter().map(|p| p.0.clone()).collect();
    if !fields.iter().all(|(_, t)| {
        records::shape::value_type(t, &params, ds, 8)
            || records::shape::opaque_value(t, ds, &d.x.owning_module)
                && sequence_views::find(t, all, ds).is_none()
    }) {
        return None;
    }
    let mut found = None;
    for f in all.values().filter(|f| observation(f)) {
        let TypX::Datatype(actual, params, _) = &**base(&f.x.params[0].x.typ) else {
            continue;
        };
        if actual != dt
            || params.len() != args.len()
            || f.x.typ_params.len() != args.len()
            || !params
                .iter()
                .zip(f.x.typ_params.iter())
                .all(|(t, n)| matches!(&**t,TypX::TypParam(p) if p==n))
        {
            continue;
        }
        let ts = f.x.typ_params.iter().cloned().zip(args.iter().cloned()).collect();
        let ret = crate::sst_util::subst_typ(&ts, &f.x.inner_ret.x.typ);
        if !matches!(&*ret,TypX::Datatype(Dt::Tuple(n),xs,_) if (1..=4).contains(n) && xs.len()==*n)
        {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some(RecordView { function: f.clone(), types: args.clone() });
    }
    found
}

impl RecordView {
    fn substitutions(&self) -> HashMap<Ident, Typ> {
        self.function.x.typ_params.iter().cloned().zip(self.types.iter().cloned()).collect()
    }
    pub(super) fn call(&self, value: Expr) -> Result<Term, VirErr> {
        let f = &self.function;
        let ts = self.substitutions();
        let span = value.span.clone();
        let arg =
            coerce(&span, atom(value)?, &crate::sst_util::subst_typ(&ts, &f.x.params[0].x.typ))?
                .expr;
        let FunctionKind::TraitMethodImpl { method, impl_path, trait_typ_args, .. } = &f.x.kind
        else {
            unreachable!()
        };
        atom(SpannedTyped::new(
            &span,
            &crate::sst_util::subst_typ(&ts, &f.x.inner_ret.x.typ),
            ExprX::Call {
                target: CallTarget::Fun(
                    CallTargetKind::DynamicResolved {
                        resolved: f.x.name.clone(),
                        typs: self.types.clone(),
                        impl_paths: Arc::new(vec![]),
                        is_trait_default: false,
                    },
                    method.clone(),
                    Arc::new(
                        trait_typ_args.iter().map(|t| crate::sst_util::subst_typ(&ts, t)).collect(),
                    ),
                    Arc::new(vec![ImplPath::TraitImplPath(impl_path.clone())]),
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
        ))
    }
    pub(super) fn unique(&self, out: &Expr) -> Result<Term, VirErr> {
        let peer = SpannedTyped::new(
            &out.span,
            &out.typ,
            ExprX::Var(VarIdent(
                Arc::new("__verus_infer_record_peer".into()),
                VarIdentDisambiguate::AirLocal,
            )),
        );
        let ExprX::Var(name) = &peer.x else { unreachable!() };
        let observed = self.call(peer.clone())?;
        let same_view = records::equal(&out.span, observed.clone(), self.call(out.clone())?)?;
        let same_value = records::equal(&out.span, atom(peer.clone())?, atom(out.clone())?)?;
        let implication = logical(&out.span, LogicalOp::Implies, same_view, same_value)?;
        atom(SpannedTyped::new(
            &out.span,
            &bool_typ(),
            ExprX::Quant(
                Quant { quant: air::ast::Quant::Forall },
                Arc::new(vec![Arc::new(VarBinderX { name: name.clone(), a: out.typ.clone() })]),
                implication.expr.new_x(ExprX::WithTriggers {
                    triggers: Arc::new(vec![Arc::new(vec![observed.expr])]),
                    body: implication.expr.clone(),
                }),
            ),
        ))
    }
    fn body(&self, all: &HashMap<Fun, Function>) -> Result<Expr, VirErr> {
        let f = &self.function;
        let body = f.x.body.as_ref().ok_or_else(|| {
            unsupported(
                &f.span,
                "constructing an abstract record requires its authored View definition",
            )
        })?;
        let ts = self.substitutions();
        let body = crate::ast_visitor::map_expr_visitor_env(
            body,
            &mut crate::ast_visitor::VisitorScopeMap::new(),
            &mut (),
            &|_, _, e| Ok(e.clone()),
            &|_, _, s| Ok(vec![s.clone()]),
            &|_, t| {
                Ok(match &**t {
                    TypX::TypParam(n) => ts.get(n).cloned().unwrap_or_else(|| t.clone()),
                    _ => t.clone(),
                })
            },
            &|_, _, p| Ok(p.clone()),
            &|_, _, p| Ok(p.clone()),
        )?;
        observations::expand_projections(&body, all)
    }
}

impl Engine<'_> {
    pub(super) fn record_view_value(&self, value: &Val, view: &RecordView) -> Result<Term, VirErr> {
        if let Node::Abstract(t) = &value.node {
            return Ok(t.clone());
        }
        let body = view.body(self.all)?;
        let mut state = State {
            locals: HashMap::from([(view.function.x.params[0].x.name.clone(), value.clone())]),
            objects: HashMap::new(),
            active: boolean(&body.span, true),
        };
        if let Some(t) = &value.identity {
            state.objects.insert(view.function.x.params[0].x.name.clone(), t.clone());
        } else if let Node::Atom(t) = &value.node {
            state.objects.insert(view.function.x.params[0].x.name.clone(), t.clone());
        }
        self.formula(&state, &body)
    }

    pub(super) fn record_output_layout(
        &self,
        mut value: Val,
        out: &Expr,
        depth: usize,
    ) -> Result<Val, VirErr> {
        if depth == 0 {
            return Err(unsupported(&out.span, "record output view depth exceeded"));
        }
        if trait_getters::shared(&out.typ).is_some() {
            return Ok(value);
        }
        if let Some(view) = find(&value.typ, self.all, self.ds) {
            value.node = Node::Abstract(view.call(out.clone())?);
            return Ok(value);
        }
        value.node = match value.node {
            Node::Fields(fs) => Node::Fields(
                fs.into_iter()
                    .map(|(op, v)| {
                        let field = field_expr(out, &op, &v.typ);
                        Ok((op, self.record_output_layout(v, &field, depth - 1)?))
                    })
                    .collect::<Result<_, VirErr>>()?,
            ),
            Node::Result(tag, a, b) => {
                let part = |variant: &str, v: Option<Box<Val>>| -> Result<_, VirErr> {
                    v.map(|v| {
                        let field = field_expr(out, &sums::field(&out.typ, variant), &v.typ);
                        self.record_output_layout(*v, &field, depth - 1).map(Box::new)
                    })
                    .transpose()
                };
                Node::Result(tag, part("Ok", a)?, part("Err", b)?)
            }
            Node::Option(tag, v) => Node::Option(
                tag,
                v.map(|v| {
                    let field = field_expr(out, &option_field(&out.typ), &v.typ);
                    self.record_output_layout(*v, &field, depth - 1).map(Box::new)
                })
                .transpose()?,
            ),
            other => other,
        };
        Ok(value)
    }
}
