//! Inventory authored syntax, before macro expansion or generated proof assistance.
//! This is not verification or a semantic classification of manual contracts.
use quote::ToTokens;
use serde_json::{Value, json};
use verus_syn::{
    self as syn,
    spanned::Spanned,
    visit::{self, Visit},
};
fn tok<T: ToTokens>(x: &T) -> String {
    x.to_token_stream().to_string()
}
fn attrs(a: &[syn::Attribute]) -> Vec<String> {
    a.iter().filter(|a| !a.path().is_ident("doc")).map(tok).collect()
}
fn is_test(a: &[String]) -> bool {
    a.iter().any(|s| s == "# [test]" || s == "# [cfg (test)]")
}
#[derive(Default)]
struct BodyAudit {
    findings: Vec<Value>,
    loops: usize,
}
impl BodyAudit {
    fn finding<T: Spanned>(&mut self, x: &T, kind: &str) {
        self.findings.push(json!({"line":x.span().start().line,"kind":kind}));
    }
}
impl<'a> Visit<'a> for BodyAudit {
    fn visit_item_fn(&mut self, f: &'a syn::ItemFn) {
        self.findings.push(json!({"line":f.span().start().line,
            "kind":"nested_function","signature":tok(&f.sig)}));
        visit::visit_item_fn(self, f);
    }
    fn visit_expr(&mut self, x: &'a syn::Expr) {
        match x {
            syn::Expr::Assume(_) => self.finding(x, "assume"),
            syn::Expr::Assert(_) | syn::Expr::AssertForall(_) => self.finding(x, "assert"),
            syn::Expr::RevealHide(_) => self.finding(x, "reveal_hide"),
            syn::Expr::Unary(u) if matches!(u.op, syn::UnOp::Proof(_)) => {
                self.finding(x, "proof_block")
            }
            syn::Expr::Verbatim(_) => self.finding(x, "unparsed_expression"),
            syn::Expr::ForLoop(l) => {
                self.loops += 1;
                if l.invariant.is_some()
                    || l.invariant_except_break.is_some()
                    || l.ensures.is_some()
                    || l.decreases.is_some()
                {
                    self.finding(x, "loop_annotation");
                }
            }
            syn::Expr::While(l) => {
                self.loops += 1;
                if l.invariant.is_some()
                    || l.invariant_except_break.is_some()
                    || l.invariant_ensures.is_some()
                    || l.ensures.is_some()
                    || l.decreases.is_some()
                {
                    self.finding(x, "loop_annotation");
                }
            }
            syn::Expr::Loop(l) => {
                self.loops += 1;
                if l.invariant.is_some()
                    || l.invariant_except_break.is_some()
                    || l.invariant_ensures.is_some()
                    || l.ensures.is_some()
                    || l.decreases.is_some()
                {
                    self.finding(x, "loop_annotation");
                }
            }
            _ => {}
        }
        visit::visit_expr(self, x);
    }
    fn visit_attribute(&mut self, a: &'a syn::Attribute) {
        // Inspect identifier tokens, including hints nested in native macro
        // attributes. Documentation/other string literals are not proof hints.
        let mut pending = vec![a.to_token_stream()];
        let mut hint = false;
        while let Some(tokens) = pending.pop() {
            for token in tokens {
                match token {
                    proc_macro2::TokenTree::Ident(name) => hint |= matches!(
                        name.to_string().as_str(),
                        "trigger" | "all_triggers" | "auto_trigger" | "no_auto_trigger" | "auto"
                    ),
                    proc_macro2::TokenTree::Group(group) => pending.push(group.stream()),
                    _ => {}
                }
            }
        }
        if hint {
            self.finding(a, "manual_trigger");
        }
        visit::visit_attribute(self, a);
    }
    fn visit_macro(&mut self, m: &'a syn::Macro) {
        self.findings.push(json!({"line":m.span().start().line,
            "kind":"macro_call","path":tok(&m.path),"tokens":tok(&m.tokens)}));
        if m.path.is_ident("proof") {
            self.finding(m, "proof_macro");
        }
        // Ordinary native assert! remains executable Rust, and is inventoried separately.
        if m.path.is_ident("assert") || m.path.is_ident("assert_eq") {
            self.finding(m, "native_assert_macro");
        }
        visit::visit_macro(self, m);
    }
}
struct Inventory {
    file: String,
    entries: Vec<Value>,
}
impl Inventory {
    fn function(
        &mut self,
        sig: &syn::Signature,
        own: &[syn::Attribute],
        inherited: &[String],
        owner: &str,
        body: Option<&syn::Block>,
        in_verus: bool,
    ) {
        let mut aa = inherited.to_vec();
        aa.extend(attrs(own));
        let mut audit = BodyAudit::default();
        if let Some(b) = body {
            audit.visit_block(b);
        }
        for a in own {
            audit.visit_attribute(a);
        }
        audit.visit_signature(sig);
        let mode = match sig.mode {
            syn::FnMode::Spec(_) | syn::FnMode::SpecChecked(_) => "spec",
            syn::FnMode::Proof(_) | syn::FnMode::ProofAxiom(_) => "proof",
            _ => "exec",
        };
        self.entries.push(json!({"kind":"function","file":self.file,"owner":owner,"name":sig.ident.to_string(),"line":sig.fn_token.span.start().line,"end_line":body.map(|b|b.span().end().line),"mode":mode,"in_verus":in_verus,"test":is_test(&aa),"attributes":aa,"signature":tok(sig),"authored_signature_spec":tok(&sig.spec),"has_body":body.is_some(),"body":body.map(tok),"loops":audit.loops,"findings":audit.findings}));
    }
    fn items(&mut self, items: &[syn::Item], inherited: &[String], owner: &str, in_verus: bool) {
        for item in items {
            match item {
                syn::Item::Fn(f)=>self.function(&f.sig,&f.attrs,inherited,owner,Some(&f.block),in_verus),
                syn::Item::Impl(i)=>{
                    let mut aa=inherited.to_vec(); aa.extend(attrs(&i.attrs));
                    let local=match &i.trait_ {Some((_,t,_))=>format!("{} as {}",tok(&i.self_ty),tok(t)), None=>tok(&i.self_ty)};
                    let o=if owner.is_empty(){local}else{format!("{}::{}",owner,local)};
                    for m in &i.items {match m {
                        syn::ImplItem::Fn(f)=>self.function(&f.sig,&f.attrs,&aa,&o,Some(&f.block),in_verus),
                        syn::ImplItem::Macro(m) if m.mac.path.is_ident("verus_trait_impl") => {
                            let inner:syn::File=syn::parse2(m.mac.tokens.clone()).expect("verus_trait_impl parse");
                            self.items(&inner.items,&aa,&o,true);
                        },
                        syn::ImplItem::Macro(m)=>self.entries.push(json!({"kind":"opaque_impl_macro","file":self.file,"owner":o,"line":m.span().start().line,"definition":tok(m)})),
                        _=>{}
                    }}
                },
                syn::Item::Trait(t)=>{
                    self.entries.push(json!({"kind":"trait","file":self.file,"owner":owner,"name":t.ident.to_string(),"line":t.span().start().line,"attributes":attrs(&t.attrs),"definition":tok(t)}));
                    let mut aa=inherited.to_vec();aa.extend(attrs(&t.attrs));
                    let o=format!("{}{}trait {}",owner,if owner.is_empty(){""}else{"::"},t.ident);
                    for m in &t.items {match m {
                        syn::TraitItem::Fn(f)=>self.function(&f.sig,&f.attrs,&aa,&o,f.default.as_ref(),in_verus),
                        syn::TraitItem::Macro(m)=>self.entries.push(json!({"kind":"opaque_trait_macro","file":self.file,"owner":o,"line":m.span().start().line,"definition":tok(m)})),
                        _=>{}
                    }}
                },
                syn::Item::Mod(m)=>{
                    if let Some((_,items))=&m.content {let mut aa=inherited.to_vec();aa.extend(attrs(&m.attrs));self.items(items,&aa,&format!("{}{}{}",owner,if owner.is_empty(){""}else{"::"},m.ident),in_verus);}
                    else {self.entries.push(json!({"kind":"external_module","file":self.file,"owner":owner,"name":m.ident.to_string(),"line":m.span().start().line,"attributes":attrs(&m.attrs)}));}
                },
                syn::Item::Macro(m) if m.mac.path.is_ident("verus")=>{
                    let inner:syn::File=syn::parse2(m.mac.tokens.clone()).expect("verus parse");
                    self.items(&inner.items,inherited,owner,true);
                },
                syn::Item::Macro(m)=>self.entries.push(json!({"kind":"opaque_item_macro","file":self.file,"owner":owner,"line":m.span().start().line,"definition":tok(m)})),
                syn::Item::AssumeSpecification(a)=>self.entries.push(json!({"kind":"assumed_specification","file":self.file,"owner":owner,"name":tok(&a.path),"line":a.span().start().line,"definition":tok(a),"ensures":a.ensures.as_ref().map(tok),"requires":a.requires.as_ref().map(tok)})),
                syn::Item::Struct(s)=>self.entries.push(json!({"kind":"struct","file":self.file,"owner":owner,"name":s.ident.to_string(),"line":s.span().start().line,"attributes":attrs(&s.attrs),"definition":tok(s)})),
                syn::Item::Enum(e)=>self.entries.push(json!({"kind":"enum","file":self.file,"owner":owner,"name":e.ident.to_string(),"line":e.span().start().line,"attributes":attrs(&e.attrs),"definition":tok(e)})),
                syn::Item::Use(_) | syn::Item::ExternCrate(_) => {},
                _=>{
                    let mut audit=BodyAudit::default();audit.visit_item(item);
                    self.entries.push(json!({"kind":"other_item","file":self.file,"owner":owner,"line":item.span().start().line,"definition":tok(item),"findings":audit.findings}));
                },
            }
        }
    }
}
fn main() {
    let root = std::env::args().nth(1).expect("source root");
    let paths: Vec<_> = std::env::args().skip(2).collect();
    assert!(!paths.is_empty(), "supply at least one source file");
    let mut out = vec![];
    for p in paths {
        let source =
            std::fs::read_to_string(std::path::Path::new(&root).join(&p)).expect("source read");
        let file = syn::parse_file(&source).unwrap_or_else(|e| panic!("{}: {}", p, e));
        let mut inv = Inventory { file: p, entries: vec![] };
        inv.items(&file.items, &attrs(&file.attrs), "", false);
        out.extend(inv.entries);
    }
    println!("{}", serde_json::to_string_pretty(&json!({
        "schema_version":1,
        "coverage":"authored syntax in the explicitly supplied source files",
        "limitations":[
            "No macro expansion, cfg evaluation, external-module discovery, type checking or proof checking is performed.",
            "Unrecognized item/trait/impl macros and every function-body macro call remain visible for manual review.",
            "Only explicit source verus!/verus_trait_impl! syntax is opened; generated contracts and compiler assistance are not authored input.",
            "Contract meaning, selected proof closure, trusted interfaces and acceptable manual input require review alongside compiler reports."
        ],
        "entries":out
    })).unwrap());
}
