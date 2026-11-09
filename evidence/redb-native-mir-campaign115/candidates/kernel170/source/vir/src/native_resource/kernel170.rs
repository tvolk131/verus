use crate::ast::*;
use crate::ast_util::{mk_tuple_field_opr, types_equal};
use crate::def::Spanned;
use crate::messages::Span;
use crate::sst::{self, Dest, Exp, ExpX, Stm, StmX};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub const LAW170: &str = "NATIVE_RAW_ALLOC_NORMAL158_V1";
pub const DISPOSITION170: &str = "PRE_REVIEW_CANDIDATE_INSPECTION_ONLY";

pub use crate::sst_to_air::inspect_native170_to_air;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal170 {
    ReviewRequired, WrongCanonicalLibrary, MissingCanonicalDefinition, WrongSource,
    WrongCallee, WrongRuntime, WrongAbi, WrongAlignmentInterpretation, UnknownFrontier,
    NonemptyFrontier, Replay, WrongInstruction, SubstitutedReturn, SubstitutedSize,
    SubstitutedAlign, UnsupportedOutcome, UnsupportedCleanup, WrongControl,
    StaleHolder, LiveDestination, Borrowed, DuplicateResource, WrongProvenance,
    ResourceLeak, BranchMismatch, InvalidFrontierCondition, WrongType,
}

pub fn operative_admission170() -> Result<(), Refusal170> { Err(Refusal170::ReviewRequired) }

pub struct Canonical170 {
    pub allocate: Function,
    pub deallocate: Function,
    pub address: Function,
    pub result: Typ,
    pub raw: Typ,
    pub dealloc: Typ,
    pub artifact_sha256: String,
}

fn canonical_path(function: &Function, name: &str) -> bool {
    function.x.name.path.krate == CrateId::Vstd
        && function.x.name.path.segments.iter().map(|segment| segment.as_str()).eq(["raw_ptr", name])
}

