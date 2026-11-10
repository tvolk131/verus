# Checked Tree Borrows external-model baseline

Integration correction: independent review `5568da00` accepts the exact
mathematical checkpoint and supplies complete expanded signatures and individual
queries for2,753 model bindings. The original507 library-root `Print Module`
outputs do **not** recursively expand every nested module/type. That medium
delivery-controller finding remains; use the bound independent supplement in
[the review](../../../../evidence/borrow-foundations-review/REPORT.md) for complete
coverage. This corrects the original nested-signature completeness claim below;
normal builds, kernel checks,2,399 individual constant queries and trust context
remain supported. Re-running the original generator alone does not recreate
the independent supplement. No mathematical source or baseline receipt changed.

**Scoped success; no native/source-validity or HMAC theorem.** Exact Simuliris
`e3abfcdc301f92250d4227fc03c00463bf57b982` compiled normally, without semantic
patches. Full recursive Rocq kernel checking passed without exclusions:
**71 internal modules, 507 libraries/loaded objects, 52,862 constants**.
All **2,399 internal constants** received individual compiled type and
`Print Assumptions` queries; the eight requested root modules account for 529.
The complete compiled signatures of all 507 libraries inventory inductives,
constructors, eliminators, record projections, generated helpers and nested
modules. This is not a handpicked successful theorem subset.

The private completed result is
`/tmp/tree-borrows-checked-foundation-20261110/baseline-ready.json`, SHA256
`f56490bb8441dbd73c37fe46fe970dc008bb8b2b67ee2b651a04c34d2b1ca523`.
It is bound to successful durable normal-build, recursive-kernel and inventory
receipts, actual object hashes, initial installed/source snapshots and final
hash guards. Readiness is **only for this checked external MODEL baseline**.
Fresh independent and combined acceptance remains the parent's responsibility.

The early scope/gate commit is `eaf8f890`; complete dependency sources were
frozen in `5d7fac41`, before proof builds. Actual closure/tool/loadpath pretrial
freeze is `cfb6ba74`, before the Simuliris build. Base is `6c74c9b1`.

## Exact scope, dependencies and commands

The authenticated completed inspection snapshot matched the inspection pins,
requested git commit and all 185 tracked git blobs before a private archive
copy was made. [upstream-source.json](upstream-source.json) inventories every
tracked source file. The original snapshot was never modified.

The actual project file is `_RocqProject`; `_CoqProject` does not exist. Its
options require Rocq 9.2. Private source builds used OCaml 4.14.2, GMP 6.3.0,
Rocq runtime/core/stdlib 9.2.0, Equations 1.3.2+9.2, std++
`c3186aad493872e3a9774ac0b42ca2f591b05835`, Iris
`b3495ef9f0805b0dc7c1b1abbdc8cf473ede121b`, and the exact transitive packages
in [dependencies.json](dependencies.json). That file records 49 package
identities including bootstrap inputs and manifests for 47,587 extracted source
files. All repository revisions, archive checksums, Git source commits and
package metadata are fixed. The official opam 2.3.0 bootstrap binary is pinned
by SHA256; it was not claimed to be source-built. No moving dependency HEAD,
shared installation/update, HOME/CODEX_HOME change, or credential was used.
Private package commands were serialized, with upstream jobs at most two.

`rocq dep -f _RocqProject` generated the internal dependency graph in
[closure.json](closure.json). Installed imports are traversed recursively by the
kernel checker; its actual import walk supplies the external closure. No
SimuLang, Stacked Borrows or example suite was compiled in this Simuliris build.
The exact executed steps were:

- Generate the upstream `Makefile.rocq`, then `make -f Makefile.rocq -j2` on the
  eight roots' `.vo` targets, letting the generated dependency graph build them.
- `rocq check -o -Q <private-simuliris>/theories simuliris <all-eight-roots>`;
  no `-norec`, admits or unsafe exclusions.
- Normal `rocq repl -q -batch` execution of generated `Check`,
  `Print Assumptions` and `Print Module` queries. No user init file was loaded.

