use crate::ast::{SpannedTyped, Typ, TypX, UnaryOp, VarAt, VarIdent};
use crate::ast_util::{QUANT_FORALL, types_equal};
use crate::context::Ctx;
use crate::def::Spanned;
use crate::sst::{self, BndX, Exp, ExpX, FuncCheckSst, LocalDeclKind, Stm, StmX};
use crate::sst_visitor::{NoScoper, Rewrite, Visitor, VisitorScopeMap, Walk};
use std::collections::HashMap;
use std::sync::Arc;

mod loop_points;

pub(super) use loop_points::decorate as decorate_loop_points;

const PREMISES: usize = 8;
const SCOPES: usize = 8;
const INSTANCES: usize = 64;
const NODES: usize = 8192;
const DEPTH: usize = 64;

struct Budget {
    remaining: usize,
    depth: usize,
}

impl Budget {
    fn new() -> Self {
        Self { remaining: NODES, depth: 0 }
    }

    fn enter(&mut self) -> Result<(), ()> {
        if self.remaining == 0 || self.depth == DEPTH {
            return Err(());
        }
        self.remaining -= 1;
        self.depth += 1;
        Ok(())
    }
}

struct Inspection {
    budget: Budget,
    assertions: HashMap<sst::AssertId, usize>,
    metadata: HashMap<sst::AssertId, usize>,
}

impl Visitor<Walk, (), NoScoper> for Inspection {
    fn visit_exp(&mut self, expression: &Exp) -> Result<(), ()> {
        self.budget.enter()?;
        if let ExpX::Bind(binder, _) = &expression.x {
            if let BndX::Quant(_, _, _, Some(provenance)) = &binder.x {
                if let Some(assert_id) = &provenance.assert_id {
                    *self.metadata.entry(assert_id.clone()).or_default() += 1;
                }
            }
        }
        let result = self.visit_exp_rec(expression);
        self.budget.depth -= 1;
        result
    }

    fn visit_stm(&mut self, statement: &Stm) -> Result<(), ()> {
        self.budget.enter()?;
        if let StmX::Assert(Some(assert_id), _, _) = &statement.x {
            *self.assertions.entry(assert_id.clone()).or_default() += 1;
        }
        let result = self.visit_stm_rec(statement);
        self.budget.depth -= 1;
        result
    }
}

struct Freeze<'a> {
    budget: Budget,
    parameters: &'a HashMap<VarIdent, Typ>,
    scope: VisitorScopeMap,
}

impl Visitor<Rewrite, (), VisitorScopeMap> for Freeze<'_> {
    fn scoper(&mut self) -> Option<&mut VisitorScopeMap> {
        Some(&mut self.scope)
    }

    fn visit_exp(&mut self, expression: &Exp) -> Result<Exp, ()> {
        self.budget.enter()?;
        let result = match &expression.x {
            ExpX::Var(name) if !self.scope.contains_key(name) => match self.parameters.get(name) {
                Some(typ) if types_equal(typ, &expression.typ) => {
                    Ok(expression.new_x(ExpX::VarAt(name.clone(), VarAt::Pre)))
                }
                _ => Err(()),
            },
            ExpX::VarAt(name, VarAt::Pre) => match self.parameters.get(name) {
                Some(typ) if types_equal(typ, &expression.typ) => Ok(expression.clone()),
                _ => Err(()),
            },
            ExpX::Loc(_)
            | ExpX::VarLoc(_)
            | ExpX::Old(..)
            | ExpX::StaticVar(_)
            | ExpX::CallLambda(..)
            | ExpX::ExecFnByName(_)
            | ExpX::Interp(_)
            | ExpX::Unary(UnaryOp::MustBeFinalized | UnaryOp::MustBeElaborated, _) => Err(()),
            ExpX::Bind(binder, _) if !matches!(&binder.x, BndX::Let(_) | BndX::Quant(..)) => {
                Err(())
            }
            _ => self.visit_exp_rec(expression),
        };
        self.budget.depth -= 1;
        result
    }
}

fn checked_use(expression: Exp) -> [Stm; 2] {
    [
        Spanned::new(expression.span.clone(), StmX::Assert(None, None, expression.clone())),
        Spanned::new(expression.span.clone(), StmX::Assume(expression)),
    ]
}

struct Decorator<'a> {
    ctx: &'a Ctx,
    inspection: &'a Inspection,
    declarations: &'a HashMap<VarIdent, sst::LocalDecl>,
    premises: Vec<Exp>,
    scopes: usize,
    instances: usize,
    remaining: usize,
}