impl Canonical170 {
    pub fn from_authenticated_library(krate: &Krate, artifact_sha256: String)
        -> Result<Self, Refusal170>
    {
        if artifact_sha256.len() != 64 || !artifact_sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(Refusal170::WrongCanonicalLibrary);
        }
        let find = |name| {
            let matches = krate.functions.iter().filter(|function| canonical_path(function, name)).collect::<Vec<_>>();
            if matches.len() != 1 { return Err(Refusal170::MissingCanonicalDefinition); }
            Ok(matches[0].clone())
        };
        let allocate = find("allocate")?;
        let deallocate = find("deallocate")?;
        let address = krate.functions.iter().filter(|function|
            function.x.name.path.krate == CrateId::Vstd && function.x.name.path.segments.iter()
                .map(|part|part.as_str()).eq(["raw_ptr","ptr_mut_specs","spec_addr"]))
            .collect::<Vec<_>>();
        if address.len()!=1 || address[0].x.mode!=Mode::Spec || address[0].x.params.len()!=1 {
            return Err(Refusal170::MissingCanonicalDefinition);
        }
        let address=address[0].clone();
        let find_resource = |name| {
            let matches = krate.datatypes.iter().filter(|datatype| match &datatype.x.name {
                Dt::Path(path) => path.krate == CrateId::Vstd
                    && path.segments.iter().map(|segment| segment.as_str()).eq(["raw_ptr", name])
                    && datatype.x.mode == Mode::Proof,
                _ => false,
            }).collect::<Vec<_>>();
            if matches.len() != 1 { return Err(Refusal170::MissingCanonicalDefinition); }
            Ok(Arc::new(TypX::Datatype(matches[0].x.name.clone(), Arc::new(vec![]), Arc::new(vec![]))))
        };
        let raw = find_resource("PointsToRaw")?;
        let dealloc = find_resource("Dealloc")?;
        if allocate.x.mode != Mode::Exec || allocate.x.params.len() != 2
            || allocate.x.require.len() != 2 || allocate.x.ensure.0.len() != 5 || !allocate.x.ensure.1.is_empty()
            || allocate.x.params.iter().any(|parameter| !matches!(&*parameter.x.typ, TypX::Int(IntRange::USize)))
            || !types_equal(&allocate.x.outer_ret.x.typ, &super::adapter164::result_type())
            || deallocate.x.params.len() != 5 || deallocate.x.mode != Mode::Exec
        { return Err(Refusal170::WrongCanonicalLibrary); }
        Ok(Self { result: allocate.x.outer_ret.x.typ.clone(), allocate, deallocate, address, raw, dealloc, artifact_sha256 })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source170 {
    pub root_source_sha256: String,
    pub owner_mir_sha256: String,
    pub owner: String,
    pub callee: String,
    pub runtime_sha256: String,
    pub target: String,
    pub abi: String,
    pub alignment: String,
    pub alignment_method_mir_sha256: String,
    pub block: usize,
    pub normal: usize,
    pub native_destination: usize,
    pub size_operand: String,
    pub align_operand: String,
}

impl Source170 {
    fn validate(&self) -> Result<(), Refusal170> {
        let digest = |text: &str| text.len() == 64 && text.bytes().all(|byte| byte.is_ascii_hexdigit());
        if !digest(&self.root_source_sha256) || !digest(&self.owner_mir_sha256)
            || !self.owner.contains("alloc_impl_runtime") || self.size_operand.is_empty() || self.align_operand.is_empty()
        { return Err(Refusal170::WrongSource); }
        if self.callee != "alloc::alloc::__rust_alloc" { return Err(Refusal170::WrongCallee); }
        if !digest(&self.runtime_sha256) || self.target != "aarch64-apple-darwin" { return Err(Refusal170::WrongRuntime); }
        if self.abi != "Rust; usize, core::mem::Alignment -> *mut u8" { return Err(Refusal170::WrongAbi); }
        if self.alignment != "core::mem::Alignment::as_usize(actual operand)" || !digest(&self.alignment_method_mir_sha256) {
            return Err(Refusal170::WrongAlignmentInterpretation);
        }
        Ok(())
    }
}

pub enum Incoming170 {
    Unknown,
    Nonempty,
    ConditionalEmpty { complete_native_frontier_is_empty: Exp },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome170 { NormalNonNull, Null, Error, Unwind, Abort, Divergence }

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Allocation170 { activation: u64, evaluation: u64, path: Vec<(u64, bool)> }

#[derive(Clone)]
struct Resource170 {
    allocation: Allocation170, pointer: Exp, raw: Exp, dealloc: Exp, bundle: Exp,
    size: Exp, align: Exp, loans: BTreeSet<u64>,
}

#[derive(Clone)]
struct Heap170 { native_pointer: Exp, generation: Allocation170, live: bool }

pub struct Kernel170 {
    source: Source170, activation: Vec<u64>, next_activation: u64, evaluation: u64,
    pc: usize, path: Vec<(u64, bool)>, issued: BTreeSet<Allocation170>,
    holders: BTreeMap<usize, Resource170>, heap: BTreeMap<Allocation170, Heap170>,
    native_locals: BTreeMap<usize, Exp>, aliases: BTreeMap<usize, Allocation170>,
    loans: BTreeMap<u64, (usize, Allocation170)>, incoming_empty: Exp,
    frontier_obligation_pending: bool, returned: bool,
}

pub struct Call170 {
    source: Source170, activation: u64, evaluation: u64, path: Vec<(u64, bool)>,
    instruction: Stm, size: Exp, align: Exp, holder: usize, outcome: Outcome170,
}

pub struct Transition170 {
    pub canonical_call: Stm,
    pub incoming_empty_obligation: Option<Exp>,
    pub native_return: Exp,
    pub raw: Exp,
    pub dealloc: Exp,
    pub allocation: Allocation170,
    pub air_local_decls: Vec<air::ast::Decl>,
}

fn identical(left: &Exp, right: &Exp) -> bool { Arc::ptr_eq(left, right) }

fn read_destination(destination: &Dest) -> Result<Exp, Refusal170> {
    if !destination.is_init { return Err(Refusal170::LiveDestination); }
    let ExpX::VarLoc(name) = &destination.dest.x else { return Err(Refusal170::WrongInstruction); };
    Ok(SpannedTyped::new(&destination.dest.span, &destination.dest.typ, ExpX::Var(name.clone())))
}

fn component(bundle: &Exp, typ: Typ, field: usize) -> Exp {
    SpannedTyped::new(&bundle.span, &typ, ExpX::UnaryOpr(UnaryOpr::Field(mk_tuple_field_opr(3, field)), bundle.clone()))
}

impl Kernel170 {
    fn exclusive_alternative(&self) -> Self {
        Self { source: self.source.clone(), activation: self.activation.clone(), next_activation: self.next_activation,
            evaluation: self.evaluation, pc: self.pc, path: self.path.clone(), issued: self.issued.clone(),
            holders: self.holders.clone(), heap: self.heap.clone(), native_locals: self.native_locals.clone(),
            aliases: self.aliases.clone(), loans: self.loans.clone(), incoming_empty: self.incoming_empty.clone(),
            frontier_obligation_pending: self.frontier_obligation_pending, returned: self.returned }
    }
    pub fn enter(source: Source170, incoming: Incoming170) -> Result<Self, Refusal170> {
        source.validate()?;
        let incoming_empty = match incoming {
            Incoming170::Unknown => return Err(Refusal170::UnknownFrontier),
            Incoming170::Nonempty => return Err(Refusal170::NonemptyFrontier),
            Incoming170::ConditionalEmpty { complete_native_frontier_is_empty: condition } => {
                if !matches!(&*condition.typ, TypX::Bool) || !matches!(&condition.x, ExpX::Var(_)) {
                    return Err(Refusal170::InvalidFrontierCondition);
                }
                condition
            }
        };
        Ok(Self { pc: source.block, source, activation: vec![1], next_activation: 2,
            evaluation: 0, path: vec![], issued: BTreeSet::new(), holders: BTreeMap::new(),
            heap: BTreeMap::new(), native_locals: BTreeMap::new(), aliases: BTreeMap::new(),
            loans: BTreeMap::new(), incoming_empty, frontier_obligation_pending: true, returned: false })
    }

