//! Native fixed-eight-byte little-endian APIs. Only canonical core identities
//! with the matching vstd proxy and concrete value signatures enter this model.
//! Generated array/bitwise relations remain subject to all exact-mode checks.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Model {
    ToLe,
    FromLe,
}

fn word() -> Typ {
    Arc::new(TypX::Int(IntRange::U(64)))
}

fn bytes() -> Typ {
    arrays::typ(Arc::new(TypX::Int(IntRange::U(8))), 8).unwrap()
}

pub(super) fn recognize(function: &Function) -> Option<Model> {
    let function_data = &function.x;
    let function_path = &function_data.name.path;
    if function_path.krate != CrateId::Core
        || function_path.segments.len() != 3
        || function_path.segments[0].as_str() != "num"
        || !function_path.segments[1].starts_with("impl")
        || function_data.mode != Mode::Exec
        || !matches!(function_data.kind, FunctionKind::Static)
        || !matches!(function_data.item_kind, ItemKind::Function)
        || !function_data.typ_params.is_empty()
        || !function_data.typ_bounds.is_empty()
        || function_data.params.len() != 1
        || function_data.params[0].x.mode != Mode::Exec
        || function_data.params[0].x.user_mut
        || function_data.inner_ret.x.mode != Mode::Exec
        || function_data.outer_ret.x.mode != Mode::Exec
        || !function_data.require.is_empty()
        || !matches!(function_data.unwind_spec, Some(UnwindSpec::NoUnwind))
        || function_data.attrs.is_unsafe
        || function_data.attrs.is_async
        || function_data.attrs.infer
        || function_data.body.is_some()
    {
        return None;
    }
    let (model, input, output, suffix) = match function_path.segments[2].as_str() {
        "to_le_bytes" => (Model::ToLe, word(), bytes(), "_to__le__bytes"),
        "from_le_bytes" => (Model::FromLe, bytes(), word(), "_from__le__bytes"),
        _ => return None,
    };
    if !models::spec_proxy(&function_data.proxy.as_ref()?.x, &["bytes"], suffix)
        || !crate::ast_util::types_equal(&function_data.params[0].x.typ, &input)
        || !crate::ast_util::types_equal(&function_data.inner_ret.x.typ, &output)
        || !crate::ast_util::types_equal(&function_data.outer_ret.x.typ, &output)
    {
        return None;
    }
    Some(model)
}

pub(super) fn source_call<'a>(
    expression: &Expr,
    functions: &'a HashMap<Fun, Function>,
) -> Option<&'a Function> {
    let ExprX::Call {
        target: CallTarget::Fun(CallTargetKind::Static, name, types, implementations, _),
        args,
        post_args: None,
        body: None,
    } = &expression.x
    else {
        return None;
    };
    let function = functions.get(name).filter(|function| recognize(function).is_some())?;
    (types.is_empty()
        && implementations.is_empty()
        && args.len() == 1
        && crate::ast_util::types_equal(&args[0].typ, &function.x.params[0].x.typ)
        && crate::ast_util::types_equal(&expression.typ, &function.x.inner_ret.x.typ))
    .then_some(function)
}

