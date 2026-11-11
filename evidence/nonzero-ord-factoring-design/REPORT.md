# NonZero Ord factoring: independent source design gate

**FEASIBLE_WITH_CONDITIONS.** A bounded inverse-inlining checker can recover two
calls to the **existing** `NonZero::<u64>::get` boundary and retain the actual
nested `<u64 as Ord>::cmp` call with its **existing** model. No additional
value-to-view fact is logically necessary on that route. This is a design
recommendation, not an implemented transformation, accepted certificate, or
ordering theorem. The present compiler cannot certify this route.

Review base: `778c5e6a2a048e4a776be65ea354f73da4de10ba`. Exact source/dataset
SHA256s are in [INPUTS.json](INPUTS.json); line citations below are defined in
[SOURCE-CITATIONS.md](SOURCE-CITATIONS.md). All 168 sealed artifacts and four
controllers match their seal. Whole typed debug bodies of all seven instances
were read, including locals, body properties and every block; the inspector and
hashes remain diagnostic. No compiler/proof invocation was performed.

## 1. Actual bodies and the bounded factoring obligation

The pinned declaration has the private genuine `NonZero<u64>` field and all
seven derives [S1]. The complete diagnostic closure is [D1]:

| Instance | Complete typed locals, in index order | Entire executable CFG |
|---|---|---|
| `partial` | `Option<Ordering>, &BorTag, &BorTag` | bb0 calls derived partial_cmp on copies of arguments; bb1 returns its result |
| `total` | `Ordering, &BorTag, &BorTag` | bb0 calls derived cmp on copies of arguments; bb1 returns its result |
| Derived `partial_cmp` | `Option<Ordering>, &BorTag, &BorTag, Ordering` | bb0 calls derived cmp into local 3; bb1 returns `Some(move 3)` |
| Derived `cmp` | `Ordering, &BorTag, &BorTag, &NonZero<u64>, &NonZero<u64>` | bb0 takes shared references to the respective field 0 and calls carrier cmp; bb1 returns |
| Carrier `cmp::<u64>` | `Ordering, &NonZero<u64>, &NonZero<u64>, &u64, u64, NonZero<u64>, &u64, u64, NonZero<u64>` | bb0 has the two getter fragments, references to results, and primitive cmp into return place; bb1 ends storage and returns |
| `get::<u64>` | `u64, NonZero<u64>` | One block: copy formal 1, Transmute to u64 into return place, Return |
| Primitive `u64::cmp` | `Ordering, &u64, &u64, u64, u64` | One block: start locals 3/4, copy dereferenced inputs, `Cmp(move 3, move 4)` into return place, end storage, Return |

All blocks are non-cleanup. The five two-block bodies use a single Call with
normal successor bb1 and **unwind Continue**; this edge must not be silently
deleted. The get and primitive bodies have no call/unwind edge. All seven are
Runtime/Optimized, argument counts 2 except get's 1, no coroutine, spread arg,
injection, taint or coverage, and empty required constants. Mentioned items are
empty for get/primitive and exactly the respective concrete trait call elsewhere.
User type annotations are empty; debug/scopes are non-executable. Carrier and
get retain `is_polymorphic=true`; the other five say false. These are observations
from complete retained bodies, not fresh compiler API checks of Copy/drop/types
or signature ABI/safety.

The narrowest rewrite leaves the argument reads and all surrounding statements
in place. In carrier bb0, statement indices are zero-based:

```text
at 3:  local5 = Copy(*argument1)             // NonZero<u64>
at 4:  local4 = Transmute<u64>(Copy(local5))
at 10: local8 = Copy(*argument2)             // NonZero<u64>
at 11: local7 = Transmute<u64>(Copy(local8))

replace only statement 4 by local4 = get::<u64>(Copy(local5))
replace only statement 11 by local7 = get::<u64>(Copy(local8))
```