    pub fn prepare_call(&self, instruction: Stm, holder: usize, outcome: Outcome170) -> Result<Call170, Refusal170> {
        if self.returned || self.pc != self.source.block { return Err(Refusal170::WrongControl); }
        let StmX::Call { fun: sst::CallTarget::NativeAllocator164, args, dest: Some(destination),
            mode: Mode::Exec, body: None, typ_args, resolved_method: None,
            is_trait_default: None, split: None, .. } = &instruction.x
        else { return Err(Refusal170::WrongInstruction); };
        if args.len() != 2 || !typ_args.is_empty() || !destination.is_init
            || args.iter().any(|value| !matches!(&*value.typ, TypX::Int(IntRange::USize)))
        { return Err(Refusal170::WrongInstruction); }
        Ok(Call170 { source: self.source.clone(), activation: *self.activation.last().unwrap(),
            evaluation: self.evaluation, path: self.path.clone(), size: args[0].clone(),
            align: args[1].clone(), instruction, holder, outcome })
    }

    pub fn allocate(&mut self, canonical: &Canonical170, call: &Call170, instruction: &Stm)
        -> Result<Transition170, Refusal170>
    {
        if call.source != self.source { return Err(Refusal170::WrongSource); }
        if call.outcome != Outcome170::NormalNonNull { return Err(Refusal170::UnsupportedOutcome); }
        if call.activation != *self.activation.last().unwrap() || call.evaluation != self.evaluation
            || call.path != self.path || self.pc != self.source.block { return Err(Refusal170::Replay); }
        let StmX::Call { args, dest: Some(destination), .. } = &instruction.x else { return Err(Refusal170::WrongInstruction); };
        if args.len() != 2 { return Err(Refusal170::WrongInstruction); }
        if !identical(&args[0], &call.size) { return Err(Refusal170::SubstitutedSize); }
        if !identical(&args[1], &call.align) { return Err(Refusal170::SubstitutedAlign); }
        if !Arc::ptr_eq(instruction, &call.instruction) { return Err(Refusal170::SubstitutedReturn); }
        if !types_equal(&destination.dest.typ, &canonical.result) { return Err(Refusal170::WrongType); }
        if self.holders.contains_key(&call.holder) { return Err(Refusal170::LiveDestination); }
        if !self.holders.is_empty() || self.heap.values().any(|entry| entry.live) || !self.loans.is_empty() {
            return Err(Refusal170::NonemptyFrontier);
        }
        let allocation = Allocation170 { activation: call.activation, evaluation: call.evaluation, path: self.path.clone() };
        if self.issued.contains(&allocation) { return Err(Refusal170::Replay); }
        let bundle = read_destination(destination)?;
        let pointer_type = Arc::new(TypX::Primitive(Primitive::Ptr, Arc::new(vec![Arc::new(TypX::Int(IntRange::U(8)))])));
        let native_return = component(&bundle, pointer_type, 0);
        let raw = component(&bundle, Arc::new(TypX::Decorate(TypDecoration::Tracked, None, canonical.raw.clone())), 1);
        let dealloc = component(&bundle, Arc::new(TypX::Decorate(TypDecoration::Tracked, None, canonical.dealloc.clone())), 2);
        let canonical_call = Spanned::new(instruction.span.clone(), StmX::Call {
            fun: sst::CallTarget::Fun(canonical.allocate.x.name.clone()), resolved_method: None,
            mode: Mode::Exec, is_trait_default: None, typ_args: Arc::new(vec![]),
            args: Arc::new(vec![call.size.clone(), call.align.clone()]), split: None,
            dest: Some(destination.clone()), assert_id: None, body: None,
        });
        self.issued.insert(allocation.clone());
        self.heap.insert(allocation.clone(), Heap170 { native_pointer: native_return.clone(), generation: allocation.clone(), live: true });
        self.native_locals.insert(self.source.native_destination, native_return.clone());
        self.holders.insert(call.holder, Resource170 { allocation: allocation.clone(), pointer: native_return.clone(),
            raw: raw.clone(), dealloc: dealloc.clone(), bundle, size: call.size.clone(), align: call.align.clone(), loans: BTreeSet::new() });
        self.evaluation += 1;
        self.pc = self.source.normal;
        let incoming_empty_obligation = self.frontier_obligation_pending.then(|| self.incoming_empty.clone());
        self.frontier_obligation_pending = false;
        Ok(Transition170 { canonical_call, incoming_empty_obligation, native_return, raw, dealloc, allocation, air_local_decls: vec![] })
    }

