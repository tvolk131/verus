# Corrected DESIGN decision memo: redb native resource gate

November 7, 2026. **PROPOSED / NOT ADOPTED / NO IMPLEMENTATION AUTHORITY.**
This is a bounded amendment to proposal043 addressing review047, not another
full review. It establishes neither a Rust-core implementation theorem nor a
native resource theorem. **All18 final redb obligations remain OPEN.**

## Evidence and decision requested

Read-only inputs (their historical audits/results are reported evidence, not
newly authenticated results of this memo):

- `/private/tmp/verus-native-slice-primitive-proposal043-0ywdS3/PROPOSAL.md`.
- `/private/tmp/verus-native-proposal043-semantic047-YqMgfR0y/REPORT.md`,
  especially F1-F6 and the selected-theorem policy critique.
- `/private/tmp/verus-native-proposal043-semantic047-YqMgfR0y/HANDOFF.md`.
- `/private/tmp/verus-native-slice-testprep042-I01FH1HU/REPORT.md`.

Recommendation: retain the existing fail-closed admission policy now; use the
outline below only as a candidate for an explicit external-law decision.
Trust adoption and any scoped-admission policy require separate decisions;
neither decision, nor later implementation, is authorized by this memo.

## 1. Common domain and inventory/rely invariant

Use one immutable resource domain D and evolving heaps H0, H1, H2 within D.
A live control block C identifies the actual data region A, its lens, extent n,
allocation generation and initialized bytes. Distinct allocation identities
remain distinct despite equal bytes or reused/equal addresses. Typed places
include instantiated body, dynamic activation, local/projections and place
initialization generation. Moves transport identities, not cloned ownership.

At stable boundaries, S and W are the exact strong and explicit-Weak ownership
contributions for C, each with a unique identity and accounted location/token.
Counts agree with these complete inventories, not merely accessible locals.
The internal implicit weak obligation is separate: W={} never means a zero
total internal weak counter. The minimal supported history excludes raw,
forgotten/leaked and other untracked contributions, FFI alias creation and
unmodeled ownership operations. Excluding those histories is an owned,
caller-discharged closed-world premise, not a consequence of an Arc type.

Inventory accounting authority is separate from byte access. Extra handles
can be accounted for without possessing exclusive bytes. Allocation permissions,
compatible shared loans, exclusive loans and suspended ancestors must all be
represented independently; a unique inventory does not create missing authority.
The success domain requires authority that can legitimately be lent exclusively
and no incompatible live loan. A failure domain need not supply exclusive bytes.

The caller owns/threads a rely excluding unauthorized concurrent or reentrant
clone, downgrade, upgrade, retirement, raw access and incompatible reads/writes
through each affected loan's lifetime. Authorized sequential A3 transitions are
allowed and update S/W before the next A1. Entry assertions are conditional
theorem premises; callers must discharge them. No crate-wide invariant or page
construction theorem is inferred.

## 2. Small revised external-law outline — proposed assumptions only

Every admitted schema must specify its typed instance/profile, domain, exact
resource inputs/outputs, and outcomes. In a supported success domain, the laws
below require **normal termination**, not just a postcondition if execution
returns. Relevant failure domains have a separately bound terminal Abort
outcome under the selected panic=abort profile, with no normal successor.
Abort supplies no normal heap/frame, destructor-completion or recovery theorem.
Unwind and unmodeled behavior refuse admission; refusal is not proof of
unreachability. If the logic cannot establish the required normal-return claim,
report that limitation and leave the gate open, rather than accept divergence
or an always-aborting model. Arithmetic preceding a primitive remains checked
caller behavior, not hidden inside its law.

- **A1 — exact Arc get_mut.** With exclusive access to the actual handle place h,
  live C/A, complete S/W, the rely and branch-appropriate permissions above,
  normally return Some iff S={h} and W={}; otherwise normally return None for
  the supported extra-strong/explicit-Weak cases. Some lends exactly A's
  [0,n) mutable lens and suspends, never duplicates, parent authority. None
  returns the legitimate incoming resources without a new write lens. Both
  conserve h, C/A identity, generation, extent, bytes and stable inventories;
  no allocation or reinitialization occurs. Incomplete premises refuse
  admission; they must not be converted into a None execution result.