These are two single-statement subgraphs (split the block at instruction
boundaries for the check), each with one entry and one normal exit. The standalone
get body substitutes formal 1 with local5/local8, and its return place with
local4/local7. Its **entire operation** then equals the original fragment.
Calling convention argument/return temporaries can be hidden only after checking
they are fresh Copy values without observable addresses, effects or drops.
The existing source scopes identify candidates; they do not establish this
equality. The checker must inspect the operation, operands, places and callee.

Sufficient conditions for this deliberately closed subset:

1. Resolve the real core NonZero ADT, get item with `[u64]`, and carrier Ord
   implementation by semantic identity and concrete signatures. Get is safe
   Rust ABI, by-value `NonZero<u64> -> u64`; carrier is
   `(&NonZero<u64>, &NonZero<u64>) -> core::cmp::Ordering`. No shims, dyn dispatch,
   user lookalikes, substitutions left open, alternative carrier or adapter.
2. Inspect the **whole** carrier and standalone getter. Require exactly the
   enumerated instructions/edges and allowed body properties, including
   unreachable blocks. Check dominance, initialization, live ranges, unique
   result definitions, uses, and no incoming edge into a fragment interior or
   outgoing edge other than its mapped continuation. No branch, loop, extra
   call, assert, write-through-reference, global/volatile/atomic access, raw
   pointer, cleanup, drop, unwind resume or hidden effect may be elided.
3. Check types recursively using compiler types, each dereference/field and
   assignment/cast output, and exact `CastKind::Transmute` with Copy operand.
   All relevant values must be Copy and need no drop in the instantiated env.
   Return locals are distinct from inputs. No moved-from read or read after
   StorageDead; `_4/_7` remain live while `_3/_6` borrow them. Preserve original
   read order, borrow sites, StorageLive/Dead order and call operands (including
   Move of shared refs). Any removal of storage markers needs a separately
   checked non-escape/no-drop justification; initial implementation can retain
   their correspondence without general storage optimization.
4. Inputs may alias **each other**: shared reads of the same genuine value are
   valid. Require no mutation between argument read and use; fresh getter
   temporaries cannot escape, overlap input storage or become observable except
   through their mapped result. No pointer identity comparison is admitted.
   Keep the shared-read/Copy identity of each logical NonZero value through VIR;
   do not copy bits into a fabricated logical NonZero value.
5. Preserve both normal return and possible unwind behavior of the remaining
   primitive call, with no cleanup. Get has an inherited no-unwind/no-invariant-
   opening summary [S2] and its inspected body has no failing operation/exit.
   Do not infer a stronger primitive no-unwind summary merely from its numeric
   postcondition. Ordinary total correctness/termination and safety checks stay.
6. Independently compare the **actual full ordinary Function**, not a plan
   descriptor, with original carrier MIR plus these two checked expansions:
   every read, borrow, call/model identity, operand order, return, type, effect
   and CFG exit must be accounted for. Check its full-domain contract separately.

The operational obligation is, for each fragment and every valid input state,
`fragment[p := n, out := x]` and evaluation of the resolved get body on the
**same n** have identical result and observable effects/exits, modulo fresh
non-escaping local storage. One can establish this by symbolic instruction
matching with explicit substitution, leaving Transmute uninterpreted on both
sides. Then the inherited get summary gives `x = n@`. This does not need a
separate arbitrary representation/view equality. If the exact expansion or
value/place map fails, the missing obligation is that operational equivalence;
the checker must reject, not assert `Transmute(n) = n@` to repair it.

## 2. Reuse primitive Ord; check concrete instantiation

**Yes, reuse the model.** Carrier bb0's actual resolved nested callee is core's
`Ord for u64::cmp`, with no instance arguments [D1]. `ExOrd::cmp` supplies the
conditional postcondition [S3]. `std_specs/num.rs:71–91`, instantiated for u64 at
line 601, supplies `obeys_cmp_spec() = true`, unsigned numeric Ordering, and the
existing assume-specification of this exact primitive method [S4]. Thus the
ordinary factored carrier can prove numeric order by normal get/get/cmp calls.
There is no need for a new primitive Cmp lowering, ordinary primitive proof,
discriminant subtraction, or new primitive trust. This narrows the inherited
PROPOSAL/MIR metadata's unnecessarily stronger primitive-proof requirement.

