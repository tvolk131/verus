# Checked composition and an exact source-foundation blocker

**No actual Rust constructor erasure certificate was established.** This lane
implemented a fault-aware typed boundary and checked conditional composition,
then proved that the completed descriptor semantics cannot supply the required
root reservation (nor the other unsupported reference/phase steps). The missing
source semantics was not replaced with a safety axiom or an execution filter.
Native0, 44 minimal +3 phase /101 writer effects open, publication=false, goal
incomplete. Canonical6/0 and full HMAC1/1 are inherited, not rerun; their original
full domains, bodies and properties remain unchanged.

Early committed interface: `6bb72bfb`. Only this new directory was changed. No
parent/sibling checkout, unfinished root output, historical checkout span, new
extraction, compiler/vstd/controller/global-policy edit, shared build/install,
external formal-source import, subdelegate, push or PR was used.

## Concrete implementation

`Interface.v` gives a **data signature**, not an asserted Rust operational
semantics. A configuration carries its actual history; a physical address keeps
allocation identity and byte offset; references separately retain their actual
provenance tag. `V_entry` uses independently defined lawful entry, actual `_1`
operand evaluation and arbitrary initialized byte lookups. It mentions no Iris
resource, owner, future run, future equality, proof flag or postcondition. No
source instance interpreting lawful entry is supplied: the predicate's intended
meaning is a requirement on the missing foundation, not a new established fact.
The definitions do not require empty history, tag0, offset0 or zero bytes.
They do not prove coverage of lawful Rust histories merely by omitting those
restrictions. That coverage requires the unavailable source interpretation.

`Composition.v` implements the invariant interface. `source_entry_certificate`
must prove observability of each original array address and construct an
invariant for **every** V_entry. The invariant can hide history existentially.
Its local certificate requires progress at running states and preservation for
**every enabled outcome**, including Fault, Normal and Unwind. Only running
outcomes require further progress; terminal states need no artificial successor.
No termination or guaranteed normal return is inferred.

The checked `execution_invariant`, `bounded_erasure` and `entry_erasure` compose
those proof obligations over finite execution prefixes. They establish fault
exclusion, normal/unwind physical-map preservation and progress on reachable
running states **conditional on that certificate**. `entry_normal_bytes` combines
this with original-address observability to retain all arbitrary initial bytes
at the SAME original subobject. It does not identify those bytes with Verus F.
`enabled_fault_refutes_certificate` checks that even one enabled fault at a
reachable state refutes a certificate. Invalid-reference/retag/raw/storage/control
faults are explicit outcomes; none can be deleted by restricting runs to defined
or successful normal executions.

`graph.json` retains actual structured IR for all six instances/44 operations,
including storage, raw formation, aggregates, call/return, all queried glue and
branches. `Graph.v` lists their exact site identities and the three additional
pre-erasure obligations: FakeRead, call unwind-to-cleanup versus selected unwind
continue, and cleanup `unwind_resume`. Counts and membership are checked Coq data
facts. They are **not operational laws**. Query-only glue is not added as an
executed destructor. No storage/FakeRead/retag/cleanup step is interpreted as
identity by this code. Per-step proofs remain absent for the actual source.

## Attempt against the missing operation

The selected unchanged root has incoming `_1: &mut [u8;128]`, StorageLive at
bb0[0..2], the two-phase `_4 = &mut (*_1)` at bb0[3], activation at bb0[4],
unsize `_4 -> _3`, and the call consuming `_3` into `_2`. Loan0 is the static
local reborrow, not an incoming dynamic loan. The pin/shape producer checks
these actual operands before generating the site inventory.

The exact required source-sensitive judgment is exposed as the unproved type
`Interface.root_reborrow_need M`: from every original V_entry, after a reachable
prefix at that operation, a source step creates a reference in `_4` to the same
allocation/offset with source reference validity; all fault outcomes are
excluded; continuing outcomes preserve lawful source state and every physical
byte. The complete milestone additionally needs the remaining operations and
phase proofs in `Composition.source_entry_certificate M Graph.graph_events`.
These are **definitions of missing obligations**, not axioms or theorems.

