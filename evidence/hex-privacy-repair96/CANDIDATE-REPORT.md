# Native hex 1.3.0 privacy-preserving specification repair

2026-11-07, America/Chicago. Implementation-subagent handoff. No integration.

## Outcome and acceptance boundary

**The repaired specification verifies the entire retained selected native-v3
scope plus canonical single-byte encoding, using the frozen operational91
engine.** All 256 byte inputs and both cases are covered symbolically. Original
arbitrary-table selection semantics are retained without a canonical-table
precondition. No engine or library modification is needed for this repair.

Six direct verifier applications were executed: two routine frontend failures,
one initial full-scope positive, the required semantic negative, a restored
fresh-cache positive, and a restored same-cache-directory replay positive.
Each positive reports 48 verified obligations, zero errors, 36 checked named
executable bodies and one checked proof body. All 32 historically selected
executable bodies are retained and checked. Both actual original encoding
constant initializers are selected with ordinary checked postconditions.

**Important cache qualification:** invocation 6 reuses exactly invocation 5's
private cache directory, but the directory remains empty. The sole inherited
Infer helper reports `cache: miss`, `cache_policy: regenerated_state` on both
runs. The frozen engine regenerates this owned/state summary rather than
persisting it in its scalar candidate cache. Invocation 6 freshly proves all
three required helper certificates and all selected bodies; it is not evidence
of a persisted-candidate cache hit. A strict requirement for an actually warmed
persisted candidate is therefore **not established**. No cache was fabricated,
no different target was substituted, and no runtime change was attempted to
manufacture that result.

This is **not final native acceptance, current-parent acceptance, historical-v3
engine acceptance, new trust adoption, or clean-checkout acceptance**. It is a
verified disposable-source repair with the controls and cache limitation stated
above. Parent independent review must resolve acceptance, including the intended
warm-cache prerequisite; no acceptance is claimed if that prerequisite is
incomplete. There is no missing-trust-permission claim or blanket-trust request.

The earlier encoding95 failure remains a failed experiment: one invocation,
three frontend opacity errors, zero checked bodies. Its files and recorded
result were not overwritten or relabeled. All 92 entries of its output manifest
still authenticate.

## Deliverables

All new work is exclusively in:

`/private/tmp/verus-hex130-privacy-repair-20261107-zm5hKz`

- `final-source.patch`: exact additive delta against the authenticated native
  archive, including the new specification module.
- `native/src/lib.rs`: original native code plus ghost-only selection,
  specifications, a closed view and an erased proof call.
- `native/src/encoding_spec.rs`: independent arithmetic ASCII policy, canonical
  table predicate, nibble arithmetic proof, and the universally quantified
  executable goal.
- `repair-versus-failed-candidate.patch`: explanatory lib.rs delta from the
  preserved failed candidate, not a relabeling of its result.
- `semantic-negative.patch`: the single original Lower-branch mutation.
- `validation-receipts.json`, `RESULTS.json`: structured outcomes and controls.
- `source-preservation-audit.json`: complete native hashes, exact inserted spans,
  erased-body hashes, and unchanged original constant initializer hashes.
- `invocation-01` through `invocation-06`: exact argv, sanitized explicit env,
  source snapshots/hashes, raw stdout/stderr, timing, export and cache results.
- `authentication-entry.json`, `authentication-pre-*.json`,
  `authentication-post-*.json`, `authentication-exit.json`: runtime, loader,
  native input and source/build qualification authentication receipts.
- `imported-model-inventory.json`, `assistance-inventory.json`,
  `source-investigation-bindings.json`, `discovery-and-provenance.json`:
  imported models, assistance, source grounding, and non-verifier side quests.
- `control.py`, `audit_repair.py`: bounded invocation and preservation/audit
  harnesses. The preserved old preparation/authentication script is imported
  read-only with its output root rebound to this fresh scratch directory.

## Exact repair and why privacy is preserved

### 1. Closed body-defined view of the private field