Bind trait DefPathHash/core crate identity, trait item, resolved impl/item,
normalized Self=u64, substitutions, signature, and actual call operands; map
that identity through the ordinary external-spec/trait-extension machinery to
the inherited primitive declaration and the actual u64 OrdSpecImpl functions.
Bind get's external-spec proxy, NonZero View, spec_get and contract too. Bind
the loaded vstd artifact covering these definitions and their dependencies,
plus exact core and executing compiler identities. Strings such as `impl#67`
or process-local DefIds are diagnostic coordinates, not canonical identities.
Ordering must be core's ADT and exact Less/Equal/Greater constructors; retain
unsigned 64-bit operand types. Wrong width/sign/constructor rejects even if a
sample result agrees. The primitive whole MIR remains a bound dependency, not
a new proof obligation to rederive its already inherited model.

Type completeness must be recomputed after instance substitution and
normalization, not decided by `is_polymorphic`. Walk signature, all local and
projection types, rvalue/operand/constant types, FnDef binders/substitutions,
mentioned/required items and nested targets. Reject unresolved type/const params,
inference/error types, escaping bound variables, unevaluated unsupported consts,
unresolved aliases/projections, opaque/unsupported generics and non-Item targets.
Permit only correctly scoped erased regions in shared references after normal
region handling; do not ban all reference lifetimes or erase higher-ranked
requirements to make a signature fit. Require exactly u64 in carrier/get args;
validate expected method late-bound regions and concrete normalized signatures.
The true flag here records generic origin; it neither proves nor refutes these
checks. The current bool recognizer's flag rejection remains unchanged [S5].

## 3. Minimal typed interface and staged charter

Illustrative interfaces only; **no implementation in this review**:

```text
resolve_ordering(tcx, actual_call) -> Result<ConcreteOrderingInstances>
  // semantic carrier/get/u64-Ord IDs, exact types/signatures/model bindings
check_get_expansion(carrier_body, get_body, typed_place_map, interval)
  -> Result<CheckedExpansion>   // private checker result, not supplied metadata
check_ordering_function(original_bodies, actual_Function, expansions, models)
  -> Result<OrderingBinding>    // includes full-domain contract; no proof token

ForeignEvidenceKind = BoolToU64 | NonZeroU64OrdViaExistingGet
OrderingBinding = {kind, semantic_instances, whole_MIRs, checked_Function,
                   expansion_maps, model_dependencies, compiler_core_target}
```

The existing bool semantic resolver, one-cast recognizer and MIR/actual-VIR
checker stay closed [S5]; add a distinct typed ordering path, rather than admit
Transmute or Ordering into that recognizer. Existing function identity
canonicalization assumes one input; two inputs, locals and dependency identities
need an injective typed binder map, preserving argument order [S6].

1. **Direct carrier first.** Add the bounded factoring/correspondence checks,
   normal call/model registration and a full ordinary executable helper for the
   actual carrier. Bind the original typed HIR call, receiver adjustments,
   caller/call-site IDs, typechecked substitutions and concrete resolved Instance;
   re-resolve at final source-call mapping. Do not rewrite function values or
   coercions, or replace the runtime source body. Prove
   `requires true; result = numeric_cmp(a@, b@)` over all
   genuine inputs with normal WF/mode/lifetime/trait/termination/erasure checks.
   It is a body-backed function, never an external-body leaf or trusted token
   [S7]. Avoid calling carrier Ord recursively or assuming a new NonZero
   OrdSpecImpl to prove itself. Existing get/u64-Ord models are the leaves.
