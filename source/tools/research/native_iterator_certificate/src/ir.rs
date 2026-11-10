//! Typed evidence encoding, not a Rust memory semantics or a proof-producing lowering.
use rustc_middle::{
    mir,
    ty::{self, TyCtxt},
};
use serde_json::{Value, json};

fn binary_tag(op: mir::BinOp) -> &'static str {
    match op {
        mir::BinOp::Add => "Add",
        mir::BinOp::Sub => "Sub",
        mir::BinOp::Mul => "Mul",
        mir::BinOp::Lt => "Lt",
        mir::BinOp::Le => "Le",
        mir::BinOp::Gt => "Gt",
        mir::BinOp::Ge => "Ge",
        mir::BinOp::Eq => "Eq",
        mir::BinOp::Ne => "Ne",
        mir::BinOp::BitXor => "BitXor",
        mir::BinOp::BitOr => "BitOr",
        mir::BinOp::BitAnd => "BitAnd",
        mir::BinOp::Offset => "Offset",
        mir::BinOp::AddWithOverflow => "AddWithOverflow",
        mir::BinOp::SubWithOverflow => "SubWithOverflow",
        _ => "unsupported_binary",
    }
}

fn cast_tag(kind: mir::CastKind) -> &'static str {
    match kind {
        mir::CastKind::Transmute => "Transmute",
        mir::CastKind::PtrToPtr => "PtrToPtr",
        mir::CastKind::IntToInt => "IntToInt",
        mir::CastKind::PointerCoercion(ty::adjustment::PointerCoercion::Unsize, _) => "Unsize",
        _ => "unsupported_cast",
    }
}

fn unary_tag(op: mir::UnOp) -> &'static str {
    match op {
        mir::UnOp::PtrMetadata => "PtrMetadata",
        mir::UnOp::Not => "Not",
        mir::UnOp::Neg => "Neg",
    }
}

fn unwind(u: mir::UnwindAction) -> Value {
    match u {
        mir::UnwindAction::Continue => json!({"action":"continue"}),
        mir::UnwindAction::Unreachable => json!({"action":"unreachable"}),
        mir::UnwindAction::Terminate(reason) => {
            json!({"action":"terminate","reason":format!("{reason:?}")})
        }
        mir::UnwindAction::Cleanup(target) => {
            json!({"action":"cleanup","target":target.as_usize()})
        }
    }
}

fn assertion<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    msg: &mir::AssertMessage<'tcx>,
) -> Value {
    match msg {
        mir::AssertKind::BoundsCheck { len, index } => {
            json!({"op":"bounds_check","length":operand(tcx,body,len),"index":operand(tcx,body,index)})
        }
        mir::AssertKind::Overflow(op, a, b) => {
            json!({"op":"overflow","kind":binary_tag(*op),"left":operand(tcx,body,a),"right":operand(tcx,body,b)})
        }
        _ => json!({"op":"unsupported_assertion","debug":format!("{msg:?}")}),
    }
}

pub fn place<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>, p: mir::Place<'tcx>) -> Value {
    let projections: Vec<_> = p.projection.iter().map(|e| match e {
        mir::ProjectionElem::Deref => json!({"op":"deref"}),
        mir::ProjectionElem::Field(f,t) => json!({"op":"field","field":f.as_usize(),"ty":t.to_string()}),
        mir::ProjectionElem::Downcast(_,v) => json!({"op":"downcast","variant":v.as_usize()}),
        mir::ProjectionElem::Index(l) => json!({"op":"index","local":l.as_usize()}),
        mir::ProjectionElem::ConstantIndex {offset,min_length,from_end} => json!({"op":"constant_index","offset":offset,"min_length":min_length,"from_end":from_end}),
        other => json!({"op":"unsupported_projection","debug":format!("{other:?}")}),
    }).collect();
    json!({"local":p.local.as_usize(),"projection":projections,"ty":p.ty(&body.local_decls,tcx).ty.to_string()})
}

