use super::*;
use crate::infer::tests::{function, input, span};

fn path(krate: CrateId, segments: &[&str]) -> Path {
    Arc::new(PathX {
        krate,
        segments: Arc::new(segments.iter().map(|segment| Arc::new((*segment).into())).collect()),
    })
}

fn literal_model() -> Function {
    let fixture = function("fixture", input("x"));
    let mut definition = fixture.x.clone();
    definition.name = Arc::new(FunX { path: path(CrateId::Core, &["panicking", "panic"]) });
    definition.proxy = Some(crate::def::Spanned {
        span: span(),
        x: path(CrateId::Vstd, &["std_specs", "core", "_verus_external_fn_specification_0_panic"]),
    });
    definition.body = None;
    Arc::make_mut(&mut definition.attrs).infer = false;
    let string = models::reference(models::primitive(Primitive::StrSlice, vec![]));
    let mut parameter = definition.params[0].x.clone();
    parameter.typ = string;
    definition.params = Arc::new(vec![definition.params[0].new_x(parameter)]);
    let never = Arc::new(TypX::Decorate(TypDecoration::Never, None, crate::ast_util::unit_typ()));
    let mut result = definition.inner_ret.x.clone();
    result.typ = never;
    definition.inner_ret = definition.inner_ret.new_x(result.clone());
    definition.outer_ret = definition.outer_ret.new_x(result);
    definition.require = Arc::new(vec![boolean(&span(), false).expr]);
    fixture.new_x(definition)
}

fn invocation(model: &Function, argument: Expr) -> Expr {
    SpannedTyped::new(
        &span(),
        &model.x.inner_ret.x.typ,
        ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::Static,
                model.x.name.clone(),
                Arc::new(vec![]),
                Arc::new(vec![]),
                CallTargetAttrs {
                    autospec: AutospecUsage::Final,
                    const_var: false,
                    assume_external_allowed: false,
                },
            ),
            args: Arc::new(vec![argument]),
            post_args: None,
            body: None,
        },
    )
}

#[test]
fn canonical_literal_panic_requires_exact_identity_signature_and_false_domain() {
    let original = literal_model();
    assert!(model(&original) == Some(Model::LiteralPanic));
    for fault in 0..10 {
        let mut definition = original.x.clone();
        match fault {
            0 => {
                definition.name =
                    Arc::new(FunX { path: path(CrateId::Vstd, &["panicking", "panic"]) })
            }
            1 => {
                definition.proxy.as_mut().unwrap().x = path(
                    CrateId::Core,
                    &["std_specs", "core", "_verus_external_fn_specification_0_panic"],
                )
            }
            2 => definition.require = Arc::new(vec![boolean(&span(), true).expr]),
            3 => definition.body = Some(input("x")),
            4 => Arc::make_mut(&mut definition.attrs).infer = true,
            5 => Arc::make_mut(&mut definition.attrs).is_unsafe = true,
            6 => definition.mode = Mode::Spec,
            7 => definition.ensure.0 = Arc::new(vec![boolean(&span(), false).expr]),
            8 => {
                let mut parameter = definition.params[0].x.clone();
                parameter.user_mut = true;
                definition.params = Arc::new(vec![definition.params[0].new_x(parameter)]);
            }
            9 => {
                let mut result = definition.inner_ret.x.clone();
                result.typ = bool_typ();
                definition.inner_ret = definition.inner_ret.new_x(result.clone());
                definition.outer_ret = definition.outer_ret.new_x(result);
            }
            _ => unreachable!(),
        }
        assert!(model(&original.new_x(definition)).is_none(), "fault {fault}");
    }
}

#[test]
fn literal_panic_never_discards_dynamic_or_eager_message_evaluation() {
    let model = literal_model();
    let all = HashMap::from([(model.x.name.clone(), model.clone())]);
    let message = SpannedTyped::new(
        &span(),
        &model.x.params[0].x.typ,
        ExprX::Const(Constant::StrSlice(Arc::new("literal".into()))),
    );
    let call = invocation(&model, message.clone());
    assert_eq!(literal(&call, &all).unwrap().len(), 1);
    assert!(is_panic_maximality_model(&model, &Arc::new(vec![])));
    let dynamic = message
        .new_x(ExprX::Var(VarIdent(Arc::new("message".into()), VarIdentDisambiguate::AirLocal)));
    assert!(literal(&invocation(&model, dynamic), &all).is_none());
    let eager = message.new_x(ExprX::Block(
        Arc::new(vec![crate::def::Spanned::new(span(), StmtX::Expr(input("x")))]),
        Some(message.clone()),
    ));
    assert!(literal(&invocation(&model, eager), &all).is_none());
    let wrong_type = SpannedTyped::new(&span(), &bool_typ(), message.x.clone());
    assert!(literal(&invocation(&model, wrong_type), &all).is_none());
}
