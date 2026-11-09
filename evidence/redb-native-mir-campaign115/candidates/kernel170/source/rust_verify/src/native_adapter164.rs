use rustc_middle::{mir, ty::{self, Ty, TyCtxt}};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use vir::ast::*;
use vir::ast_util::{bool_typ, mk_tuple_field_opr, types_equal, unit_typ};
use vir::def::Spanned;
use vir::native_resource::adapter164;

mod scalar;

#[cfg(feature = "native-semantics170-inspect")]
pub fn interpret_alignment170<'tcx>(tcx: TyCtxt<'tcx>, typ: Ty<'tcx>) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let ty::Adt(definition, arguments) = *typ.kind() else { return Err("actual nominal Alignment required".into()); };
    if tcx.crate_name(definition.did().krate).as_str() != "core"
        || tcx.item_name(definition.did()).as_str() != "Alignment"
    { return Err("wrong nominal alignment definition".into()); }
    let methods = tcx.inherent_impls(definition.did()).iter().flat_map(|implementation|
        tcx.associated_items(*implementation).in_definition_order()).filter(|item|
            item.name().as_str() == "as_usize").collect::<Vec<_>>();
    if methods.len() != 1 { return Err("ambiguous actual Alignment::as_usize".into()); }
    let method = ty::Instance::try_resolve(tcx, ty::TypingEnv::fully_monomorphized(), methods[0].def_id, arguments)
        .map_err(|_| "alignment method resolution failed")?.ok_or("alignment method absent")?;
    let name = VarIdent(Arc::new("actual_nominal_alignment_word170".into()), VarIdentDisambiguate::NativeResource(170));
    let actual = variable(&name, &usize_type());
    let interpreted = scalar::evaluate_method(tcx, method, actual, 0)?;
    if !matches!(&interpreted.x, ExprX::Var(result) if *result == name)
        || !types_equal(&interpreted.typ, &usize_type())
    { return Err("actual Alignment::as_usize not interpreted as the numeric operand word".into()); }
    Ok(hex::encode(Sha256::digest(format!("{:#?}",tcx.instance_mir(method.def)).as_bytes())))
}

fn span() -> vir::messages::Span {
    vir::messages::Span { raw_span: Arc::new(()), id: 164, data: vec![],
        as_string: "checked native MIR subset164 (not admitted)".into() }
}

fn usize_type() -> Typ { Arc::new(TypX::Int(IntRange::USize)) }

fn number(value: u128) -> Expr {
    SpannedTyped::new(&span(), &usize_type(), ExprX::Const(Constant::Int(value.into())))
}

fn native_type<'tcx>(tcx: TyCtxt<'tcx>, owner: ty::Instance<'tcx>, typ: Ty<'tcx>) -> Ty<'tcx> {
    tcx.normalize_erasing_regions(ty::TypingEnv::fully_monomorphized(),
        ty::EarlyBinder::bind(tcx, typ).instantiate(tcx, owner.args))
}

fn resolved_call<'tcx>(tcx: TyCtxt<'tcx>, owner: ty::Instance<'tcx>, body: &mir::Body<'tcx>,
    func: &mir::Operand<'tcx>) -> Result<ty::Instance<'tcx>, String>
{
    let function_type = native_type(tcx, owner, func.ty(&body.local_decls, tcx));
    let ty::FnDef(definition, arguments) = *function_type.kind() else {
        return Err("indirect native call unsupported".into());
    };
    ty::Instance::try_resolve(tcx, ty::TypingEnv::fully_monomorphized(), definition,
        arguments.skip_binder()).map_err(|_| String::from("native call resolution failed"))?
        .ok_or_else(|| "native call has no resolved Instance".into())
}

