use super::*;
use crate::infer::models as interfaces;

pub(super) fn slice(typ: &Typ) -> bool {
    matches!(&**typ, TypX::Decorate(TypDecoration::Ref, None, _))
        && matches!(&**arrays::readonly_base(typ), TypX::Primitive(Primitive::Slice, args)
            if args.len() == 1 && matches!(&*args[0], TypX::Int(IntRange::U(8))))
}

pub(super) fn slice_view(function: &Function) -> bool {
    let model = &function.x;
    let path = &model.name.path;
    if model.mode != Mode::Spec
        || model.inner_ret.x.mode != Mode::Spec
        || model.proxy.is_some()
        || model.body.is_some()
        || model.typ_params.len() != 1
        || model.params.len() != 1
        || model.params[0].x.mode != Mode::Spec
        || !model.ensure.0.is_empty()
        || path.krate != CrateId::Vstd
        || path.segments.len() != 3
        || path.segments[0].as_str() != "slice"
        || !path.segments[1].starts_with("impl")
        || path.segments[2].as_str() != "view"
    {
        return false;
    }
    let element = Arc::new(TypX::TypParam(model.typ_params[0].clone()));
    let expected =
        interfaces::reference(interfaces::primitive(Primitive::Slice, vec![element.clone()]));
    matches!(&model.kind, FunctionKind::TraitMethodImpl { method, trait_path, trait_typ_args, impl_path, .. }
        if models::path_is(&method.path, CrateId::Vstd, &["view", "View", "view"])
            && models::path_is(trait_path, CrateId::Vstd, &["view", "View"])
            && impl_path == &Arc::new(PathX { krate: path.krate.clone(), segments: Arc::new(path.segments[..2].to_vec()) })
            && trait_typ_args.len() == 1 && same(&interfaces::reference(trait_typ_args[0].clone()), &expected))
        && same(&model.params[0].x.typ, &expected)
        && same(&model.inner_ret.x.typ, &sequence_type(element.clone()))
        && model.typ_bounds.len() == 1
        && matches!(&*model.typ_bounds[0], GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), args)
            if args.len() == 1 && same(&args[0], &element))
}

pub(super) fn extend_model(function: &Function) -> bool {
    let model = &function.x;
    let path = &model.name.path;
    if model.mode != Mode::Exec
        || model.inner_ret.x.mode != Mode::Exec
        || !matches!(model.item_kind, ItemKind::Function)
        || model.body.is_some()
        || !matches!(model.kind, FunctionKind::Static)
        || model.params.iter().any(|param| param.x.mode != Mode::Exec)
        || model.typ_params.len() != 2
        || model.params.len() != 2
        || !enums::unit(&model.inner_ret.x.typ)
        || model.ensure.0.len() != 2
        || path.krate != CrateId::Alloc
        || path.segments.len() != 3
        || path.segments[0].as_str() != "vec"
        || !path.segments[1].starts_with("impl")
        || path.segments[2].as_str() != "extend_from_slice"
        || !model.proxy.as_ref().is_some_and(|proxy| {
            crate::infer::models::spec_proxy(
                &proxy.x,
                &["std_specs", "vec"],
                "_extend__from__slice",
            )
        })
    {
        return false;
    }
    let types: Vec<_> =
        model.typ_params.iter().map(|name| Arc::new(TypX::TypParam(name.clone()))).collect();
    let TypX::MutRef(vector) = &*model.params[0].x.typ else { return false };
    let Some(arguments) = collections::vec_args(vector) else { return false };
    if !arguments.iter().zip(&types).all(|(actual, expected)| same(actual, expected))
        || !same(
            &model.params[1].x.typ,
            &interfaces::reference(interfaces::primitive(Primitive::Slice, vec![types[0].clone()])),
        )
    {
        return false;
    }
    let mut sized = [0, 0];
    let mut cloned = 0;
    let mut allocator = 0;
    for bound in model.typ_bounds.iter() {
        match &**bound {
            GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), args) if args.len() == 1 => {
                let Some(index) = types.iter().position(|typ| same(typ, &args[0])) else {
                    return false;
                };
                sized[index] += 1;
            }
            GenericBoundX::Trait(TraitId::Path(path), args)
                if args.len() == 1
                    && same(&args[0], &types[0])
                    && models::path_is(path, CrateId::Core, &["clone", "Clone"]) =>
            {
                cloned += 1
            }
            GenericBoundX::Trait(TraitId::Path(path), args)
                if args.len() == 1
                    && same(&args[0], &types[1])
                    && models::path_is(path, CrateId::Core, &["alloc", "Allocator"]) =>
            {
                allocator += 1
            }
            _ => return false,
        }
    }
    sized == [1, 1] && cloned == 1 && allocator == 1
}

