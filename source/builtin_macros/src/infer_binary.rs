//! Checked scalar-wrapper binary operator extensions. The actual body selects
//! the arithmetic relation; a trait's name never supplies an assumed meaning.
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, FnArg, ImplItem, ItemImpl, Pat, Stmt};

pub(crate) fn expand(mut item: ItemImpl, vstd: TokenStream) -> syn::Result<TokenStream> {
    let bad = |tokens: TokenStream| {
        syn::Error::new_spanned(
            tokens,
            "binary wrapper inference requires a nongeneric impl with Output = Self and a direct tuple constructor over self.0 and rhs or rhs.0",
        )
    };
    if item.unsafety.is_some()
        || item.defaultness.is_some()
        || !item.generics.params.is_empty()
        || item.generics.where_clause.is_some()
        || item.items.len() != 2
    {
        return Err(bad(quote!(#item)));
    }
    let Some((None, path, _)) = &item.trait_ else { return Err(bad(quote!(#item))) };
    let seg = path.segments.last().unwrap();
    let (name, method) = match seg.ident.to_string().as_str() {
        "Add" => ("Add", "add"),
        "Sub" => ("Sub", "sub"),
        "Mul" => ("Mul", "mul"),
        "Div" => ("Div", "div"),
        "Rem" => ("Rem", "rem"),
        _ => return Err(bad(quote!(#path))),
    };
    let rhs_ty = match &seg.arguments {
        syn::PathArguments::None => syn::parse_quote!(Self),
        syn::PathArguments::AngleBracketed(a) if a.args.len() == 1 => match &a.args[0] {
            syn::GenericArgument::Type(t) => t.clone(),
            _ => return Err(bad(quote!(#a))),
        },
        _ => return Err(bad(quote!(#seg))),
    };
    let self_ty = &item.self_ty;
    if !matches!(&**self_ty,syn::Type::Path(p) if p.qself.is_none() && p.path.segments.iter().all(|s|matches!(s.arguments,syn::PathArguments::None)))
    {
        return Err(bad(quote!(#self_ty)));
    }
    let mut output = false;
    let mut method_index = None;
    for (i, m) in item.items.iter().enumerate() {
        match m {
            ImplItem::Type(t)
                if t.ident == "Output"
                    && t.generics.params.is_empty()
                    && t.generics.where_clause.is_none()
                    && matches!(&t.ty,syn::Type::Path(p) if p.path.is_ident("Self")) =>
            {
                output = true
            }
            ImplItem::Fn(f) if f.sig.ident == method => method_index = Some(i),
            _ => return Err(bad(quote!(#m))),
        }
    }
    if !output {
        return Err(bad(quote!(#item)));
    }
    let Some(i) = method_index else { return Err(bad(quote!(#item))) };
    let ImplItem::Fn(f) = &mut item.items[i] else { unreachable!() };
    if f.sig.unsafety.is_some()
        || f.sig.asyncness.is_some()
        || f.sig.constness.is_some()
        || f.sig.abi.is_some()
        || !f.sig.generics.params.is_empty()
        || f.sig.generics.where_clause.is_some()
        || f.sig.inputs.len() != 2
        || f.block.stmts.len() != 1
        || !matches!(&f.sig.inputs[0],FnArg::Receiver(r) if r.reference.is_none() && r.mutability.is_none() && r.colon_token.is_none())
    {
        return Err(bad(quote!(#f)));
    }
    let FnArg::Typed(arg) = &f.sig.inputs[1] else { return Err(bad(quote!(#f))) };
    let Pat::Ident(p) = &*arg.pat else { return Err(bad(quote!(#arg))) };
    if p.by_ref.is_some() || p.mutability.is_some() || p.subpat.is_some() {
        return Err(bad(quote!(#arg)));
    }
    let param = &p.ident;
    let Stmt::Expr(Expr::Call(c), None) = &f.block.stmts[0] else { return Err(bad(quote!(#f))) };
    if c.args.len() != 1
        || !matches!(&*c.func,Expr::Path(p) if p.qself.is_none() && (p.path.is_ident("Self") || quote!(#p).to_string()==quote!(#self_ty).to_string()))
    {
        return Err(bad(quote!(#c)));
    }
    let Expr::Binary(b) = &c.args[0] else { return Err(bad(quote!(#c))) };
    let field = |e: &Expr, id: &str| matches!(e,Expr::Field(f) if matches!(&f.member,syn::Member::Unnamed(i) if i.index==0) && matches!(&*f.base,Expr::Path(p) if p.qself.is_none() && p.path.is_ident(id)));
    if !field(&b.left, "self")
        || !(field(&b.right, &param.to_string())
            || matches!(&*b.right,Expr::Path(p) if p.qself.is_none() && p.path.is_ident(param)))
    {
        return Err(bad(quote!(#b)));
    }
    let (body_trait, body_method) = match b.op {
        syn::BinOp::Add(_) => ("Add", "add"),
        syn::BinOp::Sub(_) => ("Sub", "sub"),
        syn::BinOp::Mul(_) => ("Mul", "mul"),
        syn::BinOp::Div(_) => ("Div", "div"),
        syn::BinOp::Rem(_) => ("Rem", "rem"),
        _ => return Err(bad(quote!(#b))),
    };
    let extension = format_ident!("{}SpecImpl", name);
    let obeys = format_ident!("obeys_{}_spec", method);
    let req = format_ident!("{}_req", method);
    let spec = format_ident!("{}_spec", method);
    let body_trait = format_ident!("{}Spec", body_trait);
    let body_req = format_ident!("{}_req", body_method);
    let body_spec = format_ident!("{}_spec", body_method);
    let rhs = &b.right;
    let model = quote! {
        impl #vstd::std_specs::ops::#extension<#rhs_ty> for #self_ty {
            open spec fn #obeys()->bool {true}
            open spec fn #req(self,#param:#rhs_ty)->bool {#vstd::std_specs::ops::#body_trait::#body_req(self.0,#rhs)}
            open spec fn #spec(self,#param:#rhs_ty)->Self {#self_ty(#vstd::std_specs::ops::#body_trait::#body_spec(self.0,#rhs))}
        }
    };
    f.attrs.push(syn::parse_quote!(#[verifier::infer]));
    Ok(quote! {#[#vstd::prelude::verus_verify] #item #vstd::prelude::verus!{#model}})
}
