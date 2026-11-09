# Independent bounded production review

## Verdict

**No findings in the seven-file borrowed-byte append production delta.** This is
a bounded review of the sealed **79/73/38** candidate, not acceptance of parent
native80 additions, shared-reader/controller compatibility, broad regressions,
the whole Fedimint application, or storage/durability properties. No fixes or
subdelegation were performed.

The reviewed runtime is the final **build_8** runtime, with the final8 source and
runtime inventories. Pre-hardening observations, earlier builds, parent ongoing
runs, and three review-harness/setup attempts receive zero final acceptance
credit. The implementation's assertion-based replay scripts were read, never
executed, and never written in their sealed directory.

## Reconstruction and identity

- Initial checkout: clean HEAD `3b05c1912b9d7314ad00c832c4065f4362562e02`.
- Accepted dirty cycle79 restored ONCE with `apply_patch`, patch SHA256
  `d3e2b596b63e2c1dc8a4596a373c3abcfb338a3762ed6f1cebae281e7c4a5ceb`.
- All **2,145** baseline file hashes matched both the accepted manifest and the
  candidate's before manifest before applying the delta.
- Candidate delta applied ONCE with `apply_patch`, SHA256
  `58491583d589785f0a65a12171b740dc139aa34e69f1cc32c9e5b0c0ee5049a1`.
- All **2,145** final file hashes and all **seven** changed-file hashes matched;
  all 2,145 were checked again after the fresh probes.
- Cache identity `verus-infer-v79:borrowed-byte-append:4096`, policy
  `candidate-bound-evidence-v73`, schema **38**. Fresh certificate model inventory
  matches the sealed inventory: **380 -> 381**, exactly one new name,
  `alloc::vec::impl&%10::deref`. All 380 common fingerprints changed with the
  freshly bound artifacts; parent compatibility must use the complete inventory.

Raw restoration receipts and hash checks: `logs/restore-accepted-once.*`,
`logs/apply-candidate-once.*`, `restored-2145.json`,
`candidate-before-2145.json`, `candidate-after-2145.json`,
`changed-seven-verified.json`, `final-2145.json`, `candidate-seal.json`.
The original sealed candidate's **3,084** manifest entries also all matched.

## Production audit

The changed source was reviewed against the accepted baseline and the matching
runtime rather than a parent-modified source tree.

- `source/vir/src/infer/owned/byte_append.rs:115`: registration requires the
  canonical alloc Vec path, existing vstd external-spec proxy, core Deref
  method/trait and matching impl, exact generic arity, immutable Vec/slice
  parameter/result types, Exec modes, no body, one existing ensure, exact Sized
  and Allocator bounds, and no unexpected bounds. Both pre-lowering
  `TraitMethodImpl` and lowered `ForeignTraitMethodImpl` are handled.
- `source/vir/src/infer/owned/models.rs:26`: global gates still reject inferred,
  unsafe/async, mutable-user-parameter, authored-requirement and mismatched
  inner/outer-return models. The new enum registration changes recognition, not
  the specification or arbitrary trait dispatch.
- `source/vir/src/infer/owned/byte_append.rs:216`: use requires a resolved,
  nondefault canonical call, matching trait method/impl/type arguments, two
  primitive-byte/Global instantiations, one immutable input and immutable slice
  result, final autospec and non-special call attributes. Calls with bodies or
  post-arguments do not match. Custom allocators and non-u8 inputs cannot enter
  through generic registration.
- `source/vir/src/infer/owned/byte_append.rs:192`: the input must recursively be
  an inert read, not an evaluated-and-discarded expression. Mutable roots,
  destination-root reads, locally bound reference aliases and reference-valued
  field traversals decline. Immutable reference parameters and owned direct
  field/local roots remain the bounded supported domain.
- `source/vir/src/infer/owned/byte_append.rs:295`: the append still requires a
  supported local nonparameter destination. The source's **current full sequence**
  is read from symbolic state, ordered concatenation is retained, the old
  destination object identity is removed, and the new sequence has no stale
  identity. Repeated mutation/readback and clone-after-append probes below check
  this behavior independently.
- `source/rust_verify/src/rust_to_vir_expr.rs:1676` and `:4682`: existing frontend
  auto-deref lowering performs borrow, resolved Deref call and temporary place.
  The fresh provider VIR shows that exact canonical call and impl, not a custom
  Deref surrogate. `probes/native_provider/vir/crate.vir:1525` identifies the
  original vstd proxy and `:7287` records the resolved append-argument call.
