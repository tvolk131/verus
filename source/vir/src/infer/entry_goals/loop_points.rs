use super::*;
use crate::ast::{Dt, Primitive, TypDecoration, VarIdentDisambiguate};
use crate::infer::report::LoopPointPreparation;
use std::collections::HashSet;

const LOOPS: usize = 8;
const CAPTURES: usize = 64;
type Decline = &'static str;
type Live = HashSet<VarIdent>;

#[cfg(test)]
mod tests;

fn charge(budget: &mut Budget) -> Result<(), Decline> {
    if budget.remaining == 0 {
        return Err("budget_exhausted");
    }
    budget.remaining -= 1;
    Ok(())
}

struct Inventory {
    inspection: Inspection,
    names: HashSet<VarIdent>,
    loops: HashMap<u64, usize>,
}

impl Visitor<Walk, (), NoScoper> for Inventory {
    fn visit_exp(&mut self, expression: &Exp) -> Result<(), ()> {
        self.inspection.budget.enter()?;
        match &expression.x {
            ExpX::Var(name) | ExpX::VarLoc(name) | ExpX::VarAt(name, _) | ExpX::Old(_, name) => {
                self.names.insert(name.clone());
            }
            ExpX::Bind(binder, _) => match &binder.x {
                BndX::Let(binders) => {
                    self.names.extend(binders.iter().map(|binder| binder.name.clone()));
                }
                BndX::Quant(_, binders, _, provenance) => {
                    self.names.extend(binders.iter().map(|binder| binder.name.clone()));
                    if let Some(provenance) = provenance {
                        self.names
                            .extend(provenance.locals.iter().map(|local| local.ident.clone()));
                        self.names
                            .extend(provenance.binders.iter().map(|binder| binder.name.clone()));
                        if let Some(id) = &provenance.assert_id {
                            *self.inspection.metadata.entry(id.clone()).or_default() += 1;
                        }
                    }
                }
                BndX::Lambda(binders, _) | BndX::Choose(binders, _, _) => {
                    self.names.extend(binders.iter().map(|binder| binder.name.clone()));
                }
            },
            _ => {}
        }
        let result = self.visit_exp_rec(expression);
        self.inspection.budget.depth -= 1;
        result
    }

    fn visit_stm(&mut self, statement: &Stm) -> Result<(), ()> {
        self.inspection.budget.enter()?;
        match &statement.x {
            StmX::Loop { id, .. } => *self.loops.entry(*id).or_default() += 1,
            StmX::Assert(Some(id), _, _) => {
                *self.inspection.assertions.entry(id.clone()).or_default() += 1;
            }
            _ => {}
        }
        let result = self.visit_stm_rec(statement);
        self.inspection.budget.depth -= 1;
        result
    }
}

fn value_type(
    datatypes: &HashMap<Dt, crate::ast::Datatype>,
    typ: &Typ,
    budget: &mut Budget,
    depth: usize,
) -> Result<(), Decline> {
    charge(budget)?;
    if depth == DEPTH {
        return Err("budget_exhausted");
    }
    match &**typ {
        TypX::Bool
        | TypX::Int(_)
        | TypX::Real
        | TypX::Float(_)
        | TypX::ConstInt(_)
        | TypX::ConstBool(_)
        | TypX::TypParam(_) => Ok(()),
        TypX::Boxed(inner)
        | TypX::Decorate(
            TypDecoration::Ghost | TypDecoration::Tracked | TypDecoration::Never,
            None,
            inner,
        ) => value_type(datatypes, inner, budget, depth + 1),
        TypX::Primitive(Primitive::Array | Primitive::Slice | Primitive::Global, args)
        | TypX::Opaque { args, .. }
        | TypX::Projection { trait_typ_args: args, .. } => {
            for arg in args.iter() {
                value_type(datatypes, arg, budget, depth + 1)?;
            }
            Ok(())
        }
        TypX::Datatype(datatype, args, _) => {
            let definition = datatypes.get(datatype).ok_or("unsupported_value_type")?;
            if definition.x.typ_params.len() != args.len() {
                return Err("unsupported_value_type");
            }
            for arg in args.iter() {
                value_type(datatypes, arg, budget, depth + 1)?;
            }
            for variant in definition.x.variants.iter() {
                for field in variant.fields.iter() {
                    charge(budget)?;
                    let field = crate::sst_util::subst_typ_for_datatype(
                        &definition.x.typ_params,
                        args,
                        &field.a.0,
                    );
                    value_type(datatypes, &field, budget, depth + 1)?;
                }
            }
            Ok(())
        }
        _ => Err("unsupported_value_type"),
    }
}

