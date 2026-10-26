use super::*;
use crate::infer::tests::{Value, evaluate_with, function, input, span};

fn reference(t: Typ) -> Typ {
    Arc::new(TypX::Decorate(TypDecoration::Ref, None, t))
}
fn slice(t: Typ) -> Typ {
    reference(Arc::new(TypX::Primitive(Primitive::Slice, Arc::new(vec![t]))))
}

pub(in crate::infer) fn model_function(index: bool) -> Function {
    let base = function("model", input("x"));
    let mut f = base.x.clone();
    let t = Arc::new(TypX::TypParam(Arc::new("T".into())));
    f.name = if index { crate::def::fn_slice_index() } else { crate::def::fn_slice_len() };
    f.mode = Mode::Spec;
    f.body = None;
    f.attrs = Arc::new(FunctionAttrsX::default());
    f.typ_params = Arc::new(vec![Arc::new("T".into())]);
    f.typ_bounds = Arc::new(vec![Arc::new(GenericBoundX::Trait(
        TraitId::Sizedness(Sizedness::Sized),
        Arc::new(vec![t.clone()]),
    ))]);
    let types = if index { vec![slice(t.clone()), int_typ()] } else { vec![slice(t.clone())] };
    f.params = Arc::new(
        f.params
            .iter()
            .zip(types)
            .map(|(p, typ)| {
                let mut param = p.x.clone();
                param.typ = typ;
                param.mode = Mode::Spec;
                p.new_x(param)
            })
            .collect(),
    );
    let mut ret = f.inner_ret.x.clone();
    ret.mode = Mode::Spec;
    ret.typ = if index { t } else { Arc::new(TypX::Int(IntRange::USize)) };
    f.inner_ret = f.inner_ret.new_x(ret);
    f.outer_ret = f.inner_ret.clone();
    base.new_x(f)
}

pub(in crate::infer) fn slice_function(signed: bool) -> Function {
    let typ = Arc::new(TypX::Int(if signed { IntRange::I(8) } else { IntRange::U(8) }));
    let container = slice(typ.clone());
    let index_type = Arc::new(TypX::Int(IntRange::USize));
    let base = SpannedTyped::new(
        &span(),
        &container,
        PlaceX::Local(VarIdent(Arc::new("x".into()), VarIdentDisambiguate::AirLocal)),
    );
    let index = SpannedTyped::new(&span(), &index_type, input("y").x.clone());
    let place = SpannedTyped::new(
        &span(),
        &typ,
        PlaceX::Index(base, index, ArrayKind::Slice, BoundsCheck::Error),
    );
    let read = SpannedTyped::new(
        &span(),
        &typ,
        ExprX::ReadPlace(place, UnfinalizedReadKind { preliminary_kind: ReadKind::Copy, id: 0 }),
    );
    let number = SpannedTyped::new(
        &span(),
        &typ,
        ExprX::Const(Constant::Int(if signed { 0.into() } else { 1.into() })),
    );
    let (op, a, b) = if signed {
        (ArithOp::Sub(OverflowBehavior::Error(IntRange::I(8))), number, read)
    } else {
        (ArithOp::Add(OverflowBehavior::Error(IntRange::U(8))), read, number)
    };
    let body = SpannedTyped::new(&span(), &typ, ExprX::Binary(BinaryOp::Arith(op), a, b));
    let base = function("slice_at", body);
    let mut f = base.x.clone();
    f.params = Arc::new(
        f.params
            .iter()
            .zip([container, index_type])
            .map(|(p, typ)| {
                let mut param = p.x.clone();
                param.typ = typ;
                p.new_x(param)
            })
            .collect(),
    );
    let mut ret = f.inner_ret.x.clone();
    ret.typ = typ;
    f.inner_ret = f.inner_ret.new_x(ret);
    f.outer_ret = f.inner_ret.clone();
    base.new_x(f)
}

