# Independent D2 review: private method evidence corroborated; keep user stage inactive

Review target: prepared root `3a1f9fce40f635b96fe012620594974b18d934ec`,
worker `cc9b088c5`, production `3eba70cc0d7ad56f132056e82e1b7709ad9438c4`
integrated as `094b7a42f`, including the prospective changes. Early binding
commit `bd9a94637`; validation commit `de4ef758e`. Parent retains acceptance.

The narrow reported experiment is corroborated: three complete, manually emitted
native-method Functions pass ordinary private body queries with empty requires;
three fresh diagnostic Functions with their first equality negated return Invalid.
The emitted computations are checked against actual loaded native MIR, rather
than inferred from a method name or operation descriptor. This is experimental
compiler evidence under new TCB, not a source-to-machine simulation theorem.
No blocking logical counterexample to that exact result was found. One P3
robustness defect was reproduced below. No stage, user-client, field-export,
native bridge or application acceptance is granted by this review.

## Finding D2-R1 — P3: empty reconstructed unit panics while reporting rejection

`source/vir/src/context.rs:805` indexes the **rejected** unit's `datatypes[0]`
inside the `map_err` closure. Given an authorized Ctx, a reconstructed/pruned
unit with zero datatypes correctly fails the binding's pointer check, but error
construction then panics with index-out-of-bounds instead of returning VirErr.

The independent `boundary.py`/`boundary_tail.rs` challenge reproduces this against
our freshly built VIR library. Its caught panic is explicit in the raw log at
`/tmp/owned-cell-d2-review-boundary-20261112-1916/run.stderr`. Other stale units
with a datatype return Err as expected. The fixture first establishes a valid
synthetic canonical unit/context; it does not use unsafe code or modify production.

This is an internal error-path crash, not an authority bypass or an accepted
proof. No current user-source trigger was established; installation remains
inactive. A stable bound span or a checked optional lookup would avoid the crash.
Leave its disposition to parent; no production fix was made here.

## Material limitations and trust objections

**No false-future solver control was delivered.** The get_mut negative negates
only `current(result) = current(input).field`. Its second equality remains
`future(input).field = future(result)`. The two future/current shape corruptions
reject at authority acquisition, before SMT; they are not semantic falsification
of future/writeback behavior. This does not contradict the observed positive
query checking both equalities, or the exact three false-FIRST-contract claim.
It is material before using this milestone to authorize future-state/client
effects: a false-future query, with a healthy current relation and body, must be
distinguished from shape refusals. That additional control was not implemented
or credited in this evidence-only review.

**The frontend-to-VIR provenance boundary is compiler TCB.** The legitimate
frontend acquisition re-reads metadata/signatures/MIR and correspondence. The
public VIR `from_checked_native_unit` and negative factory take an ordinary
Krate and published integer identities; they do not demand a rustc-owned witness.
Our synthetic fixture can therefore acquire a canonical VIR authority without
any TyCtxt. It cannot acquire one for arbitrary contracts/bodies. This is not a
user-language exploit: no source attribute/import invokes that API. It means
“sole frontend acquisition path” describes current trusted call sites, not an
unforgeable cross-crate proof token. WF/backend revalidation rechecks immutable
objects and canonical structure; it does not independently run rustc acquisition.
Any future installer must preserve that trusted boundary explicitly.

Initial, merged and final crate WF, full per-Function WF/modes, and backend
queries are real. The compiler-unit callback runs after rustc analysis. These
facts do not constitute original-client lifetime, trait, erasure or native-call
lowering validation: the Functions are compiler-created from inspected native
computation, and no original client is translated through them. No original-name
F3 entry or actual user erasure trial was delivered. No blanket guarantee against
arbitrary malicious compiler code or mutations to downstream SST/AIR is claimed.

## Actual source and manual TCB inventory

All six production files were read, including the complete validators, relevant
inherited metadata/MIR checks, and ordinary borrow/resolution/query code.

