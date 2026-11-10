# Source-to-current/future boundary

This is a missing-proof inventory, not a newly defined source semantics. Nothing
in this directory interprets Rust entry validity by model ownership or by future
execution success. The independent checked statements use the existing model
semantics; their application to Rust remains conditional.

## Three distinct reference values

Let P be a possibly live caller parent with arbitrary legal ancestor history;
B0 is the owned incoming `_1` value of this invocation; A is its internal `_4`
reborrow. P need not be an allocation root. Each may be a strict interior view.
A static loan ID for `_4` cannot replace B0 or identify P.

The exact frontend equations are:

```
entry:             C(B0) = initial bytes           [source binding UNPROVED]
borrow phase 1:    C(A) = C(B0)
borrow phase 2:    B1 = update_current(B0,F(A))
update laws:       C(B1)=F(A), F(B1)=F(B0)
owned B1 resolved: C(B1)=F(B1), hence F(B0)=F(A)
unsize:            C(S)@=C(A)@, F(S)@=F(A)@       [inherited external_body]
constructor:       pending[i].C=C(S)[i], pending[i].F=F(S)[i]
                                                  [inherited assumed summary]
```

These equations come from `borrow_mut_to_sst` phases, `sst_to_air`'s
MutRefCurrent field update, the AIR prelude's two update equations, and typed
MutRef resolution. They have no physical allocation, dynamic tag, initialized
byte-state or original runtime operand interpretation. In particular the
frontend phase-2 assignment is a logical future substitution, not a proof that
the runtime array has already changed. `frontend.v` models exactly this algebra
with pairs; it is not a source representation theorem.

For normal exit with no escaped internal loan, the needed cut is:

```
Aexit.C = actual final whole subobject bytes     [physical/current binding]
Aexit.F = A.F                                   [same pending future]
Aexit.F = Aexit.C                               [typed pending release]
therefore F(B0) = actual final bytes.
```

For drop, source erasure must prove actual final bytes equal initial arbitrary
bytes. For HMAC, source simulation and ordinary25 must prove the exact final
vector, including unissued suffix, through the same physical projection. A
separate release proof for every necessary item/whole slice is required;
`has_resolved(opaque IterMut)` provides none of those releases. Array/slice
observation equality must preserve the full vector, not just visited positions.

If B0 is a child of live P, release B0 and unblock P, obtaining
`C(Pnew)=F(B0)` and `F(Pnew)=F(Pold)`. Do NOT require `F(Pold)=C(Pnew)`.
The checked MODEL `shorter_run_resources` performs exactly that recovery. If
B0's own internal child A is shorter, compose two such cuts. Lifetime end is a
resource operation and not synonymous with runtime empty drop glue or frontend
owned-value resolution.

## Existing mechanized normal-return semantics inspected

In the immutable VerusBelt foundation:

* `typing/programs.v:300` (`type_endlft`) resolves eligible typed resources,
  ends only its local lifetime, and unblocks resources using checked guards and
  receipt-indexed steps. This is not permission to end P's caller lifetime.
* `typing/programs.v:340` (`type_resolve_instr`) consumes a particular typed
  reference and derives its resolution observation; `type_resolve` composes it
  with a continuation. `source_loan_release` is a checked specialization of
  `uniq_resolve`, not a source-origin theorem despite its historical name.
* `typing/cont.v:21` (`type_jump`) extracts the continuation arguments, resolves
  the remaining typed context, and passes the original local/invariant context
  to the continuation. Returned arguments are not arbitrarily resolved.
* `typing/function.v:351,513` (`type_call_iris'`, `type_fnrec`) use the model
  function/continuation interpretations. They do not translate actual MIR.
* `typing/borrow.v:169` retains blocked parent observations
  `(F(child),F(parent))` while preserving location/current at reborrow. Actual
  reservation/activation/retag soundness is not a premise of this MODEL rule.

`resources.v` reuses these rules. Its heap observation opens the unique body
through the same ordinary lifetime context, checks agreement with the actual
MODEL heap authority, then recloses the same body and restores the context and
reference. It does not resolve a live parent to observe its current bytes. The
single later credit is an internal model resource, not a client application
premise or proof of a bounded Rust execution budget.

## Exact remaining dependency / how a proof object can bind the body

