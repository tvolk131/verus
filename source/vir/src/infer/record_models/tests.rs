use super::*;
use crate::infer::tests::{function, input};

fn then_some() -> Function {
    let mut definition = function("then_some", input("x")).x.clone();
    definition.name = Arc::new(FunX {
        path: Arc::new(PathX {
            krate: CrateId::Vstd,
            segments: Arc::new(vec![
                Arc::new("native_option".into()),
                Arc::new("then_some".into()),
            ]),
        }),
    });
    let parameter: Ident = Arc::new("Value".into());
    let value = Arc::new(TypX::TypParam(parameter.clone()));
    definition.typ_params = Arc::new(vec![parameter]);
    let mut condition = definition.params[0].x.clone();
    condition.typ = bool_typ();
    let mut argument = definition.params[1].x.clone();
    argument.typ = value.clone();
    definition.params =
        Arc::new(vec![definition.params[0].new_x(condition), definition.params[1].new_x(argument)]);
    let result = Arc::new(TypX::Datatype(
        Dt::Path(Arc::new(PathX {
            krate: CrateId::Core,
            segments: Arc::new(vec![Arc::new("option".into()), Arc::new("Option".into())]),
        })),
        Arc::new(vec![value]),
        Arc::new(vec![]),
    ));
    let mut inner = definition.inner_ret.x.clone();
    inner.typ = result.clone();
    definition.inner_ret = definition.inner_ret.new_x(inner);
    let mut outer = definition.outer_ret.x.clone();
    outer.typ = result;
    definition.outer_ret = definition.outer_ret.new_x(outer);
    definition.attrs = Arc::new(FunctionAttrsX { infer: false, ..(*definition.attrs).clone() });
    definition.ensure = (Arc::new(vec![input("x")]), Arc::new(vec![]));
    function("fixture", input("x")).new_x(definition)
}

#[test]
fn bridge_identity_proxy_modes_and_types_fail_closed() {
    let original = then_some();
    assert_eq!(native_option(&original), Some(NativeOption::ThenSome));
    for variant in 0..9 {
        let mut definition = original.x.clone();
        match variant {
            0 => {
                let mut path = definition.name.path.as_ref().clone();
                path.krate = CrateId::Core;
                definition.name = Arc::new(FunX { path: Arc::new(path) });
            }
            1 => {
                definition.proxy = Some(crate::def::Spanned {
                    span: original.span.clone(),
                    x: definition.name.path.clone(),
                })
            }
            2 => definition.mode = Mode::Spec,
            3 => definition.require = Arc::new(vec![input("x")]),
            4 => definition.ensure = (Arc::new(vec![]), Arc::new(vec![])),
            5 => definition.params = Arc::new(vec![]),
            6 => definition.typ_params = Arc::new(vec![]),
            7 => {
                let mut result = definition.inner_ret.x.clone();
                result.typ = bool_typ();
                definition.inner_ret = definition.inner_ret.new_x(result);
            }
            8 => {
                definition.attrs = Arc::new(FunctionAttrsX {
                    is_external_body: true,
                    ..(*definition.attrs).clone()
                })
            }
            _ => unreachable!(),
        }
        assert_eq!(native_option(&original.new_x(definition)), None, "variant {variant}");
    }
}

#[test]
fn callback_precision_and_domain_facts_share_an_exact_only_gate() {
    let original = function("callback", input("number"));
    for owned in [false, true] {
        for approximate in [false, true] {
            for inferred in [false, true] {
                let mut definition = original.x.clone();
                let mut attributes = definition.attrs.as_ref().clone();
                attributes.infer = inferred;
                attributes.infer_owned = owned;
                attributes.infer_allow_approx = approximate;
                definition.attrs = Arc::new(attributes);
                assert_eq!(
                    checks_exact_callbacks(&original.new_x(definition)),
                    inferred && !approximate
                );
            }
        }
    }
}