pub fn operand<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>, o: &mir::Operand<'tcx>) -> Value {
    match o {
        mir::Operand::Copy(p) => json!({"op":"copy","place":place(tcx,body,*p)}),
        mir::Operand::Move(p) => json!({"op":"move","place":place(tcx,body,*p)}),
        mir::Operand::RuntimeChecks(checks) => {
            json!({"op":"runtime_checks","value":checks.value(tcx.sess),"debug":format!("{checks:?}")})
        }
        mir::Operand::Constant(c) => {
            let env = ty::TypingEnv::fully_monomorphized();
            let bits = c.const_.try_eval_bits(tcx, env).map(|b| b.to_string());
            let evaluated = c.const_.eval(tcx, env, c.span);
            let zero_sized =
                tcx.layout_of(env.as_query_input(c.ty())).is_ok_and(|l| l.size.bytes() == 0);
            json!({"op":"constant","ty":c.ty().to_string(),"literal":format!("{:?}",c.const_),
                "bits":bits,"zero_sized":zero_sized,"evaluated":format!("{evaluated:?}"),"evaluation_ok":evaluated.is_ok()})
        }
    }
}

fn rvalue<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>, r: &mir::Rvalue<'tcx>) -> Value {
    match r {
        mir::Rvalue::Use(o, retag) => {
            json!({"op":"use","operand":operand(tcx,body,o),"retag":format!("{retag:?}")})
        }
        mir::Rvalue::Ref(_, kind, p) => {
            json!({"op":"ref","kind":format!("{kind:?}"),"mutable":matches!(kind,mir::BorrowKind::Mut{..}),"fake":matches!(kind,mir::BorrowKind::Fake(_)),"place":place(tcx,body,*p)})
        }
        mir::Rvalue::RawPtr(kind, p) => {
            json!({"op":"raw_ptr","kind":format!("{kind:?}"),"place":place(tcx,body,*p)})
        }
        mir::Rvalue::Cast(kind, o, t) => {
            json!({"op":"cast","kind":cast_tag(*kind),"operand":operand(tcx,body,o),"to":t.to_string()})
        }
        mir::Rvalue::BinaryOp(op, operands) => {
            json!({"op":"binary","kind":binary_tag(*op),"left":operand(tcx,body,&operands.0),"right":operand(tcx,body,&operands.1)})
        }
        mir::Rvalue::UnaryOp(op, o) => {
            json!({"op":"unary","kind":unary_tag(*op),"operand":operand(tcx,body,o)})
        }
        mir::Rvalue::Discriminant(p) => json!({"op":"discriminant","place":place(tcx,body,*p)}),
        mir::Rvalue::Aggregate(kind, operands) => {
            let k = match **kind {
                mir::AggregateKind::Tuple => json!({"op":"tuple"}),
                mir::AggregateKind::Adt(id, variant, args, _, _) => {
                    json!({"op":"adt","definition":tcx.def_path_str(id),"identity":format!("{:?}",tcx.def_path_hash(id)),"variant":variant.as_usize(),"args":format!("{args:?}")})
                }
                _ => json!({"op":"unsupported_aggregate","debug":format!("{kind:?}")}),
            };
            json!({"op":"aggregate","kind":k,"operands":operands.iter().map(|o|operand(tcx,body,o)).collect::<Vec<_>>()})
        }
        other => json!({"op":"unsupported_rvalue","debug":format!("{other:?}")}),
    }
}

pub fn statement<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    s: &mir::StatementKind<'tcx>,
) -> Value {
    match s {
        mir::StatementKind::Assign(a) => {
            json!({"op":"assign","destination":place(tcx,body,a.0),"value":rvalue(tcx,body,&a.1)})
        }
        mir::StatementKind::StorageLive(l) => json!({"op":"storage_live","local":l.as_usize()}),
        mir::StatementKind::StorageDead(l) => json!({"op":"storage_dead","local":l.as_usize()}),
        mir::StatementKind::Nop => json!({"op":"nop"}),
        mir::StatementKind::SetDiscriminant { place: p, variant_index } => {
            json!({"op":"set_discriminant","place":place(tcx,body,**p),"variant":variant_index.as_usize()})
        }
        mir::StatementKind::Intrinsic(i) => match &**i {
            mir::NonDivergingIntrinsic::Assume(o) => {
                json!({"op":"assume","operand":operand(tcx,body,o)})
            }
            _ => json!({"op":"unsupported_intrinsic","debug":format!("{i:?}")}),
        },
        other => json!({"op":"unsupported_statement","debug":format!("{other:?}")}),
    }
}

