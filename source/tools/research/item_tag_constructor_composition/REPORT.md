# Original Item::tag composition: scoped component proof

The first milestone closes: the original Item::tag body safely unwraps and
returns a genuine private-field BorTag with standard view/get equal to
`w & (u64::MAX >> 3)`, under INTERNAL representation validity
`item_word_wf(w) := (w & (u64::MAX >> 3)) != 0`.
Arbitrary three high flag bits remain allowed. This predicate is not an extra
HMAC/varint client condition, and no native stack transition preservation is
claimed. BorTag remains `NonZero<u64>`, with its entire nonzero-u64 domain;
original new handles every u64, including zero, highbit and MAX. No tag budget,
no-unwind or defined-future premise was introduced.

## Exact results

- Exact new/get/tag composition: **19/0 on and off**, private compile/export.
  Twelve ordinary exec bodies: BorTag Clone, original derived Eq, new, get;
  Item Clone and tag; roundtrip, compare_equal, lookup, boundaries; plus BOTH
  generated ordinary BorTag::ctor% and Item::ctor%. Ten appear in HIR-native
  checked bodies; the two constructors separately appear in original VIR and
  ordinary function_details. Five constant-definition checks and Eq's two
  additional precision queries account for the other seven queries.
- Exactly one inferred helper, BorTag::eq, exact mode, requires true. Original
  soundness, inherited-domain maximality and output exactness are all mandatory
  and PROVED on every accepted positive/control run. Original native Eq expansion
  is `{ self.0 == other.0 }`, hash
  `1774afdac1844226361985602334fc3b076cb8194e4f4efcc8c0b18fd2eb229b`.
  Off omits --infer-goals but still explicitly requests infer_derive; it is not
  an Eq inference ablation or evidence of causal automation improvement.
- False returned tag relation: **19/1**, only false_relation postcondition fails.
  Exact source constructor, get, tag and all three Eq checks remain healthy.
  No precondition failure and no interface written. Whole-run native coverage
  correctly remains uncertified, rather than pretending failed crates succeeded.
- Zero-low-bit word with all high flags set: **19/1**, only invalid_zero_low's
  tag precondition fails. This is an INVALID representation diagnostic, never a
  healthy semantic negative.
- Imported lookup client: **1/0**, exactly read's ordinary exec body, no generated
  helper. It uses the original tag/get contracts and public closed word/value
  signatures; their bodies are absent in imported VIR. Unused original Eq and
  its generated closed vocabulary are pruned entirely. The imported inference
  dependency fingerprint is present. A separate Eq/value consumer fails **1/1**:
  local equality of the two distinct closed observations is not an exported law.
  No imported Eq/value equivalence is claimed by this lane.
- Complete ORIGINAL Item::new, with checked manual packing help: **36/1**,
  only Item::new fails, at its ORIGINAL protector debug assertion. The original
  tag assertion and tag/permission debug checks discharge, and Eq is healthy.
  This is a TRUE-property proof/library gap, not a semantic negative or accepted
  constructor. Larger genuine tags still take the original source assertion
  branch; the trial's bound is only that assertion's normal-return domain.
- Constructor-support subset without Item::new: **36/0**, fifteen HIR exec
  bodies, one checked proof body and both ordinary generated constructors (17 ordinary exec bodies total).
  This independently checks all manual support used by the failed full trial.
- Actual `u64::from(bool)` all-bool property: **0/1**. Ordinary local cast mirror:
  **1/0**. The mirror diagnoses existing bool cast semantics ONLY, not the actual
  std function or a usable native/standard contract.

Exact raw original function paths, expression-byte hashes, canonicalized body
hashes, all function_details, complete native selected/excluded sets, inferred
helper contract/proof data and full attempt ledger are in RESULTS.json.
No count alone, empty helper list or native scope report supplies acceptance.

## Source fidelity and tools

Pinned Miri compiler/source is Rust1.99.1 commit
58cf8f9c35b355199727627d07ae34a4e2846d67. Both complete upstream source hashes
are checked against this checkout's native_miri_policy_pin/pins.json.
The exact combined bundle receipt is authenticated as
`2e3fcc58f6fca9364061329104710937ec2aac6aab95a5c48d068c1ca6ee9bb7`;
all eight artifact hashes are checked before each epoch and in final guards.
Parent engine 3aa7f49a, docs/review base89deb6a7, schema38/policy69/cache72,
matching vstd2250/0. No compiler/vstd rebuild or mutable parent target was used.
The original tag delivery receipt and the accepted combined review RESULTS and
completed composition source are separately hash-authenticated. No old driver
is executed and no historical checkout path is followed.

source.py extracts bodies from the explicitly pinned private source files,
adds authored closed observations/contracts and compares retained executable
body tokens with the originals after erasing INVENTORIED proof blocks. Original
`.map(BorTag)` remains unchanged and resolves to its ordinary checked constructor
FnDef. Item::tag remains `BorTag::new(self.0 & TAG_MASK).unwrap()`. The full
constructor retains ALL executable tokens: original assertion, u64::from(bool),
all three derive comparisons/debug assertions, and return. BorTag/Item fields
and all constants remain private. Permission's Structural marker is the only
extra marker; derives otherwise remain original. Other methods/Debug impls,
succ/ordering and unrelated Miri module bodies are intentionally outside closure.

