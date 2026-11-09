# Independent read-only review: hex-conservative 1.3.0 privacy repair

Date: 2026-11-07, America/Chicago.

## Bounded verdict

**The frozen evidence supports the privacy-preserving, ordinary authored
single-byte encoding repair on the qualified operational91 runtime. It does
not satisfy a strict persisted warm-candidate/cache-hit prerequisite. No final
acceptance, retention, integration, engine acceptance, or trust adoption is
authorized by this review.**

I found no additional defect in the new encoding definition, its unrestricted
helper domain, or the preserved executable statements. There is one blocking
acceptance gap and one non-blocking preservation-oracle weakness below. The
latter is independently compensated for this particular evidence set: all 32
historical body rows, including both overloaded conversions, are present.

This is an engineering/proof-evidence review, not an independent proof replay or
a mathematical proof certificate. No builds, compilers, tests, verifiers,
solvers, network operations, Git writes, or subdelegation were performed.
The supplied harnesses were read, not executed. Output is confined to this new
review scratch directory. The parent decides retention and integration.

## Ranked findings

### 1. HIGH / acceptance blocker: no persisted warm candidate was exercised

Evidence: input `invocation-05/invocation.json`, `invocation-05/result.json`,
`invocation-06/invocation.json`, `invocation-06/result.json`, and the raw
`verus-infer.functions[0].metrics` objects in both stdout files.

- Invocations 5 and 6 use exactly the same `VERUS_INFER_CACHE` directory,
  `<input>/cache-restored`.
- Both recorded before/after file inventories are `{}`. The directory is also
  actually empty at review entry and exit; it is not a populated candidate
  cache that the reviewer can infer was exercised.
- The sole inherited helper, `iter::hex_chars_to_byte`, reports `cache: miss`
  and `cache_policy: regenerated_state` in both runs. Its three required
  certificates are all `proved`; solver times are 5382 and 5631 microseconds.
- The matched source in `source/vir/src/infer/registry.rs:353` regenerates the
  owned-summary lane and does not take the scalar persistent-cache read/write
  path. The reporting policy is explicit in
  `source/rust_verify/src/infer_report.rs:209`.

Thus invocation 6 is a valid rechecked positive and a same-directory replay,
**not a persisted warm-candidate hit**. Passing proof rechecks do not discharge
the additional cache prerequisite. `RESULTS.json` and the input report disclose
this correctly; the `restored-warm` label and
`required_inference_certificates_rechecked_in_warm_run` field must not be used
in isolation as warm-cache acceptance.

There is also no stable candidate-id identity demonstrated across 5/6:
`a77449b1ab80e0c8b7ebd21c02e19e5c6670b6d9081c1cf029d0818147b7bfef`
versus
`4916a7c587241ec6f58094480600bd5de9209ee1aebc39458dcff10d471f30ec`.
This is an observation, not a diagnosed key-generation bug, and is not needed
to establish the empty-cache/miss finding.

Minimal followup: leave the strict cache gate **unmet/open**. If the parent
requires that gate, any different runtime/cache mechanism and its qualification
need separate authorization and evidence for the required inherited candidate,
including nonempty persisted state, its actual reuse, and all three rechecked
certificates. Another identical regenerated-state replay cannot supply that
evidence. Do not fabricate entries, substitute an unrelated helper, or
retroactively waive/redefine the prerequisite to accept this record.

### 2. MEDIUM / audit hardening: function-name sets collapse distinct bodies

Evidence: input `audit_repair.py:96` constructs `old_checked` from only
`item['function']`; `audit_repair.py:126` constructs the current set the same
way, and line 127 checks set inclusion.

The historical scope contains **32 checked executable rows but only 31 unique
function labels**. These distinct implementations share the reported label
`hex_conservative::error::DecodeVariableLengthBytesError::from`:

- `From<InvalidCharError>::from`, native `src/error.rs:150`.
- `From<OddLengthStringError>::from`, native `src/error.rs:157`.

The current 36 rows similarly have 35 unique labels. A missing overloaded body
could escape the set-inclusion oracle if the same-labelled overload remained;
the total-count check is not a substitute for per-body identity.

This does **not** show a missing body here. I independently matched all 32
historical rows in each of invocations 3, 5, and 6 by `rust_function` plus
source-relative file/span, mapping original lib.rs line numbers through the
insertion-only delta. Both overloads match individually. All archived files
other than lib.rs are byte-identical, including error.rs. The detailed matches
are retained in this review's `independent-evidence.json`.

