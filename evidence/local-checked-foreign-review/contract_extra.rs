
#[cfg(test)]
mod independent_review_tests {
    use super::*;
    use crate::infer::tests::{function, input};
    #[test]
    fn nested_candidate_spans_types_semantics_source_and_dependencies() {
        let original = function("review_original", input("x"));
        let at = |t: &Typ, x| SpannedTyped::new(&original.span, t, x);
        let eq = |a,b| at(&bool_typ(), ExprX::Binary(BinaryOp::Eq(Mode::Spec), a,b));
        let nested = at(&bool_typ(), ExprX::If(boolean(&original.span,true).expr,
            eq(input("x"),input("x")), Some(boolean(&original.span,false).expr)));
        let install = |s: &Function, post: Expr, dep: &str| Contract::new(boolean(&s.span,true).expr,post).unwrap().install_lifted(s,dep).unwrap();
        let mut n=0;
        let relocated = crate::ast_visitor::map_expr_visitor(&nested, &|e| {
            let mut span=e.span.clone(); span.id+=100; span.as_string="review relocated nested node".into();
            Ok(SpannedTyped::new(&span,&e.typ,e.x.clone()))
        }).unwrap();
        let a=install(&original,nested.clone(),"dependencies=A");
        let b=install(&original,relocated.clone(),"dependencies=A");
        assert_eq!(a.x.attrs.infer_candidate_id,b.x.attrs.infer_candidate_id);
        assert!(Arc::ptr_eq(&a.x.ensure.0[0],&nested));
        assert!(Arc::ptr_eq(&b.x.ensure.0[0],&relocated));
        let changed_type=crate::ast_visitor::map_expr_visitor(&nested,&|e| {
            if matches!(&e.x,ExprX::Var(_)) {Ok(SpannedTyped::new(&e.span,&Arc::new(TypX::Int(IntRange::U(16))),e.x.clone()))} else {Ok(e.clone())}
        }).unwrap();
        assert_ne!(a.x.attrs.infer_candidate_id,install(&original,changed_type,"dependencies=A").x.attrs.infer_candidate_id);
        let changed_semantics=at(&bool_typ(),ExprX::If(boolean(&original.span,true).expr,
            eq(input("x"),input("y")),Some(boolean(&original.span,false).expr)));
        assert_ne!(a.x.attrs.infer_candidate_id,install(&original,changed_semantics,"dependencies=A").x.attrs.infer_candidate_id);
        assert_ne!(a.x.attrs.infer_candidate_id,install(&original,nested.clone(),"dependencies=B").x.attrs.infer_candidate_id);
        let mut source=original.x.clone();source.body=Some(input("y"));
        assert_ne!(a.x.attrs.infer_candidate_id,install(&original.new_x(source),nested,"dependencies=A").x.attrs.infer_candidate_id);
        let _=&mut n;
    }
}
