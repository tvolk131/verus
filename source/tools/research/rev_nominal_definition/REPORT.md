# Experimental Rev nominal definition: stopped before native binding

**STOP: internal native representation/source coherence is undischarged.** No
compiler hook, observation definition, standard postcondition, trusted helper,
assumption or production fact was installed. This is a research-only partial
implementation of diagnostics, not a completed semantic-definition prototype.
It neither establishes the requested native minimal loop nor proves that a new
compiler semantics is impossible. `publication=false`, `goal_complete=false`,
native correspondences=0. Parent retains integration authority.

Preregistration c21ec836 froze INTERFACE.json and the source/domain/TCB boundaries
before any production edit. Diagnostic implementation b86260d5 adds own actual
Item MIR extraction, a checked parametric logical model, private immutable runs,
and a read-only audit. No production hooks commit exists because its entry gate
was not discharged. Evidence is committed separately. Parent HEAD was fdd0b0c6;
accepted production remains1171bdf7/engine d3b2aa6e/schema39policy74cache77.

## Exact first remaining obligation

A candidate definition is still:

```
alpha_Rev(native Rev {iter:x}) = C(alpha_I(x))
F(C(v)) = v
rev_iter<I>(r) := F(r)
```

This requires ONE nominal datatype interpretation of actual core Rev<I> for
all its valid values, parameters, aliases, boxed/generic contexts and updates,
while source field access remains private. It must not depend on which call
produced a value, nor on which module uses a generic vstd definition.

The first undischarged implementation/source-coherence obligation is the
module-uniform internal representation elaboration, including its field borrow
and writeback rule. For an arbitrary supported native value r, outer mutable
borrow q and actual private-field reborrow p = &mut r.iter, it must establish:

```
alpha_I(current(p)) = F(alpha_Rev(current(q)))
alpha_I(future(p))  = F(alpha_Rev(future(q)))
```

The second equation concerns normal writeback/future values. Unwind behavior
must remain explicit and be checked separately. The same representation must
commute with typed box/unbox, equality and generic instantiation across modules.
Only then can the checked logical lifting below apply to the actual native field.
This is an additional compiler semantic/TCB obligation, not an inherited iterator
premise and not a strengthening of trustedStdModel axioms.

Concrete source inspection pins the blocker:

* `DatatypeTransparency` in vir/ast.rs has only Never and WhenVisible. Existing
  external Rev lowering in rust_to_vir_adts.rs uses Never and zero fields; the
  transparent external-proxy branch rejects private fields.
* well_formed.rs denies opaque field/constructor access. prune.rs removes fields
  of nontransparent datatypes for a module. poly.rs selects boxed versus native
  representations using that same transparency decision and resolves field types
  from the retained field list. datatype_to_air.rs emits a datatype only for a
  transparent type. sst_to_air.rs rebuilds a field update with that datatype's
  constructor/fields.
* There is no independent internal-encoding/private-source-field descriptor or
  validated current/future elaboration for this opaque external type. Changing
  visibility, injecting a Field node, or rewriting a user assertion cannot supply
  the required all-module/all-value interpretation. The exact loaded vstd symbol
  binding and independent full actual-Function checks were therefore not reached.

The loaded native steps actually use this obligation: next/back create a mutable
borrow of field0, delegate to Enumerate::next_back/next, and retain StorageLive,
StorageDead and an `unwind continue` edge. They are not aggregate-only bodies.
No no-unwind or native safety premise is inferred. The constructor's pure shape
is insufficient to certify the shared field interpretation after these uses.
Promoting the logical Carrier to actual Rev here would assume precisely the
missing correspondence. No such promotion was made. This is an unresolved rule,
not a universal unprovability result under the newly authorized semantics.

## Fresh actual Item evidence

The exact permitted Item fixture SHA256
1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d was
copied byte for byte, with its genuine declarations, private u64 carrier, NonZero
wrapper and runtime bodies retained. Ordinary Rust compilation passes with three
appended diagnostic roots and the literal borrowed-Vec reverse-enumerate loop.
This checks erased Rust, not the Item's Verus contracts or loop lawfulness.

Own pinned rustc_private inspector re-extracts actual Item MIR from that source.
The final epoch reuses only its own already completed, source/toolchain/binary
hash-bound inspector build; it freshly analyzes the new source and loaded core
again. No old VIR or changed compiler was mixed into a positive input.
IDENTITIES.json records local Item DefId(0:56), its field DefId(0:58), crate and
DefPath hashes, source spans, four instantiated core signatures/ParamEnvs/bodies,
field identities/visibility/types and full generic/instantiated MIR digests.
Numeric DefIds are contextual diagnostics, not binding authority.

For I=Enumerate<slice::Iter<'a,genuine Item>>, the actual loaded instances are:

* Iterator::rev and Rev::new: one aggregate assignment and return; field0 is the
  same private core Rev::iter, stable core crate ID17281971461272426270,
  field hash Fingerprint(17281971461272426270,5029879297925419223).
* Rev::next and Rev::next_back: mutable field reborrow and delegated calls as
  described above, two blocks, explicit storage and unwind behavior.