    fn resource(&self, holder: usize) -> Result<&Resource170, Refusal170> {
        let resource = self.holders.get(&holder).ok_or(Refusal170::StaleHolder)?;
        let entry = self.heap.get(&resource.allocation).ok_or(Refusal170::WrongProvenance)?;
        if !entry.live || entry.generation != resource.allocation || !identical(&entry.native_pointer, &resource.pointer) {
            return Err(Refusal170::WrongProvenance);
        }
        Ok(resource)
    }

    fn movable(&self, holder: usize) -> Result<&Resource170, Refusal170> {
        let resource = self.resource(holder)?;
        if !resource.loans.is_empty() { return Err(Refusal170::Borrowed); }
        Ok(resource)
    }

    pub fn move_holder(&mut self, source: usize, destination: usize) -> Result<(), Refusal170> {
        self.movable(source)?;
        if source == destination || self.holders.contains_key(&destination) { return Err(Refusal170::LiveDestination); }
        let resource = self.holders.remove(&source).unwrap();
        self.holders.insert(destination, resource);
        Ok(())
    }

    pub fn copy_resources(&self, holder: usize) -> Result<(), Refusal170> {
        self.resource(holder)?;
        Err(Refusal170::DuplicateResource)
    }

    pub fn overwrite(&self, holder: usize) -> Result<(), Refusal170> {
        if self.holders.contains_key(&holder) { return Err(Refusal170::ResourceLeak); }
        Ok(())
    }

