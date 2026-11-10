# Independent grant, permission-gate, and carrier review

Accept A's bounded mathematical and ordinary-Verus component, B's R1/R2 gate
fix, and C's additive compiler-layout epoch. No new blocking finding within
these scopes. This is not whole-goal acceptance. Native=0; 44 minimal + 3 phase
obligations / 101 writer effects remain OPEN; publication=false;
goal_complete=false.

Review base is `53178f50`. The pretrial milestone is `94c8be61`; the initial
output/assumption audit freeze is `529c8738`, followed by the corrected exact
control-metadata freeze `8fce34bf`. Only this new evidence directory
is edited. No subdelegation, checkout creation, implementation changes,
dependency installation, toolchain/config/cache changes, push or PR occurred.
Historical checkout strings were treated as labels, never opened. New raw
outputs, binaries, source copies and caches are private under
`/tmp/miri-grant-component-review-t3-8295fb02`. No archived raw logs were
reintroduced into Git. Named completed receipts and immutable own Git inputs
were the cross-lane evidence.

## Findings by severity and location

No new P0/P1/P2 finding in the requested A/B/C scope. This conclusion does not
extend to implementation outside that scope or claim missing native bridges.

Resolved R1 (`native_miri_permission_grants/replay.py`, support authentication
in `main`): two own Python support files are mapped to unique original
canonical receipt-key suffixes and checked for identical bytes. The external
bundle separately has exactly the eight required names. The same-byte
freeze004 remapping successfully executes all six cases in this checkout,
without reading any old repository path. `results.json` records both actual
support-key mappings. The canonical success receipt and all eight hashes were
authenticated before the first fresh proof epoch.

Resolved R2 (`native_miri_permission_grants/replay.py`, `validate_component`):
`failed_functions` is exactly the selected grants function singleton for each
negative and empty for each positive. The exact Rust identities and selected
scope are also required. Independent parsing confirms all six actual outcomes,
including their aggregate counts and the sole negative diagnostic at line 33
of the matching case source. The failed crate's globally uncertified body
statuses are not evidence of three failing functions. Two successful queries,
the exact failed singleton and the healthy positive companion establish the
limited Clone-helper claim. The review used static gate inspection and real
ordinary proof runs; no poisoned-output, race or substitution experiment.

## A: source projection and ordinary component

The reviewed integrated component is `f44e2510`, originally worker
`9593b4c7`. `component.rs` SHA256 is
`23880c5435a385228b5bcd35da75a0862bc56a3a2755fa4aa1684a2c18d59092`.
The interface remains the authored internal contract in the reviewed
`INTERFACE.md`, not a new application precondition.

Independent extraction/body-token comparison accounts for all seven selected
methods against the full pinned upstream files. It erases only the explicit
ghost blocks and loop annotations, then compares executable tokens. The entire
candidate template hash additionally binds signatures, constants, declarations,
contracts and adapters. This is bounded source inspection, not a general Rust
parser or a semantic equivalence theorem. Frozen pin/compiler commit is
`58cf8f9c35b355199727627d07ae34a4e2846d67`, Rust 1.99.1.

Item is genuinely the packed `Item(u64)`, permission mask `0x3 << 61`, shift 61,
protected mask `1 << 63`. `Stack::get` retains
`self.borrows.get(idx).cloned()`. No simplified permission vector replaces the
implementation. Every stored u64 is arbitrary; tags, protectors, stack prefix
and suffix have no new restriction. An in-bounds granting index must decode to
Unique or SharedReadWrite. Unique returns granting+1; SRW skips exactly the
contiguous succeeding SRW group. The return is between granting+1 and len.
`set_permission` changes just the permission bits and establishes the requested
decoded permission. `from_bits` requires its internal two-bit range, which
`perm` proves for every u64; no application input restriction follows.

The inspected projection keeps only Stack.borrows and removes unknown-bottom,
feature-dependent cache/unique-range fields and the unused impl lifetime.
Selected methods only observe borrows. `bug!` becomes `panic!` with the same
diagnostic tokens; the internal granting contract makes those arms unreachable.
These are explicit adapters, not a proof of memory-layout or effect equivalence
with Miri. Item/Permission derives are preserved; Structural is added to
Permission, and representation visibility is widened for open specifications.

