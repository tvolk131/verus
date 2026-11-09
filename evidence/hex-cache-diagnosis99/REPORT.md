# Persisted warm-candidate gate: source-only handoff

2026-11-07, America/Chicago. No implementation, integration, adoption, or new
runtime qualification. All 18 redb obligations remain OPEN; no native acceptance.

## 1. Decision and authenticated inputs

**The strict prerequisite remains UNMET/OPEN.** The owned lane has no persisted
candidate read/write path. A replay of its freshly regenerated and freshly
proved candidate cannot satisfy an actual persisted-reuse prerequisite.
Candidate-ID variation is separate, not the cause of this empty cache.
Neither issue is repaired or waived here.

Input aliases:
- R = `/private/tmp/verus-hex130-privacy-repair-20261107-zm5hKz`.
- V = `/private/tmp/verus-hex130-independent-review-20261107-7xnzpY`.
- Source anchors are relative to the supplied worktree root.

Authenticated report pins:
- R/REPORT.md: `ac72b226b919c8eb6923b7d35ace2e8d2dbc30221f5ce5a1263db576f9bc09db`.
- V/REVIEW.md: `72e536659fb1cf25bb974a088885d29dec14d4eea8d362130e2cb4b244f6b6f8`.

`INPUT_MANIFEST.json` binds paths, sizes and hashes; all 15 R/input-bindings
entries authenticate. `SOURCE_EVIDENCE.json` contains anchored source excerpts,
SHA256 comparisons against all three source manifests and nine investigation
bindings, plus parsed raw 05/06 evidence.

Identity is the frozen operational91 receipt/source/runtime binding, NOT dirty
parent HEAD. Registry and reporting hashes match R/source-investigation-bindings:
- `source/vir/src/infer/registry.rs`:
  `96367d3c01767f1271ff621f3158077fb761cd64121356daf9035d68e86b4f43`.
- `source/rust_verify/src/infer_report.rs`:
  `86f833fd497bdbde2b3837bbf10db06c9d9c4cc1cd78d067476d56e15c3e2379`.

All 15 operational source files used here match every frozen source manifest.
Two additional inspected files, current `source/vir/src/ast.rs` and
`source/vir/src/def.rs`, DO NOT match; their observed/frozen hashes are preserved
as drift evidence and their current definitions are excluded from operational
conclusions. This is not a claim that the entire current parent matches.
Retained runtime-manifest SHA256 is
`1fa38f105d5b9fda9bda2ad3b0ee4518f083f917bc77dcbce6585ff152e476a3`;
retained driver SHA256 is
`468229c6de195c436973a2a1fdd0b7744eb0a1099f3f659ce282831556536e5f`.
These are retained receipt identities, not new qualification. Existing
build/loader/sysroot qualification remains necessary for this engine only.

## 2. Established observations and source cause

Raw R/invocation-05 and -06 invocation.json, stdout, stderr and result.json:
- Same native source-hash maps, same native input pathname and working directory,
  same `VERUS_INFER_CACHE=R/cache-restored`. Both recorded before/after inventories
  are `{}`; read-only inspection still finds the directory empty.
- Both exit 0 without timeout: 48 verified obligations, zero errors; 36 checked
  executable bodies and one proof body. These are retained selected-scope results.
- Sole inherited helper: `hex_conservative::iter::hex_chars_to_byte`,
  `summary_kind=owned_value`, `mode=exact`, `loop_proofs=[]`. Both report
  `cache=miss`, `cache_policy=regenerated_state` and identical displayed contracts.
- Soundness, safe-domain maximality and output exactness each say required/proved;
  solver times are 5382 and 5631 microseconds. Fresh checking is not persisted
  reuse. Crate interface output is separate from this private helper's export.

**Established root cause of no persisted hit:**
`source/vir/src/infer/registry.rs:360` selects `is_owned_summary`, gathers sorted
dependency keys plus datatype context, constructs an owned contract/body via
`owned::infer`, lifts observations and installs it. Lines 388-395 increment
misses, set `cache_hit=false`, mark `record_template=true`, then `continue`.
There is no `read` or `write` in that branch. Scalar persistence only occurs
later at lines 504-538 and cannot be reached for this owned candidate.
`source/rust_verify/src/infer_report.rs:191` labels owned summaries;
`:208` and `:209` report miss and regenerated_state from these fields.
This bypass, not a diagnosed I/O failure, explains the empty cache.

Why bypassing scalar persistence is deliberate, not just missing wiring:
- `registry.rs:18` states the cache must not inject AST nodes, binders, types,
  spans or executable code. The `Formula`/`Candidate` wire language at :19-50
  contains version/key/dependency/pre/post data, not a serialized compiler AST.
- Scalar decoding (:117-229) rebuilds typed terms from the CURRENT function's
  parameters/result, rejects unsupported/type-invalid forms and bounds nodes.
  Candidate read bounds bytes (:263-270); validation precedes source cache use
  (:456 onward). Valid wire shape is not a proof.
