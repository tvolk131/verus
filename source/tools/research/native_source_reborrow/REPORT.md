# Concrete reborrow MODEL; actual source interpretation remains open

Implements the minimal constructor root bb0[3], `_4 = &mut (*_1)`, over
`NativeSourceStorage.Storage.state (Model.history H)`. This is an executable,
conservative, tagged byte-stack MODEL with checked primitive algebra. **It is
not a Rust/Stacked Borrows/Tree Borrows semantics instance, source safety theorem,
or native certificate.** Native correspondences0; all44 minimal operations+3
phases/101 writer effects remain open. Publication=false, goal incomplete.
Canonical6/0 and exact HMAC1/1 unproved are inherited, not rerun. Domains,
fixtures, executable bodies, normal checking, trust and combined independent
acceptance criteria are unchanged.

Base `bc2a0b37`; early typed-interface/frozen-minimal-trial commit `e9615ccb`.
Only the NEW native_source_reborrow directory is changed. No subdelegation,
push/PR, Rust suite/rebuild, dependency install, shared-environment write,
formal-source import, source safety assumption or C/F bridge was used. Completed
artifacts named by the three integration-results records and inspection pins
were read; historical checkout paths in their source spans were treated as
labels. No parent/sibling file contents were read. An initial broad AGENTS
filename search was interrupted without output; subsequent searches were
restricted to this checkout and named completed artifacts.

## Source linkage and deliberate semantic choice

`pins.json` binds the existing rustc1.99.1 sources at
58cf8f9c35b355199727627d07ae34a4e2846d67 and the already completed research
snapshots. `INTERFACE.md` records the rule table. The controller authenticates
both the six-instance/44-operation graph and the actual borrowck extraction:
loan0 is assigned to local4, borrowed from dereferenced local1, reserved bb0[3],
activated bb0[4]. It is never the incoming dynamic provenance tag.

Pinned step.rs:94/214–239 evaluates Ref, evaluates/forces the referent place,
constructs a reference, selects `RetagMode::TwoPhase`, and writes through the
machine retag mode. If validation is disabled it explicitly calls
`retag_ptr_value`; otherwise the immediate write runs validation
(place.rs:687–706). Statement control advances on successful return
(step.rs:55–61). machine.rs:546–571 leaves retag behavior machine-specific.
syntax.rs:306–338/1350–1358 expressly leaves assignment/reference aliasing details
underdetermined. The default no-op machine hook is not a source frame theorem.

Inspected Stacked Borrows sources motivate per-byte grants and protectors but
use old Untagged raw-pointer rules, which are not imported or used. Inspected
Simuliris creates a fresh child and performs a read during retag; its Tree
Borrows Reserved-to-Active first-write behavior is distinct from borrowck
two-phase activation. Its optimization lifting assumes pool_safe and cannot
prove the missing source safety. No external semantics library was installed,
built or formally imported. These primary sources guide the explicit MODEL
choice, without supplying an actual source theorem.

The chosen byte-stack rule retains the actual parent/ancestor suffix, drops
only unprotected younger grants, and adds a fresh Reserved child. This is a
conservative authored rule; it is not claimed to be rustc's TwoPhase mode,
current Stacked Borrows, or Tree Borrows. There is **no activation or write
transition**. In particular, this rule's rejection of a represented state says
nothing about whether Rust would reject it.

## State and executable transition

The existing storage carrier is reused with an extended history, not replaced
by a semantics record. `history H` retains arbitrary prior H, arbitrary existing
region/node/grant maps, fresh identifier, and prior explicit retag history.
Regions record allocation identity, physical base, live flag and byte extent;
references retain signed interior offset and a separate dynamic tag. No ghost
MutRef unpack pointer, newly allocated caller heap, empty incoming history,
permission inferred from metadata, or future/current equation is used.

