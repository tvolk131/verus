use crate::ast::{Function, Krate, Mode, VirErr};
use crate::messages::error;

const MARKER_PREFIX: &str = "experimental-native-resource-ir:";

pub mod adapter164;

#[cfg(feature = "native-semantics170-inspect")]
pub mod kernel170;

#[cfg(feature = "native-resource-continuation160")]
pub mod continuation160;

#[cfg(feature = "experimental-native-resource")]
pub mod native;

pub fn is_marked(function: &Function) -> bool {
    function.x.attrs.infer_candidate_id.as_deref().is_some_and(|id| id.starts_with(MARKER_PREFIX))
        || adapter164::function_contains_native_call(function)
}

pub fn reject_crate(krate: &Krate, operation: &str) -> Result<(), VirErr> {
    if let Some(function) = krate.functions.iter().find(|f| is_marked(f)) {
        return Err(error(
            &function.span,
            format!(
                "experimental native-resource IR: {operation} is unavailable; conditional IR transport is not native correspondence"
            ),
        ));
    }
    Ok(())
}

pub(crate) fn check_lowering(function: &Function, check_api_safety: bool) -> Result<(), VirErr> {
    if is_marked(function)
        && (!cfg!(feature = "experimental-native-resource")
            || function.x.mode != Mode::Exec
            || check_api_safety)
    {
        return Err(error(
            &function.span,
            "experimental native-resource IR requires the default-off feature and an internal exec-body check",
        ));
    }
    Ok(())
}

#[cfg(feature = "experimental-native-resource")]
pub mod affine {
    use super::*;
    use crate::ast::*;
    use crate::ast_util::{bool_typ, int_typ, mk_eq, unit_typ};
    use crate::def::Spanned;
    use crate::messages::Span;
    use std::sync::Arc;

    pub const DESCRIPTOR_FIELDS: usize = 10;
    pub type Descriptor = [VarIdent; DESCRIPTOR_FIELDS];

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum Slot {
        Input,
        Output,
    }

    pub struct Event {
        pub span: Span,
        pub kind: EventKind,
    }

    pub enum EventKind {
        Transfer { source: Slot, destination: Slot, expected: Descriptor },
        Use { slot: Slot, expected: Descriptor },
        Consume { slot: Slot, expected: Descriptor },
        If { condition: VarIdent, then_events: Vec<Event>, else_events: Vec<Event> },
        Return,
    }

    pub struct Program {
        pub incoming: Descriptor,
        pub frame: VarIdent,
        pub events: Vec<Event>,
        pub returned_slot: Option<Slot>,
    }

    fn local(serial: u64) -> VarIdent {
        VarIdent(Arc::new("affine".into()), VarIdentDisambiguate::NativeResource(serial))
    }

    fn live(slot: Slot) -> VarIdent {
        local(match slot {
            Slot::Input => 0,
            Slot::Output => 1,
        })
    }

    fn expression(span: &Span, typ: &Typ, kind: ExprX) -> Expr {
        SpannedTyped::new(span, typ, kind)
    }

    fn variable(span: &Span, name: &VarIdent, typ: &Typ) -> Expr {
        expression(span, typ, ExprX::Var(name.clone()))
    }

    fn boolean(span: &Span, value: bool) -> Expr {
        expression(span, &bool_typ(), ExprX::Const(Constant::Bool(value)))
    }

    fn block(span: &Span, statements: Vec<Stmt>) -> Expr {
        expression(span, &unit_typ(), ExprX::Block(Arc::new(statements), None))
    }

    fn statement(expr: Expr) -> Stmt {
        Spanned::new(expr.span.clone(), StmtX::Expr(expr))
    }

    fn assertion(span: &Span, predicate: Expr, label: &str) -> Stmt {
        statement(expression(
            span,
            &unit_typ(),
            ExprX::AssertAssume {
                is_assume: false,
                expr: predicate,
                msg: Some(error(span, label)),
            },
        ))
    }

    fn declaration(span: &Span, name: VarIdent, initial: Expr, mutable: bool) -> Stmt {
        let pattern = SpannedTyped::new(
            span,
            &initial.typ,
            PatternX::Var(PatternBinding {
                name,
                by_ref: ByRef::No,
                typ: initial.typ.clone(),
                user_mut: Some(mutable),
                copy: true,
            }),
        );
        let initial = SpannedTyped::new(span, &initial.typ, PlaceX::Temporary(initial.clone()));
        Spanned::new(
            span.clone(),
            StmtX::Decl {
                pattern,
                mode: Some((Mode::Exec, DeclProph::Default)),
                init: Some(initial),
                els: None,
                assert_irrefutable: true,
            },
        )
    }

    fn assign(span: &Span, slot: Slot, value: bool) -> Stmt {
        statement(expression(
            span,
            &unit_typ(),
            ExprX::Assign {
                place: SpannedTyped::new(span, &bool_typ(), PlaceX::Local(live(slot))),
                rhs: boolean(span, value),
                op: None,
                typ: bool_typ(),
                resolve: false,
            },
        ))
    }

    fn check_parameter(function: &Function, name: &VarIdent, typ: &Typ) -> Result<(), VirErr> {
        if function.x.params.iter().any(|param| {
            param.x.name == *name
                && param.x.mode == Mode::Exec
                && !param.x.user_mut
                && crate::ast_util::types_equal(&param.x.typ, typ)
        }) {
            Ok(())
        } else {
            Err(error(
                &function.span,
                "native-resource IR operand must be an immutable typed input",
            ))
        }
    }

