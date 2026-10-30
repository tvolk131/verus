//! Opt in actual Rust-derived scalar arithmetic and comparison implementations. Generated
//! extension definitions are candidates, checked against each native derive body.
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemStruct, spanned::Spanned};

pub(crate) fn expand(
    item: ItemStruct,
    operators: Vec<syn::Ident>,
    vstd: TokenStream,
) -> syn::Result<TokenStream> {
    let bad = || {
        syn::Error::new(
            item.span(),
            "infer_derive requires a nongeneric single primitive-integer tuple field and named derived From/Into/Add/Sub/Mul/Div/Rem/PartialEq/PartialOrd/Ord implementations",
        )
    };
    if operators.is_empty()
        || !item.generics.params.is_empty()
        || item.generics.where_clause.is_some()
    {
        return Err(bad());
    }
    let syn::Fields::Unnamed(fields) = &item.fields else { return Err(bad()) };
    if fields.unnamed.len() != 1 {
        return Err(bad());
    }
    let syn::Type::Path(t) = &fields.unnamed[0].ty else { return Err(bad()) };
    if t.qself.is_some()
        || t.path.segments.len() != 1
        || !matches!(
            t.path.segments[0].ident.to_string().as_str(),
            "u8" | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
        )
    {
        return Err(bad());
    }
    let mut derives = vec![];
    for attr in &item.attrs {
        if attr.path().is_ident("derive") {
            let paths = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )?;
            derives.extend(
                paths.iter().filter_map(|p| p.segments.last()).map(|s| s.ident.to_string()),
            );
        }
    }
    let name = &item.ident;
    let mut seen = vec![];
    let mut models = vec![];
    for op in operators {
        let method = match op.to_string().as_str() {
            "From" => "from",
            "Into" => "into",
            "Add" => "add",
            "Sub" => "sub",
            "Mul" => "mul",
            "Div" => "div",
            "Rem" => "rem",
            "PartialEq" => "eq",
            "PartialOrd" => "partial_cmp",
            "Ord" => "cmp",
            _ => return Err(bad()),
        };
        if !derives.contains(&op.to_string()) || seen.contains(&method) {
            return Err(bad());
        }
        seen.push(method);
        if method == "into" {
            // derive_more::Into emits From<Wrapper> for the primitive field.
            // The marker requests checking of that actual reverse conversion;
            // it does not certify the proposed extension definition.
            let field = &fields.unnamed[0].ty;
            models.push(quote! {
                impl #name {#[doc(hidden)] pub open spec fn __verus_infer_derived_into(&self)->bool {true}}
                impl #vstd::std_specs::convert::FromSpecImpl<#name> for #field {
                    open spec fn obeys_from_spec()->bool {true}
                    open spec fn from_spec(value:#name)->Self {value.0}
                }
            });
            continue;
        }
        if matches!(method, "eq" | "partial_cmp" | "cmp") {
            let marker = format_ident!("__verus_infer_derived_{}", method);
            let extension = format_ident!("{}SpecImpl", op);
            let base = format_ident!("{}Spec", op);
            let obeys = format_ident!("obeys_{}_spec", method);
            let spec = format_ident!("{}_spec", method);
            let (result, body) = match method {
                "eq" => (quote!(bool), quote!(self.0 == rhs.0)),
                "cmp" => (
                    quote!(::core::cmp::Ordering),
                    quote!(#vstd::std_specs::cmp::#base::#spec(&self.0,&rhs.0)),
                ),
                _ => (
                    quote!(Option<::core::cmp::Ordering>),
                    quote!(#vstd::std_specs::cmp::#base::#spec(&self.0,&rhs.0)),
                ),
            };
            models.push(quote! {
                impl #name {#[doc(hidden)] pub open spec fn #marker(&self)->bool {true}}
                impl #vstd::std_specs::cmp::#extension for #name {
                    open spec fn #obeys()->bool {true}
                    open spec fn #spec(&self,rhs:&Self)->#result {#body}
                }
            });
            continue;
        }
        if method == "from" {
            let field = &fields.unnamed[0].ty;
            models.push(quote!{
                impl #name {#[doc(hidden)] pub open spec fn __verus_infer_derived_from(&self)->bool {true}}
                impl #vstd::std_specs::convert::FromSpecImpl<#field> for #name {
                    open spec fn obeys_from_spec()->bool {true}
                    open spec fn from_spec(value:#field)->Self {#name(value)}
                }
            });
            continue;
        }
        let extension = format_ident!("{}SpecImpl", op);
        let base = format_ident!("{}Spec", op);
        let marker = format_ident!("__verus_infer_derived_{}", method);
        let obeys = format_ident!("obeys_{}_spec", method);
        let req = format_ident!("{}_req", method);
        let spec = format_ident!("{}_spec", method);
        models.push(quote!{
            impl #name {#[doc(hidden)] pub open spec fn #marker(&self)->bool {true}}
            impl #vstd::std_specs::ops::#extension for #name {
                open spec fn #obeys()->bool {true}
                open spec fn #req(self,rhs:Self)->bool {#vstd::std_specs::ops::#base::#req(self.0,rhs.0)}
                open spec fn #spec(self,rhs:Self)->Self {#name(#vstd::std_specs::ops::#base::#spec(self.0,rhs.0))}
            }
        });
    }
    if seen.contains(&"partial_cmp") && derives.iter().any(|d| d == "Ord") && !seen.contains(&"cmp")
    {
        return Err(syn::Error::new(
            item.span(),
            "infer_derive(PartialOrd) also requires Ord when the native derive delegates to Ord",
        ));
    }
    Ok(quote! {#vstd::prelude::verus!{#item #(#models)*}})
}
