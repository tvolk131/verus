//! A deliberately bounded proof-only lowering of the original byte-buffer idiom.
//! Erasure uses the original body. Every mapped call carries its original callee
//! for DefId validation, and tracked arguments remain in Rust's borrow checking.
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{Expr, ItemFn, Pat, Stmt, parse_quote, spanned::Spanned};

fn unsupported<T: Spanned>(at: &T, why: &str) -> syn::Error {
    syn::Error::new(at.span(), format!("unsupported byte-buffer profile: {why}"))
}
fn local(stmt: &Stmt) -> Option<(&syn::Local, &Ident, &Expr)> {
    let Stmt::Local(l) = stmt else { return None };
    let Pat::Ident(p) = &l.pat else { return None };
    Some((l, &p.ident, &l.init.as_ref()?.expr))
}
fn path_is(e: &Expr, id: &Ident) -> bool {
    matches!(e,Expr::Path(p) if p.path.is_ident(id))
}
fn statements(e: &Expr) -> Option<&[Stmt]> {
    match e {
        Expr::Unsafe(u) => Some(&u.block.stmts),
        Expr::Block(b) => Some(&b.block.stmts),
        _ => None,
    }
}
fn statement_expr(s: &Stmt) -> Option<&Expr> {
    if let Stmt::Expr(e, _) = s { Some(e) } else { None }
}

