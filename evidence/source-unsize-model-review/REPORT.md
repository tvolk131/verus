# Independent carrier MODEL review

**Accept exact MODEL scope at 58ea8ca4.** Fresh normal `coqc`, all33 individual
assumption inspections and full recursive `coqchk -silent -o` passed. All28 Carrier
and5 Root declarations are closed; kernel axioms=[], generated helpers=0,
inspection declarations=0. No blocking mathematical or controller finding was
identified for this frozen representation component. This acceptance establishes
no Rust/native correspondence and does not complete the unsizing statement.

## Precise theorem boundary

`carrier_store` is a partial data operation on `Storage.state H`: distinct source
and destination indices, an initialized immediate `RefValue r` source, an
uninitialized immediate destination and metadata below2^64 yield exactly
`set_local s dst (Immediate (Init (SliceValue r n)))`. Natural lengths are
nonnegative; this is a metadata representability bound, not a proof of a valid
Rust slice size or allocation. Source carrier, original reference, all other
locals, caller and private physical cells, incoming history, private allocation
state, storage trace and CONTROL are preserved. `set_local`'s definition and the
closed preservation lemmas establish these MODEL facts without hidden reference
validity, initialized-byte, permission, source-safety, defined-execution,
no_unwind or C/F premises.

`RefValue` erases Rust pointee type, mutability and array length. It carries a
reference triple (allocation, offset, tag), not proof that the source is an array
reference of length n. Neither the generic interface nor `root_carrier_progress`
interprets ownership, live allocation, initialized bytes, provenance validity or
actual lawful Rust input. The progress theorem has only the two local-carrier
premises; it does not even require the source control location. These are limits
of the partial MODEL, not new application requirements or hidden permissions.

The Python selector authenticates the single original root bb0[4], destination
_3 `&mut [u8]`, source _4 `&mut [u8; 128]`, exact `Unsize` and `move`, with empty
projections. Both the committed graph and original completed six-instance,
44-operation x86_64/64-bit extraction are pinned and checked. Coq's graph theorem
only proves membership of the site token; it does not encode that typed MIR or
prove an extraction/simulation theorem. Generic10/128 laws are not common native
automation for the full applications.

`root_still_pending` conditionally preserves that same site; it never advances a
source statement. `source_dead_does_not_discard_payload` composes two MODEL
operations, preserving the destination carrier and anchors after source-local
death. It proves neither source reachability nor loan resolution. `changed_tag`
increments the natural provenance tag while allocation/offset and every
`ref_address` stay equal. Its inequality is unconditional; there is no hidden
validity/permission premise and no claim that an actual retag follows this rule.

## Interpreter guidance and open effects

Authenticated compiler58cf8f9c sources confirm: cast.rs:446-455 constructs the
pointer/length pair and calls `write_immediate`; place.rs:687-705 performs write
validation when enabled, and :730-760 includes the immediate local-write hook.
cast.rs:82-93 performs outer cast validation too. validity.rs:602-649 checks
access and may call machine retagging, potentially replacing provenance. Neither
validation nor the write hook runs in `carrier_store`. Unchanged carrier tags
are not whole-Rust retag preservation. operand.rs:847-848 reads Move/Copy alike
and leaves invalidation unfinished; source-carrier preservation is not actual
Rust move semantics or doubled permission. These are source inspection facts,
not formal imports or a Rust correctness theorem. The independently frozen
review inputs include the completed storage lane's place.rs guidance; the root
carrier's mathematical dependencies do not include Rust implementation code.

Actual incoming-reference interpretation, source entry/safety, validation,
retag/write-hook effects, move semantics, source control/phase transitions and
original/pending C/F equations all remain open. No source semantics instance or
native certificate was constructed. This pure component imports only Coq and
Interface/Graph/Storage/Carrier; it imports no Iris or VerusBelt logical module.

## Receipt chain and controller findings

Trial004's exact receipt is authenticated through committed results.json:
`371fc3d00c84b58df975d7550774f73955a90a0ceda981b4fb7e6e3daee31c14`.
All mathematical sources and objects, including the generated inspection module,
match trials001/003/004 and the fresh replay byte-for-byte. Interface/Graph/
Storage also match the original completed parent objects. Carrier.v/Root.v match
mathematical commit f6ddbcd4. Trial002 is retained as success=false,
model_proofs_kernel_checked=false, commands=[], TypeError on `len(int)`; the
scalar-count fix in eb56be58 explains it. It is setup failure, not a failed theorem
or semantic negative. No independent replay/proof/control failed.

The current dfe1a04a controller directly pins the completed parent receipt
`47f3ebf95acf6e15d3a6b17deabb0103902d48ac62d05a5a7e2fa8079ff8cd1a`.
Parent review prose/integration-results.json is not a mathematical dependency.
The storage pin file includes historical documentation hashes as data; the root
does not load/validate those prose paths. It freezes code, source selector,
support, tests and candidates before compilation, authenticates readiness and
successful durable-build-2, pins prover/runner/opam and GMP archives/manifest,
and authenticates all1,121 standard-library sources/objects/plugins. Baseline
version receipt says Coq8.20.0 compiled with OCaml4.14.2. The immutable build is
115a618dd35ff8337112cf2081a7f5391d3973d5, runner `run-rocq`; no shared files were
written and no dependencies installed.

The independent preregistration guards1,428 inputs, including the current full
TARGET before and after, with exact requirements hash
`11014c3ac73ddf13c464769f5d0d1cbd24598ab52777718d31872c70b3b8f7b0`.
Individual source inventory, compiler-printed modules, every declaration's
assumptions/statement and the declaration-free inspection source agree. Opaque
proofs printing as `Parameter` in `Print Module` are individually closed and
kernel checked, not axioms. No exclusions, new axioms, admissions, Program
obligations, unsafe checking flags or runtime-assert acceptance gates were used.
This is a source/stability review, not a filesystem security claim. Nine pure
in-memory selector controls passed normal/-O; seven goal-scope controls passed
normal/-O. No race/substitution experiment or Rust build/suite was run.

Execution note: an initial AGENTS.md discovery command used `..` in error and
returned no paths or file contents. No external checkout contents were used;
subsequent inspection used this checkout and explicit delivered completed paths.
Historical checkout paths inside receipts were treated only as labels.

## Retained evidence and full goal

Raw review receipt: `/tmp/native-source-unsize-model-review-58ea8ca4-001/receipt.json`.
Raw fresh proof receipt: same directory, `replay/receipt.json`. The initial
1,428-input map is `inputs.json`; every individual statement/assumption is under
`replay/proofs/inventory/`; aggregate inventory is `complete-assumptions.json`.
Completed original receipts remain `/tmp/native-source-unsize-20261110/trial00{1,2,3,4}/receipt.json`.
Committed results.json, assumptions.json, commands.json and SHA256SUMS contain
compact identities, exact commands and the complete33-entry assumptions inventory.

Every-u64 original canonical varint/arbitrary initialized `[u8;10]` remains the
full canonical target; exact independently frozen HMAC pad-XOR/arbitrary
initialized `[u8;128]`/all valid key lengths including0 and >128 retains prefix
AND unchanged suffix. The same actual checked automation, normal checking,
source fidelity and combined independent acceptance remain required. Canonical
6/0 and full HMAC1/1 unproved are inherited, not rerun. Native0; all44 minimal
source operations+3 phases and101 writer effects open; publication=false;
whole goal incomplete.