A cfg-gated `impl View for Table` has `V = Seq<Char>` and the closed specification
body `self.0@`. It is a real definition in the original `table` module, not an
uninterpreted declaration, assumed contract, external body, or opacity axiom.
Its field access remains within the native private-field boundary. The full
verification run checks the selected definition's well-formedness and its uses.
The native-function receipt correctly labels it
`specification_definition_or_declaration`, not `checked_exec_body`; this report
does not inflate executable/proof-body counts with specification definitions.

The original type stays exactly `pub(crate) struct Table([Char; 16]);` with its
private field and original derives. No constructor or field visibility is
widened. Crate-visible helper postconditions now use `self@` rather than exposing
`self.0` outside its module.

### 2. Check the actual constants, without publishing constructors

Both original `pub(crate)` constants retain their exact native declarations and
initializers. Each gains `verus_spec(ensures ...)` selecting its actual
initializer as an **exec constant with a checked postcondition**, rather than
the failed candidate's crate-open spec constant constructor body.

The frozen-matching native attribute rewriter, in
`source/builtin_macros/src/attr_rewrite.rs:640`, explicitly translates a native
const postcondition into exec-constant mode and places its original expression
in the checked body. This is also the supported pattern covered by
`source/rust_verify_test/tests/consts.rs`. It is not a replacement table or a
trusted constant summary. The postconditions assert canonical ASCII policy on
`Self::LOWER@` and `Self::UPPER@`, respectively, from the **actual** initializers.

The initial attempt used unqualified associated names `LOWER`/`UPPER` in these
annotations; invocation 1 diagnosed Rust name resolution. Invocation 2 fixed
those names to `Self::LOWER`/`Self::UPPER`, then diagnosed exec constants being
read from unrelated spec contracts. The final design does not read exec
constants from those contracts. It exposes their already checked *view
property*, using the ordinary checked constant postconditions.

### 3. Keep general helper semantics and prove independent policy

The helper's unconditional equations are:

```
result@[0] == self@[(byte as int) / 16]
result@[1] == self@[(byte as int) % 16]
```

No precondition is added: every legal `Table([Char;16])`, including reordered,
mixed-case or repeated entries, remains in-domain. The closed view is exactly
the private array's sequence, not a canonicalized abstraction. Conditional
canonical consequences only apply when the input view actually satisfies the
corresponding arithmetic policy; these implications do not restrict the input.

`Case::table` gets a checked postcondition that its returned view satisfies
`canonical_table(result@, self)`. Its original match and references are unchanged.
The `canonical_byte_encoding_goal(byte: u8, case: Case)` has no preconditions and
calls exactly `case.table().byte_to_hex_chars(byte)`; its required postcondition
uses the independent policy below.

The policy is arithmetic, not a copied alphabet: for `0 <= n < 16`, ASCII is
`48+n` below ten, otherwise `87+n` for Lower or `55+n` for Upper. The high index
is integer division by 16 and the low index integer remainder modulo 16.
The constants' original `Char` discriminants and actual arrays must satisfy
this policy. No assumed or replacement array appears in the proof module.

The ordinary proof `encoding_index_facts` proves the shift/mask bounds and
equalities to machine division/remainder with four bit-vector assertions. A
cfg-gated `proof!` call is inserted before the helper's original statements.
The arithmetic bridge and subsequent casts/index reasoning are checked in the
complete application run, not substituted by an executable exhaustive test.

## Full-scope validation and failed attempts

Every invocation uses the same retained cfg set, crate name and scope:

```
--crate-type lib --crate-name hex_conservative
--cfg feature="std" --cfg feature="alloc" --cfg verus_hex_vec_phase3
--infer-goals --no-cheating --output-json
--export <invocation>/hex_conservative.vir
--num-threads 1 --sysroot <original exact sysroot> --out-dir <invocation>
```

There is no application scope filter, `--no-verify`, `--no-lifetime`, admission
weakening, or `--compile`. Exact complete argv/env are retained per invocation.
The direct invocation ledger is:

