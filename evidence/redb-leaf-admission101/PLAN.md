# Fresh native admission proof allowance

Implement only in new `/private/tmp` scratch, preserving parent and frozen inputs.
Read the authenticated `DESIGN-REPORT.md` for exact native/source/import anchors.
Native pin: `c82ab865bb362bc4af067d332adc44978ff64813`. The recovered native
`btree_base.rs` SHA256 is
`c3a1f5ca0cefdde0bc7d1b40536bc4f2d68a8bce852115433621a8b37fb9ec60`.

## Actual helper preservation

Copy the entire original bodies of `leaf_fits_one_page` and
`leaf_split_required`. Only visibility, result binders, checked contracts and
ghost reasoning may change. Preserve short-circuit order and every executable
operation, including the imported `RawLeafBuilder::required_bytes` call and
`u16::try_from(num_pairs).is_ok()`. Independently audit proof-erased executable
tokens against original source ranges. Do not replace the native builder or add
a trusted conversion specification, external body, admit or stronger input domain.

Let `M` be the existing imported builder metadata specification and `p` the
payload. Under only the arithmetic bound `M+p <= usize::MAX`, establish:

- Native fit is equivalent to `M+p <= page_size && num_pairs <= 65535`.
- Native split is equivalent to `num_pairs > 1 && !fit`.
- A non-split result implies fit only when `num_pairs > 1`; preserve and test
  the oversized singleton exception and the zero-count scalar policy domain.

## Authored caller bridge

Add a clearly labeled authored `fit_then_lookup` client, not a native caller.
Require nonempty equal-length key/value arrays, exact payload and reserved-key
accounting, machine/u32 used-end bounds, matching optional widths, ordered
four-byte numeric keys and a four-byte query. Do not assume the existing
`request`, page-capacity fit or the count cap. Call the preserved native fit
helper on the original page length. Derive those missing premises only in its
true branch and call imported `build_then_lookup` on the original borrow.
Return false/None without constructing on failed admission. Preserve the exact
admission predicate and the existing accepted-branch lookup association result.

All other existing builder premises remain explicit. This does not prove native
accumulator, allocator, original `LeafBuilder::build`, Drop, ownership or I/O.

## Validation and accounting

Use only the matched private midpoint runtime and complete environment in the
assessment's native-leaf-lookup007 attempt011 invocation, with its matching
leaf_decoder, builder_step and lookup_client VIR/rlib pairs. Authenticate the
runtime/import bindings and std conversion/result sources before execution.
No current-parent or operational91 driver mixing. No runtime/toolchain copies.

Verification-only applications use full candidate scopes, one worker,
`--no-cheating`, raw JSON scope/body records and the matched release configuration.
Remove `--compile` and `--export`; no native build/execution or published proof
artifact is requested. Absence of an unrequested export is not export-refusal
evidence. All launches, failures, drafts and metadata corrections stay recorded.

Check exact-capacity equality, count 65535/65536 with adequate scalar capacity,
and the oversized singleton exception. Pair each positive with an isolated
incorrect assertion on identical valid inputs; require localized intended backend
failures, not frontend/import/precondition/time/resource failures. Exercise the
authored caller on present/absent queries, capacity rejection, and fixed/variable
key/value widths. Restore the byte-identical complete positive and verify again.

Maximum 16 fresh verifier applications, one worker, 60 seconds per application,
15 minutes total, scratch at most 128 MiB and free disk above 16 GiB. No compiler,
vstd, Cargo or harness builds, network, Git writes, parent edits or subdelegation.
Bounded checked-proof/frontend corrections are permitted inside these limits;
never weaken the domain, skip original bodies or assume an unsupported call.

Stop on authentication/runtime mismatch, an exhausted bound or a need for new
trust/engine adoption. Do not restart growth049, zero-growth050, mutation030,
padding95/provider-refused padding or refused drop-pipeline work. Preserve all
old ledgers and conditional assumptions. Return exact source deltas, preservation
audit, raw proof/control outcomes, input/output hashes, counts and limitations
in a small sealed handoff. Independent review remains a separate later task.
All 18 final redb cells stay OPEN.