2. **Original wrapper next.** Only after that proof, extend macro request and
   typed profile eligibility for nongeneric one-field genuine NonZero<u64>
   wrappers [S8]. Select the original generated Ord and PartialOrd bodies:
   respective field references into carrier cmp, then `Some(wrapper cmp)`.
   Preserve private field and all seven derives. Check both ordinary method
   bodies against authored numeric trait relations, with true obeys predicates;
   a selection marker or Eq marker supplies no law. Check all six actual
   comparison clients. Do not count other excluded derives as checked.
3. **Opacity/import last.** Keep `value = self.0@` closed and field private.
   Supply explicitly counted, normally checked ordering/value compatibility
   lemmas and matching ordinary goals-off clients; retain existing ONE checked
   Eq/value broadcast lemma plus client import as inherited practice. Export
   usable relation contracts without exporting closed bodies/representation.

The carrier/enum methods need ordinary proofs, not an inference theorem for enum
returns. If bool operator inference is used, its numeric observation is a
candidate only: fresh soundness, maximality over the full inherited domain, and
exactness must hold on both cache miss and hit. No <=2^61-1 premise, restricted
maximality domain or high-bit truncation is allowed.

## 4. Evidence/import/publication obligations

Current certificates are implicitly bool-specific; `current_identity` resolves
only From<bool> and imports cache one such identity [S6,S9]. Ordering needs an
explicit discriminant and kind-dispatched fresh resolution, with incompatible
old evidence rejected. A schema/policy compatibility decision is required before
shipping a new representation; parent owns future policy73/cache76 and schema.
This review changes no code, schema or version.

Required binding covers the full semantic carrier/get/primitive MIR, actual
checked Function (body, signature, contract, attributes and calls), checked
expansion maps, model/trait extension/spec dependency closure, loaded core and
vstd bytes, toolchain/rustc-driver/executing verifier bytes, target/features and
relevant MIR/panic/overflow/compiler settings. Whole-body fingerprints invalidate
stale evidence; they do not prove correspondence. Current approved-model
collection is not a guarantee this new nested dependency closure is complete:
ordinary foreign helpers are not inferred functions, and collection is filtered
[S10]. Explicitly collect/validate these get/Ord dependencies without adding laws.

Retain the nonempty required dependency set before private-helper pruning,
through re-export and every transitive consumer [S9]. Ordinary imports with
goals off must freshly resolve and inspect **each required kind and instance**
from their own TyCtxt, check whole-MIR/model/compiler bindings, and reject missing,
duplicate-conflicting, partial, stale or wrong-kind evidence before facts enter.
Post-pruning public helper count zero is not evidence of no dependencies. The
provider binds its actual proved Function; a hash of a reconstructed proposal or
a syntactically valid 64-digit field is insufficient to establish that proof.
Imports retain the normal trust in matching compiler-produced artifacts, not
metadata as an independent proof object.

Require successful fresh normal body query identities for every local helper
[S11], all ordinary/inference/auxiliary queries, no deferred compiler errors,
complete selected scope, and final erased Rust compilation before interface
publication [S12]. Failed or partial runs invalidate selected stale exports and
must not leave a usable selected library. `failed_functions=[]` is insufficient:
auxiliary/global or late erasure failures must still block outputs.

## 5. Adversarial acceptance controls for a future implementation

These are source/checker-design acceptance criteria, **not executed negatives**
or security/race experiments in this review.

