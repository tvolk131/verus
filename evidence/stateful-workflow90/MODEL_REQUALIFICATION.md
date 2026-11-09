# Bounded model-artifact requalification

The parent approves ONLY the exact matched cycle-two runtime binding recorded in
source/tools/stateful_bitmap_workflow90/qualification.json. This is an evidence
repair after independent review, not a third compiler/feature cycle, model change,
general fingerprint normalization or cryptographic execution attestation.

The independent review at /private/tmp/workflow90-independent-review.R9LE55 found
the complete 33347277-byte serialized logical Krate byte-identical in original
and rebuilt vstd. All 7442 functions, 382 model functions, datatypes, traits,
associated types, modules, architecture and semantic fields therefore coincide.
Only the wire order of the same 152 source-file metadata mappings differs. The
whole-artifact hash changes, correctly changing every artifact-bound fingerprint.

The original fingerprint-list oracle remains unchanged in expected.json. The new
runtime-specific oracle is independently derived from the original serialized
model functions and the exact rebuilt artifact hash, not learned from the report
being accepted. Both original/rebuilt artifact hashes and the identical logical
payload hash, source/producer/runtime/toolchain/loader bindings and independent
derivation receipts are retained. The runner accepts no other runtime or changed
compiler/model sources. The test-only loop regression source is excluded from
the production-source identity because its metadata assertion is repaired without
changing or rebuilding compiler/vstd artifacts; final reconstruction includes it.

This does not relax producer/schema/policy/import/stale-model or cache checking.
The feature-OFF matrix expects an empty goal-model inventory, as the recorded
compiler reports already do. The 382-model oracle applies only to native A ON;
applying it to OFF was an additional runner expectation error, not model drift.
Original source/native/domain/semantic allowance and ordinary-export boundaries
are unchanged. The eighteen unchanged pseudocode doctests remain an explicit
baseline/context limitation; their failed broad command is not relabeled passed.