fn proof_type<'tcx>(tcx: TyCtxt<'tcx>, typ: Ty<'tcx>) -> Result<Typ, String> {
    if scalar::word_type(tcx, typ) { return Ok(usize_type()); }
    if typ == tcx.types.bool { return Ok(bool_typ()); }
    if typ.is_unit() { return Ok(unit_typ()); }
    if typ == Ty::new_mut_ptr(tcx, tcx.types.u8) {
        return Ok(Arc::new(TypX::Primitive(Primitive::Ptr,
            Arc::new(vec![Arc::new(TypX::Int(IntRange::U(8)))]))));
    }
    Err(format!("unsupported native value type: {typ:?}"))
}

fn variable(name: &VarIdent, typ: &Typ) -> Expr {
    SpannedTyped::new(&span(), typ, ExprX::Var(name.clone()))
}

fn declaration(name: VarIdent, initial: Expr, identity: u64) -> Stmt {
    let mut binding_span = span();
    binding_span.id = identity;
    let pattern = SpannedTyped::new(&binding_span, &initial.typ, PatternX::Var(PatternBinding {
        name, by_ref: ByRef::No, typ: initial.typ.clone(), user_mut: Some(false), copy: false,
    }));
    let typ = initial.typ.clone();
    let init = SpannedTyped::new(&binding_span, &typ, PlaceX::Temporary(initial));
    Spanned::new(binding_span, StmtX::Decl { pattern, mode: Some((Mode::Exec, DeclProph::Default)),
        init: Some(init), els: None, assert_irrefutable: true })
}

#[derive(Clone)]
struct Value164 {
    expression: Expr,
    bundle: Option<Expr>,
    loan: Option<mir::Local>,
}

#[derive(Clone, Default)]
struct State164 { values: BTreeMap<mir::Local, Value164> }

impl State164 {
    fn ensure_not_borrowed(&self, local: mir::Local) -> Result<(), String> {
        if self.values.values().any(|value| value.loan == Some(local)) {
            return Err("destructive use overlaps a live parent/child loan".into());
        }
        Ok(())
    }

    fn overwrite(&mut self, local: mir::Local, value: Option<Value164>) -> Result<(), String> {
        self.ensure_not_borrowed(local)?;
        if self.values.get(&local).is_some_and(|value| value.bundle.is_some()) {
            return Err("native overwrite/drop would discard a live allocation resource pair".into());
        }
        self.values.remove(&local);
        if let Some(value) = value { self.values.insert(local, value); }
        Ok(())
    }

    fn has_resources(&self) -> bool {
        self.values.values().any(|value| value.bundle.is_some())
    }
}

pub struct Lowered164 {
    pub params: Params,
    pub body: Expr,
    pub native_return: Typ,
    pub proof_return: Typ,
    pub translated_statements: usize,
    pub translated_terminators: usize,
    pub native_allocator_instructions: usize,
}

impl Lowered164 {
    pub fn review_function(&self, native_name: &str) -> Function {
        let ret = Spanned::new(span(), ParamX {
            name: VarIdent(Arc::new("native164_return".into()), VarIdentDisambiguate::NativeResource(u64::MAX)),
            typ: self.proof_return.clone(), mode: Mode::Exec, user_mut: false, unwrapped_info: None,
        });
        Spanned::new(span(), FunctionX {
            name: Arc::new(FunX { path: Arc::new(PathX {
                krate: CrateId::Id(Arc::new("native164_internal_review".into()), 164),
                segments: Arc::new(vec![Arc::new(native_name.into())]),
            }) }),
            proxy: None, kind: FunctionKind::Static,
            visibility: Visibility { restricted_to: None },
            body_visibility: BodyVisibility::Visibility(Visibility { restricted_to: None }),
            opaqueness: Opaqueness::Opaque, owning_module: None, mode: Mode::Exec,
            typ_params: Arc::new(vec![]), typ_bounds: Arc::new(vec![]), params: self.params.clone(),
            outer_ret: ret.clone(), inner_ret: ret,
            require: Arc::new(vec![]), ensure: (Arc::new(vec![]), Arc::new(vec![])), returns: None,
            decrease: Arc::new(vec![]), decrease_when: None, decrease_by: None,
            fndef_axioms: None, mask_spec: None, atomic_update: None, unwind_spec: None,
            item_kind: ItemKind::Function,
            attrs: Arc::new(FunctionAttrsX { infer_candidate_id: Some(
                "experimental-native-resource-ir:164:unreviewed-native-MIR".into()),
                ..FunctionAttrsX::default() }),
            body: Some(self.body.clone()), return_opaque_type_eqs: Arc::new(vec![]),
            extra_dependencies: vec![], hidden: Arc::new(vec![]),
        })
    }
}

