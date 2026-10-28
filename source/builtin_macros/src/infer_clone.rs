//! Opt-in native derived Clone contracts for a sole-vector wrapper.
//! The Rust derive is retained. Its body must prove the generated view relation.
use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemStruct, spanned::Spanned};

pub(crate) fn expand(item: ItemStruct, vstd: TokenStream) -> syn::Result<TokenStream> {
    let bad = || {
        syn::Error::new(
            item.span(),
            "Clone view contract requires #[derive(Clone)] and one unnamed Vec field with a supplied direct View",
        )
    };
    if item.generics.params.len() > 2
        || item.generics.params.iter().any(|p| !matches!(p, syn::GenericParam::Type(_)))
    {
        return Err(bad());
    }
    let syn::Fields::Unnamed(fields) = &item.fields else { return Err(bad()) };
    if fields.unnamed.len() != 1 {
        return Err(bad());
    }
    let syn::Type::Path(typ) = &fields.unnamed[0].ty else { return Err(bad()) };
    let Some(seg) = typ.path.segments.last() else { return Err(bad()) };
    if seg.ident != "Vec"
        || !matches!(&seg.arguments,syn::PathArguments::AngleBracketed(a) if a.args.len()==1)
    {
        return Err(bad());
    }
    let mut clone = false;
    for attr in &item.attrs {
        if attr.path().is_ident("derive") {
            let entries = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )?;
            clone |= entries.iter().any(|p| p.segments.last().is_some_and(|s| s.ident == "Clone"));
        }
    }
    if !clone {
        return Err(bad());
    }
    let name = &item.ident;
    let (_, types, _) = item.generics.split_for_impl();
    let (view_params, _, view_where) = item.generics.split_for_impl();
    let mut generics = item.generics.clone();
    for param in generics.type_params_mut() {
        param.bounds.push(syn::parse_quote!(::core::clone::Clone));
    }
    let (params, _, where_clause) = generics.split_for_impl();
    Ok(quote! {#vstd::prelude::verus!{
     #item
     impl #view_params #name #types #view_where {
      pub broadcast proof fn __verus_infer_view_len_bound(&self)
       ensures #[trigger] self@.len() <= ::core::primitive::usize::MAX as int,
      { #vstd::std_specs::vec::axiom_spec_len(&self.0); }
     }
     impl #params #name #types #where_clause {
      pub open spec fn __verus_clone_relation(&self,result:Self)->bool{
       result@.len()==self@.len() &&
       forall|i:int| !(0<=i && i<self@.len()) || #vstd::pervasive::cloned(#[trigger] self@[i],result@[i])
      }
     }
    }})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_derive_and_public_view_relation_are_retained() {
        let input = syn::parse_quote!(
            #[derive(Clone)]
            pub struct Slots<T>(Vec<Option<T>>);
        );
        let tokens = expand(input, quote!(::vstd)).unwrap();
        assert!(tokens.to_string().contains("__verus_clone_relation"));
        assert!(!tokens.to_string().contains("external_body"));
        assert!(tokens.to_string().contains("derive (Clone)"));
        assert!(tokens.to_string().contains("pervasive :: cloned"));
    }
    #[test]
    fn unsupported_declarations_fail_closed() {
        for input in [
            quote!(
                pub struct Missing<T>(Vec<T>);
            ),
            quote!(
                #[derive(Clone)]
                pub struct Named<T> {
                    values: Vec<T>,
                }
            ),
            quote!(
                #[derive(Clone)]
                pub struct Pair<T>(Vec<T>, u8);
            ),
            quote!(
                #[derive(Clone)]
                pub struct Scalar(u64);
            ),
            quote!(
                #[derive(Clone)]
                pub struct Lifetimes<'a>(&'a Vec<u8>);
            ),
            quote!(
                #[derive(Clone)]
                pub struct Const<const N: usize>(Vec<u8>);
            ),
        ] {
            assert!(expand(syn::parse2(input).unwrap(), quote!(::vstd)).is_err());
        }
    }
}
