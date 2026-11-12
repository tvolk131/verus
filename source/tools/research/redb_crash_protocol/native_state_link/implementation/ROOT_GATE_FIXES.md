# Root coverage and metadata corrections

Prospective validation scope after independent reviews9f55f2899 and3d55de257.
The old two-marker and four-directive issues are independently closed. Complete
F1 admission still has a crate-root coverage mismatch; D1's backend type-use
guard omits type-bearing metadata. Neither finding enabled a native theorem.

The first correction checks the entire crate visitor plus both opaque type
fields explicitly. Its nine-location regression checks trait implementation
arguments/bounds, associated-type implementation RHS/arguments/bounds, trait
declaration/associated bounds, and opaque arguments/bounds, with rejected u32
and accepted u64 at both the guard and actual Ctx entry. These synthetic fixtures
do not claim to pass the full source/WF pipeline or establish a native proof.

The second correction keeps every crate-root exclusion, but ignores root verify,
verus_macro and mode flags as positive body coverage, matching get_crate_items.
The real default-verify option and actual item/module contexts still establish
coverage. Twenty-eight added ordinary/Infer controls test all three root opt-ins,
item/module/default-option positives, and retained root external/skip negatives.
Expected focused inventory:226 invocations across12 tests, retaining all198 old.
Expected complete VIR inventory:172, including the new metadata regression.

Use existing check.py/infer_experiment/infer_eval_support execution and guards.
Build a fresh compiler in a NEW private directory, no old build-cache copies.
Disable Cargo incremental and build debug information to control disk usage;
these are compiler-build settings, not disabled verifier checks. Original client
no-cheating/lifetime/trait/mode/body/erasure settings remain. Authenticate and
reuse the seven matching supporting artifacts; NORMAL vstd2257/0 is inherited,
not a new verification result. Preserve all old snapshots, logs and ENOSPC data.

Required before scoped acceptance: full focused and VIR checks, loaded-native
controls, valid direct ordinary/Infer root-coverage comparisons, final guards,
exact source/runtime/receipt/cost records and a NEW independent recheck. No pass
is claimed by this prospective document. Actual admitted-owner to checked
Function reconciliation, full native authority/correspondence, F2 and F3 remain
distinct activation gates. Every experimental source request stays inactive.
Full REDB CP1/CP2/source coupling remain unproved; HMAC/native stays DEFERRED
AND UNACHIEVED. No default or schema/policy/cache changes.
