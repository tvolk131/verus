# Current Miri entry: remaining algorithm connection

The published entry handoffs now provide 25 individually closed historical
Stacked Borrows declarations and17 closed Tree Borrows declarations. They are
conditional MODEL results accepted at that scope by independent review
`e03ce8c2`. The fresh optimized-Python parent replay is recorded in
[parent-entry-replay.json](parent-entry-replay.json). The actual compiler
type/layout checks are in [parent-native-entry-type-binding.json](parent-native-entry-type-binding.json).
Neither result establishes incoming runtime permissions or source C/F.

The current-source packed boundary component and seven closed historical
comparison laws now have independent acceptance `a4d43ec9`:
[component record](parent-miri-grant-component.json). The ordinary component
is22/0 with goals enabled and disabled, uses explicit authored contracts,
invariants and five inline proof blocks, and has no measured causal inference
gain. Its internal granting-index/permission precondition still needs proof
from actual lookup; the comparison does not import a Rust proof into Coq.
The same review resolves both permission replay-gate findings and accepts the
actual reference-carrier Scalar/ScalarPair facts. None proves runtime grants.

The active source work is unchanged `find_granting` return guarantees (including
unknown/wildcard provenance and cache paths) and the real NonZero/packed-tag
representation. Pinned package metadata enables stack-cache by default;
[source-policy record](../native_miri_feature_policy/results.json). An uncached
case alone is not the default-path result. The delivered cache prerequisite
checks exact initialization and pair insertion, plus a manual consistency
preservation lemma: [cache component](parent-miri-cache-component.json).
It is17/0 with assistance enabled and disabled; four isolated controls reject.
Independent review `cbb06ae5` accepts that bounded scope after fresh cold/warm/off
proofs and all four strict controls. Full enabled-cache lookup and global cache
preservation remain unproved. Actual cache consistency, lawful
source-context coverage, frame/separation and original C/F remain open.

The genuine `NonZero<u64>` tag lane supplies a bounded original-body subset
(45/0 enabled and disabled) and separate bitfield laws (26/0). Its closure
adapter verifies49/0, but changes `BorTag::new` and cannot establish the exact
constructor interface. All use manual proof assistance with no inferred
helpers. The [tag component record](parent-miri-tag-component.json) binds the
34-epoch retained audit and eight false controls. Independent review `613fe501`
accepts the bounded component after fresh on/off proofs and all eight controls;
the explicit adapter remains separate from exact-source acceptance. Later
integrations below close the constructor and PartialEq subsets; optional checked
addition, bool conversion and ordering remain separate needs.

A provisional frontend extension now checks the original
`NonZero::new(i).map(BorTag)` for every `u64` (2/0 on and off), including an
ordinary checked constructor `FnDef` body. Actual `NonZero::get` roundtripping
checks4/0. The [constructor record](parent-constructor-function-values.json)
keeps this source-only subproof separate from full tag/lookup acceptance.
Independent review `706a4b85` reproduced vstd2250/0 and13 focused regressions.
Its only P2 finding, two stale zero-arity test expectations, is fixed: parent
validation passed68 FnDef tests (one existing ignored test) and13 constructor
tests. Policy/cache reconciliation and parent-wide validation are complete for
the bounded combined capability.
The original `Item::tag` composition is checked below; ordering and
`Item::new` remain unproved.

The provisional equality extension now checks original derived `PartialEq::eq`
over every genuine `NonZero<u64>` tag, with no local proof blocks or proof bodies.
Its standalone result is5/0, with soundness, maximality and exactness all proved;
an ordinary imported client checks4/0. The
[equality record](parent-nonzero-derived-equality.json) binds that evidence.
`PartialOrd`/`Ord` remain unproved. Independent review `f5fe7e41` accepts the
bounded combined capability: cold/warm 6/0, import 4/0, original constructor/get/Eq
composition 10/0, and 23 focused regressions. The
[combined review record](parent-combined69-review.json) binds those results.
Parent-wide regression and canonical/observer validation passed; see the
[combined validation record](parent-combined69-validation.json).
Policy69/cache72 invalidate prior inferred
interfaces/candidates; regenerate providers and dependents with the combined
compiler. Schema38 is unchanged.