    pub fn alias_pointer(&mut self, holder: usize, local: usize) -> Result<Exp, Refusal170> {
        let resource = self.resource(holder)?.clone();
        if self.holders.contains_key(&local) { return Err(Refusal170::LiveDestination); }
        self.aliases.insert(local, resource.allocation);
        self.native_locals.insert(local, resource.pointer.clone());
        Ok(resource.pointer)
    }

    pub fn read_alias(&self, local: usize) -> Result<Exp, Refusal170> {
        let allocation = self.aliases.get(&local).ok_or(Refusal170::StaleHolder)?;
        let entry = self.heap.get(allocation).ok_or(Refusal170::WrongProvenance)?;
        if !entry.live { return Err(Refusal170::StaleHolder); }
        Ok(entry.native_pointer.clone())
    }

    pub fn borrow(&mut self, holder: usize, loan: u64) -> Result<(), Refusal170> {
        let allocation = self.movable(holder)?.allocation.clone();
        if self.loans.contains_key(&loan) { return Err(Refusal170::Borrowed); }
        self.loans.insert(loan, (holder, allocation));
        self.holders.get_mut(&holder).unwrap().loans.insert(loan);
        Ok(())
    }

    pub fn end_loan(&mut self, loan: u64) -> Result<(), Refusal170> {
        let (holder, generation) = self.loans.get(&loan).ok_or(Refusal170::StaleHolder)?.clone();
        if self.resource(holder)?.allocation != generation { return Err(Refusal170::WrongProvenance); }
        self.loans.remove(&loan);
        self.holders.get_mut(&holder).unwrap().loans.remove(&loan);
        Ok(())
    }

    pub fn deallocate(&mut self, canonical: &Canonical170, holder: usize, native_pointer: &Exp,
        size: &Exp, align: &Exp) -> Result<Stm, Refusal170>
    {
        let resource = self.movable(holder)?.clone();
        if !identical(native_pointer, &resource.pointer) { return Err(Refusal170::WrongProvenance); }
        if !identical(size, &resource.size) { return Err(Refusal170::SubstitutedSize); }
        if !identical(align, &resource.align) { return Err(Refusal170::SubstitutedAlign); }
        let statement = Spanned::new(native_pointer.span.clone(), StmX::Call {
            fun: sst::CallTarget::Fun(canonical.deallocate.x.name.clone()), resolved_method: None,
            mode: Mode::Exec, is_trait_default: None, typ_args: Arc::new(vec![]),
            args: Arc::new(vec![resource.pointer.clone(), resource.size.clone(), resource.align.clone(), resource.raw.clone(), resource.dealloc.clone()]),
            split: None, dest: None, assert_id: None, body: None,
        });
        self.holders.remove(&holder);
        self.heap.get_mut(&resource.allocation).unwrap().live = false;
        Ok(statement)
    }

    fn fork(self, decision: u64) -> Result<(Self, Self), Refusal170> {
        if self.path.iter().any(|(previous, _)| *previous == decision) { return Err(Refusal170::BranchMismatch); }
        let mut positive = self.exclusive_alternative();
        let mut negative = self;
        positive.path.push((decision, true));
        negative.path.push((decision, false));
        Ok((positive, negative))
    }

    fn join(mut positive: Self, negative: Self) -> Result<Self, Refusal170> {
        let positive_edge = positive.path.pop().ok_or(Refusal170::BranchMismatch)?;
        let mut negative_path = negative.path.clone();
        let negative_edge = negative_path.pop().ok_or(Refusal170::BranchMismatch)?;
        if positive_edge.0 != negative_edge.0 || !positive_edge.1 || negative_edge.1
            || positive.path != negative_path || positive.pc != negative.pc
            || positive.activation != negative.activation || positive.evaluation != negative.evaluation
            || positive.returned != negative.returned || positive.loans != negative.loans
            || positive.holders.len() != negative.holders.len()
            || positive.holders.iter().any(|(holder, resource)| negative.holders.get(holder)
                .is_none_or(|other| resource.allocation != other.allocation || !identical(&resource.bundle, &other.bundle) || resource.loans != other.loans))
            || positive.heap.len() != negative.heap.len()
            || positive.heap.iter().any(|(allocation, entry)| negative.heap.get(allocation).is_none_or(|other| entry.live != other.live))
        { return Err(Refusal170::BranchMismatch); }
        positive.issued.extend(negative.issued);
        Ok(positive)
    }