struct Translator164<'tcx> {
    tcx: TyCtxt<'tcx>,
    owner: ty::Instance<'tcx>,
    body: &'tcx mir::Body<'tcx>,
    fresh_name: u64,
    translated_statements: usize,
    translated_terminators: usize,
    native_allocator_instructions: usize,
    expansion: usize,
}

impl<'tcx> Translator164<'tcx> {
    fn bind(&mut self, expression: Expr, statements: &mut Vec<Stmt>) -> Expr {
        let name = VarIdent(Arc::new("native164_value".into()),
            VarIdentDisambiguate::NativeResource(self.fresh_name));
        let result = variable(&name, &expression.typ);
        statements.push(declaration(name, expression, self.fresh_name));
        self.fresh_name += 1;
        result
    }

    fn operand(&self, state: &mut State164, operand: &mir::Operand<'tcx>) -> Result<Value164, String> {
        match operand {
            mir::Operand::Move(place) | mir::Operand::Copy(place) => {
                if !place.projection.is_empty() {
                    let words = state.values.iter().filter(|(_, value)| value.bundle.is_none() && value.loan.is_none())
                        .map(|(local, value)| (*local, value.expression.clone())).collect();
                    let expression = scalar::read_word(self.tcx, self.owner, self.body, &words, operand)?;
                    if matches!(operand, mir::Operand::Move(_)) {
                        return Err("partial move unsupported rather than silently retaining a parent resource".into());
                    }
                    return Ok(Value164 { expression, bundle: None, loan: None });
                }
                let mut value = state.values.get(&place.local).cloned().ok_or("use of stale/uninitialized native holder")?;
                if matches!(operand, mir::Operand::Move(_)) {
                    state.ensure_not_borrowed(place.local)?;
                    state.values.remove(&place.local);
                } else {
                    value.bundle = None;
                }
                Ok(value)
            }
            mir::Operand::Constant(constant) => {
                let typ = native_type(self.tcx, self.owner, constant.const_.ty());
                if typ.is_unit() {
                    return Ok(Value164 { expression: vir::ast_util::mk_tuple(&span(), &Arc::new(vec![])),
                        bundle: None, loan: None });
                }
                if typ == self.tcx.types.bool {
                    let scalar = constant.const_.try_eval_scalar_int(self.tcx, ty::TypingEnv::fully_monomorphized())
                        .ok_or("boolean native constant did not evaluate")?;
                    let expression = SpannedTyped::new(&span(), &bool_typ(),
                        ExprX::Const(Constant::Bool(scalar.to_u8() != 0)));
                    return Ok(Value164 { expression, bundle: None, loan: None });
                }
                let expression = scalar::read_word(self.tcx, self.owner, self.body, &BTreeMap::new(), operand)?;
                Ok(Value164 { expression, bundle: None, loan: None })
            }
            _ => Err("unsupported native operand".into()),
        }
    }