pub fn expand(mut f: ItemFn, vstd: TokenStream) -> syn::Result<TokenStream> {
    let alloc_index=f.block.stmts.iter().position(|s|local(s).is_some_and(|(_,_,e)|
        matches!(e,Expr::Call(c) if matches!(&*c.func,Expr::Path(p) if p.path.segments.last().is_some_and(|p|p.ident=="with_capacity"))))
    ).ok_or_else(||unsupported(&f,"expected one fresh Vec::with_capacity"))?;
    if f.block.stmts.len() != alloc_index + 5 {
        return Err(unsupported(
            &f,
            "expected allocation, pointer, one forward loop, set_len, and return; aliases/reallocation/escape are outside this profile",
        ));
    }
    let (_, vector, alloc) = local(&f.block.stmts[alloc_index]).unwrap();
    let vector = vector.clone();
    let Expr::Call(alloc) = alloc else { unreachable!() };
    if alloc.args.len() != 1 {
        return Err(unsupported(alloc, "allocation arity"));
    }
    let request = alloc.args[0].clone();
    if !matches!(request, Expr::Path(_)) {
        return Err(unsupported(&request, "bind the capacity request before allocation"));
    }
    let Expr::Path(mut vec_new) = (*alloc.func).clone() else { unreachable!() };
    if vec_new.path.segments.len() != 2 {
        return Err(unsupported(&vec_new, "expected a direct Vec type path"));
    }
    if matches!(vec_new.path.segments[0].arguments, syn::PathArguments::None) {
        vec_new.path.segments[0].arguments =
            syn::PathArguments::AngleBracketed(parse_quote!(::<u8>));
    }
    let mut vec_ty = vec_new.path.clone();
    vec_ty.segments.pop();
    vec_ty.segments.pop_punct();
    let (_, pointer, acquire) = local(&f.block.stmts[alloc_index + 1])
        .ok_or_else(|| unsupported(&f, "expected a local pointer"))?;
    let pointer = pointer.clone();
    if !matches!(acquire,Expr::MethodCall(c) if c.method=="as_mut_ptr" && c.args.is_empty() && path_is(&c.receiver,&vector))
    {
        return Err(unsupported(acquire, "expected the fresh Vec's as_mut_ptr"));
    }
    let loop_expr = statement_expr(&f.block.stmts[alloc_index + 2])
        .ok_or_else(|| unsupported(&f, "expected a forward for loop"))?;
    let Expr::ForLoop(fl) = loop_expr else {
        return Err(unsupported(loop_expr, "expected a forward for loop"));
    };
    if !fl.attrs.is_empty() || fl.label.is_some() {
        return Err(unsupported(fl, "authored loop annotations/labels are outside this profile"));
    }
    let Pat::Ident(item) = &*fl.pat else {
        return Err(unsupported(fl, "expected one iterator item binding"));
    };
    let item = item.ident.clone();
    let source = (*fl.expr).clone();
    if !matches!(source, Expr::Path(_)) {
        return Err(unsupported(&source, "bind the source iterator before the loop"));
    }
    let body = if fl.body.stmts.len() == 1 {
        statement_expr(&fl.body.stmts[0]).and_then(statements).unwrap_or(&fl.body.stmts)
    } else {
        &fl.body.stmts
    };
    if body.len() != 2 {
        return Err(unsupported(fl, "loop must contain one pointer write and one pointer advance"));
    }
    let Some(Expr::Call(write)) = statement_expr(&body[0]) else {
        return Err(unsupported(fl, "expected ptr::write"));
    };
    if write.args.len() != 2 || !path_is(&write.args[0], &pointer) {
        return Err(unsupported(write, "expected a write through the current pointer"));
    }
    let Expr::Path(mut write_fn) = (*write.func).clone() else {
        return Err(unsupported(write, "expected a direct write function"));
    };
    if matches!(write_fn.path.segments.last().unwrap().arguments, syn::PathArguments::None) {
        write_fn.path.segments.last_mut().unwrap().arguments =
            syn::PathArguments::AngleBracketed(parse_quote!(::<u8>));
    }
    let value = write.args[1].clone();
    let fallible = matches!(&value,Expr::Try(t) if path_is(&t.expr,&item));
    if !fallible && !path_is(&value, &item) {
        return Err(unsupported(
            &value,
            "expected the current byte or byte?; aliased effects are unsupported",
        ));
    }
    let Some(Expr::Assign(advance)) = statement_expr(&body[1]) else {
        return Err(unsupported(fl, "expected pointer reassignment"));
    };
    if !path_is(&advance.left, &pointer) {
        return Err(unsupported(advance, "expected the current pointer assignment"));
    }
    let Expr::MethodCall(add) = &*advance.right else {
        return Err(unsupported(advance, "expected pointer add"));
    };
    if add.method != "add" || add.args.len() != 1 || !path_is(&add.receiver, &pointer) {
        return Err(unsupported(add, "expected pointer add on this allocation"));
    }
    let count = add.args[0].clone();
    let finish_stmt = statement_expr(&f.block.stmts[alloc_index + 3])
        .ok_or_else(|| unsupported(&f, "expected set_len"))?;
    let finish_expr = if let Some(ss) = statements(finish_stmt) {
        if ss.len() != 1 {
            return Err(unsupported(finish_stmt, "set_len block shape"));
        }
        statement_expr(&ss[0]).unwrap_or(finish_stmt)
    } else {
        finish_stmt
    };
    let Expr::MethodCall(finish) = finish_expr else {
        return Err(unsupported(finish_expr, "expected set_len"));
    };
    if finish.method != "set_len" || finish.args.len() != 1 || !path_is(&finish.receiver, &vector) {
        return Err(unsupported(finish, "expected set_len on the fresh Vec"));
    }
    let length = finish.args[0].clone();
    let tail = f.block.stmts.last().unwrap();
    let tail_text = tail.to_token_stream().to_string();
    if tail_text != quote!(Ok(#vector)).to_string() && tail_text != quote!(#vector).to_string() {
        return Err(unsupported(tail, "only the finished Vec may escape"));
    }
    let spare = Ident::new("VERUS_byte_spare", Span::mixed_site());
    let snapshot = Ident::new("VERUS_byte_input", Span::mixed_site());
    let iterator = Ident::new("VERUS_byte_iterator", Span::mixed_site());
    let binding = Ident::new("VERUS_byte_binding", Span::mixed_site());
    let relation =
        if fallible { quote!(Ok(#spare.contents()[i])) } else { quote!(#spare.contents()[i]) };
    let prefix_stmts = &f.block.stmts[..alloc_index];
    let lowered = quote!({
        use #vstd::std_specs::iter::IteratorSpec as _;
        #(#prefix_stmts)*
        let mut #vector = #vstd::byte_vec::native_allocate(#vec_new,#request);
        let (mut #pointer,VERUS_byte_lease) = #vstd::byte_vec::native_acquire(#vec_ty::as_mut_ptr,&mut #vector);
        proof_decl! {let tracked mut #spare = VERUS_byte_lease.get();}
        proof_decl! {let ghost #snapshot = #source; let ghost #binding = #spare;}
        #[verus_spec(#iterator =>
            invariant
                #spare.wf(),
                forall|v:&Vec<u8>| !#binding.matches(v) || #spare.matches(v),
                #spare.contents().len() == #iterator.index@,
                #spare.pointer(#pointer,#spare.contents().len() as int),
                #pointer@.addr as int == #spare.base()@.addr as int + #spare.contents().len(),
                #vstd::std_specs::iter::IteratorSpec::remaining(&#snapshot).len() <= #spare.capacity(),
                #iterator.seq() == #vstd::std_specs::iter::IteratorSpec::remaining(&#snapshot),
                forall|i:int|
                    #![all_triggers]
                    !(0<=i && i<#spare.contents().len()) || #vstd::std_specs::iter::IteratorSpec::remaining(&#snapshot)[i] == #relation,
        )]
        for #item in #source {
            #vstd::byte_vec::native_write(#write_fn,#pointer,#value,#vstd::prelude::verus_exec_expr!(Tracked(&mut #spare)));
            #pointer = #vstd::byte_vec::native_advance(<*mut u8>::add,#pointer,#count,#vstd::prelude::verus_exec_expr!(Tracked(&#spare)));
        }
        proof_decl! {let tracked VERUS_byte_receipt = #vstd::byte_vec::seal(#spare);}
        #vstd::byte_vec::native_finish(#vec_ty::set_len,&mut #vector,#length,#vstd::prelude::verus_exec_expr!(Tracked(VERUS_byte_receipt)));
        #tail
    });
    f.block = syn::parse2(lowered)?;
    // Erasure runs the ordinary annotation rewriter on the untouched input.
    f.attrs.push(parse_quote!(#[verifier::loop_isolation(false)]));
    Ok(quote!(#[verus_verify] #f))
}

/// A policy-vocabulary request for general checked two-byte prefix assistance.
/// Both query functions remain ordinary logical definitions. The library lemma's
/// universal precondition must verify before its pointwise consequence is used.
pub fn pair_semantics(
    mut f: ItemFn,
    paths: Vec<syn::Path>,
    vstd: TokenStream,
) -> syn::Result<TokenStream> {
    if paths.len() != 2 || f.sig.inputs.len() != 1 {
        return Err(unsupported(
            &f,
            "pair semantics expects two policy paths and one string input",
        ));
    }
    let Some(syn::FnArg::Typed(arg)) = f.sig.inputs.first() else {
        return Err(unsupported(&f, "pair semantics expects a shared str input"));
    };
    let Pat::Ident(name) = &*arg.pat else {
        return Err(unsupported(&f, "pair semantics input binding"));
    };
    let syn::Type::Reference(reference) = &*arg.ty else {
        return Err(unsupported(&f, "pair semantics expects &str"));
    };
    if reference.mutability.is_some()
        || !matches!(&*reference.elem,syn::Type::Path(p) if p.path.is_ident("str"))
    {
        return Err(unsupported(&f, "pair semantics expects &str"));
    }
    let input = &name.ident;
    let result_sequence = &paths[0];
    let digit = &paths[1];
    let stmt: Stmt = parse_quote! {
        proof! {
            #vstd::byte_vec::whole_pair_validity(
                #vstd::string::StringSliceAdditionalSpecFns::spec_bytes(#input),
                #result_sequence(#vstd::string::StringSliceAdditionalSpecFns::spec_bytes(#input)),
                |a:u8| #digit(a).is_some(),
            );
        }
    };
    f.block.stmts.insert(0, stmt);
    Ok(quote!(#[verus_verify] #f))
}