impl Decorator<'_> {
    fn local(&self, expression: &Exp) -> Option<Exp> {
        let ExpX::Bind(binder, _) = &expression.x else { return None };
        let BndX::Quant(quantifier, binders, _, Some(provenance)) = &binder.x else {
            return None;
        };
        if *quantifier != QUANT_FORALL
            || binders.len() != 1
            || provenance.locals.len() != 1
            || provenance.binders.len() != 1
        {
            return None;
        }
        let assert_id = provenance.assert_id.as_ref()?;
        if self.inspection.assertions.get(assert_id) != Some(&1)
            || self.inspection.metadata.get(assert_id) != Some(&1)
        {
            return None;
        }
        let original = &provenance.locals[0];
        if !matches!(&*original.typ, TypX::Int(_) | TypX::Bool) {
            return None;
        }
        let declared = self.declarations.get(&original.ident)?;
        let LocalDeclKind::AssertByVar { native } = declared.kind else { return None };
        if !matches!(original.kind, LocalDeclKind::AssertByVar { native: false })
            || provenance.binders[0].name != binders[0].name
            || !types_equal(&original.typ, &provenance.binders[0].a)
        {
            return None;
        }
        let converted = if native {
            crate::poly::coerce_typ_to_native(self.ctx, &original.typ)
        } else {
            crate::poly::coerce_typ_to_poly(self.ctx, &original.typ)
        };
        if !types_equal(&converted, &declared.typ) || !types_equal(&converted, &binders[0].a) {
            return None;
        }
        Some(SpannedTyped::new(&expression.span, &declared.typ, ExpX::Var(declared.ident.clone())))
    }

    fn instances(&mut self, local: &Exp) -> Vec<Stm> {
        let mut statements = Vec::new();
        for premise in &self.premises {
            if self.instances == INSTANCES {
                break;
            }
            let ExpX::Bind(binder, body) = &premise.x else { continue };
            let BndX::Quant(_, binders, _, _) = &binder.x else { continue };
            let replacement = if types_equal(&local.typ, &binders[0].a) {
                local.clone()
            } else if matches!(&*binders[0].a, TypX::Boxed(inner) if types_equal(inner, &local.typ))
            {
                crate::poly::coerce_exp_to_poly(self.ctx, local)
            } else if matches!(&*local.typ, TypX::Boxed(inner) if types_equal(inner, &binders[0].a))
            {
                crate::poly::coerce_exp_to_native(self.ctx, local)
            } else {
                continue;
            };
            if !types_equal(&replacement.typ, &binders[0].a) {
                continue;
            }
            let substitutions = HashMap::from([(binders[0].name.clone(), replacement)]);
            let instance = crate::sst_util::subst_exp(&HashMap::new(), &substitutions, body);
            let mut inspection = Inspection {
                budget: Budget { remaining: self.remaining.min(NODES), depth: 0 },
                assertions: HashMap::new(),
                metadata: HashMap::new(),
            };
            let within_budget = inspection.visit_exp(&instance).is_ok();
            self.remaining = inspection.budget.remaining;
            if !within_budget {
                break;
            }
            statements.extend(checked_use(instance));
            self.instances += 1;
        }
        statements
    }

    fn statement(&mut self, statement: &Stm) -> Stm {
        if self.remaining == 0 {
            return statement.clone();
        }
        self.remaining -= 1;
        match &statement.x {
            StmX::Block(statements) => {
                let mut matches = HashMap::new();
                for child in statements.iter() {
                    let StmX::Assume(expression) = &child.x else { continue };
                    if let Some(local) = self.local(expression) {
                        let ExpX::Bind(binder, _) = &expression.x else { continue };
                        let BndX::Quant(_, _, _, Some(provenance)) = &binder.x else {
                            continue;
                        };
                        matches.insert(provenance.assert_id.clone().unwrap(), local);
                    }
                }
                let mut rewritten = Vec::new();
                for child in statements.iter() {
                    let replacement = if self.scopes < SCOPES {
                        match &child.x {
                            StmX::DeadEnd(proof) => match &proof.x {
                                StmX::Block(body) => match body.last().map(|last| &last.x) {
                                    Some(StmX::Assert(Some(assert_id), _, _)) => {
                                        matches.get(assert_id).map(|local| {
                                            self.scopes += 1;
                                            let mut generated = body[..body.len() - 1].to_vec();
                                            generated.extend(self.instances(local));
                                            generated.push(body.last().unwrap().clone());
                                            let proof =
                                                proof.new_x(StmX::Block(Arc::new(generated)));
                                            child.new_x(StmX::DeadEnd(proof))
                                        })
                                    }
                                    _ => None,
                                },
                                _ => None,
                            },
                            _ => None,
                        }
                    } else {
                        None
                    };
                    rewritten.push(replacement.unwrap_or_else(|| self.statement(child)));
                }
                statement.new_x(StmX::Block(Arc::new(rewritten)))
            }
            StmX::If(condition, positive, negative) => statement.new_x(StmX::If(
                condition.clone(),
                self.statement(positive),
                negative.as_ref().map(|negative| self.statement(negative)),
            )),
            _ => statement.clone(),
        }
    }
}