- `source/vstd/std_specs/vec.rs:271`: the recognized specification already states
  returned slice view equals input Vec view. Its file bytes are unchanged from
  the accepted baseline. There is **no new vstd axiom**, general Deref theorem,
  arbitrary `T: Clone` theorem, allocator equivalence, or object/reference-identity
  claim. Import/certificate checks and publication barriers remain in place;
  only the policy constant changes in `import_export.rs`.

Soundness/safety, safe-domain maximality, exactness, ordinary caller verification,
normal Rust lifetime checking and successful native compilation remain distinct
requirements. Fresh successful application probes use all of them. This remains
conditional on existing vstd specifications, primitive u8 Clone behavior,
frontend/type/borrow checking, independent Verus obligations and the solver;
allocation success/OOM/capacity are not proved.

## Fresh reproduction and native scope

There are **98 explicitly checked outcomes**, not 98 distinct properties or
native helpers. They include repeated successful export seeding. There were
**101 compiler invocations** total: the three extra harness/setup attempts are
retained, described in `uncredited-harness-attempts.json`, and not counted as
accepted outcomes. All runner acceptance checks use explicit exceptions, never
Python `assert`, and the runners executed under `python3 -B -O`. A deliberate
false `require` also exits nonzero under optimization.

1. **Same complete input, original baseline versus final candidate.**
   `native_before-1` declines with the original static/nongeneric-call diagnostic,
   zero generated functions, and no export. `native_provider-1` uses byte-identical
   source and proves **20/0**, checks eight executable bodies, and infers six
   functions with `requires true`. Every generated function has fresh, required,
   proved soundness/maximality/exactness and positive solver work.
2. **Certified paired import.** `native_consumer-1` imports the fresh provider's
   paired VIR and native rlib, proves **4/0**, checks two executable bodies, and
   infers its composition with the same three fresh required checks.
3. **Exact original native bytes and scope.** Independent qualified-owner/item
   extraction against clean Fedimint pin
   `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236` checks both original `get_full_key`
   methods and `PrefixDatabaseTransaction::get_full_range`. Record declarations,
   private fields/order/types, the token's u32 representation and the original
   `Inner: Debug` versus unconstrained `Inner` distinction remain intact. The
   report scope checks all three original bodies. No original method was
   rewritten beyond adding inference attributes outside its item bytes.
4. **Auxiliary, not extracted native:** `append_outer`, `borrowed_contents`,
   `borrowed_local` and imported `composed` are labeled auxiliary. The provider's
   six inferred functions are the three original methods plus three auxiliaries;
   it also has two authored semantic caller goals. The consumer adds one
   auxiliary and one caller goal. The full generic transaction helper is checked;
   the public contents wrapper deliberately fixes Inner to u8 for import.
5. **Supplied 29-outcome matrix and seven extra seeded controls.** Fresh runs
   retain all requested checks. Reversal, duplication, omission, false caller
   goal and unsafe indexing generate exact candidates and then fail caller
   proofs, removing the real seeded VIR exports. Effectful/block arguments,
   cloned/destination-derived temporaries, mutable input, u16 and custom Deref
   decline separately. Extra controls cover profile alias rejection versus
   ordinary E0502, no-verify, disabled-lifetime alias rejection, native compile
   failure, old-runtime native crate incompatibility and same-runtime stale
   policy-trailer rejection.
6. **Independent probes.** A mutated local source is read twice at its two current
   sequence states; the stale-source goal fails. Clone-after-append uses the new
   destination state; the stale-identity goal fails. Renaming and a u8 Vec type
   alias succeed. A reference-valued field is ordinary-valid/native-compiled but
   declines inference. A System-allocator source with only an external TYPE proxy
   is ordinary-valid/native-compiled but fails the byte/Global coercion gate.
   Generic `T: Clone` declines. Rebound/reborrowed local aliases decline with
   lifetime checking disabled and **without native compile**.
7. **Poisoned inputs.** An exact-key, current-version serialized cache candidate
   with false precondition/true postcondition is ignored by this owned profile;
   it regenerates state and reruns all three checks, and a false caller goal
   still fails and removes the seeded export. Seven current-runtime certificate
   mutations reject schema, producer, the newly added model fingerprint,
   soundness, maximality, exactness and missing trailer, each after seeding a
   genuine consumer export.
8. **Unsupported-valid examples are not soundness failures.** Read-only local
   alias, slice-valued coercion initializer and helper-argument coercion each
   verify/native-compile ordinarily but decline inference with no stronger
   precondition or surviving seeded certificate.

