//! Identity-only leaves for read-only records containing explicitly declared
//! foreign opaque types. No representation, method, equality or crypto law.
use super::*;

pub(super) fn readonly_field(f: &Function, typ: &Typ, ds: &[Datatype], nested: bool) -> bool {
    nested
        && !f.x.attrs.infer_allow_approx
        && f.x
            .params
            .first()
            .is_some_and(|p| matches!(&*p.x.typ, TypX::Decorate(TypDecoration::Ref, None, _)))
        && declared_leaf(typ, ds)
}

pub(super) fn declared_leaf(typ: &Typ, ds: &[Datatype]) -> bool {
    let TypX::Datatype(Dt::Path(name), args, _) = &**typ else { return false };
    args.is_empty()
        && ds.iter().any(|d| {
            d.x.name == Dt::Path(name.clone())
                && d.x.proxy.as_ref().is_some_and(|p| p.x.krate != name.krate)
                && d.x.mode == Mode::Exec
                && matches!(d.x.transparency, DatatypeTransparency::Never)
                && d.x.typ_params.is_empty()
                && d.x.typ_bounds.is_empty()
                && d.x.sized_constraint.is_none()
                && !d.x.ext_equal
                && !d.x.destructor
                && d.x.user_defined_invariant_fn.is_none()
                && d.x.variants.len() == 1
                && d.x.variants[0].fields.is_empty()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{function, input, span};

    fn path(krate: &str, id: u64) -> Path {
        Arc::new(PathX {
            krate: CrateId::Id(Arc::new(krate.into()), id),
            segments: Arc::new(vec![Arc::new("Token".into())]),
        })
    }
    fn leaf() -> (Typ, Datatype) {
        let name = Dt::Path(path("foreign", 11));
        let d = DatatypeX {
            name: name.clone(),
            proxy: Some(crate::def::Spanned { span: span(), x: path("local", 12) }),
            owning_module: None,
            visibility: Visibility { restricted_to: None },
            transparency: DatatypeTransparency::Never,
            typ_params: Arc::new(vec![]),
            typ_bounds: Arc::new(vec![]),
            variants: Arc::new(vec![Variant {
                name: Arc::new("Token".into()),
                fields: Arc::new(vec![]),
                ctor_style: CtorPrintStyle::Braces,
            }]),
            mode: Mode::Exec,
            ext_equal: false,
            user_defined_invariant_fn: None,
            sized_constraint: None,
            destructor: false,
        };
        (
            Arc::new(TypX::Datatype(name, Arc::new(vec![]), Arc::new(vec![]))),
            crate::def::Spanned::new(span(), d),
        )
    }

    #[test]
    fn opaque_foreign_leaf_requires_exact_identity_and_closed_descriptor() {
        let (typ, original) = leaf();
        assert!(declared_leaf(&typ, &[original.clone()]));
        for fault in 0..14 {
            let mut d = original.x.clone();
            match fault {
                0 => d.name = Dt::Path(path("other_foreign", 13)),
                1 => d.proxy = None,
                2 => d.proxy.as_mut().unwrap().x = path("foreign", 11),
                3 => d.mode = Mode::Spec,
                4 => {
                    d.transparency =
                        DatatypeTransparency::WhenVisible(Visibility { restricted_to: None })
                }
                5 => {
                    d.typ_params =
                        Arc::new(vec![(Arc::new("T".into()), AcceptRecursiveType::Reject)])
                }
                6 => d.sized_constraint = Some(bool_typ()),
                7 => d.ext_equal = true,
                8 => d.destructor = true,
                9 => {
                    d.user_defined_invariant_fn = Some(FunWithVis {
                        visibility: Visibility { restricted_to: None },
                        fun: function("invariant", input("x")).x.name.clone(),
                    })
                }
                10 => d.variants = Arc::new(vec![]),
                11 => {
                    d.typ_bounds = Arc::new(vec![Arc::new(GenericBoundX::Trait(
                        TraitId::Sizedness(Sizedness::Sized),
                        Arc::new(vec![bool_typ()]),
                    ))])
                }
                12 => Arc::make_mut(&mut Arc::make_mut(&mut d.variants)[0].fields).push(Arc::new(
                    air::ast::BinderX {
                        name: Arc::new("hidden".into()),
                        a: (bool_typ(), Mode::Exec, Visibility { restricted_to: None }),
                    },
                )),
                13 => {
                    let second = d.variants[0].clone();
                    Arc::make_mut(&mut d.variants).push(second);
                }
                _ => unreachable!(),
            }
            assert!(!declared_leaf(&typ, &[original.new_x(d)]), "fault {fault}");
        }
        let generic = Arc::new(TypX::Datatype(
            original.x.name.clone(),
            Arc::new(vec![bool_typ()]),
            Arc::new(vec![]),
        ));
        assert!(!declared_leaf(&generic, &[original]));
    }

    #[test]
    fn foreign_leaf_admission_is_nested_exact_shared_receiver_only() {
        let (typ, d) = leaf();
        let f = function("query", input("x"));
        let mut x = f.x.clone();
        let mut p = x.params[0].x.clone();
        p.typ = Arc::new(TypX::Decorate(TypDecoration::Ref, None, typ.clone()));
        x.params = Arc::new(vec![x.params[0].new_x(p)]);
        let f = f.new_x(x);
        assert!(readonly_field(&f, &typ, &[d.clone()], true));
        assert!(!readonly_field(&f, &typ, &[d.clone()], false));
        for fault in 0..3 {
            let mut x = f.x.clone();
            let mut p = x.params[0].x.clone();
            match fault {
                0 => p.typ = Arc::new(TypX::MutRef(typ.clone())),
                1 => p.typ = typ.clone(),
                2 => Arc::make_mut(&mut x.attrs).infer_allow_approx = true,
                _ => unreachable!(),
            }
            x.params = Arc::new(vec![x.params[0].new_x(p)]);
            assert!(!readonly_field(&f.new_x(x), &typ, &[d.clone()], true));
        }
    }
}