[COMMANDS.json](COMMANDS.json) preserves actual argv, private scripts, logs,
terminal statuses, pid/pgid/startticks, receipt hashes and the separate failed
setup trials. [TOOLS.json](TOOLS.json) provides actual tool hashes;
[results.json](results.json) binds full source/tool/object/receipt manifests.
The exact runner/loadpaths/tool versions are in
[model-pretrial.json](model-pretrial.json). All 507 objects actually interned by
the checker matched either the initial installed snapshot or the 71-object
manifest bound to the successful normal-build receipt. Final guards also
checked retained sources, repository trees, archives, tools and support files.

## Assumptions and boundary

The complete recursively loaded dependency context has exactly:

- `Stdlib.Logic.FunctionalExtensionality.functional_extensionality_dep`;
- `Stdlib.Logic.Classical_Prop.classic`;
- `Stdlib.Logic.IndefiniteDescription.constructive_indefinite_description`.

[ASSUMPTIONS.json](ASSUMPTIONS.json) maps every internal constant to its exact
individual assumption output (five distinct sets, including closed context).
[PUBLIC_TYPES.json](PUBLIC_TYPES.json) retains all 529 root-module constant
types. For example, `log_rel_adequacy` depends individually on excluded middle
and constructive indefinite description; `pool_safe_implies` is closed under
the global context **with its explicit ordinary premises retained**.
[INVENTORY.json](INVENTORY.json) binds the complete private compiled signatures
and full checker declaration walk, including all generated helpers.

The kernel audit reports predicative Set, no rewrite rules, no type-in-type,
no unsafe (co)fixpoints and no assumed positivity. Source keyword observations
in [SOURCE_AUDIT.json](SOURCE_AUDIT.json) are separate from that authoritative
kernel audit. No new Coq declaration, admission, axiom stub, weakened theorem
or semantic-rule patch was authored. Three successful OCaml-package builds
initially failed an over-strict source guard because Dune relocated an identical
`.opam` version header and normalized trailing whitespace. Exact differences
and rejected receipts are retained; only that metadata equivalence is allowed.
There are no failed model proof trials or claimed semantic controls;
[TRIALS.json](TRIALS.json) distinguishes setup/tool rejections from semantics.

Ordinary hypotheses are not global axioms and are not discharged by printing
assumptions. `simulation/language.v:639` defines `pool_safe` by non-stuckness of
all reachable **model** pools. `slsls.v:63–67` consumes it inside
`sim_expr_inner`; `lifting.v:76–95` defines `SafeImplies` and uses source-pool
safety. `simulation/adequacy.v:207–208` transfers safety **from a safe source**.
The retag rules use `pool_safe_implies` to obtain source-step conditions.
None proves that the original Rust source satisfies that premise. This baseline
adds no native `source_safe` axiom and assumes no original defined execution,
no_unwind, client permissions or narrower entry domain.

`bor_semantics.v` separately requires a live call, existing parent, fresh child,
child-creation effect and retag read. Its Reserved-to-Active permission-machine
transition is not Rust borrowck two-phase activation. Relating this calculus to
pinned rustc operations, arbitrary lawful incoming references/ancestors,
protectors, cleanup/phases, actual bytes and original/pending Verus C/F remains
an independent unproved bridge. The calculus definitions and ordinary
Rocq/OCaml/GMP/host execution environment remain explicit trust boundaries.
Compiler/artifact success and hashes do not establish Rust-native validity.

Only the assigned new directory and private root were changed. No Rust rebuild,
engine/vstd/policy edit, native import, production fact, publication, push/PR,
subdelegation, live parent/sibling checkout/build read or sibling-builder wait
occurred. Public primary upstream:
[Simuliris at the exact commit](https://gitlab.mpi-sws.org/iris/simuliris/-/tree/e3abfcdc301f92250d4227fc03c00463bf57b982).

The full goal is unchanged: canonical every-u64/arbitrary initialized [10]
remains **6/0**; frozen original HMAC arbitrary initialized [128], every valid key
length including 0 and >128, XOR prefix **and unchanged suffix**, remains
**1/1 unproved**. Those application results were not rerun here. Native=0;
all **44+3** minimal/phase and **101** writer effects remain open. Same actual
checked assistance, normal fidelity/trust/inference/publication gates and fresh
combined independent acceptance remain required. Publication=false; goal
incomplete.