Complete authored/manual inventory:

* Seven executable methods: Stack::find_first_write_incompatible, Stack::get,
  Item::perm, Permission::from_bits, Permission::to_bits, Item::protected,
  Item::set_permission. Two original derived Clone bodies are also checked.
* Three authored spec definitions: decode, bits, Item::permission. Seven
  executable contracts; authored loop invariants, exit facts and a decreasing
  len-idx measure.
* Five inline proof blocks. The sole named manual library proof call is
  `vstd::std_specs::vec::axiom_spec_len`. No authored proof-function body,
  app-local snapshot fixture, new axiom/assume/external-body stub or generated
  inference helper is present.
* Seven generated Eq/PartialEq/Hash/Debug methods remain outside selected Verus
  scope. Seven constant proxies and the Structural marker have no translated
  executable body. The full compiler inventory is extracted in `results.json`;
  the reviewed `INVENTORY.json` remains the source inventory. Generated derive
  behavior and equality lowering are inherited compiler/Verus trust.

Fresh ordinary checks use the eight-file accepted verifier bundle,
`--crate-type=lib --no-cheating --output-json --multiple-errors 20`, and only
add `--infer-goals` for the enabled run. Neither body filters nor disabling
options are used. Both full-source runs are 22 verified / 0 errors, with nine
checked exec bodies, zero proof-function bodies and complete selected scope.
Normal source, lifetime, trait, mode, WF, capacity/overflow and body checking
remain enabled. Vec length bounds come from the inherited library spec, not an
added caller restriction. Inference requests/functions/assistance and interface
publication are empty: this is a manual component result with no measured
causal inference gain.

The three strengthened-property controls retain the exact original prefix and
append a single control. Each produces 22/1 and only its own assertion failure:
wrong universal Unique decoding (line 155), stopping at granting+1 through SRW
(line 158), and preserving permission bits during an update (line 156).
Independent audit binds the exact failed function singleton and diagnostic
path/span, and rejects mixed frontend, helper, precondition, invariant,
arithmetic or resource failures.

The three appended isolated executable mutants also preserve the original
prefix: Unique returns granting (23/1), Unique decoding becomes SRW (22/1),
and the update shift becomes 60 (25/1). Each has exactly one failed function,
the corresponding mutant; all emitted errors are postcondition diagnostics
inside that function. Boundary and bitfield mutants each emit two postcondition
spans for their single failed query. Original helper health comes from the
identical positive companion and diagnostic isolation, not from treating a
negative crate as globally certified.

Both literal projected Rust and annotated normal macro-erasure builds/runs
exit 0. Their outputs agree on 77,394 finite cases, checksum
4520926185573745023. Binary/source/log hashes are private and indexed. This
finite support is not a theorem over every word/vector and gives no native
source certificate.

## A: historical mathematical bridge

`GrantBridge.v` SHA256 is
`c826e0e40a83777262c5004cd532f796c6542a5486253990dcbefe6310d51ef6`.
Fresh normal Coq 8.9.1 / OCaml 4.07.1 compilation and full recursive coqchk
both exit 0. The ready/completed foundation receipts, maps, source/object
inputs, selected tools and installed Coq libraries/plugins are authenticated
and guarded before/after. Explicit legacy coqlib and `-Q .../theories stbor`
are used. No baseline builder, old worker driver, Coq 8.20 or Rocq 9.2 object
is used.

The complete source/glob/kernel own declaration inventories agree: two
definitions and five theorems, with no extra own helper, constructor,
projection or inductive. All seven are individually CLOSED:
succeeding_boundary, historical_srw_characterization, vector_boundary,
vector_suffix_characterization, above_prefix_reversal,
historical_vector_boundary_equality, granting_index_reversal.
The independent audit hashes the actual full printed types, elaborated terms
and assumptions. Every hash exactly matches the authenticated earlier Coq
receipt; the fresh .vo also matches SHA256
`80b93a24feee221931b3eadc6eabe12363867d1177032245dad20163c08e9c29`.
The complete term outputs include arithmetic/tactic-generated helper
expressions; hashes and exact types/closed status are in `results.json`.

