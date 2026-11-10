# Independent review: permission predicate and limited acceptance controller

Review base `8aa9fa72`; source helper `8aa9fa72`, controller `eb7778b5`,
integration evidence `e401fc23`. Scope/pretrial commit `133dd2ce`;
independent-runner pretrial commit `7e4c5b9c`. Only this new evidence directory
was edited. No parent/sibling checkout was read. Historical checkout labels
were treated as data; own committed bytes were authenticated through Git.

## Findings

**P2 / R1 — isolated replay fails despite the prescribed same-byte freeze
remapping.** `source/tools/research/native_miri_permission_grants/replay.py:119`
indexes the canonical receipt by `str(p)` for every `tool_artifacts` result.
That list includes the two repository-local Python support modules. Their
own-checkout paths differ from the original receipt keys even though their
committed bytes/hashes are identical. The direct `python3 -B -O` replay using
`freeze-own.json` fails with `KeyError` for this checkout's
`source/tools/infer_eval_support.py`, before any verifier command. The retained
`replay001/receipt.json` has `success=false`, no commands and no runs. Thus the
promised freeze-only isolated replay is not currently usable. Fix by comparing
these support bytes through an explicit authenticated repository-path mapping,
or by separately binding support modules to their freeze and the eight external
bundle files to canonical receipt keys. Do not read original checkout paths or
change the accepted canonical receipt. No production fix was made here.

**P2 / R2 — negative gate does not bind the failing function identity.**
`source/tools/research/native_miri_permission_grants/replay.py:81` checks a
postcondition diagnostic and the aggregate 2/1 count, but never checks
`verus-infer.failed_functions`. All three executable rows on a failed crate use
`selected_body_not_certified_by_successful_full_run`, so lines 76–77 cannot
establish which one failed. The specified requirement is one failure in
`Permission::grants` with both Clone bodies healthy. Require the exact failed
function singleton (and no failures for positives), alongside the current
identity/count/diagnostic checks. This is a source-only gate completeness
finding; no poisoned-output/substitution experiment was performed. Actual
fresh outputs have the correct singleton and pass the independent strict gate.

No new blocking finding in B. P3 is resolved by the metadata correction.

## Scoped acceptance

**A: accept the current source component and these six independently checked
outcomes; request changes to the delivered replay gate (R1/R2).** Independent
runner `run_independent.py` imports none of the delivered acceptance predicates.
It authenticates the explicit own-path freeze, the original canonical receipt
keys for identical repository bytes, and the unchanged eight-file verifier
bundle. It uses the same normal verifier flags, with `RUSTUP_AUTO_INSTALL=0` and
a private temporary directory to prevent install/shared-output work. It was
committed and frozen before the first verifier trial. The direct delivered
replay's setup failure is excluded; no verifier trial was hidden or replaced.

| Run | Goals | Verified/errors | Required failing function |
| --- | --- | --- | --- |
| enabled-cold | enabled | 3/0 | none |
| enabled-warm | enabled | 3/0 | none |
| disabled | disabled | 3/0 | none |
| disabled-guard-removed | enabled | 2/1 | Permission::grants only |
| read-inverted | enabled | 2/1 | Permission::grants only |
| readonly-guard-removed | enabled | 2/1 | Permission::grants only |

Fresh receipt: `replay002/receipt.json`. Every negative has exactly one
postcondition error at the authored ensures, and two successful executable
queries. No precondition, assertion, invariant, arithmetic, frontend, resource
or timeout rejection qualifies. Full selected scope, exact Rust identities,
body statuses, contract flags, source paths, zero proof bodies, zero inferred
functions and the empty imported-inference dependency inventory were checked.
There are six selected items: grants, two original Clone derives, the authored
spec table and two Structural declarations. The eight original Eq/PartialEq/
Hash/Debug methods are explicitly outside selected scope. Negatives' zero
certified body count is a crate-wide status; it is not mistaken for three failed
queries. `failed_functions` plus the 2/1 query inventory identifies the failure.

Independent token/source inspection matches the executable grants body and both
enum variant lists to the three exact pinned Rust files. Original derive lists
and order are preserved, with only Structural appended. The entire adapter
file is accounted for, including its single contract and explicit eight-row
truth table. No precondition, proof block/call, snapshot, inferred helper,
assumed function summary, external_body, new representation or narrowed domain
was found. Direct Boolean row evaluation supports this contract inspection;
it is not native execution or an additional model theorem. Clone generated
body semantics and equality lowering remain compiler/macro/verifier trust.

**B: accept the limited controller revision and completed-parent replay
binding.** A full AST projection from `eb7778b5^` to `eb7778b5` proves the only
changes are the 17+12 explicit failure checks, their helper, optional
`STBOR_ENTRY_OUTPUT_ROOT`, and its import in summarize.py. Original conditions
and supplied messages are exact; guards without messages receive original-line
labels. The complete remaining AST/control flow is identical. Both files parse
and compile in memory at optimize levels 0, 1 and 2. All 17+12 guard loads and
the explicit RuntimeError raise survive at every level; no runtime asserts
remain. No delivered controller was executed and summarize.py was not invoked.

Mathematical source bytes are identical across `89494dce`, `15e3c268`,
`eb7778b5^`, `eb7778b5`, `e401fc23` and `8aa9fa72`. The completed parent raw
product map validates all 31 top-level products; terminal controller exit is
finished/0, all five command receipts are finished/0 with exact controller,
source and log hashes, and declarations' 50 type/assumption output files match
their inventory and integrated independent review. The 25 declarations are
individually closed; kernel log, dependency map and loaded-object map agree on
all 282 modules. Parent own .vo hashes match both the worker inventory and
integrated independent-review hash inventory. No Coq/kernel/dependency rebuild
was performed. Foundation source/tool/object hashes are recorded and compared
as inherited completed evidence; unnamed baseline paths were not followed or
freshly rehashed. The parent’s claim of a historical `-O` launch is not a new
process-attestation result; optimization independence is checked directly from
the current exact source bytecode.

