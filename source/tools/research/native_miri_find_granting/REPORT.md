# Result: bounded cache milestone; granting caller still OPEN

Early interface/source selection: e57ea436. Checked component checkpoint: cf61f45f.
Base: 53178f50. Only this directory changed. No native correspondence, application
acceptance, publication, push, or PR. Final local acceptance is successful under
Python -O; fresh whole-goal independent acceptance remains the integration lead's
obligation.

## Accepted implementation result

Final epoch: `/tmp/native-miri-find-granting-20261110-impl-final008`.
`output/receipt.json` SHA256
`297c67eae1dc1242ad0c1f08175548fc16b2167f6c267fdf8b0a5e15ef1e0075`.
The named completed directory contains all argv, terminal exits, raw stdout/stderr,
product hashes, pretrial freeze, final guards, read-only acceptance output and its
terminal record. Raw outputs are private and are not committed here. Authenticated
own helper copies in `completed-own-helper-snapshot` were made **after** completion
and checked against the frozen pretrial digests; their copy time is not pretrial.

The enabled-cache projection proves these ordinary internal statements:

1. Exact current `StackCache::add`: head item/index becomes the supplied pair;
   every slot 1..31 equals the corresponding prior slot 0..30 in both arrays.
   Arbitrary `usize` index and arbitrary packed `Item(u64)` are accepted by this
   method. The original `copy_within` operations and all bounds checks remain.
2. Authored proof `add_effect_preserves_consistency`: if all prior 32 cache pairs
   match an arbitrary borrow sequence and the added pair also matches it, then
   all 32 resulting pairs match that same sequence. This is a structural theorem,
   not a successful-lookup or future-execution assumption. It is a checked manual
   lemma about the effect actually established by (1), not an unchecked summary.
3. Exact current `Stack::new`: borrow vector is `[item]`, unknown bottom is None,
   **all** cache pairs match that vector, and the original `unique_range` is 0..1
   exactly for decoded Unique, else 0..0. This does not establish Item tag validity.
4. Rechecked exact packed permission decoding through `Item::perm` and
   `Permission::from_bits`, using the accepted prior component's proof pattern.

Cold, warm and assistance-off results are each **17/0**. The selected local scope
is **7 executable bodies + 1 proof body**, not 17 executable bodies. There are 4
spec definitions and 8 selected auxiliary items (20 selected items total).
`INVENTORY.json` records every selected and excluded local item separately. Inferred
helper sets are empty; status is `no_inference`. Assistance has **no causal gain**.
The component is manually annotated and is not the required final automation.

Four strict controls reject at **16/1**, with exactly one named failed function:
wrong head index and shortened index shift -> `StackCache::add`; bad initialization
index -> `Stack::new`; false preservation property ->
`add_effect_preserves_consistency`. Original other helpers remain healthy in the
positive baseline and all other queries succeed in these controls. Failures are
postcondition-only, with no frontend, bounds, precondition or resource rejection.
The three source mutations also fail exact body fidelity at their intended body;
the property mutation preserves all original executable bodies. `accept.py` binds
each negative source to its exact intended canonical transformation.

## Exact source and adapter inventory

Primary source commit is Rust 58cf8f9c35b355199727627d07ae34a4e2846d67.
`SOURCE_FREEZE.json` froze the exact four lookup/cache methods before the first
trial; each epoch additionally froze full candidate, source, controller, helper,
and eight-file accepted-verifier identities. Canonical receipt success and all
8 bundle SHA256 values are authenticated. Final epochs also freeze the selected
rustc binary, rustup binary, and rustc driver. Toolchain dynamic-linking, standard
Rust library and operating-system behavior remain inherited trust.

`fidelity.py` compares exact executable tokens for `StackCache::add`, `Stack::new`,
`Item::perm` and `Permission::from_bits`, removing only ghost proof blocks and
selecting the original stack-cache cfg. Token counts are 52, 66, 18 and 58.
No indexed replacement lookup, vector algorithm, set, or tag representation was
substituted. Accepted representations are genuine Item(u64), BorTag(NonZero<u64>),
Vec<Item>, both 32-entry arrays, Option<BorTag> unknown_bottom, and Range<usize>
unique_range. No fields read by the methods are silently erased.

Adapters: `verus!` namespace/imports; chosen cfg compiled into the cache component;
unused `'tcx` impl parameter omitted; method visibility reduced for private internal
contracts; return binders, specs and contracts added. Permission adds Structural.
Original Stack/StackCache Clone+Debug derives and Item's custom Debug implementation
are outside this projection; no claim is made about them. BorTag, Item and Permission
Clone bodies are selected and checked. Derived Eq/PartialEq/Ord/PartialOrd/Hash/Debug
items automatically ignored by this verifier are explicitly outside checked-body
scope, not proved as Rust implementations. Genuine BorTag declaration alone does
not establish its hash/equality/order model.

Manual inventory: one proof block in Item::perm, containing two bit-vector
assertions establishing mask/range; one authored proof function with a quantified
case split for cache preservation. No new axiom, assumption, external-body stub,
unsafe block or no-unwind assumption was added. Standard assumed specifications
for Vec/arrays/slices, overlapping copy_within, ranges, Option, NonZero Clone and
Rust arithmetic are inherited TCB. In particular the standard implementation of
copy_within itself is not proved here. Standard assumptions, including their own
unwind clauses, do not establish a native Miri no-unwind theorem. No Coq work or
runtime finite comparison was used in this lane.

## Original caller remains unproved