The SRW characterization derives the historical list_find result using actual
lookup/first-incompatible facts, without an assumed result callback. The vector
contract supplies the independently stated boundary predicate. Reversal maps
the suffix above granting to
`take (len-1-granting) (reverse v) = reverse (drop (granting+1) v)`;
the historical removal count is len-r, and the granting lookup index becomes
len-1-granting. Bounds prevent truncated subtraction from hiding an index
error. This is a closed conditional mathematical comparison over historical
items, not an import of Verus/Rust proof terms or an operational translation of
packed Rust stacks.

The recursive checker covers 263 libraries, 41,987 constants and 274 inductive
blocks. That closure is a superset of used helpers, not an exact dependency-use
set. The broader historical foundation separately includes proof irrelevance,
dependent functional extensionality, classical excluded middle, Eq_rect_eq,
Equations Type funext and the upstream admitted FixWf_unfold. Iris sealed
iProp_solution interface fields are also reported as axiom-like in the broader
foundation inventory. These inherited contexts are not attributed to the seven
closed own declarations. No closure claim for unrelated exports follows.

## C: actual parameter carriers

The unchanged query driver SHA256 is
`bb1ecad4890077c842f887fa87c49e323b42d773f27c0107b765a8210c966a13`.
The full trial004 freeze is mapped to this checkout's identical Git bytes.
The four erasure libraries are mapped to the named completed delivered clone,
whose bundle manifest SHA256 is
`aae1f6b375c4b6f558d6cc62d4268925f959105a79bde920dc102884d72a3012`.
No old repository path is opened.

All nine commands have the expected terminal outcomes: version, driver build,
ordinary universal-trait probes and three source queries exit 0; the three
ordinary negative trait probes exit 1 with E0277. Universal [u8;N] and unsized
[u8] trait checks are freshly rerun, rather than inferred from finite array
probes. The PhantomPinned/Unpin, PhantomPinned/UnsafeUnpin and
UnsafeCell<u8>/Freeze negatives reject for their intended trait obligation.

On the pinned 64-bit target, the actual canonical `&mut [u8;U64_LEN]` and HMAC
`&mut [u8;128]` parameters are Scalar, size 8, alignment 8. HMAC's `&[u8]`
parameter is ScalarPair, size 16, alignment 8. Their referents remain separately
classified as memory layouts: byte arrays size 10/128, alignment 1; the byte
slice is unsized. The fresh complete actual-parameter records match the parent
records exactly. These are compiler facts about the actual source signatures.

Independent inspection of the pinned copy_op_no_validate branch confirms that
Scalar and reference/raw-pointer ScalarPair layouts select src_has_padding=false
and read_immediate_raw. The source cut correctly leaves successful carrier
reads/writes, destination validation, partial effects before fallible hooks,
argument protection/retagging, source write_uninit and cleanup simulation open.
Layout classification establishes no memory frame, separation, runtime grants,
source safety, physical permissions or C/F relation. Same-allocation disjoint
interiors cannot be excluded by assuming different allocation IDs.

## Attempts, guards and reproduction

`review.py prepare` authenticated 407 inputs and created same-byte own-path
freezes before any fresh epoch. Only the private grant driver's RESUMPTION
output metadata is changed. The source/controller/helper/bundle identities are
frozen in `pretrial.json`; actual commands, terminal exits, logs and before/after
snapshots persist in the private receipts. The independent `audit.py` imports
none of the delivered acceptance predicates. Its initial 202 completed-product/
source inputs were frozen before its first run. The revised freeze contains
204 inputs, including the retained first audit receipt and exclusion record.

