use crate::context::Context;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::{BinOpKind, Expr, ExprKind};
use rustc_middle::ty::{IntTy, TyKind, TypingEnv, UintTy};
use rustc_span::def_id::DefId;
use vir::ast::Constant;

pub(crate) fn size_function(ctxt: &Context<'_>, id: DefId) -> bool {
    let path = ctxt.def_id_to_vir_path(id);
    path.krate == vir::ast::CrateId::Core
        && path.segments.iter().map(|segment| segment.as_str()).collect::<Vec<_>>()
            == ["mem", "size_of"]
        && ctxt.tcx.generics_of(id).count() == 1
}

pub(crate) fn fixed_size<'tcx>(
    ctxt: &Context<'tcx>,
    owner: DefId,
    ty: rustc_middle::ty::Ty<'tcx>,
) -> Option<u64> {
    if !matches!(
        ty.kind(),
        TyKind::Bool
            | TyKind::Uint(UintTy::U8 | UintTy::U16 | UintTy::U32 | UintTy::U64 | UintTy::U128)
            | TyKind::Int(IntTy::I8 | IntTy::I16 | IntTy::I32 | IntTy::I64 | IntTy::I128)
    ) {
        return None;
    }
    ctxt.tcx
        .layout_of(TypingEnv::post_analysis(ctxt.tcx, owner).as_query_input(ty))
        .ok()
        .map(|layout| layout.size.bytes())
}

fn portable_item(ctxt: &Context<'_>, id: DefId, depth: usize) -> Option<u64> {
    if depth > 64 || !id.is_local() || ctxt.tcx.generics_of(id).count() != 0 {
        return None;
    }
    let attrs = ctxt.tcx.hir_attrs(ctxt.tcx.local_def_id_to_hir_id(id.expect_local()));
    let attributes = ctxt.get_verifier_attrs(attrs).ok()?;
    if attributes.external_body
        || attributes.external_fn_specification
        || crate::attributes::get_mode_opt(attrs).is_some_and(|mode| mode != vir::ast::Mode::Exec)
    {
        return None;
    }
    let body = ctxt.tcx.hir_body_owned_by(id.expect_local());
    if !body.params.is_empty() {
        return None;
    }
    portable_expr(ctxt, id, body.value, depth + 1)
}

fn portable_expr<'tcx>(
    ctxt: &Context<'tcx>,
    owner: DefId,
    expr: &'tcx Expr<'tcx>,
    depth: usize,
) -> Option<u64> {
    if depth > 64 {
        return None;
    }
    let types = ctxt.tcx.typeck(owner.expect_local());
    if !matches!(types.expr_ty(expr).kind(), TyKind::Uint(UintTy::Usize)) {
        return None;
    }
    let value = match &expr.kind {
        ExprKind::Lit(literal) => match literal.node {
            rustc_ast::LitKind::Int(value, _) => u64::try_from(value.get()).ok()?,
            _ => return None,
        },
        ExprKind::Block(block, _) if block.stmts.is_empty() => {
            portable_expr(ctxt, owner, block.expr?, depth + 1)?
        }
        ExprKind::Binary(operation, left, right) => {
            let left = portable_expr(ctxt, owner, left, depth + 1)?;
            let right = portable_expr(ctxt, owner, right, depth + 1)?;
            match operation.node {
                BinOpKind::Add => left.checked_add(right)?,
                BinOpKind::Sub => left.checked_sub(right)?,
                BinOpKind::Mul => left.checked_mul(right)?,
                _ => return None,
            }
        }
        ExprKind::Path(path) => {
            let Res::Def(DefKind::Const { .. } | DefKind::AssocConst { .. }, id) =
                types.qpath_res(path, expr.hir_id)
            else {
                return None;
            };
            portable_item(ctxt, id, depth + 1)?
        }
        ExprKind::Call(callee, arguments) if arguments.is_empty() => {
            let TyKind::FnDef(id, arguments) = types.expr_ty(callee).kind() else { return None };
            let arguments = arguments.skip_binder();
            if size_function(ctxt, *id) && arguments.len() == 1 {
                fixed_size(ctxt, owner, arguments[0].expect_ty())?
            } else if arguments.is_empty() {
                portable_item(ctxt, *id, depth + 1)?
            } else {
                return None;
            }
        }
        _ => return None,
    };
    (value <= u32::MAX.into()).then_some(value)
}

pub(crate) fn scalar(ctxt: &Context<'_>, id: DefId) -> Option<Constant> {
    let source_id = id;
    let id = if id.is_local() && matches!(ctxt.tcx.def_kind(id), DefKind::Fn | DefKind::AssocFn) {
        let attrs = ctxt.tcx.hir_attrs(ctxt.tcx.local_def_id_to_hir_id(id.expect_local()));
        let attributes = ctxt.get_verifier_attrs(attrs).ok()?;
        if !attributes.encoded_const || !attributes.unerased_proxy {
            return None;
        }
        let name = ctxt.tcx.item_name(id);
        let name = name.as_str().strip_prefix("VERUS_UNERASED_PROXY__")?;
        let mut matches = ctxt.tcx.hir_body_owners().filter(|candidate| {
            let candidate = candidate.to_def_id();
            matches!(
                ctxt.tcx.def_kind(candidate),
                DefKind::Const { .. } | DefKind::AssocConst { .. }
            ) && ctxt.tcx.parent(candidate) == ctxt.tcx.parent(id)
                && ctxt.tcx.item_name(candidate).as_str() == name
        });
        let candidate = matches.next()?.to_def_id();
        if matches.next().is_some() {
            return None;
        }
        candidate
    } else {
        id
    };
    if !id.is_local()
        || !matches!(ctxt.tcx.def_kind(id), DefKind::Const { .. } | DefKind::AssocConst { .. })
        || ctxt.tcx.generics_of(id).count() != 0
    {
        return None;
    }
    let portable = portable_item(ctxt, id, 0)?;
    if portable_item(ctxt, source_id, 0)? != portable {
        return None;
    }
    let ty = ctxt.tcx.type_of(id).instantiate_identity().skip_norm_wip();
    if !matches!(ty.kind(), TyKind::Uint(UintTy::Usize)) {
        return None;
    }
    let value = ctxt.tcx.const_eval_poly(id).ok()?.try_to_scalar_int()?;
    if value.to_uint(value.size()) != u128::from(portable) {
        return None;
    }
    Some(Constant::Int(value.to_uint(value.size()).into()))
}