The actual literal enabled-cache fixture retains all Concrete/Wildcard branches,
unknown-bottom tests, actual exposed FxHashSet membership, reverse searches,
32-slot cache position, early `?`, swap/reinsert and all return outcomes.
With assistance on or off it fails frontend checking at the same four operations:

- `NonZero::new(i).map(BorTag)`: constructor as a function value unsupported.
- Actual `rustc_hash::FxBuildHasher`: unsupported external type.
- Actual `Option::is_some_and`: missing specification.
- Actual slice `swap`: missing specification.

No new external specification was used to suppress these failures. The diagnostic
constructor translation `.map(|nz| BorTag(nz))` checks with genuine NonZero, but is
explicitly a changed executable syntax, not original tag-constructor acceptance.
It is excluded from lookup acceptance and no unchecked Item::tag contract is used.
A translated no-cache tagged-search baseline still fails Item::tag unwrap,
Permission::from_bits unreachable and generated loop obligations; its initial
runner label incorrectly said frontend because `encountered-error` also marks
proof errors. `ATTEMPTS.json` retains that label and records corrected classification.

Two additional isolated proof-interface diagnostics fail under ordinary checking,
with the existing standard iterator broadcasts enabled: asserting lawfulness of
`v.iter().enumerate().rev()` (0/1) and an empty for-loop over that expression (1/1).
The latter reports generated loop invariant/decreases failures. These are support
obligations, **not** semantic negative controls or a defect theorem about Miri.
Inspecting the own-checkout standard rev postcondition suggests missing linkage
between the returned Rev's inner iterator/lawfulness and its input; the diagnostic
is the evidence, and a complete support proof is still needed. No replacement
indexed loop was installed.

`SOURCE_POLICY.json` authenticates the explicitly routed completed source-inspector
receipt and primary Cargo.toml/fx.rs artifacts. Package defaults are exactly
`[stack-cache, native-lib]`; expensive-consistency-checks implies stack-cache.
Actual build flags were not observed. The Fx reexport is genuine, but underlying
alias/hasher correctness and the BorTag key model remain unproved. The accepted
cache component selects stack-cache and excludes expensive-consistency-checks;
it is not an enabled-cache **lookup** proof. No-cache trials never count as package-
default coverage. Native-lib has no selected bodies in this component.

## Typed next needs and native boundary

`NEEDS.json` gives typed minimal needs. The next unproved sequent is:

    wf_item for each decoded borrow/cache item
    /\ (len(borrows)>32 ==> cache_consistent(cache, borrows))
    /\ genuine BorTag/hasher/iterator/Option/swap checked interfaces
    => actual find_granting for arbitrary Read/Write, provenance and exposed_tags
       preserves borrows and unknown_bottom (cache may mutate), and
       Ok(Some(g)) implies inbounds(g) /\ decoded_permission(borrows[g]).grants(access).

For Write this must yield exactly Unique or SharedReadWrite and discharge the
already checked find_first_write_incompatible precondition. This implication is
**not closed**, even conditionally, by the delivered component. No Read, Concrete,
Wildcard, known-bottom, unknown-bottom, Ok(Some), Ok(None), or Err case of full
lookup is separately advertised as proved. All remain in the frozen original.

Cache initialization for exact Stack::new is now structurally checked, but the
whole-native cache invariant remains open across insert, retain, truncate,
set_unknown_bottom/clear/refill, permission updates and other transitions. The
add lemma preserves a fully consistent cache; it does not by itself prove recovery
of invalid short-stack entries as the stack grows past 32. Item constructor/finite
bound/global initialization and actual Item::tag are cross-lane obligations; no
immutable tag-lane milestone was imported or sibling state inspected for evidence.

Native=0. Canonical arbitrary-u64/all-initialized-[u8;10] 6/0 automation and exact
HMAC arbitrary-initialized-[u8;128], all valid key lengths including 0/>128,
XOR-prefix/unchanged-suffix acceptance remain unchanged and unproved here.
HMAC remains 1/1 UNPROVED; 44 minimal + 3 phase / 101 writer effects OPEN;
publication=false; goal_complete=false. Unknown-bottom/wildcard, partial effects,
61-bit Item vs 64-bit counter, entry separation/CF, physical permissions and source
safety are not established from cache or permission success.

## Retained attempts and procedural limitations

All 8 epochs / 45 verifier invocations are retained under the exact named completed
/tmp paths in `ATTEMPTS.json`; none are deleted or promoted by omission. Final008
is the sole final accepted epoch. Earlier successes are exploratory/provisional;
annotation/type/cfg/controller setup failures are separately classified. Final007
proof outcomes were as expected but its acceptance gate rejected a wrong selected-
item count (21 instead of 20); final008 corrected and refroze the gate and reran
all 11 cases. Local acceptance does not replace the parent's independent review.
Early epochs froze helper identities but did not retain every superseded helper's
full bytes or explicitly freeze the rustc driver; they are excluded from final
acceptance. Final008 has the stronger complete freeze and authenticated helper
snapshot. Its checker is itself included in the pretrial freeze and uses explicit
exceptions, not Python assertions.

At startup, normal exec failed with bwrap loopback RTM_NEWADDR Operation not
permitted. Exact read/proof commands then used the automatically reviewed tool
permission mechanism; no sandbox/network modification occurred. An instruction-
discovery command mistakenly started `find .. -name AGENTS.md`; it was interrupted
(exit 130), returned no instruction paths, and no results were consumed. This was
a procedural deviation from the own-checkout-only discovery rule, not evidence.
No sibling proof, build, process-list or live checkout data was imported.