Before final trials, identities.json binds all raw body/helper/function_detail/
native identities in the per-epoch plan. Raw VIR expression hashes retain spans;
canonical hashes tokenize S-expressions and replace ONLY this epoch's directory
with @EPOCH@, preserving quoted strings and executable identity while removing
pretty-printer wrapping differences. Full exact sources and controller copies,
argv/environment, exits, stdout/stderr, VIR, exports, private caches and products
remain in private /tmp epochs. Final guards bind immutable inputs throughout.
Normal source/lifetime/trait/mode/WF/body checks and --no-cheating are used;
no selected executable body is replaced with an unchecked contract.

## Remaining standard source bridge

The actual rustc_private-resolved core From<bool> instance is inspected through
this pinned compiler, including signature, source, all instantiated MIR locals,
blocks/statements/terminators and drop facts. It is a single Bool->U64 IntToInt
cast followed by Return, with no cleanup/drop/memory/call operations in the
instantiated foreign body. See proposal.md for exact instance/DefId/fingerprint,
typed mapping and mandatory translation, registration and proof obligations.
Installed core source and actual sysroot rlibs are separately hash-bound; no
independent source-to-stdlib rebuild equivalence is asserted.
This lane implements NO foreign-body frontend, production change or new law.
No FromSpecImpl<bool>, obeys_from_spec=true, assume_specification, axiom or
external_body was added. Actual From's inherited conditional specification does
not establish the bool conversion law here. Primitive MIR shape and the passing
local mirror do not close this gap or certify reference/provenance lowering.
Full Item::new remains UNPROVED.

## Retained failures and operational limits

attempts.jsonl and RESULTS retain every proof/build/inspection attempt, including:
missing rustup PATH setup exit128; two boundary probe forms whose bitfield
simplifications were not automatic; early mixed false/invalid runs with unhealthy
boundary companions (excluded from semantic acceptance); unassisted full source
failures; a private-constant visibility frontend error in an initially public
packing lemma (fixed by keeping the lemma private); a frozen-identity controller
rejection caused solely by path-length-dependent pretty-print wrapping (fixed by
S-expression token canonicalization, with raw hashes retained); the imported
rlib filename frontend failure (fresh provider uses libcomponent.rlib); and the
closed-observer Eq import gap. A first final audit incorrectly required unused
pruned imported functions to retain signature nodes; it was corrected to record
both complete absence and signature-without-body separately. Audit failures are
validation/controller failures, not verifier/semantic evidence. A later audit
correctly rejected a changed proposal document that early epochs had not copied;
the exact frozen document was restored from the successful audit snapshot, and
the added qualification lives in this report instead. No resource
failure, compiler panic or failure of a true property is counted as a negative.

The normal shell sandbox failed before launch with bwrap RTM_NEWADDR; the exact
require_escalated tool mechanism was used. No host/network settings changed.
One instruction search mistakenly began `find .. -name AGENTS.md -print`; it
was interrupted with Ctrl-C, exit130, before any output. This command exceeded
this checkout's intended traversal boundary; absence of output does not prove
no sibling metadata was visited. No sibling contents, builds or processes were
returned or used. This is an explicitly disclosed operational scope deviation,
not a source-proof claim. Subsequent reads stayed within this checkout/history,
immutable Git objects and explicitly named completed /tmp artifacts. No
subdelegation, process-table query, install, push/PR, production/engine/vstd/
policy change or shared toolchain/cache/config write occurred.

Only compact adapters, typed evidence, manifests and reports are committed.
Copied upstream sources, raw MIR/logs, full receipts, binaries, caches and
exports remain PRIVATE. The ledger gives exact retained paths and receipt hashes.

## Reproduce/audit

Use `python3 -B -O source/tools/research/item_tag_constructor_composition/audit_run.py`
to audit the completed explicitly named private evidence without rerunning proofs.
Explicit RuntimeError gates work under -O. Fresh proof epochs can be generated
with `python3 -B -O .../epoch.py LABEL tag` (or tag --off, false, invalid,
full-help, full-helpers). Each creates a fresh private temp/cache/output and
freezes exact source/tool/controller/body/helper identities before launch.
clients.py is the narrow separately imported lookup client controller.
The earliest bool probes put log-all products outside their epoch trees; their
raw logs were retained there and are now additionally hash-bound by retention.json
and its PRIVATE supplemental manifest. This posthoc retention is not a pre-epoch
acceptance claim; the final bool probes use fully retained per-epoch log trees.
Manual/trust inventory is in MANUAL.md; the target types are in INTERFACE.md.

Publication=false (private exports were written); native correspondences=0;
goal_complete=false. No component, kernel or MIR hash proves source safety,
physical permissions or original reference control flow. Canonical all-u64 /
arbitrary initialized exclusive[u8;10] fresh6/0 acceptance without app-local
mechanical proof/calls/snapshots remains the parent goal, not established here.
Exact bitcoin_hashes0.14 HMAC on arbitrary initialized[u8;128], EVERY lawful
keylen including0/>128, BOTH XOR-prefix and unchanged suffix remains1/1 UNPROVED.
Native0; 44minimal+3phase/101writer effects OPEN; whole-goal automation/acceptance
and independent integration remain pending. Parent's full33Infer/VIR/canonical26/
observer23 pipeline was neither inspected nor changed nor polled.