| Control | Required classification and acceptance criterion |
|---|---|
| Wrong getter, generic args, formal/result map, self/other or field projection; repeated left operand | Structural identity/correspondence rejection before any helper proof is credited, even for equal-input examples. Correct aliasing of shared inputs remains accepted. |
| Added effect, cleanup, unreachable block, extra return/use, storage escape, read-after-dead or changed return place | Whole-body/CFG/use check rejects; no statement/edge may be omitted because a label/hash subset matches. |
| Signed cmp, u32/usize/u128, reversed operands, fabricated Ordering enum or swapped constructors | Exact types/semantic target/constructor binding rejects. No discriminant arithmetic route. |
| Unresolved generic/const/projection, shim/dyn target, generic wrapper or lookalike NonZero | Conservative unsupported rejection. The actual concretely normalized true-polymorphic-flag bodies can pass only the completeness checks above. |
| False, swapped or weak Ord relation / false obeys predicate | Correct source helper still proves its full numeric contract; incompatible authored relation/client fails semantically. A vacuous conditional guarantee does not count as numeric success. A deliberately weaker true property may verify as that property only, never earn the required exact numeric certificate. |
| Full NonZero domain | Symbolic proof has true requires and no added range premise. Ordinary observations include 1, equal values, 2^61, high bit, MAX-1 and MAX, both operand orders and aliased inputs. Samples support regression only; they do not establish the domain theorem. |
| Helper unhealthy versus semantic client negative | Semantic negative counts only if all required helper body and inference precision queries are independently proved and normal source checks pass. Missing/timed-out/failed helper or frontend failure is a separate rejected run. |
| Wrong kind, empty dependency list, incomplete maps/body hash, stale model/core/compiler/target or changed actual Function | Reject before import facts; require exact dependency closure. Test goals-off import, private pruning and multi-hop re-export; no reuse of worker/parent VIR under a changed runtime. |
| Candidate/cache mismatch or poisoned candidate | Candidate cannot supply source identity or proof status. Fresh exact proofs on miss/hit; wrong candidate fails without publishing. |
| Body success but missing helper identity, auxiliary-only failure, partial selected scope, late erased Rust failure | No selected interface/library survives. Exercise global failure with `failed_functions=[]`; do not accept a successful body count as publication permission. |

## 6. Trust inventory, findings and limits

| Layer | Status |
|---|---|
| NonZero opaque View/get and primitive u64 OrdSpec/OrdSpecImpl/declaration | Inherited vstd trust, reused exactly [S2–S4]; no new axiom/assume-spec/external-body/view equivalence proposed |
| Carrier and original generated wrapper proofs | Future ordinary proofs using those leaves; currently absent, cannot be replaced by diagnostic scripts or authored mirrors |
| Inverse-inlining and MIR-to-actual-Function correspondence | New bounded frontend/checker code in the existing compiler TCB, requiring audit and controls; **not** a kernel theorem and not categorically identical to adding a std axiom |
| Rust source to instantiated optimized MIR, Copy/read/borrow/value handling, erasure, solver | Existing trusted compiler/logical pipeline plus audited extensions; no native provenance/borrow-frame adequacy claim |
| Seal/parent receipts/this review | Source and diagnostic identity evidence only; zero new proof certificates or semantic acceptances |

Material corrections to the inherited proposal: no independent transmute/view
law is needed **after** checked operational factoring, and no new primitive Cmp
semantics/proof is needed for this actual modeled nested call. Neither correction
licenses skipping the source checker or trusting metadata. Baseline stays direct
PartialOrd 5/0, direct Ord 0/1, original comparison clients 2 verified/6 failing;
Eq BorTag/CounterKey each 6/0 cold and warm. Generated ordering excluded at the
profile gate remains unverified [D2]. No broad inherited suites rerun.

Native=0; full HMAC1/1 UNPROVED; 44+3/101 remain OPEN. No Rust-input, native,
provenance, borrow-frame or whole-goal completion result follows. All commits
local; no push/PR commands or publication permission request.

Process deviations: the sandbox wrapper failed before both initial read commands
(`bwrap` loopback permission); subsequent commands used approved escalation.
One attempted read placed the named parent authentication script inside its
receipt directory; it was absent. The exact script in the named parent directory
was then read, never executed (it hardcodes the live parent root). No parent or
sibling source/build/cache/state was accessed. Historical paths were labels only;
inherited source checks used own bytes and `git show 5d79f711:<path>`. No production
implementation, builds, broad tests, network, subagents or version changes.