`--no-lifetime` is **not globally forbidden**: fresh valid old/new runtime examples
without native compile still return success and publish. Those observations get
no normal-lifetime application-proof credit. The invalid local-alias inputs are
untrusted-input-boundary tests, not accepted Rust programs or memory-safety
proofs. Pre-build_8 alias behavior is not used for any final claim.

## Audited recorded validation, not rerun

All **1,579** source inventory entries, **1,088** build inputs, **2,145** snapshot
entries, nine runtime files, nine baseline runtime files and **61** toolchain
entries match. The private build's six products match both the build receipt and
sealed runtime; final8 build-source and runtime inventories match their final
counterparts. Both native application inventories match (Fedimint 1,199, redb
132). Authoritative log selection, return codes and both stream hashes match
`validation.json`: **150 VIR**, **96 Rust** (6+9+13+18+17+3+7+21+2), and
**2,221 vstd / 0 errors**. These are audited implementation results, **not suites
rerun by this review**. No Cargo, compiler or vstd rebuild was performed.

## Immutability and excluded concurrent work

The before/after ledger has **13,881** observed paths. The stable **11,736**-entry
immutable subset is byte-and-mode identical before and after, including the
frozen packages, source snapshots, runtimes, pinned native sources and toolchain
inputs. Both stable-ledger files have SHA256
`29e17a928751779d1ddbb8af3ea108388227d94ed74c9528c2ec9dc89b9d5557`.

The remaining **2,145** paths are the explicitly excluded live parent checkout,
not frozen review inputs. Its eleven inherited differences from the older guard
are recorded; its observed file hashes did not change during this review. No
parent broad regression result, native80 addition or reader-compatibility result
is counted as fresh. Parent integration remains in progress and unreviewed here.

Own HEAD and clean native pin remain unchanged; there is no staged delta.
Shared Git refs did change during concurrent activity; exact before/after state
and the difference are retained without attribution. This review issued only
read-only Git commands with sanitized context and optional locks disabled. It
performed no commits, pushes, manual-ref/PR actions, delegation, network access,
frozen-input writes, shared Cargo-home writes or application/toolchain changes.
Only the requested restoration and seven-file delta were applied in this checkout.

## Raw receipts and reproduction

`commands-and-log-hashes.json` binds every subprocess's exact argv, cwd,
sanitized environment, return code, and raw stdout/stderr SHA256. `logs/` contains
the unabridged streams; `probes/` retains every input version, outputs, certificate
mutants and the provider VIR log. `fresh-results.json` binds each accepted outcome
to its input hash and raw invocation. Its SHA256 is
`e7cca3bf5bee6e52aee43348016573823e5a53c7bed2c9a0c530a4af6f0300bb`.

Representative fresh stdout/stderr SHA256 pairs:

- Baseline `native_before-1`:
  `0a6fed164400ecdfce011a0eed3263c0144e7cd05e2b7d5cbe371e00951fcea3` /
  `f46e0546b61148aee05c7e3ecbda8adf2469efd454ddaef89396ae2018b55975`.
- Final `native_provider-1`:
  `00cfa6cccc0d4485de5e9540a586f6206807aeba493f89a7cb08de5e1edd9856` /
  `ec2cc936708de91726bf4ca9a005aee95eee9d855af90797ee1fff5046b23cf1`.
- Imported `native_consumer-1`:
  `ab960b89f5fe5ab44e13c79c35c3abaf8fb6b5526b5dc93bb8c410a4fe097f41` /
  `cd0ec26d8df34894dea08cb7aae27cfa1facb0dc3a2276e6cc1c7e11cde53b7d`.
- Disabled-lifetime/no-compile alias `new_no_lifetime_alias-1`:
  `f2cfe6c165869512db1c0dfbd5b715f66e8ce5e1436972473bc86cc0cf8755e7` /
  `7d1c399e166dd32a4feb0e4c1a4bf6afdacb84842e5195de253a7836cefa67ef`.

Do not execute any writer script inside this sealed review or the sealed
implementation delivery. For reproduction, copy the reviewed runner sources to
new owned scratch or replay an individual receipt there, redirecting every
output/cache/tmp path to that scratch while preserving the runtime, sysroot,
source bytes and paired dependencies. Never rerun `audit.py restore` on an
already restored dirty checkout. The report does not authorize rebuilds, changes
to pinned inputs or replay of parent ongoing work.
