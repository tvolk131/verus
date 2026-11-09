use std::path::Path;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use sha2::{Digest, Sha256};
use serde_json::{json, Value};
use vir::ast::*;
use vir::context::{Ctx, GlobalCtx, WarningCtx};
use vir::def::Spanned;
use vir::native_resource::kernel170::*;
use vir::sst::{Exp, ExpX, StmX};

fn digest(bytes: &[u8]) -> String { hex::encode(Sha256::digest(bytes)) }

fn variable(span: &vir::messages::Span, name: &str, identity: u64, typ: Typ) -> Exp {
    SpannedTyped::new(span, &typ, ExpX::Var(VarIdent(Arc::new(name.into()),
        VarIdentDisambiguate::NativeResource(identity))))
}

fn globals(krate: &Krate, span: &vir::messages::Span, simplified: bool) -> Result<GlobalCtx, String> {
    GlobalCtx::new(krate, CrateId::Vstd, span.clone(), 10.0,
        Arc::new(Mutex::new(None)), Arc::new(Mutex::new(None)),
        Arc::new(WarningCtx { fun_warn_configs: HashMap::new() }), air::context::SmtSolver::Z3,
        simplified, false, false, false, false).map_err(|error| error.note.clone())
}

pub fn inspect170(source: Source170, library: &Path, expected_library_sha256: &str) -> Result<Value, String> {
    let bytes = std::fs::read(library).map_err(|error| error.to_string())?;
    let library_sha256 = digest(&bytes);
    if library_sha256 != expected_library_sha256 || library_sha256 != "353ef06dd082872df52191eb4587232ac6507ba097862e186152056a2a0d2ce4" {
        return Err("fixed genuine canonical AST byte digest mismatch; supplied nominal datatypes refused".into());
    }
    let loaded = crate::import_export::load_native170_inspection(library)?;
    let canonical = Canonical170::from_authenticated_library(&loaded, library_sha256.clone())
        .map_err(|error| format!("loaded canonical: {error:?}; allocate signatures: {:?}",
            loaded.functions.iter().filter(|function| function.x.name.path.segments.last()
                .is_some_and(|name| name.as_str() == "allocate")).map(|function|
                    (&function.x.name, &function.x.outer_ret, function.x.require.len(), function.x.ensure.0.len())).collect::<Vec<_>>()))?;
    let span = vir::messages::Span { raw_span: Arc::new(()), id: 170, data: vec![],
        as_string: "native semantic170 pre-review structural inspection".into() };
    let mut initial = globals(&loaded, &span, false)?;
    let simplified = vir::ast_simplify::simplify_krate(&mut initial, &loaded).map_err(|error| error.note.clone())?;
    let module = Spanned::new(span.clone(), ModuleX { path: Arc::new(PathX { krate: CrateId::Vstd,
        segments: Arc::new(vec![]) }), reveals: None });
    let context = Ctx::new(&simplified, globals(&simplified, &span, true)?, module,
        vec![], vec![], HashSet::new(), vir::prune::UsedBuiltins { uses_array: false,
            uses_bytestr: false, uses_pointee_metadata: false, uses_ieee_float: false }, vec![], vec![])
        .map_err(|error| error.note.clone())?;
    let word = Arc::new(TypX::Int(IntRange::USize));
    let size = variable(&span, &format!("actual_size_operand:{}", source.size_operand), 1701, word.clone());
    let align = variable(&span, &format!("actual_alignment_as_usize:{}", source.align_operand), 1702, word.clone());
    let frontier = variable(&span, "complete_incoming_native_frontier_is_empty", 1703, Arc::new(TypX::Bool));
    let enter = || Kernel170::enter(source.clone(), Incoming170::ConditionalEmpty {
        complete_native_frontier_is_empty: frontier.clone() });
    let binder = VarIdent(Arc::new("same_selected_native_call_result".into()), VarIdentDisambiguate::NativeResource(1704));
    let instruction = make_instruction170(&span, &canonical, binder.clone(), size.clone(), align.clone());
    let mut kernel = enter().map_err(|error| format!("enter: {error:?}"))?;
    let call = kernel.prepare_call(instruction.clone(), 1, Outcome170::NormalNonNull).map_err(|error| format!("prepare: {error:?}"))?;
    let (transition, air) = inspect_native170_to_air(&context, &mut kernel,
        &canonical, &call, &instruction).map_err(|error| error.note.clone())?;
    let mut controls = Vec::new();
    let mut check = |name: &str, passed: bool, detail: String| {
        controls.push(json!({"name": name,"passed":passed,"detail":detail,"kind":"structural_no_solver_no_native_credit"}));
    };
    check("same_full_native_return", Arc::ptr_eq(&kernel.native_return().unwrap(), &transition.native_return),
        format!("{:?}", transition.native_return));
    check("canonical_initialized_raw_dealloc", kernel.live_count() == 1 && air.iter().any(|statement|
        matches!(&**statement, air::ast::StmtX::Havoc(_))), format!("raw={:?};dealloc={:?}",transition.raw, transition.dealloc));
    check("mandatory_frontier_assertion", matches!(&*air[0], air::ast::StmtX::Assert(..)), format!("{:?}",air[0]));
    check("nonnull_is_checked_not_gifted",air.iter().filter(|statement|matches!(&***statement,air::ast::StmtX::Assert(..))).count()==3,
        "incoming frontier, actual layout/positive size, and SAME full-return nonnull must be established".into());
    check("initialized_result_has_canonical_declaration",transition.air_local_decls.len()==1,format!("{:?}",transition.air_local_decls));
    check("default_off_review_gate", operative_admission170().err() == Some(Refusal170::ReviewRequired), "never activated".into());
    check("replayed_issuance", kernel.allocate(&canonical,&call,&instruction).err() == Some(Refusal170::Replay), "same activation/evaluation".into());
    check("duplicate_resources", kernel.copy_resources(1).err() == Some(Refusal170::DuplicateResource), "real canonical value holder".into());
    check("live_frontier_activation_refused",kernel.enter_activation(source.block).err()==Some(Refusal170::NonemptyFrontier),"no prior-live-memory or ambient frame".into());
    check("drop_live_resources", kernel.overwrite(1).err() == Some(Refusal170::ResourceLeak), "live pair not discarded".into());
    check("untransferred_return", kernel.return_resources(&[]).err() == Some(Refusal170::ResourceLeak), "pair must be returned or consumed".into());
    check("duplicate_return", kernel.return_resources(&[1,1]).err() == Some(Refusal170::DuplicateResource), "same generated canonical bundle".into());
    for (name,incoming,expected) in [("unknown_frontier",Incoming170::Unknown,Refusal170::UnknownFrontier),
        ("nonempty_frontier",Incoming170::Nonempty,Refusal170::NonemptyFrontier)] {
        check(name,Kernel170::enter(source.clone(),incoming).err()==Some(expected),"incoming environment, not local absence".into());
    }
    for outcome in [Outcome170::Null,Outcome170::Error,Outcome170::Unwind,Outcome170::Abort,Outcome170::Divergence] {
        let mut alternative=enter().unwrap();
        let alternative_call=alternative.prepare_call(instruction.clone(),1,outcome).unwrap();
        let refused=alternative.allocate(&canonical,&alternative_call,&instruction).err()==Some(Refusal170::UnsupportedOutcome);
        check(&format!("outcome_{outcome:?}"),refused && alternative.live_count()==0,"no success pair".into());
    }
    let wrong_size=variable(&span,"substituted_size",1710,word.clone());
    let wrong_align=variable(&span,"substituted_align",1711,word);
    for (name,other_size,other_align,expected) in [
        ("substituted_size",wrong_size.clone(),align.clone(),Refusal170::SubstitutedSize),
        ("substituted_align",size.clone(),wrong_align.clone(),Refusal170::SubstitutedAlign),
    ] {
        let changed=make_instruction170(&span,&canonical,binder.clone(),other_size,other_align);
        let mut alternative=enter().unwrap();
        check(name,alternative.allocate(&canonical,&call,&changed).err()==Some(expected),"registered instruction operands cannot be replaced".into());
    }
    let changed=make_instruction170(&span,&canonical,VarIdent(Arc::new("substituted_return".into()),
        VarIdentDisambiguate::NativeResource(1712)),size.clone(),align.clone());
    check("substituted_native_return",enter().unwrap().allocate(&canonical,&call,&changed).err()==Some(Refusal170::SubstitutedReturn),"no second symbolic pointer/address equation".into());
    let alias=kernel.alias_pointer(1,9).unwrap();
    check("pointer_alias_no_duplicate_authority",Arc::ptr_eq(&alias,&transition.native_return) && kernel.live_count()==1,"pointer alias does not clone resources".into());
    kernel.borrow(1,11).unwrap();
    check("move_borrowed",kernel.move_holder(1,2).err()==Some(Refusal170::Borrowed),"canonical pair frozen by loan".into());
    check("destroy_borrowed",kernel.deallocate(&canonical,1,&transition.native_return,&size,&align).err()==Some(Refusal170::Borrowed),"cannot consume loaned pair".into());
    kernel.end_loan(11).unwrap();kernel.move_holder(1,2).unwrap();
    check("overwrite_live_moved_destination",kernel.move_holder(2,2).err()==Some(Refusal170::LiveDestination),"no holder overwrite or self-move duplication".into());
    check("moved_holder_stale",kernel.copy_resources(1).err()==Some(Refusal170::StaleHolder),"destructive transfer".into());
    let other_pointer=variable(&span,"other_full_pointer",1713,transition.native_return.typ.clone());
    check("wrong_full_pointer_provenance",kernel.deallocate(&canonical,2,&other_pointer,&size,&align).err()==Some(Refusal170::WrongProvenance),"numeric equality cannot restore authority".into());
    check("destroy_wrong_size",kernel.deallocate(&canonical,2,&transition.native_return,&wrong_size,&align).err()==Some(Refusal170::SubstitutedSize),"same actual size".into());
    check("destroy_wrong_alignment",kernel.deallocate(&canonical,2,&transition.native_return,&size,&wrong_align).err()==Some(Refusal170::SubstitutedAlign),"same numeric alignment".into());
    check("unsupported_cleanup",kernel.goto(source.normal,source.block,true).err()==Some(Refusal170::UnsupportedCleanup),"no rollback/frame".into());
    let branch_condition=variable(&span,"actual_supported_branch_discriminant",1715,Arc::new(TypX::Bool));
    let branch_scenario=|| {
        let mut alternative=enter().unwrap();
        let alternative_call=alternative.prepare_call(instruction.clone(),1,Outcome170::NormalNonNull).unwrap();
        alternative.allocate(&canonical,&alternative_call,&instruction).unwrap();
        alternative.move_holder(1,2).unwrap();
        alternative
    };
    let branch=branch_scenario().branch(&context,branch_condition.clone(),12,
        |state| {state.move_holder(2,3)?;Ok(vec![])}, |state| {state.move_holder(2,3)?;Ok(vec![])});
    check("exclusive_branch_join_and_air",branch.as_ref().is_ok_and(|(_,air)|matches!(&**air,air::ast::StmtX::Switch(_))),
        "actual guard and complementary guard in exclusive AIR Switch; no public context Clone".into());
    let branch=branch_scenario().branch(&context,branch_condition,13,
        |state| {state.move_holder(2,3)?;Ok(vec![])}, |_|Ok(vec![]));
    check("incompatible_branch_holders",branch.err()==Some(Refusal170::BranchMismatch),"no dropped/duplicated branch resource".into());
    let destroy=kernel.deallocate(&canonical,2,&transition.native_return,&size,&align).unwrap();
    check("canonical_destroy_consumes_both",matches!(&destroy.x,StmX::Call { args,.. } if args.len()==5) && kernel.live_count()==0,format!("{destroy:?}"));
    check("stale_generation_alias",kernel.read_alias(9).err()==Some(Refusal170::StaleHolder),"deallocation invalidates provenance generation".into());
    check("destroy_twice",kernel.deallocate(&canonical,2,&transition.native_return,&size,&align).err()==Some(Refusal170::StaleHolder),"no reused canonical tokens".into());
    kernel.enter_activation(source.block).unwrap();
    let fresh_instruction=make_instruction170(&span,&canonical,VarIdent(Arc::new("next_activation_native_result".into()),VarIdentDisambiguate::NativeResource(1714)),size.clone(),align.clone());
    let fresh_call=kernel.prepare_call(fresh_instruction.clone(),4,Outcome170::NormalNonNull).unwrap();
    let (fresh_transition,fresh_air)=inspect_native170_to_air(&context,&mut kernel,&canonical,&fresh_call,&fresh_instruction).map_err(|error|error.note.clone())?;
    check("repeated_static_site_distinct_activation",fresh_transition.allocation!=transition.allocation && !fresh_air.is_empty(),"new native semantic evaluation, not static-site identity".into());
    check("initialized_tracked_return",kernel.return_resources(&[4]).is_ok(),"returns actual generated canonical tuple".into());
    for (name,runtime) in [("substituted_valid_source_digest",false),("substituted_valid_runtime_digest",true)] {
        let mut changed_source=source.clone();
        if runtime {changed_source.runtime_sha256="0000000000000000000000000000000000000000000000000000000000000000".into();}
        else {changed_source.root_source_sha256="0000000000000000000000000000000000000000000000000000000000000000".into();}
        let mut other=Kernel170::enter(changed_source,Incoming170::ConditionalEmpty {complete_native_frontier_is_empty:frontier.clone()}).unwrap();
        check(name,other.allocate(&canonical,&call,&instruction).err()==Some(Refusal170::WrongSource),"valid-format changed origin cannot consume the selected original call witness".into());
    }
    for (name,change,expected) in [
        ("wrong_source",0,Refusal170::WrongSource),("wrong_callee",1,Refusal170::WrongCallee),
        ("wrong_runtime",2,Refusal170::WrongRuntime),("wrong_abi",3,Refusal170::WrongAbi),
        ("wrong_alignment_interpretation",4,Refusal170::WrongAlignmentInterpretation)] {
        let mut changed_source=source.clone();
        match change {0=>changed_source.root_source_sha256.clear(),1=>changed_source.callee="other_allocator".into(),
            2=>changed_source.target="other_target".into(),3=>changed_source.abi="usize erasure".into(),
            _=>changed_source.alignment="hardcoded8".into()}
        check(name,Kernel170::enter(changed_source,Incoming170::ConditionalEmpty {complete_native_frontier_is_empty:frontier.clone()}).err()==Some(expected),"actual source/runtime/ABI gates".into());
    }
    let failures=controls.iter().filter(|control|control["passed"]!=true).count();
    Ok(json!({"schema":"native-semantics170-structural-inspection-v1","law":LAW170,"disposition":DISPOSITION170,
        "loaded_vstd":library,"loaded_vstd_sha256":library_sha256,"canonical_allocate":format!("{:?}",canonical.allocate),
        "canonical_raw":format!("{:?}",canonical.raw),"canonical_dealloc":format!("{:?}",canonical.dealloc),
        "source_binding":format!("{source:?}"),"native_sst":format!("{instruction:?}"),
        "canonical_sst":format!("{:?}",transition.canonical_call),"air":format!("{air:#?}"),
        "conditional_native_cut_parameters":format!("{:?}",[size,align,frontier]),
        "air_initialized_result_declarations":format!("{:?}",transition.air_local_decls),
        "controls":controls,"control_failures":failures,"operative_issuance":0,"solver_runs":0,
        "accepted_original_allocations":0,"accepted_original_stores":0,"accepted_original_bodies":0,
        "native_export":"REFUSED","all18":"OPEN","original_applicability":"OPEN: complete incoming frontier, layout, nonnull branch, caller flow and runtime selection obligations"}))
}
