# Original closed tag observation: checked import compatibility

The opaque-observation composition gap is CLOSED with a checked source-level
compatibility interface under the intended standard NonZero value semantics.
The accepted provider keeps `pub closed spec fn value(&self)->u64 { self.0@ }`
exactly. It adds one ordinary checked public broadcast proof function:

```rust
pub broadcast proof fn eq_value_compatibility(a:&BorTag,b:&BorTag)
ensures #[trigger] vstd::std_specs::cmp::PartialEqSpec::eq_spec(a,b)
             == (a.value()==b.value()),
{}
```

The separately compiled original client adds precisely one ghost item:
`broadcast use component::eq_value_compatibility;`. Its entire remaining source
is byte-for-byte the original consumer (SHA256
1b8cdcbab4a22ad98459e402f9b58f09a51552b185ecbd58ca6ef75501f03c7d).
In particular the equality runtime body remains `{ a==b }`; read remains
`{ i.tag().get() }`. No runtime call to get/new is added to the equality client.
No extra requires, Eq-to-identity premise, assumption, axiom, visibility widening,
replacement Eq implementation or new engine/vstd support is used.
The empty compatibility proof body is checked, not external or trusted.
It is explicit manual semantic interface work, not a proof-free reduction.

## Frozen baseline and failed simpler alias

Before experiments, BASELINE.json authenticated the completed original provider
19/0, independent original imported Eq/value failure1/1 (only equal fails), and
lookup-only1/0, including receipt/products/body/helper/native inventories.
The source adapter exactly reproduces the original provider and consumer hashes.
It also binds the independent parent replay receipt
ba75225dcf12c21a8f7c17fcfdef493d1083aa3ed2c865d74740b0a66de476aa.
No broad parent suite was repeated.

A simpler candidate changed value to an open alias solely to the existing
public closed `__verus_infer_nonzero_u64_view()`. Provider19/0 but imported
client1/1: the alias and certified inferred Eq summary are retained, yet Rust
trait equality uses the separate closed PartialEqSpec::eq_spec observation.
No imported compatibility law relates that trait observation to the value alias.
Thus the alias alone is EXCLUDED from acceptance. Its source is fidelity-correct
but does not close the proof gap. That alias couples application semantics to a
hidden-but-public generated symbol's spelling and availability; it exposes no
private field or closed definition. The accepted compatibility interface instead
uses the public trait semantic vocabulary and preserves the original closed
value definition. Its compatibility obligation must be checked again if the
provider's value or Eq semantics change. No equality law follows just from the
unchanged Rust Eq marker.

## Exact checked outcomes

| Frozen lane | verified/errors | Interpretation |
| --- | --- | --- |
| Provider, optional goals on | 20/0 | Original runtime bodies + checked compatibility lemma |
| Same provider, optional goals off | 20/0 | Explicit infer_derive remains active |
| Separate original Eq/value + lookup client, one broadcast import | 2/0 | Intended value semantics, no extra premise |
| Same client with false returned equality relation | 1/1 | Only equal postcondition fails; healthy unchanged provider/Eq |
| Same provider, original client with broadcast removed | 1/1 | Only equal postcondition fails; interface use matters |
| Separate lookup-only client | 1/0 | No broadcast needed |
| Separate MAX/highbit/zero constructor/get clients | 3/0 | Additional ordinary experiments, not changes to original consumer |
| Constant-zero value diagnostic | 16/3 | Original new, original get, compare_equal contracts reject invalid model |

The false relation changes only `r==...` to `r!=...`; all runtime client bodies,
read goal and provider/Eq checks remain unchanged and healthy. It has no
precondition/frontend/resource error. Constant-zero is an INVALID model, not a
valid alternate semantic interface or healthy negative theorem. Its original
Eq inference still passes all three checks; actual constructor/getter contracts
reject the proposed value meaning. Failed crates write no component.vir and
claim no successfully certified native scope.

## Source adapters and complete scope

