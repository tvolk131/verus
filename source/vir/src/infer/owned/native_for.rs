//! Candidate-only total native for-loop folds. Runtime code is never rewritten.
//! The ordinary body proves total safety and the proposed fold invariant; this
//! profile always proposes `requires true`, so maximality needs no safety erasure.
use super::*;

#[derive(Clone)]
pub(in crate::infer) struct Source {
    pub loop_expr: Expr,
    pub iterator: Expr,
    pub pattern: Pattern,
    pub body: Expr,
    pub wrapper: PatternBinding,
}
fn plain(e: &Expr) -> &Expr {
    match &e.x {
        ExprX::Block(ss, Some(e)) if ss.is_empty() => plain(e),
        ExprX::EvalAndResolve(e, _) => plain(e),
        _ => e,
    }
}
fn temporary(p: &Place) -> Option<&Expr> {
    if let PlaceX::Temporary(e) = &p.x { Some(e) } else { None }
}
fn decl(s: &Stmt) -> Option<(&Pattern, Option<&Place>)> {
    if let StmtX::Decl { pattern, init, els: None, .. } = &s.x {
        Some((pattern, init.as_ref()))
    } else {
        None
    }
}
fn binding(p: &Pattern, name: &str) -> Option<PatternBinding> {
    if let PatternX::Var(b) = &p.x {
        if b.name.0.as_str() == name {
            return Some(b.clone());
        }
    }
    None
}
pub(super) fn source(e: &Expr) -> Option<Source> {
    let ExprX::UnaryOpr(UnaryOpr::LoopIsolationBoundary(_), e) = &plain(e).x else {
        return None;
    };
    let ExprX::Block(ss, Some(_)) = &plain(e).x else {
        return None;
    };
    if ss.len() != 6 {
        return None;
    }
    let (p, Some(init)) = decl(&ss[0])? else {
        return None;
    };
    binding(p, "VERUS_iter_init")?;
    let iterator = temporary(init)?.clone();
    let (_, Some(init)) = decl(&ss[5])? else {
        return None;
    };
    let ExprX::Match(_, arms, _) = &plain(temporary(init)?).x else {
        return None;
    };
    if arms.len() != 1 {
        return None;
    }
    let wrapper = binding(&arms[0].x.pattern, "VERUS_ghost_iter")?;
    let loop_expr = plain(&arms[0].x.body).clone();
    let ExprX::Loop {
        body, is_for_loop: true, assume_termination: false, atomic_call: false, ..
    } = &loop_expr.x
    else {
        return None;
    };
    let ExprX::Block(body_ss, None) = &plain(body).x else {
        return None;
    };
    if body_ss.len() != 9 {
        return None;
    }
    let (pattern, Some(_)) = decl(&body_ss[4])? else {
        return None;
    };
    let (_, Some(body)) = decl(&body_ss[8])? else {
        return None;
    };
    Some(Source {
        loop_expr: loop_expr.clone(),
        iterator,
        pattern: pattern.clone(),
        body: temporary(body)?.clone(),
        wrapper,
    })
}
pub(in crate::infer) fn sources(f: &Function) -> Vec<Source> {
    let mut found = vec![];
    if let Some(body) = &f.x.body {
        crate::ast_visitor::expr_visitor_walk(body, &mut |e| {
            if let Some(s) = source(e) {
                if !found.iter().any(|a: &Source| a.loop_expr.span.id == s.loop_expr.span.id) {
                    found.push(s);
                }
            }
            crate::ast_visitor::VisitorControlFlow::<()>::Recurse
        });
    }
    found
}
pub(super) fn scaffold_call(f: &Function, e: &Expr, all: &HashMap<Fun, Function>) -> bool {
    let mut has_for = false;
    if let Some(body) = &f.x.body {
        crate::ast_visitor::expr_visitor_walk(body, &mut |e| {
            has_for |= matches!(&e.x, ExprX::Loop { is_for_loop: true, .. });
            crate::ast_visitor::VisitorControlFlow::<()>::Recurse
        });
    }
    if !has_for {
        return false;
    }
    let ExprX::Call { target: CallTarget::Fun(_, name, ..), .. } = &e.x else {
        return false;
    };
    all.get(name).is_some_and(|f| f.x.mode == Mode::Spec)
        || models::path_is(
            &name.path,
            CrateId::Vstd,
            &["std_specs", "iter", "trigger_peek_implications"],
        )
        || models::path_is(&name.path, CrateId::Vstd, &["std_specs", "option", "spec_unwrap_or"])
        || models::path_is(&name.path, CrateId::Vstd, &["pervasive", "arbitrary"])
        || name.path.krate == CrateId::Vstd
            && name.path.segments.len() == 4
            && name.path.segments[0].as_str() == "std_specs"
            && name.path.segments[1].as_str() == "iter"
            && name.path.segments[2].as_str() == "IteratorSpec"
            && matches!(
                name.path.last_segment().as_str(),
                "peek" | "decrease" | "will_return_none" | "obeys_prophetic_iter_laws"
            )
        || models::path_is(
            &name.path,
            CrateId::Core,
            &["iter", "traits", "collect", "IntoIterator", "into_iter"],
        )
        || name.path.krate == CrateId::Core
            && name.path.segments.len() == 5
            && name.path.segments[0].as_str() == "iter"
            && name.path.segments[1].as_str() == "traits"
            && name.path.segments[2].as_str() == "collect"
            && name.path.segments[3].starts_with("impl")
            && name.path.last_segment().as_str() == "into_iter"
        || name.path.krate == CrateId::Vstd
            && name.path.segments.len() == 4
            && name.path.segments[0].as_str() == "std_specs"
            && name.path.segments[1].as_str() == "iter"
            && name.path.segments[2].starts_with("impl")
            && matches!(name.path.segments[3].as_str(), "new" | "next" | "wf" | "seq" | "index")
}
impl Engine<'_> {
    fn wrapper_call(&self, source: &Source, name: &str) -> Result<Term, VirErr> {
        let f = self
            .all
            .values()
            .find(|f| {
                f.x.mode == Mode::Spec
                    && f.x.name.path.krate == CrateId::Vstd
                    && f.x.name.path.segments.len() == 4
                    && f.x.name.path.segments[0].as_str() == "std_specs"
                    && f.x.name.path.segments[1].as_str() == "iter"
                    && f.x.name.path.segments[2].starts_with("impl")
                    && f.x.name.path.last_segment().as_str() == name
            })
            .ok_or_else(|| unsupported(&source.body.span, "missing native for-loop observation"))?;
        let TypX::Datatype(_, types, _) = &*source.wrapper.typ else {
            return Err(unsupported(&source.body.span, "native wrapper type"));
        };
        let types = types.clone();
        let ts = f.x.typ_params.iter().cloned().zip(types.iter().cloned()).collect();
        let typ = crate::sst_util::subst_typ(&ts, &f.x.inner_ret.x.typ);
        atom(SpannedTyped::new(
            &source.body.span,
            &typ,
            ExprX::Call {
                target: CallTarget::Fun(
                    CallTargetKind::Static,
                    f.x.name.clone(),
                    types,
                    Arc::new(vec![]),
                    CallTargetAttrs {
                        autospec: AutospecUsage::Final,
                        const_var: false,
                        assume_external_allowed: false,
                    },
                ),
                args: Arc::new(vec![SpannedTyped::new(
                    &source.body.span,
                    &source.wrapper.typ,
                    ExprX::Var(source.wrapper.name.clone()),
                )]),
                post_args: None,
                body: None,
            },
        ))
    }
    pub(super) fn native_for(&mut self, s: &mut State, source: &Source) -> Result<Val, VirErr> {
        let span = &source.loop_expr.span;
        if !self.plans.is_empty() {
            return Err(unsupported(span, "one native for-loop per owned constructor"));
        }
        let iterator = self.expr(s, &source.iterator)?;
        let Node::Iterator(full, false) = iterator.node else {
            return Err(unsupported(span, "native loop requires a supplied finite factory"));
        };
        let item = iterators::sequence(&full.expr.typ)
            .ok_or_else(|| unsupported(span, "native loop item view"))?
            .clone();
        let mut destinations = vec![];
        crate::ast_visitor::expr_visitor_walk(&source.body, &mut |e| {
            if let ExprX::Call { args, .. } = &e.x {
                for a in args.iter() {
                    if let ExprX::BorrowMut(p) | ExprX::TwoPhaseBorrowMut(p) = &a.x {
                        if let PlaceX::Local(n) = &p.x {
                            if !destinations.contains(n) {
                                destinations.push(n.clone());
                            }
                        }
                    }
                }
            }
            crate::ast_visitor::VisitorControlFlow::<()>::Recurse
        });
        if destinations.len() != 1 {
            return Err(unsupported(
                span,
                "native folds require one mutable collection accumulator",
            ));
        }
        let dest = destinations.pop().unwrap();
        let initial = s
            .locals
            .get(&dest)
            .ok_or_else(|| unsupported(span, "native fold accumulator"))?
            .clone();
        let Node::Seq(initial_seq) = &initial.node else {
            return Err(unsupported(span, "native fold needs a sequence accumulator"));
        };
        let acc =
            VarIdent(Arc::new("__verus_infer_fold_acc".into()), VarIdentDisambiguate::AirLocal);
        let entry =
            VarIdent(Arc::new("__verus_infer_fold_entry".into()), VarIdentDisambiguate::AirLocal);
        let acc_value =
            atom(SpannedTyped::new(span, &initial_seq.expr.typ, ExprX::Var(acc.clone())))?;
        let entry_value = atom(SpannedTyped::new(span, &item, ExprX::Var(entry.clone())))?;
        let mut step = s.clone();
        step.locals.insert(acc.clone(), Val::atom(acc_value.clone()));
        step.locals.insert(entry.clone(), Val::atom(entry_value.clone()));
        step.locals.insert(
            dest.clone(),
            Val { identity: None, typ: initial.typ.clone(), node: Node::Seq(acc_value) },
        );
        self.pattern(&mut step, &source.pattern, Val::atom(entry_value))?;
        let req_start = self.requirements.len();
        let exits_start = self.exits.len();
        self.expr(&mut step, &source.body)?;
        let requirements = self.requirements.drain(req_start..).collect::<Vec<_>>();
        if self.exits.len() != exits_start {
            return Err(unsupported(span, "native fold body cannot return"));
        }
        let Node::Seq(next) = step
            .locals
            .get(&dest)
            .ok_or_else(|| unsupported(span, "native fold step"))?
            .node
            .clone()
        else {
            return Err(unsupported(span, "native fold sequence step"));
        };
        let lambda = term(
            span,
            &Arc::new(TypX::SpecFn(
                Arc::new(vec![initial_seq.expr.typ.clone(), item.clone()]),
                initial_seq.expr.typ.clone(),
            )),
            ExprX::Closure(
                Arc::new(vec![
                    Arc::new(VarBinderX { name: acc.clone(), a: initial_seq.expr.typ.clone() }),
                    Arc::new(VarBinderX { name: entry.clone(), a: item.clone() }),
                ]),
                next.expr.clone(),
            ),
            next.size + 1,
        )?;
        // Canonicalize a single indexed update into its checked library vocabulary.
        // This changes only the proposed equation; the original loop and fold
        // invariant still prove it. Other steps retain the general fold summary.
        let mut scatter = None;
        if let ExprX::Call {
            target: CallTarget::Fun(_, name, ..),
            args,
            post_args: None,
            body: None,
        } = &next.expr.x
        {
            if models::path_is(&name.path, CrateId::Vstd, &["seq", "Seq", "update"])
                && args.len() == 3
                && matches!(&args[0].x, ExprX::Var(n) if n == &acc)
                && args[1..].iter().all(|e| {
                    crate::ast_visitor::expr_visitor_check(e, &mut |_, e| {
                        if matches!(&e.x, ExprX::Var(n) if n == &acc) { Err(()) } else { Ok(()) }
                    })
                    .is_ok()
                })
            {
                let elem = iterators::sequence(&initial_seq.expr.typ).unwrap().clone();
                let closure = |body: &Expr| {
                    term(
                        span,
                        &Arc::new(TypX::SpecFn(Arc::new(vec![item.clone()]), body.typ.clone())),
                        ExprX::Closure(
                            Arc::new(vec![Arc::new(VarBinderX {
                                name: entry.clone(),
                                a: item.clone(),
                            })]),
                            body.clone(),
                        ),
                        next.size + 1,
                    )
                };
                let index = term(
                    span,
                    &int_typ(),
                    ExprX::Unary(
                        UnaryOp::Clip { range: IntRange::Int, truncate: false },
                        args[1].clone(),
                    ),
                    next.size + 1,
                )?;
                scatter = Some((elem, closure(&index.expr)?, closure(&args[2])?));
            }
        }
        let fold = |engine: &Engine<'_>, seq: Term| {
            if let Some((elem, index, value)) = &scatter {
                return engine.lib(
                    span,
                    Model::Scatter,
                    vec![item.clone(), elem.clone()],
                    vec![seq, initial_seq.clone(), index.clone(), value.clone()],
                );
            }
            engine.lib(
                span,
                Model::SeqFold,
                vec![item.clone(), initial_seq.expr.typ.clone()],
                vec![seq, initial_seq.clone(), lambda.clone()],
            )
        };
        let actual =
            self.layout(SpannedTyped::new(span, &initial.typ, ExprX::Var(dest.clone())), 5)?;
        let Node::Seq(actual_seq) = actual.node else { unreachable!() };
        let TypX::Datatype(_, wrapper_types, _) = &*source.wrapper.typ else {
            return Err(unsupported(span, "native iterator wrapper shape"));
        };
        if wrapper_types.len() != 1 || !same(&wrapper_types[0], &source.iterator.typ) {
            return Err(unsupported(
                span,
                "native wrapper must contain the supplied factory iterator",
            ));
        }
        let mut source_seq = None;
        crate::ast_visitor::expr_visitor_walk(&source.loop_expr, &mut |e| {
            if let ExprX::Call { target: CallTarget::Fun(_, name, ..), args, .. } = &e.x {
                if name.path.krate == CrateId::Vstd
                    && name.path.segments.len() == 4
                    && name.path.segments[0].as_str() == "std_specs"
                    && name.path.segments[1].as_str() == "iter"
                    && name.path.last_segment().as_str() == "seq"
                    && args.len() == 1
                    && matches!(&args[0].x,ExprX::ReadPlace(p,_) if matches!(&p.x,PlaceX::Local(n) if n==&source.wrapper.name))
                {
                    source_seq = Some(e.clone());
                }
            }
            crate::ast_visitor::VisitorControlFlow::<()>::Recurse
        });
        let seq = atom(source_seq.ok_or_else(|| {
            unsupported(span, "missing original native iterator snapshot observation")
        })?)?;
        if !same(&seq.expr.typ, &full.expr.typ) {
            return Err(unsupported(
                span,
                "native iterator observation must retain Rust-normalized item types",
            ));
        }
        let index = self.wrapper_call(source, "index")?;
        let prefix = self.lib(span, Model::Take, vec![item.clone()], vec![seq.clone(), index])?;
        let mut invs = vec![records::equal(span, actual_seq.clone(), fold(self, prefix)?)?];
        let elem = element(&initial.typ, self.all, self.ds)?;
        let len = |engine: &Engine<'_>, seq: Term| {
            engine.standard(span, collections::Model::SeqLen, vec![elem.clone()], vec![seq])
        };
        invs.push(records::equal(
            span,
            len(self, actual_seq.clone())?,
            len(self, initial_seq.clone())?,
        )?);
        // Keep every step's readiness as an inductive invariant. This profile
        // proposes true rather than using the readiness condition as a domain;
        // ordinary verification must establish and preserve it for all inputs.
        let i =
            VarIdent(Arc::new("__verus_infer_fold_index".into()), VarIdentDisambiguate::AirLocal);
        let i_value = atom(SpannedTyped::new(span, &int_typ(), ExprX::Var(i.clone())))?;
        let entry_at = self.standard(
            span,
            collections::Model::SeqIndex,
            vec![item.clone()],
            vec![seq.clone(), i_value.clone()],
        )?;
        // A step may ignore its entry when checking readiness (for example a
        // constant destination index). Always retain an entry-index trigger so
        // the generated forall does not depend on SMT trigger guessing.
        let trigger = entry_at.expr.clone();
        let env = HashMap::from([(acc.clone(), actual_seq.clone()), (entry.clone(), entry_at)]);
        let mut req = boolean(span, true);
        for r in requirements {
            let substituted = crate::ast_visitor::map_expr_visitor(&r.expr, &|e| {
                if let ExprX::Var(n) = &e.x {
                    if let Some(value) = env.get(n) {
                        return Ok(value.expr.clone());
                    }
                }
                Ok(e.clone())
            })?;
            req = logical(span, LogicalOp::And, req, atom(substituted)?)?;
        }
        let length =
            self.standard(span, collections::Model::SeqLen, vec![item.clone()], vec![seq])?;
        let zero = term(span, &int_typ(), ExprX::Const(Constant::Int(0.into())), 1)?;
        let bounds = logical(
            span,
            LogicalOp::And,
            summary::binary(span, BinaryOp::Inequality(InequalityOp::Le), zero, i_value.clone())?,
            summary::binary(span, BinaryOp::Inequality(InequalityOp::Lt), i_value, length)?,
        )?;
        let req = logical(span, LogicalOp::Implies, bounds, req)?;
        invs.push(term(
            span,
            &bool_typ(),
            ExprX::Quant(
                Quant { quant: air::ast::Quant::Forall },
                Arc::new(vec![Arc::new(VarBinderX { name: i, a: int_typ() })]),
                req.expr.new_x(ExprX::WithTriggers {
                    triggers: Arc::new(vec![Arc::new(vec![trigger])]),
                    body: req.expr.clone(),
                }),
            ),
            req.size + 1,
        )?);
        let ExprX::Loop { invs: original, decrease, .. } = &source.loop_expr.x else {
            unreachable!()
        };
        // Generated proof nodes must not reuse executable AstIds: resolution
        // inference attaches borrow-expiration facts to those original IDs.
        let mut fresh_id = u64::MAX - 1;
        let mut fresh = vec![];
        for inv in invs {
            let inv = crate::ast_visitor::map_expr_visitor_env(
                &inv.expr,
                &mut crate::ast_visitor::VisitorScopeMap::new(),
                &mut fresh_id,
                &|id, _, e| {
                    let mut span = e.span.clone();
                    span.id = *id;
                    *id -= 1;
                    Ok(SpannedTyped::new(&span, &e.typ, e.x.clone()))
                },
                &|_, _, s| Ok(vec![s.clone()]),
                &|_, t| Ok(t.clone()),
                &|id, _, p| {
                    let mut span = p.span.clone();
                    span.id = *id;
                    *id -= 1;
                    Ok(SpannedTyped::new(&span, &p.typ, p.x.clone()))
                },
                &|_, _, p| Ok(p.clone()),
            )?;
            fresh.push(LoopInvariant { kind: LoopInvariantKind::InvariantAndEnsures, inv });
        }
        let mut invs = fresh;
        invs.extend(original.iter().cloned());
        self.plans.push(loops::Plan {
            span: span.id,
            invs: Arc::new(invs),
            decrease: decrease.clone(),
        });
        s.locals.insert(
            dest,
            Val { identity: None, typ: initial.typ, node: Node::Seq(fold(self, full)?) },
        );
        Ok(Val::atom(exits::unit_value(span)?))
    }
}

