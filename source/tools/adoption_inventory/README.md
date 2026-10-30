# Authored source inventory

This standalone tool uses the fork's Rust/Verus parser to list authored views,
contracts, attributes, function bodies, assumptions, loop clauses, proof syntax
and macro calls in explicitly selected source files. It complements the compiler's
`verus-infer.verification_scope` report. It does not classify a contract as an
appropriate application requirement or certify the absence of hidden behavior.

Build from the repository root with the pinned toolchain. An explicit target and
empty compiler-development flags keep this independent tool separate from Verus:

```sh
CARGO_TARGET_DIR=/tmp/adoption-inventory-target CARGO_ENCODED_RUSTFLAGS='' \
  cargo build --release --offline --locked \
  --manifest-path source/tools/adoption_inventory/Cargo.toml
python3 source/tools/adoption_inventory/check.py \
  /tmp/adoption-inventory-target/release/verus-adoption-source-inventory
/tmp/adoption-inventory-target/release/verus-adoption-source-inventory \
  /path/to/project src/collections.rs src/application.rs > authored-inputs.json
```

The inventory includes source syntax from disabled `cfg` branches and test modules;
it performs no cfg evaluation, macro expansion, module discovery, type checking
or proof checking. Direct `verus!` and `verus_trait_impl!` item syntax is opened.
Other item/impl/trait macros and function-body macro calls remain explicit opaque
entries for review. Supply all relevant source files, reconcile them with the
compiled native scope and inspect macros/dependency interfaces separately.

The checker exercises visibility of authored clauses, proof blocks, loop clauses,
manual triggers, assumptions, external bodies, hidden macro tokens, inherited cfg
and malformed-source rejection. Trigger findings inspect identifier tokens in
attributes, including nested native contract attributes; words in documentation
or other string literals do not count as proof hints. These are parser tests, not universal proofs of
the auditor. A source review must still decide which contracts express intended
semantics, whether a required implementation was excluded and whether trusted
interfaces or preconditions weaken the intended application claim.