source.py reuses only the pure pinned source extractor/fidelity checker from
item_tag_constructor_composition/source.py, never its writing controllers.
All BorTag derives remain Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord;
Item's original Copy, Clone, Hash, PartialEq, Eq remain. Genuine private-field
`BorTag(NonZero<u64>)` and `Item(u64)` are exact. Original runtime bodies remain
`NonZero::new(i).map(BorTag)`, `self.0.get()`, and
`BorTag::new(self.0 & TAG_MASK).unwrap()`; no casts replace the constructor FnDef.
Authored original contracts on new/get/tag remain unchanged. In particular
new checks every u64, with Some exactly for nonzero inputs and exact value;
get checks result==value and nonzero over the FULL genuine NonZero<u64> domain.
Value's private standard view meaning is unchanged, directly checked through
these original bodies with inherited standard contracts. No MAX/highbit bound.
Item's INTERNAL wf is low61!=0 with all three high flags arbitrary; neither an
application premise nor a proved native transition invariant is claimed.

Provider's exact 12 ordinary exec bodies are BorTag::clone, BorTag::eq,
BorTag::new, BorTag::get, Item::clone, Item::tag, roundtrip, compare_equal,
lookup, boundaries, BorTag::ctor%, Item::ctor%. The first ten are HIR-native
checked exec bodies; BOTH ordinary generated constructor bodies are mandatory
in VIR and function_details although absent from HIR. One additional ordinary
proof body eq_value_compatibility is checked. Five constant-definition checks
and Eq's two additional precision obligations account for 20 queries in all.
All 25 ordinary signatures/function_details, original/canonicalized body hashes,
and all 30 HIR entries are recorded, rather than counts substituting for scope.

HIR explicitly excludes exactly these seven original derived methods:
BorTag Eq::assert_fields_are_eq, Ord::cmp, PartialOrd::partial_cmp, Hash::hash;
Item Eq::assert_fields_are_eq, PartialEq::eq, Hash::hash. No claim about their
bodies. Copy is a marker without a selected runtime body. Seven HIR spec items
are eq_spec, obeys_eq_spec, __verus_infer_derived_eq,
__verus_infer_nonzero_u64_view, value, word and item_word_wf. Five constant
proxy HIR items are selected_item_without_translated_body; their original
ordinary constant definitions are checked. Other Miri functions (Item::new,
perm/protected, Permission methods, succ, Debug and transitions) remain outside
this component closure, as in the frozen baseline.

Exactly ONE inferred helper: original BorTag::eq, exact value summary,
requires true, result iff generated standard NonZero views equal. ALL original
soundness, inherited-domain maximality and output exactness obligations are
mandatory and PROVED, including controls. No extra synthesized helper.
No infer-goals causal-gain claim: the explicit infer_derive request stays active
in the off lane. Exact native generated Eq expansion evidence inherited from
the authenticated baseline remains `{ self.0 == other.0 }`, body SHA256
1774afdac1844226361985602334fc3b076cb8194e4f4efcc8c0b18fd2eb229b.

The accepted client's HIR contains read/equal checked exec bodies plus the
broadcast expansion `_::__VERUS_REVEAL_INTERNAL__`, explicitly inventoried as a
selected item without translated body, not an omitted proof or runtime body.
There are precisely two ordinary function_details and no local inference helper.
Its import retains certified Eq, generated view, eq_spec, value, word, tag, get
and compatibility proof SIGNATURES WITH NO BODIES. Public open obeys_eq_spec
and item_word_wf remain visible. Unused clone/constructor/new/constants and
provider companion signatures are pruned entirely. RESULTS records every
provider symbol as either retained signature/body fact or entire absence, for
each client; no expectation was weakened to omit generated constructors or
unused signatures. The import dependency fingerprint is present, and full
checking/import/publication barriers remain intact. Lookup-only imports prune
Eq vocabulary entirely as expected. No client sees the private field observation
bodies or compatibility proof implementation.

## Identities, epochs and validation