| New invocation | Purpose and result | Seconds |
| --- | --- | ---: |
| 1 | Full-scope candidate; associated-const name-resolution failure; no export | 0.296 |
| 2 | Full-scope candidate; unrelated specs cannot read exec constants; no export | 0.510 |
| 3 | Full-scope repaired candidate; 48 verified, zero errors, interface written | 2.351 |
| 4 | Matched semantic negative; 47 verified, one semantic proof error, no interface/export | 2.403 |
| 5 | Exact restored source, newly empty private cache; 48 verified, zero errors | 2.180 |
| 6 | Exact same source and cache directory replay; 48 verified, zero errors | 2.077 |

These are six of sixteen allowed applications. Zero minimal diagnostic probes
were used. The routine failures consumed their ordinary application budget and
are retained; they were not retrospectively declared probes. Three invocations
were reserved for and spent on the matched negative and restored controls once
the full-scope positive existed. No exhausted historical control was repeated.
The pre-run metadata field `application_credit` classifies a full-scope
application *attempt*, not an accepted proof: invocations 1, 2 and 4 earn zero
positive application proof credit. Raw pre-run metadata is preserved unchanged.

Positive verification metadata reports `is-verifying-entire-crate: true`,
`complete_selected_scope_verified: true`, `native_collection_completed: true`,
36 checked named executable bodies and one proof body. This means the entire
**selected** local Verus scope, not every Rust function in the native crate.
The 32 historic executable body identities are a subset of the positive checked
identities. The four added named executable bodies are `Case::clone` (selected
by preserving the native Case derive), `Case::table`,
`Table::byte_to_hex_chars`, and `canonical_byte_encoding_goal`.

The new encoding work is ordinary authored verification, **not** new
Infer-generated exact helpers. The sole generated helper remains the original
`iter::hex_chars_to_byte`, with the original exact contract and three mandatory
certificates (soundness, safe-domain maximality, output exactness), all proved in
each positive. The warm replay records nonzero fresh solver time for those
certificates. It reports regenerated state, not reused proof evidence.

Existing assistance is reported separately: one sequence-pattern site, one
error-callback site, two array-callback sites, and four type-invariant-entry
sites, with their ordinary checks retained. `goal_assistance` is empty. No new
encoding helper is counted as an Infer proposal or publication. The local
generated pair helper is available to same-crate callers in successful runs but
is not exported as a public Infer helper. Successful crate interface/export
output is recorded separately. Collection/proposals are not acceptance.

### Matched semantic negative

Only this original executable branch is changed, with all requirements and
proofs unchanged:

```
Case::Lower => &Table::LOWER,
```

becomes `Case::Lower => &Table::UPPER,`. Invocation 4 fails with
`error: postcondition not satisfied` at `canonical_table(result@, self)` in the
original `Case::table` body. It is not a parser, unsupported-feature, opacity,
lifetime or timeout failure. JSON reports `encountered-vir-error: false`,
`status: caller_proof_failed`, `crate_success: false`, and
`interface_written: false`; no `.vir` export is written. Existing pair-helper
certificates are not an accepted interface for this failed crate.

The original Lower branch is then restored exactly. All native source hashes
for invocations 3, 5, 6 and the final candidate are identical. The negative is
preserved only in its invocation snapshot and the one-line negative patch.

## Input authentication, source preservation and runtime binding

Authoritative source is extracted as disposable regular files/directories from
the authenticated archive, with safe relative paths and no `.git` member. It is
not a Git checkout or worktree. Native provenance identities remain:

- Native acceptance: `54b6672c00c2cbeac8bcd7c6761cf1b0bec1a6a2`.
- Historical engine: `6d80948ce8c981bd650ae851cd99c64cf7625f5d`.
- Upstream hex 1.3.0: `129db6ac54d25b08e0b927c225aefe4a8df05a57`.
- Native archive SHA256:
  `e7f4b63eced039d437b54858fc600071b41614aa66b2a9fdc27caa94c9e0de5b`.
- Original archived `src/lib.rs` SHA256:
  `c5d584fb99c620c8e086c970afcf14b36fa4bb8838d9ead1a415ef3142c67073`.
- Runtime manifest SHA256:
  `1fa38f105d5b9fda9bda2ad3b0ee4518f083f917bc77dcbce6585ff152e476a3`.