pub(super) fn borrowed_model(function: &Function) -> bool {
    let model = &function.x;
    let path = &model.name.path;
    if model.mode != Mode::Exec
        || model.inner_ret.x.mode != Mode::Exec
        || !matches!(model.item_kind, ItemKind::Function)
        || model.body.is_some()
        || model.typ_params.len() != 2
        || model.params.len() != 1
        || model.params[0].x.mode != Mode::Exec
        || model.ensure.0.len() != 1
        || path.krate != CrateId::Alloc
        || path.segments.len() != 3
        || path.segments[0].as_str() != "vec"
        || !path.segments[1].starts_with("impl")
        || path.segments[2].as_str() != "deref"
        || !model
            .proxy
            .as_ref()
            .is_some_and(|proxy| interfaces::spec_proxy(&proxy.x, &["std_specs", "vec"], "deref"))
    {
        return false;
    }
    let types: Vec<_> =
        model.typ_params.iter().map(|name| Arc::new(TypX::TypParam(name.clone()))).collect();
    let TypX::Decorate(TypDecoration::Ref, None, vector) = &*model.params[0].x.typ else {
        return false;
    };
    let Some(arguments) = collections::vec_args(vector) else { return false };
    if !arguments.iter().zip(&types).all(|(actual, expected)| same(actual, expected))
        || !same(
            &model.inner_ret.x.typ,
            &interfaces::reference(interfaces::primitive(Primitive::Slice, vec![types[0].clone()])),
        )
        || !matches!(&model.kind, FunctionKind::ForeignTraitMethodImpl {
            method, impl_path, trait_path, trait_typ_args,
        } | FunctionKind::TraitMethodImpl {
            method, impl_path, trait_path, trait_typ_args, inherit_body_from: None,
        } if models::path_is(&method.path, CrateId::Core, &["ops", "deref", "Deref", "deref"])
            && models::path_is(trait_path, CrateId::Core, &["ops", "deref", "Deref"])
            && impl_path == &Arc::new(PathX { krate: path.krate.clone(), segments: Arc::new(path.segments[..2].to_vec()) })
            && trait_typ_args.len() == 1 && same(&trait_typ_args[0], vector))
    {
        return false;
    }
    let mut sized = [0, 0];
    let mut allocator = 0;
    for bound in model.typ_bounds.iter() {
        match &**bound {
            GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized), args) if args.len() == 1 => {
                let Some(index) = types.iter().position(|typ| same(typ, &args[0])) else {
                    return false;
                };
                sized[index] += 1;
            }
            GenericBoundX::Trait(TraitId::Path(path), args)
                if args.len() == 1
                    && same(&args[0], &types[1])
                    && models::path_is(path, CrateId::Core, &["alloc", "Allocator"]) =>
            {
                allocator += 1;
            }
            _ => return false,
        }
    }
    sized == [1, 1] && allocator == 1
}

fn byte_vector(typ: &Typ) -> bool {
    collections::vec_args(arrays::readonly_base(typ)).is_some_and(|arguments| {
        matches!(&*arguments[0], TypX::Int(IntRange::U(8)))
            && matches!(&*arguments[1], TypX::Datatype(Dt::Path(path), args, impls)
                if models::path_is(path, CrateId::Alloc, &["alloc", "Global"])
                    && args.is_empty() && impls.is_empty())
    })
}

fn inert_place(place: &Place, destination: &VarIdent, parameters: &Params) -> bool {
    if matches!(&**arrays::readonly_base(&place.typ), TypX::MutRef(_)) {
        return false;
    }
    match &place.x {
        PlaceX::Local(name) => {
            name != destination
                && (!matches!(&*place.typ, TypX::Decorate(TypDecoration::Ref, None, _))
                    || parameters.iter().any(|parameter| &parameter.x.name == name))
        }
        PlaceX::Field(_, parent) => {
            !matches!(&*place.typ, TypX::Decorate(TypDecoration::Ref, None, _))
                && inert_place(parent, destination, parameters)
        }
        PlaceX::Temporary(expression) => inert_read(expression, destination, parameters),
        _ => false,
    }
}

fn inert_read(expression: &Expr, destination: &VarIdent, parameters: &Params) -> bool {
    matches!(&expression.x, ExprX::ReadPlace(place, _) if inert_place(place, destination, parameters))
}