The root signatures retain the source lifetime binders. Region-erased instance
MIR is diagnostic, not authority to erase lifetimes or infer source Copy. The
source preserves receiver evaluation/move once; no into_inner or runtime proxy
is present. Ordinary Rust controls independently reject field privacy (E0616),
receiver reuse after move (E0382), and Vec mutation during the live readonly
iterator borrow (E0502). These are compiler negatives, not semantic assertions
or tests of a new implementation's refusal gates.

## Checked logical compatibility, and its limits

compatibility.rs has **10/0 ordinary proof bodies, zero executable proof targets
and zero inferred/generated helpers**. It uses a separate logical Carrier<I>,
not the actual core datatype and not an executable algorithm replacement.
C/F computation, eta and injectivity hold uniformly for all model values. A
generic context exercises ordinary polymorphic lowering. Further checked bodies
prove reverse-sequence identities; lift an inner back/front transition relation
to the reversed front/back relation; preserve lawfulness, None-exhaustion and
metric existence/decrease; swap arbitrary peek functions; map observations at
arbitrary worlds; and transport generic shared-reference values and enumerated
indices. The indexed-reference lemma has an explicit mathematical index-range
premise. No application premise is added or claimed derived for every native state.

These proofs use inherited Seq/Option/spec-function trust and ordinary Verus
checks. Transition relations and world mapping are explicit logical hypotheses/
definitions. They are not assumed facts about actual private fields, native
references, provenance or prophecy. In particular this proves neither native
box/unbox coherence nor the native current/future equations above. It is a checked
conditional model of core reversal laws, not full compatibility of all vstd or
all Rev consumers under an installed new definition.

Three logical clients (false inner, false lawfulness, false index) each report
**10/1**, with the ten model proofs healthy and exactly the intended assertion
failing. A fourth, missing-frame client, supplies both inner and outer inherited
transition relations plus current coherence and still fails its asserted final
field identity, **10/1**. That failure is a diagnostic support boundary; it is
not a native counterexample or a proof of general non-entailment. The source
inspection identifies the absent elaboration rule independently of SMT failure.

The unchanged arbitrary &Vec<u64> baselines are freshly rerun off/on with
--no-cheating --compile, all normal checks, full crate and no filters:

| probe | off | on |
|---|---:|---:|
| available constructor/remaining/none/decrease observations |1/0|1/0|
| returned Rev lawfulness |0/1|0/1|
| returned inner identity |0/1|0/1|
| literal empty-body reverse-enumerate forloop |1/1|1/1|

All eight request/generate zero helpers. Only each successful available case
certifies its one exec body. Failed whole crates certify zero exec/proof bodies;
partial verified counts are not whole-crate certificates. The Item MIR loop is
ordinary Rust only; no successful native Verus loop or causal assistance gain
is claimed. INVENTORY.json retains exact selected body inventories, including
spec declarations and zero helpers.

## Authentication, stopped gates and inherited limits

Final immutable epoch: /tmp/rev-nominal-definition-sy4c6v_f. RESULTS.json binds its
raw sources, diagnostics, logs, inspector and products. Private freeze.json binds
all unchanged production sources, controllers, exact eight tools, pinned compiler/
core/std/alloc artifacts, target and environment. The exact parent74 receipt and
all eight bundle files were authenticated and copied privately. The solver was
copied from the explicitly allowed exact source with its pinned digest and
executable mode. Final guards rehash all those inputs. No Cargo dependencies
were needed; no shared Cargo cache was read or modified. RUSTUP_HOME was explicit.

Two prior immutable epochs retain excluded setup errors: missing rustup in the
private PATH, and spec-function-field method syntax. Preliminary successful
subresults are not pooled as final acceptance. ATTEMPTS.json also retains the
sandbox-launch and Git-identity setup events. No panic/frontend/resource outcome
is counted as a semantic negative. audit.py accepts diagnostic evidence using
exceptions, identically under normal Python and -O.

The following remain **unimplemented/unvalidated**, rather than silently waived:
compiler hooks and descriptor, exact loaded observation semantic identity, all
actual Function/representation validators before/after WF/autospec/modes, complete
supported-value/consumer inventory, source coherence and effects/cleanup rules,
all-selected local-only refusal before pruning (including zero-helper/off cases),
semantic-definition fingerprint mismatch refusal, fresh matching compiler/macros/
full NORMAL vstd after semantic edits, corruption matrix, useful native loop,
cold/warm/cache/export/import states and independent semantic review. No modified
artifact is available for publication/import. The preregistered rejection gates
are obligations, not implemented compiler defenses. Global epochs are unchanged.

Wrapper28/0 with5 candidates/15freshchecks, original Item132/0, packet31/0 and
bool provider/ordinary import compatibility remain inherited parent results,
not rerun here. Production source was unchanged, but that does not demonstrate
compatibility of any proposed new semantics. Full matching rebuild/regression
acceptance belongs after an actual representation implementation and review.

Tagged fallback probes were not attempted because useful minimal loop success
was not reached. No full tagged lookup/cache prelude, native tail frame, ordering
publication/import or full HMAC capability is claimed. Fallback wf/cache invariants
remain authored/conditional. Native0, full HMAC1/1 and44+3/101 remain OPEN.