    fn rvalue(&self, state: &mut State164, rvalue: &mir::Rvalue<'tcx>) -> Result<Value164, String> {
        match rvalue {
            mir::Rvalue::Use(operand, retag) => {
                let typ = native_type(self.tcx, self.owner, operand.ty(&self.body.local_decls, self.tcx));
                if retag.yes() && !scalar::word_type(self.tcx, typ) && typ != self.tcx.types.bool
                    && !typ.is_unit() {
                    return Err("native reference/pointer retag is not interpreted".into());
                }
                self.operand(state, operand)
            }
            mir::Rvalue::Cast(mir::CastKind::Transmute, operand, destination_type) => {
                let source_type = native_type(self.tcx, self.owner, operand.ty(&self.body.local_decls, self.tcx));
                let destination_type = native_type(self.tcx, self.owner, *destination_type);
                if !scalar::word_type(self.tcx, source_type) || !scalar::word_type(self.tcx, destination_type) {
                    return Err("native transmute is not a checked scalar word reinterpretation".into());
                }
                if !scalar::transmute_preserves_validity(self.tcx, source_type, destination_type) {
                    return Err("native transmute into a restricted-validity word requires checked path validity".into());
                }
                let value = self.operand(state, operand)?;
                if value.bundle.is_some() || value.loan.is_some() { return Err("transmute cannot transport resources/loans".into()); }
                Ok(value)
            }
            mir::Rvalue::Ref(_, _, place) if place.projection.is_empty() => {
                let root = state.values.get(&place.local).ok_or("borrow of stale native holder")?;
                if root.loan.is_some() { return Err("reborrow unsupported".into()); }
                Ok(Value164 { expression: root.expression.clone(), bundle: None, loan: Some(place.local) })
            }
            mir::Rvalue::BinaryOp(operation, operands) if matches!(operation, mir::BinOp::Eq | mir::BinOp::Ne) => {
                let left = self.operand(state, &operands.0)?;
                let right = self.operand(state, &operands.1)?;
                if left.bundle.is_some() || right.bundle.is_some() || left.loan.is_some() || right.loan.is_some()
                    || !types_equal(&left.expression.typ, &right.expression.typ) {
                    return Err("comparison cannot consume/duplicate tracked native outputs or loans".into());
                }
                if !matches!(&*left.expression.typ, TypX::Int(IntRange::USize) | TypX::Bool) {
                    return Err("native pointer/aggregate equality requires canonical address/provenance semantics and is not interpreted".into());
                }
                let operator = if *operation == mir::BinOp::Eq { BinaryOp::Eq(Mode::Exec) } else { BinaryOp::Ne };
                Ok(Value164 { expression: SpannedTyped::new(&span(), &bool_typ(),
                    ExprX::Binary(operator, left.expression, right.expression)), bundle: None, loan: None })
            }
            _ => Err(format!("unsupported native rvalue (no silent Ordinary effect): {rvalue:?}")),
        }
    }

    fn raw_allocator(&self, callee: ty::Instance<'tcx>) -> bool {
        self.tcx.crate_name(callee.def_id().krate).as_str() == "alloc"
            && self.tcx.def_path_str(callee.def_id()) == "alloc::alloc::__rust_alloc"
    }

