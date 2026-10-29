//! Bounded type/visibility inspection; no candidate behavior is assumed here.
use super::*;

pub(in crate::infer) fn root_type(f: &Function) -> Option<&Typ> {
    match &*f.x.params.first()?.x.typ {
        TypX::MutRef(t) | TypX::Decorate(TypDecoration::Ref, None, t) => Some(t),
        _ => None,
    }
}
pub(super) fn candidate(f: &Function) -> bool {
    root_type(f).is_some_and(|t|matches!(&**t,TypX::Datatype(Dt::Path(p),..)
        if !models::path_is(p,CrateId::Alloc,&["vec","Vec"]) && !models::path_is(p,CrateId::Core,&["option","Option"])))
        && (!mutable(f) || enums::unit(&f.x.inner_ret.x.typ))
}
pub(in crate::infer) fn fields(typ: &Typ, datatypes: &[Datatype]) -> Option<Vec<(FieldOpr, Typ)>> {
    let TypX::Datatype(dt, args, _) = &**typ else { return None };
    let d = datatypes.iter().find(|d| &d.x.name == dt)?;
    if d.x.mode != Mode::Exec
        || d.x.proxy.is_some()
        || d.x.destructor
        || d.x.user_defined_invariant_fn.is_some()
        || d.x.variants.len() != 1
        || d.x.variants[0].fields.is_empty()
        || d.x.variants[0].fields.len() > 8
        || d.x.typ_params.len() != args.len()
    {
        return None;
    }
    let ts =
        d.x.typ_params.iter().zip(args.iter()).map(|(p, t)| (p.0.clone(), t.clone())).collect();
    d.x.variants[0]
        .fields
        .iter()
        .map(|field| {
            (field.a.1 == Mode::Exec).then(|| {
                (
                    FieldOpr {
                        datatype: dt.clone(),
                        variant: d.x.variants[0].name.clone(),
                        field: field.name.clone(),
                        get_variant: false,
                        check: VariantCheck::None,
                    },
                    crate::sst_util::subst_typ(&ts, &field.a.0),
                )
            })
        })
        .collect()
}
/// A bounded value enum. Payload eligibility is checked recursively by value_type,
/// so this descriptor never grants a heap/reference abstraction by itself.
pub(in crate::infer) fn variants(typ: &Typ, datatypes: &[Datatype]) -> Option<Vec<(Ident, Vec<(Ident, Typ)>)>> {
    let TypX::Datatype(dt,args,_)=&**typ else{return None;};
    // Core Option/Result retain their dedicated layouts. In particular, do not
    // admit compiler-internal ControlFlow/Infallible through this value profile.
    if !matches!(dt,Dt::Path(p) if matches!(p.krate,CrateId::Id(..))){return None;}
    let d=datatypes.iter().find(|d|&d.x.name==dt)?;
    if d.x.mode!=Mode::Exec || d.x.proxy.is_some() || d.x.destructor
        || d.x.user_defined_invariant_fn.is_some() || !(2..=8).contains(&d.x.variants.len())
        || d.x.typ_params.len()>4 || d.x.typ_params.len()!=args.len()
        || d.x.variants.iter().any(|v|v.fields.len()>4 || v.fields.iter().any(|f|f.a.1!=Mode::Exec))
    {return None;}
    let ts=d.x.typ_params.iter().zip(args.iter()).map(|(p,t)|(p.0.clone(),t.clone())).collect();
    Some(d.x.variants.iter().map(|v|(v.name.clone(),v.fields.iter().map(|f|
        (f.name.clone(),crate::sst_util::subst_typ(&ts,&f.a.0))).collect())).collect())
}
fn associated_type(t:&Typ,params:&[Ident],depth:usize)->bool {
    if depth==0{return false;}
    match &**t {
        TypX::TypParam(p)=>params.contains(p),
        TypX::Projection{trait_typ_args,..}=>!trait_typ_args.is_empty() && trait_typ_args.len()<=4
            && trait_typ_args.iter().all(|t|associated_type(t,params,depth-1)),
        TypX::Datatype(_,args,_)=>args.len()<=4 && args.iter().all(|t|associated_type(t,params,depth-1)),
        _=>scalar(t),
    }
}