#[derive(Clone)]
struct Capture {
    root: VarIdent,
    current: bool,
    rhs: Exp,
    read: Exp,
    declaration: sst::LocalDecl,
}

struct Snapshot<'a> {
    datatypes: &'a HashMap<Dt, crate::ast::Datatype>,
    declarations: &'a HashMap<VarIdent, sst::LocalDecl>,
    live: &'a Live,
    names: &'a mut HashSet<VarIdent>,
    captures: &'a mut Vec<Capture>,
    budget: &'a mut Budget,
    scope: VisitorScopeMap,
}

impl Snapshot<'_> {
    fn capture(&mut self, expression: &Exp, root: &Exp, current: bool) -> Result<Exp, Decline> {
        let ExpX::Var(name) = &root.x else { return Err("unsupported_current_root") };
        if self.scope.contains_key(name) {
            return Err("unsupported_current_root");
        }
        let declared = self.declarations.get(name).ok_or("missing_live_root")?;
        if !self.live.contains(name) || !types_equal(&declared.typ, &root.typ) {
            return Err("missing_live_root");
        }
        if current {
            let TypX::MutRef(inner) = &*root.typ else { return Err("unsupported_current_root") };
            if !types_equal(inner, &expression.typ)
                && !matches!(&*expression.typ, TypX::Boxed(value) if types_equal(inner, value))
            {
                return Err("capture_type_mismatch");
            }
        }
        value_type(self.datatypes, &expression.typ, self.budget, 0)?;
        if let Some(capture) =
            self.captures.iter().find(|capture| capture.root == *name && capture.current == current)
        {
            return if types_equal(&capture.read.typ, &expression.typ) {
                Ok(capture.read.clone())
            } else {
                Err("capture_type_mismatch")
            };
        }
        if self.captures.len() == CAPTURES {
            return Err("budget_exhausted");
        }
        let mut serial = 0;
        let fresh = loop {
            charge(self.budget)?;
            let fresh = VarIdent(
                Arc::new("infer_loop_value".into()),
                VarIdentDisambiguate::VirTemp(serial),
            );
            if self.names.insert(fresh.clone()) {
                break fresh;
            }
            serial += 1;
        };
        let read = expression.new_x(ExpX::Var(fresh.clone()));
        let declaration = Arc::new(sst::LocalDeclX {
            ident: fresh,
            typ: expression.typ.clone(),
            kind: LocalDeclKind::TempViaAssign,
        });
        self.captures.push(Capture {
            root: name.clone(),
            current,
            rhs: expression.clone(),
            read: read.clone(),
            declaration,
        });
        Ok(read)
    }
}

impl Visitor<Rewrite, Decline, VisitorScopeMap> for Snapshot<'_> {
    fn scoper(&mut self) -> Option<&mut VisitorScopeMap> {
        Some(&mut self.scope)
    }

    fn visit_exp(&mut self, expression: &Exp) -> Result<Exp, Decline> {
        self.budget.enter().map_err(|_| "budget_exhausted")?;
        let result = match &expression.x {
            ExpX::Var(name) if !self.scope.contains_key(name) => {
                self.capture(expression, expression, false)
            }
            ExpX::Unary(UnaryOp::MutRefCurrent, root) => self.capture(expression, root, true),
            ExpX::Loc(_)
            | ExpX::VarLoc(_)
            | ExpX::VarAt(..)
            | ExpX::Old(..)
            | ExpX::StaticVar(_)
            | ExpX::CallLambda(..)
            | ExpX::ExecFnByName(_)
            | ExpX::Interp(_)
            | ExpX::Unary(
                UnaryOp::MutRefFinal(_)
                | UnaryOp::MutRefFuture(_)
                | UnaryOp::MustBeFinalized
                | UnaryOp::MustBeElaborated,
                _,
            ) => Err("unsupported_value_expression"),
            ExpX::Bind(binder, _) if !matches!(&binder.x, BndX::Let(_) | BndX::Quant(..)) => {
                Err("unsupported_wrapper")
            }
            _ => self.visit_exp_rec(expression),
        };
        self.budget.depth -= 1;
        result
    }
}

