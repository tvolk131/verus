//! Checked comparison adapters for transparent scalar tuple wrappers.
//! Runtime bodies are retained. Generated extension specs are proved by the
//! actual implementations; no comparison law is assumed from a trait name.
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, FnArg, ImplItem, ItemImpl, Pat, Stmt};

fn bad<T: quote::ToTokens>(item: T) -> syn::Error {
    syn::Error::new_spanned(
        item,
        "Verus Infer comparison impl requires a nongeneric scalar tuple wrapper, shared receivers, direct self.0/other.0 comparisons, and PartialOrd::partial_cmp delegating to the fields",
    )
}
fn field(e: &Expr, name: &str) -> bool {
    matches!(e,Expr::Field(f) if matches!(&f.member,syn::Member::Unnamed(i) if i.index==0)
        && matches!(&*f.base,Expr::Path(p) if p.qself.is_none() && p.path.is_ident(name)))
}
fn method(f: &syn::ImplItemFn) -> syn::Result<(syn::Ident, &Expr)> {
    if f.sig.unsafety.is_some()
        || f.sig.asyncness.is_some()
        || f.sig.constness.is_some()
        || f.sig.abi.is_some()
        || !f.sig.generics.params.is_empty()
        || f.sig.generics.where_clause.is_some()
        || f.sig.inputs.len() != 2
        || f.block.stmts.len() != 1
        || !matches!(&f.sig.inputs[0],FnArg::Receiver(r) if r.reference.is_some() && r.mutability.is_none() && r.colon_token.is_none())
    {
        return Err(bad(f));
    }
    let FnArg::Typed(arg) = &f.sig.inputs[1] else {
        return Err(bad(f));
    };
    let Pat::Ident(param) = &*arg.pat else {
        return Err(bad(arg));
    };
    if param.by_ref.is_some()
        || param.mutability.is_some()
        || param.subpat.is_some()
        || !matches!(&*arg.ty,syn::Type::Reference(r) if r.mutability.is_none())
    {
        return Err(bad(arg));
    }
    let Stmt::Expr(body, None) = &f.block.stmts[0] else {
        return Err(bad(f));
    };
    Ok((param.ident.clone(), body))
}
pub(crate) fn expand(mut item: ItemImpl, vstd: TokenStream) -> syn::Result<TokenStream> {
    if item.unsafety.is_some()
        || !item.generics.params.is_empty()
        || item.generics.where_clause.is_some()
        || item.defaultness.is_some()
    {
        return Err(bad(item));
    }
    let Some((None, path, _)) = &item.trait_ else {
        return Err(bad(item));
    };
    let segment = path.segments.last().ok_or_else(|| bad(path))?;
    if !matches!(segment.arguments, syn::PathArguments::None) {
        return Err(bad(path));
    }
    let is_eq = segment.ident == "PartialEq";
    if !(is_eq || segment.ident == "PartialOrd") {
        return Err(bad(path));
    }
    let self_ty = &item.self_ty;
    if !matches!(&**self_ty,syn::Type::Path(p) if p.qself.is_none() && p.path.segments.iter().all(|s|matches!(s.arguments,syn::PathArguments::None)))
    {
        return Err(bad(self_ty));
    }
    let mut models = None;
    let mut comparisons = 0;
    let mut partial_cmp = 0;
    for member in &mut item.items {
        let ImplItem::Fn(f) = member else {
            return Err(bad(member));
        };
        let (param, body) = method(f)?;
        let name = f.sig.ident.to_string();
        if !is_eq && name == "partial_cmp" {
            let Expr::MethodCall(call) = body else {
                return Err(bad(body));
            };
            if call.method != "partial_cmp"
                || call.turbofish.is_some()
                || !field(&call.receiver, "self")
                || call.args.len() != 1
                || !matches!(&call.args[0],Expr::Reference(r) if r.mutability.is_none() && field(&r.expr,&param.to_string()))
            {
                return Err(bad(body));
            }
            models = Some(quote! {
                impl #vstd::std_specs::cmp::PartialOrdSpecImpl for #self_ty {
                    open spec fn obeys_partial_cmp_spec()->bool{true}
                    open spec fn partial_cmp_spec(&self,#param:&Self)->Option<::core::cmp::Ordering>{
                        #vstd::std_specs::cmp::PartialOrdSpec::partial_cmp_spec(&self.0,&#param.0)
                    }
                }
            });
            partial_cmp += 1;
        } else {
            if !(if is_eq {
                name == "eq"
            } else {
                matches!(name.as_str(), "lt" | "le" | "gt" | "ge")
            }) {
                return Err(bad(f));
            }
            let Expr::Binary(binary) = body else {
                return Err(bad(body));
            };
            if !matches!(
                binary.op,
                syn::BinOp::Eq(_)
                    | syn::BinOp::Ne(_)
                    | syn::BinOp::Lt(_)
                    | syn::BinOp::Le(_)
                    | syn::BinOp::Gt(_)
                    | syn::BinOp::Ge(_)
            ) || !field(&binary.left, "self")
                || !field(&binary.right, &param.to_string())
            {
                return Err(bad(body));
            }
            if is_eq {
                models = Some(quote! {
                    impl #vstd::std_specs::cmp::PartialEqSpecImpl for #self_ty {
                        open spec fn obeys_eq_spec()->bool{true}
                        open spec fn eq_spec(&self,#param:&Self)->bool{#body}
                    }
                });
            }
            f.attrs.push(syn::parse_quote!(#[verifier::infer]));
            comparisons += 1;
        }
    }
    if (is_eq && (comparisons != 1 || item.items.len() != 1))
        || (!is_eq && (partial_cmp != 1 || !(1..=4).contains(&comparisons)))
    {
        return Err(bad(item));
    }
    Ok(quote! {
        #[#vstd::prelude::verus_verify] #item
        #vstd::prelude::verus!{#models}
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn comparisons_retain_bodies_and_reject_extra_effects() {
        for input in [
            quote!(impl PartialEq for Count{fn eq(&self,rhs:&Self)->bool{self.0==rhs.0}}),
            quote!(impl PartialOrd for Count{
                fn partial_cmp(&self,rhs:&Self)->Option<core::cmp::Ordering>{self.0.partial_cmp(&rhs.0)}
                fn lt(&self,rhs:&Self)->bool{self.0<rhs.0}
            }),
        ] {
            let original: ItemImpl = syn::parse2(input).unwrap();
            let before: Vec<_> = original
                .items
                .iter()
                .map(|m| match m {
                    ImplItem::Fn(f) => {
                        let b = &f.block;
                        quote!(#b).to_string()
                    }
                    _ => unreachable!(),
                })
                .collect();
            let out = expand(original, quote!(::vstd)).unwrap();
            assert!(!out.to_string().contains("external_body"));
            let parsed: syn::File = syn::parse2(out).unwrap();
            let syn::Item::Impl(out) = &parsed.items[0] else { panic!("missing original impl") };
            let after: Vec<_> = out
                .items
                .iter()
                .map(|m| match m {
                    ImplItem::Fn(f) => {
                        let b = &f.block;
                        quote!(#b).to_string()
                    }
                    _ => unreachable!(),
                })
                .collect();
            assert_eq!(before, after);
        }
        for input in [
            quote!(impl PartialEq for Count{fn eq(&self,rhs:&Self)->bool{self.0+1==rhs.0}}),
            quote!(impl PartialEq for Count{fn eq(&self,rhs:&Self)->bool{panic!();true}}),
            quote!(impl PartialOrd for Count{fn lt(&self,rhs:&Self)->bool{self.0<rhs.0}}),
        ] {
            assert!(expand(syn::parse2(input).unwrap(), quote!(::vstd)).is_err());
        }
    }
}
