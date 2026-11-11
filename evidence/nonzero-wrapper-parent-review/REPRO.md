# Review evidence and commands

PRIVATE_ROOT points to this review's own mode0700 raw build/runtime/receipt directory. Do not reset it or reuse its immutable labels for another experiment. The source target is parent d3b2aa6e; only the new evidence directory changes after that parent. TOOL_FREEZE.json is the compact pretrial compiler/core/model/tool identity record; each private command has freeze.json, stdout, stderr and receipt.json, and captured controller bytes keyed by SHA256. Raw sources/MIR/VIR/AIR/binaries remain private.

Executed with Python3 -B and WRAPPER_EPOCH=proofs: build.py setup; resume.py build; resume.py vstd; checks.py build; controls.py semantics/compatibility/exports/shapes; checks.py run; additional.py; mir_inventory.py; global_controls.py. Setup used only authorized checksum-listed offline Cargo inputs and the exact authorized solver/Item. All Cargo commands are offline/locked/jobs2 Rust1.99.1. RUSTUP_HOME is explicit. Full applications use --no-cheating, normal lifetime/trait/body/erasure/entire-crate verification and all three exact candidate checks.

Two extracted controller functions initially omitted import aliases. resume_controls.txt supplies those bindings, binds itself in new receipt dependencies, preserves completed nohelper receipts and appends the remaining PartialOrd-only controls/proposals. resume_global.txt similarly resumes absent global-control receipts after infrastructure143, authenticating exact sources before using completed receipts and never recreating deleted selected artifacts. off_requests.txt adds twelve goals-off wrapper/mixed/private controls and binds itself as a dependency. Final extra.py fixes the original import omission; old executing bytes remain captured in receipt snapshots. None of these controller corrections changes compiler/source/tool/proof inputs.

The final read-only acceptance commands are:

```
WRAPPER_EPOCH=proofs python3 -B evidence/nonzero-wrapper-parent-review/audit.py
WRAPPER_EPOCH=proofs python3 -O -B evidence/nonzero-wrapper-parent-review/audit.py
```

They use explicit exceptions/require checks, compare RESULTS.json, rehash source/tools/compiler/core/fixtures/dependencies/controller snapshots/logs and all query inventories, and require all68 selected-output controls,52 compiler tests,149 VIR tests and8 library tests. VALIDATION.json records read-only before/after evidence hashes and agreement. SEAL.json binds compact final evidence. No historical source audit is run against current parent epochs: authenticate.py binds the permitted historical final manifest to c4ddc83e through own Git, retaining out-of-scope historical path labels unread.

No ordering interface, native theorem, broad parent suite or whole-goal result is produced by this review.
