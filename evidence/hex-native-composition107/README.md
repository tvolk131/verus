# Native hex single-byte composition107 — conditional independent review passes

This independently bounded implementation lane adds an executable composition
for arbitrary `u8` and both `Case` variants through the actual retained native
`Case::table`, `Table::byte_to_hex_chars` and private `hex_chars_to_byte` calls.
It introduces no replacement encoder/decoder or visibility weakening. It does
not cover `decode_to_array::<1>`, string conversion or the unsafe buffer path.

The [candidate report](CANDIDATE-REPORT.md) records three direct applications:
two full positives each report 54 verified obligations, 38 executable bodies
and two proof bodies; the swapped-nibble `0xab` negative fails only its intended
assertion. The corrected `0xba` control verifies, and restoration is byte-exact.
The implementation identity/span audit reports all 32 historical bodies retained.
Fresh independent review confirms these bounded claims. Metadata and shell-wrapper failures
remain in the original receipts; they are not erased or counted as proof success.

`CANDIDATE-HANDOFF.tar.gz` retains every authenticated output-manifest payload
plus the manifest and handoff. Every retained member was compared byte-for-byte,
and all source hashes matched before and after retention. [RESULTS.json](RESULTS.json)
binds the member/archive identities separately from theorem acceptance.
[SOURCE.patch](SOURCE.patch) is a candidate delta against the frozen repair,
not an integrated parent application tree. [VALIDATION-RECEIPTS.json](VALIDATION-RECEIPTS.json)
retains the implementation application's exact outcomes and scope accounting.

Evidence is conditional on the frozen operational91 runtime and its imported
standard/library models. No compiler/source provenance repair, current-parent
runtime acceptance, native resource proof, trust adoption or semantic integration
is established. The strict persisted owned warm-cache gate remains **UNMET**.
All eighteen final redb cells remain **OPEN**; S0 and existing admission/export
barriers remain unchanged. The other parallel implementation lanes continue.

## Independent review and rechecks

The unchanged [independent review](INDEPENDENT-REVIEW.md) finds no new
theorem-specific blocking defect. Three fresh applications reproduce the 54/0
positive, one intended backend assertion with no export, and byte-identical
54/0 restoration. All 32 historical implementation identities match, including
both distinct `From` overloads. Scope is 38 executable plus two proof bodies,
not 54 bodies. The reviewer authenticates all 132 candidate payloads, 134 parent
archive members and 202 immutable bindings at entry and exit.

`INDEPENDENT-REVIEW-HANDOFF.tar.gz` retains all 59 authenticated review payloads
plus manifest and handoff; every member was compared byte-for-byte and original
hashes matched before/after retention. [REVIEW-RESULTS.json](REVIEW-RESULTS.json)
binds these identities separately from the unchanged candidate archive. The
warm-cache and compiler/source gates remain open; no acceptance or adoption follows.