Minimal followup: in a separately retained audit layer, compare multisets of
fully qualified Rust implementation identity plus normalized source span, and
assert the original 32-row cardinality. Do not edit the frozen input harness or
rerun verification just to repair this documentary oracle.

## Source and semantic review

References below are relative to the authenticated input unless identified as
matched parent/compiler sources.

### Closed View is a definition, not a new assumption

`native/src/lib.rs:400` retains `pub(crate) struct Table([Char; 16]);`: its tuple
field remains private. The cfg-gated View implementation is in the original
`table` module. At line 407, `closed spec fn view(&self) -> Self::V { self.0@ }`
has a concrete body and `V = Seq<Char>`. There is no replacement representation,
uninterpreted declaration, external body, assumed contract, new invariant, or
axiom in this insertion. The private field is projected only within its legal
module boundary. Consumers use checked contracts on the abstract view rather
than constructing or opening the private field across the boundary.

The raw scope classifies this item as
`specification_definition_or_declaration`, not `checked_exec_body` or
`checked_proof_body`. That is appropriate for this body-defined spec function;
the implementation must not inflate body-proof counts with it. Existing
ordinary Verus semantics and imported library models remain the trust base.

### Both actual exec constant initializers are retained and checked

At `native/src/lib.rs:415` and line 426, the original LOWER and UPPER constants
are selected with ordinary canonical-table postconditions on `Self::LOWER@`
and `Self::UPPER@`. Their original arrays and all Char discriminants remain
unchanged. Independent initializer/signature text hashes are:

- LOWER: `754c3561c3083f2911913a2dc9c1e7ee7b8535d7d413d1082cef153653bb1abb`.
- UPPER: `63a6a752241472d9ed2dc9b1e3bb430235cd7a1863e3a3ea7442e91d84eb105e`.

Matched compiler source `source/builtin_macros/src/attr_rewrite.rs:640`
rewrites an annotated constant with ensures to Exec mode with its **original
initializer expression** as the block. Matched
`source/rust_verify/src/rust_to_vir_func.rs:3466` lowers the actual body and
postconditions, rejects requires/recommends, and retains `Some(vir_body)` when
there is no external-body attribute. The ordinary associated-constant test at
`source/rust_verify_test/tests/consts.rs:470` also documents a wrong initializer
failing its ensures; I inspected this source but did not execute the test.

The raw positive `func-details` contain the translated
`hex_conservative::table::impl&%0::LOWER` and `...::UPPER` entries, and the full
selected runs succeed. The native scope's proxy rows themselves say
`selected_item_without_translated_body`: **those rows alone are not body-proof
receipts**, and the constants are not added to the 36 named exec-body count.
The const conclusion rests on the actual source/translation and successful
full-scope evidence, not a fabricated opaque-constant summary. No direct
initializer-mutation negative was run; the retained negative is the Case branch
mutation described below.

### Independent ASCII property covers the complete stated domain

`native/src/encoding_spec.rs:5` specifies arithmetic ASCII policy: below ten,
48+n; otherwise 87+n for Lower and 55+n for Upper. It does not call the
implementation, read either actual table, or copy an alphabet as its oracle.
`required_encoding` uses integer division/remainder by 16 for the high/low
positions. The four assertions at line 37 prove the shift/mask bounds and their
equality to machine division/remainder by bit-vector reasoning.

The selected executable goal at line 43 takes an arbitrary `u8` and `Case`,
has no preconditions, and calls exactly
`case.table().byte_to_hex_chars(byte)`. These symbolic arguments range over all
256 legal bytes and both enum variants; this is not a sampled test.

`native/src/lib.rs:462` gives the general helper two unconditional selection
equations against the actual Table view. There is no new `requires`, no
canonical-table entry restriction, and no replacement or narrowed Table type.
Mixed-case, reordered, and repeated legal Char entries remain in-domain.
Canonical ASCII conclusions are implications conditional on the actual input
view satisfying the appropriate canonical predicate. The source does not
assume that all Tables are one of the two canonical constants.

### Executable statements, signatures, privacy, derives, and history retained

I independently read the authenticated historical tar archive in memory. It
contains 53 regular files and no link/special-file members. The final native
tree contains those 53 plus only `src/encoding_spec.rs`. Only existing
`src/lib.rs` differs; its complete diff consists of insertions. I independently
reconstructed `final-source.patch` exactly, including its new-file section.