- **A2 — exact Arc deref.** A valid shared borrow of h normally returns the
  same A/extent/generation read lens, with compatible permission suspension.
  The shared loan survives return until its checked end. Value-level deref
  elision is not a substitute for this resource effect.
- **O1 — Option<&mut [u8]>::unwrap.** Some normally returns the exact contained
  lens/loan, consuming its payload once; no fresh owner or independent mutable
  token. None reaches the bound terminal abort path, never a normal success.
  Some/discriminant and normal-success obligations cannot be discharged by
  vacuous normal-return ensures clauses.
- **S1 — len, range index, reference/array unsizing.** Valid len/unsizing normally
  preserve origin and return the actual lens extent; coercion allocates nothing.
  Valid indexing requires 0<=s<=e<=length and checked absolute base+s/base+e
  composition within the live underlying allocation. It normally lends exactly
  that child, suspending mutable ancestors as appropriate; shared indexing lends
  reads only. Reversed/out-of-bounds ranges select the bound panic/abort path.
  [k,k), including k=length, keeps origin, generation and reference obligations;
  empty footprint implies neither fresh ownership nor address inequality.
- **S2 — copy_from_slice::<u8>.** Valid exclusive destination, initialized source
  read authority, equal lengths, valid bounds and compatible nonoverlap imply
  normal unit return with H'[A][destination]=source's pre-call bytes and no other
  byte update. Zero-length copy is an empty heap update, not an exemption from
  reference validity/permission compatibility; nonoverlap does not require
  unequal machine addresses. Unequal lengths select bound panic/abort. Preserve
  underlying source authority and identity/frame resources, while temporary call
  borrows are transported/ended separately, not declared identical to all loans.
- **S3 — observation and assertion.** Valid u8 slice/reference equality normally
  returns precisely length/byte equality through live read resources, not owner
  equality. Include the actual shared/mutable-reference wrappers. True assertions
  normally continue; false assertions use the bound terminal panic closure.
  Formatting/cleanup is covered or proved unreachable, never ignored by name.
- **A3 — separately reviewed extra-handle extension.** Exact clone/downgrade
  normally create one fresh strong/explicit-Weak identity on C within checked
  counter bounds. Exact nonfinal retirement consumes only that extra identity
  once; original h stays live. The bounded histories are strong 1->2->1 and
  explicit Weak 0->1->0, retaining the implicit weak obligation. Bind the actual
  instances, mem::drop wrapper, Arc/Weak destructor, drop flags and recursive
  glue, or encapsulate that exact reviewed closure in a canonical primitive
  schema. Counter-overflow/abort conditions must be excluded by checked bounds
  or covered explicitly. A generic rule for the spelling drop is inadmissible.
  No final-h release, payload destruction, deallocation or WritablePage Drop.

These are external TCB candidates, not checked core/alloc bodies. Review047
reports absent matching library implementation sources and null contract
schemas. Captured names, hashes, Debug substitutions and the older 60 call rows
cannot adopt these laws or bind prospective A3 instances. Admission needs typed
resolved instances/substitutions and owner environment, exact library/build and
schema versions, target/ABI/cfg/debug/panic/compiler/runtime bindings; borrow
correspondence must retain lifetimes. Any missing/mismatched binding refuses.

## 3. Checked engineering obligations — never additional primitive axioms

| Item | Required implementation evidence, still OPEN |
|---|---|
| E1 | Carry an affine resource channel through VIR/SST/AIR and all transformations; enforce stable D, fresh allocation/control-block/handle/activation identities, typed overlap, generations, conservation and frames. No finite-checker oracle or fresh-world reset after writes. |
| E2 | Check original body/CFG/profile to GHOST/EXEC/erased-call occurrence correspondence, evaluated places/projections, argument/result transport and every successor. Evaluate expressions once and preserve reservation/activation phases and ordinary Rust borrow checking. |
| E3 | Check native mem_mut, mem and helper/caller bodies before exporting effects. Transport the exact get_mut->unwrap->mem_mut->caller loan subtree through returns. A second activation is fresh in D, starts at H1, and preserves the first update. No trusted application summary. |
| E4 | Consume activation-specific affine end witnesses only at actual original-NLL/CFG ends with all conflicting descendants closed on every relevant successor. Reject stale/consumed, foreign-domain, wrong-generation/owner/body/place/interval witnesses atomically. Permit genuine cross-body transport via checked mapping. HasResolved/value resolution is not retirement; returned mem read loans remain live. |
| E5 | Check joins, partial initialization, normal/None/abort paths, cleanup/drop flags/glue, arithmetic and reachable dependencies. A panic=abort flag cannot erase captured cleanup edges. Unknown reachable instances, indirect targets or unsupported effects fail closed. |
| E6 | Enforce the decided dependency-closed admission policy and evidence labels. Each dependency is a checked body or a separately reviewed bound primitive, never merely captured. Preserve default-OFF, ordinary Infer full-scope guards and certified import/export barriers. |