The constructor-erasure worker's proof must be indexed by the actual typed
function syntax and original operand evaluation judgment. A future compiler
integration must check that this syntax is the body being lowered and transport
the operand/loan interpretation along its derivation. The per-body theorem must
cover every independently lawful entry and every run including faults, prove
progress/validity preservation, and cover pre-erasure/monomorphized/frontend
phase correspondence. Its normal-exit case must yield the physical/current and
same-future bindings above plus a typed release at that cut. A checked theorem
about a newly chosen model program cannot provide that syntax-indexed proof.
Hashes can authenticate theorem inputs and compilation artifacts; they cannot
replace this derivation or prove the source effect.

For the pinned native-drop input the static chain is exactly `_1` dereference
-> `_4` exclusive two-phase borrow at bb0[3] -> activation/unsize `_3` at bb0[4]
-> `iter_mut` result `_2`. The pre-erasure bb1 FakeRead and bb2 cleanup/resume
remain obligations, as do storage and retag semantics. The monomorphized graph's
three empty glue queries are not three runtime destructor calls. Static
borrowck completion supplies no dynamic original-loan or observation proof.
The normal-exit adapter does not handle exceptional exit: the source dependency
must prove cleanup simulation or checked exclusion; it cannot remove fault or
unwind executions by restricting the input domain.

The existing source_entry and reference_adequacy investigations exhibit no
formal runtime/history judgment with these bindings. Consequently this lane
cannot close native ExitObservation_f unconditionally. It delivers checked
algebra/resource components and the exact UNPROVED source interface. No new
opaque observation relation defined as the desired conclusion is introduced.

## HMAC requirements retained

The extension keeps 128 arbitrary initialized bytes, a shared slice of every
valid length, dangling aligned empty keys (including q=p), and same-allocation
nonoverlapping interiors. The source correspondence must cover actual Zip.index,
fixed inner pointers, both indexed getters, ordinary XOR store, None, distance
intrinsic and cleanup. It must preserve the whole pending array observation,
not infer an unissued suffix frame from resolved owned iterator. Preserve
immutable key aliases and lawful ancestors without introducing whole-allocation
freeability or forbidding unrelated concurrent caller writes. Only this
invocation's footprint is framed. Canonical all-u64/arbitrary[10] and full HMAC
arbitrary[128]/all-valid-key prefix plus suffix still require the same checked
automation; no such shared assistance is established here.

## Completed ordinary caller addendum

The parent supplied commit `a8df10a0` and exact completed receipt
`/tmp/verus-infer-buffer-cycle-20261109/parent-caller-ordinary-003/proofs/receipt.json`
(SHA256 `1b9063d276c3ef505562143614531a0191230ee91bf3ca07d5b2c63d2af82f4c`).
Only that commit's five named files and the named completed artifacts were read;
no live parent/ordinary-caller checkout or unfinished output was used.
`ModelCallerOrdinaryWriter.caller.caller_xor_na_verified` provides the required
ordinary-access MODEL computation with caller future/frame preservation.
Its symbolic credits and aliveness hypotheses remain model preconditions;
independent acceptance is pending. It removes the former ordinary-access support
gap for a future HMAC composition, not the source entry/exit binding. It is not
imported by the bounded drop adapter, which needs no XOR computation. The complete
ordinary25 observed model result and accepted caller9 remain separately scoped.

## Delivered typed source interface (3d668d29)

The later parent-delivered `NativeConstructorErasure.Interface` supplies the
uninstantiated `source_semantics`, `source_heap`, tagged `reference`,
`ref_address`, independent `V_entry` and `execution` with Continue/Normal/Unwind/
Fault outcomes. This replaces generic notation for the dependency's concrete
types; it supplies NO actual source instance or certificate. Its
`Composition.entry_normal_bytes` is conditional on a source_entry_certificate.
Only these two completed modules are imported by `candidates/source.v`.

`initialized_bytes_unique` proves whole-vector agreement at the SAME physical
subobject from actual lookup uniqueness and equal length. The final-vector
adapter combines physical interpretations of pending CURRENT and final bytes
with the existing merge/same-future/typed release equations. The drop theorem
uses entry_normal_bytes on the same original operand from V_entry and retains
full-certificate fault exclusion. The original entry-C/F and pending-exit
bindings remain explicit UNPROVED source obligations; they are not obtained
from the record signature, source selection, hashes or certificate flags.
No normal-run-only safety restriction is introduced. Root storage-prefix work
and the still-unproved root reborrow cannot be bypassed by this composition.
Pointer-distance component2c0b8f23 is not needed for drop and is not imported;
its lawful-source-to-span/intrinsic interpretation remains future HMAC work.