| File | Audited obligation and result |
|---|---|
| `rust_verify/src/checked_owned_cell_lowering.rs` | `build`, `corresponds`, `propose`, fresh `method_authority_kind`, test pipeline. Typed parameters/returns, original input place, field/variant, Copy/Move/reborrow and native-origin mapping agree. Fresh SpanContext IDs replace reused Temporary IDs. No omitted executable statement/block is permitted by the native checker. |
| `vir/src/internal_repr.rs` | Full current Function attributes, generics/bounds, visibility/local ownership, modes, empty requires, exact bodies and four canonical ensures; separate negative query kind. Borrowed non-Clone/nonserializable authority pins actual Krate/Datatype/Function Arcs. Backend owns a strong unit Arc. Canonical empty tuple is the sole extra datatype. |
| `vir/src/well_formed.rs` | Explicit native-method entry validates the unit and exact Function, forces no_cheating=true/no_verify=false, then uses ordinary checking. Field access exemption requires the actual descriptor returned by WF lookup and the exact method object. Ordinary source entry remains unprivileged. |
| `vir/src/context.rs` | Ordinary Ctx rejects owned-descriptor units containing Functions without authority. Empty-Function metadata context remains available but refuses Function lowering. Authorized creation validates descriptor/attribute map object identity and real helper module. Later Function checks require actual descriptor and func_map pointer; whole-unit check requires exact unit. D2-R1 affects its rejection diagnostic. |
| `vir/src/ast_to_sst_func.rs` | Revalidation at all four exposed Function entries: declaration, axioms, body definition and combined Function lowering. No new callable summary or source registration is introduced here. |
| `vir/src/ast_to_sst_crate.rs` | Whole-unit revalidation precedes normal Function lowering and elaboration. All three Functions and three full-body queries are required by the native unit test. |

Paths above are relative to `source/`. The new native acquisition/correspondence,
canonical Function validator and scoped access/backend APIs are additional
compiler TCB. Inherited rustc metadata/MIR/trait analysis, the existing datatype
encoding, ownership/prophecy lowering, AIR prelude, Z3, toolchain and normal vstd
remain trust inputs. The distinct native/source historical `0/44 + 3/101` record
is unchanged and receives no credit from these three private method queries.

`checked_owned_cell.rs:46–318` checks the native UnsafeCell lang item, pinned
core/ADT/private-field identities, transparent representation, native generics,
?Sized, no destructor and !Sync. Method acquisition checks inherent native DefIds,
const/safe Rust signatures, resolution to the original item, and get_mut's same
bound input/output region before erasure. Complete optimized MIR is restricted to
two locals, one block, one return-place assignment and Return; no cleanup/call/
drop/cast/extra executable computation survives. `corresponds` independently
matches the resulting actual Function's places, types and read modes. The VIR
validator then checks contracts and complete attribute/state exclusions. Full
FunctionX fields were cross-checked against `ast.rs`; the attribute destructure
is exhaustive, with every current value constrained.

The four positive postconditions are:

1. new: `result.field = input`.
2. into_inner: `result = input.field`.
3. get_mut: `current(result) = current(input).field`.
4. get_mut: `future(input).field = future(result)`.

No constructor-history premise, supplied native summary, PCell replacement,
extra require, admit or custom axiom establishes these query outcomes. New and
into_inner have no resolution tail. get_mut retains normal
`EvalAndResolve(BorrowMut(original input.field), HasResolved(original input))`.
Both independent validators constrain its exact type/place, Spec read, sentinel
read ID, boolean predicate and unit tail; five corruptions reject. The return
reference itself is not resolved by that tail.

The inherited `resolution_inference.rs` explanation covers this exact reborrow
case. Ordinary `borrow_mut_to_sst` first relates the returned borrow's current
value to the original field, then updates that field with the borrow's arbitrary
future. `EvalAndResolve` evaluates this computation before the original-input
resolvedness tail. Thus the tail uses the already updated input reference; it
does not independently invent the returned current/future or a field assignment.
This reasoning relies on existing ownership/prophecy semantics and the new
correct native-to-Function matching; body matching alone is not a machine proof.

## Validation and provenance

`BINDING.json` authenticates all 14 delivered source hashes, all eight snapshot
tools, and the original native-query executable before copying/replay. The
snapshot receipt and delivered root replay receipt match the exact supplied
hashes. `authenticate.py` additionally hashes all 11 explicitly listed original
receipt/log sets (33 files), including failures. Those terminal artifacts were
read only for hash authentication; actor checkout/build/bundle and mutable
shared caches were not read. Four exact root adoptions and earlier root review
are background, not independent D2 acceptance or a repeated root-gate review.

