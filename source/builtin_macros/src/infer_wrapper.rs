//! Small source adapter for tuple-wrapper trait implementations. All generated
//! models are ordinary Verus definitions; the original executable is retained
//! and receives the normal three inference checks, including trait-domain equality.
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, FnArg, ImplItem, ItemImpl, Pat, Stmt, spanned::Spanned};

fn error<T: quote::ToTokens>(item: T) -> syn::Error {
    syn::Error::new_spanned(
        item,
        "Verus Infer wrapper impl supports nongeneric From with a direct constructor/projection, or AddAssign with one self.0 += rhs[.0] statement",
    )
}

fn value(e: &Expr, param: &syn::Ident) -> bool {
    match e {
        Expr::Path(p) => p.qself.is_none() && p.path.is_ident(param),
        Expr::Field(f) => {
            matches!(&f.member,syn::Member::Unnamed(i) if i.index==0)
                && matches!(&*f.base,Expr::Path(p) if p.qself.is_none() && p.path.is_ident(param))
        }
        _ => false,
    }
}

pub(crate) fn expand(mut item: ItemImpl, vstd: TokenStream) -> syn::Result<TokenStream> {
    if item.trait_.as_ref().and_then(|(_,p,_)|p.segments.last())
        .is_some_and(|s|s.ident=="PartialEq" || s.ident=="PartialOrd") {
        return crate::infer_compare::expand(item,vstd);
    }
    if item.trait_.as_ref().and_then(|(_,p,_)|p.segments.last())
        .is_some_and(|s|matches!(s.ident.to_string().as_str(),"Add"|"Sub"|"Mul"|"Div"|"Rem")) {
        return crate::infer_binary::expand(item,vstd);
    }
    if item.unsafety.is_some()
        || !item.generics.params.is_empty()
        || item.generics.where_clause.is_some()
        || item.defaultness.is_some()
        || item.items.len() != 1
    {
        return Err(error(item));
    }
    let Some((None, trait_path, _)) = &item.trait_ else { return Err(error(item)) };
    let trait_name = trait_path.segments.last().ok_or_else(|| error(trait_path))?;
    let name = trait_name.ident.to_string();
    let arg = match &trait_name.arguments {
        syn::PathArguments::AngleBracketed(a) if a.args.len() == 1 => match &a.args[0] {
            syn::GenericArgument::Type(t) => Some(t.clone()),
            _ => return Err(error(a)),
        },
        syn::PathArguments::None => None,
        _ => return Err(error(trait_name)),
    };
    let self_ty = &item.self_ty;
    if !matches!(&**self_ty,syn::Type::Path(p) if p.qself.is_none() && p.path.segments.iter().all(|s|matches!(s.arguments,syn::PathArguments::None)))
    {
        return Err(error(self_ty));
    }
    let ImplItem::Fn(method) = &mut item.items[0] else { return Err(error(&item.items[0])) };
    if method.sig.unsafety.is_some()
        || method.sig.asyncness.is_some()
        || method.sig.constness.is_some()
        || method.sig.abi.is_some()
        || !method.sig.generics.params.is_empty()
        || method.sig.generics.where_clause.is_some()
        || method.block.stmts.len() != 1
    {
        return Err(error(method));
    }
    let Some(FnArg::Typed(input)) = method.sig.inputs.last() else { return Err(error(method)) };
    let Pat::Ident(param) = &*input.pat else { return Err(error(input)) };
    if param.by_ref.is_some() || param.mutability.is_some() || param.subpat.is_some() {
        return Err(error(param));
    }
    let param = &param.ident;
    let Stmt::Expr(body, _) = &method.block.stmts[0] else { return Err(error(&method.block)) };
    let model = if name == "From" && method.sig.ident == "from" && method.sig.inputs.len() == 1 {
        let Some(arg) = arg else { return Err(error(trait_path)) };
        let direct = value(body, param)
            || matches!(body,Expr::Call(c) if c.args.len()==1 && value(&c.args[0],param) && matches!(&*c.func,Expr::Path(p) if p.qself.is_none() && (p.path.is_ident("Self") || quote!(#p).to_string()==quote!(#self_ty).to_string())));
        if !direct {
            return Err(error(body));
        }
        quote! {
            impl #vstd::std_specs::convert::FromSpecImpl<#arg> for #self_ty {
                open spec fn obeys_from_spec()->bool {true}
                open spec fn from_spec(#param:#arg)->Self {#body}
            }
        }
    } else if name == "AddAssign"
        && method.sig.ident == "add_assign"
        && method.sig.inputs.len() == 2
    {
        if !matches!(&method.sig.inputs[0],FnArg::Receiver(r) if r.reference.is_some() && r.mutability.is_some() && r.colon_token.is_none())
        {
            return Err(error(&method.sig));
        }
        let Expr::Binary(b) = body else { return Err(error(body)) };
        let self_name = syn::Ident::new("self", body.span());
        if !matches!(b.op, syn::BinOp::AddAssign(_))
            || !matches!(&*b.left, Expr::Field(_))
            || !value(&b.left, &self_name)
            || !value(&b.right, param)
        {
            return Err(error(body));
        }
        let rhs = &b.right;
        let rhs_ty = arg.unwrap_or_else(|| syn::parse_quote!(Self));
        quote! {
            impl #vstd::std_specs::ops::AddAssignSpecImpl<#rhs_ty> for #self_ty {
                open spec fn obeys_add_assign_spec()->bool {true}
                open spec fn add_assign_req(&self,#param:#rhs_ty)->bool {
                    #vstd::std_specs::ops::AddSpec::add_req(self.0,#rhs)
                }
                open spec fn add_assign_spec(&self,#param:#rhs_ty)->&Self {
                    &#self_ty(#vstd::std_specs::ops::AddSpec::add_spec(self.0,#rhs))
                }
            }
        }
    } else {
        return Err(error(trait_path));
    };
    method.attrs.push(syn::parse_quote!(#[verifier::infer]));
    Ok(quote! {
        #[#vstd::prelude::verus_verify]
        #item
        #vstd::prelude::verus! { #model }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn direct_impls_generate_models_without_rewriting_executable_bodies() {
        for input in [
            quote!(impl From<usize> for Count {fn from(n:usize)->Self {Count(n)}}),
            quote!(impl From<Count> for usize {fn from(n:Count)->Self {n.0}}),
            quote!(impl core::ops::AddAssign for Count {fn add_assign(&mut self,n:Self) {self.0 += n.0;}}),
        ] {
            let original: syn::ItemImpl = syn::parse2(input).unwrap();
            let body = quote!(#original).to_string();
            let before = original
                .items
                .iter()
                .find_map(|item| match item {
                    ImplItem::Fn(f) => Some(&f.block),
                    _ => None,
                })
                .unwrap();
            let before = quote!(#before).to_string();
            let tokens = expand(original, quote!(::vstd)).unwrap();
            let expanded: syn::File = syn::parse2(tokens.clone()).unwrap();
            let syn::Item::Impl(implementation) = &expanded.items[0] else {
                panic!("missing original impl")
            };
            let ImplItem::Fn(method) = &implementation.items[0] else {
                panic!("missing original method")
            };
            let after = &method.block;
            assert_eq!(before, quote!(#after).to_string());
            let output = tokens.to_string();
            assert!(output.contains("verifier :: infer"));
            assert!(output.contains(if body.contains("AddAssign") {
                "add_assign_req"
            } else {
                "from_spec"
            }));
        }
    }
    #[test]
    fn unsupported_effects_and_trait_shapes_fail_explicitly() {
        for input in [
            quote!(impl From<usize> for Count {fn from(n:usize)->Self {Count(n+1)}}),
            quote!(impl core::ops::AddAssign for Count {fn add_assign(&mut self,n:Self) {self.0 -= n.0;}}),
            quote!(
                impl<T> From<T> for Count {
                    fn from(n: T) -> Self {
                        Count(n)
                    }
                }
            ),
            quote!(impl core::ops::AddAssign for Count {fn add_assign(&mut self,n:Self) {self.0 += n.0; self.0 += 1;}}),
        ] {
            assert!(expand(syn::parse2(input).unwrap(), quote!(::vstd)).is_err());
        }
    }
}