    fn check_descriptor(function: &Function, descriptor: &Descriptor) -> Result<(), VirErr> {
        for field in descriptor {
            check_parameter(function, field, &int_typ())?;
        }
        Ok(())
    }

    fn check_authority(
        function: &Function,
        program: &Program,
        span: &Span,
        slot: Slot,
        expected: &Descriptor,
        statements: &mut Vec<Stmt>,
    ) -> Result<(), VirErr> {
        check_descriptor(function, expected)?;
        statements.push(assertion(
            span,
            variable(span, &live(slot), &bool_typ()),
            "NRE100_SOURCE_LIVE",
        ));
        for (incoming, expected) in program.incoming.iter().zip(expected) {
            statements.push(assertion(
                span,
                mk_eq(
                    &span,
                    &variable(span, incoming, &int_typ()),
                    &variable(span, expected, &int_typ()),
                ),
                "NRE102_DESCRIPTOR_IDENTITY",
            ));
        }
        Ok(())
    }

    fn exit(program: &Program, span: &Span) -> Vec<Stmt> {
        let mut statements = Vec::new();
        for slot in [Slot::Input, Slot::Output] {
            statements.push(assertion(
                span,
                mk_eq(
                    span,
                    &variable(span, &live(slot), &bool_typ()),
                    &boolean(span, program.returned_slot == Some(slot)),
                ),
                "NRE103_EXIT_SHAPE",
            ));
        }
        statements.push(assertion(
            span,
            mk_eq(
                span,
                &variable(span, &program.frame, &int_typ()),
                &variable(span, &local(2), &int_typ()),
            ),
            "NRE104_SYMBOLIC_FRAME",
        ));
        statements
    }

    fn lower_events(
        function: &Function,
        program: &Program,
        events: &[Event],
        depth: usize,
        remaining: &mut usize,
    ) -> Result<Vec<Stmt>, VirErr> {
        if depth > 32 || events.len() > *remaining {
            return Err(error(&function.span, "native-resource IR structural limit exceeded"));
        }
        *remaining -= events.len();
        let mut statements = Vec::new();
        for event in events {
            let span = &event.span;
            match &event.kind {
                EventKind::Use { slot, expected } | EventKind::Consume { slot, expected } => {
                    check_authority(function, program, span, *slot, expected, &mut statements)?;
                    if matches!(event.kind, EventKind::Consume { .. }) {
                        statements.push(assign(span, *slot, false));
                    }
                }
                EventKind::Transfer { source, destination, expected } => {
                    check_authority(function, program, span, *source, expected, &mut statements)?;
                    statements.push(assertion(
                        span,
                        boolean(span, source != destination),
                        "NRE105_DISTINCT_SLOTS",
                    ));
                    statements.push(assertion(
                        span,
                        expression(
                            span,
                            &bool_typ(),
                            ExprX::Unary(
                                UnaryOp::Not,
                                variable(span, &live(*destination), &bool_typ()),
                            ),
                        ),
                        "NRE101_DESTINATION_EMPTY",
                    ));
                    statements.push(assign(span, *source, false));
                    statements.push(assign(span, *destination, true));
                }
                EventKind::If { condition, then_events, else_events } => {
                    check_parameter(function, condition, &bool_typ())?;
                    let then_body = block(
                        span,
                        lower_events(function, program, then_events, depth + 1, remaining)?,
                    );
                    let else_body = block(
                        span,
                        lower_events(function, program, else_events, depth + 1, remaining)?,
                    );
                    statements.push(statement(expression(
                        span,
                        &unit_typ(),
                        ExprX::If(
                            variable(span, condition, &bool_typ()),
                            then_body,
                            Some(else_body),
                        ),
                    )));
                }
                EventKind::Return => {
                    statements.extend(exit(program, span));
                    statements.push(statement(expression(span, &unit_typ(), ExprX::Return(None))));
                }
            }
        }
        Ok(statements)
    }

    pub fn instrument(function: &Function, program: &Program) -> Result<Function, VirErr> {
        let empty_body = function.x.body.as_ref().is_some_and(
            |body| matches!(&body.x, ExprX::Block(statements, None) if statements.is_empty()),
        );
        if function.x.mode != Mode::Exec
            || !matches!(function.x.kind, FunctionKind::Static)
            || !empty_body
            || !crate::ast_util::is_unit(&function.x.inner_ret.x.typ)
            || !function.x.typ_params.is_empty()
            || function.x.attrs.infer
            || function.x.attrs.is_external_body
            || function.x.attrs.infer_candidate_id.is_some()
            || function
                .x
                .params
                .iter()
                .any(|param| matches!(param.x.name.1, VarIdentDisambiguate::NativeResource(_)))
        {
            return Err(error(
                &function.span,
                "native-resource IR requires an internal empty exec fixture",
            ));
        }
        check_descriptor(function, &program.incoming)?;
        check_parameter(function, &program.frame, &int_typ())?;
        let span = &function.span;
        let mut statements = vec![
            declaration(span, live(Slot::Input), boolean(span, true), true),
            declaration(span, live(Slot::Output), boolean(span, false), true),
            declaration(span, local(2), variable(span, &program.frame, &int_typ()), false),
        ];
        statements.extend(lower_events(function, program, &program.events, 0, &mut 256)?);
        statements.extend(exit(program, span));
        let mut definition = function.x.clone();
        let mut attributes = (*definition.attrs).clone();
        attributes.infer_candidate_id = Some(format!("{MARKER_PREFIX}v1:affine"));
        definition.attrs = Arc::new(attributes);
        definition.body = Some(block(span, statements));
        Ok(function.new_x(definition))
    }
}

#[cfg(test)]
mod tests;
