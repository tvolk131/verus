# Independent review of the conservative source-local reborrow MODEL

Review scope: mathematical delivery `aaca63b9`, early interface `e9615ccb`,
parent replay fix `312a5554`, integrated base `6c74c9b1`. Early committed scope
and receipt/product freeze: `ceb9bce2`; review-controller freeze: `0a0834ce`.
**Mathematical decision: accept at the exact conditional MODEL scope. Controller
decision: accept the current integration001 replay; no dependency authentication
weakening found.** No blocking defect was found within that scope. Native-source
acceptance is refused: its interpretation and coverage remain unproved. Only this
new evidence directory changed. This is not combined whole-goal acceptance.

## Independent validation and retained failures

Final guarded review003 succeeded with controller freeze `261d2258`. Fresh normal
coqc compiled Model/Laws/Span/Controls plus the inspection module in private
review002 output. That successful completed compilation was authenticated and
reused for the final audit, which rechecked every input and product hash and ran
another full recursive coqchk. No unsafe exclusions, admissions, new axioms,
unchecked computation shortcuts or unresolved Program obligations were used.

Compiler-discovered inventory is exactly **134**: **109 constants, 7 inductive/
record blocks, 18 constructors**. Fixpoints, definitions, theorem constants,
generated eliminators, record projections and constructors are all inventoried.
The independent module-output parser agrees with the frozen source inventory.
Every constant has an exact verbose kernel trace entry; types/constructors are
checked as part of complete module inductive-block traversal. No declaration is
excluded merely because it lacks a separate trace line. Each of all134 individual
Print Assumptions outputs is closed. Recursive context summary reports no axioms,
type-in-type, unsafe (co)fixpoints, assumed positivity or rewrite rules.