The inserted module, imports, selection/specification attributes, View
implementation, and proof call are ghost/cfg-gated. Original derives and
original Table/Case/function declarations remain. After removing exactly the
cfg-gated proof call from `byte_to_hex_chars`, both original function signatures
and bodies compare byte-for-byte:

- `Case::table` original and erased:
  `d69f978e683f95b7dcc1867162e3df26328d52cea011d60eb36468f5e2ad7a42`.
- `Table::byte_to_hex_chars` original and erased:
  `9fa3ac9d8e0a7339cbd4ea60ea7d1037d857a691c48ae14a25e21960aaa8756c`.

This is a textual/structural erasure audit, not a fresh native compilation or
ABI claim. All other archive files, original requirements, and historical
selected-body source text are unchanged.

## Raw proof evidence, controls, and trust boundary

I crosschecked raw stdout JSON, raw stderr, result return codes, stored
stdout/stderr hashes, source snapshots, and pre-run/result source-hash maps.
The summaries in `validation-receipts.json` are consistent with the material
reviewed. Every retained invocation has the same native cfg set and whole
selected-scope invocation shape: std, alloc, `verus_hex_vec_phase3`,
`--infer-goals`, `--no-cheating`, `--output-json`, one worker, explicit original
sysroot, explicit export and out directory. There is no verification filter,
no-verify, no-lifetime, compile invocation, or application substitution.

- Invocation 1: return 1, associated-constant name resolution errors; zero
  checked bodies and no export. Invocation 2: return 1, exec-constant use in
  unrelated specs rejected; zero checked bodies and no export. Neither is
  positive proof credit.
- Invocations 3, 5, and 6: return 0, **48 verified obligations**, zero errors,
  `complete_selected_scope_verified: true`, **36 checked named exec bodies and
  one checked proof body**, interface written and an actual 376089-byte VIR
  export in each separate invocation directory. This is not 48 source bodies
  and not every Rust body in the crate.
- All 32 historical executable body rows match individually in each positive.
  The four added exec bodies are Case::clone from its preserved native derive,
  Case::table, Table::byte_to_hex_chars, and canonical_byte_encoding_goal.
- Exactly one inherited Infer helper remains. Its original exact contract is
  identical to the authenticated historical restored-case receipt. Soundness,
  safe-domain maximality, and output exactness are each required and proved in
  every positive. The helper is available to same-crate callers, not exported
  as a public Infer helper. A successful crate VIR export is a separate fact.
- No new encoding function is an Infer-generated proposal; they are authored
  ordinary exec/proof/spec code. Goal assistance is empty. Existing assistance
  remains one sequence-pattern site, one error-callback site, two array-callback
  sites, and four type-invariant-entry sites. The inventory agrees with all
  three raw positive runs; it is not silently counted as a helper certificate.
- The 382 current imported function/fingerprint records agree exactly across
  all positives and with `imported-model-inventory.json`. The historical
  engine's inventory has 291. These are different qualified runtime contexts,
  not falsely identical trust inventories. Backend and imported-spec trust
  remain explicit; no new assumed specification or trust rule is adopted.

The historical baseline was independently matched to the `restored` case and
its command inside authenticated `inputs/historical-v3-receipt.json`, not
accepted solely because its separate baseline filename says historical.

### Semantic negative is genuine, localized, and restored

Invocation 4 differs from the final positive lib.rs in exactly the original
Lower branch: `&Table::LOWER` becomes `&Table::UPPER`. Its encoding_spec.rs and
all other native source hashes are unchanged. This violates the independent
Lower ASCII contract, e.g. uppercase A has ASCII 65 rather than required 97
at nibble 10; no special source restriction makes that distinction unreachable.

Raw stderr at line 81 reports `postcondition not satisfied` at
`canonical_table(result@, self)` in `Case::table`, showing the changed branch.
Raw JSON reports 47 verified obligations, one error,
`encountered-vir-error: false`, `caller_proof_failed`, `crate_success: false`,
and `interface_written: false`. Return code is 1, no timeout is recorded, and
there is actually no VIR export. This is not a parser, privacy, unsupported
feature, or lifetime failure. Locally proved inherited certificates in the
failed run do not confer crate/interface acceptance or positive proof credit.

Invocation 5, invocation 6, invocation 3, and the final native tree have exactly
the same complete 54-file source-hash map. The original branch was restored,
not just asserted restored in prose. The negative exists only in its snapshot
and explanatory patch.