- Owned values include sequences, records, nested Option/Result, variants and
  complete-value observations (`owned.rs:150`); their allowed imported formulas
  also include restricted model calls and binders (:2440). Scalar `Formula` does
  not represent this whole language. This actual helper needs nested Result,
  a two-field error tuple, Option observation calls to `decoding_spec::digit`,
  variant tests/projections and bitwise formulas. Directly feeding such formulas
  to scalar encode is not supported (fallback `unreachable!` at registry :110).
- Owned inference also returns checked proof decoration, not only pre/post
  (`owned.rs:2223`). `contract.rs:82` validates that decoration before installation.
  `owned/loops.rs:430` rejects executable-code changes after stripping permitted
  decoration; callback validation delegates to that gate (`callbacks.rs:339`).
  The narrow helper has no reported loop proofs; this does not authorize bypass
  of body preservation or a general arbitrary-body serializer.
- The parallel collection-lane comment (`registry.rs:451`) explicitly rejects
  decoding arbitrary state expressions through scalar candidate-cache format.
  This corroborates the format boundary; it is not proof of an owned codec.

## 3. Candidate IDs: established facts vs hypotheses

Observed 05 ID:
`a77449b1ab80e0c8b7ebd21c02e19e5c6670b6d9081c1cf029d0818147b7bfef`.
Observed 06 ID:
`4916a7c587241ec6f58094480600bd5de9209ee1aebc39458dcff10d471f30ec`.
Do not confuse this installed candidate ID with the cache lookup key.

Established source dependency chain:
1. `registry.rs:11` hashes VERSION, configuration, Debug-formatted original typed
   function and dependency list for the lookup key. The owned dependency list
   includes all accumulated keys, sorted, followed by datatype Debug text (:361).
   Empty reported helper dependencies are not evidence of no key dependencies:
   reporting lists referenced inferred callees (`infer_report.rs:175`), not all
   model/vocabulary/type dependencies in the registry key.
2. Matched `source/rust_verify/src/query.rs:550` builds configuration from Debug
   of `self.args`, model fingerprints and architecture. Matched `config.rs:77`
   derives ArgsX Debug and :79 includes the export path. Raw argv differs at
   indices 16 (05/06 export destination) and 22 (out-dir). Therefore an actually
   varying configuration input is identified: **the export path**. Do not claim
   out-dir is also a key input without tracing its storage in ArgsX.
3. `registry.rs:600` adds wrapper fingerprint; `contract.rs:105` then hashes
   original function, actual pre/post, decorated body, nonlinear flag and
   provenance/key to produce the installed candidate ID. Displayed contract
   equality alone cannot establish equal typed formulas or provenance.

Thus changing export destination is a confirmed source-level identity input,
and would obstruct same-key reuse even after adding persistence if left in
the configuration. It is NOT the explanation for the existing owned branch's
unconditional regeneration. Full exact hash preimages were not retained, so
this investigation cannot isolate every contributor or reproduce either ID.

Hypotheses only: run-local typed-IR/span metadata, datatype/model changes or
ordering might also change Debug inputs. No specific span bug, semantic change
or corruption is established. Owned dependency
keys are explicitly sorted and wrapper shapes use BTreeMap (`newtypes.rs:10`),
so casually blaming those HashMap iterations is not supported. A future
identity investigation needs per-component canonical digests/preimages, not
another solver replay or an assertion that equal native source implies equal ID.

## 4. Smallest honest engineering task, separately authorized

Propose one bounded, opt-in **owned-candidate round-trip and semantic-key lane
for the existing exact pair decoder**, not the whole owned/state language.
First deliver a reviewed schema/key specification and codec/identity fixtures;
only then, under separate implementation/qualification authority, connect disk
load/store and prove reuse of that same inherited helper. This proposal does
not itself authorize any source edits or runtime experiment. If that authority
is unavailable, stop with this prerequisite open, not with another replay.

Minimum data/correspondence contract:
- Versioned, size/node/depth/integer-bounded declarative data for exactly the
  observed formula fragment: typed parameter/result slots; booleans/integers;
  nested sum/tuple construction, field/variant operations; conditionals;
  approved pure digit observation calls and required arithmetic/bit operations.
  Reject unknown tags, wrong types/arity/variants, invalid parameter/result use,
  unsupported binders, resource overflows and unapproved calls. Do not serialize
  arbitrary VIR, executable bodies, proof evidence or trusted-model authority.
- Rebuild nodes against current typed signature, datatype declarations,
  instantiated type arguments and resolved observation/model definitions. Rebind
  locations/identifiers from current source. Preserve complete Result/error
  value correspondence, not only the Ok byte or a lossy view. This is an owned
  summary despite scalar inputs; substituting an unrelated scalar helper fails.
- Source eligibility must be checked before accepting any hit. Preserve current
  exact-only/signature restrictions (`owned.rs:2228`), no handwritten-contract
  bypass, no assumed/external body, no new mutation/borrow/heap features. Keep
  original executable body, selected theorem domain and exactness obligations.
  Rebuild permissible decorations from validated current structure, or reject
  decorated shapes outside this initial loop-free fragment; never load code.