pub(in crate::infer) fn value_type(t: &Typ, params: &[Ident], datatypes: &[Datatype], depth: usize) -> bool {
    if depth == 0 {
        return false;
    }
    if scalar(t) {
        return true;
    }
    match &**t {
        TypX::TypParam(p) => params.contains(p),
        // Associated-type syntax is bounded separately from runtime payload
        // nesting: Option<&(H::Hash,u16)> has the same value shape as (T,u16).
        TypX::Projection{..} => associated_type(t,params,4),
        TypX::Datatype(Dt::Tuple(n), ts, _) if *n <= 4 && ts.len() == *n => {
            ts.iter().all(|t| value_type(t, params, datatypes, depth - 1))
        }
        TypX::Datatype(Dt::Path(p), ts, _)
            if models::path_is(p, CrateId::Core, &["option", "Option"]) && ts.len() == 1 =>
        {
            value_type(&ts[0], params, datatypes, depth - 1)
        }
        TypX::Decorate(TypDecoration::Ref, None, t) => value_type(t, params, datatypes, depth - 1),
        _ => fields(t, datatypes)
            .is_some_and(|fs| fs.iter().all(|(_, t)| value_type(t, params, datatypes, depth - 1)))
            || variants(t,datatypes).is_some_and(|vs|vs.iter().all(|(_,fs)|
                fs.iter().all(|(_,t)|value_type(t,params,datatypes,depth-1)))),
    }
}
pub(in crate::infer) fn bounds(bounds: &GenericBounds, params: &[Ident]) -> bool {
    let mut recognized = 0;
    for p in params {
        let t = Arc::new(TypX::TypParam(p.clone()));
        let (mut sized, mut copy) = (0, 0);
        for b in bounds.iter() {
            match &**b {
                GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), ts)
                    if ts.len() == 1 && same(&ts[0], &t) =>
                {
                    sized += 1
                }
                GenericBoundX::Trait(TraitId::Path(path), ts)
                    if ts.len() == 1
                        && same(&ts[0], &t)
                        && models::path_is(path, CrateId::Core, &["marker", "Copy"]) =>
                {
                    copy += 1
                }
                _ => {}
            }
        }
        if sized != 1 || copy > 1 {
            return false;
        }
        recognized += sized + copy;
    }
    recognized == bounds.len()
}