The frame is permission-indexed by actual allocation/interval: A outside the
write, distinct B, both sources, h/C, extent/generation, page.buffer, offset and
other opaque resources. This gate requires each source disjoint from the whole
target A, not merely its written interval. For P_TWICE instantiate read, liveness,
disjointness and frame premises for first AND second; compatible shared sources
may alias one another. For owner pairs, bind the actual right/B target and left/A
frame, not a textual role swap. Same-allocation split-copy is a separate extension.
Prove nonwrapping start+8 and absolute offsets in each debug profile; debug-on
overflow and debug-off wrapping are not interchangeable failure semantics.

Retain all17 prospective expectations and matched controls in both debug profiles.
Each resource case inherits the full relevant BASE closure (checked native/helper/
caller bodies, A1/A2/O1/S1/S2, S3 where used, E1-E6); A3 is additional, not a
replacement. Short per-case requires lists are not closure. After exact extra
retirement the positive must normally succeed; with an extra strong/Weak still
live, requested normal success must fail or checked None->abort must be shown,
with no later drop/read treated as a normal successor. Owner/frame and loan-end
negatives must fail their intended resource claims, not unrelated frontend checks.
The separate borrow-conflict control gives borrow-check credit only. Completion
needs actual nonempty resource obligations and checked results in native methods
and selected callers/helpers, not capture counts or VC generation alone. This
memo supplies no result and does not erase later reported borrow/typecheck evidence.

## 4. Independent decision options

**Trust choice:**

- **T0 — defer:** adopt no external laws; retain the present fail-closed gate.
- **T1 — consider the baseline only:** commission review of A1/A2/O1/S1-S3 and
  their total/outcome schemas. A3 cases remain unsupported and uncredited;
  this can only be a substep, never completion of the full gate.
- **T2 — consider full alias coverage:** additionally commission the exact A3
  and glue boundary. All four strong/Weak cases remain required. Commissioning
  review is not accepting any TCB axiom or authorizing implementation.

**Scope choice, independent of T0/T1/T2:**

- **S0 — retain whole-native admission (recommended now).** No selected result
  is admitted/exported while the existing native/full-scope closure is unresolved.
- **S1 — separately review a selected-theorem policy.** Require explicit typed
  roots and requested outcomes; a least-fixed-point graph covering calls,
  coercions, arithmetic/assertions, closures and reachable cleanup/glue; checked
  reachability for omitted successors; and per-edge primitive/body status and
  invalidation on binding changes. Unsupported bodies stay visibly UNVERIFIED.
  Use a distinct scoped claim namespace with no ordinary Infer certificate or
  whole-crate/publication-ready label. Until a certificate/consumer policy is
  separately reviewed, selected evidence remains nonexportable. No name whitelist
  or bypass of existing full-scope/import/export guards implements this choice.

## 5. Unresolved premises and stop

Pinned-library outcome/termination correspondence, canonical schemas/bindings,
closed-history/rely and byte/frame premise discharge, checked E1-E6, full-matrix
results, and trust/scope adoption are all unresolved. Page construction/Drop,
cache/locks/storage/publication/commit/recovery remain outside this conditional
gate. Historical provenance/preservation qualifications are unchanged.

Only this new memo was written in its fresh private directory. Parent source,
existing artifacts, STATUS and trust were not changed. No compiler/vstd/Cargo/
harness builds, verifier/solver runs, native executions, network, Git operations,
old-task retries or subdelegation occurred. The separate zero-growth050 diagnostic
was neither inspected nor touched/resumed; no previously refused task was repeated.

**STOP: decision-ready design amendment only. Native resource proof credit 0;
new trust 0. All18 final redb obligations and trust/scope adoption remain OPEN.**