**Terminal-state boundary:** check.py writes acceptance.json at line 149 before
the final foundation guard at line 153. This ordering is inherited, unchanged.
The file is provisional alone. A failed final guard prevents products.json and
writes an error terminal receipt; summarize.py requires the products and a zero
controller exit before writing status. This review requires all three records,
not acceptance.json alone. The authenticated completed run has all three and
no terminal error. The final foundation check remains active under optimization.

**P3: accept metadata-only correction.** Compared directly against immutable
`995809af`, MILESTONE changes only the collection label and adds an explanatory
classification. Every original receipt field and exit is equal, including
`interface-02=1`; the other four recorded exits remain zero. results.json changes
only the milestone evidence hash and explanatory correction record. Entry.v is
identical. No new Tree/Coq acceptance follows from this label change.

## Evidence and exclusions

`pretrial-inventory.json` inventories initial file bytes/hashes (including all
named parent accepted025 files), original controls, pinned sources, bundle and
receipt inputs. `freeze-own.json` lists the six explicit repository-path
remappings and unchanged external inputs. `runner-freeze.json` binds the
independent runner before any verifier trial. `replay002/receipt.json` contains
exact argv/cwd/environment, full initial stability snapshots, run outcomes,
function identities, trusted model fingerprints, source/stdout/stderr hashes
and final guards. `static-normal.json` and `static-optimized.json` contain the
full guard/message/bytecode inventories, Git source identities, raw product
map, all five historical command receipts, recorded foundation tool/source
hashes, 282-module hashes, closure/trust context and P3 comparison. Their
semantic contents agree after removing only Python optimization mode.
`COMMANDS.json` and `FINAL_INVENTORY.json` index exact reproduction commands,
Git identities and final evidence hashes.

Excluded, retained setup/controller failures:

* Parent001 receipt SHA256
  `a9c698f5a1159e8649588603b52cde41cbd89340221aa2d12f5556a1073177a0`:
  initial selected executable inventory omitted the two Clone derives.
* Parent002 receipt SHA256
  `a70e285175ae4b313ac0ace42d3a72871f4a8023d63e6b76a0585072aa7944e3`:
  auxiliary spec/Structural items were misclassified. Neither parent setup has
  an accepted run; their positive ordinary compiler outcomes are not negatives.
* Own replay001: R1 receipt-key setup failure; zero verifier commands.
* `static-setup-failure001.json`: independent reviewer initially read
  integrated `compiled_modules` (printed module text list) as a hash dictionary.
  Corrected only the review script to read committed PRIVATE_ARTIFACTS hashes;
  retained the failure and reran the pure audit normally and optimized. No
  mathematical rejection or accepted run derives from this schema failure.
* Initial sandbox launch failed before executing its read-only command
  (`bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted`). Subsequent
  scoped commands used reviewed escalation. No toolchain/shared files changed.

Accepted parent helper receipt SHA256:
`31c75c1118c787cef6420e65d96fdbc5ad8fc39bc1a0f146a1b8b8330d354ce3`.
Canonical receipt SHA256:
`2dda5d93603528fe2623f4e37bc7cbaa48a537e3184c653a1abe1a4028f4ed3a`.
Parent entry acceptance SHA256:
`e178df9b7adb2714ed6ae177271db895da11e972ab2488d7088d8a1445b666e0`.
Parent products SHA256:
`42c2d0ed744fd929bad5c4cf05b10b493aec6d2159f5f0f6e2e08970dee135dc`.
Parent terminal SHA256:
`ff1b76b862ff5f6a716a315bacdbfb6abb209f9ad03a5095fe1ecdda09b0eb21`.
Other exact hashes are in the machine-readable inventories, not abbreviated.

## Trust and whole-goal boundary

A inherits the pinned source acquisition, Rust compiler, derive and Verus macro
translation, equality semantics, verifier/backend/SMT and vstd specifications.
No full Rust/compiler/Miri suite was run. The exact pinned source selection
proves no source/frame/runtime permission relation, stack/cache/provenance
invariant, grant lookup, FnEntry transport, protector/retag effects, physical
bytes or C/F correspondence. Inference assistance is redundant: disabled goals
also prove 3/0. This provides no missing causal shared-automation transfer.

B inherits its previously accepted conditional historical model, Coq checking,
completed tool/source/object provenance and imported trust. The actual
282-module context records proof_irrelevance, functional_extensionality_dep,
JMeq_eq and eq_rect_eq; individual own declarations are closed. This is not a
change to the broader foundation trust context (recorded separately by the
integrated review), the model domain, or an actual-current-Miri source theorem.
P3 preserves all prior proof/domain/trust limitations.

The full goal stays exact canonical every u64 over arbitrary initialized
exclusive [u8;10], inherited accepted 6/0; independent bitcoin_hashes 0.14 exact
HMAC over arbitrary initialized [u8;128], every valid keylen including 0 and
>128, XOR prefix AND unchanged suffix remains 1/1 UNPROVED. Neither full goal
was rerun or promoted. Shared actual checked automation, normal fidelity,
inference/publication gates and fresh whole-goal acceptance remain required.
Native=0; 44 minimal + 3 phase / 101 writer effects OPEN;
publication=false; goal_complete=false.
