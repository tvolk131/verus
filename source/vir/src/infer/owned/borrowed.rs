//! Private logical definitions for exact borrowed projections. These are normal
//! spec functions; the original getter must prove its equation and precision.
use super::*;

fn value_name(f: &Function) -> Fun {
    let mut path = (*f.x.name.path).clone();
    Arc::make_mut(&mut path.segments).push(Arc::new("__verus_infer_borrow_value".into()));
    Arc::new(FunX { path: Arc::new(path) })
}

fn projection(f: &Function, e: &Expr) -> Option<Expr> {
    fn place(f: &Function, p: &Place) -> Option<Place> {
        match &p.x {
            PlaceX::Local(n)
                if f.x
                    .params
                    .iter()
                    .any(|p| &p.x.name == n && trait_getters::shared(&p.x.typ).is_some()) =>
            {
                Some(p.clone())
            }
            PlaceX::Temporary(e) => Some(p.new_x(PlaceX::Temporary(projection(f, e)?))),
            PlaceX::Field(op, base) if op.check == VariantCheck::None && !op.get_variant => {
                Some(p.new_x(PlaceX::Field(op.clone(), place(f, base)?)))
            }
            _ => None,
        }
    }
    match &e.x {
        ExprX::Block(ss, Some(tail)) if ss.is_empty() => projection(f, tail),
        ExprX::ReadPlace(p, _) => Some(e.new_x(ExprX::ReadPlace(
            place(f, p)?,
            UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: u64::MAX },
        ))),
        ExprX::Var(n)
            if f.x
                .params
                .iter()
                .any(|p| &p.x.name == n && trait_getters::shared(&p.x.typ).is_some()) =>
        {
            Some(e.clone())
        }
        _ => None,
    }
}

pub(crate) fn prepare(fs: &[Function], imported: &[Function]) -> Result<Vec<Function>, VirErr> {
    let mut out = fs.to_vec();
    let mut names = fs
        .iter()
        .chain(imported)
        .map(|f| f.x.name.clone())
        .collect::<std::collections::HashSet<_>>();
    for f in fs.iter().filter(|f| {
        f.x.attrs.infer
            && f.x.mode == Mode::Exec
            && matches!(f.x.kind, FunctionKind::Static)
            && identity_borrow_return(f)
    }) {
        let Some(body) = f.x.body.as_ref().and_then(|e| projection(f, e)) else {
            continue;
        };
        let ret = trait_getters::shared(&f.x.inner_ret.x.typ).unwrap().clone();
        let body = coerce(&body.span, atom(body.clone())?, &ret)?.expr;
        let name = value_name(f);
        if !names.insert(name.clone()) {
            return Err(unsupported(&f.span, "borrowed observation name collision"));
        }
        let mut x = f.x.clone();
        x.name = name;
        x.mode = Mode::Spec;
        x.proxy = None;
        let visibility = Visibility { restricted_to: x.owning_module.clone() };
        x.body_visibility = BodyVisibility::Visibility(visibility.clone());
        x.opaqueness = Opaqueness::Revealed { visibility };
        x.params = Arc::new(
            x.params
                .iter()
                .map(|p| {
                    let mut p_x = p.x.clone();
                    p_x.mode = Mode::Spec;
                    p.new_x(p_x)
                })
                .collect(),
        );
        let mut p = x.inner_ret.x.clone();
        p.mode = Mode::Spec;
        p.typ = ret;
        x.inner_ret = x.inner_ret.new_x(p);
        x.outer_ret = x.inner_ret.clone();
        x.require = Arc::new(vec![]);
        x.ensure = (Arc::new(vec![]), Arc::new(vec![]));
        x.returns = None;
        x.decrease = Arc::new(vec![]);
        x.decrease_when = None;
        x.decrease_by = None;
        x.fndef_axioms = None;
        x.mask_spec = None;
        x.atomic_update = None;
        x.unwind_spec = None;
        x.attrs = Arc::new(FunctionAttrsX::default());
        x.body = Some(body);
        x.extra_dependencies = vec![];
        x.hidden = Arc::new(vec![]);
        out.push(f.new_x(x));
    }
    Ok(out)
}

pub(crate) fn is_value(f: &Function) -> bool {
    is_value_name(&f.x.name) && is_pure_observation(f)
}
pub(crate) fn is_value_name(f: &Fun) -> bool {
    f.path.last_segment().as_str() == "__verus_infer_borrow_value"
}
pub(crate) fn source(f: &Function) -> Option<Fun> {
    if !is_value(f) {
        return None;
    }
    let mut path = (*f.x.name.path).clone();
    Arc::make_mut(&mut path.segments).pop()?;
    Some(Arc::new(FunX { path: Arc::new(path) }))
}

pub(super) fn value(f: &Function, all: &HashMap<Fun, Function>) -> Option<Function> {
    all.get(&value_name(f)).filter(|f| is_value(f)).cloned()
}

impl Engine<'_> {
    pub(super) fn borrowed_value(&self, s: &State, f: &Function) -> Result<Option<Term>, VirErr> {
        let Some(spec) = value(f, self.all) else {
            return Ok(None);
        };
        let args = f
            .x
            .params
            .iter()
            .map(|p| {
                self.object(s, &SpannedTyped::new(&p.span, &p.x.typ, ExprX::Var(p.x.name.clone())))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(trait_getters::instantiate(
            &spec,
            &Arc::new(f.x.typ_params.iter().map(|p| Arc::new(TypX::TypParam(p.clone()))).collect()),
            args,
            &f.x.typ_bounds,
        )?))
    }
}