The executable `Model.execute` first checks exact current control and selected
site. It requires local1 to evaluate to the SAME original operand and local4
to be live uninitialized. Positive n, live allocation extent, nonnegative offset,
non-null physical address and a bounded64-bit physical extent are checked.
The address/region fields are MODEL data; their faithful relation to actual
runtime pointers remains unproved. Child freshness is checked against current
nodes, recorded allocation/retag/assignment history, and each accessed byte.
The independent `registered` predicate and `fresh_global` law explain when the
node registry accounts for all existing byte-grant tags, including outside the
selected span. They are structural MODEL premises, not new application requires
or coverage of every lawful actual Rust history.

For each byte, `valid_suffix` checks node existence, range membership, explicit
parent links through a root, and mutable current permission (Reserved/Active).
Protector state is an explicit per-grant flag. `seek_parent` may discard a
prefix only when every discarded entry is unprotected. It preserves the parent
and all represented ancestor grants, including live protected ancestors.
Disabled/Frozen retained ancestors are rejected; protected younger entries
block the transition. `no_tag` prevents child reuse within each affected stack.

`prepare` computes every byte grant before committing. A successful commit adds
the child node with its actual parent tag and exact extent, installs each byte's
new grant stack, increments the tag counter, appends ChildAllocated and one
ByteRetagged event per byte and LocalAssigned4, emits LocalWriteHook4, writes
`Immediate (Init (RefValue child))` to local4 and advances to bb0[4]. The child
retains the exact allocation and offset. Prior H, region map, all caller cells,
private storage and non-target locals are preserved. Existing node lookups are
preserved away from the fresh child. Outside-span grant stacks remain unchanged;
inside-span ancestor suffixes and every protected grant remain unchanged.

`ready` and `byte_ready` describe current structural range, lineage, permissions,
protection and freshness. Neither is defined by a successful transition, desired
postcondition, certificate, future defined run, or ty_own. Their mathematical
coverage is conditional and explicitly **does not cover all lawful Rust inputs**.
Input histories/regions/grants are not fabricated by the progress theorem.

## Checked theorem scope

- `model_progress` derives an executable successful step from those structural
  conditions, performing per-byte grants internally.
- `all_outcomes_at_ready` includes every enabled result, with faults excluded
  under the MODEL premises; `all_advanced_shape` independently characterizes
  every successful result without assuming ready.
- `all_advanced_memory`, `all_advanced_control_local`, `commit_nodes`,
  `commit_grant_frame`, `commit_trace`: byte heap/private storage/history-frame,
  local/control and registry/history laws for every successful result.
- `advanced_ancestor_and_protector`, `advanced_child_valid`: retained actual
  suffixes/protectors and structurally valid new child on every accessed byte.
- `four_step_model_prefix`: composes the three existing checked StorageLive
  transitions and this new MODEL operation. The old Prefix.execute itself still
  reports Unsupported at bb0[3]; this new module handles the selected next step.
- `generic_span_bytes`, `ten_byte_model`, `pad128_model`: arbitrary initialized
  byte lists and full caller-cell preservation. The10 case is a generic MODEL
  corollary, **not an actual canonical-source certificate**. No key-length
  restriction is introduced; this primitive has no key operand at all.
- `arbitrary_interior_context`: nonzero interior offset17 in allocation7, a
  parent under an existing protected ancestor, generic positive n within the
  modeled physical extent, arbitrary prior H/history satisfying freshness,
  arbitrary caller memory and arbitrary outside-span grant context.
- Mathematical controls: supported live protected ancestor; rejected protected
  younger grant, Disabled parent/ancestor, missing parent, child-tag reuse and
  null physical address. These are MODEL controls, not all-Rust safety claims.

Unsize, raw formation, getters, any other retag, activation, protector cleanup,
normal and unwind exits remain Unsupported here. Unsupported is an explicit
result with no continuing/normal state, never an identity transition. No
original-reference C/F resolution is inferred. Parent-delivered unsize carrier
work is outside this module's formal dependencies and cannot extend this
primitive to a whole cast/hook theorem.

## Remaining source sequent