Base098a8be6; scope/interface commit377d32a0. Immutable accepted bundle is
/tmp/verus-infer-buffer-cycle-20261109/parent-combined69-bundle-3aa7f49a,
receipt2e3fcc58f6fca9364061329104710937ec2aac6aab95a5c48d068c1ca6ee9bb7.
Receipt success/final guards/schema38/policy69/cache72/vstd2250 and ALL eight
artifact hashes are authenticated before each fresh epoch and final audits.
Pinned compiler Rust1.99.1 commit58cf8f9c35b355199727627d07ae34a4e2846d67;
exact compiler, driver, sysroot rlibs and pinned Miri source hashes are guarded.
No compiler rebuild or installed/shared cache/config/toolchain changes.

17 proof epochs are retained in attempts.jsonl. Each privately freezes exact
source and executable-body identities, controller snapshots, expected complete
scope/helper contract, argv/env, tools and input hashes before launch, and
retains exits, raw logs, products and final guards. Initial discovery probes
are explicitly not exact generated-VIR identity replays. All EIGHT selected
final epochs additionally freeze/replay full ordinary VIR body, function_detail,
complete HIR and helper identities derived from completed probes in
identities.json. Path-only S-expression token canonicalization is inherited
from the pure inventory parser; raw hashes also remain in RESULTS/receipts.
Full receipts, copied upstream sources, raw logs/VIR/MIR, exports, binaries and
caches remain private in /tmp. Compact typed inventories/adapters are committed.
No historical checkout reads, old writing drivers, sibling/live parent
inspection, subdelegation, installs, network, push/PR or production edit.

Audits check every epoch product/input receipt, final identity, exact scopes,
all3 Eq proofs, sole control failures, private export barriers and runtime source
fidelity using explicit gates valid under Python -O. A first direct audit
rejected earlier probes because later metadata added broadcast_imports; it had
no durable file log (tool transcript only). The corrected audit compares the
original frozen metadata fields for earlier probes while requiring full schema
identity for final replays. A second, durably logged audit caught an incomplete
client HIR expectation: broadcast use adds the explicit internal reveal marker
above. The audit now checks the exact expanded set, with no entry omitted;
its pre-frozen identities already included that marker. These are controller
validation failures, not verifier or semantic evidence; no proof epoch or old
receipt was changed. audits.jsonl and OPERATIONS.json retain these distinctions.
The initial bwrap RTM_NEWADDR startup failure was handled only with the exact
require_escalated permission mechanism, no host/network modifications.

Recheck completed evidence without rerunning proofs:
`python3 -B -O source/tools/research/tag_observation_exports/audit_run.py`.
This reads only named completed epochs and writes fresh private audit logs plus
owned RESULTS/audits ledger. replay.py requires fresh labels/outputs; it never
executes an old driver or overwrites completed evidence.

## Limits

Manual cost: one authored empty checked broadcast proof body, one client
broadcast import/automatic instantiation; zero local proof blocks, explicit
lemma/theorem calls, bit-vector assertions or snapshots. The accepted consumer
has no new runtime calls. Boundary clients are explicitly new ordinary tests.
Inherited trust includes the pinned Verus/Rust frontend and lifetime/trait/mode/
WF/logic checking, Z3, matching vstd, standard NonZero view/new/get/inverse/Clone,
Option map/unwrap and FnDef specifications, structural derive and trait dispatch.
Standard NonZero implementations are not themselves body-checked here.

This is source-interface component acceptance only. No production/native Rust
safety, physical permission, reference C/F, application or full Item::new result.
Native ordering/OrdSpec and actual From<bool> remain outside this milestone;
no sibling foreign-bool implementation is read or used. Full canonical6/0
remains the parent's existing acceptance, with no new whole-goal claim. Exact
HMAC arbitrary initialized[u8;128], all lawful lengths including0/>128,
both XOR prefix and unchanged suffix remains1/1 UNPROVED. Native0;
44minimal+3phase/101writer effects OPEN; independent whole-goal acceptance and
actual checked automation on both frozen full properties remain required.
Publication=false (private local exports only); goal_complete=false.