fn record_bounds(bounds:&GenericBounds,params:&[Ident]) -> bool {
    bounds.len()<=24 && params.iter().all(|p|bounds.iter().filter(|b|
        matches!(&***b,GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized),ts) if ts.len()==1 && matches!(&*ts[0],TypX::TypParam(n) if n==p))).count()==1)
        && bounds.iter().all(|b|match &**b {
            GenericBoundX::Trait(_,ts)=>!ts.is_empty() && ts.len()<=4 && ts.iter().all(|t|associated_type(t,params,4)),
            GenericBoundX::TypEquality(_,ts,_,t)=>!ts.is_empty() && ts.len()<=4 && ts.iter().all(|t|associated_type(t,params,4)) && associated_type(t,params,4),
            _=>false,
        })
}
#[derive(Clone)]
pub(in crate::infer) struct Projection {
    pub op: FieldOpr,
    pub typ: Typ,
}
#[derive(Clone)]
pub(in crate::infer) struct Field {
    pub path: Vec<Projection>,
    pub typ: Typ,
    pub element: Option<Typ>,
    pub view: Option<sequence_views::SequenceView>,
}
pub(in crate::infer) struct Shape {
    pub typ: Typ,
    pub fields: Vec<Field>,
}
pub(in crate::infer) fn same_path(a: &[FieldOpr], b: &[FieldOpr]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(a, b)| {
            a.datatype == b.datatype
                && a.variant == b.variant
                && a.field == b.field
                && a.check == b.check
                && a.get_variant == b.get_variant
        })
}
pub(super) fn shape(f: &Function, datatypes: &[Datatype], all: &HashMap<Fun, Function>) -> Result<Shape, VirErr> {
    let bad = || {
        unsupported(
            &f.span,
            "record state requires one transparent record, bounded scalar/Vec fields, value parameters and a supported return",
        )
    };
    let typ = root_type(f).filter(|_| candidate(f)).ok_or_else(bad)?;
    if !mutable(f) && enums::unit(&f.x.inner_ret.x.typ) {
        return Err(unsupported(&f.span, "read-only record queries require a value return"));
    }
    if f.x.params.len() > 8
        || f.x.typ_params.len() > 2
        || !record_bounds(&f.x.typ_bounds, &f.x.typ_params)
        || f.x.params.iter().skip(1).any(|p| {
            matches!(&*p.x.typ, TypX::MutRef(_))
                || trait_getters::shared(&p.x.typ).is_some_and(|t|!matches!(&**t,TypX::TypParam(_)|TypX::Projection{..}))
                || !value_type(&p.x.typ, &f.x.typ_params, datatypes, 4)
        })
        || !value_type(&f.x.inner_ret.x.typ, &f.x.typ_params, datatypes, 4)
    {
        return Err(bad());
    }
    fn walk(
        f: &Function,
        t: &Typ,
        ds: &[Datatype],
        path: Vec<Projection>,
        out: &mut Vec<Field>,
        depth: usize,
        all: &HashMap<Fun, Function>,
    ) -> Result<(), VirErr> {
        let bad = || {
            unsupported(
                &f.span,
                "record state requires visible transparent record fields or a certified direct sequence view, bounded depth and at most sixteen leaves",
            )
        };
        if depth == 0 || out.len() >= 16 {
            return Err(bad());
        }
        if let Some(ts) = collections::vec_args(t) {
            if ts.len() != 2
                || !matches!(&*ts[1],TypX::Datatype(Dt::Path(p),xs,_) if xs.is_empty() && models::path_is(p,CrateId::Alloc,&["alloc","Global"]))
                || !value_type(&ts[0], &f.x.typ_params, ds, 4)
            {
                return Err(bad());
            }
            out.push(Field { path, typ: t.clone(), element: Some(ts[0].clone()), view: None });
            return Ok(());
        }
        if let Some(view) = sequence_views::find(t, all, ds) {
            out.push(Field { path, typ: t.clone(), element: Some(view.element.clone()), view: Some(view) });
            return Ok(());
        }
        // Ordinary scalar/Option/tuple payloads remain leaves; transparent
        // application records are traversed, including scalar newtype fields.
        if scalar(t)
            || matches!(&**t, TypX::TypParam(_) | TypX::Projection{..} | TypX::Datatype(Dt::Tuple(_), ..))
            || matches!(&**t,TypX::Datatype(Dt::Path(p),.. ) if models::path_is(p,CrateId::Core,&["option","Option"]))
        {
            if !value_type(t, &f.x.typ_params, ds, 4) {
                return Err(bad());
            }
            out.push(Field { path, typ: t.clone(), element: None, view: None });
            return Ok(());
        }
        let TypX::Datatype(dt, ..) = &**t else { return Err(bad()) };
        let d = ds.iter().find(|d| &d.x.name == dt).ok_or_else(bad)?;
        if !matches!(&d.x.transparency,DatatypeTransparency::WhenVisible(vis) if crate::ast_util::is_visible_to_opt(vis,&f.x.owning_module))
        {
            return Err(bad());
        }
        for (op, typ) in fields(t, ds).ok_or_else(bad)? {
            let mut path = path.clone();
            path.push(Projection { op, typ: typ.clone() });
            walk(f, &typ, ds, path, out, depth - 1, all)?;
        }
        Ok(())
    }
    let mut leaves = vec![];
    walk(f, typ, datatypes, vec![], &mut leaves, 5, all)?;
    Ok(Shape { typ: typ.clone(), fields: leaves })
}