Coq8.20 does not print separate inductive-check lines: its
[complete module-field traversal](https://raw.githubusercontent.com/coq/coq/V8.20.0/checker/mod_checking.ml)
checks inductive blocks through
[CheckInductive](https://raw.githubusercontent.com/coq/coq/V8.20.0/checker/checkInductive.ml).
Pinned snapshots in upstream-pins.json explain that coverage accounting; they
are inspection sources, not a claim to have reproduced the checker binary.
The existing pinned Coq8.20.0 binary remains the inherited checker trust boundary.
The inspection module contains only import/Redirect Print/Check commands;
there are no marker definitions, lemmas or other hidden declarations.

Original, parent and fresh runs have identical V/VO identities for all four math
modules, all six copied dependency modules and the inspection module. Full
normal coqc and recursive coqchk passed. Pure parser/source-shape/static-loan/
inventory/kernel-summary controls pass **3/3 under normal Python -B and -B -O**;
Coq Controls are checked mathematical cases, not runtime Rust experiments.

The review retained two controller failures separately:

- review001: compilation and both kernel runs passed, but the reviewer incorrectly
  required nonexistent inductive trace lines. Freeze `0a0834ce` and failed receipt
  remain pinned. No model readiness acceptance was emitted.
- review002: fresh compilation and both kernel runs, exact coverage and identities
  passed; the reviewer then appended stdout's version banner after stderr's trust
  summary and rejected that combined text. Freeze `51d3ddef` and failed receipt
  remain pinned. Final003 parses the summary stream independently.

Both are reviewer setup/parser failures, not semantic controls or proof failures.
Inherited development records separately retain failed trials001–010/012–015
and successful preliminary011/final016; those failed trials were not rerun here.
All final source/tool/dependency/product guards passed before review003 success.

[results.json](results.json), [commands.json](commands.json),
[hashes.json](hashes.json), [inventory.json](inventory.json) and
[development.json](development.json) give exact identities and commands.
Final review receipt SHA256:
`0bdbc838347582eadf04aa26dae17b6d0387e5e45b64825bb29a9547b4b98cf8`.
Fresh review002 compilation receipt SHA256:
`bccde9cd3729bad7620ce9be1512a846e295d9668bcdbf5f3f8b45bfecbf7e53`.
Raw receipts/logs stay in precisely named private
`/tmp/source-reborrow-model-review-6c74c9b1-{001,002,003}`; only compact evidence
is committed. Shared tools/products were read only. Required baseline-ready AND
successful durable-build-2, all523 baseline sources/284 objects, all1121 Coq
stdlib/source/object/plugin pins, exact runner/coqc/coqchk/opam/loadpaths and GMP
manifest `332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`
plus archives were authenticated and guarded. Fresh recursive checking covers the
actual imported closure; no new full foundation build is claimed.

## Mathematical reading and exact premises

`Model.execute` handles precisely root instance0/bb0[3]. It checks current
control, an incoming local1 immediate reference equal to the full original
operand record (allocation, signed offset, dynamic tag), and live uninitialized
local4. It does not substitute static borrowck loan0 for the incoming tag.
Positive generic n is required. `span_ok` finds a live region with matching
allocation, nonnegative base/offset, contained extent, strictly positive
physical start, and region/span end at most 2^64. These are authored structural
bounds, not a Rust allocation, target-layout or isize adequacy theorem.

`byte_ready` independently describes a current stack split into unprotected
non-parent younger grants and an existing parent/ancestor suffix. The suffix
requires matching node ranges, parent links ending at a root, and current
Reserved/Active permissions. `ready` adds current local/control/range and
freshness predicates and quantifies `byte_ready` across the span. Neither
predicate mentions successful execution, the desired result, future-definedness,
ty_own or an actual-source certificate. `model_progress` derives success rather
than taking successful grants as an input. `prepare` computes all affected grant
stacks before the pure `commit_reborrow` constructs the next state.

The commit creates a Reserved unprotected child, its node with original parent
tag and exact extent, one allocation event, one retag event per byte, and a
local4 assignment event. It increments next_tag, emits LocalWriteHook4, assigns
the child reference preserving allocation/offset, and advances to bb0[4].
`all_outcomes_at_ready` quantifies every result of **this deterministic MODEL**;
`all_advanced_shape` extracts prerequisites for any successful result without
requiring ready. Neither quantifies actual Rust outcomes.

Memory/frame laws preserve every caller cell and its projected initialized-byte
map, original operand, private stack/fresh-stack, prior opaque H, regions and
all locals except4. Old node lookup is preserved for tags other than the fresh
child. The grant frame preserves every address outside the selected span.
Per-byte laws preserve the exact parent/ancestor suffix and every protected
entry, while discarding only unprotected younger entries. Child validity is
proved in the extended node list. These are substantive equations about the
executable model, not only hashes, metadata or an arithmetic carrier.

Freshness has explicit limits. Execution checks node tags, explicit retag trace
and each affected byte stack. `registered h` separately requires **all** byte
grant tags (including outside the span) to have nodes. `fresh_global` uses it
with `fresh_node` to rule out an outside tag collision. Neither ready nor
`arbitrary_interior_context` assumes registered. Thus success/ready alone does
not prove global tag freshness for arbitrary outside maps; the theorem never
claims that stronger conclusion. Opaque prior H is preserved, not interpreted
or searched for hidden tags. No global history well-formedness, uniqueness,
source permission interpretation or registry preservation theorem is inferred.
A future source relation must establish any such invariants it needs.

`Span` composes the three checked StorageLive steps with this operation, then
preserves arbitrary initialized lists at generic positive n, including10 and128.
The list cells are modeled with Z values; preservation applies in particular to
valid byte values but is not a byte-representation correspondence theorem.
The selected actual graph site has array type128, so ten_byte_model is a generic
MODEL corollary, not canonical-source coverage. There is no key operand and no
key-length restriction in this primitive; it supplies no HMAC writer theorem.

`Controls.arbitrary_interior_context` has allocation7/base4096/interior offset17,
parent11 under protected Active ancestor5, child42, generic positive n bounded
by 2^64-4160, arbitrary caller/private maps and prior H, arbitrary explicit
trace subject to fresh_trace42, and the required per-byte grant stack. It allows
arbitrary outside grant data, subject to the global-freshness qualification
above. It is a witness family, not a theorem that all lawful source histories
fit this shape. Protected younger, Disabled parent/ancestor, absent parent,
reused child and null-address rejections are MODEL controls. Frozen suffixes
also fail by mutable_permission's definition; no Rust rejection is concluded.

## Source and controller boundary

This authored conservative tagged byte-stack operation is **not Rust, Stacked
Borrows or Tree Borrows semantics**. There is no activation or write transition.
Borrowck two-phase activation is not identified with a Reserved-to-Active model
step. Pinned compiler step.rs constructs a reference and runs machine-specific
retag/validation before assignment; its default no-op hook is not a safety or
frame theorem. Pinned assignment/reference documentation leaves alias semantics
underdetermined. The inspected SB/TB rules provide guidance, not a formally
imported semantics or source safety certificate. Exact source/static-loan identity
does not interpret a runtime pointer or justify access.

[history-audit.json](history-audit.json) records immutable `git show` checks.
The four math sources, pins.json and final-freeze.json equal the mathematical
delivery, and INTERFACE equals the early commit. Comparing freezes removes only
three unused dependency integration-results JSON documents, changes only the
controller digest among retained entries, and preserves inventory/goal pins.
The actual controller change is that single input-list edit. Each completed
receipt remains loaded from pins.json, hash-authenticated, success-checked,
included in the initial/final guard, and compared against its loaded bytes.
Every named completed product is authenticated; exact current parent/storage
V sources are compared with the delivered sources before copying their V/VO.
No dependency authentication was removed with the documentary inputs.

The historical worker report/results correctly describe its old controller and
freeze; they are historical identities, not the current replay recipe. The
current README/integration001-freeze is used for fresh replay. No historical
checkout path was followed. Only this checkout, exact named completed products
and the existing pinned toolchain/source snapshots were read.

The review controller guards the whole initial TARGET snapshot and independently
checks exact requirements pin
`11014c3ac73ddf13c464769f5d0d1cbd24598ab52777718d31872c70b3b8f7b0`.
It additionally authenticates the original worker receipt
`1d135594952edc6fcfc89c50cecc8c6d7dbdc5477541aebd887a1fecf4b8f414`
and parent receipt
`2ba00adce2ef5ac24c4d681574616157b006fd6d9067b45131865cf52fd7c74d`
through current integration-results and retains all their exact V/VO/output pins.
No production checking flags or policy files are edited.

## Next native sequent and unchanged full goal

`NativeConstructorErasure.Interface.root_reborrow_need M_Rust` remains unproved;
there is no actual M_Rust instance. For independently lawful actual entry S0
with the original reference r and arbitrary initialized A, every actual prefix
reaching bb0[3] still needs a relation R to an appropriate model state and:

```
exists s, R(S,s) /\ ready s (eval_S _1) 128;
exists S' child, RustStep(S,bb0[3],Continue S') /\
  eval_S' _4 = child /\ same_actual_subobject(r,child) /\ RustRefValid(S',child);
forall o, RustStep(S,bb0[3],o) ->
  Continue S' => exists t, Model.step(s,selected_site,128,Advanced t) /\
                         R(S',t) /\ caller_bytes(S') = caller_bytes(S)
  Fault _    => False
  other      => justified source-specific treatment.
```

This is an obligation specification, not supplied Coq code or an axiom. R must
cover actual physical addresses, allocation/subobject bounds, all lawful incoming
ancestor/protector histories, local representation, retag/validation/assignment,
and every enabled source outcome. If a lawful history is excluded by this
conservative rule, expand/justify the model or use another proved transition;
do not narrow lawful entry to ready or already-successful execution. Source
safety, defined executions, no_unwind, permissions or correspondence are not
assumed here. Raw/default retag, activation, unsize hooks, cleanup/exception
phases, original-reference C/F and the remaining writer/source steps stay open.

Original every-u64/arbitrary initialized exclusive[10] canonical6/0 and original
independently frozen arbitrary initialized[128]/every valid key length (including
0 and >128) HMAC XOR prefix **AND unchanged suffix** 1/1 unproved are inherited,
not rerun. The same actual checked assistance, normal fidelity/trust/inference/
publication gates and fresh combined independent acceptance are unchanged.
Native0; all44 minimal+3 phase/101 writer effects open; publication=false;
whole goal incomplete. No engine/vstd/global changes, Rust rebuild/full suite,
installs, shared-environment mutation, credentials, subdelegation, push/PR,
production fact publication or filesystem experiments were performed.