    fn block(&mut self, block: mir::BasicBlock, mut state: State164, mut path: BTreeSet<mir::BasicBlock>)
        -> Result<Expr, String>
    {
        self.expansion += 1;
        if !path.insert(block) || self.expansion > 256 {
            return Err("native loops/recursive control-flow or excessive path expansion unsupported".into());
        }
        let data = &self.body.basic_blocks[block];
        if data.is_cleanup { return Err("native cleanup is unsupported, not erased".into()); }
        let mut statements = Vec::new();
        for statement in &data.statements {
            self.translated_statements += 1;
            match &statement.kind {
                mir::StatementKind::Assign(assignment) => {
                    let (destination, rvalue) = &**assignment;
                    let local = destination.as_local().ok_or("native projected/destructive write unsupported")?;
                    let mut value = self.rvalue(&mut state, rvalue)?;
                    if value.loan.is_none() && value.bundle.is_none() {
                        let typ = native_type(self.tcx, self.owner, self.body.local_decls[local].ty);
                        if !types_equal(&proof_type(self.tcx, typ)?, &value.expression.typ) {
                            return Err("native assignment type mismatch".into());
                        }
                        value.expression = self.bind(value.expression, &mut statements);
                    }
                    state.overwrite(local, Some(value))?;
                }
                mir::StatementKind::StorageDead(local) => state.overwrite(*local, None)?,
                mir::StatementKind::StorageLive(local) => {
                    if state.values.contains_key(local) { return Err("StorageLive overlaps initialized native holder".into()); }
                }
                mir::StatementKind::Nop => {}
                _ => return Err(format!("unsupported native statement (not assumed): {:?}", statement.kind)),
            }
        }
        self.translated_terminators += 1;
        let tail = match &data.terminator().kind {
            mir::TerminatorKind::Goto { target } => self.block(*target, state, path)?,
            mir::TerminatorKind::Return => {
                let result = state.values.remove(&mir::RETURN_PLACE).ok_or("native return is uninitialized")?;
                if state.has_resources() || state.values.values().any(|value| value.loan.is_some())
                    || result.loan.is_some() {
                    return Err("native return leaves an untransferred allocation pair or live loan".into());
                }
                result.bundle.unwrap_or(result.expression)
            }
            mir::TerminatorKind::SwitchInt { discr, targets } => {
                let value = self.operand(&mut state, discr)?;
                if value.bundle.is_some() || value.loan.is_some() { return Err("switch cannot consume resources".into()); }
                let mut tail = self.block(targets.otherwise(), state.clone(), path.clone())?;
                for (constant, target) in targets.iter().collect::<Vec<_>>().into_iter().rev() {
                    let branch = self.block(target, state.clone(), path.clone())?;
                    if !types_equal(&branch.typ, &tail.typ) {
                        return Err("native join has incompatible resource outputs".into());
                    }
                    let constant = if matches!(&*value.expression.typ, TypX::Bool) {
                        if constant > 1 { return Err("invalid boolean switch".into()); }
                        SpannedTyped::new(&span(), &bool_typ(), ExprX::Const(Constant::Bool(constant != 0)))
                    } else { number(constant) };
                    let condition = SpannedTyped::new(&span(), &bool_typ(),
                        ExprX::Binary(BinaryOp::Eq(Mode::Exec), value.expression.clone(), constant));
                    tail = SpannedTyped::new(&span(), &tail.typ, ExprX::If(condition, branch, Some(tail.clone())));
                }
                tail
            }
            mir::TerminatorKind::Call { func, args, destination, target: Some(target), unwind, .. } => {
                let callee = resolved_call(self.tcx, self.owner, self.body, func)?;
                let local = destination.as_local().ok_or("native projected call output unsupported")?;
                let result = if self.raw_allocator(callee) {
                    if !matches!(unwind, mir::UnwindAction::Unreachable) {
                        return Err("native allocator unwind/error/cleanup effect unsupported, not framed away".into());
                    }
                    if args.len() != 2 || state.has_resources() || state.values.values().any(|value| value.loan.is_some()) {
                        return Err("native allocator requires exact operands and empty incoming resource/loan environment (no frame gift)".into());
                    }
                    let size_type = native_type(self.tcx, self.owner, args[0].node.ty(&self.body.local_decls, self.tcx));
                    let alignment_type = native_type(self.tcx, self.owner, args[1].node.ty(&self.body.local_decls, self.tcx));
                    let pointer_type = native_type(self.tcx, self.owner, destination.ty(&self.body.local_decls, self.tcx).ty);
                    if size_type != self.tcx.types.usize || pointer_type != Ty::new_mut_ptr(self.tcx, self.tcx.types.u8) {
                        return Err("native raw allocator ABI mismatch".into());
                    }
                    let size = self.operand(&mut state, &args[0].node)?;
                    let alignment = self.operand(&mut state, &args[1].node)?;
                    if size.loan.is_some() || alignment.loan.is_some() { return Err("borrowed allocator operand unsupported".into()); }
                    let numeric = scalar::alignment_as_usize(self.tcx, alignment_type, alignment.expression)?;
                    let numeric = self.bind(numeric, &mut statements);
                    let call = SpannedTyped::new(&span(), &adapter164::result_type(), ExprX::Call {
                        target: CallTarget::NativeAllocator164,
                        args: Arc::new(vec![size.expression, numeric]), post_args: None, body: None,
                    });
                    adapter164::validate_call(&call, match &call.x { ExprX::Call { args, .. } => args, _ => unreachable!() }, &None)
                        .map_err(|error| error.note.clone())?;
                    let bundle = self.bind(call, &mut statements);
                    let expression = SpannedTyped::new(&span(), &proof_type(self.tcx, pointer_type)?,
                        ExprX::UnaryOpr(UnaryOpr::Field(mk_tuple_field_opr(3, 0)), bundle.clone()));
                    self.native_allocator_instructions += 1;
                    Value164 { expression, bundle: Some(bundle), loan: None }
                } else {
                    if !matches!(unwind, mir::UnwindAction::Unreachable | mir::UnwindAction::Continue) {
                        return Err("scalar call cleanup/termination edge unsupported".into());
                    }
                    if args.len() != 1 { return Err("unmodeled native call has no blanket summary".into()); }
                    let value = self.operand(&mut state, &args[0].node)?;
                    if value.bundle.is_some() || value.loan.is_some() { return Err("scalar call cannot transport resources/loans".into()); }
                    let expression = scalar::evaluate_method(self.tcx, callee, value.expression, 0)?;
                    Value164 { expression: self.bind(expression, &mut statements), bundle: None, loan: None }
                };
                state.overwrite(local, Some(result))?;
                self.block(*target, state, path)?
            }
            _ => return Err(format!("unsupported native terminator/outcome: {:?}", data.terminator().kind)),
        };
        let typ = tail.typ.clone();
        Ok(SpannedTyped::new(&span(), &typ, ExprX::Block(Arc::new(statements), Some(tail))))
    }
}