pub(super) fn pure(e: &Expr) -> Result<(), VirErr> {
    crate::ast_visitor::expr_visitor_check(e, &mut |_, e| {
        let valid = match &e.x {
            ExprX::Const(_)
            | ExprX::Var(_)
            | ExprX::Unary(
                UnaryOp::Not
                | UnaryOp::BitNot(_)
                | UnaryOp::Clip { .. }
                | UnaryOp::CoerceMode { .. },
                _,
            )
            | ExprX::UnaryOpr(
                UnaryOpr::Field(_) | UnaryOpr::IsVariant { .. } | UnaryOpr::IntegerTypeBound(_),
                _,
            )
            | ExprX::Binary(..)
            | ExprX::Logical(..)
            | ExprX::Quant(..)
            | ExprX::WithTriggers { .. }
            | ExprX::Closure(..)
            | ExprX::Ctor(_, _, _, None)
            | ExprX::If(_, _, Some(_)) => true,
            ExprX::ReadPlace(_, _) => loops::pure(e).is_ok(),
            ExprX::Call {
                target: CallTarget::Fun(_, name, ..),
                post_args: None,
                body: None,
                ..
            } => {
                name.path.krate == CrateId::Vstd
                    && (models::path_is(&name.path, CrateId::Vstd, &["seq_update", "scatter"])
                        || models::path_is(&name.path, CrateId::Vstd, &["view", "View", "view"])
                        || name.path.segments.len() == 3
                            && matches!(name.path.segments[0].as_str(), "seq" | "seq_lib")
                            && matches!(
                                name.path.last_segment().as_str(),
                                "new" | "empty" | "len" | "index" | "update" | "take" | "fold_left"
                            )
                        || name.path.segments.len() == 4
                            && name.path.segments[0].as_str() == "std_specs"
                            && name.path.segments[1].as_str() == "iter"
                            && matches!(name.path.last_segment().as_str(), "seq" | "index"))
            }
            _ => false,
        };
        if valid { Ok(()) } else { Err(unsupported(&e.span, "nonpure native fold invariant")) }
    })
}