Prominently false/open: actual_source_instance, all_lawful_rust_inputs_covered,
source_validity_interpretation_proved, source_certificate_proved,
source_cf_bindings_proved. MODEL progress does not establish any of them.

The exact existing typed source obligation `NativeConstructorErasure.Interface.root_reborrow_need M_Rust` remains unproved; no actual M_Rust instance exists here. The next missing interpretation/simulation sequent is, schematically, for an **independently** lawful
actual Rust entry S0 with the original r and arbitrary initialized A of length128,
and every actual prefix reaching S at bb0[3]:

```
exists s, R_actual_storage_history S s /\ ready s (eval_S _1) 128;
exists S' child, RustStep S bb0[3] (Continue S') /\
  eval_S' _4 = child /\ same_actual_subobject r child /\ RustRefValid S' child;
forall o, RustStep S bb0[3] o ->
  match o with
  | Continue S' => exists t, Model.step s selected_site 128 (Advanced t) /\
      R_actual_storage_history S' t /\ caller_bytes S' = caller_bytes S
  | Fault _ => False
  | Normal _ | Unwind _ => justified source-specific outcome handling
  end.
```

Neither R nor the actual Rust semantics/lawful entry interpretation is supplied
as an axiom or instantiated by redefining lawful as ready. The relation must
cover actual physical addresses, non-nullness, allocation/subobject extent,
provenance, arbitrary incoming ancestor/protector histories, local representation,
validation/retag/assignment effects and **all** outcomes, for every lawful input.
If the conservative rule excludes a lawful case, its coverage must be expanded
or a different justified transition supplied; application domains cannot shrink.
The complete source entry certificate for the other43 operations+3 phases,
101 writer effects, and original C/F bindings remains open. Independent review,
matching combined acceptance and whole-goal completion are not supplied here.

## Validation and replay

Final trial016 receipt: `/tmp/native-source-reborrow-e9615ccb-trial016/receipt.json`.
It passes normal coqc and full recursive coqchk for **134 declarations**; every
individual assumptions list is empty and the recursive kernel reports no axioms.
`final-freeze.json` is byte-identical to pretrial trial-016.json. Trial011 is a
successful119-declaration preliminary version; trials001–010 and012–015 are
retained development failures. Pure parser/gate tests also pass under Python -B -O
(`/tmp/native-source-reborrow-e9615ccb-optimized-parser/receipt.json`).
Exact inventory and hashes are recorded in results.json/assumptions.json.
All new declarations (including Fixpoints, constructors, generated eliminators,
record constructors and projections) are compared with Coq Print Module and
individually inspected with Print Assumptions. The inspection module defines no
helpers. No exclusions, new axioms, admissions, unsafe checking or Program
obligations. Full recursive coqchk checks new objects and imported dependencies.

The controller verifies baseline-ready AND successful durable-build-2, exact
VerusBelt115a618d523 sources/284 objects, runner/coqc/coqchk/opam/loadpaths,
1121 Coq source/object/plugin dependencies, the exact GMP manifest
332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37 and archives,
three completed integration receipts/products, exact current dependency source
hashes, compiler/research primary sources and completed extraction. It copies
already checked .v/.vo dependencies under the exact NativeConstructorErasure
and NativeSourceStorage module identities; only new modules compile privately.
Input/private-copy/object/inventory guards are retained through success.
Pure parser/unit controls test source shape, static-loan identity, inventory and
kernel-summary rejection. No filesystem race/substitution/security experiment.

Run the pinned final replay with a fresh output directory:

```
python3 -B source/tools/research/native_source_reborrow/check.py \
  --freeze source/tools/research/native_source_reborrow/final-freeze.json \
  --output /tmp/fresh-native-source-reborrow-replay
```

Development failures are enumerated in development.json with freeze/receipt/log
hashes. They are proof-development/type/parser/setup errors, never semantic
negatives or evidence of Rust safety/unsafety. Raw logs stay in private /tmp;
only compact artifacts are committed. No Python runtime assert gates acceptance.
