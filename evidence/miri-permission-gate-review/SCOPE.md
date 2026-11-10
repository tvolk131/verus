# Independent permission predicate and controller review

Owned writes: ONLY new evidence/miri-permission-gate-review/. Initial checkout
HEAD: 8aa9fa72. No production changes, installs, rebuilds, checkout creation,
subdelegation, sibling/parent checkout reads, shared artifact mutation, push/PR,
race/security/substitution experiments, native execution, or new model facts.

A: inspect exact pinned Permission::grants/enum bytes and authored exhaustive
contract; run committed replay.py with an explicit own-checkout-only remapping
of freeze003, leaving completed external bundle, sources and receipts exact.
Fresh six-outcome outputs are kept here. Independently inspect selected item
identities, body statuses, diagnostics, source/control/tool hashes and guards.

B: compare immutable Git before/after AST, normal/optimized compiled bytecode,
identical accepted math source history and completed parent accepted025 products.
Do not execute check.py/summarize.py or rebuild Coq. acceptance.json alone is
provisional until final guard/product map AND successful controller exit.
P3: compare 995809af MILESTONE/results to current, preserving receipt fields/exits.

All acceptance checks use explicit exceptions, never Python assert. External
paths may be read only if explicitly authorized completed artifacts. Old
checkout labels in JSON are strings; never follow them. Inherited baseline
source/tool/object hashes may be inventoried as recorded, not claimed freshly
rehash-validated unless within named authorized artifact directories.

Canonical every-u64 / arbitrary initialized exclusive [u8;10] remains inherited
6/0, not replayed. Independent bitcoin_hashes 0.14 exact arbitrary initialized
[u8;128], every valid keylen incl. 0/>128, XOR prefix AND unchanged suffix stays
1/1 UNPROVED. Native=0; 44 minimal+3 phase/101 writer effects OPEN;
publication=false; goal_complete=false. No helper/model-to-runtime promotion.
