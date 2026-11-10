# Foundation reuse experiment: no native facts installed

The [reference-adequacy investigation](../native_reference_adequacy/CORE_RULE_PROPOSAL.md)
isolates the missing input/primitive/exit judgments. Existing current/future
equations and an abstract native-constructor summary do not provide runtime
allocation or provenance. A spec-only null-pointer repack can preserve those
equations. Reinterpreting its ghost pointer as physical authority is invalid.

The next experiment reuses an existing mechanized semantics instead of creating
another copyable heap trace. The pinned input is
[VerusBelt](https://github.com/verus-lang/verus-belt/tree/115a618dd35ff8337112cf2081a7f5391d3973d5),
commit `115a618dd35ff8337112cf2081a7f5391d3973d5`. Its
[map of semantic types and rules](https://github.com/verus-lang/verus-belt/blob/115a618dd35ff8337112cf2081a7f5391d3973d5/README.md)
locates unique-borrow interpretation/resolution and pointer-permission proofs.
These are candidate foundations, not a proof of Rust 1.99.1 native iterators.
The published model and vstd interfaces have stated differences; their meanings
must be compared before a theorem is reused.

The concrete target is a checked interpretation of the **same actual input
borrow**, exposing its location and borrowed initialized contents with the
current/future relation and a justified release operation. A borrowed subobject
does not supply allocation ownership, deallocation authority or a permission
that outlives the source loan. Cover all 128 arbitrary bytes and subobjects in a
larger allocation. No supplied raw permission or extra application requires may
replace the original exclusive-reference input.

Any useful foundation theorem still needs a checked connection to the actual
typed native MIR, primitive/retag/exception semantics and final(original array).
The source-derived 115/0 model trace does not provide that connection. The full
frozen HMAC writer and shared automation remain required after the minimal
constructor/drop diagnostic. No foreign theorem or artifact hash alone may
authorize a Verus fact or change publication.

The read-only local dependency is
`/tmp/verus-infer-buffer-cycle-20261109/verus-belt-115a618d`.
[foundation-pin.json](foundation-pin.json) binds the inspected files. Rocq is
not installed at this checkpoint; the dependency's requested toolchain and proof
assumptions must be checked in a private experiment. Production compiler, vstd,
global policy/cache/schema and dependency trust are unchanged.
