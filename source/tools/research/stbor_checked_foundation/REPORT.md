# Checked historical Stacked Borrows baseline

**Ready at EXTERNAL MODEL scope only.** The unmodified official artifact at
`d1471b10fe4cb4f506371b2e3acdd927d0a0c076` compiled normally. Full recursive
`coqchk` succeeded for **518 modules**: all44 `_CoqProject` modules, all250
installed third-party modules, and224 imported Coq modules. No `-admit`,
`-norec`, unsafe checking mode, unsupported-phase omission, or new Coq
axiom/admission was used by the controllers. This is not a native bridge,
application proof, or claim that every upstream declaration has a proof body.

The final [RESULTS.json](RESULTS.json) is authoritative. Lowercase
[results.json](results.json) deliberately retains the historical pretrial state
unchanged, so the initial controller guard remains reproducible.

## Durable acceptance and identities

Completed artifacts are under `/tmp/stbor-checked-foundation-20261110/`:

| File | SHA256 |
| --- | --- |
| `completed-build-receipt.json` | `0d85e9ec05f29413973137f03bc840ccb1e22170d50169b2837331e58770f649` |
| `baseline-ready.json` | `9259b8f303e52e38f1b47709429f2be23a81a4e44ed557b6058184a95e653e51` |
| `receipts/upstream-build.json` | `a9a8bc8f585cd7b01e0f1bd40593b63768d089722eb92bf397770099f784f5c6` |
| `receipts/recursive-kernel-check.json` | `4c3760de9b34079e01a7e2b19f734bbe84fb65fefa0a9a8898e2c691f5bfa9cf` |

`baseline-ready.json` was created only after successful durable build/kernel
receipts, completed inventories, and matching final guards. Its completed
receipt binds exact runner/command vectors, exits, logs, tools, maps and guards.
[COMMANDS.json](COMMANDS.json) indexes45 retained finished command/hook receipts,
including failures. Main long jobs retain pid/pgid/startticks. No own job remains
running. Initial scope/interface commit: `93818f89`; dependency freeze:
`d4ade759`; original complete trial controller: `0e533a85`/`a75aba83`;
additive record inventory: `ab57c9d2`; selected final controller checkpoint:
`c610a86e`. Base: `6c74c9b1e45a885fe88ab7a2eb01b9a20fc003ee`.

The completed inspection snapshot was authenticated against its committed
`pins.json`, Git commit/tree and **every tracked file**, then copied privately.
[upstream.json](upstream.json) pins all52 tracked files. All44 project modules
are in [OBJECTS.json](OBJECTS.json). The tracked `theories/lang/concurrency.v`
is not listed by upstream `_CoqProject`; it was retained/pinned, and is not
claimed compiled. No project module or optimization phase was skipped.

## Private toolchain and patches

Normal private opam2.3.0 installation selected OCaml4.07.1, Coq8.9.1,
camlp5 7.08, num1.3, ocamlfind1.8.1, Paco4.0.0, Equations1.2.1+8.9,
Iris `627984129ca88df5a6c89fca3b64cfa146d582c3` and stdpp
`16eb222ff70c5e8ed9a41dda3ce18f95e03909df`. Repository snapshots, package
metadata, archives and compiler inputs were frozen before formal compilation;
no unpinned dev HEAD was resolved. [pins.json](pins.json) binds the complete
private pretrial manifest. Packages built serially, each with at most two jobs.

Only toolchain/transport changes were used; exact diffs are in `patches/`:

* Maintained OCaml4.07.1 opam metadata at official opam-repository commit
  `5095d4fd3728a0b7783c24f4b45dbe663c9f4d7b`, with GCC10 `-fcommon` and
  official alternate-signal-stack backport
  `00b8c4d503732343d5d01761ad09650fe50ff3a0` for modern glibc.
* The stock Coq opam recipe's Makefile-only camlp5 printer compatibility patch
  was applied even with OCaml4.07.1. No kernel/formal source was patched.
* Coq/Equations release archive checksum corrections followed complete payload
  comparison against official Git commits
  `e1d8a33ccb9ffecde340531cbf9e8fa9d916550e` and
  `69160e69b629587f9dfed88c545c1e7f13cd18b0`. Coq's export-omitted files are
  only Git metadata; all program/proof sources matched. Checksums were not
  disabled. The original checksum failures remain setup trials.

The selected `coqc` hash is
`46f278986e06298cad00f042257e769b8271b993641de723b630ce3553cc2adf`;
`coqchk` is
`2560b874e1c2f9e2dd2c501f51ad4aace1212d12eb0363e6b2f997fafa88dd7f`.
The normal runner hash is
`822f5c82a4f87e8a8a25b86b9508d9bcc33912c3494274d158da9f451c7f6e`.
All remaining binaries/configuration/runtime identities and actual versions,
loadpaths and loaded libraries are bound by RESULTS/receipt map hashes.
Coq loadpaths contain this private switch/artifact, not the existing Coq8.20
environment. The installed prefix, including library/plugin products, remained
unchanged through final acceptance.

No apt/global install, shared opam root/cache/configuration, HOME/CODEX_HOME
repurposing, existing toolchain mutation, credential use or Rust rebuild occurred.

## Inventory, assumptions and exact prerequisites