All 53 extracted archive files are inventoried. Exactly one existing file,
`src/lib.rs`, differs; its diff consists only of inserted lines. Every other
archive file is byte-identical. The only added native source file is the
cfg-gated `encoding_spec.rs`. All original lines, existing specifications,
requirements and selected scopes remain. Both original encoding functions'
signatures and erased bodies match byte-for-byte, both constant initializers
match byte-for-byte, and all original derives and field privacy are unchanged.
Final source hashes and exact insertions are in `source-preservation-audit.json`.

The frozen matched runtime is used **read-only in place**:

`/private/tmp/verus-operational91-runtime.q91/runtime`

All nine manifest artifacts authenticate before and after every direct
application and at entry/exit: launcher, driver, builtin, builtin macros,
state-machine macros, vstd rlib, vstd VIR, recorded vstd builder (never executed),
and Z3. All 30 loader libraries and the recorded rustc artifact authenticate too.
Recorded historical successful compiler/vstd build receipts and their logs are
authenticated, not replayed. Matching source/build qualification is preserved.
The nine inspected frontend/test/cache source files match their frozen source
manifest entries; details are in `source-investigation-bindings.json`.

The exact original sysroot is
`/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin`.
The original loader is its `lib` directory followed by
`lib/rustlib/aarch64-apple-darwin/lib` (full exact value in each invocation).
`VERUS_USE_RUSTUP=0`, the matching `VERUS_Z3_PATH`, explicit sysroot and original
matching loader environment are retained. Only private scratch HOME/TMPDIR/XDG
and Infer-cache paths and output destinations are rebound; no secret-bearing
ambient environment is inherited by verification.

No runtime artifact is copied or changed, and no new hardening96 driver is mixed
with old libraries. No compiler, vstd or Cargo build, network operation, Git
operation/worktree creation, native executable test, alternate runtime or trust
adoption is performed.

### Trust and independent review

The proof remains conditional on the ordinary Verus backend and matching
imported vstd/library specifications, exactly as the runtime reports. The source
delta contains no `assume`, `admit`, axiom, external body, or assumed spec. Closed
visibility is not treated as authority to assume correctness.

Positive frozen runs inventory 382 imported model fingerprints; the historical
engine's receipt inventories 291. These are not relabeled as identical engine
trust inventories. The matching runtime and library artifact hashes, concrete
current inventory and historical qualifications are retained. No new library
or trust rule is installed or adopted by this source repair. Parent independent
review is required before any successful proof/trust-relevant change is adopted.

## Budgets, bounded side quests and handoff

Six direct applications take 9.817 seconds total verifier wall time; each is
below 90 seconds, with one worker and no overlapping verifier. Fresh scratch is
about 10.3 MiB at audit, below 256 MiB. Authentication and the shell preflights
check more than 16 GiB available before every work batch; the bounded controller
checks free space, allocation and its 15-minute deadline before every direct
application. Final exact elapsed/allocation and output hashes are in
`handoff-finalization.json` and `output-manifest.json`.

The authorized side quest is ordinary specification repair: native const
annotation semantics, closed-view visibility, and cache-policy interpretation.
Routine name/exec-mode errors were corrected rather than invoking the old
first-error stopping rule. Exploratory path-name mistakes, one read-only
`find` permission diagnostic and a zsh local PATH-name mistake caused no
verifier application or out-of-scratch write; they are documented separately.

The runtime deliberately does not persist the inherited state summary, so the
remaining concrete limitation is the requested warm-cache interpretation, not
a justification for silently changing the engine or adopting trust. No engine
gap remains demonstrated for the native canonical-byte specification itself.
Do not claim an actual warmed candidate hit from this handoff.

No encoder iterator, encoding vector, whole-crate-native, other-target, other
hex version, or current-parent acceptance is claimed. No redb provider-blocked
task, growth049, zero-growth050, mutation030, drop-pipeline or distinct cached
rust-hex 0.4.3 lane is retried. All 18 redb obligations remain OPEN; S0 and the
ordinary Infer/full-scope/import/export barriers are unchanged. Stop at this
handoff; parent arranges review and any integration.