    pub fn branch<Positive, Negative>(self, ctx: &crate::context::Ctx, condition: Exp, decision: u64,
        positive: Positive, negative: Negative) -> Result<(Self, air::ast::Stmt), Refusal170>
    where Positive: FnOnce(&mut Self) -> Result<Vec<air::ast::Stmt>, Refusal170>,
        Negative: FnOnce(&mut Self) -> Result<Vec<air::ast::Stmt>, Refusal170>
    {
        if !matches!(&*condition.typ, TypX::Bool) { return Err(Refusal170::WrongType); }
        let (mut positive_state, mut negative_state) = self.fork(decision)?;
        let positive_air = positive(&mut positive_state)?;
        let negative_air = negative(&mut negative_state)?;
        let merged = Self::join(positive_state, negative_state)?;
        let air = crate::sst_to_air::native170_branch_air(ctx, &condition, positive_air, negative_air)
            .map_err(|_| Refusal170::WrongInstruction)?;
        Ok((merged, air))
    }

    pub fn enter_activation(&mut self, entry: usize) -> Result<(), Refusal170> {
        if self.returned || entry != self.source.block { return Err(Refusal170::WrongControl); }
        if !self.holders.is_empty() || !self.loans.is_empty() || self.heap.values().any(|entry| entry.live) { return Err(Refusal170::NonemptyFrontier); }
        self.activation.push(self.next_activation);
        self.next_activation += 1;
        self.pc = entry;
        Ok(())
    }

    pub fn goto(&mut self, expected: usize, successor: usize, cleanup: bool) -> Result<(), Refusal170> {
        if cleanup { return Err(Refusal170::UnsupportedCleanup); }
        if self.pc != expected || self.returned || expected != self.source.block || successor != self.source.normal {
            return Err(Refusal170::WrongControl);
        }
        self.pc = successor;
        Ok(())
    }

    pub fn return_resources(&mut self, holders: &[usize]) -> Result<Vec<Exp>, Refusal170> {
        let unique = holders.iter().copied().collect::<BTreeSet<_>>();
        if unique.len() != holders.len() { return Err(Refusal170::DuplicateResource); }
        if self.returned { return Err(Refusal170::Replay); }
        if !self.loans.is_empty() { return Err(Refusal170::Borrowed); }
        if unique.len() != self.holders.len() || self.holders.keys().any(|holder| !unique.contains(holder)) { return Err(Refusal170::ResourceLeak); }
        let mut outputs = Vec::new();
        for holder in holders {
            self.movable(*holder)?;
            outputs.push(self.holders.remove(holder).unwrap().bundle);
        }
        self.returned = true;
        Ok(outputs)
    }

    pub fn native_return(&self) -> Result<Exp, Refusal170> {
        self.native_locals.get(&self.source.native_destination).cloned().ok_or(Refusal170::StaleHolder)
    }

    pub fn live_count(&self) -> usize { self.holders.len() }
}

pub fn make_instruction170(span: &Span, canonical: &Canonical170, binder: VarIdent, size: Exp, align: Exp) -> Stm {
    Spanned::new(span.clone(), StmX::Call { fun: sst::CallTarget::NativeAllocator164,
        resolved_method: None, mode: Mode::Exec, is_trait_default: None, typ_args: Arc::new(vec![]),
        args: Arc::new(vec![size, align]), split: None,
        dest: Some(Dest { dest: SpannedTyped::new(span, &canonical.result, ExpX::VarLoc(binder)), is_init: true }),
        assert_id: None, body: None })
}