[DECLARATIONS.tsv](DECLARATIONS.tsv) covers1,247 compiler-discovered binding
entries. All constants, including generated helpers/eliminators/projections,
have exact `Check` and `Print Assumptions` outputs. Every kernel inductive block
and source record is also printed in full:43 block outputs retain implicit
constructors/mutual declarations that have no explicit source constructor name.
Raw complete types, assumptions, full declarations, glob origins and output
hashes are in `manifests/declaration-inventory.json`. No new proof declaration
was introduced by these read-only queries.

**1,185 entries are closed under the global context;62 depend only on
`Coq.Logic.JMeq.JMeq_eq`.** [EXPORTED_INTERFACE.json](EXPORTED_INTERFACE.json)
contains exact selected semantics/progress/retag/adequacy types and assumptions.
All five requested retag/progress/inversion lemmas are closed under the global
context, but retain their explicit operational premises:

* `retag_head_step`: nonempty call stack, `valid_block`, and model state WF.
* `retag_change_ref_NZST`: positive model type size and an already successful
  reference retag. `retag_Some` and active-protector preservation also take a
  successful retag. Inversion takes an actual model transition.
* `adequacy_never_stuck`: simulation, both state WF judgments and source
  never-stuckness. `adequacy_classical` instead takes `stuck_decidable_1`,
  simulation and WF. `classically_stuck_decidable` explicitly takes excluded
  middle; its name does not make that premise a proved fact.
* `sim_prog_sim_classical`: stuck-decidability, `has_main`, and the local-function
  simulation. The adequacy exports above depend on JMeq equality as inventoried.

The broader518-module checker context additionally lists standard Coq
proof irrelevance, dependent functional extensionality, classical excluded
middle, Eq_rect_eq, Equations Type `funext`, and the upstream **admitted**
`Equations.Type.Subterm.FixWf_unfold`. It also reports six Iris
`iProp_solution` interface fields as axiom-like constants: `iprop.v:118–156`
provides a checked sealed module implementation. These context entries are
retained verbatim in RESULTS; they are not falsely attributed to every artifact
export. Unbuilt Equations examples/tests contain additional admissions and are
not claimed as checked library modules. No upstream proof is claimed closed
merely because its object exists. Iris's inherited warning suppression
`-w -convert_concl_no_check` is recorded; it does not disable the subsequent
full recursive kernel checks.

## Maps, guards and trial limits

The completed receipt binds the full source/object map for all518 loaded
modules, source hashes matching initial payloads, object hashes, all250
installed dependency objects, compiler-discovered module coverage, assumption
maps, actual loadpaths/configuration and complete command/log receipts.
`object-dependency-map.json` records the dependencies/imports serialized by Coq
in each already checked `.vo`; its module set equals the kernel-checked set and
every dependency is inside that set. These recorded dependencies can be
transitive, not merely direct textual `Require` edges.

`vo_summary.ml` is a normally compiled inspection-only reader using Coq8.9.1's
standard summary layout/segment reader. It adds no proof axiom and does not
replace kernel checking. Its source, executable, compile/query receipts and
controller freeze are bound to the completed receipt. Source-based coqdep maps
with resolution warnings were rejected as incomplete, despite zero exit codes.

Final guards matched:52 upstream files,36,899 installed files,5,858 initial
dependency payload files,8,990 build-entry snapshot files,118 platform/runtime
files,16 downloads and24 original controller files. Additive inventory and
object-reader controller epochs were separately frozen and guarded, with their
hashes transitively bound into the receipt. All full maps/logs/products remain
private; no upstream tree, binary or raw build log is committed.

[failed-trials.json](failed-trials.json) and
[later-failed-trials.json](later-failed-trials.json) retain bootstrap,
authentication/path, checksum, source-scanner and metadata-reader setup trials.
There were **no failed formal proof trials or application semantic controls**.
Early bootstrap errors predate the durable runner and have transcript diagnostics
rather than original raw log files; early OCaml internal stdout was managed by
opam, while its successful top-level log and per-command exit/source receipts
remain. All subsequent formal build command logs were captured privately.
One broad process-list diagnostic inadvertently displayed unrelated process
command lines; none were used as evidence, and no sibling checkout/source/product
files were opened. Later process checks were private-PGID scoped. Historical
checkout paths in receipts remain labels, not authorization to inspect them.

## Unchanged native/application boundary

The artifact actually has Default/TwoPhase/FnEntry/RawRt; TwoPhase maps
`UniqueRef true` to SharedReadWrite. It has old Untagged raw tags and an explicit
bounds TODO. This is not pinned Rust1.99.1/current Miri semantics. Model retag
heap stuttering does not establish grant success or actual Rust validity.
Neither simulation adequacy nor closed model progress proves the original
source's safety, lawful-entry coverage, no-unwind, physical permission,
source correspondence, original-reference C/F, or refinement of conservative134.
The parent candidate at `d6b9bd6e` was not compiled or imported here.

Canonical every-u64/arbitrary initialized exclusive[10] remains inherited6/0.
The original independently frozen HMAC writer, arbitrary initialized[128] and
every valid key length including0 and>128, **XOR prefix AND unchanged suffix**,
remains1/1 unproved. Native=0; all44 minimal+3 phase/101 writer effects remain
open; publication=false; whole goal incomplete. Same actual checked assistance,
normal fidelity/trust/inference/publication gates and fresh combined independent
acceptance remain required. No new native fact/certificate or Infer import is
provided by this baseline.
