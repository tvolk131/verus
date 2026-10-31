//! Closed library interfaces used by owned collection candidates.
use super::*;
pub(super) use crate::infer::models::path_is;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Model {
    New,
    Capacity,
    Push,
    Repeat,
    Clone,
    Empty,
    SeqNew,
    Take,
    SeqPush,
    SeqFilter,
    SeqFold,
    Scatter,
    ForSeq,
    ForIndex,
}

pub(crate) fn model(f: &Function) -> Option<Model> {
    let x = &f.x;
    if x.attrs.infer
        || x.attrs.is_unsafe
        || x.attrs.is_async
        || x.params.iter().any(|p| p.x.user_mut)
        || !x.require.is_empty()
        || !x.ensure.1.is_empty()
        || !same(&x.inner_ret.x.typ, &x.outer_ret.x.typ)
    {
        return None;
    }
    let p = &x.name.path;
    if x.mode == Mode::Spec
        && x.typ_params.len() == 1
        && x.params.len() == 1
        && p.krate == CrateId::Vstd
        && p.segments.len() == 4
        && p.segments[0].as_str() == "std_specs"
        && p.segments[1].as_str() == "iter"
        && p.segments[2].starts_with("impl")
        && matches!(&*x.params[0].x.typ,TypX::Datatype(Dt::Path(path),ts,_) if path_is(path,CrateId::Vstd,&["std_specs","iter","VerusForLoopWrapper"]) && ts.len()==1 && matches!(&*ts[0],TypX::TypParam(n) if n==&x.typ_params[0]))
    {
        if p.last_segment().as_str() == "seq" && iterators::sequence(&x.inner_ret.x.typ).is_some() {
            return Some(Model::ForSeq);
        }
        if p.last_segment().as_str() == "index" && same(&x.inner_ret.x.typ, &int_typ()) {
            return Some(Model::ForIndex);
        }
    }

    let spec = if models::path_is(p, CrateId::Vstd, &["seq_update", "scatter"]) {
        Some(Model::Scatter)
    } else if models::path_is(p, CrateId::Vstd, &["seq", "Seq", "empty"]) {
        Some(Model::Empty)
    } else if models::path_is(p, CrateId::Vstd, &["seq", "Seq", "new"]) {
        Some(Model::SeqNew)
    } else if models::path_is(p, CrateId::Vstd, &["seq", "Seq", "take"])
        || p.krate == CrateId::Vstd
            && p.segments.len() == 3
            && p.segments[0].as_str() == "seq"
            && p.segments[1].starts_with("impl")
            && p.segments[2].as_str() == "take"
    {
        Some(Model::Take)
    } else if models::path_is(p, CrateId::Vstd, &["seq", "Seq", "push"]) {
        Some(Model::SeqPush)
    } else if p.krate == CrateId::Vstd
        && p.segments.len() == 3
        && p.segments[0].as_str() == "seq_lib"
        && p.segments[1].starts_with("impl")
        && p.segments[2].as_str() == "fold_left"
    {
        Some(Model::SeqFold)
    } else if models::path_is(p, CrateId::Vstd, &["seq", "Seq", "filter"])
        || p.krate == CrateId::Vstd
            && p.segments.len() == 3
            && matches!(p.segments[0].as_str(), "seq" | "seq_lib")
            && p.segments[1].starts_with("impl")
            && p.segments[2].as_str() == "filter"
    {
        Some(Model::SeqFilter)
    } else {
        None
    };
    if let Some(k) = spec {
        if x.mode != Mode::Spec
            || x.proxy.is_some()
            || x.typ_params.len()
                != if matches!(k, Model::SeqFold | Model::Scatter) { 2 } else { 1 }
        {
            return None;
        }
        let t = Arc::new(TypX::TypParam(x.typ_params[0].clone()));
        let seq = sequence_type(t.clone());
        let valid = match k {
            Model::Empty => x.params.is_empty(),
            Model::SeqNew => {
                x.params.len() == 2
                    && matches!(&*x.params[0].x.typ, TypX::Int(IntRange::Nat))
                    && matches!(&*x.params[1].x.typ,TypX::SpecFn(args,ret) if args.len()==1 && same(&args[0],&int_typ()) && same(ret,&t))
            }
            Model::Take => {
                x.params.len() == 2
                    && same(&x.params[0].x.typ, &seq)
                    && same(&x.params[1].x.typ, &int_typ())
            }
            Model::SeqPush => {
                x.params.len() == 2
                    && same(&x.params[0].x.typ, &seq)
                    && same(&x.params[1].x.typ, &t)
            }
            Model::SeqFold => {
                let b = Arc::new(TypX::TypParam(x.typ_params[1].clone()));
                x.params.len() == 3
                    && same(&x.params[0].x.typ, &seq)
                    && same(&x.params[1].x.typ, &b)
                    && same(&x.inner_ret.x.typ, &b)
                    && matches!(&*x.params[2].x.typ,TypX::SpecFn(args,ret) if args.len()==2 && same(&args[0],&b) && same(&args[1],&t) && same(ret,&b))
            }
            Model::Scatter => {
                let b = Arc::new(TypX::TypParam(x.typ_params[1].clone()));
                let output = sequence_type(b.clone());
                x.params.len() == 4
                    && same(&x.params[0].x.typ, &seq)
                    && same(&x.params[1].x.typ, &output)
                    && same(&x.inner_ret.x.typ, &output)
                    && matches!(&*x.params[2].x.typ, TypX::SpecFn(args, ret)
                        if args.len()==1 && same(&args[0], &t) && same(ret, &int_typ()))
                    && matches!(&*x.params[3].x.typ, TypX::SpecFn(args, ret)
                        if args.len()==1 && same(&args[0], &t) && same(ret, &b))
            }
            Model::SeqFilter => {
                x.params.len() == 2
                    && same(&x.params[0].x.typ, &seq)
                    && matches!(&*x.params[1].x.typ,TypX::SpecFn(args,ret) if args.len()==1 && same(&args[0],&t) && same(ret,&bool_typ()))
            }
            _ => false,
        };
        return (valid
            && (matches!(k, Model::SeqFold | Model::Scatter) || same(&x.inner_ret.x.typ, &seq))
            && x.params.iter().all(|p| p.x.mode == Mode::Spec))
        .then_some(k);
    }
    if x.mode != Mode::Exec || x.body.is_some() || x.params.iter().any(|p| p.x.mode != Mode::Exec) {
        return None;
    }
    let proxy = &x.proxy.as_ref()?.x;
    if proxy.krate != CrateId::Vstd
        || proxy.segments.len() != 3
        || proxy.segments[0].as_str() != "std_specs"
        || proxy.segments[1].as_str() != "vec"
        || !proxy.segments[2].starts_with("_verus_external_fn_specification_")
    {
        return None;
    }
    let name = p.segments.last()?.as_str();
    let k = if p.krate == CrateId::Alloc
        && p.segments.len() == 3
        && p.segments[0].as_str() == "vec"
        && p.segments[1].starts_with("impl")
    {
        match name {
            "new" => Model::New,
            "with_capacity" => Model::Capacity,
            "push" => Model::Push,
            "clone" => Model::Clone,
            _ => return None,
        }
    } else if models::path_is(p, CrateId::Alloc, &["vec", "from_elem"]) {
        Model::Repeat
    } else {
        return None;
    };
    let allocator = matches!(k, Model::Push | Model::Clone);
    let ts: Vec<_> = x.typ_params.iter().map(|n| Arc::new(TypX::TypParam(n.clone()))).collect();
    if ts.len() != if allocator { 2 } else { 1 } {
        return None;
    }
    let alloc = if allocator {
        ts[1].clone()
    } else {
        Arc::new(TypX::Datatype(
            Dt::Path(Arc::new(PathX {
                krate: CrateId::Alloc,
                segments: Arc::new(vec![Arc::new("alloc".into()), Arc::new("Global".into())]),
            })),
            Arc::new(vec![]),
            Arc::new(vec![]),
        ))
    };
    let vector = |t: &Typ| {
        collections::vec_args(t)
            .is_some_and(|args| same(&args[0], &ts[0]) && same(&args[1], &alloc))
    };
    let usize_type = |t: &Typ| matches!(&**t, TypX::Int(IntRange::USize));
    let valid = match k {
        Model::New => x.params.is_empty() && vector(&x.inner_ret.x.typ),
        Model::Capacity => {
            x.params.len() == 1 && usize_type(&x.params[0].x.typ) && vector(&x.inner_ret.x.typ)
        }
        Model::Push => {
            x.params.len() == 2
                && matches!(&*x.params[0].x.typ,TypX::MutRef(t) if vector(t))
                && same(&x.params[1].x.typ, &ts[0])
                && enums::unit(&x.inner_ret.x.typ)
        }
        Model::Clone => {
            x.params.len() == 1
                && matches!(&*x.params[0].x.typ,TypX::Decorate(TypDecoration::Ref,None,t) if vector(t))
                && vector(&x.inner_ret.x.typ)
        }
        Model::Repeat => {
            x.params.len() == 2
                && same(&x.params[0].x.typ, &ts[0])
                && usize_type(&x.params[1].x.typ)
                && vector(&x.inner_ret.x.typ)
        }
        _ => false,
    };
    let mut sized = vec![0; ts.len()];
    let mut cloned = vec![0; ts.len()];
    let mut allocators = 0;
    for b in x.typ_bounds.iter() {
        match &**b {
            GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), args) if args.len() == 1 => {
                sized[ts.iter().position(|t| same(t, &args[0]))?] += 1;
            }
            GenericBoundX::Trait(TraitId::Path(p), args)
                if args.len() == 1 && models::path_is(p, CrateId::Core, &["clone", "Clone"]) =>
            {
                cloned[ts.iter().position(|t| same(t, &args[0]))?] += 1;
            }
            GenericBoundX::Trait(TraitId::Path(p), args)
                if allocator
                    && args.len() == 1
                    && same(&args[0], &ts[1])
                    && models::path_is(p, CrateId::Core, &["alloc", "Allocator"]) =>
            {
                allocators += 1;
            }
            _ => return None,
        }
    }
    if valid
        && sized.iter().all(|n| *n == 1)
        && allocators == usize::from(allocator)
        && cloned
            .iter()
            .enumerate()
            .all(|(i, n)| *n == usize::from(k == Model::Clone || k == Model::Repeat && i == 0))
    {
        return Some(k);
    }

    None
}

pub(crate) fn call(e: &Expr, all: &HashMap<Fun, Function>) -> Option<Model> {
    let ExprX::Call {
        target: CallTarget::Fun(kind, name, ..), post_args: None, body: None, ..
    } = &e.x
    else {
        return None;
    };
    let f = match kind {
        CallTargetKind::Static => all.get(name)?,
        CallTargetKind::DynamicResolved { resolved, is_trait_default: false, .. } => {
            all.get(resolved)?
        }
        _ => return None,
    };
    model(f)
}