pub(super) fn decorate(check: &FuncCheckSst, parameters: &sst::Pars, ctx: &Ctx) -> FuncCheckSst {
    let unchanged = || check.clone();
    if !matches!(check.post_condition.kind, sst::PostConditionKind::Ensures)
        || check.local_decls.len() > NODES
        || parameters.len() > NODES
    {
        return unchanged();
    }
    let mut inspection =
        Inspection { budget: Budget::new(), assertions: HashMap::new(), metadata: HashMap::new() };
    if inspection.visit_stm(&check.body).is_err() {
        return unchanged();
    }
    let mut parameter_types = HashMap::new();
    for parameter in parameters.iter() {
        if parameter_types.insert(parameter.x.name.clone(), parameter.x.typ.clone()).is_some() {
            return unchanged();
        }
    }
    let mut declarations = HashMap::new();
    for declaration in check.local_decls.iter() {
        if declarations.insert(declaration.ident.clone(), declaration.clone()).is_some() {
            return unchanged();
        }
    }
    for (name, typ) in &parameter_types {
        let Some(declared) = declarations.get(name) else { return unchanged() };
        if !matches!(declared.kind, LocalDeclKind::Param { .. }) || !types_equal(typ, &declared.typ)
        {
            return unchanged();
        }
    }
    let mut premises = Vec::new();
    for requirement in check.reqs.iter() {
        if inspection.visit_exp(requirement).is_err() {
            return unchanged();
        }
        if premises.len() == PREMISES {
            continue;
        }
        let ExpX::Bind(binder, _) = &requirement.x else { continue };
        let BndX::Quant(quantifier, binders, _, None) = &binder.x else { continue };
        if *quantifier != QUANT_FORALL || binders.len() != 1 {
            continue;
        }
        let mut freeze = Freeze {
            budget: Budget { remaining: inspection.budget.remaining, depth: 0 },
            parameters: &parameter_types,
            scope: VisitorScopeMap::new(),
        };
        if let Ok(premise) = freeze.visit_exp(requirement) {
            premises.push(premise);
        }
        inspection.budget.remaining = freeze.budget.remaining;
    }
    if premises.is_empty() {
        return unchanged();
    }
    let mut decorator = Decorator {
        ctx,
        inspection: &inspection,
        declarations: &declarations,
        premises,
        scopes: 0,
        instances: 0,
        remaining: inspection.budget.remaining,
    };
    let mut result = check.clone();
    result.body = decorator.statement(&check.body);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{VarBinderX, VarIdentDisambiguate};

    fn variable(name: &str) -> Exp {
        SpannedTyped::new(
            &super::super::tests::span(),
            &Arc::new(TypX::Int(crate::ast::IntRange::Int)),
            ExpX::Var(VarIdent(Arc::new(name.into()), VarIdentDisambiguate::AirLocal)),
        )
    }

    fn quantifier(bound: &Exp, body: Exp) -> Exp {
        let ExpX::Var(name) = &bound.x else { panic!() };
        let binders =
            Arc::new(vec![Arc::new(VarBinderX { name: name.clone(), a: bound.typ.clone() })]);
        let binder = Spanned::new(
            bound.span.clone(),
            BndX::Quant(QUANT_FORALL, binders, Arc::new(vec![]), None),
        );
        body.new_x(ExpX::Bind(binder, body.clone()))
    }

    fn inspection() -> Inspection {
        Inspection { budget: Budget::new(), assertions: HashMap::new(), metadata: HashMap::new() }
    }

    #[test]
    fn checked_assert_and_assume_share_the_exact_arc() {
        let expression = variable("proof_local");
        let [checked, used] = checked_use(expression.clone());
        let StmX::Assert(None, None, checked) = &checked.x else { panic!() };
        let StmX::Assume(used) = &used.x else { panic!() };
        assert!(Arc::ptr_eq(checked, used));
        assert!(Arc::ptr_eq(checked, &expression));
    }

    #[test]
    fn entry_freezing_uses_typed_parameters_and_not_same_spelling_locals() {
        let parameter = variable("state");
        let ExpX::Var(name) = &parameter.x else { panic!() };
        let parameters = HashMap::from([(name.clone(), parameter.typ.clone())]);
        let mut freeze = Freeze {
            budget: Budget::new(),
            parameters: &parameters,
            scope: VisitorScopeMap::new(),
        };
        let frozen = freeze.visit_exp(&parameter).unwrap();
        assert!(matches!(&frozen.x, ExpX::VarAt(actual, VarAt::Pre) if actual == name));
        let shadow =
            parameter.new_x(ExpX::Var(VarIdent(name.0.clone(), VarIdentDisambiguate::VirParam)));
        assert!(freeze.visit_exp(&shadow).is_err());
        let wrong_type =
            SpannedTyped::new(&parameter.span, &Arc::new(TypX::Bool), parameter.x.clone());
        assert!(freeze.visit_exp(&wrong_type).is_err());
    }

    #[test]
    fn freezing_keeps_bound_variables_and_existing_entry_snapshots() {
        let bound = variable("query");
        let parameter = variable("entry");
        let ExpX::Var(name) = &parameter.x else { panic!() };
        let parameters = HashMap::from([(name.clone(), parameter.typ.clone())]);
        let body =
            parameter.new_x(ExpX::Binary(sst::BinaryOp::Eq, bound.clone(), parameter.clone()));
        let mut freeze = Freeze {
            budget: Budget::new(),
            parameters: &parameters,
            scope: VisitorScopeMap::new(),
        };
        let frozen = freeze.visit_exp(&quantifier(&bound, body)).unwrap();
        let ExpX::Bind(_, body) = &frozen.x else { panic!() };
        let ExpX::Binary(_, left, right) = &body.x else { panic!() };
        assert!(matches!(left.x, ExpX::Var(_)));
        assert!(matches!(right.x, ExpX::VarAt(_, VarAt::Pre)));
        let entry = parameter.new_x(ExpX::VarAt(name.clone(), VarAt::Pre));
        assert!(Arc::ptr_eq(&freeze.visit_exp(&entry).unwrap(), &entry));
    }

    #[test]
    fn mutable_locations_and_nonentry_snapshots_are_declined() {
        let variable = variable("state");
        let ExpX::Var(name) = &variable.x else { panic!() };
        let parameters = HashMap::from([(name.clone(), variable.typ.clone())]);
        let mut freeze = Freeze {
            budget: Budget::new(),
            parameters: &parameters,
            scope: VisitorScopeMap::new(),
        };
        assert!(freeze.visit_exp(&variable.new_x(ExpX::VarLoc(name.clone()))).is_err());
        assert!(
            freeze
                .visit_exp(&variable.new_x(ExpX::Old(Arc::new("late".into()), name.clone())))
                .is_err()
        );
        assert!(freeze.visit_exp(&variable.new_x(ExpX::Loc(variable.clone()))).is_err());
    }

    #[test]
    fn substitution_avoids_capture_at_the_actual_proof_local() {
        let bound = variable("binder");
        let local = variable("proof_local");
        let ExpX::Var(bound_name) = &bound.x else { panic!() };
        let ExpX::Var(local_name) = &local.x else { panic!() };
        let body = local.new_x(ExpX::Binary(sst::BinaryOp::Eq, bound.clone(), local.clone()));
        let nested = quantifier(&local, body);
        let result = crate::sst_util::subst_exp(
            &HashMap::new(),
            &HashMap::from([(bound_name.clone(), local.clone())]),
            &nested,
        );
        let ExpX::Bind(binder, body) = &result.x else { panic!() };
        let BndX::Quant(_, binders, _, _) = &binder.x else { panic!() };
        assert_ne!(&binders[0].name, local_name);
        let ExpX::Binary(_, left, right) = &body.x else { panic!() };
        assert!(matches!(&left.x, ExpX::Var(actual) if actual == local_name));
        assert!(matches!(&right.x, ExpX::Var(actual) if actual == &binders[0].name));
    }

    #[test]
    fn depth_nodes_and_partial_traversal_are_bounded() {
        let expression = variable("query");
        let mut nested = expression.clone();
        for _ in 0..DEPTH {
            nested = expression.new_x(ExpX::Unary(UnaryOp::Not, nested));
        }
        assert!(inspection().visit_exp(&nested).is_err());
        let statements = (0..NODES)
            .map(|_| Spanned::new(expression.span.clone(), StmX::Assume(expression.clone())))
            .collect();
        let statement = Spanned::new(expression.span.clone(), StmX::Block(Arc::new(statements)));
        assert!(inspection().visit_stm(&statement).is_err());
        let mut budget = Budget { remaining: 0, depth: 0 };
        assert!(budget.enter().is_err());
        assert_eq!(budget.depth, 0);
        assert_eq!(PREMISES * SCOPES, INSTANCES);
    }

    #[test]
    fn duplicate_exact_assert_ids_are_visible_not_guessed_from_deadends() {
        let expression = variable("query");
        let assert_id = Arc::new(vec![7]);
        let assertion = Spanned::new(
            expression.span.clone(),
            StmX::Assert(Some(assert_id.clone()), None, expression),
        );
        let statement =
            assertion.new_x(StmX::Block(Arc::new(vec![assertion.clone(), assertion.clone()])));
        let mut inspection = inspection();
        inspection.visit_stm(&statement).unwrap();
        assert_eq!(inspection.assertions[&assert_id], 2);
        assert!(inspection.metadata.is_empty());
    }
}