The first independent audit exited 1 because the reviewer incorrectly required
ordinary_contract=true for the two assertion-only controls wrong_permission
and wrong_bitfield. Both intentionally have no contract; wrong_boundary has
requires and correctly reports ordinary_contract=true. This was an independent
audit expectation failure, not a proof or semantic-control rejection. The exact
old source/freeze are reachable in `529c8738`; the terminal and exclusion record
are retained in `audit-attempts.json`. The correction requires the exact three
metadata values rather than dropping the contract check. No verifier/Coq/source
epoch was changed or rerun. The corrected audit exits 0 under -O, with final
guards; `audit-terminal.json` records that terminal result.

All acceptance gates are explicit exceptions/checks; relevant controllers have
no Python assert gates and compile at optimization levels 0/1/2. Delivered
epochs and independent acceptance run under Python -B -O. Final source/tool/
object/product guards are required as well as integer command exits. Complete
private raw-product hashes are bound by the compact committed results.

Reproduction in the original isolated review checkout:

```
python3 -B -O evidence/miri-grant-component-review/review.py prepare
python3 -B -O evidence/miri-grant-component-review/review.py grant-suite
python3 -B -O evidence/miri-grant-component-review/review.py grant-mutations
python3 -B -O evidence/miri-grant-component-review/review.py coq
python3 -B -O evidence/miri-grant-component-review/review.py permission
python3 -B -O evidence/miri-grant-component-review/review.py carrier
python3 -B -O evidence/miri-grant-component-review/audit.py
```

The fixed private root and one-attempt guards intentionally reject overwriting
an existing epoch. New reproduction requires a newly frozen private root and
same-byte own-path remapping, rather than touching these completed products.

Accepted proof/control trials and their exact hashes are indexed by
`results.json`; exact argv persist in its referenced private receipts.
Excluded retained trials are the literal loop-termination frontend baseline
(not a semantic negative), and both no-manual-block runs (14 verified / 3
failed queries with mixed precondition/postcondition/overflow/invariant errors;
causality only). The predecessor RECOVERY attempt inventory is authenticated as
historical metadata, not promoted to fresh proofs. The six initial incomplete
Verus trials, eight Coq elaboration failures and superseded earlier suite stay
excluded. No new resource failure qualifies as evidence.

The first ordinary sandbox read failed before command startup with
`bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted`; reviewed
escalation was used thereafter. Two exploratory reads asked for nonexistent
inventory/manifest names and one guessed place.rs location was absent; these
are read-only discovery errors, not proof epochs. No sandbox/network change
was made. One final-inventory generation attempt incorrectly tried binding
later permission report metadata to the earlier implementation checkpoint; it
is retained as private finalmetadata001.json, then corrected to distinguish
base/report bytes from implementation bytes. No proof outcome follows from
that metadata error. All substantive proof epochs have terminal controller
exit 0. SOURCE_INVENTORY.json binds all 42 current component files to immutable
base Git blobs, with separate implementation checkpoints for selected sources.

## Remaining interface and full-goal boundary

The next actual-source sequent is unchanged: if the real Stack::find_granting
for Write returns Ok(Some(g)) in post-state S', prove g is in bounds and its
decoded permission is Unique or SRW. Concrete, wildcard, unknown-bottom and
enabled-cache paths require justified actual representation/cache invariants,
initialization/preservation and packed tag/search dependencies. This review
does not assume those invariants or prove the real caller meets the internal
component contract. Permission::grants alone does not discharge that sequent.

Fallible visitor partial effects, constructor/tag width, wildcard approximation,
SRW insertion, entry grants/types/ancestors/context, source/carrier separation,
cleanup/unwind, source validity and actual C/F remain open. No assumed defined
future execution, pool_safe, source-safety or no-unwind premise is introduced.

Canonical remains exactly every u64 over an arbitrary initialized exclusive
[u8;10], inherited accepted 6/0 without an accepted app-local mechanical
proof/call/snapshot fixture. Independently frozen bitcoin_hashes 0.14 exact
HMAC remains 1/1 UNPROVED over arbitrary initialized [u8;128] and EVERY valid
key length including 0 and >128, with BOTH XOR prefix and unchanged suffix.
Neither application goal is rerun or promoted here. Shared actual checked
automation, all normal source/inference/publication gates and fresh whole-goal
independent acceptance remain required. Parent owns integration and completion.