pub fn lower_native_instance<'tcx>(tcx: TyCtxt<'tcx>, owner: ty::Instance<'tcx>) -> Result<Lowered164, String> {
    if tcx.data_layout.pointer_size().bits() != 64 {
        return Err("native164 scalar interpretation requires a 64-bit native target".into());
    }
    let body = tcx.instance_mir(owner.def);
    let mut params = Vec::new();
    let mut state = State164::default();
    for local in body.args_iter() {
        let typ = proof_type(tcx, native_type(tcx, owner, body.local_decls[local].ty))?;
        let name = VarIdent(Arc::new("native164_arg".into()),
            VarIdentDisambiguate::NativeResource(local.as_usize() as u64));
        let parameter = Spanned::new(span(), ParamX { name: name.clone(), typ: typ.clone(),
            mode: Mode::Exec, user_mut: false, unwrapped_info: None });
        state.values.insert(local, Value164 { expression: variable(&name, &typ), bundle: None, loan: None });
        params.push(parameter);
    }
    let native_return = proof_type(tcx, native_type(tcx, owner, body.return_ty()))?;
    let mut translator = Translator164 { tcx, owner, body, fresh_name: 1_000_000,
        translated_statements: 0, translated_terminators: 0, native_allocator_instructions: 0, expansion: 0 };
    let body = translator.block(mir::START_BLOCK, state, BTreeSet::new())?;
    Ok(Lowered164 { params: Arc::new(params), proof_return: body.typ.clone(), body, native_return,
        translated_statements: translator.translated_statements,
        translated_terminators: translator.translated_terminators,
        native_allocator_instructions: translator.native_allocator_instructions })
}