impl Engine<'_> {
    fn borrowed_bytes(
        &mut self,
        state: &mut State,
        expression: &Expr,
        destination: &VarIdent,
    ) -> Result<Term, VirErr> {
        let bad = || {
            unsupported(
                &expression.span,
                "byte coercion requires an inert resolved Vec<u8, Global> borrow",
            )
        };
        let mut call = expression;
        while let ExprX::ReadPlace(place, _) = &call.x {
            let PlaceX::Temporary(inner) = &place.x else { return Err(bad()) };
            call = inner;
        }
        let ExprX::Call {
            target:
                CallTarget::Fun(
                    CallTargetKind::DynamicResolved {
                        resolved, typs, is_trait_default: false, ..
                    },
                    method,
                    types,
                    impls,
                    attributes,
                ),
            args,
            post_args: None,
            body: None,
        } = &call.x
        else {
            return Err(bad());
        };
        let Some(model) = self
            .all
            .get(resolved)
            .filter(|model| models::model(model) == Some(Model::BorrowedBytes))
        else {
            return Err(bad());
        };
        let (FunctionKind::ForeignTraitMethodImpl { method: expected, impl_path, .. }
        | FunctionKind::TraitMethodImpl {
            method: expected,
            impl_path,
            inherit_body_from: None,
            ..
        }) = &model.x.kind
        else {
            return Err(bad());
        };
        if method != expected
            || attributes.const_var
            || attributes.assume_external_allowed
            || attributes.autospec != AutospecUsage::Final
            || typs.len() != 2
            || types.len() != 1
            || args.len() != 1
            || !byte_vector(&types[0])
            || !byte_vector(&args[0].typ)
            || !matches!(&*args[0].typ, TypX::Decorate(TypDecoration::Ref, None, _))
            || !same(arrays::readonly_base(&args[0].typ), &types[0])
            || !collections::vec_args(&types[0]).is_some_and(|args| {
                args.iter().zip(typs.iter()).all(|(actual, expected)| same(actual, expected))
            })
            || !impls
                .iter()
                .any(|path| matches!(path, ImplPath::TraitImplPath(path) if path == impl_path))
            || !slice(&call.typ)
            || !inert_read(&args[0], destination, &self.f.x.params)
        {
            return Err(bad());
        }
        let value = self.expr(state, &args[0])?;
        let Node::Seq(contents) = value.node else { return Err(bad()) };
        Ok(contents)
    }

    pub(super) fn extend_bytes(
        &mut self,
        state: &mut State,
        expression: &Expr,
        arguments: &Exprs,
    ) -> Result<Val, VirErr> {
        let bad = || {
            unsupported(
                &expression.span,
                "byte append requires canonical u8/Global types, a local receiver and an immutable slice argument",
            )
        };
        let ExprX::Call {
            target: CallTarget::Fun(CallTargetKind::Static, _, types, _, attributes),
            ..
        } = &expression.x
        else {
            return Err(bad());
        };
        if types.len() != 2
            || !matches!(&*types[0], TypX::Int(IntRange::U(8)))
            || attributes.const_var
            || attributes.assume_external_allowed
            || !matches!(&*types[1], TypX::Datatype(Dt::Path(path), args, impls)
                if models::path_is(path, CrateId::Alloc, &["alloc", "Global"]) && args.is_empty() && impls.is_empty())
            || arguments.len() != 2
            || !slice(&arguments[1].typ)
            || !enums::unit(&expression.typ)
        {
            return Err(bad());
        }
        let (ExprX::BorrowMut(place) | ExprX::TwoPhaseBorrowMut(place)) = &arguments[0].x else {
            return Err(bad());
        };
        let PlaceX::Local(destination) = &place.x else { return Err(bad()) };
        if self.f.x.params.iter().any(|param| &param.x.name == destination) {
            return Err(bad());
        }
        let before = state.locals.get(destination).cloned().ok_or_else(bad)?;
        let Some(vector) = collections::vec_args(&before.typ) else { return Err(bad()) };
        if !vector.iter().zip(types.iter()).all(|(actual, expected)| same(actual, expected)) {
            return Err(bad());
        }
        let Node::Seq(prefix) = before.node else { return Err(bad()) };
        let suffix = if matches!(&arguments[1].x, ExprX::ReadPlace(tail, _)
            if matches!(&tail.x, PlaceX::Local(name) if name != destination))
        {
            let tail = self.expr(state, &arguments[1])?.scalar(&expression.span)?;
            self.lib(&expression.span, Model::SliceView, vec![types[0].clone()], vec![tail])?
        } else {
            self.borrowed_bytes(state, &arguments[1], destination)?
        };
        let result = self.lib(
            &expression.span,
            Model::Concat,
            vec![types[0].clone()],
            vec![prefix, suffix],
        )?;
        state.objects.remove(destination);
        state.locals.insert(
            destination.clone(),
            Val { identity: None, typ: before.typ, node: Node::Seq(result) },
        );
        Ok(Val::atom(exits::unit_value(&expression.span)?))
    }
}

#[cfg(test)]
mod tests;