pub fn terminator<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    t: &mir::TerminatorKind<'tcx>,
) -> Value {
    match t {
        mir::TerminatorKind::Goto { target } => json!({"op":"goto","target":target.as_usize()}),
        mir::TerminatorKind::SwitchInt { discr, targets } => {
            json!({"op":"switch","operand":operand(tcx,body,discr),
            "cases":targets.iter().map(|(v,b)|json!({"value":v.to_string(),"target":b.as_usize()})).collect::<Vec<_>>(),"otherwise":targets.otherwise().as_usize()})
        }
        mir::TerminatorKind::Return => json!({"op":"return"}),
        mir::TerminatorKind::Unreachable => json!({"op":"unreachable"}),
        mir::TerminatorKind::UnwindResume => json!({"op":"unwind_resume"}),
        mir::TerminatorKind::UnwindTerminate(reason) => {
            json!({"op":"unwind_terminate","reason":format!("{reason:?}")})
        }
        mir::TerminatorKind::Call { func, args, destination, target, unwind: u, .. } => {
            json!({"op":"call","function":operand(tcx,body,func),
            "arguments":args.iter().map(|a|operand(tcx,body,&a.node)).collect::<Vec<_>>(),"destination":place(tcx,body,*destination),
            "target":target.map(|b|b.as_usize()),"unwind":unwind(*u)})
        }
        mir::TerminatorKind::Drop { place: p, target, unwind: u, .. } => {
            json!({"op":"drop","place":place(tcx,body,*p),"target":target.as_usize(),"unwind":unwind(*u)})
        }
        mir::TerminatorKind::Assert { cond, expected, msg, target, unwind: u } => {
            json!({"op":"assert","operand":operand(tcx,body,cond),"expected":expected,
            "message":assertion(tcx,body,msg),"target":target.as_usize(),"unwind":unwind(*u)})
        }
        other => json!({"op":"unsupported_terminator","debug":format!("{other:?}")}),
    }
}

pub fn type_shape<'tcx>(tcx: TyCtxt<'tcx>, t: ty::Ty<'tcx>) -> Value {
    match t.kind() {
        ty::Adt(def, args) => {
            json!({"kind":"adt","identity":format!("{:?}",tcx.def_path_hash(def.did())),"repr_transparent":def.repr().transparent(),
            "definition":tcx.def_path_str(def.did()),"has_destructor":def.has_dtor(tcx),
            "variants":def.variants().iter().map(|v|json!({"name":v.name.to_string(),"fields":v.fields.iter().map(|f|json!({"name":f.name.to_string(),"ty":f.ty(tcx,args).skip_norm_wip().to_string()})).collect::<Vec<_>>()})).collect::<Vec<_>>()})
        }
        ty::Ref(_, p, m) => json!({"kind":"ref","pointee":p.to_string(),"mutable":m.is_mut()}),
        ty::RawPtr(p, m) => json!({"kind":"raw_ptr","pointee":p.to_string(),"mutable":m.is_mut()}),
        ty::Array(p, n) => {
            json!({"kind":"array","element":p.to_string(),"length":format!("{n:?}")})
        }
        ty::Slice(p) => json!({"kind":"slice","element":p.to_string()}),
        ty::Tuple(fields) => {
            json!({"kind":"tuple","fields":fields.iter().map(|f|f.to_string()).collect::<Vec<_>>()})
        }
        ty::Uint(_) | ty::Int(_) | ty::Bool | ty::Never | ty::FnDef(..) => {
            json!({"kind":"scalar_or_function","ty":t.to_string()})
        }
        _ => json!({"kind":"unsupported_type","ty":t.to_string()}),
    }
}
