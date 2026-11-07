use super::*;
use crate::infer::tests::span;

fn path(krate: CrateId, segments: &[&str]) -> Path {
    Arc::new(PathX {
        krate,
        segments: Arc::new(segments.iter().map(|segment| Arc::new((*segment).into())).collect()),
    })
}

fn canonical_range() -> (Typ, Datatype) {
    let name = Dt::Path(path(CrateId::Core, &["ops", "range", "Range"]));
    let parameter: Ident = Arc::new("Idx".into());
    let parameter_type = Arc::new(TypX::TypParam(parameter.clone()));
    let visibility = Visibility { restricted_to: None };
    let definition = DatatypeX {
        name: name.clone(),
        proxy: Some(crate::def::Spanned {
            span: span(),
            x: path(CrateId::Vstd, &["std_specs", "range", "ExRange"]),
        }),
        owning_module: None,
        visibility: visibility.clone(),
        transparency: DatatypeTransparency::WhenVisible(visibility.clone()),
        typ_params: Arc::new(vec![(parameter, AcceptRecursiveType::RejectInGround)]),
        typ_bounds: Arc::new(vec![Arc::new(GenericBoundX::Trait(
            TraitId::Sizedness(Sizedness::Sized),
            Arc::new(vec![parameter_type.clone()]),
        ))]),
        variants: Arc::new(vec![Variant {
            name: Arc::new("Range".into()),
            fields: Arc::new(
                ["start", "end"]
                    .iter()
                    .map(|name| {
                        Arc::new(air::ast::BinderX {
                            name: Arc::new((*name).into()),
                            a: (parameter_type.clone(), Mode::Exec, visibility.clone()),
                        })
                    })
                    .collect(),
            ),
            ctor_style: CtorPrintStyle::Braces,
        }]),
        mode: Mode::Exec,
        ext_equal: false,
        user_defined_invariant_fn: None,
        sized_constraint: None,
        destructor: false,
    };
    let typ = Arc::new(TypX::Datatype(
        name,
        Arc::new(vec![Arc::new(TypX::Int(IntRange::U(64)))]),
        Arc::new(vec![]),
    ));
    (typ, crate::def::Spanned::new(span(), definition))
}

#[test]
fn only_canonical_transparent_range_proxy_layout_is_admitted() {
    let (typ, original) = canonical_range();
    assert_eq!(shape::fields(&typ, &[original.clone()]).unwrap().len(), 2);
    for fault in 0..11 {
        let mut definition = original.x.clone();
        match fault {
            0 => {
                definition.proxy.as_mut().unwrap().x =
                    path(CrateId::Vstd, &["std_specs", "range", "OtherRange"])
            }
            1 => definition.destructor = true,
            2 => definition.mode = Mode::Spec,
            3 => definition.transparency = DatatypeTransparency::Never,
            4 => definition.typ_bounds = Arc::new(vec![]),
            5 => Arc::make_mut(&mut definition.variants)[0].name = Arc::new("OtherRange".into()),
            6..=9 => {
                let fields = &mut Arc::make_mut(&mut definition.variants)[0].fields;
                let field = Arc::make_mut(&mut Arc::make_mut(fields)[0]);
                match fault {
                    6 => field.name = Arc::new("other".into()),
                    7 => field.a.0 = bool_typ(),
                    8 => field.a.1 = Mode::Spec,
                    9 => field.a.2.restricted_to = Some(path(CrateId::Core, &["ops"])),
                    _ => unreachable!(),
                }
            }
            10 => definition.visibility.restricted_to = Some(path(CrateId::Core, &["ops"])),
            _ => unreachable!(),
        }
        assert!(shape::fields(&typ, &[original.new_x(definition)]).is_none(), "fault {fault}");
    }
    let TypX::Datatype(name, _, implementations) = &*typ else { unreachable!() };
    let other =
        Arc::new(TypX::Datatype(name.clone(), Arc::new(vec![bool_typ()]), implementations.clone()));
    assert!(shape::fields(&other, &[original]).is_none());
}
