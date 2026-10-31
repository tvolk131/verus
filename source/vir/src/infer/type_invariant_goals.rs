//! Use an existing checked type invariant at ordinary executable entry.
//! This is the same operator as source use_type_invariant, not a new axiom.
//! Construction and mutable-borrow restoration checks remain in normal lowering.
use super::*;

pub(super) fn prepare(
    functions: &[Function],
    datatypes: &[Datatype],
    report: &mut GenerationReport,
) -> Vec<Function> {
    functions.iter().map(|f| {
        if f.x.mode!=Mode::Exec || f.x.attrs.infer || f.x.attrs.is_external_body
            || f.x.attrs.is_unsafe || f.x.attrs.is_async {return f.clone();}
        let Some(body)=&f.x.body else {return f.clone()};
        let mut stmts=vec![];
        for p in f.x.params.iter().filter(|p|p.x.mode==Mode::Exec) {
            let mut typ=p.x.typ.clone();
            let mut value=SpannedTyped::new(&p.span,&typ,ExprX::Var(p.x.name.clone()));
            if let TypX::MutRef(inner)=&*typ {
                value=SpannedTyped::new(&p.span,inner,ExprX::Unary(UnaryOp::MutRefCurrent,value));
                typ=inner.clone();
            }
            let typ=crate::ast_util::undecorate_typ(&typ);
            if crate::user_defined_type_invariants::check_typ_ok_for_use_typ_invariant(&p.span,&value.typ).is_err() {continue;}
            let TypX::Datatype(dt,..)=&*typ else {continue};
            if !datatypes.iter().any(|d|&d.x.name==dt && d.x.user_defined_invariant_fn.as_ref().is_some_and(|inv|
                f.x.owning_module.as_ref().is_some_and(|module|crate::ast_util::is_visible_to(&inv.visibility,module)))){continue;}
            let expr=SpannedTyped::new(&p.span,&crate::ast_util::unit_typ(),
                ExprX::AssertAssumeUserDefinedTypeInvariant{is_assume:true,expr:value,
                    fun:crate::fun!(CrateId::Internal=>"use_type_invariant_fake_placeholder_fun")});
            stmts.push(crate::def::Spanned::new(p.span.clone(),StmtX::Expr(expr)));
        }
        if stmts.is_empty(){return f.clone();}
        report.type_invariant_entries.push((f.x.name.clone(),stmts.len()));
        let mut x=f.x.clone();
        x.body=Some(SpannedTyped::new(&body.span,&body.typ,ExprX::Block(Arc::new(stmts),Some(body.clone()))));
        f.new_x(x)
    }).collect()
}