pub(super) fn value(
    span: &Span,
    function_data: &Function,
    args: Vec<Term>,
) -> Result<Term, VirErr> {
    let model = recognize(function_data)
        .ok_or_else(|| unsupported(span, "unapproved byte conversion model"))?;
    if args.len() != 1
        || !(crate::ast_util::types_equal(&args[0].expr.typ, &function_data.x.params[0].x.typ)
            || model == Model::ToLe && matches!(&*args[0].expr.typ, TypX::Int(IntRange::Int)))
        || args[0].view.is_some()
    {
        return Err(unsupported(span, "byte conversion argument types or count"));
    }
    let number = |number: usize| term(span, &word(), ExprX::Const(Constant::Int(number.into())), 1);
    let binary = |op, left, right| {
        summary::binary(span, BinaryOp::Bitwise(op, BitshiftBehavior::Allow), left, right)
    };
    let arg = if model == Model::ToLe {
        bitwise::coerce(span, &word(), args[0].clone())?
    } else {
        args[0].clone()
    };
    match model {
        Model::ToLe => {
            let mut elements = Vec::new();
            for index in 0..8 {
                let shifted = if index == 0 {
                    arg.clone()
                } else {
                    binary(BitwiseOp::Shr, arg.clone(), number(index * 8)?)?
                };
                let masked = binary(BitwiseOp::BitAnd, shifted, number(255)?)?;
                elements.push(summary::unary(
                    span,
                    UnaryOp::Clip { range: IntRange::U(8), truncate: true },
                    masked,
                )?);
            }
            arrays::literal(span, &bytes(), elements)
        }
        Model::FromLe => {
            let mut output = None;
            for index in 0..8 {
                let array_index = term(
                    span,
                    &Arc::new(TypX::Int(IntRange::USize)),
                    ExprX::Const(Constant::Int(index.into())),
                    1,
                )?;
                let byte = arrays::index(span, arg.clone(), array_index)?;
                let byte = bitwise::coerce(span, &word(), byte)?;
                let shifted = if index == 0 {
                    byte
                } else {
                    binary(
                        BitwiseOp::Shl(IntegerTypeBitwidth::Width(64), false),
                        byte,
                        number(index * 8)?,
                    )?
                };
                output = Some(match output {
                    None => shifted,
                    Some(previous) => binary(BitwiseOp::BitOr, previous, shifted)?,
                });
            }
            Ok(output.unwrap())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{Value, evaluate_with, function, input, span};

    fn path(krate: CrateId, parts: &[&str]) -> Path {
        Arc::new(PathX {
            krate,
            segments: Arc::new(parts.iter().map(|segment| Arc::new((*segment).into())).collect()),
        })
    }

    fn model(kind: Model) -> Function {
        let original = function("model", input("x"));
        let mut function_data = original.x.clone();
        let (name, proxy, input, output) = match kind {
            Model::ToLe => ("to_le_bytes", "_to__le__bytes", word(), bytes()),
            Model::FromLe => ("from_le_bytes", "_from__le__bytes", bytes(), word()),
        };
        function_data.name =
            Arc::new(FunX { path: path(CrateId::Core, &["num", "impl_test", name]) });
        function_data.proxy = Some(crate::def::Spanned {
            span: span(),
            x: path(CrateId::Vstd, &["bytes", &format!("_verus_external_fn_specification{proxy}")]),
        });
        let mut parameter = function_data.params[0].x.clone();
        parameter.typ = input;
        function_data.params = Arc::new(vec![function_data.params[0].new_x(parameter)]);
        let mut ret = function_data.inner_ret.x.clone();
        ret.typ = output;
        function_data.inner_ret = function_data.inner_ret.new_x(ret);
        function_data.outer_ret = function_data.inner_ret.clone();
        function_data.body = None;
        function_data.attrs =
            Arc::new(FunctionAttrsX { is_external_body: true, ..FunctionAttrsX::default() });
        function_data.unwind_spec = Some(UnwindSpec::NoUnwind);
        original.new_x(function_data)
    }

    #[test]
    fn identity_signature_and_policy_are_closed() {
        for kind in [Model::ToLe, Model::FromLe] {
            let model = model(kind);
            assert_eq!(recognize(&model), Some(kind));
            for mutation in 0..10 {
                let mut function_data = model.x.clone();
                match mutation {
                    0 => {
                        function_data.name = Arc::new(FunX {
                            path: path(
                                CrateId::Id(Arc::new("lookalike".into()), 1),
                                &["num", "impl_test", "to_le_bytes"],
                            ),
                        })
                    }
                    1 => function_data.proxy = None,
                    2 => function_data.unwind_spec = None,
                    3 => function_data.require = Arc::new(vec![boolean(&span(), false).expr]),
                    4 => {
                        let mut parameter = function_data.params[0].x.clone();
                        parameter.typ = Arc::new(TypX::Int(IntRange::U(32)));
                        function_data.params =
                            Arc::new(vec![function_data.params[0].new_x(parameter)]);
                    }
                    5 => {
                        let mut parameter = function_data.params[0].x.clone();
                        parameter.typ = models::reference(parameter.typ);
                        function_data.params =
                            Arc::new(vec![function_data.params[0].new_x(parameter)]);
                    }
                    6 => {
                        let mut ret = function_data.inner_ret.x.clone();
                        ret.typ = arrays::typ(Arc::new(TypX::Int(IntRange::U(8))), 4).unwrap();
                        function_data.inner_ret = function_data.inner_ret.new_x(ret);
                    }
                    7 => Arc::make_mut(&mut function_data.attrs).is_unsafe = true,
                    8 => function_data.typ_params = Arc::new(vec![Arc::new("T".into())]),
                    _ => function_data.body = Some(input("x")),
                }
                assert!(recognize(&model.new_x(function_data)).is_none(), "{kind:?} {mutation}");
            }
        }
    }

    #[test]
    fn generated_values_match_native_rust_at_every_byte_boundary() {
        let to = model(Model::ToLe);
        let from = model(Model::FromLe);
        let arg = |typ: Typ| Term {
            expr: SpannedTyped::new(&span(), &typ, input("x").x.clone()),
            size: 1,
            view: None,
        };
        let encoded = value(&span(), &to, vec![arg(word())]).unwrap();
        let decoded = value(&span(), &from, vec![arg(bytes())]).unwrap();
        let array = |xs: [u8; 8]| {
            Value::Array(xs.into_iter().map(|byte| Value::Int(byte.into())).collect())
        };
        for position in 0..8 {
            for byte in 0..=255u64 {
                for background in [0, u64::MAX, 0x0123456789abcdef] {
                    let number = (background & !(255 << (position * 8))) | (byte << (position * 8));
                    let expected = number.to_le_bytes();
                    assert_eq!(
                        evaluate_with(&encoded.expr, &|_| Value::Int(number.into())),
                        array(expected)
                    );
                    assert_eq!(
                        evaluate_with(&decoded.expr, &|_| array(expected)),
                        Value::Int(u64::from_le_bytes(expected).into())
                    );
                }
            }
        }
        assert!(value(&span(), &to, vec![arg(bytes())]).is_err());
        assert!(value(&span(), &from, vec![arg(word())]).is_err());
    }
}
