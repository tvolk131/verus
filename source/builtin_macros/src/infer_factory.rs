//! A source adapter for checked finite iterator-view proofs. Executable erasure
//! returns the original item; the verification scaffold evaluates its receiver
//! and callback once, in source order, and invokes that same adapter method.
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Expr, FnArg, ItemFn, ReturnType, Stmt, spanned::Spanned};

pub(crate) fn expand(
    item: ItemFn,
    view: TokenStream,
    vstd: TokenStream,
) -> syn::Result<TokenStream> {
    let bad = || {
        syn::Error::new(
            item.span(),
            "iterator view proof requires a shared receiver and a single returned map/filter_map chain with a direct closure",
        )
    };
    if item.sig.unsafety.is_some()
        || item.sig.asyncness.is_some()
        || item.sig.constness.is_some()
        || item.sig.abi.is_some()
        || item.sig.inputs.len() != 1
        || item.block.stmts.len() != 1
        || !matches!(&item.sig.inputs[0],FnArg::Receiver(r) if r.reference.is_some() && r.mutability.is_none())
    {
        return Err(bad());
    }
    let Stmt::Expr(Expr::MethodCall(call), None) = &item.block.stmts[0] else {
        return Err(bad());
    };
    let helper = match call.method.to_string().as_str() {
        "map" => syn::Ident::new("prove_map_factory", Span::call_site()),
        "filter_map" => syn::Ident::new("prove_sparse_factory", Span::call_site()),
        _ => return Err(bad()),
    };
    if call.args.len() != 1 || !matches!(&call.args[0], Expr::Closure(_)) {
        return Err(bad());
    }
    let ReturnType::Type(arrow, typ) = &item.sig.output else {
        return Err(bad());
    };
    if !matches!(&**typ, syn::Type::ImplTrait(_)) {
        return Err(bad());
    }
    let ret = syn::Ident::new("__verus_infer_factory_result", Span::mixed_site());
    let input = syn::Ident::new("__verus_infer_factory_input", Span::mixed_site());
    let source = &call.receiver;
    let closure = &call.args[0];
    let method = &call.method;
    let turbofish = &call.turbofish;
    let attrs = &item.attrs;
    let vis = &item.vis;
    let mut sig = item.sig.clone();
    sig.output = ReturnType::Default;
    let where_clause = sig.generics.where_clause.take();
    Ok(quote! {#vstd::prelude::verus!{
        #(#attrs)* #vis #sig #arrow (#ret:#typ) #where_clause
          ensures #vstd::std_specs::iter_adapters::finite_native_view(#ret,#view),
        {
            let #input=#source;
            let #ret=#input.#method #turbofish (#closure);
            proof {#vstd::iterator_view::#helper(#input,#ret,#view);}
            #ret
        }
    }})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scaffold_retains_one_adapter_call_and_one_evaluation_of_each_argument() {
        for (method, helper) in
            [("map", "prove_map_factory"), ("filter_map", "prove_sparse_factory")]
        {
            let item:syn::ItemFn=syn::parse_str(&format!(
                "pub fn entries<'a>(&'a self)->impl Iterator<Item=u8>+'a where Self:Sized {{self.storage.iter().{method}(|value| *value)}}"
            )).unwrap();
            let tokens = expand(item, quote!(::vstd::prelude::Seq::empty()), quote!(::vstd))
                .unwrap()
                .to_string();
            assert_eq!(tokens.matches("self . storage . iter ()").count(), 1);
            assert_eq!(tokens.matches("| value | * value").count(), 1);
            assert_eq!(tokens.matches(&format!(". {method}")).count(), 1);
            assert!(tokens.contains(helper));
            assert!(tokens.contains("where Self : Sized"));
            assert!(!tokens.contains("external_body"));
            assert!(!tokens.contains("assume"));
        }
    }
    #[test]
    fn unsupported_source_shapes_fail_closed() {
        for source in [
            "fn iter(&mut self)->impl Iterator<Item=u8>{self.data.iter().map(|v|*v)}",
            "fn iter(&self)->impl Iterator<Item=u8>{let x=1;self.data.iter().map(|v|*v)}",
            "fn iter(&self)->impl Iterator<Item=u8>{self.data.iter().map(callback)}",
            "fn iter(&self)->impl Iterator<Item=u8>{self.data.iter().filter(|v|true)}",
            "unsafe fn iter(&self)->impl Iterator<Item=u8>{self.data.iter().map(|v|*v)}",
        ] {
            assert!(
                expand(syn::parse_str(source).unwrap(), quote!(self.view()), quote!(::vstd))
                    .is_err()
            );
        }
    }
}