fn quantified(expression: &Exp) -> Option<(&crate::ast::VarBinder<Typ>, &Exp)> {
    let mut current = expression;
    for _ in 0..DEPTH {
        match &current.x {
            ExpX::Bind(binder, body) => match &binder.x {
                BndX::Let(_) => current = body,
                BndX::Quant(quantifier, binders, _, None)
                    if *quantifier == QUANT_FORALL && binders.len() == 1 =>
                {
                    return Some((&binders[0], body));
                }
                _ => return None,
            },
            _ => return None,
        }
    }
    None
}

fn empty_setup(statement: &Stm) -> bool {
    matches!(&statement.x, StmX::Block(children) if children.iter().all(empty_setup))
}

fn remove_quantifier(expression: &Exp, renamed: &Exp, budget: &mut Budget) -> Result<Exp, Decline> {
    charge(budget)?;
    let ExpX::Bind(binder, body) = &expression.x else { return Err("unsupported_wrapper") };
    match &binder.x {
        BndX::Let(_) => {
            Ok(expression
                .new_x(ExpX::Bind(binder.clone(), remove_quantifier(body, renamed, budget)?)))
        }
        BndX::Quant(..) => Ok(renamed.clone()),
        _ => Err("unsupported_wrapper"),
    }
}

fn bounded_expression(expression: &Exp, budget: &mut Budget) -> Result<(), Decline> {
    let mut inspection = Inspection {
        budget: Budget { remaining: budget.remaining, depth: 0 },
        assertions: HashMap::new(),
        metadata: HashMap::new(),
    };
    let result = inspection.visit_exp(expression);
    budget.remaining = inspection.budget.remaining;
    result.map_err(|_| "budget_exhausted")
}

fn instantiate(
    expression: &Exp,
    replacement: &Exp,
    temporary: VarIdent,
    budget: &mut Budget,
) -> Result<Exp, Decline> {
    bounded_expression(expression, budget)?;
    let (binder, body) = quantified(expression).ok_or("unsupported_wrapper")?;
    let placeholder = replacement.new_x(ExpX::Var(temporary.clone()));
    let renamed = crate::sst_util::subst_exp(
        &HashMap::new(),
        &HashMap::from([(binder.name.clone(), placeholder)]),
        body,
    );
    let unbound = remove_quantifier(expression, &renamed, budget)?;
    bounded_expression(&unbound, budget)?;
    let result = crate::sst_util::subst_exp(
        &HashMap::new(),
        &HashMap::from([(temporary, replacement.clone())]),
        &unbound,
    );
    bounded_expression(&result, budget)?;
    Ok(result)
}

fn point_instance(
    ctx: &Ctx,
    expression: &Exp,
    local: &Exp,
    temporary: VarIdent,
    budget: &mut Budget,
) -> Result<Exp, Decline> {
    let (binder, _) = quantified(expression).ok_or("unsupported_wrapper")?;
    let typ = &binder.a;
    let replacement = if types_equal(&local.typ, typ) {
        local.clone()
    } else if matches!(&**typ, TypX::Boxed(inner) if types_equal(inner, &local.typ)) {
        crate::poly::coerce_exp_to_poly(ctx, local)
    } else if matches!(&*local.typ, TypX::Boxed(inner) if types_equal(inner, typ)) {
        crate::poly::coerce_exp_to_native(ctx, local)
    } else {
        return Err("proof_local_type_mismatch");
    };
    if !types_equal(typ, &replacement.typ) {
        return Err("proof_local_type_mismatch");
    }
    instantiate(expression, &replacement, temporary, budget)
}