`Boundary.root_descriptor_attempt` attempts the reservation transition against
the exact completed `NativeMirLrust.fragment.source_step`. The checked
`root_descriptor_attempt_impossible` proves that this type is empty, universally
over every descriptor list and heap state, using the existing definition's
`unsupported_rejected`. Nine further checked theorems do the same for activation,
unsize, raw-reference formation, retag, storage, FakeRead, drop, unwind and source
phase. This is an exact diagnostic about the supplied formal foundation. It is
neither a Rust counterexample nor a claim that all other possible source
foundations are impossible. The earliest executable unsupported site is already
StorageLive at bb0[0]; there is no checked prefix to the reborrow yet.

Consequently no actual `M` can currently be justified from those supplied
artifacts. Their physical descriptor simulation does not define lawful Rust
history or retag/reference validity, and gives no source progress rule for these
operations. A fresh independently authored descriptor/accessor would not repair
that absence. This lane stopped at this typed blocker rather than manufacture
whole-caller ownership or define source validity by future proof success.

## Integration interfaces and remaining needs

The C/F lane can target `Interface.source_heap M final`, `ref_address original j`
and `execution M initial (Normal final)`; the conditional byte result is
`Composition.entry_normal_bytes`. A genuine source instance/certificate must be
proved first. Original-reference future observation, caller ancestor continuation
and C/F correspondence remain the parent-assigned lane; no final=current law is
introduced here. Pointer-distance is not used by this constructor milestone.

To advance the source lane, supply an independently justified bounded Rust
configuration/entry/step semantics (or an established theorem at that exact
scope) including arbitrary lawful ancestor/protector histories and strict
interior subobjects. Then prove the root obligation and the remaining
reference/phase/normal/cleanup operations in that semantics. An arbitrary record
with the right field types is insufficient: binding an authored semantics to
selected MIR is itself an unproved metatheorem. There is deliberately no
`RustMeaning` axiom, boolean certificate consumer or publication hook here.

## Validation and replay

Final trial006 passed normal coqc for four modules and a declaration-free
assumptions inspection module, then recursive coqchk of all five with no
exclusions or unsafe flags. The 96-entry inventory covers every new declaration,
constructor, record projection, generated eliminator and helper, and is compared
with Coq's own Print Module inventory. Every declaration is closed; the recursive
kernel summary also has **zero axioms**, unsafe fixpoints, assumed positivity,
type-in-type or rewrite rules. There are zero Program obligations. Universally
quantified certificate/semantic arguments remain explicit in the statements and
TRUST_INVENTORY.json; closed assumptions does not discharge those arguments.

The replay authenticates baseline-ready AND successful named durable-build-2
receipt, immutable VerusBelt115a618d's 523 sources/284 objects, runner/coqc/coqchk/
opam, loadpaths, exact GMP manifest332763a9, completed trial018 fragment sources/
objects and extraction/source bindings. All copied inputs and produced proof
objects/inventories remain guarded through final success. Raw outputs stay
private under `/tmp/native-constructor-erasure-6bb72bfb-trial006/`; results.json
contains receipt, proof and log identities. final-freeze.json is byte-identical
to the retained trial006 pretrial freeze.

Four in-memory structured-IR controls reject the wrong root operand, changed
activation, wrong unsize operand and wrong constructor operand without relying
on changed-file hashes. These are **shape controls**, not source-safety proofs.
The unsupported-step theorems and fault-refutation theorem are separately kernel
checked. Development/setup/validator failures are retained in development.json
and their private receipts; none is a semantic negative or native result.

```sh
python3 -B source/tools/research/native_constructor_erasure/check.py \
  --freeze source/tools/research/native_constructor_erasure/final-freeze.json \
  --output /tmp/fresh-native-constructor-erasure-replay
```

No publication or status change is authorized by this result. Root integration
and fresh independent review remain necessary even for these bounded conditional
proofs and diagnostics.