The original `Item::tag` composition now checks19/0 on/off with private genuine
carriers, no local proof blocks/calls, and all three required Eq checks. Imported
lookup checks1/0. The [source-component record](parent-item-tag-composition.json)
binds the fresh parent replay and retained evidence. Its internal representation
predicate allows arbitrary high flag bits; no preservation across native stack
transitions or extra application precondition is claimed. Full original
`Item::new` remains36/1 at its protector debug assertion. Actual bool conversion
is0/1; the inspected one-cast MIR and passing local mirror are not a checked
foreign-body bridge.

Imported Eq/value compatibility is now checked with one authored public
broadcast lemma and one client broadcast import. The original closed value
body, private fields and runtime calls remain unchanged. Fresh parent results
are20/0 for the provider on/off and2/0 for the separately compiled client.
The false returned relation fails only its intended postcondition; removing
the import reproduces the original1/1 gap. A constant-zero observation fails
original constructor/getter contracts and is classified separately. The
[observation record](parent-tag-observation-exports.json) authenticates all17
retained epochs and all eight fresh parent outcomes, including complete Eq
obligations and absent imported closed/proof bodies. This is manual semantic
interface work, with no causal automation gain claimed. The simpler public
alias alone remains insufficient.

The next comparison must use the current compiler-matched source in
[native_miri_policy_pin](../native_miri_policy_pin/pins.json), Rust commit
58cf8f9c35b355199727627d07ae34a4e2846d67. The following are source-inspection
facts, not a verified translation or a claim about all lawful Rust callers.

* Current `Stack` stores a bottom-to-top vector and an optional unknown bottom.
  `set_unknown_bottom` clears that vector. In `find_granting`, a wildcard can
  match the unknown bottom; a concrete tag smaller than the unknown bound can
  also match it. Both return `Ok(None)` when no known item grants access.
  Reversing only the known vector into an old-model stack therefore loses an
  entire represented permission case. An empty known vector with an unknown
  bottom is not an old empty stack. Coverage needs a richer relation or a
  justified source unreachability theorem, not an extra client restriction.
* `grant` with an actual access first performs `access` and pushes the new item
  at the resulting top. Its no-access SRW branch instead inserts at the first
  write-incompatible position. Wildcard/unknown no-access cases forget the
  known stack. These branches cannot be identified with one blanket stack-pop
  rule or with the conservative authored Reserved-child model.
* Current `pop_items_after` pops an item **before** invoking its fallible visitor.
  A visitor failure can leave a partially changed stack. Historical pure option
  failure supplies no corresponding final stack. An all-outcomes relation must
  address this distinction; Miri interpreter errors also must not be mislabeled
  as Rust unwinding or silently discarded.
* `Item` is the actual packed `u64`: low61 bits are the tag, two bits are the
  permission, and bit63 is the possible-protector flag. `Item::new` checks the
  tag against `u64::MAX >> 3`. Thus packed-item construction has a61-bit check,
  separate from the wider BorTag counter. Instrumentation failure at either
  boundary needs an explicit classification; no tag-budget precondition may be
  added to the original buffer functions.

The accepted `find_first_write_incompatible` component supplies its own bounds
and first-incompatible contract. The next connection is to discharge its
granting-index and granting-permission preconditions from the actual lookup,
including cache, wildcard and unknown-bottom outcomes. The frozen lookup also
exposes unsupported hasher types, Option and swap
interfaces, and reverse-iterator lawfulness. These are separate frontend or
library proof needs, recorded in
[NEEDS.json](../native_miri_find_granting/NEEDS.json). Any adapter, authored
precondition and unsupported source construct must remain explicit; successful
helper verification is not a full native certificate.

Fresh parent validation covers 624 Infer tests across33 targets,705 compiler
regressions total,148 VIR tests, and148 Python tests in each mode. The canonical
26-case audit and23-case observer replay pass. One native smoke-test launcher
needed rustup first in PATH; its targeted rerun passed and the original failed
batch remains retained.

Full canonical is6/0 cold/warm with no local proof bodies or inferred helpers.
The frozen full HMAC XOR-prefix-and-suffix property remains1/1 unproved in fresh
cold/warm runs. Native=0, all44+3/101 obligations remain open; publication=false,
goal_complete=false. Necessary shared checked work on both full properties and
fresh whole-goal acceptance remain required.
