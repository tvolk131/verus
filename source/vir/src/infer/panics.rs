//! Closed recognition of a literal-message panic. The existing panic model has
//! requires false; message construction is admitted only as a complete wrapper.
use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Model {
    Panic,
    Format,
}

pub(super) fn model(f: &Function) -> Option<Model> {
    let x = &f.x;
    if x.mode != Mode::Exec
        || !matches!(x.kind, FunctionKind::Static)
        || x.body.is_some()
        || x.params.len() != 1
        || !x.typ_params.is_empty()
        || !x.typ_bounds.is_empty()
        || x.attrs.infer
        || x.attrs.is_external_body
        || x.attrs.is_unsafe
        || x.attrs.is_async
        || x.attrs.autospec.is_some()
        || x.params[0].x.mode != Mode::Exec
        || x.params[0].x.user_mut
        || !x.ensure.0.is_empty()
        || !x.ensure.1.is_empty()
        || x.returns.is_some()
        || !crate::ast_util::types_equal(&x.inner_ret.x.typ, &x.outer_ret.x.typ)
    {
        return None;
    }
    let proxy = &x.proxy.as_ref()?.x;
    let fmt = |t: &Typ| {
        matches!(&**t,TypX::Datatype(Dt::Path(p),ts,_)
        if models::path_is(p,CrateId::Core,&["fmt","Arguments"]) && ts.is_empty())
    };
    if models::path_is(&x.name.path, CrateId::Core, &["panicking", "panic_fmt"])
        && models::spec_proxy(proxy, &["std_specs", "core"], "_panic__fmt")
        && fmt(&x.params[0].x.typ)
        && x.require.len() == 1
        && matches!(x.require[0].x, ExprX::Const(Constant::Bool(false)))
        && matches!(&*x.inner_ret.x.typ,TypX::Decorate(TypDecoration::Never,None,t) if enums::unit(t))
    {
        return Some(Model::Panic);
    }
    if x.name.path.krate == CrateId::Core
        && x.name.path.segments.len() == 3
        && x.name.path.segments[0].as_str() == "fmt"
        && x.name.path.segments[1].starts_with("impl")
        && x.name.path.last_segment().as_str() == "from_str"
        && models::spec_proxy(proxy, &["std_specs", "fmt"], "_from__str")
        && x.require.is_empty()
        && fmt(&x.inner_ret.x.typ)
        && matches!(&*x.params[0].x.typ,TypX::Decorate(TypDecoration::Ref,None,t)
            if matches!(&**t,TypX::Primitive(Primitive::StrSlice,ts) if ts.is_empty()))
    {
        return Some(Model::Format);
    }
    None
}

fn call<'a>(e: &'a Expr, all: &HashMap<Fun, Function>, kind: Model) -> Option<&'a Exprs> {
    let ExprX::Call {
        target: CallTarget::Fun(CallTargetKind::Static, n, ts, impls, attrs),
        args,
        post_args: None,
        body: None,
    } = &e.x
    else {
        return None;
    };
    (ts.is_empty()
        && impls.is_empty()
        && args.len() == 1
        && !attrs.const_var
        && !attrs.assume_external_allowed
        && matches!(attrs.autospec, AutospecUsage::Final)
        && all.get(n).and_then(model) == Some(kind))
    .then_some(args)
}

pub(super) fn literal(e: &Expr, all: &HashMap<Fun, Function>) -> Option<Vec<Expr>> {
    let args = call(e, all, Model::Panic)?;
    let message = call(&args[0], all, Model::Format)?;
    matches!(message[0].x, ExprX::Const(Constant::StrSlice(_)))
        .then(|| vec![e.clone(), args[0].clone()])
}