#[test]
fn exact_slice_domains_and_outputs_match_concrete_signed_and_unsigned_reads() {
    for signed in [false, true] {
        let inferred = infer_function(&slice_function(signed)).unwrap();
        let ExprX::Binary(BinaryOp::Eq(_), _, output) = &inferred.x.ensure.0[0].x else { panic!() };
        let alphabet: Vec<i128> =
            if signed { vec![-128, -127, -1, 0, 1, 127] } else { vec![0, 1, 254, 255] };
        let mut vectors = vec![vec![]];
        for n in 1..=3u32 {
            for mut encoding in 0..alphabet.len().pow(n) {
                let mut values = vec![];
                for _ in 0..n {
                    values.push(alphabet[encoding % alphabet.len()]);
                    encoding /= alphabet.len();
                }
                vectors.push(values);
            }
        }
        vectors.extend(if signed {
            (-128..=127).map(|n| vec![n]).collect::<Vec<_>>()
        } else {
            (0..=255).map(|n| vec![n]).collect()
        });
        vectors.push(vec![1; 300]);
        for values in vectors {
            for index in [0, 1, 2, 3, 64, 299, 300, usize::MAX] {
                let expected = values.get(index).and_then(|n| {
                    if signed {
                        (*n as i8).checked_neg().map(i128::from)
                    } else {
                        (*n as u8).checked_add(1).map(i128::from)
                    }
                });
                let variable = |name: &str| match name {
                    "x" => Value::Array(values.iter().copied().map(Value::Int).collect()),
                    "y" => Value::Int(index as i128),
                    _ => panic!("unexpected input"),
                };
                assert_eq!(
                    evaluate_with(&inferred.x.require[0], &variable),
                    Value::Bool(expected.is_some()),
                    "signed={signed}, values={values:?}, index={index}"
                );
                if let Some(value) = expected {
                    assert_eq!(evaluate_with(output, &variable), Value::Int(value));
                }
            }
        }
    }
}

#[test]
fn slice_builders_reject_mutability_and_wrong_element_types() {
    let unsigned = input("x").typ.clone();
    let valid = slice(unsigned.clone());
    assert!(input_type(&valid));
    assert!(!input_type(&reference(valid.clone())));
    let TypX::Decorate(_, _, inner) = &*valid else { panic!() };
    assert!(!input_type(inner));
    let mutable = Arc::new(TypX::MutRef(inner.clone()));
    assert!(element(&mutable).is_none());
    assert!(!compatible(&valid, &slice(bool_typ())));
    assert!(element(&slice(valid.clone())).is_none());
    assert!(!substitutes(&valid, &arrays::typ(unsigned.clone(), 2).unwrap()));
    assert!(substitutes(&arrays::typ(unsigned, 2).unwrap(), &valid));
    let value = Term { expr: input("x"), size: 1 };
    assert!(length(&span(), value.clone()).is_err());
    assert!(index(&span(), value.clone(), value).is_err());
}

#[test]
fn canonical_slice_models_and_cache_roundtrips_are_checked() {
    let imports = [model_function(false), model_function(true)];
    for f in &imports {
        assert!(model(f));
        let mut bad = f.x.clone();
        bad.name = crate::infer::tests::function("lookalike", input("x")).x.name.clone();
        assert!(!model(&f.new_x(bad)));
        let mut bad = f.x.clone();
        bad.typ_bounds = Arc::new(vec![]);
        assert!(!model(&f.new_x(bad)));
    }
    let mut registry = registry::Registry::new(None);
    let f = slice_function(false);
    let first = registry.infer_with_imports(&[f.clone()], &imports).unwrap();
    let second = registry.infer_with_imports(&[f.clone()], &imports).unwrap();
    assert_eq!((registry.hits, registry.misses), (1, 1));
    assert_eq!(format_contract(&first[0]), format_contract(&second[0]));
    let mut missing = registry::Registry::new(None);
    assert!(
        missing
            .infer_with_imports(&[f], &[])
            .unwrap_err()
            .note
            .contains("requires vstd slice models")
    );
}
