//! Couple a first-party exact-hint opt-in to its selected native method body.
//! A logical capability definition alone is never evidence for executable hints.
use super::*;

fn trait_method(f: &Function, krate: CrateId, tail: &[&str], method_name: &str) -> Option<Typ> {
    let FunctionKind::TraitMethodImpl { trait_path, method, trait_typ_args, .. } = &f.x.kind else {
        return None;
    };
    (models::path_is(trait_path, krate, tail) && method.path.last_segment().as_str() == method_name)
        .then(|| trait_typ_args.first().cloned())
        .flatten()
}
pub fn opt_in(f: &Function) -> bool {
    trait_method(f, CrateId::Vstd, &["std_specs", "iter", "IteratorSpec"], "has_exact_size_hint")
        .is_some()
        && f.x.name.path.krate != CrateId::Vstd
        && !matches!(&f.x.kind,FunctionKind::TraitMethodImpl{inherit_body_from:Some(source),..} if source.path.krate==CrateId::Vstd)
        && !f.x.body.as_ref().is_some_and(|b| {
            let mut e = b;
            while let ExprX::Block(stmts, Some(tail)) = &e.x {
                if !stmts.is_empty() {
                    break;
                }
                e = tail;
            }
            matches!(&e.x, ExprX::Const(Constant::Bool(false)))
        })
}
pub fn validate(functions: &[Function]) -> Result<(), VirErr> {
    for law in functions.iter().filter(|f| opt_in(f)) {
        let typ = trait_method(
            law,
            CrateId::Vstd,
            &["std_specs", "iter", "IteratorSpec"],
            "has_exact_size_hint",
        )
        .unwrap();
        for method in ["next", "size_hint"] {
            let checked = functions.iter().any(|f| {
                trait_method(f, CrateId::Core, &["iter", "traits", "iterator", "Iterator"], method)
                    .is_some_and(|self_ty| crate::ast_util::types_equal(&self_ty, &typ))
                    && f.x.mode == Mode::Exec
                    && f.x.body.is_some()
                    && !f.x.attrs.is_external_body
            });
            if !checked {
                return Err(error(
                    &law.span,
                    format!(
                        "checked exact-size-hint opt-in requires the actual selected {method} implementation body"
                    ),
                ));
            }
        }
    }
    Ok(())
}