- Define a deterministic semantic key with explicit compiler/schema policy,
  original typed body/signature/attributes, cfg/features/target/architecture,
  safety/precision settings, checked model/vocabulary definitions, datatype and
  transitive callee dependencies. Remove only demonstrated nonsemantic output
  destinations from lookup identity; do not erase trust, source or safety
  differences. Keep audit paths separately. Do not weaken candidate/proof
  evidence binding to obtain a stable displayed ID. Lookup identity and fresh
  installed proof identity may be separate, with explicit correspondence.
- Candidate cache is untrusted proposal data. A real disk load must be observable
  as such (entry path/hash, decoded semantic digest, hit origin, generation not
  used). Atomic publication and corrupt/stale/partial input handling must not
  turn I/O failure or in-memory reuse into a claimed persisted hit. Existing
  scalar writes ignore errors and retain in-memory entries (`registry.rs:272`);
  their hit metric alone would not establish cross-process disk reuse.
- After installation, freshly register the ACTUAL candidate and check all three
  certificates against the unchanged original body. `infer_proof.rs:43` registers
  fresh evidence; :85 checks identity, :107 resets outcomes for workers.
  `infer_proof/policy.rs:67` requires all three in exact mode. Cache entries must
  never contain proof success, accepted flags, or reusable solver outcomes.

## 5. Required later evidence matrix; none executed here

Future controls require separate authority; frozen/excluded lanes stay immutable.
Every run needs exact argv/env, source/model/runtime hashes,
start/end cache inventories, actual loaded entry bytes/hash, proof outcomes,
full selected-scope/body identities, return code, diagnostics and export status.

1. **OFF / no-cache baseline:** owned persistence explicitly disabled; target and
   exact theorem scope unchanged. No candidate read/write or claimed hit. Check
   all three certificates plus original selected caller obligations. This OFF
   concerns candidate caching, not `--infer-goals`, trust or proof checking.
2. **ON / cold:** new isolated empty directory; genuine generation and miss;
   capture nonempty, valid versioned entry for this inherited helper and the
   generated-vs-decoded correspondence. Three certificates freshly checked.
   A file appearing alone is insufficient warm evidence.
3. **ON / warm / separate process:** terminate cold process; launch a fresh one
   with no seeded in-memory registry, same semantic inputs and that directory.
   Prove an actual persisted read/decoded install for the inherited decoder,
   not regeneration. Record disk key/path/hash, canonical component digests,
   loaded-vs-cold semantic equivalence and fresh installed proof identity.
   All three certificates must run and pass AGAIN. Full selected source scope
   must pass; retain the original 32-body qualified-identity multiset audit from
   V/REVIEW.md:69 rather than relying on collapsed function-name sets.
4. **Identity controls:** vary only export/out paths and process-local state;
   semantic lookup should remain stable if these are proven nonsemantic. Vary
   source body/signature, relevant callee/digit definition, datatype, cfg/target,
   model/trust fingerprint, safety/precision/compiler/schema policy individually:
   require correct invalidation/miss or explicit unsupported refusal, never a
   stale accepted hit. Show which component caused invalidation. No broad
   canonicalization that hides a meaningful difference is acceptable.
5. **Bad-entry/failure controls:** truncated/oversized/malformed/wrong-version or
   key/dependency/type/shape-inconsistent entries fail closed or fall back with
   honest miss telemetry, never proof credit. A separately authorized
   well-typed but wrong candidate must fail the relevant required certificate;
   invalid/skipped/incomplete/mismatched/duplicate evidence must not publish an
   inferred contract. A caller failure after successful helper checks must leave
   crate/interface export blocked. I/O write failure cannot count as persisted
   warm success. Saving untrusted candidate DATA before proof is not saving proof
   success or publishing an accepted interface; choose/document retention policy.
6. **OFF-again / restored controls:** disabling persistence must not read even a
   populated directory; restored identical source must recover ordinary proof
   behavior. These do not replace the required cross-process ON/warm disk hit.

Scalar tests (`registry.rs:699`, :730, :757, :767) were read, not run. They cover
corruption/version/configuration/type/eligibility refusal. Two Registry objects
in one process with a scalar fixture do not qualify owned cross-process reuse.

## 6. Remaining gates and stop boundary

Still required: separate authorization; schema/key/correspondence and code
reviews; new source/build/runtime/loader qualification for a changed engine;
the control matrix above; independent review and an explicit parent decision.
Existing operational91 passes confer none of these or broader native acceptance.

No supplied scripts were executed/imported. No builds, tests, compiler,
verifier, solver, native execution, network, Git writes, parent edits,
subdelegation, cache fabrication, assumption/trust adoption, certification-policy
or theorem-scope change occurred. growth049, zero-growth050, mutation030,
padding95 and drop-pipeline stop/refusal lanes remain untouched. All 18 redb
obligations remain OPEN. No native acceptance. Stop at handoff.

Outputs: this report, `SOURCE_EVIDENCE.json`, `INPUT_MANIFEST.json`,
`OUTPUT_MANIFEST.json`. The output manifest hashes all other output files and
explicitly excludes itself to avoid circularity. Final input reauthentication,
walltime, scratch allocation and free-space checks are recorded there.