## Authentication and provenance

Input root: `/private/tmp/verus-hex130-privacy-repair-20261107-zm5hKz`.

At **both entry and exit**, independent enumeration and hashing authenticate
all **143 listed file byte lengths and SHA256 hashes**, plus the pinned
manifest itself: **144 regular files total**, zero unlisted files, zero
symlinks, zero special files. The same exact path set is present at both
boundaries. The final report handoff requires these checks to pass.

- `output-manifest.json`, 19985 bytes:
  `3ef7d0e40cf0150e40ee071bfd1629e8f440f5246b42633708ca80206f5c8d62`.
- Input `REPORT.md`, 18232 bytes:
  `ac72b226b919c8eb6923b7d35ace2e8d2dbc30221f5ce5a1263db576f9bc09db`.
- Final `native/src/lib.rs`:
  `41d8e48a89103fd0fb018e81a5c2af13e75c8d0eb96eba677e33f40742869715`.
- Final `native/src/encoding_spec.rs`:
  `c9369b01b2fdcc238dc3120670ac8c13ce7b245193fe0158a17a1d6d5a980c45`.
- Authoritative parent `evidence/verus-infer-hex-v3/native-source.tar.gz`,
  76834 bytes:
  `e7f4b63eced039d437b54858fc600071b41614aa66b2a9fdc27caa94c9e0de5b`.
- Original archived lib.rs:
  `c5d584fb99c620c8e086c970afcf14b36fa4bb8838d9ead1a415ef3142c67073`.

All nine source files named by `source-investigation-bindings.json` were
hashed before relying on them and agree with the frozen compiler-source
manifest, not just with the research document's assertions. This does not
authenticate the entire current parent as a new accepted engine.

The 15 supplied runtime authentication receipts have consistent runtime,
qualification/build, loader, rustc, and parent bindings, including pre/post
receipts for each of the six invocations. I also independently hashed the
currently referenced nine runtime artifacts, 30 loader libraries, rustc, and
eight qualification artifacts: 48 unique paths match their recorded lengths
and hashes. No artifact or toolchain was copied or executed. This validates
the evidence bindings; it is not a fresh build or independent reproduction of
the historic runtime. The original matched loader/sysroot and frozen
operational91 limitation remain essential.

I authenticated all 11 preserved-input bindings. I additionally checked the
old failed95 output manifest's **92 listed lengths and hashes**, both in its
old scratch and the sealed parent RAW-HANDOFF archive, whose 93 regular files
include that manifest. Its manifest SHA256 is
`2cdabfadce92408867a8babda4f200d8c781a62b5e8e16bf8f9fd0f3e9367c8f`.
The sealed archive matches its preserved binding
`158c00719e26f3752f373aecc1d66c0492d42a76b75bf69bd29fb88b7675dd01`.
The parent failed record still says
`NOT_ACCEPTED_FRONTEND_VISIBILITY_BLOCKER`, one failed invocation, zero
checked exec/proof bodies, and zero proof credit. It remains failed and was
not edited or relabelled.

## Minimal parent followups and stopping boundary

1. Keep strict persisted-warm-candidate acceptance explicitly incomplete;
   decide any separately authorized next lane without converting this replay
   into a cache hit or treating this review as final acceptance.
2. Preserve this independent row-level retention audit, or add a separate
   qualified-identity/multiset oracle before relying on the original
   name-set check in future work. Frozen evidence stays immutable.
3. If retaining the repair, retain raw invocation evidence, exact source/patch
   and manifests, model/assistance inventories, the frozen runtime
   qualification, and the original failed95 record separately. Do not install
   or adopt source/trust changes on this review's authority.

There is no current-parent, historical-engine, clean-checkout, whole-native-
crate, iterator/vector-encoding, other-target, or other-version acceptance.
This review closes no redb task: **all 18 remain OPEN**. No S0, admission,
import/export, or ordinary full-selected-scope barrier is waived.

Review-generated receipts are in this directory: `authentication-entry.json`,
`authentication-exit.json`, `source-bindings-authentication.json`,
`preserved-authentication.json`, `preserved-failure-review.json`,
`runtime-readonly-authentication.json`, and `independent-evidence.json`.
`handoff.json` supplies this report's byte count and SHA256. Free space stayed
above 16 GiB at checked boundaries; only small review artifacts were written.
Stop at report handoff; the parent decides all retention/integration actions.