struct State<'a> {
    ctx: &'a Ctx,
    inventory: Inventory,
    declarations: HashMap<VarIdent, sst::LocalDecl>,
    added: Vec<sst::LocalDecl>,
    loops: usize,
    scopes: usize,
    instances: usize,
    captures: usize,
    active_loop: Option<u64>,
    point_counts: HashMap<u64, usize>,
    observations: Vec<LoopPointPreparation>,
}

impl State<'_> {
    fn observe(
        &mut self,
        statement: &Stm,
        id: u64,
        prepared: usize,
        captures: usize,
        reason: Option<Decline>,
    ) {
        let Some(function) = self.ctx.fun.as_ref() else { return };
        self.observations.push(LoopPointPreparation {
            function: function.current_fun.clone(),
            source: statement.span.clone(),
            loop_id: Some(id),
            prepared,
            captures,
            reason,
        });
    }

    fn local(&self, expression: &Exp) -> Option<Exp> {
        Decorator {
            ctx: self.ctx,
            inspection: &self.inventory.inspection,
            declarations: &self.declarations,
            premises: Vec::new(),
            scopes: 0,
            instances: 0,
            remaining: 0,
        }
        .local(expression)
    }

    fn initialized(&self, destination: &sst::Dest, live: &mut Live) {
        let mut location = &destination.dest;
        for _ in 0..DEPTH {
            match &location.x {
                ExpX::Loc(inner) if types_equal(&location.typ, &inner.typ) => location = inner,
                _ => break,
            }
        }
        if let ExpX::VarLoc(name) = &location.x {
            if self
                .declarations
                .get(name)
                .is_some_and(|declared| types_equal(&declared.typ, &location.typ))
            {
                live.insert(name.clone());
            }
        }
    }

    fn statement(
        &mut self,
        statement: &Stm,
        live: &mut Live,
        premises: &[Exp],
        depth: usize,
    ) -> Result<Stm, Decline> {
        charge(&mut self.inventory.inspection.budget)?;
        if depth == DEPTH {
            return Err("budget_exhausted");
        }
        match &statement.x {
            StmX::Block(children) => {
                let mut result = Vec::new();
                for (position, child) in children.iter().enumerate() {
                    let matched = match (&child.x, children.get(position + 1).map(|next| &next.x)) {
                        (StmX::DeadEnd(proof), Some(StmX::Assume(expression)))
                            if !premises.is_empty() =>
                        {
                            match (&proof.x, self.local(expression)) {
                                (StmX::Block(body), Some(local)) => match body
                                    .last()
                                    .map(|last| &last.x)
                                {
                                    Some(StmX::Assert(Some(id), _, _)) => {
                                        let ExpX::Bind(binder, _) = &expression.x else {
                                            return Err("malformed_provenance");
                                        };
                                        let BndX::Quant(_, _, _, Some(provenance)) = &binder.x
                                        else {
                                            return Err("malformed_provenance");
                                        };
                                        if provenance.assert_id.as_ref() != Some(id) {
                                            None
                                        } else {
                                            if self.scopes == SCOPES {
                                                return Err("budget_exhausted");
                                            }
                                            self.scopes += 1;
                                            let mut generated = body[..body.len() - 1].to_vec();
                                            for premise in premises {
                                                if self.instances == INSTANCES {
                                                    return Err("budget_exhausted");
                                                }
                                                let mut serial = 0;
                                                let temporary = loop {
                                                    charge(&mut self.inventory.inspection.budget)?;
                                                    let name = VarIdent(
                                                        Arc::new("infer_loop_binder".into()),
                                                        VarIdentDisambiguate::VirTemp(serial),
                                                    );
                                                    if self.inventory.names.insert(name.clone()) {
                                                        break name;
                                                    }
                                                    serial += 1;
                                                };
                                                let instance = point_instance(
                                                    self.ctx,
                                                    premise,
                                                    &local,
                                                    temporary,
                                                    &mut self.inventory.inspection.budget,
                                                )?;
                                                generated.extend(checked_use(instance));
                                                self.instances += 1;
                                                if let Some(id) = self.active_loop {
                                                    *self.point_counts.entry(id).or_default() += 1;
                                                }
                                            }
                                            generated.push(body.last().unwrap().clone());
                                            Some(child.new_x(StmX::DeadEnd(
                                                proof.new_x(StmX::Block(Arc::new(generated))),
                                            )))
                                        }
                                    }
                                    _ => None,
                                },
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    result.push(match matched {
                        Some(matched) => matched,
                        None => self.statement(child, live, premises, depth + 1)?,
                    });
                }
                Ok(statement.new_x(StmX::Block(Arc::new(result))))
            }
            StmX::If(condition, positive, negative) => {
                let mut positive_live = live.clone();
                let mut negative_live = live.clone();
                let positive = self.statement(positive, &mut positive_live, premises, depth + 1)?;
                let negative = negative
                    .as_ref()
                    .map(|negative| {
                        self.statement(negative, &mut negative_live, premises, depth + 1)
                    })
                    .transpose()?;
                *live = positive_live.intersection(&negative_live).cloned().collect();
                Ok(statement.new_x(StmX::If(condition.clone(), positive, negative)))
            }
            StmX::Assign { lhs, rhs } => {
                if types_equal(&lhs.dest.typ, &rhs.typ) {
                    self.initialized(lhs, live);
                }
                Ok(statement.clone())
            }
            StmX::Call { dest, body: None, .. } => {
                if let Some(dest) = dest {
                    self.initialized(dest, live);
                }
                Ok(statement.clone())
            }
            StmX::Loop { id, loop_isolation, pre_stms, cond, invs, body, .. } => {
                if self.inventory.loops.get(id) != Some(&1) {
                    self.observe(statement, *id, 0, 0, Some("duplicate_loop_id"));
                    return Ok(statement.clone());
                }
                if self.loops == LOOPS {
                    return Err("budget_exhausted");
                }
                self.loops += 1;
                let mut header_live = live.clone();
                for pre in pre_stms.iter() {
                    self.statement(pre, &mut header_live, &[], depth + 1)?;
                }
                let mut captures = Vec::new();
                let mut frozen = Vec::new();
                let mut reason = None;
                if !loop_isolation {
                    reason = Some("unsupported_nonisolated_loop");
                } else if cond.as_ref().is_some_and(|(setup, _)| !empty_setup(setup)) {
                    reason = Some("effectful_or_nonempty_condition_setup");
                } else {
                    if let Some((_, condition)) = cond {
                        let mut condition_captures = Vec::new();
                        let mut snapshot = Snapshot {
                            datatypes: &self.ctx.datatype_map,
                            declarations: &self.declarations,
                            live: &header_live,
                            names: &mut self.inventory.names,
                            captures: &mut condition_captures,
                            budget: &mut self.inventory.inspection.budget,
                            scope: VisitorScopeMap::new(),
                        };
                        if let Err(decline) = snapshot.visit_exp(condition) {
                            reason = Some(decline);
                        }
                    }
                    if reason.is_none() {
                        for invariant in invs.iter().filter(|invariant| invariant.at_entry) {
                            charge(&mut self.inventory.inspection.budget)?;
                            if quantified(&invariant.inv).is_none() {
                                continue;
                            }
                            if frozen.len() == PREMISES {
                                return Err("budget_exhausted");
                            }
                            let before = captures.len();
                            let mut snapshot = Snapshot {
                                datatypes: &self.ctx.datatype_map,
                                declarations: &self.declarations,
                                live: &header_live,
                                names: &mut self.inventory.names,
                                captures: &mut captures,
                                budget: &mut self.inventory.inspection.budget,
                                scope: VisitorScopeMap::new(),
                            };
                            match snapshot.visit_exp(&invariant.inv) {
                                Ok(premise) => frozen.push(premise),
                                Err("budget_exhausted") => return Err("budget_exhausted"),
                                Err(decline) => {
                                    captures.truncate(before);
                                    reason = Some(decline);
                                }
                            }
                        }
                    }
                }
                if self.captures + captures.len() > CAPTURES {
                    return Err("budget_exhausted");
                }
                let mut body_live = header_live;
                let enclosing = self.active_loop.replace(*id);
                let rewritten = self.statement(body, &mut body_live, &frozen, depth + 1)?;
                self.active_loop = enclosing;
                let prepared = self.point_counts.get(id).copied().unwrap_or(0);
                let rewritten = if prepared == 0 {
                    rewritten
                } else {
                    if self.captures + captures.len() > CAPTURES {
                        return Err("budget_exhausted");
                    }
                    let mut prefix = Vec::new();
                    for capture in &captures {
                        self.added.push(capture.declaration.clone());
                        prefix.push(crate::ast_to_sst::init_var(
                            &statement.span,
                            &capture.declaration.ident,
                            &capture.rhs,
                        ));
                    }
                    for premise in &frozen {
                        prefix.extend(checked_use(premise.clone()));
                    }
                    self.captures += captures.len();
                    prefix.push(rewritten);
                    body.new_x(StmX::Block(Arc::new(prefix)))
                };
                self.observe(
                    statement,
                    *id,
                    prepared,
                    if prepared == 0 { 0 } else { captures.len() },
                    reason.or(if prepared == 0 { Some("no_matching_proof_point") } else { None }),
                );
                let mut result = statement.x.clone();
                let StmX::Loop { body, .. } = &mut result else { unreachable!() };
                *body = rewritten;
                Ok(statement.new_x(result))
            }
            _ => Ok(statement.clone()),
        }
    }
}

pub(crate) fn decorate(
    check: &FuncCheckSst,
    parameters: &sst::Pars,
    ctx: &Ctx,
) -> (FuncCheckSst, Vec<LoopPointPreparation>) {
    let Some(function) =
        ctx.fun.as_ref().and_then(|function| ctx.func_map.get(&function.current_fun))
    else {
        return (check.clone(), Vec::new());
    };
    if let Some(reason) = super::super::eligibility::authored_loop_points(function) {
        return (
            check.clone(),
            vec![LoopPointPreparation {
                function: function.x.name.clone(),
                source: function.span.clone(),
                loop_id: None,
                prepared: 0,
                captures: 0,
                reason: Some(reason),
            }],
        );
    }
    let mut inventory = Inventory {
        inspection: Inspection {
            budget: Budget::new(),
            assertions: HashMap::new(),
            metadata: HashMap::new(),
        },
        names: HashSet::new(),
        loops: HashMap::new(),
    };
    let mut declarations = HashMap::new();
    let mut live = Live::new();
    let result = (|| {
        if check.local_decls.len() > NODES || parameters.len() > NODES {
            return Err("budget_exhausted");
        }
        for declaration in check.local_decls.iter() {
            charge(&mut inventory.inspection.budget)?;
            inventory.names.insert(declaration.ident.clone());
            if declarations.insert(declaration.ident.clone(), declaration.clone()).is_some() {
                return Err("duplicate_declaration");
            }
        }
        for parameter in parameters.iter() {
            let declared = declarations.get(&parameter.x.name).ok_or("missing_live_root")?;
            if !matches!(declared.kind, LocalDeclKind::Param { .. })
                || !types_equal(&parameter.x.typ, &declared.typ)
            {
                return Err("capture_type_mismatch");
            }
            live.insert(parameter.x.name.clone());
        }
        inventory.visit_func_check(check).map_err(|_| "budget_exhausted")?;
        Ok(())
    })();
    let declined = |reason| {
        vec![LoopPointPreparation {
            function: function.x.name.clone(),
            source: check.body.span.clone(),
            loop_id: None,
            prepared: 0,
            captures: 0,
            reason: Some(reason),
        }]
    };
    if let Err(reason) = result {
        return (check.clone(), declined(reason));
    }
    let mut state = State {
        ctx,
        inventory,
        declarations,
        added: Vec::new(),
        loops: 0,
        scopes: 0,
        instances: 0,
        captures: 0,
        active_loop: None,
        point_counts: HashMap::new(),
        observations: Vec::new(),
    };
    match state.statement(&check.body, &mut live, &[], 0) {
        Ok(body) => {
            let mut result = check.clone();
            result.body = body;
            let mut declarations = check.local_decls.as_ref().clone();
            declarations.extend(state.added);
            result.local_decls = Arc::new(declarations);
            (result, state.observations)
        }
        Err(reason) => (check.clone(), declined(reason)),
    }
}