| Validation | Result and scope |
|---|---|
| Unmodified original D2 native executable, exact requested test | Passed: 3 positive full-body queries, 3 Invalid false-first-contract full-body queries; no unknown/resource result accepted. The source invokes ordinary initial/merged/final WF, full Function WF/modes, resolution, simplify, prune, fresh acquisition, normal AST-to-SST/elaboration/poly and body-to-AIR queries. |
| Original mutation groups in that same unit | 54 MIR +4 signature/region, 45 emitted correspondence, 26 full-Function/contract and 5 resolution controls rerun; ordinary field privacy, copied unit/Function/descriptor and stale func_map refusals included. They remain structural controls rather than extra SMT falsifications. |
| Original root refusal/admission executables with D2 tools | 5 +7 tests passed, reproducing the original 226 compiler invocations. Harnesses are unchanged and resolve the D2 tools beside their relocated executables. Old native/VIR binaries were not run as D2 validation. |
| Fresh complete VIR source library build | 172/172 unit tests, including 9 internal-representation/metadata tests (8 inherited Rev tests). Fresh offline dependencies and build directory; no copied build tree, rust_verify/verifier-macro/vstd rebuild or disabled verifier check. Required VIR/air dependencies, including vir-macros, were built. |
| Existing actual Rev source probes | All 8 ordinary/Infer outcomes preserved under --no-cheating --compile: available observations 1/0; lawful 0/1; inner identity 0/1; literal loop 1/1, each in both modes. Failed crates grant no whole-crate proof credit. This is actual source compatibility, not installation of Internal Rev authority. |
| Fresh-library synthetic API challenge | Exact Arc alias succeeds; reconstructed unit/Function and stale func_map reject; positive/diagnostic authority do not transfer; default context refuses; metadata context is inert for Functions. D2-R1 is caught and recorded as a known panic, not a clean rejection. No native or solver proof credit. |

The compiler and native test executable were **worker-built artifacts replayed
independently**, not rebuilt by this reviewer. Seven other tools are inherited
unchanged, with normal vstd 2257/0, not reverified. Only the VIR library and its
required dependencies were freshly built from this prepared checkout. The
synthetic boundary executable links those fresh libraries. All raw outputs,
temporary source, tool copies, binaries and caches remain in unique `/tmp`
directories named by `VALIDATION.json`; only compact review evidence is committed.

Retained review failures: the normal sandbox failed before command execution
(`bwrap: loopback: Failed RTM_NEWADDR`); authorized escalated commands were used.
The first local commit needed command-local author identity. The first replay
wrapper expected 3 refusal tests, but the original executable passed 5; its
receipt stays failed, and a new output reran all 12 with correct 5/7 inventories.
The first fresh `cargo test -p vir` passed all 172 units, then failed 18 doctests
on documentation pseudocode/snippets. A separate complete `--lib` run passed;
the full Cargo invocation still remains failed. Those failing examples' source
files are unchanged by D2, and no doctest repair or unrelated regression suite
was pursued. No failure was erased or counted as a semantic negative.

## Bounded value checkpoint and remaining gates

This review stops at corroborated private-method evidence, the explicit
false-future coverage objection, the compiler-factory TCB boundary, and reproduced
D2-R1. COST.json records the conservative active charge, exact command costs,
build exclusion, and disk floor. CPU and monetary measurements are unavailable.
No new prerequisite or implementation phase starts here.

Every experimental user request still refuses before imports/pruning/cache reuse;
selected explicit/Cargo outputs are invalidated. Internal descriptors refuse wire
serialization/deserialization, and authority is absent from FunctionStub/attrs/
serialized records. No client call rewrite, public selector, source installer or
trusted registration was found. Rev-only behavior retains its existing family
and fast path. No global schema39/policy78/cache81 or default change is made.

Actual admitted→translated/checked reconciliation, original client-body/call and
current/future/store/writeback correspondence, distinct private F2 arbitrary-entry
final-input-state proof with false-old/deleted-store controls, and original-name
F3 erasure/compile entries remain FUTURE gates. Original client body proofs=0;
F2 queries=0; F3 entries=0. Complete stage/F1 acceptance remains withheld.
CP1=false; CP2=false; source_coupling_proved=false; native_bridge_proved=false.
The REDB theorem and Mutex/guard/cache/write/publication/recovery coupling remain
unproved. HMAC/native remains DEFERRED AND UNACHIEVED.
